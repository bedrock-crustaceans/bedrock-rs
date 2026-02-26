use super::super::types::NetworkBlockPosition;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 125)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct LecternUpdatePacket {
    pub new_page_to_show: i8,
    pub total_pages: i8,
    pub position_of_lectern_to_update: NetworkBlockPosition,
}