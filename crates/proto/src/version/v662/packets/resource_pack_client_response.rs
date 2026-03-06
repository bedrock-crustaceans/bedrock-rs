use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 8)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ResourcePackClientResponsePacket<V: ProtoVersion> {
    pub response: V::ResourcePackResponse,
    #[vec_repr(u16)]
    #[vec_endianness(le)]
    pub downloading_packs: Vec<String>,
}
