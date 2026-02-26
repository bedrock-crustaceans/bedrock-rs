use super::super::types::{ActorLink, ActorRuntimeID, ActorUniqueID, DataItem, PropertySyncData};
use bedrockrs_macros::{gamepacket, ProtoCodec};
use vek::{Vec2, Vec3};

#[gamepacket(id = 13)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct AddActorPacket {
    pub target_actor_id: ActorUniqueID,
    pub target_runtime_id: ActorRuntimeID,
    pub actor_type: String,
    #[endianness(le)]
    pub position: Vec3<f32>,
    #[endianness(le)]
    pub velocity: Vec3<f32>,
    #[endianness(le)]
    pub rotation: Vec2<f32>,
    #[endianness(le)]
    pub y_head_rotation: f32,
    #[endianness(le)]
    pub y_body_rotation: f32,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub attributes: Vec<AttributeEntry>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub actor_data: Vec<DataItem>, // VERIFY: vec_repr & vec_endianness
    pub synced_properties: PropertySyncData,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub actor_links: Vec<ActorLink>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct AttributeEntry {
    pub attribute_name: String,
    #[endianness(le)]
    pub min_value: f32,
    #[endianness(le)]
    pub current_value: f32,
    #[endianness(le)]
    pub max_value: f32,
}
