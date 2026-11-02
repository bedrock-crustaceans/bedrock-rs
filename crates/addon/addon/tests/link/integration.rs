//! The six rule groups on one world: every group finds something in it, the pass tells the
//! same findings in the same order every time, and no reference is told by two groups. The
//! script group's rules are rules of the load (20 S1 to S7): its finding is the stack's
//! `scripts()` collection's, the same every time, and the pass does not repeat it.

use std::collections::BTreeMap;

use bedrock_addon::diagnostic::{Diagnostic, Diagnostics};
use bedrock_addon::load::LoadOptions;

use super::common::{behavior_pack, reference, resource_pack, stacks_of};

/// A block whose geometry, texture, loot table and culling rules nothing defines: the
/// content group's (20 B5, B9, B10) and the resource group's (R30).
const BLOCK: &str = r#"{"format_version": "1.26.20", "minecraft:block": {"description": {"identifier": "demo:lamp"}, "components": {
    "minecraft:geometry": {"identifier": "geometry.no_lamp", "culling": "demo:no_culling"},
    "minecraft:material_instances": {"*": {"texture": "no_lamp_tile"}},
    "minecraft:loot": "loot_tables/no_lamp.json"}}}"#;

/// An item whose icon and placed entity nothing defines: the resource group's (R32) and the
/// content group's (I3).
const ITEM: &str = r#"{"format_version": "1.26.30", "minecraft:item": {"description": {"identifier": "demo:wand"}, "components": {
    "minecraft:icon": "no_wand_icon",
    "minecraft:entity_placer": {"entity": "demo:no_placed"}}}}"#;

/// An entity whose loot table, taming item and transformation nothing defines: the entity
/// group's (E12, E15, E14).
const ENTITY: &str = r#"{"format_version": "1.26.50", "minecraft:entity": {"description": {"identifier": "demo:pet", "is_spawnable": true, "is_summonable": true}, "components": {
    "minecraft:loot": {"table": "loot_tables/no_pet.json"},
    "minecraft:tameable": {"tame_items": ["demo:no_treat"]},
    "minecraft:transformation": {"into": "demo:no_grown"}}}}"#;

/// A loot table whose entry names an item nothing defines: the content group's (L1).
const LOOT: &str = r#"{"pools": [{"rolls": 1, "entries": [{"type": "item", "name": "demo:no_drop", "weight": 1}]}]}"#;

/// A feature that places a feature nothing defines: the world group's (W1).
const FEATURE: &str = r#"{"format_version": "1.13.0", "minecraft:aggregate_feature": {"description": {"identifier": "demo:heap"}, "features": ["demo:no_feature"]}}"#;

/// The client entity of `demo:pet`: its material, particle, geometry, animation and render
/// controller are the visual group's (R1 to R5).
const CLIENT_ENTITY: &str = r#"{"format_version": "1.10.0", "minecraft:client_entity": {"description": {"identifier": "demo:pet",
    "materials": {"default": "no_material"},
    "textures": {"default": "textures/entity/no_pet"},
    "geometry": {"default": "geometry.no_pet"},
    "particle_effects": {"puff": "demo:no_puff"},
    "animations": {"walk": "animation.no_pet.walk"},
    "render_controllers": ["controller.render.no_pet"]}}}"#;

/// A client entity of no entity: the visual group's (R10).
const ORPHAN: &str = r#"{"format_version": "1.10.0", "minecraft:client_entity": {"description": {"identifier": "demo:no_entity"}}}"#;

/// An attachable of no item: the visual group's (R11).
const ATTACHABLE: &str = r#"{"format_version": "1.10.0", "minecraft:attachable": {"description": {"identifier": "demo:held", "item": "demo:no_item"}}}"#;

/// `blocks.json`: a block nothing defines, and a texture and a sound of the block above that
/// nothing defines: the resource group's (R27, R28, R29).
const BLOCKS_JSON: &str = r#"{"format_version": "1.21.40", "demo:no_block": {"textures": "no_tile"}, "demo:lamp": {"textures": "no_side_tile", "sound": "no_sound"}}"#;

