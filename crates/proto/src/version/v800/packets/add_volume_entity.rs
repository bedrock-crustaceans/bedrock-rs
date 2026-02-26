use super::super::types::{EntityNetID, NetworkBlockPosition};
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 166)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct AddVolumeEntityPacket {
    pub entity_network_id: EntityNetID,
    #[nbt]
    pub components: nbtx::Value, // TODO: NBT Structure
    pub json_identifier: String,
    pub instance_name: String,
    pub min_bounds: NetworkBlockPosition,
    pub max_bounds: NetworkBlockPosition,
    #[endianness(var)]
    pub dimension_type: i32,
    pub engine_version: String,
}