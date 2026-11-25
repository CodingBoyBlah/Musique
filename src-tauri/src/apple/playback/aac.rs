//! AAC / HE-AAC decoding bridge between Apple Music streams, Symphonia, and `rusty_aac`.
//!
//! Symphonia's built-in AAC decoder supports AAC-LC only and decodes HE-AAC (`32:ctrp64`) at
//! half sample rate without Spectral Band Replication (SBR) or Parametric Stereo (PS). This module
//! implements a Symphonia [`Decoder`] wrapper ([`AppleAacDecoder`]) that inspects the stream's
//! `AudioSpecificConfig` (`extra_data`): streams signalling SBR or PS are routed to `rusty_aac`
//! for full-rate HE-AAC v1/v2 reconstruction, while plain AAC-LC stays on Symphonia's decoder.

use std::io::{ErrorKind, Read, Seek, SeekFrom};
use std::sync::OnceLock;
use std::time::Duration;

use rodio::source::SeekError;
use rodio::{ChannelCount, Sample as RodioSample, SampleRate, Source};
use symphonia::core::audio::{
    AsAudioBufferRef, AudioBuffer, AudioBufferRef, Channels, SampleBuffer, Signal, SignalSpec,
};
use symphonia::core::codecs::{
    CodecDescriptor, CodecParameters, CodecRegistry, Decoder, DecoderOptions, FinalizeResult,
    CODEC_TYPE_AAC, CODEC_TYPE_NULL,
};
use symphonia::core::errors::{unsupported_error, Error as SymError, Result as SymResult};
use symphonia::core::formats::{FormatOptions, FormatReader, Packet, SeekMode, SeekTo, SeekedTo};
use symphonia::core::io::{MediaSource, MediaSourceStream};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::support_codec;
use symphonia::core::units::Duration as SymDuration;
use symphonia::default::codecs::AacDecoder as SymphoniaAacDecoder;

use crate::errors::AppError;

/// Standard Apple CoreAudio AAC-LC encoder priming delay at the core sample rate.
pub const PRIMING_LC_FRAMES: u32 = 2112;

/// Additional SBR filterbank delay introduced by HE-AAC at the 2x output sample rate.
pub const PRIMING_HE_SBR_DELAY: u32 = 962;

/// Total HE-AAC priming delay at the 2x SBR output rate (`2112 * 2 + 962 = 5186` frames).
pub const PRIMING_HE_FRAMES: u32 = PRIMING_LC_FRAMES * 2 + PRIMING_HE_SBR_DELAY;

/// Returns parsed SBR/PS details when `extra_data` (`AudioSpecificConfig`) signals SBR or PS.
///
/// Plain AAC-LC and unrecognized/empty configs return `None`, leaving them on Symphonia's decoder.
pub fn he_aac_config(extra_data: Option<&[u8]>) -> Option<rusty_aac::sbr::SbrConfig> {
    let data = extra_data?;
    let cfg = rusty_aac::sbr::parse_sbr_config(data).ok()?;
    (cfg.sbr_present || cfg.ps_present).then_some(cfg)
}

/// Computes encoder + SBR priming frames at the decoder's output sample rate.
pub fn priming_frames(he_config: Option<rusty_aac::sbr::SbrConfig>) -> u32 {
    match he_config {
        Some(cfg) if cfg.output_sample_rate > cfg.core_sample_rate => PRIMING_HE_FRAMES,
        Some(_) => PRIMING_LC_FRAMES + PRIMING_HE_SBR_DELAY,
        None => PRIMING_LC_FRAMES,
    }
}

enum AacBackend {
    Symphonia(SymphoniaAacDecoder),
    RustyHeAac {
        decoder: rusty_aac::AacDecoder,
        extra:   Box<[u8]>,
        buf:     AudioBuffer<f32>,
    },
}

/// Symphonia [`Decoder`] for AAC that routes HE-AAC (SBR/PS) streams to `rusty_aac` and leaves
/// plain AAC-LC on Symphonia's built-in AAC decoder.
pub struct AppleAacDecoder {
    backend: AacBackend,
    params:  CodecParameters,
}

impl AppleAacDecoder {
    #[cfg(test)]
    fn uses_rusty_aac(&self) -> bool {
        matches!(self.backend, AacBackend::RustyHeAac { .. })
    }
}

