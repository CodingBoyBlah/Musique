//! the fragment index (`sidx`) at the front of a fragmented mp4.
//!
//! youtube's audio is a fragmented mp4: a short header, an index, then ~800
//! ten-second fragments for a two hour episode. symphonia's mp4 reader, given
//! a seekable file, walks the header of every fragment before it plays a
//! thing - over a streaming download that means waiting for the whole file.
//! the index already says where every fragment starts and when, so playback
//! opens without the walk and a seek reopens straight at the right fragment.

use std::io::{Read, Seek, SeekFrom};

#[derive(Debug, Clone, PartialEq)]
pub struct Segments {
    /// where the header (ftyp + moov + sidx) ends and fragments begin
    pub init_end: u64,
    /// each fragment's (byte offset, start time in ms), in order
    pub starts:   Vec<(u64, u64)>,
}

impl Segments {
    /// the fragment holding `ms`
    pub fn at(&self, ms: u64) -> (u64, u64) {
        let i = self.starts.partition_point(|&(_, t)| t <= ms).saturating_sub(1);
        self.starts[i]
    }
}

fn be(b: &[u8]) -> u64 {
    b.iter().fold(0, |acc, &x| (acc << 8) | x as u64)
}

/// read the index, if the file starts ftyp/moov/.../sidx. None for anything
/// else (plain mp4, mp3...)
pub fn read_index<R: Read + Seek>(r: &mut R, len: u64) -> Option<Segments> {
    let mut pos = 0u64;
    for _ in 0..8 {
        r.seek(SeekFrom::Start(pos)).ok()?;
        let mut h = [0u8; 8];
        r.read_exact(&mut h).ok()?;
        let mut size = be(&h[..4]);
        let kind = &h[4..8];
        let mut header = 8;
        if size == 1 {
            let mut big = [0u8; 8];
            r.read_exact(&mut big).ok()?;
            size = be(&big);
            header = 16;
        }
        if size < header || pos + size > len {
            return None;
        }
        match kind {
            // anything that isn't an mp4 from its first bytes (an mp3...)
            _ if pos == 0 && kind != b"ftyp" => return None,
            b"sidx" => {
                let mut body = vec![0u8; (size - header) as usize];
                r.read_exact(&mut body).ok()?;
                return parse_sidx(&body, pos + size);
            }
            // fragments or media before any index: nothing to go on
            b"moof" | b"mdat" => return None,
            _ => pos += size,
        }
    }
    None
}

/// `body` is the sidx box after its 8-byte header; `end` is where it ends
fn parse_sidx(body: &[u8], end: u64) -> Option<Segments> {
    let version = *body.first()?;
    // version/flags, reference_ID, timescale
    let timescale = be(body.get(8..12)?);
    let (earliest, first_offset, rest) = if version == 0 {
        (be(body.get(12..16)?), be(body.get(16..20)?), 20)
    } else {
        (be(body.get(12..20)?), be(body.get(20..28)?), 28)
    };
    if timescale == 0 {
        return None;
    }
    let count = be(body.get(rest + 2..rest + 4)?) as usize;
    let mut starts = Vec::with_capacity(count);
    let mut byte = end + first_offset;
    let mut t = earliest;
    for i in 0..count {
        let e = body.get(rest + 4 + i * 12..rest + 4 + i * 12 + 12)?;
        let reference = be(&e[..4]);
        // a reference to another index rather than to media: not a layout
        // this handles
        if reference & 0x8000_0000 != 0 {
            return None;
        }
        starts.push((byte, t * 1000 / timescale));
        byte += reference & 0x7fff_ffff;
        t += be(&e[4..8]);
    }
    (!starts.is_empty()).then_some(Segments { init_end: end, starts })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn boxed(kind: &[u8], body: &[u8]) -> Vec<u8> {
        let mut v = ((body.len() + 8) as u32).to_be_bytes().to_vec();
        v.extend_from_slice(kind);
        v.extend_from_slice(body);
        v
    }

    fn sidx(refs: &[(u32, u32)]) -> Vec<u8> {
        let mut b = vec![0, 0, 0, 0]; // version 0, flags
        b.extend_from_slice(&1u32.to_be_bytes()); // reference_ID
        b.extend_from_slice(&1000u32.to_be_bytes()); // timescale
        b.extend_from_slice(&0u32.to_be_bytes()); // earliest
        b.extend_from_slice(&0u32.to_be_bytes()); // first_offset
        b.extend_from_slice(&0u16.to_be_bytes());
        b.extend_from_slice(&(refs.len() as u16).to_be_bytes());
        for &(size, dur) in refs {
            b.extend_from_slice(&size.to_be_bytes());
            b.extend_from_slice(&dur.to_be_bytes());
            b.extend_from_slice(&0x9000_0000u32.to_be_bytes());
        }
        boxed(b"sidx", &b)
    }

    #[test]
    fn reads_the_fragment_table() {
        let mut file = boxed(b"ftyp", &[0; 16]);
        file.extend(boxed(b"moov", &[0; 100]));
        file.extend(sidx(&[(500, 10_000), (400, 10_000), (300, 5_000)]));
        let init_end = file.len() as u64;
        file.extend(vec![0; 1200]);
        let len = file.len() as u64;
        let seg = read_index(&mut Cursor::new(file), len).unwrap();
        assert_eq!(seg.init_end, init_end);
        assert_eq!(seg.starts, vec![(init_end, 0), (init_end + 500, 10_000), (init_end + 900, 20_000)]);
        assert_eq!(seg.at(0), (init_end, 0));
        assert_eq!(seg.at(12_345), (init_end + 500, 10_000));
        assert_eq!(seg.at(99_999), (init_end + 900, 20_000));
    }

    #[test]
    fn no_index_is_none() {
        let mut file = boxed(b"ftyp", &[0; 16]);
        file.extend(boxed(b"moov", &[0; 100]));
        file.extend(boxed(b"mdat", &[0; 100]));
        let len = file.len() as u64;
        assert!(read_index(&mut Cursor::new(file), len).is_none());
        let mp3 = vec![0xff, 0xfb, 0x90, 0x64, 0, 0, 0, 0, 0, 0];
        assert!(read_index(&mut Cursor::new(mp3), 10).is_none());
    }
}
