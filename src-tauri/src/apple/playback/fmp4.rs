//! Minimal ISO-BMFF parser for CENC-encrypted fragmented MP4 streams (`ftyp` + `moov`,
//! followed by `moof`/`mdat` pairs).
//!
//! Bounds-checks every offset against the supplied slice so incomplete or malformed network
//! buffers yield `None`/`Partial` cleanly instead of panicking.

const ENCA: &[u8; 4] = b"enca";
const MP4A: &[u8; 4] = b"mp4a";
const SCHEME_CENC: &[u8; 4] = b"cenc";

/// Guard against malformed `trun`/`senc` sample counts.
const MAX_FRAGMENT_SAMPLES: usize = 1 << 20;

const FULLBOX_FLAGS_MASK: u32 = 0x00ff_ffff;

// `tfhd` flags (ISO/IEC 14496-12)
const TFHD_BASE_DATA_OFFSET: u32 = 0x0000_0001;
const TFHD_SAMPLE_DESC_INDEX: u32 = 0x0000_0002;
const TFHD_DEFAULT_DURATION: u32 = 0x0000_0008;
const TFHD_DEFAULT_SIZE: u32 = 0x0000_0010;

// `trun` flags (ISO/IEC 14496-12)
const TRUN_DATA_OFFSET: u32 = 0x0000_0001;
const TRUN_FIRST_SAMPLE_FLAGS: u32 = 0x0000_0004;
const TRUN_SAMPLE_DURATION: u32 = 0x0000_0100;
const TRUN_SAMPLE_SIZE: u32 = 0x0000_0200;
const TRUN_SAMPLE_FLAGS: u32 = 0x0000_0400;
const TRUN_COMPOSITION_OFFSET: u32 = 0x0000_0800;

// `senc` flags (ISO/IEC 23001-7)
const SENC_SUBSAMPLES: u32 = 0x0000_0002;

// `saiz` / `saio` flags
const AUX_INFO_TYPE_PRESENT: u32 = 0x0000_0001;

#[derive(Clone, Copy, Debug)]
struct BoxSpan {
    kind:  [u8; 4],
    body:  usize,
    end:   usize,
    total: usize,
}

/// Encrypted track metadata extracted from `moov`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EncryptedTrack {
    pub track_id:    u32,
    pub iv_size:     u8,
    pub constant_iv: Option<[u8; 16]>,
}

/// Parsed initialization segment (`ftyp`..`moov`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InitInfo {
    /// Byte offset immediately after `moov`.
    pub end:       usize,
    /// Offsets of `enca` sample-entry boxes to relabel as `mp4a` in place.
    pub entries:   Vec<usize>,
    pub tracks:    Vec<EncryptedTrack>,
    /// Media timescale from `mdhd`.
    pub timescale: u32,
}

/// One sample inside an `mdat` box with its CENC IV and optional subsample runs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SampleEntry {
    pub start: usize,
    pub len:   usize,
    pub iv:    [u8; 16],
    /// `(clear_bytes, encrypted_bytes)` runs; empty means full-sample encryption.
    pub subs:  Vec<(u32, u32)>,
}

impl SampleEntry {
    pub fn end(&self) -> usize {
        self.start + self.len
    }
}

/// Result of parsing one top-level box starting at `origin`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FragmentStep {
    Fragment {
        samples:      Vec<SampleEntry>,
        next:         usize,
        decode_ticks: Option<u64>,
    },
    Other {
        next: usize,
    },
    Partial,
}

fn be16(data: &[u8], at: usize) -> Option<u16> {
    let b = data.get(at..at + 2)?;
    Some(u16::from_be_bytes([b[0], b[1]]))
}

fn be32(data: &[u8], at: usize) -> Option<u32> {
    let b = data.get(at..at + 4)?;
    Some(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
}

fn be64(data: &[u8], at: usize) -> Option<u64> {
    let b: [u8; 8] = data.get(at..at + 8)?.try_into().ok()?;
    Some(u64::from_be_bytes(b))
}

/// Reads the 4-byte kind and total byte length of a box header (up to 16 bytes for extended size).
pub fn box_extent(head: &[u8]) -> Option<([u8; 4], usize)> {
    let size = be32(head, 0)? as usize;
    let kind: [u8; 4] = head.get(4..8)?.try_into().ok()?;
    let total = match size {
        1 => usize::try_from(be64(head, 8)?).ok()?,
        0..8 => return None,
        n => n,
    };
    let min_hdr = if size == 1 { 16 } else { 8 };
    (total >= min_hdr).then_some((kind, total))
}

fn read_box(data: &[u8], at: usize) -> Option<BoxSpan> {
    let size = be32(data, at)? as usize;
    let kind: [u8; 4] = data.get(at + 4..at + 8)?.try_into().ok()?;
    let (body, total) = match size {
        1 => (at + 16, usize::try_from(be64(data, at + 8)?).ok()?),
        0..8 => return None,
        n => (at + 8, n),
    };
    let end = at.checked_add(total)?;
    (body <= end && end <= data.len()).then_some(BoxSpan {
        kind,
        body,
        end,
        total,
    })
}

struct Children<'a> {
    data: &'a [u8],
    at:   usize,
    end:  usize,
}

