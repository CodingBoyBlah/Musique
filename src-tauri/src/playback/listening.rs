// reports what we actually played to spotify's listening history
//
// connect state (spirc) only tells spotify what a device is doing *right now*.
// recently played, the "Playlist name · 5 tracks played" rows in Recents and
// play counts all come from a separate stream of events the official clients
// POST to the event service (gabo) when a track stops: one `RawCoreStream`
// per listen, carrying the track, how long it really played and the context
// (playlist / album / artist) it played from. librespot never sends those,
// which is why nothing played here ever showed up in the spotify app.
//
// only audio that actually reached the output counts. `CountingSink` sits
// around the real sink and adds up the frames it accepted, so pausing, seeking,
// loading and buffering add nothing, and the silent fallback sink (used when
// there's no usable audio device) is never wrapped. a listen is the frames
// rendered between the player starting one play request and the next one
// taking over / the track ending / a stop.
//
// field values + schema ids follow spotify desktop 1.2.96 and librespot PR
// #1759, which confirmed against a real account that events shaped like this
// land in listening history. the protobufs are hand encoded (tiny `pb` module
// below) because librespot-protocol doesn't compile the event sender protos.

use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

use flate2::{write::GzEncoder, Compression};
use librespot_core::{config, FileId, Session, SpotifyUri};
use librespot_metadata::audio::{AudioFileFormat, AudioFiles};
use librespot_playback::{
    audio_backend::{Sink, SinkResult},
    config::Bitrate,
    convert::Converter,
    decoder::AudioPacket,
    player::PlayerEvent,
    NUM_CHANNELS, SAMPLE_RATE,
};
use rand::RngCore;
use sha1::{Digest, Sha1};
use tokio::sync::{mpsc, oneshot};

// schema ids observed in spotify desktop 1.2.96.518. the receiver takes other
// values without complaint but history ingestion can quietly drop the event,
// so these stay pinned instead of tracking librespot's own version
const REPORTING_VERSION: &str = "1.2.96.518";
const REPORTING_VERSION_CODE: i64 = 129_600_518;
const REPORTING_CORE_VERSION: i64 = 6_005_400_000_002_005;
const REPORTING_SDK: &str =
    "0.9.4-rl-essopt-loginsend-onlinesend-bcdsend-heartbeat300.0s/30.0s-modern-payload125kB-batch100";

// what the frontend sends for Liked Songs. spotify's own context uri for it
// embeds the username, which only the session knows
const LIKED_SONGS: &str = "spotify:collection:tracks";

// counting sink

pub struct CountingSink {
    inner:  Box<dyn Sink>,
    frames: Arc<AtomicU64>,
}

impl CountingSink {
    pub fn new(inner: Box<dyn Sink>, frames: Arc<AtomicU64>) -> Self {
        CountingSink { inner, frames }
    }
}

impl Sink for CountingSink {
    fn start(&mut self) -> SinkResult<()> {
        self.inner.start()
    }

    fn stop(&mut self) -> SinkResult<()> {
        self.inner.stop()
    }

    fn write(&mut self, packet: AudioPacket, converter: &mut Converter) -> SinkResult<()> {
        let frames = packet.samples().map_or(0, |s| s.len() / NUM_CHANNELS as usize);
        self.inner.write(packet, converter)?;
        // only counted once the output took it, a failed write played nothing
        self.frames.fetch_add(frames as u64, Ordering::Relaxed);
        Ok(())
    }
}

// listen tracking

#[derive(Clone, Copy, PartialEq)]
enum End {
    TrackDone,
    EndPlay,
}

struct Listen {
    play_request_id: u64,
    uri:             SpotifyUri,
    context:         Option<String>,
    reason_start:    &'static str,
    frames_at_start: u64,
    started_at:      Option<SystemTime>,
    file:            Option<(FileId, AudioFileFormat)>,
}

struct Report {
    uri:          SpotifyUri,
    context:      Option<String>,
    file:         Option<(FileId, AudioFileFormat)>,
    reason_start: &'static str,
    reason_end:   &'static str,
    started_at:   SystemTime,
    ended_at:     SystemTime,
    ms_played:    u32,
}

enum Cmd {
    Report(Box<Report>),
    Flush(oneshot::Sender<()>),
}

