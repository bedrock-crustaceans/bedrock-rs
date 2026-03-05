use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::{ProtoCodec, ProtoCodecLE};
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
            &((self.a as i32)
                | ((self.r as i32) << 8)
                | ((self.g as i32) << 16)
                | ((self.b as i32) << 24)),
            stream,
        )?;
        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let v = <i32 as ProtoCodecLE>::deserialize(stream)?;
        Ok(Color {
            a: v as i8,
            r: (v >> 8) as i8,
            g: (v >> 16) as i8,
            b: (v >> 24) as i8,
        })
    }

    fn size_hint(&self) -> usize {
        size_of::<i32>()
    }
}