impl Iterator for Children<'_> {
    type Item = (usize, BoxSpan);

    fn next(&mut self) -> Option<Self::Item> {
        if self.at + 8 > self.end {
            return None;
        }
        let Some(bx) = read_box(self.data, self.at).filter(|b| b.end <= self.end) else {
            self.at = self.end;
            return None;
        };
        let offset = self.at;
        self.at += bx.total;
        Some((offset, bx))
    }
}

fn children(data: &[u8], body: usize, end: usize) -> Children<'_> {
    Children { data, at: body, end }
}

fn find_child(data: &[u8], body: usize, end: usize, kind: &[u8; 4]) -> Option<BoxSpan> {
    children(data, body, end)
        .find(|(_, b)| &b.kind == kind)
        .map(|(_, b)| b)
}

fn find_descendant(data: &[u8], body: usize, end: usize, kind: &[u8; 4]) -> Option<BoxSpan> {
    for (_, bx) in children(data, body, end) {
        if &bx.kind == kind {
            return Some(bx);
        }
        if let Some(found) = find_descendant(data, bx.body, bx.end, kind) {
            return Some(found);
        }
    }
    None
}

/// Parses `ftyp`..`moov` at the front of the file. Returns `None` until `moov` is complete.
pub fn read_init(data: &[u8]) -> Option<InitInfo> {
    let mut at = 0usize;
    let moov = loop {
        let bx = read_box(data, at)?;
        if &bx.kind == b"moov" {
            break bx;
        }
        at = bx.end;
    };

    let mut init = InitInfo {
        end:       moov.end,
        entries:   Vec::new(),
        tracks:    Vec::new(),
        timescale: 0,
    };

    for (_, trak) in children(data, moov.body, moov.end).filter(|(_, b)| &b.kind == b"trak") {
        let track_id = parse_tkhd_id(data, trak.body, trak.end).unwrap_or(0);
        if let Some(ts) = parse_mdhd_timescale(data, trak.body, trak.end) {
            init.timescale = ts;
        }
        let Some(stsd) = find_descendant(data, trak.body, trak.end, b"stsd") else {
            continue;
        };
        if stsd.body + 8 > stsd.end {
            continue;
        }
        for (entry_at, entry) in children(data, stsd.body + 8, stsd.end) {
            if &entry.kind != ENCA {
                continue;
            }
            // AudioSampleEntry adds 28 bytes (reserved, data_ref_idx, channel/sample fields)
            // before nested boxes such as `esds` and `sinf`.
            if entry.body + 28 > entry.end {
                continue;
            }
            let Some((scheme_ok, iv_size, constant_iv)) =
                parse_sinf(data, entry.body + 28, entry.end)
            else {
                continue;
            };
            if !scheme_ok {
                log::warn!("cenc: skipping non-cenc encrypted track {track_id}");
                continue;
            }
            init.entries.push(entry_at);
            init.tracks.push(EncryptedTrack {
                track_id,
                iv_size,
                constant_iv,
            });
        }
    }

    Some(init)
}

fn parse_tkhd_id(data: &[u8], body: usize, end: usize) -> Option<u32> {
    let tkhd = find_child(data, body, end, b"tkhd")?;
    let at = match *data.get(tkhd.body)? {
        0 => tkhd.body + 12,
        _ => tkhd.body + 20,
    };
    (at + 4 <= tkhd.end).then(|| be32(data, at)).flatten()
}

fn parse_mdhd_timescale(data: &[u8], body: usize, end: usize) -> Option<u32> {
    let mdia = find_child(data, body, end, b"mdia")?;
    let mdhd = find_child(data, mdia.body, mdia.end, b"mdhd")?;
    let at = match *data.get(mdhd.body)? {
        0 => mdhd.body + 12,
        _ => mdhd.body + 20,
    };
    (at + 4 <= mdhd.end)
        .then(|| be32(data, at))
        .flatten()
        .filter(|&ts| ts > 0)
}

