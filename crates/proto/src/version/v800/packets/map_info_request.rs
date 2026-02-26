use super::super::types::ActorUniqueID;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 68)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct MapInfoRequestPacket {
    pub map_unique_id: ActorUniqueID,
    #[vec_repr(u32)]
    #[vec_endianness(le)]
    pub client_pixels_list: Vec<ClientPixelsListEntry>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientPixelsListEntry {
    #[endianness(le)]
    pub pixel: u32,
    #[endianness(le)]
    pub index: u16,
}
