//! `spotify.exe --apple-sync-probe <catalog song id>`: measures, against decoded samples,
//! whether the position Apple playback reports is the audio actually played.
//!
//! Every figure is a cross-correlation lag in frames between two decodes of the same
//! track: a decode from the start (the reference timeline) and the audio a seek, a
//! ring pull or the other encode produced. 0 is sample exact; a negative lag means
//! the audio played is from earlier in the song than the position claims, so lyrics
//! keyed to that position run early by that much.
//!
//! Needs `MUSIQUE_APPLE_MEDIA_USER_TOKEN` (or a signed-in Apple account) and the
//! Widevine CDM. Read-only; nothing is played aloud.

// reached only through main.rs's `--apple-sync-probe` dispatch
#![allow(dead_code)]

use std::time::{Duration, Instant};

use rodio::Source;

use crate::apple::playback::aac::SymSource;
use crate::apple::playback::source::AppleSource;
use crate::apple::playback::stream::MediaStream;
use crate::apple::playback::{self, AppleQuality, PreparedTrack};
use crate::errors::AppError;

/// Search window either side of the claimed position, in frames (~180ms at 44.1k).
const MAX_LAG: i64 = 8000;
/// Frames compared per measurement (~0.75s at 44.1k).
const WINDOW: usize = 32_768;
/// Frames skipped at the head of a fresh decode: a reset AAC decoder's first
/// frame is missing its overlap half and would only blur the correlation.
const SETTLE: usize = 4096;

pub fn apple_sync_probe() -> i32 {
    let mut args = std::env::args().skip_while(|a| a != "--apple-sync-probe").skip(1);
    let Some(id) = args.next().filter(|a| !a.starts_with("--")) else {
        eprintln!("usage: --apple-sync-probe <apple music catalog song id>");
        return 2;
    };
    let Ok(runtime) = tokio::runtime::Runtime::new() else { return 1 };
    match runtime.block_on(run(&id)) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("apple sync probe failed: {e}");
            1
        }
    }
}