fn parse_sinf(data: &[u8], body: usize, end: usize) -> Option<(bool, u8, Option<[u8; 16]>)> {
    let sinf = find_child(data, body, end, b"sinf")?;
    if let Some(schm) = find_child(data, sinf.body, sinf.end, b"schm") {
        if schm.body + 8 <= schm.end {
            let scheme = &data[schm.body + 4..schm.body + 8];
            if scheme != SCHEME_CENC {
                return Some((false, 0, None));
            }
        }
    }
    let schi = find_child(data, sinf.body, sinf.end, b"schi")?;
    let tenc = find_child(data, schi.body, schi.end, b"tenc")?;
    let iv_size = *data.get(tenc.body + 7)?;
    let constant_iv = if iv_size == 0 && tenc.body + 25 <= tenc.end {
        let const_len = usize::from(*data.get(tenc.body + 24)?).min(16);
        let raw = data.get(tenc.body + 25..tenc.body + 25 + const_len)?;
        let mut iv = [0u8; 16];
        iv[..const_len].copy_from_slice(raw);
        Some(iv)
    } else {
        None
    };
    Some((true, iv_size, constant_iv))
}

/// Relabels every `enca` sample entry box type in `buf` to `mp4a` in place without changing offsets.
pub fn relabel_enca_to_mp4a(buf: &mut [u8], init: &InitInfo) {
    for &at in &init.entries {
        if let Some(kind) = buf.get_mut(at + 4..at + 8) {
            kind.copy_from_slice(MP4A);
        }
    }
}

/// Parses one `moof` + `mdat` fragment starting at `origin` in the file.
pub fn read_fragment(window: &[u8], origin: usize, tracks: &[EncryptedTrack]) -> FragmentStep {
    let Some(moof) = read_box(window, 0) else {
        return FragmentStep::Partial;
    };
    if &moof.kind != b"moof" {
        return FragmentStep::Other {
            next: origin + moof.end,
        };
    }
    let Some(mdat) = read_box(window, moof.end) else {
        return FragmentStep::Partial;
    };
    if &mdat.kind != b"mdat" {
        log::warn!("cenc: moof at {origin} is not followed by mdat");
        return FragmentStep::Other {
            next: origin + moof.end,
        };
    }

    let mut samples = Vec::new();
    let mut decode_ticks = None;
    for (_, traf) in children(window, moof.body, moof.end).filter(|(_, b)| &b.kind == b"traf") {
        decode_ticks = decode_ticks.or_else(|| parse_tfdt(window, &traf));
        let found = parse_traf(window, origin, &mdat, &traf, tracks);
        if samples.is_empty() {
            samples = found;
        } else {
            samples.extend(found);
        }
    }

    FragmentStep::Fragment {
        samples,
        next: origin + mdat.end,
        decode_ticks,
    }
}

fn parse_tfdt(data: &[u8], traf: &BoxSpan) -> Option<u64> {
    let tfdt = find_child(data, traf.body, traf.end, b"tfdt")?;
    match *data.get(tfdt.body)? {
        0 => be32(data, tfdt.body + 4).map(u64::from),
        _ => be64(data, tfdt.body + 4),
    }
}

#[derive(Clone, Copy, Default)]
struct TfhdInfo {
    base_data_offset: Option<u64>,
    default_size:     Option<u32>,
}

fn parse_tfhd(data: &[u8], tfhd: &BoxSpan) -> TfhdInfo {
    let Some(flags) = be32(data, tfhd.body).map(|w| w & FULLBOX_FLAGS_MASK) else {
        return TfhdInfo::default();
    };
    let mut at = tfhd.body + 8;
    let mut info = TfhdInfo::default();
    if flags & TFHD_BASE_DATA_OFFSET != 0 {
        info.base_data_offset = be64(data, at);
        at += 8;
    }
    if flags & TFHD_SAMPLE_DESC_INDEX != 0 {
        at += 4;
    }
    if flags & TFHD_DEFAULT_DURATION != 0 {
        at += 4;
    }
    if flags & TFHD_DEFAULT_SIZE != 0 {
        info.default_size = be32(data, at).filter(|&s| s > 0);
    }
    info
}

struct TrunInfo {
    data_offset: i64,
    sizes:       Vec<u32>,
}

fn parse_trun(data: &[u8], trun: &BoxSpan, tfhd: &TfhdInfo) -> Option<TrunInfo> {
    let flags = be32(data, trun.body)? & FULLBOX_FLAGS_MASK;
    let count = be32(data, trun.body + 4)? as usize;
    if count > MAX_FRAGMENT_SAMPLES {
        log::warn!("cenc: ignoring trun declaring {count} samples");
        return None;
    }

    let mut at = trun.body + 8;
    let data_offset = if flags & TRUN_DATA_OFFSET != 0 {
        let off = i64::from(be32(data, at)? as i32);
        at += 4;
        off
    } else {
        0
    };
    if flags & TRUN_FIRST_SAMPLE_FLAGS != 0 {
        at += 4;
    }

    let mut sizes = Vec::with_capacity(count.min(4096));
    for _ in 0..count {
        if flags & TRUN_SAMPLE_DURATION != 0 {
            at += 4;
        }
        if flags & TRUN_SAMPLE_SIZE != 0 {
            sizes.push(be32(data, at)?);
            at += 4;
        } else {
            sizes.push(tfhd.default_size?);
        }
        if flags & TRUN_SAMPLE_FLAGS != 0 {
            at += 4;
        }
        if flags & TRUN_COMPOSITION_OFFSET != 0 {
            at += 4;
        }
        if at > trun.end {
            return None;
        }
    }

    Some(TrunInfo { data_offset, sizes })
}