struct State {
    current:   Option<Listen>,
    // the request that was just closed. spirc stops the player after
    // EndOfTrack, and that straggling Stopped must not open a fresh empty listen
    closed:    Option<u64>,
    // (track uri, context uri) of the last play the app asked for. a listen
    // only inherits the context when it's for that same track; anything else
    // (e.g a phone driving this device over connect) goes in with no context
    requested: Option<(String, Option<String>)>,
    // audio files of the last TrackChanged. a preloaded or repeated track skips
    // the Loading event, so TrackChanged can arrive before its listen opens
    last_files: Option<(SpotifyUri, AudioFiles)>,
    last_track_done: Option<Instant>,
    tx:        Option<mpsc::UnboundedSender<Cmd>>,
}

pub struct ListenTracker {
    state:   Mutex<State>,
    frames:  Arc<AtomicU64>,
    formats: [AudioFileFormat; 7],
}

impl ListenTracker {
    /// `frames` must be the counter the player's `CountingSink` writes to.
    pub fn new(session: Session, bitrate: Bitrate, frames: Arc<AtomicU64>) -> Arc<Self> {
        let (tx, rx) = mpsc::unbounded_channel();
        tauri::async_runtime::spawn(run_reporter(session, rx));
        Arc::new(Self::with_channel(tx, bitrate, frames))
    }

    fn with_channel(tx: mpsc::UnboundedSender<Cmd>, bitrate: Bitrate, frames: Arc<AtomicU64>) -> Self {
        ListenTracker {
            state: Mutex::new(State {
                current:         None,
                closed:          None,
                requested:       None,
                last_files:      None,
                last_track_done: None,
                tx:              Some(tx),
            }),
            frames,
            formats: format_order(bitrate),
        }
    }

    /// Remember which context the app is about to play `track_uri` from.
    /// Anything that isn't a spotify uri (the app's own pseudo contexts like
    /// "search" or "radio") is dropped, and the listen is reported as a plain
    /// track play.
    pub fn set_requested(&self, track_uri: &str, context: Option<String>) {
        let context = context.filter(|c| c.starts_with("spotify:") && c.len() > "spotify:".len());
        self.state.lock().unwrap().requested = Some((track_uri.to_string(), context));
    }

    pub fn on_event(&self, event: &PlayerEvent) {
        match event {
            PlayerEvent::Loading { play_request_id, track_id, .. } =>
                self.open(*play_request_id, track_id, false),
            PlayerEvent::Playing { play_request_id, track_id, .. } =>
                self.open(*play_request_id, track_id, true),
            PlayerEvent::TrackChanged { audio_item } => {
                let mut s = self.state.lock().unwrap();
                let file = pick_file(&self.formats, &audio_item.files);
                if let Some(cur) = s.current.as_mut() {
                    if cur.uri == audio_item.track_id && cur.file.is_none() {
                        cur.file = file;
                    }
                }
                s.last_files = Some((audio_item.track_id.clone(), audio_item.files.clone()));
            }
            PlayerEvent::EndOfTrack { play_request_id, .. } =>
                self.close_if(*play_request_id, Some(End::TrackDone)),
            PlayerEvent::Stopped { play_request_id, .. } =>
                self.close_if(*play_request_id, Some(End::EndPlay)),
            // nothing played, nothing to report
            PlayerEvent::Unavailable { play_request_id, .. } =>
                self.close_if(*play_request_id, None),
            _ => {}
        }
    }

    /// End whatever is playing as a listen and stop reporting. Used when the
    /// session is torn down or replaced.
    pub fn close(&self) {
        let mut s = self.state.lock().unwrap();
        self.finish(&mut s, End::EndPlay);
        s.tx = None;
    }

    /// `close`, then wait (up to `timeout`) for queued reports to go out, so a
    /// sign-out or quit doesn't lose the track that was playing.
    pub async fn close_and_flush(&self, timeout: Duration) {
        let done = {
            let mut s = self.state.lock().unwrap();
            self.finish(&mut s, End::EndPlay);
            let (done_tx, done_rx) = oneshot::channel();
            let sent = s.tx.take().is_some_and(|tx| tx.send(Cmd::Flush(done_tx)).is_ok());
            sent.then_some(done_rx)
        };
        if let Some(done) = done {
            let _ = tokio::time::timeout(timeout, done).await;
        }
    }

