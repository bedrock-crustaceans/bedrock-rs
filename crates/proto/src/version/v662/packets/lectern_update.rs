use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 125)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct LecternUpdatePacket<V: ProtoVersion> {
    pub new_page_to_show: i8,
    pub total_pages: i8,
    pub position_of_lectern_to_update: V::NetworkBlockPosition,
}