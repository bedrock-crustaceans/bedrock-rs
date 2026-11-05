//! The rows of the suite: one per stack accessor (`Collection`), each with the document types
//! its files are, the level at which the writer promises each comes back, and the files each
//! type claims.

#[cfg_attr(
    not(feature = "full"),
    allow(unused_imports, reason = "a build without a family has no row of it")
)]
use bedrock_addon as ba;

#[cfg_attr(
    not(feature = "full"),
    allow(unused_imports, reason = "a build without a family has no row of it")
)]
use super::{Kind, Level, Row, Side};

/// `key` is a file below `directory` (at any depth) with `extension`.
fn below(key: &str, directory: &str, extension: &str) -> bool {
    key.strip_prefix(directory)
        .is_some_and(|rest| rest.starts_with('/') && rest.ends_with(extension))
}

/// `key` is a file directly in `directory` with `extension`.
fn within(key: &str, directory: &str, extension: &str) -> bool {
    key.strip_prefix(directory)
        .and_then(|rest| rest.strip_prefix('/'))
        .is_some_and(|name| !name.contains('/') && name.ends_with(extension))
}

/// A document type of a row.
#[cfg_attr(
    not(feature = "full"),
    allow(unused_macros, reason = "a build without a family has no row of it")
)]
macro_rules! kind {
    ($name:literal, $ty:ty, $level:ident, $side:ident, $claims:expr) => {
        Kind {
            name: $name,
            level: Level::$level,
            side: Side::$side,
            claims: $claims,
            carry: super::carry::<$ty>,
            same: super::same::<$ty>,
            as_written: None,
        }
    };
    ($name:literal, $ty:ty, $level:ident, $side:ident, $claims:expr, chain) => {
        Kind {
            name: $name,
            level: Level::$level,
            side: Side::$side,
            claims: $claims,
            carry: super::carry::<$ty>,
            same: super::same::<$ty>,
            as_written: Some({
                fn as_written(c: &super::Carry<'_>) -> super::AsWritten {
                    // A `JsonDocument`'s `parse_as_written` is the trait's; the others' are inherent.
                    #[allow(unused_imports)]
                    use ba::document::JsonDocument as _;
                    let Ok(first) = <$ty>::parse_as_written(c.path, c.bytes, c.cx) else {
                        return super::AsWritten::Rejected;
                    };
                    let text = match first.to_json() {
                        Ok(text) => text,
                        Err(error) => return super::AsWritten::NotWritten(format!("{error:?}")),
                    };
                    let Ok(again) = <$ty>::parse_as_written(c.path, text.as_bytes(), c.cx) else {
                        return super::AsWritten::Unreadable(text);
                    };
                    super::AsWritten::Written {
                        same_value: format!("{:?}", first.value())
                            == format!("{:?}", again.value())
                            && first.unknown() == again.unknown(),
                        read_at: format!("{:?}", first.version().parsed_as),
                        written_at: format!("{:?}", again.version().parsed_as),
                        rewritten: again.to_json().ok().as_ref() == Some(&text),
                    }
                }
                as_written
            }),
        }
    };
}