fn parse_traf(
    data: &[u8],
    origin: usize,
    mdat: &BoxSpan,
    traf: &BoxSpan,
    tracks: &[EncryptedTrack],
) -> Vec<SampleEntry> {
    let tfhd_box = find_child(data, traf.body, traf.end, b"tfhd");
    let track_id = tfhd_box
        .and_then(|b| be32(data, b.body + 4))
        .unwrap_or(0);
    let Some(enc_track) = tracks
        .iter()
        .find(|t| t.track_id == track_id)
        .or_else(|| tracks.first().filter(|_| tracks.len() == 1))
    else {
        return Vec::new();
    };

    let Some(trun_box) = find_child(data, traf.body, traf.end, b"trun") else {
        return Vec::new();
    };
    let tfhd = tfhd_box.map(|b| parse_tfhd(data, &b)).unwrap_or_default();
    let Some(trun) = parse_trun(data, &trun_box, &tfhd) else {
        return Vec::new();
    };

    let base_in_window: Option<u64> = match tfhd.base_data_offset {
        Some(base) => base.checked_sub(origin as u64),
        None => Some(0),
    };
    let Some(start) = base_in_window
        .and_then(|b| b.checked_add_signed(trun.data_offset))
        .and_then(|s| usize::try_from(s).ok())
        .filter(|&s| s >= mdat.body && s <= mdat.end)
    else {
        log::warn!("cenc: trun data_offset points outside mdat");
        return Vec::new();
    };

    let (ivs, mut subs) = if let Some(senc) = find_child(data, traf.body, traf.end, b"senc") {
        parse_senc(data, &senc, enc_track.iv_size, trun.sizes.len())
    } else if let Some(aux) = parse_saiz_saio(data, traf, origin, base_in_window, enc_track.iv_size)
    {
        aux
    } else {
        (Vec::new(), Vec::new())
    };

    if ivs.is_empty() && enc_track.iv_size != 0 {
        log::warn!("cenc: fragment missing per-sample IVs");
        return Vec::new();
    }

    let mut samples = Vec::with_capacity(trun.sizes.len());
    let mut cursor = start;
    let mut iv = enc_track.constant_iv.unwrap_or([0u8; 16]);
    for (idx, len) in trun.sizes.into_iter().enumerate() {
        let len = len as usize;
        if let Some(&sample_iv) = ivs.get(idx) {
            iv = sample_iv;
        }
        let end = cursor.saturating_add(len);
        if end > mdat.end {
            log::warn!("cenc: sample {idx} runs past mdat end");
            break;
        }
        if len > 0 {
            samples.push(SampleEntry {
                start: origin + cursor,
                len,
                iv,
                subs: subs.get_mut(idx).map(std::mem::take).unwrap_or_default(),
            });
        }
        cursor = end;
    }

    samples
}

type SencEntries = (Vec<[u8; 16]>, Vec<Vec<(u32, u32)>>);

fn parse_senc(data: &[u8], senc: &BoxSpan, declared_iv: u8, expected: usize) -> SencEntries {
    let Some(flags) = be32(data, senc.body).map(|w| w & FULLBOX_FLAGS_MASK) else {
        return SencEntries::default();
    };
    let Some(count) = be32(data, senc.body + 4).map(|c| c as usize) else {
        return SencEntries::default();
    };
    if count > MAX_FRAGMENT_SAMPLES.min(expected.max(1) * 2) {
        log::warn!("cenc: ignoring senc with implausible count {count}");
        return SencEntries::default();
    }
    let Some(raw) = data.get(senc.body + 8..senc.end) else {
        return SencEntries::default();
    };
    let has_subsamples = flags & SENC_SUBSAMPLES != 0;

    for iv_size in [declared_iv, 16, 8, 0] {
        if let Some(parsed) = fit_senc_entries(raw, iv_size, count, has_subsamples) {
            return parsed;
        }
    }
    log::warn!("cenc: could not fit senc box with any IV size");
    SencEntries::default()
}

