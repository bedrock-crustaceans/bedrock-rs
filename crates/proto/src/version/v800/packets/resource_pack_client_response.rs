use super::super::enums::ResourcePackResponse;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 8)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ResourcePackClientResponsePacket {
    pub response: ResourcePackResponse,
    #[vec_repr(u16)]
    #[vec_endianness(le)]
    pub downloading_packs: Vec<String>,
}