    fn open(&self, play_request_id: u64, track_id: &SpotifyUri, playing: bool) {
        let now = SystemTime::now();
        let mut s = self.state.lock().unwrap();
        if s.closed == Some(play_request_id) {
            return;
        }
        if let Some(cur) = s.current.as_mut() {
            if cur.play_request_id == play_request_id {
                if playing && cur.started_at.is_none() {
                    cur.started_at = Some(now);
                }
                return;
            }
        }
        // a new request took over, so whatever was playing ended here
        self.finish(&mut s, End::EndPlay);
        // only spotify audio goes to spotify's history, never local files
        if !matches!(track_id, SpotifyUri::Track { .. } | SpotifyUri::Episode { .. }) {
            return;
        }

        let uri_str = track_id.to_uri().ok();
        let context = match &s.requested {
            Some((uri, ctx)) if uri_str.as_deref() == Some(uri.as_str()) => ctx.clone(),
            _ => None,
        };
        let reason_start = if s.last_track_done.is_some_and(|t| t.elapsed() < Duration::from_secs(30)) {
            "trackdone"
        } else {
            "playbtn"
        };
        let file = match &s.last_files {
            Some((uri, files)) if uri == track_id => pick_file(&self.formats, files),
            _ => None,
        };
        s.current = Some(Listen {
            play_request_id,
            uri: track_id.clone(),
            context,
            reason_start,
            frames_at_start: self.frames.load(Ordering::Relaxed),
            started_at: playing.then_some(now),
            file,
        });
    }

    fn close_if(&self, play_request_id: u64, end: Option<End>) {
        let mut s = self.state.lock().unwrap();
        if s.current.as_ref().is_none_or(|c| c.play_request_id != play_request_id) {
            return;
        }
        match end {
            Some(end) => self.finish(&mut s, end),
            None => {
                s.current = None;
                s.closed = Some(play_request_id);
            }
        }
    }

    fn finish(&self, s: &mut State, end: End) {
        let Some(listen) = s.current.take() else { return };
        s.closed = Some(listen.play_request_id);
        s.last_track_done = (end == End::TrackDone).then(Instant::now);

        let frames = self.frames.load(Ordering::Relaxed).saturating_sub(listen.frames_at_start);
        let ms_played = (frames * 1000 / SAMPLE_RATE as u64).min(i32::MAX as u64) as u32;
        if ms_played == 0 {
            return;
        }
        let ended_at = SystemTime::now();
        let report = Report {
            uri: listen.uri,
            context: listen.context,
            file: listen.file,
            reason_start: listen.reason_start,
            reason_end: match end {
                End::TrackDone => "trackdone",
                End::EndPlay => "endplay",
            },
            started_at: listen
                .started_at
                .unwrap_or_else(|| ended_at - Duration::from_millis(ms_played as u64)),
            ended_at,
            ms_played,
        };
        if let Some(tx) = &s.tx {
            let _ = tx.send(Cmd::Report(Box::new(report)));
        }
    }
}

// same preference order the player uses to pick a file for each bitrate, so
// the reported file is the one that actually streamed
fn format_order(bitrate: Bitrate) -> [AudioFileFormat; 7] {
    use AudioFileFormat::*;
    match bitrate {
        Bitrate::Bitrate96 => [OGG_VORBIS_96, MP3_96, OGG_VORBIS_160, MP3_160, MP3_256, OGG_VORBIS_320, MP3_320],
        Bitrate::Bitrate160 => [OGG_VORBIS_160, MP3_160, OGG_VORBIS_96, MP3_96, MP3_256, OGG_VORBIS_320, MP3_320],
        Bitrate::Bitrate320 => [OGG_VORBIS_320, MP3_320, MP3_256, OGG_VORBIS_160, MP3_160, OGG_VORBIS_96, MP3_96],
    }
}

fn pick_file(order: &[AudioFileFormat], files: &AudioFiles) -> Option<(FileId, AudioFileFormat)> {
    order.iter().find_map(|f| files.get(f).map(|id| (*id, *f)))
}

fn format_label(format: AudioFileFormat) -> &'static str {
    match format {
        AudioFileFormat::OGG_VORBIS_96 => "Vorbis 96 kbps",
        AudioFileFormat::OGG_VORBIS_160 => "Vorbis 160 kbps",
        AudioFileFormat::MP3_96 => "MP3 96 kbps",
        AudioFileFormat::MP3_160 => "MP3 160 kbps",
        AudioFileFormat::MP3_256 => "MP3 256 kbps",
        AudioFileFormat::MP3_320 => "MP3 320 kbps",
        _ => "Vorbis 320 kbps",
    }
}

