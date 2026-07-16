use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct SubChunkPos {
    #[endianness(le)]
    pub x: i32,
    #[endianness(le)]
    pub y: i32,
    #[endianness(le)]
    pub z: i32,
}
