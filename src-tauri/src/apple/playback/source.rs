//! `rodio::Source` adapter for Apple Music streams: unseekable fMP4 opening, splice-based
//! seeking, AAC/HE-AAC priming-frame trimming, and catalog-duration tail trimming.

use std::time::Duration;

use rodio::source::SeekError;
use rodio::{ChannelCount, Sample, SampleRate, Source};

use crate::errors::AppError;

use super::aac::SymSource;
use super::stream::MediaStream;

fn priming_duration(priming_frames: u32, sample_rate: u32) -> Duration {
    Duration::from_secs_f64(f64::from(priming_frames) / f64::from(sample_rate.max(1)))
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct EditTrim {
    skip: Duration,
    take: Option<Duration>,
}

/// Wraps a `Source` to drop `skip` frames from the front and stop after `take` duration.
struct Trimmed<S> {
    inner:        S,
    skip_samples: u64,
    take_samples: Option<u64>,
    duration:     Option<Duration>,
}

impl<S: Source> Trimmed<S> {
    fn new(inner: S, skip: Duration, take: Option<Duration>) -> Self {
        let rate = inner.sample_rate().max(1) as f64;
        let channels = inner.channels().max(1) as u64;
        let skip_samples = (skip.as_secs_f64() * rate).round() as u64 * channels;
        let take_samples = take.map(|d| (d.as_secs_f64() * rate).round() as u64 * channels);
        Self {
            inner,
            skip_samples,
            take_samples,
            duration: take,
        }
    }

    fn reset_limits(&mut self, skip: Duration, take: Option<Duration>) {
        let rate = self.inner.sample_rate().max(1) as f64;
        let channels = self.inner.channels().max(1) as u64;
        self.skip_samples = (skip.as_secs_f64() * rate).round() as u64 * channels;
        self.take_samples = take.map(|d| (d.as_secs_f64() * rate).round() as u64 * channels);
        self.duration = take;
    }
}

impl<S: Iterator<Item = Sample>> Iterator for Trimmed<S> {
    type Item = Sample;

    fn next(&mut self) -> Option<Self::Item> {
        while self.skip_samples > 0 {
            self.inner.next()?;
            self.skip_samples -= 1;
        }
        if let Some(remaining) = &mut self.take_samples {
            if *remaining == 0 {
                return None;
            }
            *remaining -= 1;
        }
        self.inner.next()
    }
}

impl<S: Source> Source for Trimmed<S> {
    fn current_span_len(&self) -> Option<usize> {
        self.inner.current_span_len()
    }

    fn channels(&self) -> ChannelCount {
        self.inner.channels()
    }

    fn sample_rate(&self) -> SampleRate {
        self.inner.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.duration.or_else(|| self.inner.total_duration())
    }

    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        self.inner.try_seek(pos)
    }
}

/// Decoded audio source for an Apple Music track (CENC fMP4 or plain upload).
pub struct AppleSource {
    media:    MediaStream,
    duration: Option<Duration>,
    decoder:  Trimmed<SymSource>,
}

impl AppleSource {
    /// Opens a decoder over `media` starting at `at`, returning `(source, actual_start_ms)`.
    ///
    /// Blocking: reads the MP4 header (and seeks/skips to `at`). Call via `spawn_blocking`.
    pub fn open(
        media: MediaStream,
        duration: Option<Duration>,
        at: Duration,
    ) -> Result<(Self, u32), AppError> {
        if media.is_plain() {
            let mut dec = SymSource::open(media.reader(), media.total_bytes(), true, None)?;
            let mut started_ms = 0u32;
            if !at.is_zero() {
                match dec.try_seek(at) {
                    Ok(()) => started_ms = at.as_millis().min(u32::MAX as u128) as u32,
                    Err(e) => {
                        log::warn!("apple: initial seek to {}ms failed: {e}", at.as_millis());
                    }
                }
            }
            let take = duration.map(|d| d.saturating_sub(Duration::from_millis(started_ms as u64)));
            return Ok((
                Self {
                    media,
                    duration,
                    decoder: Trimmed::new(dec, Duration::ZERO, take),
                },
                started_ms,
            ));
        }

        let (decoder, started_ms) = open_cenc_trimmed(&media, duration, at, false)?;
        Ok((
            Self {
                media,
                duration,
                decoder,
            },
            started_ms,
        ))
    }
}

