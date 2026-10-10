use std::collections::HashMap;
use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 166, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct AddVolumeEntityPacket<V: ProtoVersion> {
    pub entity_network_id: V::EntityNetID,
    #[nbt]
    pub components: HashMap<String, nbtx::Value>,
    pub json_identifier: String,
    pub instance_name: String,
    pub min_bounds: V::NetworkBlockPosition,
    pub max_bounds: V::NetworkBlockPosition,
    #[endianness(var)]
    pub dimension_type: i32,
    pub engine_version: String,
}