async fn prepare(id: &str, quality: AppleQuality) -> Result<PreparedTrack, AppError> {
    let c = crate::apple::client::current().await?;
    let prepared =
        playback::prepare_with_creds(c.http(), &c.bearer(), c.user_token(), id, quality).await?;
    // seeks anywhere in the track need every fragment indexed
    let waited = Instant::now();
    while !prepared.media.done() && waited.elapsed() < Duration::from_secs(60) {
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    Ok(prepared)
}

async fn run(id: &str) -> Result<(), AppError> {
    lyrics_report(id).await;

    let lc = prepare(id, AppleQuality::High256).await?;
    let he = prepare(id, AppleQuality::Low64).await.ok();
    let duration = lc.resolved.duration;
    println!("catalog duration {:?}", duration);

    tokio::task::spawn_blocking(move || measure(lc, he, duration))
        .await
        .map_err(|e| AppError::Playback(format!("probe task: {e}")))?
}

fn measure(lc: PreparedTrack, he: Option<PreparedTrack>, duration: Option<Duration>) -> Result<(), AppError> {
    let media = lc.media.clone();
    if let Some(init) = media.init_header() {
        println!("LC init: {}", describe_init(&init));
    }

    // the reference timeline: what playing from the start produces
    let (mut from_start, started) = AppleSource::open(media.clone(), duration, Duration::ZERO)?;
    let rate = from_start.sample_rate();
    let ch = from_start.channels() as usize;
    let total_s = duration.map_or(180.0, |d| d.as_secs_f64()).min(400.0);
    let reference = mono(&mut from_start, ch, (total_s * rate as f64) as usize);
    println!(
        "LC: {rate}Hz x{ch}, started_ms {started}, reference {:.1}s decoded",
        reference.len() as f64 / rate as f64
    );

    let priming = {
        let dec = SymSource::open(media.reader(), None, false, Some("audio/mp4"))?;
        dec.priming_frames()
    };
    println!("decoder priming {priming} frames ({:.1}ms)", priming as f64 * 1000.0 / rate as f64);

    // 1. seeks the way the app makes them (AppleSource::open at a position, which is
    //    also what try_seek does), against the same splice with the priming applied
    println!("\n== seek splice: app vs priming-corrected ==");
    let span = reference.len() as f64 / rate as f64;
    let targets: Vec<f64> = [1.5, 4.0, 9.7, 23.3, 47.9, 71.2, 98.6, 133.1, 171.4, 205.9]
        .into_iter()
        .filter(|t| t + 2.0 < span)
        .collect();
    for &t in &targets {
        let at = Duration::from_secs_f64(t);
        let entry = media.splice_entry(at);
        let (mut seeked, started_ms) = AppleSource::open(media.clone(), duration, at)?;
        let app = mono(&mut seeked, ch, SETTLE + WINDOW);
        let app_lag = lag(&reference, &app, started_ms as f64 / 1000.0 * rate as f64);

        let fixed_lag = corrected_splice(&media, at, ch)
            .and_then(|fixed| lag(&reference, &fixed, t * rate as f64));
        println!(
            "seek {t:>6.1}s: reports {started_ms}ms, first fragment {:?}, into {:?} -> app lag {} | corrected lag {}",
            entry.map(|e| e.in_first_fragment),
            entry.map(|e| e.into),
            fmt_lag(app_lag, rate),
            fmt_lag(fixed_lag, rate),
        );
    }

    // 2. the playout ring: heard_ms against the audio pulled through it
    println!("\n== playout ring clock ==");
    ring_report(&media, duration, &reference, rate, ch)?;

    // 3. HE-AAC (64k) against AAC-LC (256k): the priming trim of the other encode
    if let Some(he) = he {
        println!("\n== HE-AAC 64k vs AAC-LC 256k (from start) ==");
        if let Some(init) = he.media.init_header() {
            println!("HE init: {}", describe_init(&init));
        }
        let (mut he_src, _) = AppleSource::open(he.media.clone(), duration, Duration::ZERO)?;
        let he_rate = he_src.sample_rate();
        let he_ch = he_src.channels() as usize;
        let he_priming = SymSource::open(he.media.reader(), None, false, Some("audio/mp4"))
            .map(|d| d.priming_frames())
            .unwrap_or(0);
        println!("HE: {he_rate}Hz x{he_ch}, decoder priming {he_priming} frames");
        if he_rate != rate {
            println!("HE output rate differs from LC; skipping the comparison");
        } else {
            let he_ref = mono(&mut he_src, he_ch, (span.min(60.0) * rate as f64) as usize);
            for t in [5.0, 20.0, 40.0] {
                let start = (t * rate as f64) as usize;
                if start + WINDOW + MAX_LAG as usize >= he_ref.len() {
                    continue;
                }
                let window = &he_ref[start..start + WINDOW];
                println!("HE at {t:>4.1}s: lag {}", fmt_lag(lag(&reference, window, start as f64), rate));
            }
            for t in [9.7, 47.9] {
                let at = Duration::from_secs_f64(t);
                if let Ok((mut s, started_ms)) = AppleSource::open(he.media.clone(), duration, at) {
                    let w = mono(&mut s, he_ch, SETTLE + WINDOW);
                    let fixed = corrected_splice(&he.media, at, he_ch)
                        .and_then(|f| lag(&reference, &f, t * rate as f64));
                    println!(
                        "HE seek {t:>4.1}s: app lag {} | corrected lag {}",
                        fmt_lag(lag(&reference, &w, started_ms as f64 / 1000.0 * rate as f64), rate),
                        fmt_lag(fixed, rate)
                    );
                }
            }
        }
    }
    Ok(())
}

/// Pulls a ring the way the device does and compares `heard_ms` with the audio
/// that actually came out, before and after a seek through the ring.
fn ring_report(
    media: &MediaStream,
    duration: Option<Duration>,
    reference: &[f32],
    rate: u32,
    ch: usize,
) -> Result<(), AppError> {
    use std::sync::atomic::Ordering;

    let (source, _) = AppleSource::open(media.clone(), duration, Duration::ZERO)?;
    let (mut ring_source, ring) = crate::playout::prebuffer(source, Duration::ZERO, false, None);
    let rendered_ms = |ring: &crate::playout::Ring| {
        ring.rendered_frames.load(Ordering::Relaxed) as f64 * 1000.0 / 44_100.0
    };

    // pull at roughly real time so the ring behaves as it does under a device
    let per_tick = rate as usize * ch / 100;
    let started = Instant::now();
    let mut worst = 0f64;
    let mut ticks = 0u64;
    while started.elapsed() < Duration::from_secs(6) {
        for _ in 0..per_tick {
            let _ = ring_source.next();
        }
        ticks += 1;
        let heard = ring.heard_ms() as f64;
        let played = rendered_ms(&ring);
        worst = worst.max((heard - played).abs());
        if ticks % 100 == 0 {
            println!("t={:.1}s heard {heard:.0}ms, real audio pulled {played:.1}ms, diff {:+.1}ms", started.elapsed().as_secs_f64(), heard - played);
        }
        let due = started + Duration::from_millis(ticks * 10);
        if let Some(wait) = due.checked_duration_since(Instant::now()) {
            std::thread::sleep(wait);
        }
    }
    println!("from start: worst |heard - pulled| {worst:.1}ms over 6s");

    // a seek through the ring, pulled at real time: how long until its audio sounds,
    // whether that audio is the target, and what the clock says meanwhile
    let target_s = 47.9f64.min(reference.len() as f64 / rate as f64 - 3.0);
    let target = Duration::from_secs_f64(target_s);
    ring_source.try_seek(target).map_err(|e| AppError::Playback(format!("ring seek: {e}")))?;
    let before = rendered_ms(&ring);
    let asked = Instant::now();
    let mut out: Vec<f32> = Vec::with_capacity((SETTLE + WINDOW) * 2);
    let mut first_sound: Option<Duration> = None;
    let mut ticks = 0u64;
    while out.len() < SETTLE + WINDOW && asked.elapsed() < Duration::from_secs(20) {
        for _ in 0..per_tick / ch {
            let mut sum = 0.0;
            for _ in 0..ch {
                sum += ring_source.next().unwrap_or(0.0);
            }
            // the exact zeros before the seek's audio (and its first, faded-from-zero frame)
            if out.is_empty() && sum == 0.0 {
                continue;
            }
            out.push(sum / ch as f32);
        }
        if first_sound.is_none() && rendered_ms(&ring) > before {
            first_sound = Some(asked.elapsed());
            println!("first audio of the seek after {:?}, heard_ms {}", asked.elapsed(), ring.heard_ms());
        }
        ticks += 1;
        let due = asked + Duration::from_millis(ticks * 10);
        if let Some(wait) = due.checked_duration_since(Instant::now()) {
            std::thread::sleep(wait);
        }
    }
    println!(
        "ring seek to {target_s:.1}s: content lag {} (0 is exact)",
        fmt_lag(lag(reference, &out, target_s * rate as f64), rate)
    );
    let heard = ring.heard_ms() as f64;
    let played = rendered_ms(&ring) - before;
    println!(
        "after seek: heard {heard:.0}ms vs target + real audio pulled {:.1}ms (diff {:+.1}ms); wall clock since seek {:.0}ms",
        target_s * 1000.0 + played,
        heard - (target_s * 1000.0 + played),
        asked.elapsed().as_secs_f64() * 1000.0
    );
    Ok(())
}

/// The splice a seek to `at` makes, with the decoder's priming trimmed in every
/// fragment (fragment decode times count it), as the proposed fix does.
fn corrected_splice(media: &MediaStream, at: Duration, ch: usize) -> Option<Vec<f32>> {
    let e = media.splice_entry(at)?;
    let mut dec = SymSource::open(media.reader_at(e), None, false, Some("audio/mp4")).ok()?;
    let rate = dec.sample_rate() as f64;
    let skip = (e.into.as_secs_f64() * rate).round() as usize + dec.priming_frames() as usize;
    for _ in 0..skip * ch {
        dec.next()?;
    }
    Some(mono(&mut dec, ch, SETTLE + WINDOW))
}

/// Decodes `frames` frames as a mono mix.
fn mono<I: Iterator<Item = f32>>(src: &mut I, ch: usize, frames: usize) -> Vec<f32> {
    let mut out = Vec::with_capacity(frames);
    'outer: for _ in 0..frames {
        let mut sum = 0.0;
        for _ in 0..ch {
            match src.next() {
                Some(s) => sum += s,
                None => break 'outer,
            }
        }
        out.push(sum / ch as f32);
    }
    out
}

