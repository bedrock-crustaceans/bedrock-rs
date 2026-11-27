//! The JSON document types of the crate, each with a small document its family loads: the
//! list the audits of `non_finite.rs` and `writable.rs` walk, one test per type.
//!
//! `document_types!(each)` hands the list to the macro `each`, one row per type:
//! `name: Type, "path" [in context], document [, [hand-written places]];`. A hand-written
//! place is `(name, text with `@` for the number, path of the number)`.

#![allow(dead_code, reason = "each audit uses its part")]

use bedrock_addon::load::{Experiment, Experiments, LoadContext, LoadOptions};
use bedrock_addon::pack::PackType;
use bedrock_addon::version::MinEngineVersion;

/// The context of a pack of `pack_type` whose manifest says `min_engine_version`.
pub fn pack(pack_type: PackType, min_engine_version: (u16, u16, u16)) -> LoadContext {
    let (major, minor, patch) = min_engine_version;
    LoadContext::for_pack(
        &LoadOptions::default(),
        uuid::Uuid::from_u128(7),
        pack_type,
        Some(MinEngineVersion::new(major, minor, patch)),
        false,
    )
}

/// [`pack`] with the Beta APIs toggle on.
pub fn beta(pack_type: PackType, min_engine_version: (u16, u16, u16)) -> LoadContext {
    let mut cx = pack(pack_type, min_engine_version);
    cx.beta_apis = true;
    cx
}

/// A resource pack whose Molang version follows its engine version (client entities).
pub fn client(min_engine_version: (u16, u16, u16)) -> LoadContext {
    let mut cx = pack(PackType::Resource, min_engine_version);
    cx.molang_version = bedrock_addon::molang::pack_molang_version(cx.min_engine_version.as_ref());
    cx
}

/// A behaviour pack of a world with the POI experiment.
pub fn poi() -> LoadContext {
    let mut experiments = Experiments::new();
    experiments.insert(Experiment::Poi);
    let options = LoadOptions {
        experiments,
        ..LoadOptions::default()
    };
    LoadContext::for_pack(
        &options,
        uuid::Uuid::from_u128(7),
        PackType::Behavior,
        Some(MinEngineVersion::new(1, 26, 40)),
        false,
    )
}

/// The context the data-driven UI documents are read with: a pack that is not the game's, at
/// the newest engine, whose component catalogue the vanilla roots of the corpus are written
/// for (an engine pinned at 1.26.20 refuses seven of the nine: `Panel.Spacing` is later).
pub fn ddui() -> LoadContext {
    let mut cx = LoadContext::detached();
    cx.is_base_game_pack = false;
    cx
}