/// A particle effect whose material and texture nothing defines: the resource group's (R38).
const PARTICLE: &str = r#"{"format_version": "1.10.0", "particle_effect": {"description": {"identifier": "demo:spark", "basic_render_parameters": {"material": "no_particle_material", "texture": "textures/particle/no_spark"}}, "components": {}}}"#;

/// The registries a rule compares with only when the stack has them: a material list and its
/// file (R1), a terrain atlas (R28) and an item atlas (R32), none with a name used above.
const REGISTRIES: [(&str, &str); 4] = [
    (
        "materials/common.json",
        r#"[{"path": "materials/entity.material"}]"#,
    ),
    (
        "materials/entity.material",
        r#"{"materials": {"version": "1.0.0", "entity_alphatest": {"vertexShader": "shaders/entity.vertex", "fragmentShader": "shaders/entity.fragment"}}}"#,
    ),
    (
        "textures/terrain_texture.json",
        r#"{"texture_name": "atlas.terrain", "texture_data": {"stone": {"textures": "textures/blocks/stone"}}}"#,
    ),
    (
        "textures/item_texture.json",
        r#"{"texture_name": "atlas.items", "texture_data": {"gem": {"textures": "textures/items/gem"}}}"#,
    ),
];

/// A behaviour pack whose script module names an entry the pack does not have: the script
/// group's (S1), told when the scripts are loaded.
#[cfg(feature = "script")]
fn script_pack() -> bedrock_addon::pack::Pack {
    use bedrock_addon::storage::PackPath;
    let manifest = format!(
        r#"{{"format_version": 2,
    "header": {{"name": "bp 3", "description": "", "uuid": "{}", "version": [1, 0, 0], "min_engine_version": [1, 21, 0]}},
    "modules": [{{"type": "script", "uuid": "{}", "version": [1, 0, 0], "entry": "scripts/no_entry.js"}}],
    "dependencies": [{{"module_name": "@minecraft/server", "version": "2.0.0"}}]}}"#,
        uuid::Uuid::from_u128(3),
        uuid::Uuid::from_u128(1_000_003),
    );
    let files = [
        (
            PackPath::from_static("manifest.json"),
            manifest.into_bytes(),
        ),
        (PackPath::from_static("scripts/main.js"), Vec::new()),
    ];
    bedrock_addon::pack::Pack::from_files(files, &bedrock_addon::pack::OpenOptions::default())
        .expect("a pack")
}

fn world() -> bedrock_addon::pack::StackSet {
    #[allow(unused_mut, reason = "the script pack is the feature's")]
    let mut behavior = vec![behavior_pack(
        1,
        &[
            ("blocks/lamp.json", BLOCK),
            ("items/wand.json", ITEM),
            ("entities/pet.json", ENTITY),
            ("loot_tables/drop.json", LOOT),
            ("features/heap.json", FEATURE),
        ],
    )];
    #[cfg(feature = "script")]
    behavior.push(script_pack());
    let resource = vec![resource_pack(
        2,
        &[
            ("entity/pet.entity.json", CLIENT_ENTITY),
            ("entity/orphan.entity.json", ORPHAN),
            ("attachables/orphan.json", ATTACHABLE),
            ("blocks.json", BLOCKS_JSON),
            ("particles/spark.particle.json", PARTICLE),
            REGISTRIES[0],
            REGISTRIES[1],
            REGISTRIES[2],
            REGISTRIES[3],
        ],
    )];
    stacks_of(behavior, resource, LoadOptions::default())
}

/// Where a finding is and what it is: the pack, the file, the JSON path and the code.
fn place(finding: &Diagnostic) -> (String, String, String, &'static str) {
    (
        finding
            .location
            .pack
            .as_ref()
            .map(|pack| pack.name().to_owned())
            .unwrap_or_default(),
        finding
            .location
            .path
            .as_ref()
            .map(|path| path.as_str().to_owned())
            .unwrap_or_default(),
        finding.location.json_path.to_string(),
        finding.code.name(),
    )
}

