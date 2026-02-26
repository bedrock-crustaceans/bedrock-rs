use super::super::types::EntityNetID;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 167)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct RemoveVolumeEntityPacket {
    pub entity_network_id: EntityNetID,
    #[endianness(var)]
    pub dimension_type: i32,
}