/// Hands the list of document types to the macro `$each`.
macro_rules! document_types {
    ($each:ident) => {
        $each! {
        // --- behaviour packs ---
        block: bedrock_addon::families::block::Block, "blocks/x.json",
            r#"{"format_version": "1.21.60", "minecraft:block": {"description": {"identifier": "demo:x"}, "components": {"minecraft:friction": 0.4}}}"#;
        block_cereal: bedrock_addon::families::block::Block, "blocks/x.json",
            r#"{"format_version": "1.26.20", "minecraft:block": {"description": {"identifier": "demo:x"}, "components": {"minecraft:friction": 0.4}}}"#;
        voxel_shape: bedrock_addon::families::voxel_shape::VoxelShape, "shapes/x.json",
            r#"{"format_version": "1.21.110", "minecraft:voxel_shape": {"description": {"identifier": "demo:x"}, "shape": {"boxes": [{"min": [0, 0, 0], "max": [16, 8, 16]}]}}}"#;
        item: bedrock_addon::families::item::Item, "items/x.json",
            r#"{"format_version": "1.26.50", "minecraft:item": {"description": {"identifier": "demo:x"}, "components": {"minecraft:max_stack_size": 16}}}"#;
        item_beta: bedrock_addon::families::item::Beta, "items/x.json" in beta(PackType::Behavior, (1, 26, 30)),
            r#"{"format_version": "beta", "minecraft:item": {"description": {"identifier": "demo:x"}, "components": {"minecraft:max_stack_size": 1}}}"#;
        crafting_catalog: bedrock_addon::families::crafting_catalog::CraftingCatalog, "item_catalog/crafting_item_catalog.json",
            r#"{"format_version": "1.21.60", "minecraft:crafting_items_catalog": {"categories": [{"category_name": "construction", "groups": [{"group_identifier": {"name": "demo:x"}, "items": []}]}]}}"#;
        entity: bedrock_addon::families::entity::Entity, "entities/x.json",
            r#"{"format_version": "1.21.60", "minecraft:entity": {"description": {"identifier": "demo:x", "is_spawnable": true, "is_summonable": true}, "components": {"minecraft:health": {"value": 10, "max": 10}}, "events": {"demo:e": {"add": {"component_groups": []}}}}}"#;
        entity_beta: bedrock_addon::families::entity::Beta, "entities/x.json" in beta(PackType::Behavior, (1, 26, 30)),
            r#"{"format_version": "beta", "minecraft:entity": {"description": {"identifier": "demo:x"}, "components": {}}}"#;
        spawn_rules: bedrock_addon::families::spawn_rules::SpawnRules, "spawn_rules/x.json",
            r#"{"format_version": "1.8.0", "minecraft:spawn_rules": {"description": {"identifier": "demo:x", "population_control": "animal"}, "conditions": [{"minecraft:spawns_on_surface": {}}]}}"#;
        spawn_group: bedrock_addon::families::spawn_groups::SpawnGroup, "spawn_groups/x.json",
            r#"{"format_version": "1.8.0", "minecraft:spawn_groups": {"description": {"identifier": "demo:x", "population_control": "monster"}, "conditions": [{"minecraft:spawns_on_surface": {}}]}}"#;
        biome: bedrock_addon::families::biome::Biome, "biomes/x.json",
            r#"{"format_version": "1.21.110", "minecraft:biome": {"description": {"identifier": "demo:x"}, "components": {}}}"#;
        biome_beta: bedrock_addon::families::biome::Beta, "biomes/demo.biome.json" in beta(PackType::Behavior, (1, 26, 30)),
            r#"{"format_version": "beta", "minecraft:biome": {"description": {"identifier": "test:beta"}, "components": {}, "environment_attributes": {"minecraft:experimental_float_attr_1": {"operation": "add", "value": 1.0}}}}"#;
        feature: bedrock_addon::families::feature::Feature, "features/test.json",
            r#"{"format_version": "1.13.0", "minecraft:ore_feature": {"description": {"identifier": "demo:x"}, "count": 4.0}}"#;
        feature_rule: bedrock_addon::families::feature_rules::FeatureRule, "feature_rules/x.json",
            r#"{"format_version": "1.13.0", "minecraft:feature_rules": {"description": {"identifier": "demo:x", "places_feature": "demo:feature"}, "conditions": {"placement_pass": "surface_pass"}}}"#;
        dimension: bedrock_addon::families::dimension::Dimension, "dimensions/overworld.json",
            r#"{"format_version": "1.18.0", "minecraft:dimension": {"description": {"identifier": "minecraft:overworld"}, "components": {"minecraft:dimension_bounds": {"min": -64, "max": 320}, "minecraft:generation": {"generator_type": "void"}}}}"#;
        jigsaw_structure: bedrock_addon::families::jigsaw::JigsawStructure, "worldgen/structures/a.json" in pack(PackType::Behavior, (1, 26, 20)),
            r#"{"format_version": "1.21.20", "minecraft:jigsaw": {"description": {"identifier": "demo:a"}, "step": "lakes", "start_pool": "demo:p", "max_depth": 1, "start_height": {"type": "constant", "value": {"absolute": 0}}}}"#;
        template_pool: bedrock_addon::families::jigsaw::TemplatePool, "worldgen/template_pools/p.json" in pack(PackType::Behavior, (1, 26, 20)),
            r#"{"format_version": "1.21.20", "minecraft:template_pool": {"description": {"identifier": "demo:p"}, "elements": [{"element": {"element_type": "minecraft:single_pool_element", "location": "aaaa"}}]}}"#;
        structure_set: bedrock_addon::families::jigsaw::StructureSet, "worldgen/structure_sets/s.json" in pack(PackType::Behavior, (1, 26, 20)),
            r#"{"format_version": "1.21.20", "minecraft:structure_set": {"description": {"identifier": "minecraft:trail_ruins"}, "placement": {"type": "minecraft:random_spread", "salt": 1, "separation": 4, "spacing": 9}, "structures": [{"structure": "minecraft:trail_ruins", "weight": 1}]}}"#;
        processor_list: bedrock_addon::families::jigsaw::ProcessorList, "worldgen/processors/l.json" in pack(PackType::Behavior, (1, 26, 20)),
            r#"{"format_version": "1.21.20", "minecraft:processor_list": {"description": {"identifier": "demo:l"}, "processors": [{"processor_type": "minecraft:rule", "rules": [{"output_state": {"name": "minecraft:stone"}}]}]}}"#;
        loot_table: bedrock_addon::families::loot::LootTable, "loot_tables/x.json",
            r#"{"pools": [{"rolls": 1, "entries": [{"type": "item", "name": "minecraft:stone"}]}]}"#;
        recipe: bedrock_addon::families::recipe::Recipe, "recipes/x.json",
            r#"{"format_version": "1.12.0", "minecraft:recipe_shapeless": {"description": {"identifier": "demo:x"}, "tags": ["crafting_table"], "ingredients": ["minecraft:stone"], "result": {"item": "minecraft:stone", "count": 1}}}"#;
        trade_table: bedrock_addon::families::trade::TradeTable, "trading/x.json",
            r#"{"tiers": [{"trades": [{"wants": [{"item": "minecraft:emerald"}], "gives": [{"item": "minecraft:bread", "quantity": 6}]}]}]}"#;
        dialogue: bedrock_addon::families::dialogue::DialogueFile, "dialogue/x.json",
            r#"{"format_version": "1.17.0", "minecraft:npc_dialogue": {"scenes": [{"scene_tag": "x", "text": "Hi."}]}}"#;
        camera_preset: bedrock_addon::families::camera::CameraPreset, "cameras/presets/x.json",
            r#"{"format_version": "1.19.50", "minecraft:camera_preset": {"identifier": "demo:x", "inherit_from": "minecraft:free"}}"#;
        camera_list: bedrock_addon::families::camera::CameraList, "cameras/lists/test.json",
            r#"{"format_version": "1.19.50", "minecraft:camera_list": {"identifier": "ex:l", "cameras": ["minecraft:free"]}}"#;
        camera_spline: bedrock_addon::families::camera::CameraSpline, "cameras/splines/test.json",
            r#"{"format_version": "1.26.0", "minecraft:camera_custom_splines": {"splines": [{"name": "ex:s", "total_time": 4.0, "control_points": [{"position": [0, 0, 0]}, {"position": [1, 2, 3]}], "progress_key_frames": [{"progress": 0.0, "time": 0.0}], "rotation_key_frames": [{"rotation": [0, 0, 0], "time": 0.0}]}]}}"#;
        aim_assist_preset: bedrock_addon::families::aim_assist::AimAssistPreset, "aim_assist/presets/test.json",
            r#"{"format_version": "1.21.50", "minecraft:aim_assist_preset": {"identifier": "test:p"}}"#;
        aim_assist_categories: bedrock_addon::families::aim_assist::AimAssistCategories, "aim_assist/categories/test.json",
            r#"{"format_version": "1.21.50", "minecraft:aim_assist_categories": {"categories": []}}"#;
        tick_list: bedrock_addon::families::function::TickList, "functions/tick.json", r#"{"values": ["x"]}"#;
        behavior_tree: bedrock_addon::families::behavior_tree::BehaviorTree, "behavior_trees/x.json",
            r#"{"minecraft:behavior_tree": {"minecraft:name": "x", "minecraft:root": {"minecraft:wait_ticks": {"num_of_ticks": 1}}}}"#;
        poi_type: bedrock_addon::families::poi::PoiType, "poi/types/x.json" in poi(),
            r#"{"format_version": "1.26.40", "minecraft:poi_block": {"description": {"identifier": "demo:x"}, "blocks": ["minecraft:bell"]}}"#;
        poi_tag: bedrock_addon::families::poi::PoiTag, "poi/document.json" in poi(),
            r#"{"format_version": "1.26.40", "minecraft:poi_tag": {"description": {"identifier": "test:civic"}}}"#;
        server_sound_definitions: bedrock_addon::families::server_sound_definitions::SoundMetadataFile, "sounds/sound_definitions.json" in beta(PackType::Behavior, (1, 26, 40)),
            r#"{"format_version": "1.26.60", "minecraft:server_sound_definitions": [{"sound_event_name": "demo:click"}]}"#;
        manifest: bedrock_addon::manifest::Manifest, "manifest.json",
            r#"{"format_version": 2, "header": {"name": "x", "uuid": "00000000-0000-0000-0000-000000000001", "version": [1, 0, 0], "min_engine_version": [1, 21, 0]}, "modules": [{"type": "data", "uuid": "00000000-0000-0000-0000-000000000002", "version": [1, 0, 0]}], "metadata": {"authors": ["a"]}}"#;

        // --- resource packs ---
        blocks_json: bedrock_addon::families::blocks_json::BlocksJson, "blocks.json",
            r#"{"format_version": "1.21.40", "dirt": {"textures": "d", "client_entity": {"a": [1, 2]}}}"#;
        block_culling: bedrock_addon::families::block_culling::CullingRules, "block_culling/test.json",
            r#"{"format_version": "1.21.80", "minecraft:block_culling_rules": {"description": {"identifier": "test:culling_parsing"}, "rules": [{"geometry_part": {"bone": "bone1", "cube": 2, "face": "east"}, "direction": "north"}]}}"#;
        attachable: bedrock_addon::families::attachable::Attachable, "attachables/x.json" in client((1, 21, 0)),
            r#"{"format_version": "1.8.0", "minecraft:attachable": {"description": {"identifier": "demo:ghost", "materials": {"default": "entity"}}}}"#;
        client_entity: bedrock_addon::families::client_entity::ClientEntity, "entity/x.entity.json" in client((1, 21, 0)),
            r#"{"format_version": "1.10", "minecraft:client_entity": {"description": {"identifier": "demo:ghost", "materials": {"default": "entity"}}}}"#;
        atomic_client_entity: bedrock_addon::families::atomic_client_entity::AtomicClientEntity, "atomic_client_entities/wisp.json" in client((1, 21, 0)),
            r#"{"format_version": "beta", "minecraft:atomic_client_entity": {"description": {"identifier": "demo:wisp"}}}"#;
        client_biome: bedrock_addon::families::client_biome::ClientBiome, "biomes/demo.client_biome.json" in pack(PackType::Resource, (1, 26, 30)),
            r##"{"format_version": "1.21.40", "minecraft:client_biome": {"description": {"identifier": "minecraft:mesa_plateau"}, "components": {"minecraft:sky_color": {"sky_color": "#102030"}}}}"##;
        client_biome_beta: bedrock_addon::families::client_biome::Beta, "biomes/demo.client_biome.json" in beta(PackType::Resource, (1, 26, 30)),
            r##"{"format_version": "beta", "minecraft:client_biome": {"description": {"identifier": "myns:glade"}, "components": {"minecraft:sky_color": {"sky_color": "#102030"}}, "environment_attributes": {"minecraft:experimental_float_attr_1": {"operation": "add", "value": 1.0}}}}"##;
        biomes_client: bedrock_addon::families::biomes_client::LegacyClientBiomes, "biomes_client.json" in pack(PackType::Resource, (1, 26, 30)),
            r##"{"biomes": {"plains": {"water_surface_color": "#44aff5"}, "default": {}}}"##;
        camera_entity: bedrock_addon::families::camera::CameraEntity, "cameras/test.json",
            r#"{"format_version": "1.26.10", "minecraft:camera_entity": {"description": {"identifier": "ex:all"}, "components": {"minecraft:camera": {"field_of_view": 80}, "minecraft:camera_orbit": {}}}}"#;
        particle: bedrock_addon::families::particle::ParticleEffect, "particles/demo.particle.json" in pack(PackType::Resource, (1, 21, 100)),
            // (O-5); a 1.10.0 document keeps one, and its upgraded value has no text for it.
            r#"{"format_version": "1.20.80", "particle_effect": {"description": {"identifier": "demo:spark", "basic_render_parameters": {"material": "particles_alpha", "texture": "textures/particle/particles"}}, "components": {"minecraft:emitter_lifetime_once": {}, "minecraft:emitter_rate_instant": {"num_particles": 1}, "minecraft:emitter_shape_sphere": {"radius": 1}}}}"#;
        animation: bedrock_addon::families::animation::AnimationFile, "animations/x.json" in client((1, 21, 0)),
            r#"{"format_version": "1.10", "animations": {"animation.x": {}}}"#;
        animation_controller: bedrock_addon::families::animation_controller::AnimationControllerFile, "animation_controllers/x.json" in client((1, 16, 0)),
            r#"{"format_version": "1.10", "animation_controllers": {"controller.animation.x": {"initial_state": "default", "states": {"default": {}, "idle": {}}}}}"#;
        geometry: bedrock_addon::families::geometry::GeometryFile, "models/entity/x.json",
            r#"{"format_version": "1.21.0", "minecraft:geometry": [{"description": {"identifier": "geometry.x"}, "bones": [{"name": "b", "pivot": [0, 0, 0]}]}]}"#;
        render_controller: bedrock_addon::families::render_controller::RenderControllerFile, "render_controllers/x.json" in pack(PackType::Resource, (1, 21, 100)),
            r#"{"format_version": "1.8.0", "render_controllers": {"controller.render.test": {"geometry": "Geometry.default", "materials": [{"*": "Material.default"}], "textures": ["Texture.default"]}}}"#;
        material_file: bedrock_addon::families::material::MaterialFile, "materials/test.material",
            r#"{"materials": {"version": "1.0.0", "x": {"zz_unknown": {"a": [1, "x"]}}}}"#;
        material_list: bedrock_addon::families::material::MaterialList, "materials/common.json",
            r#"[{"path": "materials/entity.material"}]"#;
        fog: bedrock_addon::families::fog::FogSettings, "fogs/x.json",
            r#"{"format_version": "1.16.100", "minecraft:fog_settings": {"description": {"identifier": "demo:x"}, "distance": {"air": {"fog_start": 0, "fog_end": 1, "fog_color": [1, 1, 1], "render_distance_type": "fixed"}}}}"#;
        lighting: bedrock_addon::families::deferred_settings::LightingSettings, "test.json",
            r#"{"format_version": "1.26", "minecraft:lighting_settings": {}}"#;
        atmosphere: bedrock_addon::families::deferred_settings::AtmosphereSettings, "test.json",
            r#"{"format_version": "1.26.20", "minecraft:atmosphere_settings": {"description": {"identifier": "t:a"}}}"#;
        water: bedrock_addon::families::deferred_settings::WaterSettings, "test.json",
            r#"{"format_version": "1.26", "minecraft:water_settings": {"description": {"identifier": "test:w"}}}"#;
        color_grading: bedrock_addon::families::deferred_settings::ColorGradingSettings, "test.json",
            r#"{"format_version": "1.21.90", "minecraft:color_grading_settings": {"description": {"identifier": "t:g"}, "color_grading": {"midtones": {"contrast": [1, 1, 1]}}}}"#;
        cubemap: bedrock_addon::families::deferred_settings::CubemapSettings, "test.json",
            r#"{"format_version": "1.21.130", "minecraft:cubemap_settings": {"description": {"identifier": "t:c"}, "lighting": {"ambient_light_illuminance": 1}}}"#;
        shadows: bedrock_addon::families::deferred_settings::ShadowSettings, "test.json",
            r#"{"format_version": "1.21.80", "minecraft:shadow_settings": {"shadow_style": "soft_shadows", "mystery": {"a": [1, 2]}}}"#;
        pbr_fallback: bedrock_addon::families::deferred_settings::PbrFallbackSettings, "test.json",
            r#"{"format_version": "1.21.40", "minecraft:pbr_fallback_settings": {"blocks": {"global_metalness_emissive_roughness_subsurface": [0, 0, 255, 0]}}}"#;
        local_light: bedrock_addon::families::deferred_settings::LocalLightSettings, "test.json",
            r##"{"format_version": "1.21.120", "minecraft:local_light_settings": {"minecraft:torch": {"light_type": "point_light", "light_color": "#EFE39D"}}}"##;
        terrain_texture: bedrock_addon::families::texture_atlas::TextureAtlas, "textures/terrain_texture.json",
            r#"{"texture_name": "atlas.terrain", "texture_data": {"dirt": {"textures": "textures/blocks/dirt"}, "stone": {"textures": ["textures/blocks/stone", {"path": "textures/blocks/stone2"}]}}}"#;
        flipbook_textures: bedrock_addon::families::flipbooks::Flipbooks, "textures/flipbook_textures.json",
            r#"[{"flipbook_texture": "textures/blocks/x", "atlas_tile": "x", "ticks_per_frame": 1}]"#;
        textures_list: bedrock_addon::families::texture_list::TextureList, "textures/textures_list.json",
            r#"["textures/blocks/sulfur", "textures/blocks/polished_sulfur"]"#,
            [
                ("an element of the list", r#"["textures/blocks/sulfur", @]"#, "1"),
                ("an object in the list", r#"["textures/blocks/sulfur", {"zz_probe": @}]"#, "1/zz_probe"),
            ];
        texture_set: bedrock_addon::families::texture_set::TextureSet, "textures/blocks/x.texture_set.json",
            r#"{"format_version": "1.21.30", "minecraft:texture_set": {"color": [1, 2, 3, 4]}}"#;
        legacy_path_mapping: bedrock_addon::families::legacy_path_mapping::LegacyPathMapping, "texture_backwards_compatibility_mapping.json",
            r#"{"textures/old": {"back_compat_texture": "textures/new", "uv": [0, 0], "uv_size": [16, 16], "base_size": [64, 64]}}"#;
        sound_definitions: bedrock_addon::families::sound_definitions::SoundDefinitions, "sounds/sound_definitions.json",
            r#"{"format_version": "1.20.20", "sound_definitions": {"x": {"min_distance": 2, "sounds": ["a", {"name": "b", "weight": 3}]}}}"#;
        sounds: bedrock_addon::families::sounds_json::SoundBindings, "sounds.json",
            r#"{"block_sounds": {"stone": {"events": {"break": {"sound": "dig.stone", "volume": [0.5, 1]}}, "pitch": 1}}, "entity_sounds": {"defaults": {"events": {"ambient": "mob.pig.say"}}}, "interactive_sounds": {"block_sounds": {}}}"#;
        music_definitions: bedrock_addon::families::music::MusicDefinitions, "sounds/music_definitions.json",
            r#"{"game": {"event_name": "music.game", "min_delay": 60, "max_delay": 180.5, "replace": false}, "format_version": "1.0.0"}"#;
        languages: bedrock_addon::families::languages::Languages, "texts/languages.json", r#"["en_US"]"#,
            [
                ("an element of the list", r#"["en_US", @]"#, "1"),
                ("an object in the list", r#"["en_US", {"zz_probe": @}]"#, "1/zz_probe"),
            ];
        language_names: bedrock_addon::families::languages::LanguageNames, "texts/language_names.json",
            r#"[["en_US", "English (United States)"], ["en_GB", "English (United Kingdom)"]]"#,
            [
                ("an element of a row", r#"[["en_US", "English (United States)", @]]"#, "0/2"),
                ("an object in the list", r#"[["en_US", "English (United States)"], {"zz_probe": @}]"#, "1/zz_probe"),
            ];
        ui_file: bedrock_addon::families::json_ui::UiFile, "ui/x.json",
            r#"{"namespace": "demo", "panel": {"type": "panel", "size": [5, 5], "controls": [{"child": {"type": "label", "text": "x"}}]}}"#;
        ui_defs: bedrock_addon::families::json_ui::UiDefsList, "ui/_ui_defs.json",
            r#"{"ui_defs": ["ui/b.json", "ui/a.json"], "note": {"kept": true}}"#;
        ui_global_variables: bedrock_addon::families::json_ui::GlobalVariables, "ui/_global_variables.json",
            r#"{"$b": [1, 0, 0], "$a": {"x": 1}, "$c": "text"}"#,
            [("a variable", r#"{"$b": @}"#, "$b"), ("an element of a variable", r#"{"$b": [1, @]}"#, "$b/1")];
        ui_texture_info: bedrock_addon::families::ui_texture_info::UiTextureInfo, "textures/ui/x.json",
            r#"{"nineslice_size": 1, "base_size": [4, 4]}"#;
        font_metadata: bedrock_addon::families::font::FontMetadata, "font/font_metadata.json",
            r#"{"version": 1, "fonts": [{"font_format": "ttf", "font_name": "A", "font_file": "font/a", "version": 1, "target_font_render_size": 32, "lowPerformanceCompatible": false, "generate_pages_on_start": [0, {"first_page": 1, "last_page": 2}]}, {"font_format": "bitmap", "font_name": "B", "ascii_font_file": "font/b", "unicode_file_prefix": "font/glyph_"}], "font_aliases": [{"alias": "X", "font_scale_factor_override": 1.5, "fonts": [{"font_reference": "A", "font_scale": 1, "font_language_code": "ja_JP", "font_ranges": [{"first": 0, "last": 255}]}]}]}"#;
        emoticons: bedrock_addon::families::font::EmoticonList, "font/emoticons.json",
            r#"[{"name": "a", "code": "0xE000"}]"#,
            [("an element of the list", r#"[{"name": "a", "code": "0xE000"}, @]"#, "1")];
        ddui_root: bedrock_addon::families::ddui::UiRoot, "ddui/root/test.json" in ddui(),
            r#"{"format_version": "1.21.130", "minecraft:ui-root": {"description": {"identifier": "demo:screen"}, "attribs": {"root_point": "minecraft:screen"}, "layout": {"markup": [{"component": "Panel", "children": [{"component": "Panel.Text", "attribs": {"text": "hi"}}]}]}}}"#;
        ddui_composition: bedrock_addon::families::ddui::UiComposition, "ddui/composition/test.json" in ddui(),
            r#"{"format_version": "1.21.130", "minecraft:ui-composition": {"description": {"identifier": "demo:c"}, "attribs": {}, "layout": {"markup": [{"component": "ExtensionPoint", "attribs": {"name": 5}}]}}}"#;
        ddui_style: bedrock_addon::families::ddui::UiStyle, "ddui/styling/test.json" in ddui(),
            r#"{"format_version": "beta", "minecraft:ui-style": {"description": {"identifier": "demo:style"}, "components": {"minecraft:form_panel": {"padding": [1, 2]}, "minecraft:form_label": {"color": [1, 2, 3, 4], "shadow": true}}}}"#;
        splashes: bedrock_addon::families::splashes::Splashes, "splashes.json",
            r#"{"canMerge": true, "splashes": ["a"], "conditional": [{"requires": {"platforms": ["x"]}, "splashes": ["b"]}]}"#,
            [("a splash", r#"{"splashes": ["a", @]}"#, "splashes/1"), ("canMerge", r#"{"canMerge": @, "splashes": ["a"]}"#, "canMerge")];
        loading_messages: bedrock_addon::families::loading_messages::LoadingMessages, "loading_messages.json",
            r#"{"beginner_loading_messages": ["a", {"message": "b", "condition": "x"}], "store_progress_tooltips": [{"k": 1}]}"#,
            [("a message", r#"{"beginner_loading_messages": ["a", @]}"#, "beginner_loading_messages/1")];
        credits: bedrock_addon::families::credits::Credits, "credits/credits.json",
            r#"[{"section": "A", "disciplines": [{"discipline": "d", "titles": [{"title": "t", "names": ["x"]}]}]}]"#,
            [("a name", r#"[{"section": "A", "disciplines": [{"discipline": "d", "titles": [{"title": "t", "names": ["x", @]}]}]}]"#, "0/disciplines/0/titles/0/names/1")];
        skin_pack: bedrock_addon::families::skin::SkinPack, "skins.json",
            r#"{"serialize_name": "x", "localization_name": "y", "skins": [{"localization_name": "S", "geometry": "geometry.humanoid.custom", "texture": "a.png", "type": "free", "cape": "c.png", "animations": {"a": "b"}, "enable_attachables": true, "hide_armor": false, "held_item_ignores_lighting": true}]}"#,
            [("a skin that is a number", r#"{"serialize_name": "x", "localization_name": "y", "skins": [@]}"#, "skins/0")];
        persona_piece_meta: bedrock_addon::families::persona::PieceMetadata, "pieces/hat/hat.meta.json",
            r##"{"piece_id": "bb29fbdf-5331-ca8e-01a4-b352cd46a324", "piece_name": "hat", "piece_type": "persona_hair", "piece_sub_type": "persona_head", "body_variant": "medium", "clothing_map": "c.png", "face_clothing_map": "f.png", "zone": ["head", "hair"], "texture_sources": [{"texture": "t", "tint_map": "m.tga", "animated": true, "frames": 2, "use_face_uv": true}], "geometry_sources": [{"geometry": "geometry.hat", "arm_size": "slim", "body_size": "medium", "side": "left", "zone": ["head"], "is_face": "true", "texture": "g.png", "tint_map": "gm.tga", "animated": false, "frames": 3}], "animation_sources": [{"name": "wave", "animationFile": "wave.animation.json"}], "tint_color": "#112233", "tint_base_color": {"r_color": "#862f0b", "g_color": "#102030", "b_color": "#405060", "a_color": "#708090"}, "allow_tint_override": false}"##;
        persona_piece_store_info: bedrock_addon::families::persona::PieceStoreInfo, "pieces/hat/hat.store_info.json",
            r#"{"piece_store_name": "n", "piece_thumbnail": "textures/persona_thumbnails/hat_Thumbnail_0", "kept": {"a": 1}}"#;

        }
    };
}