/// Lag (frames) at which `test[SETTLE..]` best matches `reference` around `claimed`,
/// the reference frame the test's first frame is supposed to be. `test` shorter than
/// the settle window is compared from its start.
fn lag(reference: &[f32], test: &[f32], claimed: f64) -> Option<i64> {
    let skip = if test.len() > SETTLE + 1024 { SETTLE } else { 0 };
    let window = &test[skip..test.len().min(skip + WINDOW)];
    let base = claimed.round() as i64 + skip as i64;
    if window.len() < 1024 || window.iter().all(|s| s.abs() < 1e-4) {
        return None;
    }
    let energy_t: f64 = window.iter().map(|&s| (s as f64) * (s as f64)).sum();
    let mut best = (f64::MIN, 0i64);
    for l in -MAX_LAG..=MAX_LAG {
        let at = base + l;
        if at < 0 || at as usize + window.len() > reference.len() {
            continue;
        }
        let r = &reference[at as usize..at as usize + window.len()];
        let mut dot = 0f64;
        let mut er = 0f64;
        for (a, b) in window.iter().zip(r) {
            dot += (*a as f64) * (*b as f64);
            er += (*b as f64) * (*b as f64);
        }
        let score = dot / (energy_t * er).sqrt().max(1e-12);
        if score > best.0 {
            best = (score, l);
        }
    }
    (best.0 > 0.5).then_some(best.1)
}