/// The names the reference findings of `found` are about.
fn names(found: &Diagnostics) -> Vec<String> {
    found
        .iter()
        .filter_map(reference)
        .map(|(_, name, _, _)| name)
        .collect()
}

#[test]
fn the_six_groups_on_one_world_tell_each_reference_once_and_the_same_every_time() {
    let set = world();
    let found = set.link();

    // The same findings in the same order: twice on one world, and on the world built again.
    let text = |found: &Diagnostics| format!("{found:#?}");
    assert_eq!(text(&found), text(&set.link()), "two passes over one world");
    assert_eq!(text(&found), text(&world().link()), "the world built twice");

    // No place holds two findings of one code: a reference two groups check is told once.
    let mut places: BTreeMap<_, Vec<String>> = BTreeMap::new();
    for finding in &found {
        places
            .entry(place(finding))
            .or_default()
            .push(finding.to_string());
    }
    let twice: Vec<_> = places.iter().filter(|(_, told)| told.len() > 1).collect();
    assert!(twice.is_empty(), "told more than once: {twice:#?}");

    // The script group's rule is the load's: told there, the same every time, and not again
    // by the pass.
    #[cfg(feature = "script")]
    {
        let told = |set: &bedrock_addon::pack::StackSet| -> Vec<String> {
            set.behavior
                .scripts()
                .iter()
                .flat_map(bedrock_addon::families::script::ScriptBundle::findings)
                .map(|finding| format!("{finding:?}"))
                .collect()
        };
        let first = told(&set);
        assert_eq!(first.len(), 1, "script (S1): the missing entry: {first:#?}");
        let at: Vec<String> = set
            .behavior
            .scripts()
            .iter()
            .flat_map(bedrock_addon::families::script::ScriptBundle::findings)
            .map(|finding| finding.location.json_path.to_string())
            .collect();
        assert_eq!(at, ["modules/0/entry"]);
        assert_eq!(first, told(&set), "two readings of one world");
        assert_eq!(first, told(&world()), "the world built twice");
        let repeated: Vec<_> = found
            .iter()
            .filter(|finding| first.contains(&format!("{finding:?}")))
            .collect();
        assert!(
            repeated.is_empty(),
            "the pass repeats a load rule: {repeated:#?}"
        );
        let about: Vec<_> = found
            .iter()
            .filter(|finding| {
                finding.location.path.as_ref().is_some_and(|path| {
                    path.as_str() == "manifest.json"
                        || std::path::Path::new(path.as_str())
                            .extension()
                            .is_some_and(|ext| ext.eq_ignore_ascii_case("js"))
                })
            })
            .collect();
        assert!(about.is_empty(), "the pass tells of a script: {about:#?}");
    }

    // Every group ran on it: each of these names is one only that group's rule tells.
    if cfg!(feature = "full") {
        let told = names(&found);
        for (group, name) in [
            ("entity (E15)", "demo:no_treat"),
            ("entity (E14)", "demo:no_grown"),
            ("content (B9)", "no_lamp_tile"),
            ("content (I3)", "demo:no_placed"),
            ("content (L1)", "demo:no_drop"),
            ("world (W1)", "demo:no_feature"),
            ("visual (R1)", "no_material"),
            ("visual (R2)", "demo:no_puff"),
            ("visual (R10)", "demo:no_entity"),
            ("visual (R11)", "demo:no_item"),
            ("resource (R27)", "demo:no_block"),
            ("resource (R28)", "no_side_tile"),
            ("resource (R30)", "demo:no_culling"),
            ("resource (R32)", "no_wand_icon"),
            ("resource (R38)", "no_particle_material"),
        ] {
            assert_eq!(
                told.iter().filter(|told| *told == name).count(),
                1,
                "{group}: `{name}` is told once; the pass told {told:#?}"
            );
        }
    }
}