// reporter

async fn run_reporter(session: Session, mut rx: mpsc::UnboundedReceiver<Cmd>) {
    let mut sender = EventSender::new();
    while let Some(cmd) = rx.recv().await {
        match cmd {
            Cmd::Report(report) => {
                let sent = tokio::time::timeout(Duration::from_secs(60), sender.send(&session, &report)).await;
                match sent {
                    Ok(Ok(())) => eprintln!(
                        "[listening] reported {} ({} ms) from {}",
                        report.uri,
                        report.ms_played,
                        report.context.as_deref().unwrap_or("no context"),
                    ),
                    Ok(Err(e)) => eprintln!("[listening] report for {} failed: {e}", report.uri),
                    Err(_) => eprintln!("[listening] report for {} timed out", report.uri),
                }
            }
            Cmd::Flush(done) => {
                let _ = done.send(());
            }
        }
    }
}

struct EventSender {
    app_session_id: [u8; 16],
    sequence_id:    [u8; 20],
    sequence:       i64,
    clock:          Instant,
}

impl EventSender {
    fn new() -> Self {
        let mut app_session_id = [0u8; 16];
        let mut sequence_id = [0u8; 20];
        rand::thread_rng().fill_bytes(&mut app_session_id);
        rand::thread_rng().fill_bytes(&mut sequence_id);
        EventSender { app_session_id, sequence_id, sequence: 0, clock: Instant::now() }
    }

    async fn send(&mut self, session: &Session, report: &Report) -> Result<(), String> {
        // the track's audio identity. history ingestion expects it; if the
        // lookup fails the event still goes out and spotify decides
        let audio_id = match session.spclient().get_audio_files_metadata(&report.uri).await {
            Ok(bytes) => pb::find_bytes(&bytes, 4).map(<[u8]>::to_vec),
            Err(e) => {
                eprintln!("[listening] audio files lookup failed for {}: {e}", report.uri);
                None
            }
        };
        let content_uri = report.uri.to_uri().map_err(|e| e.to_string())?;
        let play_context = match report.context.as_deref() {
            Some(LIKED_SONGS) => format!("spotify:user:{}:collection", session.username()),
            Some(ctx) => ctx.to_string(),
            None => content_uri.clone(),
        };

        let message = raw_core_stream(report, &content_uri, &play_context, audio_id.as_deref(), &self.app_session_id);
        self.sequence += 1;
        let body = gzip(&self.envelope(session, report, message))?;

        // a transient rejection is retried with the SAME bytes (same sequence
        // number), so a retry can never turn into a second listen
        for attempt in 0..3u32 {
            let response = session
                .spclient()
                .publish_events(&body)
                .await
                .map_err(|e| e.to_string())?;
            match rejection(&response) {
                None => return Ok(()),
                Some((true, reason)) if attempt < 2 => {
                    eprintln!("[listening] transient rejection (reason {reason}), retrying");
                    tokio::time::sleep(Duration::from_secs(1 << attempt)).await;
                }
                Some((_, reason)) => return Err(format!("event rejected, reason {reason}")),
            }
        }
        unreachable!()
    }

    fn envelope(&self, session: &Session, report: &Report, message: Vec<u8>) -> Vec<u8> {
        use pb::Msg;
        let fragment = |name: &str, data: Vec<u8>| Msg::new().str(1, name).bytes(2, &data);
        let client_id = hex_decode(&session.client_id()).unwrap_or_default();
        let installation_id = &Sha1::digest(session.device_id().as_bytes())[..16];

        let envelope = Msg::new()
            .str(2, "RawCoreStream")
            .msg(3, fragment("message", message))
            .msg(3, fragment("context_client_id", Msg::new().bytes(1, &client_id).finish()))
            .msg(3, fragment("context_installation_id", Msg::new().bytes(1, installation_id).finish()))
            .msg(3, fragment(
                "context_application_desktop",
                Msg::new()
                    .str(1, REPORTING_VERSION)
                    .int(2, REPORTING_VERSION_CODE)
                    .bytes(3, &self.app_session_id)
                    .finish(),
            ))
            .msg(3, fragment(
                "context_device_desktop",
                Msg::new()
                    .str(1, config::OS)
                    .str(2, "librespot")
                    .str(3, "librespot")
                    .str(4, session.device_id())
                    .str(5, &config::os_version())
                    .finish(),
            ))
            .msg(3, fragment("context_time", Msg::new().int(1, millis(report.ended_at)).finish()))
            .msg(3, fragment(
                "context_monotonic_clock",
                Msg::new().int(1, 1).int(2, self.clock.elapsed().as_millis() as i64).finish(),
            ))
            .msg(3, fragment(
                "context_sdk",
                Msg::new().str(1, REPORTING_SDK).str(2, "cpp").finish(),
            ))
            .msg(3, Msg::new().str(1, "context_client_context_id"))
            .bytes(4, &self.sequence_id)
            .int(5, self.sequence);

        // PublishEventsRequest { repeated EventEnvelope event = 1 }
        Msg::new().msg(1, envelope).finish()
    }
}

