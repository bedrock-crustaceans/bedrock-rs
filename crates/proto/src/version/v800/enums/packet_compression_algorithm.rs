use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u16)]
#[enum_endianness(le)]
#[repr(u16)]
pub enum PacketCompressionAlgorithm {
    ZLib = 0,
    Snappy = 1,
    None = 0xffff,
}