use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 32)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct MobArmorEquipmentPacket<V: ProtoVersion> {
    pub target_runtime_id: V::ActorRuntimeID,
    pub head: V::NetworkItemStackDescriptor,
    pub torso: V::NetworkItemStackDescriptor,
    pub legs: V::NetworkItemStackDescriptor,
    pub feet: V::NetworkItemStackDescriptor,
}