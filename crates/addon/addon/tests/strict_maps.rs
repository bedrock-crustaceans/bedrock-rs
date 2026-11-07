//! A JSON map level given an array (`[]`, which facet would read
//! as an empty map) is an `InvalidType` at that level, in every family that has a map level
//! read through core's `parse_at`. One test per family with a map level; the families without
//! one are listed below.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use bedrock_addon::diagnostic::{Diagnostic, Diagnostics, Kind};
use bedrock_addon::document::{Content, Document, Rejected};
use bedrock_addon::load::LoadContext;
use bedrock_addon::storage::PackPath;

/// The findings of a load, loaded or rejected.
#[allow(dead_code, reason = "used by the tests of the compiled families")]
fn findings<T>(outcome: Result<Document<T>, Rejected>) -> Diagnostics {
    match outcome {
        Ok(document) => document.diagnostics().clone(),
        Err(rejected) => rejected.diagnostics,
    }
}

/// The `InvalidType` findings at `path`.
#[allow(dead_code, reason = "used by the tests of the compiled families")]
fn invalid_type_at<'a>(d: &'a Diagnostics, path: &str) -> Vec<&'a Diagnostic> {
    d.iter()
        // By legacy name: the variant and the definitions that were it.
        .filter(|f| f.code.kind() == Kind::WrongType)
        .filter(|f| f.location.json_path.to_string() == path)
        .collect()
}

/// Loads `text` as content `T` at `path` and requires one `InvalidType` at `at`.
#[allow(dead_code, reason = "used by the tests of the compiled families")]
#[track_caller]
fn refuses<T: Content>(path: &'static str, text: &str, at: &str) {
    let d = findings(T::parse(
        &PackPath::from_static(path),
        text.as_bytes(),
        &LoadContext::detached(),
    ));
    assert_eq!(invalid_type_at(&d, at).len(), 1, "{text}\n{d:#?}");
}

#[cfg(feature = "block")]
#[test]
fn block_components_that_are_an_array_are_a_type_error() {
    refuses::<bedrock_addon::families::block::Block>(
        "blocks/x.json",
        r#"{"format_version": "1.26.20", "minecraft:block": {"description": {"identifier": "demo:x"}, "components": []}}"#,
        "minecraft:block/components",
    );
}

// read from the merged JSON with the JsonCpp fork's rules, where an array for a table is
// vanilla's own finding (`sound_definitions section is malformed`) or walks nothing.
#[cfg(feature = "server_sound_definitions")]
#[test]
fn sound_metadata_tags_that_are_an_array_are_a_type_error() {
    let mut cx = LoadContext::detached();
    cx.beta_apis = true;
    let d = findings(bedrock_addon::families::server_sound_definitions::SoundMetadataFile::parse(
        &PackPath::from_static("sounds/sound_definitions.json"),
        br#"{"format_version": "1.26.60", "minecraft:server_sound_definitions": [{"sound_event_name": "demo:x", "tags": []}]}"#,
        &cx,
    ));
    assert_eq!(
        invalid_type_at(&d, "minecraft:server_sound_definitions/0/tags").len(),
        1,
        "{d:#?}"
    );
}

#[cfg(feature = "item")]
#[test]
fn item_components_that_are_an_array_are_a_type_error() {
    refuses::<bedrock_addon::families::item::Item>(
        "items/x.json",
        r#"{"format_version": "1.21.60", "minecraft:item": {"description": {"identifier": "demo:x"}, "components": []}}"#,
        "minecraft:item/components",
    );
}

#[cfg(feature = "entity")]
#[test]
fn entity_component_groups_that_are_an_array_are_a_type_error() {
    refuses::<bedrock_addon::families::entity::Entity>(
        "entities/x.json",
        r#"{"format_version": "1.26.50", "minecraft:entity": {"description": {"identifier": "demo:x"}, "components": {}, "component_groups": []}}"#,
        "minecraft:entity/component_groups",
    );
}

#[cfg(feature = "biome")]
#[test]
fn biome_components_that_are_an_array_are_a_type_error() {
    refuses::<bedrock_addon::families::biome::Biome>(
        "biomes/x.biome.json",
        r#"{"format_version": "1.26.30", "minecraft:biome": {"description": {"identifier": "demo:x"}, "components": []}}"#,
        "minecraft:biome/components",
    );
}