/// Every row, in the order of the accessor list.
#[allow(clippy::too_many_lines, reason = "one statement a row")]
#[allow(clippy::vec_init_then_push, reason = "each row is behind its feature")]
pub fn rows() -> Vec<Row> {
    #[allow(unused_mut, reason = "each row is behind its feature")]
    let mut rows: Vec<Row> = Vec::new();
    #[cfg(feature = "block")]
    rows.push(Row::of(
        "Blocks",
        vec![kind!(
            "block",
            ba::families::block::Block,
            Value,
            Behavior,
            |k| below(k, "blocks", ".json"),
            chain
        )],
    ));
    #[cfg(feature = "item")]
    rows.push(Row::of(
        "Items",
        vec![kind!(
            "item",
            ba::families::item::Item,
            Value,
            Behavior,
            |k| below(k, "items", ".json"),
            chain
        )],
    ));
    #[cfg(feature = "crafting_catalog")]
    rows.push(Row::of(
        "CraftingCatalog",
        vec![kind!(
            "crafting_catalog",
            ba::families::crafting_catalog::CraftingCatalog,
            Value,
            Behavior,
            |k| k == "item_catalog/crafting_item_catalog.json",
            chain
        )],
    ));
    #[cfg(feature = "entity")]
    rows.push(Row::of(
        "Entities",
        vec![kind!(
            "entity",
            ba::families::entity::Entity,
            Value,
            Behavior,
            |k| below(k, "entities", ".json"),
            chain
        )],
    ));
    #[cfg(feature = "biome")]
    rows.push(Row::of(
        "Biomes",
        vec![kind!(
            "biome",
            ba::families::biome::Biome,
            Value,
            Behavior,
            |k| below(k, "biomes", ".json"),
            chain
        )],
    ));
    #[cfg(feature = "client_biome")]
    rows.push(Row::of(
        "ClientBiomes",
        vec![kind!(
            "client_biome",
            ba::families::client_biome::ClientBiome,
            Value,
            Resource,
            |k| below(k, "biomes", ".json"),
            chain
        )],
    ));
    #[cfg(feature = "biomes_client")]
    rows.push(Row::of(
        "LegacyClientBiomes",
        vec![kind!(
            "biomes_client",
            ba::families::biomes_client::LegacyClientBiomes,
            Value,
            Resource,
            |k| k == "biomes_client.json"
        )],
    ));
    #[cfg(all(feature = "spawn_rules", feature = "spawn_groups"))]
    rows.push(Row::of(
        "SpawnRules",
        vec![kind!(
            "spawn_rules",
            ba::families::spawn_rules::SpawnRules,
            Value,
            Behavior,
            |k| below(k, "spawn_rules", ".json")
        )],
    ));
    #[cfg(all(feature = "spawn_rules", feature = "spawn_groups"))]
    rows.push(Row::of(
        "SpawnGroups",
        vec![kind!(
            "spawn_group",
            ba::families::spawn_groups::SpawnGroup,
            Value,
            Behavior,
            |k| below(k, "spawn_groups", ".json") || below(k, "definitions/spawn_groups", ".json")
        )],
    ));
    #[cfg(feature = "feature")]
    rows.push(Row::of(
        "Features",
        vec![kind!(
            "feature",
            ba::families::feature::Feature,
            Value,
            Behavior,
            |k| below(k, "features", ".json") || below(k, "definitions/features", ".json"),
            chain
        )],
    ));
    #[cfg(feature = "jigsaw")]
    rows.push(Row::of(
        "Jigsaw",
        vec![
            kind!(
                "jigsaw_structure",
                ba::families::jigsaw::JigsawStructure,
                Value,
                Behavior,
                |k| below(k, "worldgen/structures", ".json")
            ),
            kind!(
                "template_pool",
                ba::families::jigsaw::TemplatePool,
                Value,
                Behavior,
                |k| below(k, "worldgen/template_pools", ".json")
            ),
            kind!(
                "structure_set",
                ba::families::jigsaw::StructureSet,
                Value,
                Behavior,
                |k| below(k, "worldgen/structure_sets", ".json")
            ),
            kind!(
                "processor_list",
                ba::families::jigsaw::ProcessorList,
                Value,
                Behavior,
                |k| below(k, "worldgen/processors", ".json")
            ),
        ],
    ));
    #[cfg(feature = "recipe")]
    rows.push(Row::of(
        "Recipes",
        vec![kind!(
            "recipe",
            ba::families::recipe::Recipe,
            Value,
            Behavior,
            |k| { below(k, "recipes", ".json") }
        )],
    ));
    #[cfg(all(feature = "animation", feature = "animation_controller"))]
    rows.push(Row::of(
        "Animations",
        vec![kind!(
            "animation",
            ba::families::animation::AnimationFile,
            Value,
            Both,
            |k| below(k, "animations", ".json")
        )],
    ));
    #[cfg(all(feature = "animation", feature = "animation_controller"))]
    rows.push(Row::of(
        "AnimationControllers",
        vec![kind!(
            "animation_controller",
            ba::families::animation_controller::AnimationControllerFile,
            Value,
            Both,
            |k| below(k, "animation_controllers", ".json")
        )],
    ));
    #[cfg(feature = "feature_rules")]
    rows.push(Row::of(
        "FeatureRules",
        vec![kind!(
            "feature_rule",
            ba::families::feature_rules::FeatureRule,
            Value,
            Behavior,
            |k| below(k, "feature_rules", ".json")
                || below(k, "definitions/feature_rules", ".json")
        )],
    ));
    #[cfg(feature = "dimension")]
    rows.push(Row::of(
        "Dimensions",
        vec![kind!(
            "dimension",
            ba::families::dimension::Dimension,
            Value,
            Behavior,
            |k| below(k, "dimensions", ".json"),
            chain
        )],
    ));
    #[cfg(feature = "poi")]
    rows.push(Row::of(
        "Poi",
        vec![
            kind!(
                "poi_type",
                ba::families::poi::PoiType,
                Value,
                Behavior,
                |k| below(k, "poi", "_type.json")
            ),
            kind!("poi_tag", ba::families::poi::PoiTag, Value, Behavior, |k| {
                below(k, "poi", "_tag.json")
            }),
        ],
    ));
    #[cfg(feature = "dialogue")]
    rows.push(Row::of(
        "Dialogue",
        vec![kind!(
            "dialogue",
            ba::families::dialogue::DialogueFile,
            Value,
            Behavior,
            |k| below(k, "dialogue", ".json")
        )],
    ));
    #[cfg(feature = "trade")]
    rows.push(Row::of(
        "TradeTables",
        vec![kind!(
            "trade_table",
            ba::families::trade::TradeTable,
            Value,
            Behavior,
            |k| below(k, "trading", ".json"),
            chain
        )],
    ));
    #[cfg(feature = "structure")]
    rows.push(Row::of(
        "Structures",
        vec![kind!(
            "structure",
            ba::families::structure::Structure,
            Exact,
            Behavior,
            |k| below(k, "structures", ".mcstructure")
        )],
    ));
    #[cfg(feature = "loot")]
    rows.push(Row::of(
        "LootTables",
        vec![kind!(
            "loot_table",
            ba::families::loot::LootTable,
            Value,
            Behavior,
            |k| below(k, "loot_tables", ".json")
        )],
    ));
    #[cfg(feature = "function")]
    rows.push(Row::of(
        "Functions",
        vec![kind!(
            "function",
            ba::families::function::Function,
            Lines,
            Behavior,
            |k| below(k, "functions", ".mcfunction")
        )],
    ));
    #[cfg(feature = "function")]
    rows.push(Row::of(
        "TickFunctions",
        vec![kind!(
            "tick_list",
            ba::families::function::TickList,
            Value,
            Behavior,
            |k| k == "functions/tick.json"
        )],
    ));
    #[cfg(feature = "languages")]
    rows.push(Row::of(
        "Languages",
        vec![
            kind!(
                "languages",
                ba::families::languages::Languages,
                Value,
                Both,
                |k| k == "texts/languages.json"
            ),
            kind!(
                "language_names",
                ba::families::languages::LanguageNames,
                Value,
                Both,
                |k| k == "texts/language_names.json"
            ),
        ],
    ));
    #[cfg(feature = "lang")]
    rows.push(Row::of(
        "Translations",
        vec![kind!(
            "lang",
            ba::families::lang::LangFile,
            Lines,
            Both,
            |k| within(k, "texts", ".lang")
        )],
    ));
    // Script files have no document type: a copy keeps their bytes (`read_only`).
    #[cfg(feature = "script")]
    rows.push(Row::of("Scripts", Vec::new()));
    #[cfg(feature = "behavior_tree")]
    rows.push(Row::of(
        "BehaviorTrees",
        vec![kind!(
            "behavior_tree",
            ba::families::behavior_tree::BehaviorTree,
            Value,
            Behavior,
            |k| below(k, "behavior_trees", ".json")
        )],
    ));
    #[cfg(feature = "voxel_shape")]
    rows.push(Row::of(
        "VoxelShapes",
        vec![kind!(
            "voxel_shape",
            ba::families::voxel_shape::VoxelShape,
            Value,
            Behavior,
            |k| below(k, "shapes", ".json")
        )],
    ));
    #[cfg(feature = "geometry")]
    rows.push(Row::of(
        "Geometries",
        vec![kind!(
            "geometry",
            ba::families::geometry::GeometryFile,
            Value,
            Resource,
            |k| below(k, "models", ".json"),
            chain
        )],
    ));
    #[cfg(feature = "camera")]
    rows.push(Row::of(
        "Cameras",
        vec![
            kind!(
                "camera_preset",
                ba::families::camera::CameraPreset,
                Value,
                Behavior,
                |k| below(k, "cameras/presets", ".json"),
                chain
            ),
            kind!(
                "camera_list",
                ba::families::camera::CameraList,
                Value,
                Behavior,
                |k| below(k, "cameras/lists", ".json")
            ),
            kind!(
                "camera_spline",
                ba::families::camera::CameraSpline,
                Value,
                Behavior,
                |k| below(k, "cameras/splines", ".json")
            ),
        ],
    ));
    #[cfg(feature = "aim_assist")]
    rows.push(Row::of(
        "AimAssist",
        vec![
            kind!(
                "aim_assist_preset",
                ba::families::aim_assist::AimAssistPreset,
                Value,
                Behavior,
                |k| below(k, "aim_assist/presets", ".json"),
                chain
            ),
            kind!(
                "aim_assist_categories",
                ba::families::aim_assist::AimAssistCategories,
                Value,
                Behavior,
                |k| below(k, "aim_assist/categories", ".json")
            ),
        ],
    ));
    #[cfg(feature = "sound_definitions")]
    rows.push(Row::of(
        "SoundDefinitions",
        vec![kind!(
            "sound_definitions",
            ba::families::sound_definitions::SoundDefinitions,
            ValueRaw,
            Resource,
            |k| k == "sounds/sound_definitions.json",
            chain
        )],
    ));
    #[cfg(feature = "sounds_json")]
    rows.push(Row::of(
        "SoundsJson",
        vec![kind!(
            "sounds",
            ba::families::sounds_json::SoundBindings,
            ValueRaw,
            Resource,
            |k| k == "sounds.json"
        )],
    ));
    #[cfg(feature = "music")]
    rows.push(Row::of(
        "Music",
        vec![kind!(
            "music_definitions",
            ba::families::music::MusicDefinitions,
            ValueRaw,
            Resource,
            |k| k == "sounds/music_definitions.json"
        )],
    ));
    // Audio files have no document type: a copy keeps their bytes (`read_only`).
    #[cfg(feature = "sound_file")]
    rows.push(Row::of("SoundFiles", Vec::new()));
    #[cfg(feature = "server_sound_definitions")]
    rows.push(Row::of(
        "ServerSoundDefinitions",
        vec![kind!(
            "server_sound_definitions",
            ba::families::server_sound_definitions::SoundMetadataFile,
            Value,
            Behavior,
            |k| k == "sounds/sound_definitions.json"
        )],
    ));
    #[cfg(feature = "deferred_settings")]
    rows.push(Row::of(
        "DeferredSettings",
        vec![
            kind!(
                "lighting",
                ba::families::deferred_settings::LightingSettings,
                Value,
                Resource,
                |k| k == "lighting/global.json"
            ),
            kind!(
                "atmosphere",
                ba::families::deferred_settings::AtmosphereSettings,
                Value,
                Resource,
                |k| within(k, "atmospherics", ".json")
            ),
            kind!(
                "water",
                ba::families::deferred_settings::WaterSettings,
                Value,
                Resource,
                |k| within(k, "water", ".json")
            ),
            kind!(
                "color_grading",
                ba::families::deferred_settings::ColorGradingSettings,
                Value,
                Resource,
                |k| within(k, "color_grading", ".json")
            ),
            kind!(
                "cubemap",
                ba::families::deferred_settings::CubemapSettings,
                Value,
                Resource,
                |k| within(k, "cubemaps", ".json")
            ),
            kind!(
                "shadows",
                ba::families::deferred_settings::ShadowSettings,
                Value,
                Resource,
                |k| k == "shadows/global.json"
            ),
            kind!(
                "pbr_fallback",
                ba::families::deferred_settings::PbrFallbackSettings,
                Value,
                Resource,
                |k| k == "pbr/global.json"
            ),
            kind!(
                "local_light",
                ba::families::deferred_settings::LocalLightSettings,
                Value,
                Resource,
                |k| k == "local_lighting/local_lighting.json"
            ),
        ],
    ));
    #[cfg(feature = "json_ui")]
    rows.push(Row::of(
        "Ui",
        vec![
            kind!(
                "ui_defs",
                ba::families::json_ui::UiDefsList,
                ValueRaw,
                Resource,
                |k| k == "ui/_ui_defs.json"
            ),
            kind!(
                "ui_global_variables",
                ba::families::json_ui::GlobalVariables,
                ValueRaw,
                Resource,
                |k| k == "ui/_global_variables.json"
            ),
            kind!(
                "ui_file",
                ba::families::json_ui::UiFile,
                ValueRaw,
                Resource,
                |k| below(k, "ui", ".json")
                    && k != "ui/_ui_defs.json"
                    && k != "ui/_global_variables.json"
            ),
        ],
    ));
    #[cfg(feature = "blocks_json")]
    rows.push(Row::of(
        "ClientBlocks",
        vec![kind!(
            "blocks_json",
            ba::families::blocks_json::BlocksJson,
            ValueRaw,
            Resource,
            |k| k == "blocks.json"
        )],
    ));
    #[cfg(feature = "block_culling")]
    rows.push(Row::of(
        "BlockCulling",
        vec![kind!(
            "block_culling",
            ba::families::block_culling::CullingRules,
            Value,
            Resource,
            |k| below(k, "block_culling", ".json")
        )],
    ));
    #[cfg(feature = "ddui")]
    rows.push(Row::of(
        "DataDrivenUi",
        vec![
            kind!(
                "ddui_root",
                ba::families::ddui::UiRoot,
                Value,
                Resource,
                |k| below(k, "ddui/root", ".json")
            ),
            kind!(
                "ddui_composition",
                ba::families::ddui::UiComposition,
                Value,
                Resource,
                |k| below(k, "ddui/composition", ".json")
            ),
            kind!(
                "ddui_style",
                ba::families::ddui::UiStyle,
                Value,
                Resource,
                |k| below(k, "ddui/styling", ".json")
            ),
        ],
    ));
    #[cfg(feature = "texture_atlas")]
    rows.push(Row::of(
        "TextureAtlases",
        vec![kind!(
            "terrain_texture",
            ba::families::texture_atlas::TextureAtlas,
            ValueRaw,
            Resource,
            |k| k == "textures/terrain_texture.json" || k == "textures/item_texture.json"
        )],
    ));
    #[cfg(feature = "flipbooks")]
    rows.push(Row::of(
        "Flipbooks",
        vec![kind!(
            "flipbook_textures",
            ba::families::flipbooks::Flipbooks,
            ValueRaw,
            Resource,
            |k| k == "textures/flipbook_textures.json"
        )],
    ));
    #[cfg(feature = "texture_list")]
    rows.push(Row::of(
        "TextureLists",
        vec![kind!(
            "textures_list",
            ba::families::texture_list::TextureList,
            Value,
            Resource,
            |k| k == "textures/textures_list.json"
        )],
    ));
    #[cfg(feature = "texture_set")]
    rows.push(Row::of(
        "TextureSets",
        vec![kind!(
            "texture_set",
            ba::families::texture_set::TextureSet,
            Value,
            Resource,
            |k| below(k, "textures", ".texture_set.json"),
            chain
        )],
    ));
    // Images have no document type: a copy keeps their bytes (`read_only`).
    #[cfg(feature = "texture_file")]
    rows.push(Row::of("TextureFiles", Vec::new()));
    #[cfg(feature = "splashes")]
    rows.push(Row::of(
        "Splashes",
        vec![kind!(
            "splashes",
            ba::families::splashes::Splashes,
            Value,
            Resource,
            |k| k == "splashes.json"
        )],
    ));
    #[cfg(feature = "loading_messages")]
    rows.push(Row::of(
        "LoadingMessages",
        vec![kind!(
            "loading_messages",
            ba::families::loading_messages::LoadingMessages,
            Value,
            Resource,
            |k| k == "loading_messages.json"
        )],
    ));
    #[cfg(feature = "credits")]
    rows.push(Row::of(
        "Credits",
        vec![kind!(
            "credits",
            ba::families::credits::Credits,
            Value,
            Resource,
            |k| k == "credits/credits.json"
        )],
    ));
    #[cfg(feature = "font")]
    rows.push(Row::of(
        "Fonts",
        vec![
            kind!(
                "font_metadata",
                ba::families::font::FontMetadata,
                Value,
                Resource,
                |k| k == "font/font_metadata.json"
            ),
            kind!(
                "emoticons",
                ba::families::font::EmoticonList,
                Value,
                Resource,
                |k| k == "font/emoticons.json"
            ),
        ],
    ));
    #[cfg(feature = "ui_texture_info")]
    rows.push(Row::of(
        "UiTextureInfos",
        vec![kind!(
            "ui_texture_info",
            ba::families::ui_texture_info::UiTextureInfo,
            Value,
            Resource,
            |k| below(k, "textures/ui", ".json")
        )],
    ));
    #[cfg(feature = "render_controller")]
    rows.push(Row::of(
        "RenderControllers",
        vec![kind!(
            "render_controller",
            ba::families::render_controller::RenderControllerFile,
            Value,
            Resource,
            |k| below(k, "render_controllers", ".json")
        )],
    ));
    #[cfg(feature = "fog")]
    rows.push(Row::of(
        "Fogs",
        vec![kind!(
            "fog",
            ba::families::fog::FogSettings,
            Value,
            Resource,
            |k| within(k, "fogs", ".json"),
            chain
        )],
    ));
    #[cfg(feature = "client_entity")]
    rows.push(Row::of(
        "ClientEntities",
        vec![kind!(
            "client_entity",
            ba::families::client_entity::ClientEntity,
            Value,
            Resource,
            |k| below(k, "entity", ".json"),
            chain
        )],
    ));
    #[cfg(feature = "atomic_client_entity")]
    rows.push(Row::of(
        "AtomicClientEntities",
        vec![kind!(
            "atomic_client_entity",
            ba::families::atomic_client_entity::AtomicClientEntity,
            Value,
            Resource,
            |k| below(k, "atomic_client_entities", ".json")
        )],
    ));
    #[cfg(feature = "attachable")]
    rows.push(Row::of(
        "Attachables",
        vec![kind!(
            "attachable",
            ba::families::attachable::Attachable,
            Value,
            Resource,
            |k| below(k, "attachables", ".json") || below(k, "definitions/attachables", ".json"),
            chain
        )],
    ));
    #[cfg(feature = "material")]
    rows.push(Row::of(
        "Materials",
        vec![
            kind!(
                "material_file",
                ba::families::material::MaterialFile,
                Value,
                Resource,
                |k| below(k, "materials", ".material")
            ),
            kind!(
                "material_list",
                ba::families::material::MaterialList,
                Value,
                Resource,
                |k| within(k, "materials", ".json")
            ),
        ],
    ));
    #[cfg(feature = "particle")]
    rows.push(Row::of(
        "Particles",
        vec![kind!(
            "particle",
            ba::families::particle::ParticleEffect,
            Value,
            Resource,
            |k| below(k, "particles", ".json"),
            chain
        )],
    ));
    rows
}
