//! Framing between the app and the CDM host process.
//!
//! Both directions carry `[tag: u8][len: u32 LE][payload]`. A request's tag is an opcode; a
//! response's tag is a status. The host speaks first, with a hello frame: `OK` and
//! `"<PROTOCOL>\n<module version>\n<interface version>"`, or `FAILED` and why it could not
//! open the module.

use std::io::{self, Read, Write};

/// Bumped whenever a frame changes shape; the app refuses a host that answers otherwise.
pub const PROTOCOL: &str = "musique-widevine/1";

/// Largest payload either side accepts.
pub const CEILING: usize = 64 << 20;

// Requests.
/// init data -> `[session_len u16][session][license request]`
pub const GENERATE: u8 = 1;
/// `[session_len u16][session][license]` -> `[usable keys u32]`
pub const UPDATE: u8 = 2;
/// `[kid_len u8][kid][iv 16][count u32][(clear u32, cipher u32) * count][sample]` -> sample
pub const DECRYPT: u8 = 3;
/// `[kid_len u8][kid][n u32]` then n * `[iv 16][count u32][pairs][len u32][sample]`
/// -> every sample, concatenated in order
pub const DECRYPT_MANY: u8 = 4;
/// `[session]` -> empty
pub const CLOSE: u8 = 5;

// Responses.
pub const OK: u8 = 0;
pub const FAILED: u8 = 1;
/// A decrypt found no usable key for the key id (no license yet, or it expired).
pub const NO_KEY: u8 = 2;

/// Writes one frame. `payload` may be split in parts so callers need not join them first.
pub fn write_frame(out: &mut impl Write, tag: u8, parts: &[&[u8]]) -> io::Result<()> {
    let len: usize = parts.iter().map(|p| p.len()).sum();
    if len > CEILING {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "frame too large"));
    }
    let mut head = [0u8; 5];
    head[0] = tag;
    head[1..].copy_from_slice(&(len as u32).to_le_bytes());
    out.write_all(&head)?;
    for part in parts {
        out.write_all(part)?;
    }
    out.flush()
}

/// Reads one frame into `payload` (reusing its allocation) and answers its tag.
pub fn read_frame(input: &mut impl Read, payload: &mut Vec<u8>) -> io::Result<u8> {
    let mut head = [0u8; 5];
    input.read_exact(&mut head)?;
    let len = u32::from_le_bytes([head[1], head[2], head[3], head[4]]) as usize;
    if len > CEILING {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "frame too large"));
    }
    payload.clear();
    payload.resize(len, 0);
    input.read_exact(payload)?;
    Ok(head[0])
}

/// A cursor over a request or response payload.
pub struct Reader<'a> {
    bytes: &'a [u8],
}

impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    pub fn take(&mut self, n: usize) -> io::Result<&'a [u8]> {
        if self.bytes.len() < n {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "short frame"));
        }
        let (head, rest) = self.bytes.split_at(n);
        self.bytes = rest;
        Ok(head)
    }

    pub fn u8(&mut self) -> io::Result<u8> {
        Ok(self.take(1)?[0])
    }

    pub fn u16(&mut self) -> io::Result<u16> {
        let b = self.take(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }

    pub fn u32(&mut self) -> io::Result<u32> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    pub fn rest(&mut self) -> &'a [u8] {
        std::mem::take(&mut self.bytes)
    }
}
