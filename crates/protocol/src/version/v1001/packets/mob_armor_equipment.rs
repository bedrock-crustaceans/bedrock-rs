use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 32)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct MobArmorEquipmentPacket<V: ProtoVersion> {
    pub target_runtime_id: V::ActorRuntimeID,
    pub head: V::NetworkItemStackDescriptorV2,
    pub torso: V::NetworkItemStackDescriptorV2,
    pub legs: V::NetworkItemStackDescriptorV2,
    pub feet: V::NetworkItemStackDescriptorV2,
    pub body: V::NetworkItemStackDescriptorV2,
}