#[cfg(feature = "client_biome")]
#[test]
fn client_biome_components_that_are_an_array_are_a_type_error() {
    refuses::<bedrock_addon::families::client_biome::ClientBiome>(
        "biomes/x.client_biome.json",
        r#"{"format_version": "1.21.130", "minecraft:client_biome": {"description": {"identifier": "demo:x"}, "components": []}}"#,
        "minecraft:client_biome/components",
    );
    // Below 1.21.130 the struct reads the whole document.
    refuses::<bedrock_addon::families::client_biome::ClientBiome>(
        "biomes/x.client_biome.json",
        r#"{"format_version": "1.21.40", "minecraft:client_biome": {"description": {"identifier": "demo:x"}, "components": []}}"#,
        "minecraft:client_biome/components",
    );
}

#[cfg(feature = "recipe")]
#[test]
fn a_shaped_recipe_key_that_is_an_array_is_a_type_error() {
    refuses::<bedrock_addon::families::recipe::Recipe>(
        "recipes/x.json",
        r##"{"format_version": "1.20.10", "minecraft:recipe_shaped": {"description": {"identifier": "demo:r"}, "tags": ["crafting_table"], "pattern": ["#"], "key": [], "result": "minecraft:stone"}}"##,
        "minecraft:recipe_shaped/key",
    );
}

#[cfg(feature = "animation")]
#[test]
fn animations_that_are_an_array_are_a_type_error() {
    refuses::<bedrock_addon::families::animation::AnimationFile>(
        "animations/x.json",
        r#"{"format_version": "1.8.0", "animations": []}"#,
        "animations",
    );
}

// Aim assist's priority tables are map levels too.
#[cfg(feature = "aim_assist")]
#[test]
fn aim_assist_priorities_that_are_an_array_are_a_type_error() {
    refuses::<bedrock_addon::families::aim_assist::AimAssistCategories>(
        "aim_assist/categories/categories.json",
        r#"{"format_version": "1.21.50", "minecraft:aim_assist_categories": {"categories": [{"name": "demo:c", "priorities": {"entities": []}}]}}"#,
        "minecraft:aim_assist_categories/categories/0/priorities/entities",
    );
}