/// Opens a CENC fMP4 stream unseekably without byte length so Symphonia skips full-track
/// sample scanning, splicing `[0..moov_end] + [target_fragment..]` when `at > 0`.
fn open_cenc_trimmed(
    media: &MediaStream,
    duration: Option<Duration>,
    at: Duration,
    strict_seek: bool,
) -> Result<(Trimmed<SymSource>, u32), AppError> {
    let entry = if !at.is_zero() {
        let found = media.splice_entry(at);
        if found.is_none() && strict_seek {
            return Err(AppError::Playback(format!(
                "cannot seek apple stream to {}ms (fragment unavailable)",
                at.as_millis()
            )));
        }
        found
    } else {
        None
    };

    let reader = match entry {
        Some(e) => media.reader_at(e),
        None => media.reader(),
    };

    let dec = SymSource::open(reader, None, false, Some("audio/mp4"))?;
    let default_priming = priming_duration(dec.priming_frames(), dec.sample_rate());

    match entry {
        Some(e) => {
            let head_priming = if e.in_first_fragment {
                default_priming
            } else {
                Duration::ZERO
            };
            let skip = e.into + head_priming;
            let take = duration.map(|d| d.saturating_sub(at));
            let started_ms = at.as_millis().min(u32::MAX as u128) as u32;
            Ok((Trimmed::new(dec, skip, take), started_ms))
        }
        None => {
            let edit = media.init_header().as_deref().and_then(parse_mp4_edit_list);
            let skip = edit.map(|e| e.skip).unwrap_or(default_priming);
            let take = edit.and_then(|e| e.take).or(duration);
            Ok((Trimmed::new(dec, skip, take), 0))
        }
    }
}

impl Iterator for AppleSource {
    type Item = Sample;

    fn next(&mut self) -> Option<Self::Item> {
        self.decoder.next()
    }
}

impl Source for AppleSource {
    fn current_span_len(&self) -> Option<usize> {
        self.decoder.current_span_len()
    }

    fn channels(&self) -> ChannelCount {
        self.decoder.channels()
    }

    fn sample_rate(&self) -> SampleRate {
        self.decoder.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.decoder.total_duration()
    }

    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        if self.media.is_plain() {
            self.decoder.inner.try_seek(pos)?;
            let take = self.duration.map(|d| d.saturating_sub(pos));
            self.decoder.reset_limits(Duration::ZERO, take);
            return Ok(());
        }

        let started = std::time::Instant::now();
        let (new_decoder, _) = open_cenc_trimmed(&self.media, self.duration, pos, true)
            .map_err(|e| SeekError::Other(Box::new(std::io::Error::other(e.to_string()))))?;
        self.decoder = new_decoder;
        eprintln!("[apple] decoder reopened at {}ms in {}ms", pos.as_millis(), started.elapsed().as_millis());
        Ok(())
    }
}

fn parse_mp4_edit_list(data: &[u8]) -> Option<EditTrim> {
    let moov = mp4_child(data, b"moov")?;
    let mvhd_scale = mp4_timescale(mp4_child(moov, b"mvhd")?)?;
    let trak = mp4_child(moov, b"trak")?;
    let mdhd_scale = mp4_timescale(mp4_child(mp4_child(trak, b"mdia")?, b"mdhd")?)?;
    let elst = mp4_child(mp4_child(trak, b"edts")?, b"elst")?;

    let version = *elst.first()?;
    let count = u32::from_be_bytes(elst.get(4..8)?.try_into().ok()?);
    let width = if version == 1 { 20usize } else { 12usize };
    let mut at = 8usize;

    for _ in 0..count {
        let entry = elst.get(at..at + width)?;
        let (segment, media) = if version == 1 {
            (
                u64::from_be_bytes(entry.get(0..8)?.try_into().ok()?),
                i64::from_be_bytes(entry.get(8..16)?.try_into().ok()?),
            )
        } else {
            (
                u64::from(u32::from_be_bytes(entry.get(0..4)?.try_into().ok()?)),
                i64::from(i32::from_be_bytes(entry.get(4..8)?.try_into().ok()?)),
            )
        };
        if media >= 0 {
            return Some(EditTrim {
                skip: Duration::from_secs_f64(media as f64 / mdhd_scale as f64),
                take: (segment > 0)
                    .then(|| Duration::from_secs_f64(segment as f64 / mvhd_scale as f64)),
            });
        }
        at += width;
    }
    None
}