impl Decoder for AppleAacDecoder {
    fn try_new(params: &CodecParameters, options: &DecoderOptions) -> SymResult<Self> {
        if params.codec != CODEC_TYPE_AAC {
            return unsupported_error("aac: invalid codec type");
        }

        if let Some(sbr) = he_aac_config(params.extra_data.as_deref()) {
            let extra: Box<[u8]> = params.extra_data.as_deref().unwrap_or_default().into();
            let decoder = rusty_aac::AacDecoder::with_config_bytes(&extra).map_err(map_rusty_err)?;

            let mut updated = params.clone();
            updated.with_sample_rate(sbr.output_sample_rate);

            let initial_ch = initial_channels(&extra, sbr, params.channels);
            if let Some(mask) = map_channels(initial_ch) {
                updated.with_channels(mask);
            }
            let spec = SignalSpec::new(
                sbr.output_sample_rate.max(1),
                map_channels(initial_ch).unwrap_or(Channels::FRONT_LEFT | Channels::FRONT_RIGHT),
            );
            let buf = AudioBuffer::new(0, spec);

            return Ok(Self {
                backend: AacBackend::RustyHeAac {
                    decoder,
                    extra,
                    buf,
                },
                params: updated,
            });
        }

        let inner = SymphoniaAacDecoder::try_new(params, options)?;
        let updated = inner.codec_params().clone();
        Ok(Self {
            backend: AacBackend::Symphonia(inner),
            params: updated,
        })
    }

    fn supported_codecs() -> &'static [CodecDescriptor] {
        &[support_codec!(
            CODEC_TYPE_AAC,
            "aac",
            "Advanced Audio Coding (LC + HE-AAC SBR/PS)"
        )]
    }

    fn reset(&mut self) {
        match &mut self.backend {
            AacBackend::Symphonia(inner) => inner.reset(),
            AacBackend::RustyHeAac {
                decoder,
                extra,
                buf,
            } => {
                buf.clear();
                if let Ok(fresh) = rusty_aac::AacDecoder::with_config_bytes(extra) {
                    *decoder = fresh;
                }
            }
        }
    }

    fn codec_params(&self) -> &CodecParameters {
        match &self.backend {
            AacBackend::Symphonia(inner) => inner.codec_params(),
            AacBackend::RustyHeAac { .. } => &self.params,
        }
    }

    fn decode(&mut self, packet: &Packet) -> SymResult<AudioBufferRef<'_>> {
        match &mut self.backend {
            AacBackend::Symphonia(inner) => inner.decode(packet),
            AacBackend::RustyHeAac { decoder, buf, .. } => {
                match decoder.decode(packet.buf(), None) {
                    Ok(decoded) => write_interleaved_to_audio_buffer(&decoded, buf),
                    Err(rusty_aac::Error::Again) => {
                        buf.clear();
                        Ok(buf.as_audio_buffer_ref())
                    }
                    Err(err) => {
                        buf.clear();
                        Err(map_rusty_err(err))
                    }
                }
            }
        }
    }

    fn finalize(&mut self) -> FinalizeResult {
        match &mut self.backend {
            AacBackend::Symphonia(inner) => inner.finalize(),
            AacBackend::RustyHeAac { .. } => FinalizeResult::default(),
        }
    }

    fn last_decoded(&self) -> AudioBufferRef<'_> {
        match &self.backend {
            AacBackend::Symphonia(inner) => inner.last_decoded(),
            AacBackend::RustyHeAac { buf, .. } => buf.as_audio_buffer_ref(),
        }
    }
}

fn write_interleaved_to_audio_buffer<'a>(
    decoded: &rusty_aac::DecodedAudio,
    buf: &'a mut AudioBuffer<f32>,
) -> SymResult<AudioBufferRef<'a>> {
    let frames = decoded.frames();
    let nch = usize::from(decoded.channels);
    if frames == 0 || nch == 0 {
        buf.clear();
        return Ok(buf.as_audio_buffer_ref());
    }
    let expected = frames.saturating_mul(nch);
    if decoded.samples.len() < expected {
        buf.clear();
        return Err(SymError::DecodeError("aac: truncated decoded PCM buffer"));
    }

    let channels = map_channels(decoded.channels)
        .ok_or(SymError::Unsupported("aac: unsupported channel layout"))?;
    let spec = SignalSpec::new(decoded.sample_rate.max(1), channels);
    if *buf.spec() != spec || buf.capacity() < frames {
        *buf = AudioBuffer::new(frames as u64, spec);
    }

    buf.clear();
    buf.render_reserved(Some(frames));
    {
        let mut planes = buf.planes_mut();
        let slices = planes.planes();
        for f in 0..frames {
            let base = f * nch;
            for c in 0..nch {
                slices[c][f] = decoded.samples[base + c];
            }
        }
    }
    Ok(buf.as_audio_buffer_ref())
}

