use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 148)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemStackResponsePacket<V: ProtoVersion> {
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub responses: Vec<V::ItemStackResponseInfo>,
}