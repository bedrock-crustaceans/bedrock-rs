use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 68)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct MapInfoRequestPacket<V: ProtoVersion> {
    pub map_unique_id: V::ActorUniqueID,
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