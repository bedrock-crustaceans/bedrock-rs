use super::super::enums::GameType;
use super::super::types::{ActorRuntimeID, ActorUniqueID, ItemData, LevelSettings, NetworkPermissions, SyncedPlayerMovementSettings};
use bedrockrs_macros::{gamepacket, ProtoCodec};
use uuid::Uuid;
use vek::{Vec2, Vec3};

#[gamepacket(id = 11)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct StartGamePacket {
    pub target_actor_id: ActorUniqueID,
    pub target_runtime_id: ActorRuntimeID,
    pub actor_game_type: GameType,
    #[endianness(le)]
    pub position: Vec3<f32>,
    #[endianness(le)]
    pub rotation: Vec2<f32>,
    pub settings: LevelSettings,
    pub level_id: String,
    pub level_name: String,
    pub template_content_identity: String,
    pub is_trial: bool,
    pub movement_settings: SyncedPlayerMovementSettings,
    #[endianness(le)]
    pub current_level_time: u64,
    #[endianness(var)]
    pub enchantment_seed: i32,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub block_properties: Vec<BlockProperty>,
    pub multiplayer_correlation_id: String,
    pub enable_item_stack_net_manager: bool,
    pub server_version: String,
    #[nbt]
    pub player_property_data: nbtx::Value, // TODO: NBT Structure,
    #[endianness(le)]
    pub server_block_type_registry_checksum: u64,
    pub world_template_id: Uuid,
    pub server_enabled_client_side_generation: bool,
    pub block_network_ids_are_hashes: bool,
    pub network_permissions: NetworkPermissions,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct BlockProperty {
    pub block_name: String,
    #[nbt]
    pub block_definition: nbtx::Value, // TODO: NBT Structure
}