// RawCoreStream (proto2, field numbers from desktop 1.2.96.518)
fn raw_core_stream(
    r: &Report,
    content_uri: &str,
    play_context: &str,
    audio_id: Option<&[u8]>,
    app_session_id: &[u8],
) -> Vec<u8> {
    let played = r.ms_played as i64;
    let mut m = pb::Msg::new()
        .bytes(1, &random_id())
        .bytes(2, &[0u8; 16])
        .str(3, "");
    if let Some((file_id, _)) = r.file {
        m = m.bytes(4, &file_id.0);
    }
    m = m
        .str(5, "audio")
        .str(9, "librespot")
        .str(10, r.reason_start)
        .str(11, "librespot")
        .str(12, r.reason_end)
        .int(13, millis(r.started_at))
        .int(14, played)
        .int(15, played)
        .int(16, 0)
        .int(17, 0)
        .int(19, 0)
        .bool(20, false)
        .str(22, format_label(r.file.map_or(AudioFileFormat::OGG_VORBIS_320, |f| f.1)))
        .str(23, play_context)
        .str(24, content_uri)
        .str(25, "")
        .bool(26, false)
        .bool(27, false)
        .str(28, "context")
        .str(29, "unknown")
        .str(32, "none")
        .int(37, REPORTING_CORE_VERSION)
        .str(38, "full")
        .bool(39, true)
        .int(40, 0)
        .bool(42, false)
        .bool(43, false)
        .str(44, "local")
        .str(45, "boombox")
        .int(46, 0)
        .int(50, 0)
        .int(51, 0)
        .bytes(53, &random_id())
        .int(56, millis(r.ended_at))
        .str(59, &hex_encode(app_session_id));
    if let Some(id) = audio_id.filter(|id| !id.is_empty()) {
        m = m.bytes(63, hex_encode(id).as_bytes());
    }
    m.int(67, 0)
        .str(69, "boombox")
        .str(71, "context-player")
        .str(78, "librespot")
        .str(79, "librespot")
        .str(80, "computer")
        .finish()
}

// PublishEventsResponse { repeated EventError error = 1 { index = 1; transient = 2; reason = 3 } }
// -> (transient, reason) of the first error, None when the event was accepted
fn rejection(response: &[u8]) -> Option<(bool, u64)> {
    let error = pb::find_bytes(response, 1)?;
    let fields = pb::fields(error).unwrap_or_default();
    let varint = |n| fields.iter().find_map(|(f, v)| match v {
        pb::Value::Varint(x) if *f == n => Some(*x),
        _ => None,
    });
    Some((varint(2).unwrap_or(0) != 0, varint(3).unwrap_or(0)))
}

fn gzip(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data).map_err(|e| e.to_string())?;
    encoder.finish().map_err(|e| e.to_string())
}

fn millis(t: SystemTime) -> i64 {
    t.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_millis() as i64
}

// random uuid v4 bytes
fn random_id() -> [u8; 16] {
    let mut id = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut id);
    id[6] = (id[6] & 0x0f) | 0x40;
    id[8] = (id[8] & 0x3f) | 0x80;
    id
}

fn hex_encode(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(DIGITS[(b >> 4) as usize] as char);
        s.push(DIGITS[(b & 0x0f) as usize] as char);
    }
    s
}

fn hex_decode(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(s.get(i..i + 2)?, 16).ok())
        .collect()
}

// just enough protobuf to write the event messages and read two fields back
mod pb {
    #[derive(Default)]
    pub struct Msg(Vec<u8>);

    impl Msg {
        pub fn new() -> Self {
            Msg::default()
        }

