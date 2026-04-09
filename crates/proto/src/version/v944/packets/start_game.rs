use std::collections::HashMap;
use crate::version::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, packet};
use uuid::Uuid;

#[packet(id = 11)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct StartGamePacket<V: ProtoVersion> {
    pub target_actor_id: V::ActorUniqueID,
    pub target_runtime_id: V::ActorRuntimeID,
    pub actor_game_type: V::GameType,
    #[endianness(le)]
    pub position: (f32, f32, f32),
    #[endianness(le)]
    pub rotation: (f32, f32),
    pub settings: V::LevelSettings,
    pub level_id: String,
    pub level_name: String,
    pub template_content_identity: String,
    pub is_trial: bool,
    pub movement_settings: V::SyncedPlayerMovementSettings,
    #[endianness(le)]
    pub current_level_time: u64,
    #[endianness(var)]
    pub enchantment_seed: i32,
    pub block_properties: Vec<BlockProperty>,
    pub multiplayer_correlation_id: String,
    pub enable_item_stack_net_manager: bool,
    pub server_version: String,
    #[nbt]
    pub player_property_data: HashMap<String, nbtx::Value>,
    #[endianness(le)]
    pub server_block_type_registry_checksum: u64,
    pub world_template_id: Uuid,
    pub server_enabled_client_side_generation: bool,
    pub block_network_ids_are_hashes: bool,
    pub network_permissions: V::NetworkPermissions,
    pub server_join_information: Option<ServerJoinInformation>,
    pub server_id: String,
    pub world_id: String,
    pub scenario_id: String,
    pub owner_id: String,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct BlockProperty {
    pub block_name: String,
    #[nbt]
    pub block_definition: HashMap<String, nbtx::Value>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct ServerJoinInformation {
    pub gathering_join_info: Option<GatheringJoinInfo>,
    pub store_entry_point_info: Option<StoreEntryPointInfo>,
    pub presence_info: Option<PresenceInfo>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct GatheringJoinInfo {
    pub experience_id: String,
    pub experience_name: String,
    pub experience_world_id: String,
    pub experience_world_name: String,
    pub creator_id: String,
    pub unknown1: Uuid, // TODO: find out what this is
    pub unknown2: Uuid, // TODO: find out what this is
    pub server_id: String,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct StoreEntryPointInfo {
    pub store_id: String,
    pub store_name: String,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct PresenceInfo {
    pub experience_name: String,
    pub world_name: String,
}