fn initial_channels(
    extra: &[u8],
    sbr: rusty_aac::sbr::SbrConfig,
    param_channels: Option<Channels>,
) -> u16 {
    if let Ok(stream) = rusty_aac::config::parse(extra) {
        let ch = stream.channels();
        if sbr.ps_present || (sbr.sbr_present && ch == 1) {
            // Mono core with SBR/PS expands to stereo in rusty_aac when PS is present or implicit.
            return 2;
        }
        if ch > 0 {
            return ch as u16;
        }
    }
    param_channels.map_or(2, |c| c.count().max(1) as u16)
}

fn map_channels(count: u16) -> Option<Channels> {
    match count {
        1 => Some(Channels::FRONT_LEFT),
        2 => Some(Channels::FRONT_LEFT | Channels::FRONT_RIGHT),
        3 => Some(Channels::FRONT_LEFT | Channels::FRONT_RIGHT | Channels::FRONT_CENTRE),
        4 => Some(
            Channels::FRONT_LEFT
                | Channels::FRONT_RIGHT
                | Channels::FRONT_CENTRE
                | Channels::REAR_CENTRE,
        ),
        5 => Some(
            Channels::FRONT_LEFT
                | Channels::FRONT_RIGHT
                | Channels::FRONT_CENTRE
                | Channels::SIDE_LEFT
                | Channels::SIDE_RIGHT,
        ),
        6 => Some(
            Channels::FRONT_LEFT
                | Channels::FRONT_RIGHT
                | Channels::FRONT_CENTRE
                | Channels::SIDE_LEFT
                | Channels::SIDE_RIGHT
                | Channels::LFE1,
        ),
        7 => Some(
            Channels::FRONT_LEFT
                | Channels::FRONT_RIGHT
                | Channels::FRONT_CENTRE
                | Channels::SIDE_LEFT
                | Channels::SIDE_RIGHT
                | Channels::REAR_CENTRE
                | Channels::LFE1,
        ),
        8 => Some(
            Channels::FRONT_LEFT
                | Channels::FRONT_RIGHT
                | Channels::FRONT_CENTRE
                | Channels::SIDE_LEFT
                | Channels::SIDE_RIGHT
                | Channels::FRONT_LEFT_WIDE
                | Channels::FRONT_RIGHT_WIDE
                | Channels::LFE1,
        ),
        _ => None,
    }
}

fn map_rusty_err(err: rusty_aac::Error) -> SymError {
    match err {
        rusty_aac::Error::InvalidData(_) => SymError::DecodeError("aac: invalid HE-AAC data"),
        rusty_aac::Error::Unsupported(_) | rusty_aac::Error::Unimplemented(_) => {
            SymError::Unsupported("aac: unsupported HE-AAC feature")
        }
        rusty_aac::Error::Again | rusty_aac::Error::Eof => SymError::IoError(
            std::io::Error::new(ErrorKind::UnexpectedEof, "aac: unexpected stream state"),
        ),
        _ => SymError::DecodeError("aac: HE-AAC decode failure"),
    }
}

static CODEC_REGISTRY: OnceLock<CodecRegistry> = OnceLock::new();

/// Global Symphonia codec registry with [`AppleAacDecoder`] overriding `CODEC_TYPE_AAC`.
pub fn codec_registry() -> &'static CodecRegistry {
    CODEC_REGISTRY.get_or_init(|| {
        let mut registry = CodecRegistry::new();
        symphonia::default::register_enabled_codecs(&mut registry);
        registry.register_all::<AppleAacDecoder>();
        registry
    })
}

struct StreamMediaSource<R: Read + Seek + Send + Sync> {
    inner:       R,
    byte_len:    Option<u64>,
    is_seekable: bool,
}