        fn varint(&mut self, mut v: u64) {
            while v >= 0x80 {
                self.0.push(v as u8 | 0x80);
                v >>= 7;
            }
            self.0.push(v as u8);
        }

        fn key(&mut self, field: u32, wire_type: u8) {
            self.varint(((field as u64) << 3) | wire_type as u64);
        }

        pub fn int(mut self, field: u32, v: i64) -> Self {
            self.key(field, 0);
            self.varint(v as u64);
            self
        }

        pub fn bool(self, field: u32, v: bool) -> Self {
            self.int(field, v as i64)
        }

        pub fn bytes(mut self, field: u32, v: &[u8]) -> Self {
            self.key(field, 2);
            self.varint(v.len() as u64);
            self.0.extend_from_slice(v);
            self
        }

        pub fn str(self, field: u32, v: &str) -> Self {
            self.bytes(field, v.as_bytes())
        }

        pub fn msg(self, field: u32, m: Msg) -> Self {
            self.bytes(field, &m.0)
        }

        pub fn finish(self) -> Vec<u8> {
            self.0
        }
    }

    pub enum Value<'a> {
        Varint(u64),
        Bytes(&'a [u8]),
        Fixed,
    }

    fn read_varint(buf: &[u8], pos: &mut usize) -> Option<u64> {
        let mut v = 0u64;
        for shift in (0..64).step_by(7) {
            let b = *buf.get(*pos)?;
            *pos += 1;
            v |= ((b & 0x7f) as u64) << shift;
            if b < 0x80 {
                return Some(v);
            }
        }
        None
    }

    /// Top-level fields of a message, or None when it's malformed.
    pub fn fields(buf: &[u8]) -> Option<Vec<(u32, Value<'_>)>> {
        let mut out = Vec::new();
        let mut pos = 0;
        while pos < buf.len() {
            let key = read_varint(buf, &mut pos)?;
            let field = (key >> 3) as u32;
            let value = match key & 7 {
                0 => Value::Varint(read_varint(buf, &mut pos)?),
                1 | 5 => {
                    pos += if key & 7 == 1 { 8 } else { 4 };
                    Value::Fixed
                }
                2 => {
                    let len = read_varint(buf, &mut pos)? as usize;
                    let end = pos.checked_add(len).filter(|&e| e <= buf.len())?;
                    let bytes = &buf[pos..end];
                    pos = end;
                    Value::Bytes(bytes)
                }
                _ => return None,
            };
            out.push((field, value));
        }
        (pos == buf.len()).then_some(out)
    }

