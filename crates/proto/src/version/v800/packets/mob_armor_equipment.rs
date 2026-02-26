use super::super::types::{ActorRuntimeID, NetworkItemStackDescriptor};
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 32)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct MobArmorEquipmentPacket {
    pub target_runtime_id: ActorRuntimeID,
    pub head: NetworkItemStackDescriptor,
    pub torso: NetworkItemStackDescriptor,
    pub legs: NetworkItemStackDescriptor,
    pub feet: NetworkItemStackDescriptor,
    pub body: NetworkItemStackDescriptor,
}