fn fmt_lag(l: Option<i64>, rate: u32) -> String {
    match l {
        Some(l) => format!("{l:+} frames ({:+.1}ms)", l as f64 * 1000.0 / rate as f64),
        None => "no match".into(),
    }
}

/// mdhd timescale and edit list of an init segment, read directly.
fn describe_init(init: &[u8]) -> String {
    let find = |tag: &[u8; 4]| init.windows(4).position(|w| w == tag);
    let be32 = |at: usize| init.get(at..at + 4).map(|b| u32::from_be_bytes(b.try_into().unwrap()));
    let mut out = String::new();
    if let Some(p) = find(b"mdhd") {
        let body = p + 4;
        let ts_at = if init.get(body) == Some(&1) { body + 20 } else { body + 12 };
        out += &format!("mdhd timescale {:?}", be32(ts_at));
    }
    if let Some(p) = find(b"elst") {
        let body = p + 4;
        let v1 = init.get(body) == Some(&1);
        let count = be32(body + 4).unwrap_or(0);
        out += &format!(", elst v{} x{count}", if v1 { 1 } else { 0 });
        let first = body + 8;
        if v1 {
            let seg = init.get(first..first + 8).map(|b| u64::from_be_bytes(b.try_into().unwrap()));
            let media = init.get(first + 8..first + 16).map(|b| i64::from_be_bytes(b.try_into().unwrap()));
            out += &format!(" segment {seg:?} media_time {media:?}");
        } else {
            out += &format!(" segment {:?} media_time {:?}", be32(first), be32(first + 4).map(|v| v as i32));
        }
    } else {
        out += ", no elst";
    }
    out
}

/// Which lyric document an Apple song gets, and how Apple's own catalog TTML for the
/// song compares with it.
async fn lyrics_report(id: &str) {
    println!("== lyrics ==");
    let isrc = crate::apple::api::track_isrc(id).await;
    println!("catalog isrc: {isrc:?}");
    let Ok(c) = crate::apple::client::current().await else { return };
    let native = super::providers::applemusic_native::fetch_song(&c, id).await;
    match &native {
        Some(cand) => println!(
            "apple catalog TTML: {} lines, {} with words, first at {:?}ms",
            cand.lines.len(),
            cand.lines.iter().filter(|l| !l.words.is_empty()).count(),
            cand.lines.first().map(|l| l.time_ms)
        ),
        None => println!("apple catalog TTML: none"),
    }
    if let Ok(Some(isrc)) = isrc {
        let track = super::TrackRef {
            id: id.into(),
            name: String::new(),
            artist: String::new(),
            album: String::new(),
            isrc: Some(isrc),
            duration_ms: 0,
        };
        match super::providers::applemusic::fetch(&track).await {
            Some(b) => {
                println!("binimum by isrc: {} lines, exact {}", b.lines.len(), b.exact);
                if let Some(n) = &native {
                    let diffs: Vec<i64> = n
                        .lines
                        .iter()
                        .zip(&b.lines)
                        .map(|(a, b)| b.time_ms - a.time_ms)
                        .collect();
                    let same_text = n.lines.iter().zip(&b.lines).filter(|(a, b)| a.text == b.text).count();
                    println!(
                        "binimum - catalog line starts (first 12): {:?}; {same_text}/{} lines same text",
                        &diffs[..diffs.len().min(12)],
                        n.lines.len().min(b.lines.len())
                    );
                }
            }
            None => println!("binimum by isrc: none"),
        }
    }
}
