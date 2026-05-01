use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Debug, Clone)]
#[enum_repr(i32)]
#[enum_endianness(var)]
pub enum EditorWorldType {
    NotEditor = 0,
    Project = 1,
    TestLevel = 2,
}
