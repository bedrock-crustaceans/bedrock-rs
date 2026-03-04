use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct LevelSettings<V: ProtoVersion> {
    #[endianness(le)]
    pub seed: u64,
    pub spawn_settings: V::SpawnSettings,
    pub generator_type: V::GeneratorType,
    pub game_type: V::GameType,
    pub is_hardcore_enabled: bool,
    pub game_difficulty: V::Difficulty,
    pub default_spawn_block_position: V::NetworkBlockPosition,
    pub achievements_disabled: bool,
    pub editor_world_type: V::EditorWorldType,
    pub is_created_in_editor: bool,
    pub is_exported_from_editor: bool,
    #[endianness(var)]
    pub day_cycle_stop_time: i32,
    pub education_edition_offer: V::EducationEditionOffer,
    pub education_features_enabled: bool,
    pub education_product_id: String,
    #[endianness(le)]
    pub rain_level: f32,
    #[endianness(le)]
    pub lightning_level: f32,
    pub has_confirmed_platform_locked_content: bool,
    pub multiplayer_enabled: bool,
    pub lan_broadcasting_enabled: bool,
    pub xbox_live_broadcast_setting: V::GamePublishSetting,
    pub platform_broadcast_setting: V::GamePublishSetting,
    pub commands_enabled: bool,
    pub texture_packs_required: bool,
    pub rule_data: V::GameRulesChangedPacketData,
    pub experiments: V::Experiments,
    pub bonus_chest_enabled: bool,
    pub starting_map_enabled: bool,
    pub player_permissions: V::PlayerPermissionLevel,
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
    pub base_game_version: V::BaseGameVersion,
    #[endianness(le)]
    pub limited_world_width: i32,
    #[endianness(le)]
    pub limited_world_depth: i32,
    pub nether_type: bool,
    pub edu_shared_uri_resource: V::EduSharedUriResource,
    pub override_force_experimental_gameplay: Option<bool>,
    pub chat_restriction_level: V::ChatRestrictionLevel,
    pub disable_player_interactions: bool,
}
