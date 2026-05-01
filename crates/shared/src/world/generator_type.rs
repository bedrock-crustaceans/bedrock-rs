use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Debug, Clone)]
#[enum_repr(i32)]
#[enum_endianness(var)]
pub enum GeneratorType {
    Legacy = 0,
    Overworld = 1,
    Flat = 2,
    Nether = 3,
    End = 4,
    Void = 5,
}
