//! the rodio source an episode plays through on the youtube backend: stereo,
//! time-stretched to the podcast speed (the same WSOLA the librespot sink
//! uses, so speech keeps its pitch), and keeping its own clock.
//!
//! the clock is why this exists as a source and not a sink setting. rodio's
//! `Sink::get_pos` counts samples *played*, so at 2x it would say 30 seconds
//! have passed when a minute of the episode has - the transcript, the resume
//! point and the seek bar all want the episode's own time.

use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::Duration;

use rodio::{source::SeekError, source::UniformSourceIterator, ChannelCount, Sample, SampleRate, Source};

use crate::stretch::TimeStretch;

use super::{
    mp4::{read_index, Segments},
    remote::RemoteFile,
};

/// frames pulled from the decoder per refill (~23ms at 44.1k)
const BLOCK: usize = 1024;

pub struct EpisodeSource<S: Source> {
    inner:     UniformSourceIterator<S>,
    rate:      SampleRate,
    /// episode position in ms, read by the player
    clock:     Arc<AtomicU64>,
    base_ms:   u64,
    consumed:  u64,
    stretcher: Option<TimeStretch>,
    scratch:   Vec<Sample>,
    out:       Vec<Sample>,
    idx:       usize,
    done:      bool,
}

impl<S: Source> EpisodeSource<S> {
    pub fn new(inner: S, clock: Arc<AtomicU64>, start_ms: u64) -> Self {
        let rate = inner.sample_rate();
        clock.store(start_ms, Ordering::Relaxed);
        Self {
            inner: UniformSourceIterator::new(inner, 2, rate),
            rate,
            clock,
            base_ms: start_ms,
            consumed: 0,
            stretcher: None,
            scratch: Vec::with_capacity(BLOCK * 2),
            out: Vec::new(),
            idx: 0,
            done: false,
        }
    }

    fn refill(&mut self) {
        self.scratch.clear();
        for _ in 0..BLOCK * 2 {
            match self.inner.next() {
                Some(s) => self.scratch.push(s),
                None => {
                    self.done = true;
                    break;
                }
            }
        }
        self.consumed += (self.scratch.len() / 2) as u64;
        self.clock.store(self.base_ms + self.consumed * 1000 / self.rate.max(1) as u64, Ordering::Relaxed);

        let speed = crate::stretch::speed() as f64;
        if (speed - 1.0).abs() < 1e-3 {
            self.stretcher = None;
        } else if self.stretcher.as_ref().is_none_or(|s| (s.speed() - speed).abs() > 1e-3) {
            self.stretcher = Some(TimeStretch::new(speed));
        }
        match &mut self.stretcher {
            Some(st) => self.out = st.process(&self.scratch),
            None => std::mem::swap(&mut self.out, &mut self.scratch),
        }
        self.idx = 0;
    }
}

impl<S: Source> Iterator for EpisodeSource<S> {
    type Item = Sample;

    fn next(&mut self) -> Option<Sample> {
        loop {
            if let Some(&s) = self.out.get(self.idx) {
                self.idx += 1;
                return Some(s);
            }
            if self.done {
                return None;
            }
            // the stretcher can take a block or two before it has output
            self.refill();
        }
    }
}

impl<S: Source> Source for EpisodeSource<S> {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> ChannelCount {
        2
    }

    fn sample_rate(&self) -> SampleRate {
        self.rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }

    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        self.inner.try_seek(pos)?;
        self.out.clear();
        self.idx = 0;
        self.stretcher = None;
        self.done = false;
        self.base_ms = pos.as_millis() as u64;
        self.consumed = 0;
        self.clock.store(self.base_ms, Ordering::Relaxed);
        Ok(())
    }
}

/// the decoder under an episode. seeks the ordinary way, except in a
/// fragmented mp4 (youtube's audio), where it reopens at the fragment instead
/// of letting the demuxer walk the whole file - see `mp4`.
pub struct EpisodeDecoder {
    dec:      rodio::Decoder<RemoteFile>,
    /// kept to open spliced views from
    file:     RemoteFile,
    segments: Option<Arc<Segments>>,
    mime:     Option<String>,
}

fn build(file: RemoteFile, seekable: bool, mime: Option<&str>) -> Result<rodio::Decoder<RemoteFile>, String> {
    let len = file.len();
    let mut builder = rodio::Decoder::builder()
        .with_data(file)
        .with_byte_len(len)
        .with_seekable(seekable)
        // accurate seeking in mp3 reads from the start of the file, i.e.
        // waits for the download to get there; coarse jumps straight to it
        .with_coarse_seek(true);
    if let Some(m) = mime {
        builder = builder.with_mime_type(m);
    }
    builder.build().map_err(|e| e.to_string())
}

impl EpisodeDecoder {
    /// probe the format. blocking - it reads the start of the file
    pub fn open(file: RemoteFile, mime: Option<String>) -> Result<Self, String> {
        let segments = {
            let mut probe = file.reopen();
            let len = probe.len();
            read_index(&mut probe, len).map(Arc::new)
        };
        // with an index the demuxer doesn't need to scan (and mustn't)
        let dec = build(file.reopen(), segments.is_none(), mime.as_deref())?;
        Ok(Self { dec, file, segments, mime })
    }

    pub fn segments(&self) -> Option<Arc<Segments>> {
        self.segments.clone()
    }
}

impl Iterator for EpisodeDecoder {
    type Item = Sample;

    fn next(&mut self) -> Option<Sample> {
        self.dec.next()
    }
}

impl Source for EpisodeDecoder {
    fn current_span_len(&self) -> Option<usize> {
        self.dec.current_span_len()
    }

    fn channels(&self) -> ChannelCount {
        self.dec.channels()
    }

    fn sample_rate(&self) -> SampleRate {
        self.dec.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }

    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        let Some(segments) = &self.segments else {
            return self.dec.try_seek(pos);
        };
        let ms = pos.as_millis() as u64;
        let (byte, start) = segments.at(ms);
        self.dec = build(self.file.spliced(segments.init_end, byte), false, self.mime.as_deref())
            .map_err(|e| SeekError::Other(Box::new(std::io::Error::other(e))))?;
        // the fragment starts up to ~10s early; decode the difference away
        let skip = ms.saturating_sub(start) * self.dec.sample_rate() as u64 / 1000 * self.dec.channels() as u64;
        for _ in 0..skip {
            if self.dec.next().is_none() {
                break;
            }
        }
        Ok(())
    }
}
