use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 148)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemStackResponsePacket<V: ProtoVersion> {
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub responses: Vec<V::ItemStackResponseInfo>,
}