    pub fn find_bytes(buf: &[u8], field: u32) -> Option<&[u8]> {
        fields(buf)?.into_iter().find_map(|(f, v)| match v {
            Value::Bytes(b) if f == field => Some(b),
            _ => None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protobuf_round_trip() {
        let m = pb::Msg::new().int(1, 300).str(2, "hi").bytes(4, &[1, 2, 3]).finish();
        // 300 = 0xac 0x02
        assert_eq!(&m[..3], &[0x08, 0xac, 0x02]);
        assert_eq!(pb::find_bytes(&m, 2), Some(&b"hi"[..]));
        assert_eq!(pb::find_bytes(&m, 4), Some(&[1u8, 2, 3][..]));
        assert!(pb::fields(&[0x0a, 0x05, 0x00]).is_none(), "truncated length must not parse");
    }

    #[test]
    fn reads_event_rejections() {
        assert_eq!(rejection(&[]), None);
        // error { index 0, reason 2 }
        assert_eq!(rejection(&[0x0a, 0x02, 0x18, 0x02]), Some((false, 2)));
        // error { transient true, reason 2 }
        assert_eq!(rejection(&[0x0a, 0x04, 0x10, 0x01, 0x18, 0x02]), Some((true, 2)));
    }

    const A: &str = "spotify:track:4PTG3Z6ehGkBFwjybzWkR8";
    const B: &str = "spotify:track:72AZ3V52rs9NfgMNhALxln";

    fn tracker() -> (ListenTracker, mpsc::UnboundedReceiver<Cmd>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (ListenTracker::with_channel(tx, Bitrate::Bitrate320, Arc::new(AtomicU64::new(0))), rx)
    }

    fn uri(s: &str) -> SpotifyUri {
        SpotifyUri::from_uri(s).unwrap()
    }

    fn play(t: &ListenTracker, id: u64, track: &str) {
        t.on_event(&PlayerEvent::Loading { play_request_id: id, track_id: uri(track), position_ms: 0 });
        t.on_event(&PlayerEvent::Playing { play_request_id: id, track_id: uri(track), position_ms: 0 });
    }

    fn render(t: &ListenTracker, ms: u64) {
        t.frames.fetch_add(ms * SAMPLE_RATE as u64 / 1000, Ordering::Relaxed);
    }

    fn next_report(rx: &mut mpsc::UnboundedReceiver<Cmd>) -> Option<Report> {
        match rx.try_recv() {
            Ok(Cmd::Report(r)) => Some(*r),
            _ => None,
        }
    }

    #[test]
    fn a_finished_track_reports_rendered_time_and_its_context() {
        let (t, mut rx) = tracker();
        t.set_requested(A, Some("spotify:playlist:37i9dQZF1DXcBWIGoYBM5M".into()));
        play(&t, 1, A);
        render(&t, 30_000);
        t.on_event(&PlayerEvent::EndOfTrack { play_request_id: 1, track_id: uri(A) });
        // spirc stops the player right after; that must not become a second listen
        t.on_event(&PlayerEvent::Stopped { play_request_id: 1, track_id: uri(A) });

        let r = next_report(&mut rx).expect("listen reported");
        assert_eq!(r.uri, uri(A));
        assert_eq!(r.ms_played, 30_000);
        assert_eq!(r.context.as_deref(), Some("spotify:playlist:37i9dQZF1DXcBWIGoYBM5M"));
        assert_eq!((r.reason_start, r.reason_end), ("playbtn", "trackdone"));
        assert!(next_report(&mut rx).is_none());

        // the next track follows on from a finished one
        t.set_requested(B, Some("spotify:playlist:37i9dQZF1DXcBWIGoYBM5M".into()));
        play(&t, 2, B);
        render(&t, 1_000);
        t.close();
        let r = next_report(&mut rx).expect("partial listen reported on close");
        assert_eq!((r.reason_start, r.reason_end, r.ms_played), ("trackdone", "endplay", 1_000));
    }

    #[test]
    fn switching_tracks_closes_the_previous_listen() {
        let (t, mut rx) = tracker();
        t.set_requested(A, Some("spotify:album:1".into()));
        play(&t, 1, A);
        render(&t, 5_000);
        // pausing and seeking render nothing, so they add nothing
        t.on_event(&PlayerEvent::Paused { play_request_id: 1, track_id: uri(A), position_ms: 5_000 });
        t.on_event(&PlayerEvent::Seeked { play_request_id: 1, track_id: uri(A), position_ms: 90_000 });
        t.on_event(&PlayerEvent::Playing { play_request_id: 1, track_id: uri(A), position_ms: 90_000 });
        render(&t, 2_000);
        // B was never requested with a context (e.g. started from a phone)
        play(&t, 2, B);

        let r = next_report(&mut rx).unwrap();
        assert_eq!((r.uri, r.ms_played, r.reason_end), (uri(A), 7_000, "endplay"));
        render(&t, 3_000);
        t.close();
        let r = next_report(&mut rx).unwrap();
        assert_eq!((r.uri, r.context), (uri(B), None));
    }

    #[test]
    fn unplayed_and_unavailable_tracks_are_not_reported() {
        let (t, mut rx) = tracker();
        play(&t, 1, A);
        t.on_event(&PlayerEvent::Unavailable { play_request_id: 1, track_id: uri(A) });
        play(&t, 2, B);
        t.on_event(&PlayerEvent::Stopped { play_request_id: 2, track_id: uri(B) });
        t.close();
        assert!(next_report(&mut rx).is_none());
    }

    #[test]
    fn app_pseudo_contexts_are_not_sent_to_spotify() {
        let (t, mut rx) = tracker();
        t.set_requested(A, Some("search".into()));
        play(&t, 1, A);
        render(&t, 1_000);
        t.close();
        assert_eq!(next_report(&mut rx).unwrap().context, None);
    }

    #[test]
    fn hex_helpers() {
        assert_eq!(hex_encode(&[0x0a, 0xff]), "0aff");
        assert_eq!(hex_decode("65b708").unwrap(), vec![0x65, 0xb7, 0x08]);
        assert!(hex_decode("abc").is_none());
        let bytes: Vec<u8> = (0..=255).collect();
        let expected: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(hex_encode(&bytes), expected);
        assert_eq!(hex_decode(&expected), Some(bytes));
    }
}
