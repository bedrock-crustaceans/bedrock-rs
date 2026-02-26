use super::super::enums::{ChatRestrictionLevel, Difficulty, EditorWorldType, EducationEditionOffer, GamePublishSetting, GameType, GeneratorType, PlayerPermissionLevel};
use super::super::types::{BaseGameVersion, EduSharedUriResource, Experiments, GameRulesChangedPacketData, NetworkBlockPosition, SpawnSettings};
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct LevelSettings {
    #[endianness(le)]
    pub seed: u64,
    pub spawn_settings: SpawnSettings,
    pub generator_type: GeneratorType,
    pub game_type: GameType,
    pub is_hardcore_enabled: bool,
    pub game_difficulty: Difficulty,
    pub default_spawn_block_position: NetworkBlockPosition,
    pub achievements_disabled: bool,
    pub editor_world_type: EditorWorldType,
    pub is_created_in_editor: bool,
    pub is_exported_from_editor: bool,
    #[endianness(var)]
    pub day_cycle_stop_time: i32,
    pub education_edition_offer: EducationEditionOffer,
    pub education_features_enabled: bool,
    pub education_product_id: String,
    #[endianness(le)]
    pub rain_level: f32,
    #[endianness(le)]
    pub lightning_level: f32,
    pub has_confirmed_platform_locked_content: bool,
    pub multiplayer_enabled: bool,
    pub lan_broadcasting_enabled: bool,
    pub xbox_live_broadcast_setting: GamePublishSetting,
    pub platform_broadcast_setting: GamePublishSetting,
    pub commands_enabled: bool,
    pub texture_packs_required: bool,
    pub rule_data: GameRulesChangedPacketData,
    pub experiments: Experiments,
    pub bonus_chest_enabled: bool,
    pub starting_map_enabled: bool,
    pub player_permissions: PlayerPermissionLevel,
    #[endianness(le)]
    pub server_chunk_tick_range: i32,
    pub locked_behaviour_pack: bool,
    pub locked_resource_pack: bool,
    pub from_locked_template: bool,
    pub use_msa_gamer_tags: bool,
    pub from_template: bool,
    pub has_locked_template_settings: bool,
    pub only_spawn_v1_villagers: bool,
    pub persona_disabled: bool,
    pub custom_skins_disabled: bool,
    pub emote_chat_muted: bool,
    pub base_game_version: BaseGameVersion,
    #[endianness(le)]
    pub limited_world_width: i32,
    #[endianness(le)]
    pub limited_world_depth: i32,
    pub nether_type: bool,
    pub edu_shared_uri_resource: EduSharedUriResource,
    pub override_force_experimental_gameplay: Option<bool>,
    pub chat_restriction_level: ChatRestrictionLevel,
    pub disable_player_interactions: bool,
    pub server_id: String,
    pub world_id: String,
    pub scenario_id: String,
}