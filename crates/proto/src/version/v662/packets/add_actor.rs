use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, gamepacket};
use vek::{Vec2, Vec3};

#[gamepacket(id = 13)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct AddActorPacket<V: ProtoVersion> {
    pub target_actor_id: V::ActorUniqueID,
    pub target_runtime_id: V::ActorRuntimeID,
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

    pub attributes: Vec<AttributeEntry>,

    pub actor_data: Vec<V::DataItem>, // TODO: verify vec_repr & vec_endianness
    pub synced_properties: V::PropertySyncData,

    pub actor_links: Vec<V::ActorLink>,
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
