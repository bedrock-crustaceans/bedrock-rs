use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 31)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct MobEquipmentPacket<V: ProtoVersion> {
    pub target_runtime_id: V::ActorRuntimeID,
    pub item: V::NetworkItemStackDescriptorV2,
    pub slot: i8,
    pub selected_slot: i8,
    pub container_id: V::ContainerID,
}