fn fit_senc_entries(
    raw: &[u8],
    iv_size: u8,
    count: usize,
    has_subsamples: bool,
) -> Option<SencEntries> {
    let mut at = 0usize;
    let mut ivs = Vec::with_capacity(count);
    let mut subs = Vec::with_capacity(count);

    for _ in 0..count {
        if iv_size > 0 {
            let take = usize::from(iv_size).min(16);
            let mut iv = [0u8; 16];
            iv[..take].copy_from_slice(raw.get(at..at + take)?);
            ivs.push(iv);
            at += usize::from(iv_size);
        }
        if has_subsamples {
            let sub_count = usize::from(be16(raw, at)?);
            at += 2;
            let mut runs = Vec::with_capacity(sub_count);
            for _ in 0..sub_count {
                let clear = u32::from(be16(raw, at)?);
                let encrypted = be32(raw, at + 2)?;
                runs.push((clear, encrypted));
                at += 6;
            }
            subs.push(runs);
        } else {
            subs.push(Vec::new());
        }
    }

    (at == raw.len()).then_some((ivs, subs))
}

/// Fallback when a fragment provides `saiz` + `saio` auxiliary info without a `senc` box.
fn parse_saiz_saio(
    data: &[u8],
    traf: &BoxSpan,
    _origin: usize,
    base_in_window: Option<u64>,
    iv_size: u8,
) -> Option<SencEntries> {
    let saiz = find_child(data, traf.body, traf.end, b"saiz")?;
    let saio = find_child(data, traf.body, traf.end, b"saio")?;

    let saiz_flags = be32(data, saiz.body)? & FULLBOX_FLAGS_MASK;
    let mut at = saiz.body + 4;
    if saiz_flags & AUX_INFO_TYPE_PRESENT != 0 {
        at += 8;
    }
    let default_info_size = *data.get(at)?;
    let sample_count = be32(data, at + 1)? as usize;
    if sample_count > MAX_FRAGMENT_SAMPLES {
        return None;
    }
    at += 5;

    let mut info_sizes = Vec::with_capacity(sample_count);
    if default_info_size == 0 {
        let raw_sizes = data.get(at..at + sample_count)?;
        info_sizes.extend_from_slice(raw_sizes);
    } else {
        info_sizes.resize(sample_count, default_info_size);
    }

    let saio_ver_flags = be32(data, saio.body)?;
    let saio_version = (saio_ver_flags >> 24) as u8;
    let saio_flags = saio_ver_flags & FULLBOX_FLAGS_MASK;
    let mut sat = saio.body + 4;
    if saio_flags & AUX_INFO_TYPE_PRESENT != 0 {
        sat += 8;
    }
    let entry_count = be32(data, sat)?;
    sat += 4;
    if entry_count == 0 {
        return None;
    }
    let aux_offset: i64 = if saio_version == 0 {
        i64::from(be32(data, sat)? as i32)
    } else {
        i64::try_from(be64(data, sat)?).ok()?
    };
    let aux_start = usize::try_from(base_in_window?.checked_add_signed(aux_offset)?).ok()?;

    let mut cursor = aux_start;
    let mut ivs = Vec::with_capacity(sample_count);
    let mut subs = Vec::with_capacity(sample_count);
    let iv_len = usize::from(iv_size).min(16);

    for sz in info_sizes {
        let entry_len = usize::from(sz);
        let entry = data.get(cursor..cursor + entry_len)?;
        cursor += entry_len;
        if entry_len < iv_len {
            return None;
        }
        let mut iv = [0u8; 16];
        iv[..iv_len].copy_from_slice(&entry[..iv_len]);
        ivs.push(iv);

        if entry_len > iv_len + 2 {
            let sub_count = usize::from(be16(entry, iv_len)?);
            let mut sub_at = iv_len + 2;
            let mut runs = Vec::with_capacity(sub_count);
            for _ in 0..sub_count {
                let clear = u32::from(be16(entry, sub_at)?);
                let enc = be32(entry, sub_at + 2)?;
                runs.push((clear, enc));
                sub_at += 6;
            }
            subs.push(runs);
        } else {
            subs.push(Vec::new());
        }
    }

    Some((ivs, subs))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn boxed(kind: &[u8; 4], body: &[u8]) -> Vec<u8> {
        let mut out = ((body.len() + 8) as u32).to_be_bytes().to_vec();
        out.extend_from_slice(kind);
        out.extend_from_slice(body);
        out
    }

    fn joined(parts: &[Vec<u8>]) -> Vec<u8> {
        parts.iter().flatten().copied().collect()
    }

    fn synthetic_init(track_id: u32, iv_size: u8, scheme: &[u8; 4]) -> Vec<u8> {
        let mut schm_body = vec![0u8; 4];
        schm_body.extend_from_slice(scheme);
        schm_body.extend_from_slice(&0x0001_0000u32.to_be_bytes());
        let schm = boxed(b"schm", &schm_body);

        let tenc = boxed(b"tenc", &[0, 0, 0, 0, 0, 0, 1, iv_size, 0]);
        let schi = boxed(b"schi", &tenc);
        let sinf = boxed(b"sinf", &joined(&[schm, schi]));

        let mut enca_body = vec![0u8; 28];
        enca_body.extend_from_slice(&sinf);
        let enca = boxed(ENCA, &enca_body);

        let mut stsd_body = vec![0, 0, 0, 0, 0, 0, 0, 1];
        stsd_body.extend_from_slice(&enca);
        let stbl = boxed(b"stbl", &boxed(b"stsd", &stsd_body));
        let minf = boxed(b"minf", &stbl);

        let mut mdhd_body = vec![0u8; 12];
        mdhd_body.extend_from_slice(&44_100u32.to_be_bytes());
        mdhd_body.extend_from_slice(&[0u8; 8]);
        let mdia = boxed(b"mdia", &joined(&[boxed(b"mdhd", &mdhd_body), minf]));

        let mut tkhd_body = vec![0u8; 12];
        tkhd_body.extend_from_slice(&track_id.to_be_bytes());
        tkhd_body.extend_from_slice(&[0u8; 60]);
        let trak = boxed(b"trak", &joined(&[boxed(b"tkhd", &tkhd_body), mdia]));
        let moov = boxed(b"moov", &trak);

        joined(&[boxed(b"ftyp", b"isom"), moov])
    }

    fn synthetic_fragment(track_id: u32, sizes: &[u32], ivs: &[[u8; 16]], iv_bytes: usize) -> Vec<u8> {
        let mut tfhd_body = vec![0, 0, 0, 0];
        tfhd_body.extend_from_slice(&track_id.to_be_bytes());
        let tfhd = boxed(b"tfhd", &tfhd_body);

        let mut tfdt_body = vec![0u8; 4];
        tfdt_body.extend_from_slice(&88_200u32.to_be_bytes());
        let tfdt = boxed(b"tfdt", &tfdt_body);

        let mut senc_body = vec![0, 0, 0, 0];
        senc_body.extend_from_slice(&(ivs.len() as u32).to_be_bytes());
        for iv in ivs {
            senc_body.extend_from_slice(&iv[..iv_bytes]);
        }
        let senc = boxed(b"senc", &senc_body);

        let mut trun_body = vec![0, 0, 0x02, 0x01];
        trun_body.extend_from_slice(&(sizes.len() as u32).to_be_bytes());
        let offset_at = trun_body.len();
        trun_body.extend_from_slice(&0u32.to_be_bytes());
        for size in sizes {
            trun_body.extend_from_slice(&size.to_be_bytes());
        }
        let trun = boxed(b"trun", &trun_body);
        let trun_len = trun.len();

        let mut moof = boxed(b"moof", &boxed(b"traf", &joined(&[tfhd, tfdt, senc, trun])));
        let offset = (moof.len() + 8) as u32;
        let at = moof.len() - trun_len + 8 + offset_at;
        moof[at..at + 4].copy_from_slice(&offset.to_be_bytes());

        let payload: Vec<u8> = sizes
            .iter()
            .enumerate()
            .flat_map(|(idx, &sz)| vec![idx as u8 + 1; sz as usize])
            .collect();
        joined(&[moof, boxed(b"mdat", &payload)])
    }

    #[test]
    fn parses_and_relabels_cenc_init_segment() {
        let mut data = synthetic_init(7, 16, b"cenc");
        let init = read_init(&data).unwrap();
        assert_eq!(init.end, data.len());
        assert_eq!(init.timescale, 44_100);
        assert_eq!(
            init.tracks,
            vec![EncryptedTrack {
                track_id:    7,
                iv_size:     16,
                constant_iv: None,
            }]
        );
        assert_eq!(&data[init.entries[0] + 4..init.entries[0] + 8], ENCA);

        relabel_enca_to_mp4a(&mut data, &init);
        assert_eq!(&data[init.entries[0] + 4..init.entries[0] + 8], MP4A);
        assert_eq!(read_init(&data).unwrap().end, init.end);
    }

    #[test]
    fn rejects_truncated_init_and_skips_cbcs_track() {
        let data = synthetic_init(1, 16, b"cenc");
        assert!(read_init(&data[..data.len() - 10]).is_none());
        assert!(read_init(&[]).is_none());

        let cbcs = synthetic_init(1, 16, b"cbcs");
        let parsed = read_init(&cbcs).unwrap();
        assert!(parsed.tracks.is_empty());
        assert!(parsed.entries.is_empty());
    }

    #[test]
    fn parses_fragment_samples_with_16_and_8_byte_ivs() {
        let init = synthetic_init(1, 16, b"cenc");
        let ivs = [[1u8; 16], [2u8; 16], [3u8; 16]];
        let sizes = [12u32, 24, 36];
        let full = joined(&[init.clone(), synthetic_fragment(1, &sizes, &ivs, 16)]);

        let parsed_init = read_init(&full).unwrap();
        let FragmentStep::Fragment {
            samples,
            next,
            decode_ticks,
        } = read_fragment(&full[parsed_init.end..], parsed_init.end, &parsed_init.tracks)
        else {
            panic!("expected whole fragment");
        };
        assert_eq!(next, full.len());
        assert_eq!(decode_ticks, Some(88_200));
        assert_eq!(samples.len(), 3);
        for (i, sample) in samples.iter().enumerate() {
            assert_eq!(sample.iv, ivs[i]);
            assert_eq!(&full[sample.start..sample.end()], vec![i as u8 + 1; sample.len]);
        }

        // Fragment with 8-byte IVs when tenc declared 16 (inferred fit).
        let iv8_frag = synthetic_fragment(1, &[16, 16], &[[9u8; 16], [8u8; 16]], 8);
        let full8 = joined(&[init, iv8_frag]);
        let FragmentStep::Fragment { samples, .. } =
            read_fragment(&full8[parsed_init.end..], parsed_init.end, &parsed_init.tracks)
        else {
            panic!("expected whole 8-byte iv fragment");
        };
        assert_eq!(samples[0].iv[..8], [9u8; 8]);
        assert_eq!(samples[0].iv[8..], [0u8; 8]);
        assert_eq!(samples[1].iv[..8], [8u8; 8]);
        assert_eq!(samples[1].iv[8..], [0u8; 8]);
    }

    #[test]
    fn parses_senc_subsamples_and_handles_partial_and_other_boxes() {
        let mut senc_body = vec![0, 0, 0, SENC_SUBSAMPLES as u8];
        senc_body.extend_from_slice(&1u32.to_be_bytes());
        senc_body.extend_from_slice(&[7u8; 16]);
        senc_body.extend_from_slice(&2u16.to_be_bytes());
        senc_body.extend_from_slice(&10u16.to_be_bytes());
        senc_body.extend_from_slice(&32u32.to_be_bytes());
        senc_body.extend_from_slice(&4u16.to_be_bytes());
        senc_body.extend_from_slice(&16u32.to_be_bytes());
        let data = boxed(b"senc", &senc_body);
        let senc_box = read_box(&data, 0).unwrap();
        let (ivs, subs) = parse_senc(&data, &senc_box, 16, 1);
        assert_eq!(ivs, vec![[7u8; 16]]);
        assert_eq!(subs, vec![vec![(10, 32), (4, 16)]]);

        let init = synthetic_init(1, 16, b"cenc");
        let full = joined(&[
            init.clone(),
            synthetic_fragment(1, &[16, 16], &[[1u8; 16], [2u8; 16]], 16),
        ]);
        let parsed = read_init(&full).unwrap();
        assert_eq!(
            read_fragment(&full[parsed.end..full.len() - 4], parsed.end, &parsed.tracks),
            FragmentStep::Partial
        );

        let skipped = joined(&[boxed(b"free", &[0u8; 12]), boxed(b"moof", &[])]);
        assert_eq!(
            read_fragment(&skipped, 0, &[]),
            FragmentStep::Other { next: 20 }
        );
    }

    #[test]
    fn parses_saiz_saio_fallback_when_senc_absent() {
        let init = synthetic_init(1, 8, b"cenc");
        let parsed_init = read_init(&init).unwrap();

        let mut tfhd_body = vec![0, 0, 0, 0];
        tfhd_body.extend_from_slice(&1u32.to_be_bytes());
        let tfhd = boxed(b"tfhd", &tfhd_body);

        // saiz: default_info_size = 8, count = 2
        let mut saiz_body = vec![0, 0, 0, 0, 8];
        saiz_body.extend_from_slice(&2u32.to_be_bytes());
        let saiz = boxed(b"saiz", &saiz_body);

        // saio: version 0, 1 offset pointing right at the start of mdat body (before samples)
        let mut saio_body = vec![0, 0, 0, 0];
        saio_body.extend_from_slice(&1u32.to_be_bytes());
        let saio_offset_at = saio_body.len();
        saio_body.extend_from_slice(&0u32.to_be_bytes());
        let saio = boxed(b"saio", &saio_body);
        let saio_len = saio.len();

        let mut trun_body = vec![0, 0, 0x02, 0x01];
        trun_body.extend_from_slice(&2u32.to_be_bytes());
        let trun_offset_at = trun_body.len();
        trun_body.extend_from_slice(&0u32.to_be_bytes());
        trun_body.extend_from_slice(&10u32.to_be_bytes());
        trun_body.extend_from_slice(&12u32.to_be_bytes());
        let trun = boxed(b"trun", &trun_body);
        let trun_len = trun.len();

        let mut moof = boxed(b"moof", &boxed(b"traf", &joined(&[tfhd, saiz, saio, trun])));
        let mdat_body_off = (moof.len() + 8) as u32;
        // Put saio at mdat_body_off, trun samples right after 16 bytes of aux IVs.
        let saio_patch_at = moof.len() - trun_len - saio_len + 8 + saio_offset_at;
        moof[saio_patch_at..saio_patch_at + 4].copy_from_slice(&mdat_body_off.to_be_bytes());
        let trun_patch_at = moof.len() - trun_len + 8 + trun_offset_at;
        moof[trun_patch_at..trun_patch_at + 4]
            .copy_from_slice(&(mdat_body_off + 16).to_be_bytes());

        let mut mdat_payload = Vec::new();
        mdat_payload.extend_from_slice(&[0xAAu8; 8]);
        mdat_payload.extend_from_slice(&[0xBBu8; 8]);
        mdat_payload.extend_from_slice(&[1u8; 10]);
        mdat_payload.extend_from_slice(&[2u8; 12]);
        let frag = joined(&[moof, boxed(b"mdat", &mdat_payload)]);
        let full = joined(&[init, frag]);

        let FragmentStep::Fragment { samples, .. } =
            read_fragment(&full[parsed_init.end..], parsed_init.end, &parsed_init.tracks)
        else {
            panic!("expected fragment via saiz/saio");
        };
        assert_eq!(samples.len(), 2);
        assert_eq!(samples[0].iv[..8], [0xAAu8; 8]);
        assert_eq!(samples[1].iv[..8], [0xBBu8; 8]);
        assert_eq!(&full[samples[0].start..samples[0].end()], &[1u8; 10]);
        assert_eq!(&full[samples[1].start..samples[1].end()], &[2u8; 12]);
    }

    #[test]
    fn parses_tfhd_default_sample_size_and_64bit_box_header() {
        let init = synthetic_init(1, 16, b"cenc");
        let parsed_init = read_init(&init).unwrap();

        // tfhd with default-sample-size flag (0x10) = 14 bytes
        let mut tfhd_body = vec![0, 0, 0, TFHD_DEFAULT_SIZE as u8];
        tfhd_body.extend_from_slice(&1u32.to_be_bytes());
        tfhd_body.extend_from_slice(&14u32.to_be_bytes());
        let tfhd = boxed(b"tfhd", &tfhd_body);

        let mut senc_body = vec![0, 0, 0, 0];
        senc_body.extend_from_slice(&2u32.to_be_bytes());
        senc_body.extend_from_slice(&[5u8; 16]);
        senc_body.extend_from_slice(&[6u8; 16]);
        let senc = boxed(b"senc", &senc_body);

        // trun with data-offset (0x01) only, no per-sample sizes
        let mut trun_body = vec![0, 0, 0x00, 0x01];
        trun_body.extend_from_slice(&2u32.to_be_bytes());
        let offset_at = trun_body.len();
        trun_body.extend_from_slice(&0u32.to_be_bytes());
        let trun = boxed(b"trun", &trun_body);
        let trun_len = trun.len();

        let mut moof = boxed(b"moof", &boxed(b"traf", &joined(&[tfhd, senc, trun])));
        let mdat_hdr_size = 16u32; // 64-bit extended size mdat header
        let offset = (moof.len() as u32) + mdat_hdr_size;
        let at = moof.len() - trun_len + 8 + offset_at;
        moof[at..at + 4].copy_from_slice(&offset.to_be_bytes());

        let mut mdat64 = 1u32.to_be_bytes().to_vec();
        mdat64.extend_from_slice(b"mdat");
        mdat64.extend_from_slice(&((16 + 28) as u64).to_be_bytes());
        mdat64.extend_from_slice(&[0x11u8; 14]);
        mdat64.extend_from_slice(&[0x22u8; 14]);

        let full = joined(&[init, moof, mdat64]);
        let FragmentStep::Fragment { samples, .. } =
            read_fragment(&full[parsed_init.end..], parsed_init.end, &parsed_init.tracks)
        else {
            panic!("expected fragment with tfhd default size and 64-bit mdat");
        };
        assert_eq!(samples.len(), 2);
        assert_eq!(samples[0].len, 14);
        assert_eq!(samples[1].len, 14);
        assert_eq!(&full[samples[0].start..samples[0].end()], &[0x11u8; 14]);
        assert_eq!(&full[samples[1].start..samples[1].end()], &[0x22u8; 14]);
    }
}