impl<R: Read + Seek + Send + Sync> MediaSource for StreamMediaSource<R> {
    fn is_seekable(&self) -> bool {
        self.is_seekable
    }

    fn byte_len(&self) -> Option<u64> {
        self.byte_len
    }
}

impl<R: Read + Seek + Send + Sync> Read for StreamMediaSource<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<R: Read + Seek + Send + Sync> Seek for StreamMediaSource<R> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.inner.seek(pos)
    }
}

/// `rodio::Source` backed by Symphonia's demuxer and [`codec_registry()`].
///
/// Exists because `rodio` 0.21 binds `Decoder` to `symphonia::default::get_codecs()` without a
/// hook to override AAC decoding with [`AppleAacDecoder`].
pub struct SymSource {
    decoder:        Box<dyn Decoder>,
    format:         Box<dyn FormatReader>,
    track_id:       u32,
    buffer:         SampleBuffer<RodioSample>,
    offset:         usize,
    spec:           SignalSpec,
    total_duration: Option<Duration>,
    priming:        u32,
    seekable:       bool,
}

impl SymSource {
    /// Probes container bytes from `reader` and initializes the matching decoder.
    pub fn open<R: Read + Seek + Send + Sync + 'static>(
        reader: R,
        byte_len: Option<u64>,
        seekable: bool,
        mime_type: Option<&str>,
    ) -> Result<Self, AppError> {
        let mut hint = Hint::new();
        if let Some(mime) = mime_type {
            hint.mime_type(mime);
        }

        let mss = MediaSourceStream::new(
            Box::new(StreamMediaSource {
                inner: reader,
                byte_len,
                is_seekable: seekable,
            }) as Box<dyn MediaSource>,
            Default::default(),
        );

        let probed = symphonia::default::get_probe()
            .format(
                &hint,
                mss,
                &FormatOptions {
                    enable_gapless: true,
                    ..Default::default()
                },
                &MetadataOptions::default(),
            )
            .map_err(|e| AppError::Playback(format!("cannot probe apple audio container: {e}")))?;

        let mut format = probed.format;
        let track = format
            .default_track()
            .filter(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .or_else(|| {
                format
                    .tracks()
                    .iter()
                    .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            })
            .ok_or_else(|| AppError::Playback("apple stream contains no playable track".into()))?;

        let track_id = track.id;
        let total_duration = track
            .codec_params
            .time_base
            .zip(track.codec_params.n_frames)
            .map(|(base, frames)| base.calc_time(frames).into());
        let he_cfg = (track.codec_params.codec == CODEC_TYPE_AAC)
            .then(|| he_aac_config(track.codec_params.extra_data.as_deref()))
            .flatten();
        let priming = priming_frames(he_cfg);

        let mut decoder = codec_registry()
            .make(&track.codec_params, &DecoderOptions::default())
            .map_err(|e| AppError::Playback(format!("cannot create apple audio decoder: {e}")))?;

        let decoded = loop {
            let packet = match format.next_packet() {
                Ok(packet) => packet,
                Err(SymError::IoError(err)) if err.kind() == ErrorKind::UnexpectedEof => {
                    break decoder.last_decoded();
                }
                Err(err) => {
                    return Err(AppError::Playback(format!(
                        "cannot read first apple audio packet: {err}"
                    )));
                }
            };
            if packet.track_id() != track_id {
                continue;
            }
            match decoder.decode(&packet) {
                Ok(decoded) if decoded.frames() > 0 => break decoded,
                Ok(_) | Err(SymError::DecodeError(_)) => continue,
                Err(err) => {
                    return Err(AppError::Playback(format!(
                        "cannot decode first apple audio packet: {err}"
                    )));
                }
            }
        };

        let spec = *decoded.spec();
        let buffer = interleaved_sample_buffer(decoded, &spec);

        Ok(Self {
            decoder,
            format,
            track_id,
            buffer,
            offset: 0,
            spec,
            total_duration,
            priming,
            seekable,
        })
    }

    /// Expected leading encoder/SBR priming frames at [`Self::sample_rate()`].
    pub fn priming_frames(&self) -> u32 {
        self.priming
    }

    fn refine_position(&mut self, seek_res: SeekedTo) -> Result<(), SeekError> {
        let Some(tb) = self.decoder.codec_params().time_base else {
            return Err(SeekError::NotSupported {
                underlying_source: "apple::playback::SymSource",
            });
        };
        let ch = self.channels().max(1) as usize;
        let mut samples_to_skip = (Duration::from(
            tb.calc_time(seek_res.required_ts.saturating_sub(seek_res.actual_ts)),
        )
        .as_secs_f32()
            * self.sample_rate() as f32
            * ch as f32)
            .ceil() as usize;
        samples_to_skip -= samples_to_skip % ch;

        for _ in 0..samples_to_skip {
            self.next();
        }
        Ok(())
    }
}

