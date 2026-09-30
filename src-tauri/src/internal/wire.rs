//! just enough protobuf wire format to talk to endpoints whose .proto files
//! librespot-protocol ships incomplete (canvaz-cache has no request message and
//! the response lost its repeated field). four wire types, no reflection.

pub fn put_varint(buf: &mut Vec<u8>, mut v: u64) {
    loop {
        let byte = (v & 0x7f) as u8;
        v >>= 7;
        if v == 0 {
            buf.push(byte);
            return;
        }
        buf.push(byte | 0x80);
    }
}

/// length-delimited field (strings, bytes, embedded messages)
pub fn put_bytes(buf: &mut Vec<u8>, field: u32, data: &[u8]) {
    put_varint(buf, ((field as u64) << 3) | 2);
    put_varint(buf, data.len() as u64);
    buf.extend_from_slice(data);
}

#[derive(Debug, PartialEq)]
pub enum Value<'a> {
    Varint(u64),
    Fixed64(u64),
    Bytes(&'a [u8]),
    Fixed32(u32),
}

fn varint(data: &[u8], pos: &mut usize) -> Option<u64> {
    let mut out = 0u64;
    for shift in (0..64).step_by(7) {
        let b = *data.get(*pos)?;
        *pos += 1;
        out |= ((b & 0x7f) as u64) << shift;
        if b & 0x80 == 0 {
            return Some(out);
        }
    }
    None
}

/// every top-level (field number, value) in a message, in wire order. `None`
/// on a truncated/garbled buffer rather than a half-read list.
pub fn fields(data: &[u8]) -> Option<Vec<(u32, Value<'_>)>> {
    let mut out = Vec::new();
    let mut pos = 0;
    while pos < data.len() {
        let tag = varint(data, &mut pos)?;
        let field = (tag >> 3) as u32;
        let value = match tag & 7 {
            0 => Value::Varint(varint(data, &mut pos)?),
            1 => {
                let b = data.get(pos..pos + 8)?;
                pos += 8;
                Value::Fixed64(u64::from_le_bytes(b.try_into().ok()?))
            }
            2 => {
                let len = varint(data, &mut pos)? as usize;
                let b = data.get(pos..pos.checked_add(len)?)?;
                pos += len;
                Value::Bytes(b)
            }
            5 => {
                let b = data.get(pos..pos + 4)?;
                pos += 4;
                Value::Fixed32(u32::from_le_bytes(b.try_into().ok()?))
            }
            _ => return None,
        };
        out.push((field, value));
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let mut inner = Vec::new();
        put_bytes(&mut inner, 1, b"spotify:track:x");
        let mut outer = Vec::new();
        put_bytes(&mut outer, 1, &inner);
        put_varint(&mut outer, (2 << 3) as u64);
        put_varint(&mut outer, 300);

        let f = fields(&outer).unwrap();
        assert_eq!(f.len(), 2);
        let Value::Bytes(b) = f[0].1 else { panic!() };
        assert_eq!(fields(b).unwrap()[0], (1, Value::Bytes(b"spotify:track:x")));
        assert_eq!(f[1], (2, Value::Varint(300)));
    }

    #[test]
    fn rejects_truncation() {
        assert!(fields(&[0x0a, 0x05, b'a']).is_none());
    }
}
