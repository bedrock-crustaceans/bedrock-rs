use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, gamepacket};
use uuid::Uuid;
use vek::{Vec2, Vec3};

#[gamepacket(id = 12)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct AddPlayerPacket<V: ProtoVersion> {
    pub uuid: Uuid,
    pub player_name: String,
    pub target_runtime_id: V::ActorRuntimeID,
    pub platform_chat_id: String,
    #[endianness(le)]
    pub position: Vec3<f32>,
    #[endianness(le)]
    pub velocity: Vec3<f32>,
    #[endianness(le)]
    pub rotation: Vec2<f32>,
    #[endianness(le)]
    pub y_head_rotation: f32,
    pub carried_item: V::NetworkItemStackDescriptor,
    pub player_game_type: V::GameType,

    pub entity_data: Vec<V::DataItem>, // VERIFY: vec_repr & vec_endianness
    pub synced_properties: V::PropertySyncData,
    pub abilities_data: V::SerializedAbilitiesData,

    pub actor_links: Vec<V::ActorLink>,
    pub device_id: String,
    pub build_platform: V::BuildPlatform,
}
