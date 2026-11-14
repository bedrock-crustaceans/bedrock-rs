use crate::ProtoCodecVAR;
use std::convert::TryInto;
use std::io::{Read, Write};

use crate::ProtoCodec;
use crate::error::ProtoCodecError;

impl ProtoCodec for String {
    fn serialize<W: Write>(&self, buf: &mut W) -> Result<(), ProtoCodecError>
    where
        Self: Sized,
    {
        let len = self.len().try_into()?;

        <u32 as ProtoCodecVAR>::serialize(&len, buf)?;
        buf.write_all(self.as_bytes())?;

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError>
    where
        Self: Sized,
    {
        let len: usize = <u32 as ProtoCodecVAR>::deserialize(stream)?.try_into()?;

        let mut string_buf = Vec::new();
        stream
            .by_ref()
            .take(len as u64)
            .read_to_end(&mut string_buf)?;
        if string_buf.len() != len {
            return Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof).into());
        }

        Ok(String::from_utf8(string_buf)?)
    }

    fn size_hint(&self) -> usize {
        // 4 = u32 String size
        self.len() + 4
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn string_round_trips() {
        let bytes = [0x05, b'h', b'e', b'l', b'l', b'o'];
        let string = String::deserialize(&mut Cursor::new(bytes)).unwrap();
        assert_eq!(string, "hello");
        let mut encoded = Vec::new();
        string.serialize(&mut encoded).unwrap();
        assert_eq!(encoded, bytes);
    }

    #[test]
    fn string_shorter_than_its_prefix_is_an_eof_error() {
        let result = String::deserialize(&mut Cursor::new([0x05, b'h', b'i']));
        assert!(
            matches!(result, Err(ProtoCodecError::IOError(ref e)) if e.kind() == std::io::ErrorKind::UnexpectedEof),
            "expected EOF, got {result:?}"
        );
    }
}
