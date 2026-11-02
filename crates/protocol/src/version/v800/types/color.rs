use bedrock_protocol_core::error::ProtoCodecError;
use bedrock_protocol_core::{ProtoCodec, ProtoCodecLE};
use std::io::{Read, Write};

#[derive(Clone, Debug)]
pub struct Color {
    r: i8,
    g: i8,
    b: i8,
    a: i8,
}

impl ProtoCodec for Color {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        <i32 as ProtoCodecLE>::serialize(
            &(i32::from(self.a)
                | (i32::from(self.r) << 8)
                | (i32::from(self.g) << 16)
                | (i32::from(self.b) << 24)),
            stream,
        )?;
        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let packed = <i32 as ProtoCodecLE>::deserialize(stream)?;
        let [alpha, red, green, blue] = packed.to_le_bytes().map(|byte| i8::from_le_bytes([byte]));
        Ok(Color {
            r: red,
            g: green,
            b: blue,
            a: alpha,
        })
    }

    fn size_hint(&self) -> usize {
        size_of::<i32>()
    }
}