fn interleaved_sample_buffer(
    decoded: AudioBufferRef<'_>,
    spec: &SignalSpec,
) -> SampleBuffer<RodioSample> {
    let duration = SymDuration::from(decoded.capacity() as u64);
    let mut buffer = SampleBuffer::<RodioSample>::new(duration, *spec);
    buffer.copy_interleaved_ref(decoded);
    buffer
}

impl Iterator for SymSource {
    type Item = RodioSample;

    fn next(&mut self) -> Option<Self::Item> {
        while self.offset >= self.buffer.len() {
            let decoded = loop {
                let packet = self.format.next_packet().ok()?;
                if packet.track_id() != self.track_id {
                    continue;
                }
                match self.decoder.decode(&packet) {
                    Ok(decoded) if decoded.frames() > 0 => break decoded,
                    Ok(_) | Err(SymError::DecodeError(_)) => continue,
                    Err(_) => return None,
                }
            };
            decoded.spec().clone_into(&mut self.spec);
            self.buffer = interleaved_sample_buffer(decoded, &self.spec);
            self.offset = 0;
        }

        let sample = *self.buffer.samples().get(self.offset)?;
        self.offset += 1;
        Some(sample)
    }
}

impl Source for SymSource {
    fn current_span_len(&self) -> Option<usize> {
        Some(self.buffer.len().saturating_sub(self.offset))
    }

    fn channels(&self) -> ChannelCount {
        self.spec.channels.count().max(1) as ChannelCount
    }

    fn sample_rate(&self) -> SampleRate {
        self.spec.rate.max(1)
    }

    fn total_duration(&self) -> Option<Duration> {
        self.total_duration
    }

    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        if !self.seekable || self.decoder.codec_params().time_base.is_none() {
            return Err(SeekError::NotSupported {
                underlying_source: "apple::playback::SymSource",
            });
        }

        let target = self
            .total_duration
            .map_or(pos, |total| pos.min(total));
        let active_channel = self.offset % self.channels() as usize;

        let seek_res = self
            .format
            .seek(
                SeekMode::Accurate,
                SeekTo::Time {
                    time:     target.into(),
                    track_id: Some(self.track_id),
                },
            )
            .map_err(|e| SeekError::Other(Box::new(e)))?;

        self.decoder.reset();
        self.offset = usize::MAX;
        self.refine_position(seek_res)?;

