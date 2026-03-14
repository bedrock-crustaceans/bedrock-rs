use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 32)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct MobArmorEquipmentPacket<V: ProtoVersion> {
    pub target_runtime_id: V::ActorRuntimeID,
    pub head: V::NetworkItemStackDescriptor,
    pub torso: V::NetworkItemStackDescriptor,
    pub legs: V::NetworkItemStackDescriptor,
    pub feet: V::NetworkItemStackDescriptor,
}