#[cfg(feature = "particle")]
#[test]
fn particle_map_levels_that_are_an_array_are_a_type_error() {
    let effect = |members: &str| {
        format!(
            r#"{{"format_version": "1.26.10", "particle_effect": {{"description": {{"identifier": "demo:x", "basic_render_parameters": {{"material": "m", "texture": "t"}}}}, {members}}}}}"#
        )
    };
    let components = |components: &str| effect(&format!(r#""components": {{{components}}}"#));
    for (text, at) in [
        (effect(r#""components": []"#), "components"),
        (effect(r#""components": {}, "curves": []"#), "curves"),
        (effect(r#""components": {}, "events": []"#), "events"),
        (
            effect(
                r#""components": {}, "curves": {"variable.c": {"type": "bezier_chain", "input": 0, "nodes": []}}"#,
            ),
            "curves/variable.c/nodes",
        ),
        (
            components(r#""minecraft:emitter_lifetime_events": {"timeline": []}"#),
            "components/minecraft:emitter_lifetime_events/timeline",
        ),
        (
            components(r#""minecraft:emitter_lifetime_events": {"travel_distance_events": []}"#),
            "components/minecraft:emitter_lifetime_events/travel_distance_events",
        ),
        (
            components(r#""minecraft:particle_lifetime_events": {"timeline": []}"#),
            "components/minecraft:particle_lifetime_events/timeline",
        ),
    ] {
        refuses::<bedrock_addon::families::particle::ParticleEffect>(
            "particles/x.json",
            &text,
            &format!("particle_effect/{at}"),
        );
    }
}

#[cfg(feature = "render_controller")]
#[test]
fn render_controller_maps_that_are_an_array_are_a_type_error() {
    refuses::<bedrock_addon::families::render_controller::RenderControllerFile>(
        "render_controllers/x.json",
        r#"{"format_version": "1.8.0", "render_controllers": []}"#,
        "render_controllers",
    );
    refuses::<bedrock_addon::families::render_controller::RenderControllerFile>(
        "render_controllers/x.json",
        r#"{"format_version": "1.8.0", "render_controllers": {"controller.render.x": {"geometry": "geometry.default", "textures": ["texture.default"], "arrays": {"textures": []}}}}"#,
        "render_controllers/controller.render.x/arrays/textures",
    );
}

#[cfg(feature = "fog")]
#[test]
fn fog_keyframes_that_are_an_array_are_a_type_error() {
    refuses::<bedrock_addon::families::fog::FogSettings>(
        "fogs/x.json",
        r#"{"format_version": "1.26.20", "minecraft:fog_settings": {"description": {"identifier": "demo:x"}, "volumetric": {"density": {"air": {"uniform": true, "max_density": []}}}}}"#,
        "minecraft:fog_settings/volumetric/density/air/max_density",
    );
}

// the slot maps of a client entity, at each of its versions, and the object form of a
// list entry.
#[cfg(feature = "client_entity")]
#[test]
fn client_entity_slot_maps_that_are_an_array_are_a_type_error() {
    for version in ["1.8.0", "1.10.0", "1.26.0"] {
        for map in [
            "materials",
            "textures",
            "geometry",
            "particle_effects",
            "particle_emitters",
            "animations",
            "sound_effects",
        ] {
            refuses::<bedrock_addon::families::client_entity::ClientEntity>(
                "entity/x.entity.json",
                &format!(
                    r#"{{"format_version": "{version}", "minecraft:client_entity": {{"description": {{"identifier": "demo:x", "{map}": []}}}}}}"#
                ),
                &format!("minecraft:client_entity/description/{map}"),
            );
        }
    }
    refuses::<bedrock_addon::families::client_entity::ClientEntity>(
        "entity/x.entity.json",
        r#"{"format_version": "1.26.0", "minecraft:client_entity": {"description": {"identifier": "demo:x", "scripts": {"variables": []}}}}"#,
        "minecraft:client_entity/description/scripts/variables",
    );
    refuses::<bedrock_addon::families::client_entity::ClientEntity>(
        "entity/x.entity.json",
        r#"{"format_version": "1.8.0", "minecraft:client_entity": {"description": {"identifier": "demo:x", "animations": {"a": "animation.a"}, "animation_controllers": [[]]}}}"#,
        "minecraft:client_entity/description/animation_controllers/0",
    );
}

#[cfg(feature = "atomic_client_entity")]
#[test]
fn atomic_client_entity_maps_that_are_an_array_are_a_type_error() {
    for map in [
        "materials",
        "textures",
        "geometry",
        "animations",
        "particle_effects",
        "sound_effects",
    ] {
        refuses::<bedrock_addon::families::atomic_client_entity::AtomicClientEntity>(
            "atomic_client_entities/x.json",
            &format!(
                r#"{{"format_version": "beta", "minecraft:atomic_client_entity": {{"description": {{"identifier": "demo:x", "{map}": []}}}}}}"#
            ),
            &format!("minecraft:atomic_client_entity/description/{map}"),
        );
    }
}

/// The families with no JSON map level, each the crate `bedrock_addon_family_<name>` under
/// `crates/addon/families/`, and the map-typed fields their sources may hold because no JSON
/// object is read into them (an NBT compound, a `.lang` file, a registry or an index built in
/// memory). A new map-typed field in one of them fails here, so its JSON level gets a test
/// above.
const MAP_FREE: [(&str, &[&str]); 12] = [
    ("spawn_rules", &[]),
    ("spawn_groups", &[]),
    ("feature", &[]),
    (
        "feature_rules",
        &["pub unknown_passes: indexmap::IndexMap<String, Registry<FeatureRule>>,"],
    ),
    ("dimension", &[]),
    ("dialogue", &[]),
    ("trade", &[]),
    (
        "structure",
        &[
            "pub palette: IndexMap<String, Palette>,",
            "pub block_position_data: BTreeMap<i32, BlockPositionData>,",
        ],
    ),
    ("loot", &[]),
    ("function", &[]),
    (
        "poi",
        &[
            "pub(crate) blocks: IndexMap<BlockId, PoiTypeId>,",
            "pub(crate) tags: IndexMap<PoiTagId, TagNode>,",
        ],
    ),
    ("lang", &["pub strings: BTreeMap<String, String>,"]),
];

/// Every `.rs` file below `dir`.
fn rust_files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// The field declarations of a map type in `source`, trimmed.
fn map_fields(source: &str) -> Vec<String> {
    source
        .lines()
        .map(str::trim)
        .filter(|line| {
            let Some((name, ty)) = line.split_once(": ") else {
                return false;
            };
            let name = name
                .trim_start_matches("pub(crate) ")
                .trim_start_matches("pub ");
            let ty = ty
                .trim_start_matches("Option<")
                .trim_start_matches("indexmap::")
                .trim_start_matches("std::collections::");
            !name.is_empty()
                && name.chars().all(|c| c.is_ascii_lowercase() || c == '_')
                && ["IndexMap<", "HashMap<", "BTreeMap<"]
                    .iter()
                    .any(|map| ty.starts_with(map))
        })
        .map(str::to_owned)
        .collect()
}

#[test]
fn the_families_without_a_json_map_level_stay_without_one() {
    let families = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../families");
    if !families.is_dir() {
        eprintln!("skipped: no crates/addon/families/ beside this crate (a package built alone)");
        return;
    }
    for (family, allowed) in MAP_FREE {
        let mut files = Vec::new();
        rust_files(&families.join(family).join("src"), &mut files);
        assert!(!files.is_empty(), "{family}");
        let mut found: Vec<String> = files
            .iter()
            .flat_map(|file| map_fields(&std::fs::read_to_string(file).unwrap()))
            .collect();
        found.sort();
        let mut allowed: Vec<String> = allowed.iter().map(|s| (*s).to_owned()).collect();
        allowed.sort();
        assert_eq!(
            found, allowed,
            "{family}: a new map-typed field needs a strict_maps test"
        );
    }
}

#[test]
fn the_map_field_scan_sees_a_map_field() {
    assert_eq!(
        map_fields("    pub levels: Option<IndexMap<String, Level>>,\n    let x: u8 = 1;"),
        ["pub levels: Option<IndexMap<String, Level>>,"]
    );
    assert_eq!(
        map_fields("pub(crate) seen: std::collections::HashMap<Id, usize>,"),
        ["pub(crate) seen: std::collections::HashMap<Id, usize>,"]
    );
    assert!(map_fields("let map: IndexMap<String, u8> = IndexMap::new();").is_empty());
}

#[cfg(feature = "deferred_settings")]
#[test]
fn deferred_settings_map_levels_that_are_an_array_are_a_type_error() {
    use bedrock_addon::families::deferred_settings::{CubemapSettings, LocalLightSettings};
    // The block map of the local lights.
    refuses::<LocalLightSettings>(
        "local_lighting/local_lighting.json",
        r#"{"format_version": "1.21.120", "minecraft:local_light_settings": []}"#,
        "minecraft:local_light_settings",
    );
    // A keyframe map (its own text, "Expected keyframes.").
    refuses::<CubemapSettings>(
        "cubemaps/x.json",
        r#"{"format_version": "1.21.130", "minecraft:cubemap_settings": {"lighting": {"ambient_light_illuminance": []}}}"#,
        "minecraft:cubemap_settings/lighting/ambient_light_illuminance",
    );
}

#[cfg(feature = "geometry")]
#[test]
fn geometry_locators_that_are_an_array_are_a_type_error() {
    refuses::<bedrock_addon::families::geometry::GeometryFile>(
        "models/entity/x.json",
        r#"{"format_version": "1.21.0", "minecraft:geometry": [{"description": {"identifier": "geometry.x"}, "bones": [{"name": "body", "locators": []}]}]}"#,
        "minecraft:geometry/0/bones/0/locators",
    );
}

#[cfg(feature = "geometry")]
#[test]
fn geometry_item_display_transforms_that_are_an_array_are_a_type_error() {
    refuses::<bedrock_addon::families::geometry::GeometryFile>(
        "models/entity/x.json",
        r#"{"format_version": "1.21.0", "minecraft:geometry": [{"description": {"identifier": "geometry.x"}, "item_display_transforms": []}]}"#,
        "minecraft:geometry/0/item_display_transforms",
    );
}

// `parse_at`; its map levels refuse an array all the same: the root, the wrapper's `materials`
// (no object, so no wrapper and no material) and a material's own value.
#[cfg(feature = "material")]
#[test]
fn a_material_map_that_is_an_array_is_a_type_error() {
    use bedrock_addon::families::material::MaterialFile;
    refuses::<MaterialFile>("materials/x.material", "[]", "");
    refuses::<MaterialFile>("materials/x.material", r#"{"materials": []}"#, "materials");
    refuses::<MaterialFile>(
        "materials/x.material",
        r#"{"materials": {"version": "1.0.0", "x": []}}"#,
        // The path the member has in the file, the wrapper's member included.
        "materials/x",
    );
    refuses::<MaterialFile>(
        "materials/x.material",
        r#"{"x": {"frontFace": []}}"#,
        "x/frontFace",
    );
}

// JSON, opaque); its reader takes every object level as raw members after looking at the
// first token itself, so an array at each of them is the same type error.
#[cfg(feature = "ddui")]
#[test]
fn data_driven_ui_object_levels_that_are_an_array_are_a_type_error() {
    use bedrock_addon::families::ddui::{UiComposition, UiRoot};
    let root = |description: &str, attribs: &str, layout: &str| {
        format!(
            r#"{{"format_version": "1.21.130", "minecraft:ui-root": {{"description": {description}, "attribs": {attribs}, "layout": {layout}}}}}"#
        )
    };
    let description = r#"{"identifier": "demo:screen"}"#;
    let attribs = r#"{"root_point": "minecraft:screen"}"#;
    let layout = r#"{"markup": [{"component": "Panel", "children": []}]}"#;
    for (text, at) in [
        (root("[]", attribs, layout), "minecraft:ui-root/description"),
        (root(description, "[]", layout), "minecraft:ui-root/attribs"),
        (root(description, attribs, "[]"), "minecraft:ui-root/layout"),
        (
            root(description, attribs, r#"{"markup": [[]]}"#),
            "minecraft:ui-root/layout/markup/0",
        ),
        (
            root(
                description,
                attribs,
                r#"{"markup": [{"component": "Panel", "children": [[]]}]}"#,
            ),
            "minecraft:ui-root/layout/markup/0/children/0",
        ),
    ] {
        refuses::<UiRoot>("ddui/root/x.json", &text, at);
    }
    refuses::<UiComposition>(
        "ddui/composition/x.json",
        r#"{"format_version": "1.21.130", "minecraft:ui-composition": {"description": {"identifier": "demo:part"}, "attribs": [], "layout": {"markup": [{"component": "Form.Divider"}]}}}"#,
        "minecraft:ui-composition/attribs",
    );
}

/// `blocks.json`: the entries are a map level (block name to entry), read member by member; an
/// entry given as an array is an `InvalidType` at its key (L4), a warning, since the client skips the member and reads the rest of the file. The file
/// itself given as an array is skipped whole (`MalformedInput`, L2). Culling rules have no map
/// level.
#[cfg(feature = "blocks_json")]
#[test]
fn a_blocks_json_entry_that_is_an_array_is_a_type_error() {
    refuses::<bedrock_addon::families::blocks_json::BlocksJson>(
        "blocks.json",
        r#"{"format_version": "1.21.40", "obsidian": [], "glass": {"textures": "g"}}"#,
        "obsidian",
    );
    let d = findings(bedrock_addon::families::blocks_json::BlocksJson::parse(
        &PackPath::from_static("blocks.json"),
        b"[]",
        &LoadContext::detached(),
    ));
    assert!(
        d.iter()
            // By legacy name: the variant and the definitions that were it.
            .any(|f| matches!(f.code.id(), "blocks_json/not_json")),
        "{d:#?}"
    );
}

// the slot maps of an attachable, at each of its versions, and
// the object form of a list entry. `item` given an array is rule 15's `InvalidValue`
// (tests/attachable/rows.rs::a15_item_wrong_type), not an `InvalidType`.
#[cfg(feature = "attachable")]
#[test]
fn attachable_slot_maps_that_are_an_array_are_a_type_error() {
    use bedrock_addon::families::attachable::Attachable;
    for version in ["1.8.0", "1.10.0", "1.26.0"] {
        for map in [
            "materials",
            "textures",
            "geometry",
            "particle_effects",
            "particle_emitters",
            "animations",
            "sound_effects",
        ] {
            refuses::<Attachable>(
                "attachables/x.json",
                &format!(
                    r#"{{"format_version": "{version}", "minecraft:attachable": {{"description": {{"identifier": "demo:x", "{map}": []}}}}}}"#
                ),
                &format!("minecraft:attachable/description/{map}"),
            );
        }
    }
    refuses::<Attachable>(
        "attachables/x.json",
        r#"{"format_version": "1.26.0", "minecraft:attachable": {"description": {"identifier": "demo:x", "scripts": {"variables": []}}}}"#,
        "minecraft:attachable/description/scripts/variables",
    );
    refuses::<Attachable>(
        "attachables/x.json",
        r#"{"format_version": "1.8.0", "minecraft:attachable": {"description": {"identifier": "demo:x", "animation_controllers": [[]]}}}"#,
        "minecraft:attachable/description/animation_controllers/0",
    );
    refuses::<Attachable>(
        "attachables/x.json",
        r#"{"format_version": "1.10.0", "minecraft:attachable": {"description": {"identifier": "demo:x", "render_controllers": [[]]}}}"#,
        "minecraft:attachable/description/render_controllers/0",
    );
}

// from the file's root as the game's reader takes it); the one object-or-other member is a
// tint, where an array is the family's own M10.
#[cfg(feature = "persona")]
#[test]
fn persona_tint_that_is_an_array_is_a_type_error() {
    use bedrock_addon::families::persona::PieceMetadata;
    for member in ["tint_color", "tint_base_color"] {
        refuses::<PieceMetadata>(
            "pieces/hat/hat.meta.json",
            &format!(
                r#"{{"piece_id": "x", "piece_name": "hat", "piece_type": "persona_hair", "{member}": []}}"#
            ),
            member,
        );
    }
}