        for _ in 0..active_channel {
            self.next();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asc_routing_detects_sbr_ps_and_leaves_lc_on_symphonia() {
        // Plain AAC-LC (stereo 44100 Hz): 0x12 0x10 -> AOT 2, no SBR/PS.
        let lc_asc = [0x12u8, 0x10];
        assert!(he_aac_config(Some(&lc_asc)).is_none());
        assert_eq!(priming_frames(None), PRIMING_LC_FRAMES);

        // Explicit hierarchical HE-AAC v1 (AOT 5, 22050 core -> 44100 ext, stereo):
        // 00101 (5) | 0111 (22050) | 0010 (ch=2) | 0100 (44100) | 00010 (AOT 2) | 000
        let he_v1_asc = [0x2B, 0x92, 0x08, 0x00];
        let v1 = he_aac_config(Some(&he_v1_asc)).expect("HE-AAC v1 must be detected");
        assert!(v1.sbr_present);
        assert!(!v1.ps_present);
        assert_eq!(v1.core_sample_rate, 22_050);
        assert_eq!(v1.output_sample_rate, 44_100);
        assert_eq!(priming_frames(Some(v1)), 5186);

        // Explicit hierarchical HE-AAC v2 from ISO corpus (AOT 29, 22050 core -> 44100 ext, mono+PS):
        let he_v2_asc = [0xEB, 0x8A, 0x08, 0x00];
        let v2 = he_aac_config(Some(&he_v2_asc)).expect("HE-AAC v2 must be detected");
        assert!(v2.sbr_present && v2.ps_present);
        assert_eq!(v2.core_sample_rate, 22_050);
        assert_eq!(v2.output_sample_rate, 44_100);
        assert_eq!(priming_frames(Some(v2)), 5186);

        // Backward-compatible signalling with sbrPresentFlag = 0 must NOT select rusty_aac:
        // AOT 2 (00010), sf 6=24000 (0110), ch=1 (0001), GASpecificConfig(000),
        // sync ext 0x2B7 (01010110111), ext AOT 5 (00101), sbrPresentFlag=0 (0)
        // bits: 00010 0110 0001 000 01010110111 00101 0 = 33 bits
        let sbr_off_asc = [0b0001_0011, 0b0000_1000, 0b0101_0110, 0b1110_0101, 0x00];
        assert!(he_aac_config(Some(&sbr_off_asc)).is_none());

        // Backward-compatible SBR signalling with sbrPresentFlag = 1 and ext sf 3 (48000):
        // bits: 00010 0110 0010 000 | 01010110111 | 00101 | 1 | 0011 = 37 bits
        let sbr_on_asc = [0b0001_0011, 0b0001_0000, 0b0101_0110, 0b1110_0101, 0b1001_1000];
        let bc = he_aac_config(Some(&sbr_on_asc)).expect("backward-compat SBR must be detected");
        assert!(bc.sbr_present);
        assert_eq!(bc.core_sample_rate, 24_000);
        assert_eq!(bc.output_sample_rate, 48_000);
        assert_eq!(priming_frames(Some(bc)), 5186);
    }

    #[test]
    fn apple_aac_decoder_routes_by_asc_and_decodes_through_codec_registry() {
        // Build an encoded raw AAC-LC packet via rusty_aac's encoder to verify both
        // Symphonia LC passthrough and CodecRegistry registration.
        let sr = 44_100u32;
        let pcm: Vec<f32> = (0..4096)
            .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / sr as f32).sin() * 0.4)
            .collect();
        let mut enc = rusty_aac::AacEncoder::new(rusty_aac::AacEncoderConfig::default());
        enc.push_pcm(&pcm, 2, sr).unwrap();
        enc.finish();
        let packet_bytes = enc.next_packet().unwrap().data;

        let lc_asc = rusty_aac::audio_specific_config_bytes(sr, 2);
        let mut lc_params = CodecParameters::new();
        lc_params
            .for_codec(CODEC_TYPE_AAC)
            .with_sample_rate(sr)
            .with_extra_data(lc_asc.into());

        let mut lc_dec = AppleAacDecoder::try_new(&lc_params, &DecoderOptions::default()).unwrap();
        assert!(!lc_dec.uses_rusty_aac(), "AAC-LC must stay on Symphonia");
        let pkt = Packet::new_from_slice(0, 0, 1024, &packet_bytes);
        let decoded = lc_dec.decode(&pkt).unwrap();
        assert_eq!(decoded.spec().rate, 44_100);
        assert_eq!(decoded.spec().channels.count(), 2);
        assert!(decoded.frames() > 0);

        // HE-AAC v2 ASC routes to rusty_aac via both direct construction and CodecRegistry.
        let he_asc: Box<[u8]> = Box::new([0xEB, 0x8A, 0x08, 0x00]);
        let mut he_params = CodecParameters::new();
        he_params
            .for_codec(CODEC_TYPE_AAC)
            .with_sample_rate(22_050)
            .with_extra_data(he_asc);

        let mut he_dec = AppleAacDecoder::try_new(&he_params, &DecoderOptions::default()).unwrap();
        assert!(he_dec.uses_rusty_aac(), "HE-AAC must route to rusty_aac");
        assert_eq!(he_dec.codec_params().sample_rate, Some(44_100));
        assert_eq!(he_dec.codec_params().channels.map(|c| c.count()), Some(2));

