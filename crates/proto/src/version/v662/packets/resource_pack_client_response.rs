use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 8)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ResourcePackClientResponsePacket<V: ProtoVersion> {
    pub response: V::ResourcePackResponse,
    #[vec_repr(u16)]
    #[vec_endianness(le)]
    pub downloading_packs: Vec<String>,
}