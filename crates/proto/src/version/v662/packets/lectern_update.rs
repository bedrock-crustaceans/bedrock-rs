use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 125)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct LecternUpdatePacket<V: ProtoVersion> {
    pub new_page_to_show: i8,
    pub total_pages: i8,
    pub position_of_lectern_to_update: V::NetworkBlockPosition,
}