        // Decode a 22050 Hz mono core access unit through the HE-AAC v2 decoder:
        // verifies SBR dual-rate upsampling (44100 Hz, 2048 frames) and PS mono->stereo output.
        let core_sr = 22_050u32;
        let mono_pcm: Vec<f32> = (0..2048)
            .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / core_sr as f32).sin() * 0.4)
            .collect();
        let mut core_enc = rusty_aac::AacEncoder::new(rusty_aac::AacEncoderConfig::default());
        core_enc.push_pcm(&mono_pcm, 1, core_sr).unwrap();
        core_enc.finish();
        let core_pkt = core_enc.next_packet().unwrap().data;
        let he_pkt = Packet::new_from_slice(0, 0, 1024, &core_pkt);
        let he_decoded = he_dec.decode(&he_pkt).unwrap();
        assert_eq!(he_decoded.spec().rate, 44_100);
        assert_eq!(he_decoded.spec().channels.count(), 2);
        assert_eq!(he_decoded.frames(), 2048);

        // Corrupt packet clears last_decoded per Symphonia's Decoder contract and reset recovers.
        let corrupt = Packet::new_from_slice(0, 0, 1024, &[0x00]);
        assert!(he_dec.decode(&corrupt).is_err());
        assert_eq!(he_dec.last_decoded().frames(), 0);
        he_dec.reset();
        assert!(he_dec.decode(&he_pkt).is_ok());

        // Registry returns AppleAacDecoder (not default Symphonia AAC, which rejects AOT 29 / SBR).
        let mut reg_dec = codec_registry()
            .make(&he_params, &DecoderOptions::default())
            .expect("registered AppleAacDecoder should accept HE-AAC v2 params");
        assert_eq!(reg_dec.codec_params().sample_rate, Some(44_100));
        reg_dec.reset();
    }

    #[test]
    fn he_aac_pcm_transfer_and_sym_source_adts_decode() {
        // Encode mono AAC-LC ADTS and wrap HE-AAC AudioBuffer transfer validation + SymSource.
        let sr = 44_100u32;
        let pcm: Vec<f32> = (0..4096)
            .map(|i| (2.0 * std::f32::consts::PI * 330.0 * i as f32 / sr as f32).sin() * 0.5)
            .collect();
        let mut enc = rusty_aac::AacEncoder::new(rusty_aac::AacEncoderConfig::default());
        enc.push_pcm(&pcm, 2, sr).unwrap();
        enc.finish();

        // Check planar interleaved buffer conversion on Stereo 44100 Hz output.
        let fake_audio = rusty_aac::DecodedAudio {
            sample_rate: 48_000,
            channels:    2,
            samples:     vec![0.25, -0.25, 0.5, -0.5],
            pts:         None,
        };
        let mut ab = AudioBuffer::unused();
        let cref = write_interleaved_to_audio_buffer(&fake_audio, &mut ab).unwrap();
        assert_eq!(cref.frames(), 2);
        assert_eq!(cref.spec().rate, 48_000);
        assert_eq!(cref.spec().channels.count(), 2);
        let mut sb = SampleBuffer::<f32>::new(2, *cref.spec());
        sb.copy_interleaved_ref(cref);
        assert_eq!(sb.samples(), &[0.25, -0.25, 0.5, -0.5]);

        // Also check ADTS decoding through SymSource + AppleAacDecoder (AAC-LC path).
        let mut adts = Vec::new();
        while let Ok(pkt) = enc.next_packet() {
            let hdr = rusty_aac::AdtsHeader {
                object_type:  2,
                sample_rate:  sr,
                channels:     2,
                frame_length: 7 + pkt.data.len(),
                header_len:   7,
            };
            adts.extend_from_slice(&rusty_aac::write_adts_header(&hdr));
            adts.extend_from_slice(&pkt.data);
        }
        let byte_len = adts.len() as u64;
        let mut src = SymSource::open(std::io::Cursor::new(adts), Some(byte_len), true, None)
            .expect("SymSource should open ADTS stream");
        assert_eq!(src.sample_rate(), sr);
        assert_eq!(src.channels(), 2);
        assert_eq!(src.priming_frames(), PRIMING_LC_FRAMES);
        let samples: Vec<f32> = (&mut src).take(2048).collect();
        assert_eq!(samples.len(), 2048);
        assert!(samples.iter().any(|s| s.abs() > 0.01));
    }
}