fn mp4_child<'a>(data: &'a [u8], kind: &[u8; 4]) -> Option<&'a [u8]> {
    let mut at = 0usize;
    while at + 8 <= data.len() {
        let raw_size = u32::from_be_bytes(data.get(at..at + 4)?.try_into().ok()?) as u64;
        let name = data.get(at + 4..at + 8)?;
        let (hdr, size) = match raw_size {
            1 => (16usize, u64::from_be_bytes(data.get(at + 8..at + 16)?.try_into().ok()?)),
            0 => (8usize, (data.len() - at) as u64),
            n => (8usize, n),
        };
        let size = usize::try_from(size).ok()?;
        if size < hdr || at + size > data.len() {
            return None;
        }
        if name == kind {
            return Some(&data[at + hdr..at + size]);
        }
        at += size;
    }
    None
}

fn mp4_timescale(header: &[u8]) -> Option<u64> {
    let off = if *header.first()? == 1 { 20 } else { 12 };
    let scale = u32::from_be_bytes(header.get(off..off + 4)?.try_into().ok()?);
    (scale > 0).then_some(u64::from(scale))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trimmed_skips_head_and_cuts_tail() {
        struct Ramp {
            next: u32,
            max:  u32,
        }
        impl Iterator for Ramp {
            type Item = f32;
            fn next(&mut self) -> Option<f32> {
                (self.next < self.max).then(|| {
                    self.next += 1;
                    self.next as f32
                })
            }
        }
        impl Source for Ramp {
            fn current_span_len(&self) -> Option<usize> { None }
            fn channels(&self) -> ChannelCount { 2 }
            fn sample_rate(&self) -> SampleRate { 1000 }
            fn total_duration(&self) -> Option<Duration> { None }
            fn try_seek(&mut self, _: Duration) -> Result<(), SeekError> { Ok(()) }
        }

        let trimmed = Trimmed::new(
            Ramp { next: 0, max: 200 },
            Duration::from_millis(5),
            Some(Duration::from_millis(10)),
        );
        let out: Vec<f32> = trimmed.collect();
        // 5ms * 1000Hz * 2ch = 10 samples skipped; 10ms * 1000Hz * 2ch = 20 samples taken
        assert_eq!(out.len(), 20);
        assert_eq!(out.first().copied(), Some(11.0));
        assert_eq!(out.last().copied(), Some(30.0));
    }

    #[test]
    fn he_aac_priming_drops_exact_5186_output_frames() {
        use super::super::aac::PRIMING_HE_FRAMES;
        struct FrameCounter {
            next: u32,
            max:  u32,
        }
        impl Iterator for FrameCounter {
            type Item = f32;
            fn next(&mut self) -> Option<f32> {
                (self.next < self.max).then(|| {
                    self.next += 1;
                    self.next as f32
                })
            }
        }
        impl Source for FrameCounter {
            fn current_span_len(&self) -> Option<usize> { None }
            fn channels(&self) -> ChannelCount { 2 }
            fn sample_rate(&self) -> SampleRate { 44_100 }
            fn total_duration(&self) -> Option<Duration> { None }
            fn try_seek(&mut self, _: Duration) -> Result<(), SeekError> { Ok(()) }
        }

        let skip = priming_duration(PRIMING_HE_FRAMES, 44_100);
        let mut trimmed = Trimmed::new(
            FrameCounter {
                next: 0,
                max:  (PRIMING_HE_FRAMES + 4) * 2,
            },
            skip,
            None,
        );
        // 5186 stereo frames skipped -> first sample is (5186 * 2 + 1).
        assert_eq!(trimmed.next(), Some((PRIMING_HE_FRAMES * 2 + 1) as f32));
        let rest: Vec<f32> = trimmed.collect();
        assert_eq!(rest.len(), 7);
    }
}
