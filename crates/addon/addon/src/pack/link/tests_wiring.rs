//! Each lookup against a stack that has one definition of its kind: the lookup reads the
//! collection it says it reads.

#![allow(
    unused_imports,
    dead_code,
    reason = "each feature set uses part of the helpers"
)]

use bedrock_addon_core::diagnostic::ReferenceHint;
use bedrock_addon_core::id::{Id, Kind};
use bedrock_addon_core::load::{Experiment, Experiments, LoadOptions};
use bedrock_addon_core::pack_identity::PackSide;

use super::support::{hosted, pack, set, set_of, set_with};
use crate::pack::StackSet;
use crate::pack::link::{LinkCx, LinkOptions, Target};

const ALL: LinkOptions = LinkOptions {
    lints: true,
    client: true,
};

fn id<K: Kind>(text: &str) -> Id<K> {
    Id::parse(text)
}

/// The file a target is defined in.
fn file(target: &Target<'_>) -> Option<String> {
    target.origin().map(|origin| origin.path.to_string())
}

/// The file a target is defined in, with the side of its stack.
fn place(target: &Target<'_>) -> Option<(PackSide, String)> {
    target
        .origin()
        .map(|origin| (origin.side, origin.path.to_string()))
}

fn case(name: &str) -> Target<'static> {
    Target::Missing(Some(ReferenceHint::CaseMismatch(name.to_owned())))
}

const NO_NAMESPACE: Target<'static> = Target::Missing(Some(ReferenceHint::MissingNamespace));
const MISSING: Target<'static> = Target::Missing(None);

/// What the collections loaded so far said: the message of a failed "defined" assertion, so a
/// fixture the load rejected is told from a lookup that reads the wrong collection.
fn said(set: &StackSet) -> String {
    let behavior = set.behavior.report();
    let resource = set.resource.report();
    let all: Vec<_> = behavior.iter().chain(resource.iter()).collect();
    format!("the loaded collections reported: {all:#?}")
}

/// The options of a world with `experiment` on.
fn with_experiment(experiment: Experiment) -> LoadOptions {
    let mut experiments = Experiments::new();
    experiments.insert(experiment);
    LoadOptions {
        experiments,
        ..LoadOptions::default()
    }
}

// ---- blocks ----------------------------------------------------------------------------------

#[cfg(all(feature = "block", feature = "blocks_json"))]
#[test]
fn a_client_block_is_found_by_its_blocks_json_key() {
    let blocks = r#"{"format_version": "1.21.40", "demo:lamp": {"textures": "lamp"}}"#;
    let set = set(&[], &[("blocks.json", blocks)]);
    let cx = LinkCx::new(&set, ALL);
    let found = cx.client_block(&id("demo:lamp"));
    assert_eq!(
        place(&found),
        Some((PackSide::Resource, "blocks.json".to_owned())),
        "{}",
        said(&set)
    );
    assert_eq!(cx.client_block(&id("demo:other")), MISSING);
    assert_eq!(cx.client_block(&id("demo:Lamp")), case("demo:lamp"));
    assert_eq!(cx.client_block(&id("lamp")), NO_NAMESPACE);
    // The entries have no list of the game's behind them: a block of the game without an entry
    // has none.
    assert_eq!(cx.client_block(&id("minecraft:stone")), MISSING);
    // The behaviour stack's blocks are another collection.
    assert_eq!(cx.block(&id("demo:lamp")), MISSING);
}

#[cfg(all(feature = "block", feature = "voxel_shape"))]
#[test]
fn a_voxel_shape_is_found_by_its_identifier() {
    let shape = r#"{"format_version": "1.21.110", "minecraft:voxel_shape": {"description": {"identifier": "demo:half"}, "shape": {"boxes": [{"min": [0, 0, 0], "max": [16, 8, 16]}]}}}"#;
    let set = set(&[("shapes/half.json", shape)], &[]);
    let cx = LinkCx::new(&set, ALL);
    let found = cx.voxel_shape(&id("demo:half"));
    assert_eq!(
        place(&found),
        Some((PackSide::Behavior, "shapes/half.json".to_owned())),
        "{}",
        said(&set)
    );
    assert_eq!(cx.voxel_shape(&id("demo:whole")), MISSING);
    assert_eq!(cx.voxel_shape(&id("demo:Half")), case("demo:half"));
    #[cfg(feature = "block_culling")]
    assert_eq!(cx.culling_rules(&id("demo:half")), MISSING);
}

#[cfg(all(feature = "block", feature = "block_culling"))]
#[test]
fn culling_rules_are_found_by_their_identifier() {
    let rules = r#"{"format_version": "1.21.80", "minecraft:block_culling_rules": {"description": {"identifier": "demo:lamp_culling"}, "rules": [{"geometry_part": {"bone": "shade"}, "direction": "up"}]}}"#;
    let set = set(&[], &[("block_culling/lamp.json", rules)]);
    let cx = LinkCx::new(&set, ALL);
    let found = cx.culling_rules(&id("demo:lamp_culling"));
    assert_eq!(
        place(&found),
        Some((PackSide::Resource, "block_culling/lamp.json".to_owned())),
        "{}",
        said(&set)
    );
    assert_eq!(cx.culling_rules(&id("demo:other_culling")), MISSING);
    assert_eq!(
        cx.culling_rules(&id("demo:Lamp_Culling")),
        case("demo:lamp_culling")
    );
    #[cfg(feature = "voxel_shape")]
    assert_eq!(cx.voxel_shape(&id("demo:lamp_culling")), MISSING);
}

// ---- entities, spawn rules, biomes -----------------------------------------------------------

#[cfg(feature = "client_entity")]
#[test]
fn a_client_entity_is_found_by_its_identifier() {
    let pet = r#"{"format_version": "1.10.0", "minecraft:client_entity": {"description": {"identifier": "demo:pet"}}}"#;
    let set = set(&[], &[("entity/pet.entity.json", pet)]);
    let cx = LinkCx::new(&set, ALL);
    let found = cx.client_entity(&id("demo:pet"));
    assert_eq!(
        place(&found),
        Some((PackSide::Resource, "entity/pet.entity.json".to_owned())),
        "{}",
        said(&set)
    );
    assert_eq!(cx.client_entity(&id("demo:pat")), MISSING);
    assert_eq!(cx.client_entity(&id("demo:Pet")), case("demo:pet"));
    // The game's own entities have their client entities.
    assert_eq!(cx.client_entity(&id("minecraft:pig")), Target::BuiltIn);
    // The behaviour stack's entities are another collection.
    assert_eq!(cx.entity(&id("demo:pet")), MISSING);
}

#[cfg(feature = "spawn_rules")]
#[test]
fn spawn_rules_are_found_by_their_entity() {
    let rules = r#"{"format_version": "1.8.0", "minecraft:spawn_rules": {"description": {"identifier": "demo:pet", "population_control": "animal"}, "conditions": [{"minecraft:weight": {"default": 1}}]}}"#;
    let set = set(&[("spawn_rules/pet.json", rules)], &[]);
    let cx = LinkCx::new(&set, ALL);
    let found = cx.spawn_rules(&id("demo:pet"));
    assert_eq!(
        place(&found),
        Some((PackSide::Behavior, "spawn_rules/pet.json".to_owned())),
        "{}",
        said(&set)
    );
    assert_eq!(cx.spawn_rules(&id("demo:pat")), MISSING);
    assert_eq!(cx.spawn_rules(&id("demo:Pet")), case("demo:pet"));
    // No list of the game's spawn rules: an entity of the game has none here.
    assert_eq!(cx.spawn_rules(&id("minecraft:pig")), MISSING);
}

#[cfg(feature = "biome")]
#[test]
fn a_biome_is_a_packs_the_games_or_missing() {
    let moor = r#"{"format_version": "1.26.30", "minecraft:biome": {"description": {"identifier": "demo:moor"}, "components": {"minecraft:climate": {"temperature": 0.5}}}}"#;
    let set = set(&[("biomes/moor.json", moor)], &[]);
    let cx = LinkCx::new(&set, ALL);
    let found = cx.biome(&id("demo:moor"));
    assert_eq!(
        place(&found),
        Some((PackSide::Behavior, "biomes/moor.json".to_owned())),
        "{}",
        said(&set)
    );
    // Without a host the crate's own list answers, a bare name being `minecraft:`'s.
    assert_eq!(cx.biome(&id("minecraft:plains")), Target::BuiltIn);
    assert_eq!(cx.biome(&id("plains")), Target::BuiltIn);
    assert_eq!(cx.biome(&id("minecraft:no_such_biome")), MISSING);
    assert_eq!(cx.biome(&id("demo:fen")), MISSING);
    assert_eq!(cx.biome(&id("demo:Moor")), case("demo:moor"));
}

// ---- world generation ------------------------------------------------------------------------

#[cfg(feature = "feature")]
#[test]
fn a_feature_is_found_by_its_identifier() {
    let vein = r#"{"format_version": "1.13.0", "minecraft:ore_feature": {"description": {"identifier": "demo:vein"}, "count": 16}}"#;
    let set = set(&[("features/vein.json", vein)], &[]);
    let cx = LinkCx::new(&set, ALL);
    let found = cx.feature(&id("demo:vein"));
    assert_eq!(
        place(&found),
        Some((PackSide::Behavior, "features/vein.json".to_owned())),
        "{}",
        said(&set)
    );
    assert_eq!(cx.feature(&id("demo:lode")), MISSING);
    assert_eq!(cx.feature(&id("demo:Vein")), case("demo:vein"));
    assert_eq!(cx.feature(&id("vein")), NO_NAMESPACE);
    // A feature is no jigsaw structure, and the structure features of the game are no feature
    // documents.
    assert_eq!(cx.jigsaw_structure(&id("demo:vein")), MISSING);
    assert_eq!(cx.feature(&id("minecraft:ancient_city")), MISSING);
}

/// The features the game registers in code (no file of the game defines them, and its
/// own feature rules and features place them) are the game's without a host; a host that
/// answers for the kind is believed, as for every list of the game's names.
#[cfg(feature = "feature")]
#[test]
fn a_feature_the_game_registers_in_code_is_the_games_without_a_host() {
    const CODE: [&str; 4] = [
        "minecraft:legacy:forest_tree_feature",
        "minecraft:kelp_feature",
        "minecraft:_basalt_pillar_feature",
        "minecraft:ice_spike_feature",
    ];
    let plain = set(&[], &[]);
    let cx = LinkCx::new(&plain, ALL);
    for name in CODE {
        assert_eq!(cx.feature(&id(name)), Target::BuiltIn, "{name}");
    }
    // As written; a feature a file of the game's behaviour pack defines is not in the
    // list (the pack or a host answers for it), and a feature type is no feature.
    assert_eq!(cx.feature(&id("minecraft:Kelp_Feature")), MISSING);
    assert_eq!(cx.feature(&id("minecraft:andesite_feature")), MISSING);
    assert_eq!(cx.feature(&id("minecraft:ore_feature")), MISSING);
    assert_eq!(cx.feature(&id("demo:kelp_feature")), MISSING);
    let hosted = set_with(
        &[],
        &[],
        &hosted(&[("feature", "minecraft:andesite_feature")]),
    );
    let cx = LinkCx::new(&hosted, ALL);
    assert_eq!(
        cx.feature(&id("minecraft:andesite_feature")),
        Target::BuiltIn
    );
    assert_eq!(cx.feature(&id("minecraft:kelp_feature")), MISSING);
}

/// One document of each of the four jigsaw kinds, each named `demo:ruin`: the structure starts
/// in the pool and the set places the structure.
#[cfg(feature = "jigsaw")]
const JIGSAW: [(&str, &str); 4] = [
    (
        "worldgen/template_pools/ruin.json",
        r#"{"format_version": "1.21.20", "minecraft:template_pool": {"description": {"identifier": "demo:ruin"}, "elements": [{"element": {"element_type": "minecraft:single_pool_element", "location": "demo/ruin"}, "weight": 1}]}}"#,
    ),
    (
        "worldgen/processors/ruin.json",
        r#"{"format_version": "1.21.20", "minecraft:processor_list": {"description": {"identifier": "demo:ruin"}, "processors": [{"processor_type": "minecraft:rule", "rules": [{"input_predicate": {"predicate_type": "minecraft:block_match", "block": "minecraft:gravel"}, "output_state": {"name": "minecraft:dirt"}}]}]}}"#,
    ),
    (
        "worldgen/structures/ruin.json",
        r#"{"format_version": "1.21.20", "minecraft:jigsaw": {"description": {"identifier": "demo:ruin"}, "step": "surface_structures", "start_pool": "demo:ruin", "max_depth": 4, "start_height": {"type": "constant", "value": {"absolute": 0}}}}"#,
    ),
    (
        "worldgen/structure_sets/ruin.json",
        r#"{"format_version": "1.21.20", "minecraft:structure_set": {"description": {"identifier": "demo:ruin"}, "placement": {"type": "minecraft:random_spread", "salt": 83469867, "separation": 8, "spacing": 34, "spread_type": "triangular"}, "structures": [{"structure": "demo:ruin", "weight": 1}]}}"#,
    ),
];

#[cfg(feature = "jigsaw")]
#[test]
fn a_template_pool_is_found_by_its_identifier() {
    let set = set(&JIGSAW, &[]);
    let cx = LinkCx::new(&set, ALL);
    let found = cx.template_pool(&id("demo:ruin"));
    assert_eq!(
        place(&found),
        Some((
            PackSide::Behavior,
            "worldgen/template_pools/ruin.json".to_owned()
        )),
        "{}",
        said(&set)
    );
    assert_eq!(cx.template_pool(&id("demo:fort")), MISSING);
    assert_eq!(cx.template_pool(&id("demo:Ruin")), case("demo:ruin"));
}

#[cfg(feature = "jigsaw")]
#[test]
fn a_processor_list_is_found_by_its_identifier() {
    let set = set(&JIGSAW, &[]);
    let cx = LinkCx::new(&set, ALL);
    let found = cx.processor_list(&id("demo:ruin"));
    assert_eq!(
        place(&found),
        Some((
            PackSide::Behavior,
            "worldgen/processors/ruin.json".to_owned()
        )),
        "{}",
        said(&set)
    );
    assert_eq!(cx.processor_list(&id("demo:fort")), MISSING);
    assert_eq!(cx.processor_list(&id("demo:Ruin")), case("demo:ruin"));
}

#[cfg(feature = "jigsaw")]
#[test]
fn a_jigsaw_structure_is_a_packs_a_structure_feature_of_the_game_or_missing() {
    let set = set(&JIGSAW, &[]);
    let cx = LinkCx::new(&set, ALL);
    let found = cx.jigsaw_structure(&id("demo:ruin"));
    assert_eq!(
        place(&found),
        Some((
            PackSide::Behavior,
            "worldgen/structures/ruin.json".to_owned()
        )),
        "{}",
        said(&set)
    );
    // Without a host the structure features of the crate's own list answer.
    assert_eq!(
        cx.jigsaw_structure(&id("minecraft:ancient_city")),
        Target::BuiltIn
    );
    assert_eq!(
        cx.jigsaw_structure(&id("minecraft:no_such_structure")),
        MISSING
    );
    assert_eq!(cx.jigsaw_structure(&id("demo:fort")), MISSING);
    assert_eq!(cx.jigsaw_structure(&id("demo:Ruin")), case("demo:ruin"));
}

#[cfg(feature = "jigsaw")]
#[test]
fn a_host_answers_for_the_jigsaw_structures_of_the_game() {
    let options = hosted(&[("jigsaw structure", "minecraft:moon_base")]);
    let set = set_with(&JIGSAW, &[], &options);
    let cx = LinkCx::new(&set, ALL);
    assert_eq!(
        cx.jigsaw_structure(&id("minecraft:moon_base")),
        Target::BuiltIn
    );
    assert_eq!(cx.jigsaw_structure(&id("minecraft:ancient_city")), MISSING);
    assert_eq!(
        file(&cx.jigsaw_structure(&id("demo:ruin"))).as_deref(),
        Some("worldgen/structures/ruin.json"),
        "{}",
        said(&set)
    );
}

#[cfg(feature = "jigsaw")]
#[test]
fn a_structure_set_is_found_by_its_identifier() {
    let set = set(&JIGSAW, &[]);
    let cx = LinkCx::new(&set, ALL);
    let found = cx.structure_set(&id("demo:ruin"));
    assert_eq!(
        place(&found),
        Some((
            PackSide::Behavior,
            "worldgen/structure_sets/ruin.json".to_owned()
        )),
        "{}",
        said(&set)
    );
    assert_eq!(cx.structure_set(&id("demo:fort")), MISSING);
    assert_eq!(cx.structure_set(&id("demo:Ruin")), case("demo:ruin"));
    // The game's structure features are no structure sets.
    assert_eq!(cx.structure_set(&id("minecraft:ancient_city")), MISSING);
}

/// Vanilla reads the dimension data of its own three dimensions only (`identifier: currently
/// only 'overworld', 'nether' and 'the_end' are supported`), so the one a pack defines is one
/// of the game's: the pack's document answers before the list does.
#[cfg(feature = "dimension")]
#[test]
fn a_dimension_is_a_packs_the_games_or_missing() {
    let overworld = r#"{"format_version": "1.21.60", "minecraft:dimension": {"description": {"identifier": "minecraft:overworld"}, "components": {"minecraft:dimension_bounds": {"min": -64, "max": 320}, "minecraft:generation": {"generator_type": "void"}}}}"#;
    let set = set(&[("dimensions/overworld.json", overworld)], &[]);
    let cx = LinkCx::new(&set, ALL);
    let found = cx.dimension(&id("minecraft:overworld"));
    assert_eq!(
        place(&found),
        Some((PackSide::Behavior, "dimensions/overworld.json".to_owned())),
        "{}",
        said(&set)
    );
    // Without a host the crate's own list answers for the dimensions no pack defines.
    assert_eq!(cx.dimension(&id("minecraft:nether")), Target::BuiltIn);
    assert_eq!(cx.dimension(&id("minecraft:the_end")), Target::BuiltIn);
    assert_eq!(cx.dimension(&id("minecraft:no_such_dimension")), MISSING);
    assert_eq!(cx.dimension(&id("demo:sky")), MISSING);
    assert_eq!(
        cx.dimension(&id("minecraft:Overworld")),
        case("minecraft:overworld")
    );
}

#[cfg(feature = "dimension")]
#[test]
fn without_a_pack_the_dimensions_of_the_game_are_built_in() {
    let set = set(&[], &[]);
    let cx = LinkCx::new(&set, ALL);
    assert_eq!(cx.dimension(&id("minecraft:overworld")), Target::BuiltIn);
    assert_eq!(cx.dimension(&id("minecraft:sky")), MISSING);
}

// ---- structures ------------------------------------------------------------------------------

/// A world whose behaviour pack has a one-block structure at each of `paths`.
#[cfg(feature = "structure")]
fn set_of_structures(paths: &[&'static str], options: &LoadOptions) -> StackSet {
    use bedrock_addon_core::pack_identity::PackOrigin;

    let manifest = r#"{ "format_version": 2,
  "header": { "name": "pack 1", "description": "", "uuid": "00000000-0000-4000-8000-000000000001",
              "version": [1, 0, 0], "min_engine_version": [1, 21, 0] },
  "modules": [{ "type": "data", "uuid": "00000000-0000-4000-9000-000000000001", "version": [1, 0, 0] }] }"#;
    let bytes = crate::families::structure::Structure::new([1, 1, 1])
        .to_bytes()
        .expect("a structure encodes");
    let bytes: &'static [u8] = Box::leak(bytes.into_boxed_slice());
    let files: Vec<(&'static str, Option<&'static [u8]>)> =
        paths.iter().map(|path| (*path, Some(bytes))).collect();
    let pack = crate::pack::tests::pack_from(manifest, &files, PackOrigin::World);
    super::support::set_of(vec![pack], Vec::new(), options)
}

#[cfg(feature = "structure")]
const STRUCTURES: [&str; 2] = [
    "structures/demo/tower.mcstructure",
    "structures/hut.mcstructure",
];

#[cfg(feature = "structure")]
#[test]
fn a_structure_is_found_by_the_name_structure_load_gives_its_file() {
    let set = set_of_structures(&STRUCTURES, &LoadOptions::default());
    let cx = LinkCx::new(&set, ALL);
    // The first folder is the namespace; a file directly in `structures/` is `mystructure:`'s.
    let found = cx.structure(&id("demo:tower"));
    assert_eq!(
        place(&found),
        Some((
            PackSide::Behavior,
            "structures/demo/tower.mcstructure".to_owned()
        )),
        "{}",
        said(&set)
    );
    assert_eq!(
        file(&cx.structure(&id("mystructure:hut"))).as_deref(),
        Some("structures/hut.mcstructure"),
        "{}",
        said(&set)
    );
    assert_eq!(cx.structure(&id("demo:hut")), MISSING);
    assert_eq!(cx.structure(&id("demo:keep")), MISSING);
    assert_eq!(cx.structure(&id("demo:Tower")), case("demo:tower"));
    assert_eq!(cx.structure(&id("tower")), NO_NAMESPACE);
}

#[cfg(feature = "structure")]
#[test]
fn a_structure_template_is_the_file_at_the_location_without_its_namespace() {
    let set = set_of_structures(&STRUCTURES, &LoadOptions::default());
    let cx = LinkCx::new(&set, ALL);
    // The namespace of the location is not part of the path, whatever it is.
    for location in ["minecraft:demo/tower", "other:demo/tower", "demo/tower"] {
        assert_eq!(
            place(&cx.structure_template(&id(location))),
            Some((
                PackSide::Behavior,
                "structures/demo/tower.mcstructure".to_owned()
            )),
            "{location}: {}",
            said(&set)
        );
    }
    assert_eq!(
        file(&cx.structure_template(&id("demo:hut"))).as_deref(),
        Some("structures/hut.mcstructure"),
        "{}",
        said(&set)
    );
    // `/structure load` would name the file `demo:tower`; a pool element names its path.
    assert_eq!(cx.structure_template(&id("demo:tower")), MISSING);
    assert_eq!(cx.structure_template(&id("demo:demo/keep")), MISSING);
}

#[cfg(feature = "structure")]
#[test]
fn a_host_answers_for_the_structure_templates_of_the_game() {
    let options = hosted(&[("structure", "minecraft:village/plains/houses/small")]);
    let set = set_of_structures(&STRUCTURES, &options);
    let cx = LinkCx::new(&set, ALL);
    assert_eq!(
        cx.structure_template(&id("minecraft:village/plains/houses/small")),
        Target::BuiltIn
    );
    assert_eq!(
        cx.structure_template(&id("minecraft:village/plains/houses/large")),
        MISSING
    );
    assert_eq!(
        file(&cx.structure_template(&id("x:demo/tower"))).as_deref(),
        Some("structures/demo/tower.mcstructure"),
        "{}",
        said(&set)
    );
}

// ---- trades, functions -----------------------------------------------------------------------

#[cfg(feature = "trade")]
#[test]
fn a_trade_table_is_found_by_its_path_in_the_pack() {
    let table = r#"{"tiers": [{"trades": [{"wants": [{"item": "minecraft:wheat"}], "gives": [{"item": "minecraft:emerald"}]}]}]}"#;
    let set = set(&[("trading/economy_trades/farmer_trades.json", table)], &[]);
    let cx = LinkCx::new(&set, ALL);
    let found = cx.trade_table(&id("trading/economy_trades/farmer_trades.json"));
    assert_eq!(
        place(&found),
        Some((
            PackSide::Behavior,
            "trading/economy_trades/farmer_trades.json".to_owned()
        )),
        "{}",
        said(&set)
    );
    assert_eq!(cx.trade_table(&id("trading/farmer_trades.json")), MISSING);
    assert_eq!(
        cx.trade_table(&id("trading/economy_trades/farmer_trades")),
        MISSING
    );
    assert_eq!(
        cx.trade_table(&id("trading/economy_trades/Farmer_Trades.json")),
        case("trading/economy_trades/farmer_trades.json")
    );
}

#[cfg(feature = "function")]
#[test]
fn a_function_is_found_by_its_path_below_functions_without_the_extension() {
    let set = set(
        &[
            ("functions/hello.mcfunction", "say hi\n"),
            ("functions/demo/setup.mcfunction", "say ready\n"),
        ],
        &[],
    );
    let cx = LinkCx::new(&set, ALL);
    let found = cx.function(&id("hello"));
    assert_eq!(
        place(&found),
        Some((PackSide::Behavior, "functions/hello.mcfunction".to_owned())),
        "{}",
        said(&set)
    );
    assert_eq!(
        file(&cx.function(&id("demo/setup"))).as_deref(),
        Some("functions/demo/setup.mcfunction"),
        "{}",
        said(&set)
    );
    assert_eq!(cx.function(&id("goodbye")), MISSING);
    assert_eq!(cx.function(&id("setup")), MISSING);
    assert_eq!(cx.function(&id("functions/hello")), MISSING);
    assert_eq!(cx.function(&id("hello.mcfunction")), MISSING);
}

// ---- cameras, aim assist ---------------------------------------------------------------------

#[cfg(feature = "camera")]
#[test]
fn a_camera_preset_is_a_packs_the_games_or_missing() {
    let preset = r#"{"format_version": "1.21.60", "minecraft:camera_preset": {"identifier": "demo:cam", "inherit_from": "minecraft:free"}}"#;
    let set = set(&[("cameras/presets/cam.json", preset)], &[]);
    let cx = LinkCx::new(&set, ALL);
    let found = cx.camera_preset(&id("demo:cam"));
    assert_eq!(
        place(&found),
        Some((PackSide::Behavior, "cameras/presets/cam.json".to_owned())),
        "{}",
        said(&set)
    );
    assert_eq!(cx.camera_preset(&id("minecraft:free")), Target::BuiltIn);
    assert_eq!(cx.camera_preset(&id("minecraft:no_such_preset")), MISSING);
    assert_eq!(cx.camera_preset(&id("demo:other")), MISSING);
    assert_eq!(cx.camera_preset(&id("demo:Cam")), case("demo:cam"));
    // A camera preset is no aim-assist preset.
    #[cfg(feature = "aim_assist")]
    assert_eq!(cx.aim_assist_preset(&id("demo:cam")), MISSING);
}

#[cfg(feature = "aim_assist")]
const AIM_ASSIST: [(&str, &str); 2] = [
    (
        "aim_assist/presets/aim.json",
        r#"{"format_version": "1.21.50", "minecraft:aim_assist_preset": {"identifier": "demo:aim"}}"#,
    ),
    (
        "aim_assist/categories/targets.json",
        r#"{"format_version": "1.21.50", "minecraft:aim_assist_categories": {"categories": [{"name": "demo:targets", "priorities": {"entity_default": 9}}]}}"#,
    ),
];

#[cfg(feature = "aim_assist")]
#[test]
fn an_aim_assist_preset_is_found_by_its_identifier() {
    let set = set(&AIM_ASSIST, &[]);
    let cx = LinkCx::new(&set, ALL);
    let found = cx.aim_assist_preset(&id("demo:aim"));
    assert_eq!(
        place(&found),
        Some((PackSide::Behavior, "aim_assist/presets/aim.json".to_owned())),
        "{}",
        said(&set)
    );
    assert_eq!(cx.aim_assist_preset(&id("demo:targets")), MISSING);
    assert_eq!(cx.aim_assist_preset(&id("demo:Aim")), case("demo:aim"));
    assert_eq!(cx.camera_preset(&id("demo:aim")), MISSING);
}

#[cfg(feature = "aim_assist")]
#[test]
fn an_aim_assist_category_is_found_by_its_name() {
    let set = set(&AIM_ASSIST, &[]);
    let cx = LinkCx::new(&set, ALL);
    let found = cx.aim_assist_category(&id("demo:targets"));
    assert_eq!(
        place(&found),
        Some((
            PackSide::Behavior,
            "aim_assist/categories/targets.json".to_owned()
        )),
        "{}",
        said(&set)
    );
    assert_eq!(cx.aim_assist_category(&id("demo:aim")), MISSING);
    assert_eq!(
        cx.aim_assist_category(&id("demo:Targets")),
        case("demo:targets")
    );
}

// ---- points of interest, behaviour trees ---------------------------------------------------

/// A type that carries the tag `demo:smithy`, and the document of the tag `demo:craft`.
#[cfg(feature = "poi")]
const POI: [(&str, &str); 2] = [
    (
        "poi/blocks/forge.json",
        r#"{"format_version": "1.26.40", "minecraft:poi_block": {"description": {"identifier": "demo:forge"}, "blocks": ["minecraft:anvil"], "tags": ["demo:smithy"]}}"#,
    ),
    (
        "poi/tags/craft.json",
        r#"{"format_version": "1.26.40", "minecraft:poi_tag": {"description": {"identifier": "demo:craft"}, "children": ["demo:smithy"]}}"#,
    ),
];

#[cfg(feature = "poi")]
#[test]
fn a_poi_type_is_found_by_its_identifier() {
    let set = set_with(&POI, &[], &with_experiment(Experiment::Poi));
    let cx = LinkCx::new(&set, ALL);
    let found = cx.poi_type(&id("demo:forge"));
    assert_eq!(
        place(&found),
        Some((PackSide::Behavior, "poi/blocks/forge.json".to_owned())),
        "{}",
        said(&set)
    );
    assert_eq!(cx.poi_type(&id("demo:craft")), MISSING);
    assert_eq!(cx.poi_type(&id("demo:Forge")), case("demo:forge"));
}

#[cfg(feature = "poi")]
#[test]
fn a_poi_tag_is_found_when_it_has_a_tag_document() {
    let set = set_with(&POI, &[], &with_experiment(Experiment::Poi));
    let cx = LinkCx::new(&set, ALL);
    let found = cx.poi_tag(&id("demo:craft"));
    assert_eq!(
        place(&found),
        Some((PackSide::Behavior, "poi/tags/craft.json".to_owned())),
        "{}",
        said(&set)
    );
    // A tag only a type carries has no document; a type is no tag.
    assert_eq!(cx.poi_tag(&id("demo:smithy")), MISSING);
    assert_eq!(cx.poi_tag(&id("demo:forge")), MISSING);
    assert_eq!(cx.poi_tag(&id("demo:Craft")), case("demo:craft"));
}

#[cfg(feature = "poi")]
#[test]
fn without_the_poi_experiment_no_poi_document_is_defined() {
    let set = set(&POI, &[]);
    let cx = LinkCx::new(&set, ALL);
    assert_eq!(cx.poi_type(&id("demo:forge")), MISSING);
    assert_eq!(cx.poi_tag(&id("demo:craft")), MISSING);
}

#[cfg(feature = "behavior_tree")]
#[test]
fn a_behavior_tree_is_found_by_its_name() {
    let tree = r#"{"minecraft:behavior_tree": {"minecraft:name": "patrol", "minecraft:root": {"minecraft:wait": {}}}}"#;
    let set = set(&[("behavior_trees/patrol.json", tree)], &[]);
    let cx = LinkCx::new(&set, ALL);
    let found = cx.behavior_tree(&id("patrol"));
    assert_eq!(
        place(&found),
        Some((PackSide::Behavior, "behavior_trees/patrol.json".to_owned())),
        "{}",
        said(&set)
    );
    assert_eq!(cx.behavior_tree(&id("guard")), MISSING);
    assert_eq!(cx.behavior_tree(&id("Patrol")), case("patrol"));
}

// ---- geometries ------------------------------------------------------------------------------

/// `geometry.Lamp`: the bone `Body` of two cubes with the locator `Lead`.
#[cfg(feature = "geometry")]
const LAMP_GEOMETRY: &str = r#"{"format_version": "1.16.0", "minecraft:geometry": [{"description": {"identifier": "geometry.Lamp"}, "bones": [{"name": "Body", "cubes": [{"origin": [0, 0, 0], "size": [1, 1, 1], "uv": [0, 0]}, {"origin": [0, 1, 0], "size": [1, 1, 1], "uv": [0, 0]}], "locators": {"Lead": [0, 1, 0]}}]}]}"#;

#[cfg(feature = "geometry")]
#[test]
fn a_geometry_is_found_without_ascii_case() {
    let set = set(&[], &[("models/entity/lamp.geo.json", LAMP_GEOMETRY)]);
    let cx = LinkCx::new(&set, ALL);
    for name in ["geometry.Lamp", "geometry.lamp", "geometry.LAMP"] {
        assert_eq!(
            place(&cx.geometry(&id(name))),
            Some((PackSide::Resource, "models/entity/lamp.geo.json".to_owned())),
            "{name}: {}",
            said(&set)
        );
    }
    assert_eq!(cx.geometry(&id("geometry.lamps")), MISSING);
    assert_eq!(cx.geometry(&id("geometry.lam")), MISSING);
}

#[cfg(feature = "geometry")]
#[test]
fn a_bone_of_a_geometry_is_found_by_its_name_without_case() {
    let set = set(&[], &[("models/entity/lamp.geo.json", LAMP_GEOMETRY)]);
    let cx = LinkCx::new(&set, ALL);
    for name in ["Body", "body", "BODY"] {
        let bone = cx
            .geometry_bone(&id("geometry.lamp"), name)
            .unwrap_or_else(|| panic!("{name}: {}", said(&set)));
        // The geometry keeps a bone's name lower-cased.
        assert_eq!(bone.name, "body", "{name}");
        assert_eq!(bone.cubes.as_ref().map(Vec::len), Some(2), "{name}");
    }
    assert!(cx.geometry_bone(&id("geometry.LAMP"), "body").is_some());
    assert!(cx.geometry_bone(&id("geometry.lamp"), "tail").is_none());
    assert!(cx.geometry_bone(&id("geometry.lamp"), "bod").is_none());
    // A locator is no bone, and a geometry the stack does not have has none.
    assert!(cx.geometry_bone(&id("geometry.lamp"), "lead").is_none());
    assert!(cx.geometry_bone(&id("geometry.lamps"), "body").is_none());
}

#[cfg(feature = "geometry")]
#[test]
fn a_locator_of_a_geometry_is_found_by_its_name_without_case() {
    let set = set(&[], &[("models/entity/lamp.geo.json", LAMP_GEOMETRY)]);
    let cx = LinkCx::new(&set, ALL);
    for name in ["Lead", "lead", "LEAD"] {
        assert!(
            cx.geometry_has_locator(&id("geometry.lamp"), name),
            "{name}: {}",
            said(&set)
        );
    }
    assert!(cx.geometry_has_locator(&id("geometry.LAMP"), "lead"));
    assert!(!cx.geometry_has_locator(&id("geometry.lamp"), "hook"));
    // A bone is no locator, and a geometry the stack does not have has none.
    assert!(!cx.geometry_has_locator(&id("geometry.lamp"), "body"));
    assert!(!cx.geometry_has_locator(&id("geometry.lamps"), "lead"));
}

#[cfg(feature = "geometry")]
#[test]
fn a_geometry_older_than_1_12_has_the_bones_and_locators_of_its_parents() {
    let mobs = r#"{"format_version": "1.8.0", "geometry.base": {"bones": [{"name": "Body", "cubes": [{"origin": [0, 0, 0], "size": [1, 1, 1], "uv": [0, 0]}], "locators": {"Lead": [0, 14, -6]}}]}, "geometry.npc:geometry.base": {"bones": [{"name": "Hat"}]}, "geometry.guard:geometry.npc": {"bones": [{"name": "Spear", "locators": {"Tip": [0, 20, 0]}}]}}"#;
    let set = set(&[], &[("models/mobs.json", mobs)]);
    let cx = LinkCx::new(&set, ALL);
    for name in ["geometry.base", "geometry.npc", "geometry.guard"] {
        assert_eq!(
            file(&cx.geometry(&id(name))).as_deref(),
            Some("models/mobs.json"),
            "{name}: {}",
            said(&set)
        );
    }
    // The key with the parent is no name.
    assert_eq!(cx.geometry(&id("geometry.npc:geometry.base")), MISSING);
    let bone = |geometry: &str, bone: &str| {
        cx.geometry_bone(&id(geometry), bone)
            .map(|bone| (bone.name.as_str(), bone.cubes.as_ref().map(Vec::len)))
    };
    // An own bone, one of the parent, one of the parent's parent.
    assert_eq!(bone("geometry.guard", "spear"), Some(("spear", None)));
    assert_eq!(bone("geometry.guard", "HAT"), Some(("hat", None)));
    assert_eq!(bone("geometry.guard", "body"), Some(("body", Some(1))));
    assert_eq!(bone("geometry.npc", "body"), Some(("body", Some(1))));
    // A parent has nothing of its children.
    assert_eq!(bone("geometry.npc", "spear"), None);
    assert_eq!(bone("geometry.base", "hat"), None);
    assert_eq!(bone("geometry.guard", "tail"), None);
    assert!(cx.geometry_has_locator(&id("geometry.guard"), "tip"));
    assert!(cx.geometry_has_locator(&id("geometry.guard"), "LEAD"));
    assert!(cx.geometry_has_locator(&id("geometry.npc"), "lead"));
    assert!(cx.geometry_has_locator(&id("geometry.base"), "lead"));
    assert!(!cx.geometry_has_locator(&id("geometry.npc"), "tip"));
    assert!(!cx.geometry_has_locator(&id("geometry.guard"), "hook"));
}

// ---- render controllers, fog, deferred settings ----------------------------------------------

#[cfg(feature = "render_controller")]
#[test]
fn a_render_controller_is_found_without_ascii_case() {
    let controllers = r#"{"format_version": "1.8.0", "render_controllers": {"controller.render.lamp": {"geometry": "geometry.default", "textures": ["texture.default"]}}}"#;
    let set = set(&[], &[("render_controllers/lamp.json", controllers)]);
    let cx = LinkCx::new(&set, ALL);
    for name in [
        "controller.render.lamp",
        "controller.render.Lamp",
        "CONTROLLER.RENDER.LAMP",
    ] {
        assert_eq!(
            place(&cx.render_controller(&id(name))),
            Some((
                PackSide::Resource,
                "render_controllers/lamp.json".to_owned()
            )),
            "{name}: {}",
            said(&set)
        );
    }
    assert_eq!(
        cx.render_controller(&id("controller.render.lamps")),
        MISSING
    );
    assert_eq!(cx.render_controller(&id("controller.render.lam")), MISSING);
}

// The game registers `controller.render.Foo` under a name its lower-cased references never
// reach (the apostrophe before it turns the folding of the file's keys off). The crate's own
// registry folds the key and reports it, and the lookup resolves it.
#[cfg(feature = "render_controller")]
#[test]
fn a_render_controller_the_game_registered_unfolded_resolves_by_its_folded_name() {
    let controllers = "{\"format_version\": \"1.8.0\", \"render_controllers\": {\n// it's\n\"controller.render.Foo\": {\"geometry\": \"geometry.default\", \"textures\": [\"texture.default\"]}}}";
    let set = set(&[], &[("render_controllers/foo.json", controllers)]);
    let cx = LinkCx::new(&set, ALL);
    let warned = set
        .resource
        .render_controllers()
        .diagnostics()
        .filter(|finding| finding.code.id() == "core/case_folding_disabled_at_line_unresolvable")
        .count();
    assert_eq!(warned, 1, "{}", said(&set));
    for name in ["controller.render.foo", "controller.render.Foo"] {
        let found = cx.render_controller(&id(name));
        assert_eq!(
            file(&found).as_deref(),
            Some("render_controllers/foo.json"),
            "{name}: {}",
            said(&set)
        );
        // The place keeps the key as the file writes it.
        assert_eq!(
            found.origin().and_then(|origin| origin.key.as_deref()),
            Some("controller.render.Foo"),
            "{name}"
        );
    }
    assert_eq!(cx.render_controller(&id("controller.render.fo")), MISSING);
}

#[cfg(feature = "fog")]
#[test]
fn a_fog_is_found_by_its_identifier() {
    let fog = r#"{"format_version": "1.26.20", "minecraft:fog_settings": {"description": {"identifier": "demo:mist"}, "distance": {"air": {"fog_end": 1}}}}"#;
    let set = set(&[], &[("fogs/mist.json", fog)]);
    let cx = LinkCx::new(&set, ALL);
    let found = cx.fog(&id("demo:mist"));
    assert_eq!(
        place(&found),
        Some((PackSide::Resource, "fogs/mist.json".to_owned())),
        "{}",
        said(&set)
    );
    assert_eq!(cx.fog(&id("demo:haze")), MISSING);
    assert_eq!(cx.fog(&id("demo:Mist")), case("demo:mist"));
}

/// One document of each of the five deferred settings kinds, each named `demo:dusk`.
#[cfg(feature = "deferred_settings")]
const DEFERRED: [(&str, &str); 5] = [
    (
        "lighting/dusk.json",
        r#"{"format_version": "1.26.0", "minecraft:lighting_settings": {"description": {"identifier": "demo:dusk"}, "emissive": {"desaturation": 0.5}}}"#,
    ),
    (
        "water/dusk.json",
        r#"{"format_version": "1.26.20", "minecraft:water_settings": {"description": {"identifier": "demo:dusk"}}}"#,
    ),
    (
        "atmospherics/dusk.json",
        r#"{"format_version": "1.26.20", "minecraft:atmosphere_settings": {"description": {"identifier": "demo:dusk"}}}"#,
    ),
    (
        "color_grading/dusk.json",
        r#"{"format_version": "1.21.90", "minecraft:color_grading_settings": {"description": {"identifier": "demo:dusk"}}}"#,
    ),
    (
        "cubemaps/dusk.json",
        r#"{"format_version": "1.21.130", "minecraft:cubemap_settings": {"description": {"identifier": "demo:dusk"}}}"#,
    ),
];

/// The world with only the document of [`DEFERRED`] at `path`.
#[cfg(feature = "deferred_settings")]
fn set_of_deferred(path: &str) -> StackSet {
    let only: Vec<(&'static str, &str)> = DEFERRED
        .iter()
        .copied()
        .filter(|(file, _)| *file == path)
        .collect();
    assert_eq!(only.len(), 1, "{path}");
    set(&[], &only)
}

#[cfg(feature = "deferred_settings")]
#[test]
fn lighting_settings_are_found_by_their_identifier() {
    let set = set_of_deferred("lighting/dusk.json");
    let cx = LinkCx::new(&set, ALL);
    assert_eq!(
        place(&cx.lighting_settings(&id("demo:dusk"))),
        Some((PackSide::Resource, "lighting/dusk.json".to_owned())),
        "{}",
        said(&set)
    );
    assert_eq!(cx.lighting_settings(&id("demo:dawn")), MISSING);
    assert_eq!(cx.lighting_settings(&id("demo:Dusk")), case("demo:dusk"));
    assert_eq!(cx.water_settings(&id("demo:dusk")), MISSING);
    assert_eq!(cx.atmosphere_settings(&id("demo:dusk")), MISSING);
    assert_eq!(cx.color_grading_settings(&id("demo:dusk")), MISSING);
    assert_eq!(cx.cubemap_settings(&id("demo:dusk")), MISSING);
}

#[cfg(feature = "deferred_settings")]
#[test]
fn water_settings_are_found_by_their_identifier() {
    let set = set_of_deferred("water/dusk.json");
    let cx = LinkCx::new(&set, ALL);
    assert_eq!(
        place(&cx.water_settings(&id("demo:dusk"))),
        Some((PackSide::Resource, "water/dusk.json".to_owned())),
        "{}",
        said(&set)
    );
    assert_eq!(cx.water_settings(&id("demo:dawn")), MISSING);
    assert_eq!(cx.water_settings(&id("demo:Dusk")), case("demo:dusk"));
    assert_eq!(cx.lighting_settings(&id("demo:dusk")), MISSING);
    assert_eq!(cx.atmosphere_settings(&id("demo:dusk")), MISSING);
    assert_eq!(cx.color_grading_settings(&id("demo:dusk")), MISSING);
    assert_eq!(cx.cubemap_settings(&id("demo:dusk")), MISSING);
}

#[cfg(feature = "deferred_settings")]
#[test]
fn atmosphere_settings_are_found_by_their_identifier() {
    let set = set_of_deferred("atmospherics/dusk.json");
    let cx = LinkCx::new(&set, ALL);
    assert_eq!(
        place(&cx.atmosphere_settings(&id("demo:dusk"))),
        Some((PackSide::Resource, "atmospherics/dusk.json".to_owned())),
        "{}",
        said(&set)
    );
    assert_eq!(cx.atmosphere_settings(&id("demo:dawn")), MISSING);
    assert_eq!(cx.atmosphere_settings(&id("demo:Dusk")), case("demo:dusk"));
    assert_eq!(cx.lighting_settings(&id("demo:dusk")), MISSING);
    assert_eq!(cx.water_settings(&id("demo:dusk")), MISSING);
    assert_eq!(cx.color_grading_settings(&id("demo:dusk")), MISSING);
    assert_eq!(cx.cubemap_settings(&id("demo:dusk")), MISSING);
}

#[cfg(feature = "deferred_settings")]
#[test]
fn color_grading_settings_are_found_by_their_identifier() {
    let set = set_of_deferred("color_grading/dusk.json");
    let cx = LinkCx::new(&set, ALL);
    assert_eq!(
        place(&cx.color_grading_settings(&id("demo:dusk"))),
        Some((PackSide::Resource, "color_grading/dusk.json".to_owned())),
        "{}",
        said(&set)
    );
    assert_eq!(cx.color_grading_settings(&id("demo:dawn")), MISSING);
    assert_eq!(
        cx.color_grading_settings(&id("demo:Dusk")),
        case("demo:dusk")
    );
    assert_eq!(cx.lighting_settings(&id("demo:dusk")), MISSING);
    assert_eq!(cx.water_settings(&id("demo:dusk")), MISSING);
    assert_eq!(cx.atmosphere_settings(&id("demo:dusk")), MISSING);
    assert_eq!(cx.cubemap_settings(&id("demo:dusk")), MISSING);
}

#[cfg(feature = "deferred_settings")]
#[test]
fn cubemap_settings_are_found_by_their_identifier() {
    let set = set_of_deferred("cubemaps/dusk.json");
    let cx = LinkCx::new(&set, ALL);
    assert_eq!(
        place(&cx.cubemap_settings(&id("demo:dusk"))),
        Some((PackSide::Resource, "cubemaps/dusk.json".to_owned())),
        "{}",
        said(&set)
    );
    assert_eq!(cx.cubemap_settings(&id("demo:dawn")), MISSING);
    assert_eq!(cx.cubemap_settings(&id("demo:Dusk")), case("demo:dusk"));
    assert_eq!(cx.lighting_settings(&id("demo:dusk")), MISSING);
    assert_eq!(cx.water_settings(&id("demo:dusk")), MISSING);
    assert_eq!(cx.atmosphere_settings(&id("demo:dusk")), MISSING);
    assert_eq!(cx.color_grading_settings(&id("demo:dusk")), MISSING);
}

// ---- materials -------------------------------------------------------------------------------

/// `materials/common.json` listing `materials/my.material`, which defines `mine` with the
/// variant `glow`.
#[cfg(feature = "material")]
const MATERIALS: [(&str, &str); 2] = [
    (
        "materials/common.json",
        r#"[{"path": "materials/my.material"}]"#,
    ),
    (
        "materials/my.material",
        r#"{"materials": {"version": "1.0.0", "mine": {"variants": [{"glow": {}}]}}}"#,
    ),
];

#[cfg(feature = "material")]
#[test]
fn a_material_is_found_by_its_name_or_the_name_of_a_variant() {
    let set = set(&[], &MATERIALS);
    let cx = LinkCx::new(&set, ALL);
    for name in ["mine", "mine.glow"] {
        assert_eq!(
            place(&cx.material(&id(name))),
            Some((PackSide::Resource, "materials/my.material".to_owned())),
            "{name}: {}",
            said(&set)
        );
    }
    // As written; and without a host no material of the game is known.
    assert_eq!(cx.material(&id("Mine")), MISSING);
    assert_eq!(cx.material(&id("mine.dim")), MISSING);
    assert_eq!(cx.material(&id("yours")), MISSING);
    assert_eq!(cx.material(&id("entity")), MISSING);
}

#[cfg(feature = "material")]
#[test]
fn a_host_answers_for_the_materials_of_the_game() {
    let set = set_with(&[], &MATERIALS, &hosted(&[("material", "entity")]));
    let cx = LinkCx::new(&set, ALL);
    assert_eq!(cx.material(&id("entity")), Target::BuiltIn);
    assert_eq!(cx.material(&id("Entity")), MISSING);
    assert_eq!(cx.material(&id("entity_alphatest")), MISSING);
    assert_eq!(
        file(&cx.material(&id("mine"))).as_deref(),
        Some("materials/my.material"),
        "{}",
        said(&set)
    );
}

/// The yes-or-no of [`LinkCx::material`] without the place, for a rule that resolves
/// many references: the same names exist, the same are missing, with a host and
/// without one.
#[cfg(feature = "material")]
#[test]
fn a_material_exists_exactly_where_the_material_lookup_resolves() {
    const NAMES: [&str; 8] = [
        "mine",
        "mine.glow",
        "Mine",
        "mine.dim",
        "yours",
        "entity",
        "Entity",
        "entity_alphatest",
    ];
    let plain = set(&[], &MATERIALS);
    let hosted = set_with(&[], &MATERIALS, &hosted(&[("material", "entity")]));
    for (set, existing) in [
        (&plain, &["mine", "mine.glow"][..]),
        (&hosted, &["mine", "mine.glow", "entity"][..]),
    ] {
        let cx = LinkCx::new(set, ALL);
        for name in NAMES {
            assert_eq!(
                cx.material_exists(&id(name)),
                existing.contains(&name),
                "{name}"
            );
            assert_eq!(
                cx.material_exists(&id(name)),
                !cx.material(&id(name)).is_missing(),
                "{name}: as the lookup with the place answers"
            );
        }
    }
}

#[cfg(feature = "material")]
#[test]
fn the_materials_of_a_group_are_resolved_once_for_the_pass() {
    use crate::families::material::MaterialGroupKind;

    let set = set(&[], &MATERIALS);
    let cx = LinkCx::new(&set, ALL);
    let common = cx.resolved_materials(MaterialGroupKind::Common);
    assert!(common.contains_key("mine"), "{}", said(&set));
    assert!(!common.contains_key("yours"));
    assert!(std::ptr::eq(
        common,
        cx.resolved_materials(MaterialGroupKind::Common)
    ));
    // Each group has its own.
    let fancy = cx.resolved_materials(MaterialGroupKind::Fancy);
    assert!(!std::ptr::eq(common, fancy));
    assert!(std::ptr::eq(
        fancy,
        cx.resolved_materials(MaterialGroupKind::Fancy)
    ));
    assert!(!fancy.contains_key("mine"));
}

// ---- particles, animations -------------------------------------------------------------------

#[cfg(feature = "particle")]
#[test]
fn a_particle_is_found_by_its_identifier() {
    let spark = r#"{"format_version": "1.10.0", "particle_effect": {"description": {"identifier": "demo:spark", "basic_render_parameters": {"material": "particles_alpha", "texture": "textures/particle/particles"}}, "components": {"minecraft:emitter_lifetime_once": {}, "minecraft:emitter_rate_instant": {"num_particles": 1}}}}"#;
    let set = set(&[], &[("particles/spark.json", spark)]);
    let cx = LinkCx::new(&set, ALL);
    let found = cx.particle(&id("demo:spark"));
    assert_eq!(
        place(&found),
        Some((PackSide::Resource, "particles/spark.json".to_owned())),
        "{}",
        said(&set)
    );
    assert_eq!(cx.particle(&id("demo:smoke")), MISSING);
    assert_eq!(cx.particle(&id("demo:Spark")), case("demo:spark"));
}

/// An animation and a controller on each side, each under a name of its own.
#[cfg(all(feature = "animation", feature = "animation_controller"))]
const BEHAVIOR_ANIMATIONS: [(&str, &str); 2] = [
    (
        "animations/walk.json",
        r#"{"format_version": "1.10.0", "animations": {"animation.demo.walk": {"loop": true, "timeline": {"0.0": "/say hi"}}}}"#,
    ),
    (
        "animation_controllers/move.json",
        r#"{"format_version": "1.10.0", "animation_controllers": {"controller.animation.demo.move": {"initial_state": "default", "states": {"default": {}}}}}"#,
    ),
];

#[cfg(all(feature = "animation", feature = "animation_controller"))]
const CLIENT_ANIMATIONS: [(&str, &str); 2] = [
    (
        "animations/fly.animation.json",
        r#"{"format_version": "1.8.0", "animations": {"animation.demo.fly": {"loop": true}}}"#,
    ),
    (
        "animation_controllers/flap.animation_controllers.json",
        r#"{"format_version": "1.10.0", "animation_controllers": {"controller.animation.demo.flap": {"initial_state": "default", "states": {"default": {}}}}}"#,
    ),
];

#[cfg(all(feature = "animation", feature = "animation_controller"))]
#[test]
fn a_behavior_animation_is_found_by_its_name() {
    let set = set(&BEHAVIOR_ANIMATIONS, &CLIENT_ANIMATIONS);
    let cx = LinkCx::new(&set, ALL);
    assert_eq!(
        place(&cx.behavior_animation(&id("animation.demo.walk"))),
        Some((PackSide::Behavior, "animations/walk.json".to_owned())),
        "{}",
        said(&set)
    );
    assert_eq!(
        cx.behavior_animation(&id("animation.demo.Walk")),
        case("animation.demo.walk")
    );
    // Neither the resource stack's animation nor a controller of its own side.
    assert_eq!(cx.behavior_animation(&id("animation.demo.fly")), MISSING);
    assert_eq!(
        cx.behavior_animation(&id("controller.animation.demo.move")),
        MISSING
    );
}

#[cfg(all(feature = "animation", feature = "animation_controller"))]
#[test]
fn a_behavior_animation_controller_is_found_by_its_name() {
    let set = set(&BEHAVIOR_ANIMATIONS, &CLIENT_ANIMATIONS);
    let cx = LinkCx::new(&set, ALL);
    assert_eq!(
        place(&cx.behavior_animation_controller(&id("controller.animation.demo.move"))),
        Some((
            PackSide::Behavior,
            "animation_controllers/move.json".to_owned()
        )),
        "{}",
        said(&set)
    );
    assert_eq!(
        cx.behavior_animation_controller(&id("controller.animation.demo.Move")),
        case("controller.animation.demo.move")
    );
    assert_eq!(
        cx.behavior_animation_controller(&id("controller.animation.demo.flap")),
        MISSING
    );
    assert_eq!(
        cx.behavior_animation_controller(&id("animation.demo.walk")),
        MISSING
    );
}

#[cfg(all(feature = "animation", feature = "animation_controller"))]
#[test]
fn a_client_animation_is_found_by_its_name() {
    let set = set(&BEHAVIOR_ANIMATIONS, &CLIENT_ANIMATIONS);
    let cx = LinkCx::new(&set, ALL);
    assert_eq!(
        place(&cx.client_animation(&id("animation.demo.fly"))),
        Some((
            PackSide::Resource,
            "animations/fly.animation.json".to_owned()
        )),
        "{}",
        said(&set)
    );
    assert_eq!(
        cx.client_animation(&id("animation.demo.Fly")),
        case("animation.demo.fly")
    );
    assert_eq!(cx.client_animation(&id("animation.demo.walk")), MISSING);
    assert_eq!(
        cx.client_animation(&id("controller.animation.demo.flap")),
        MISSING
    );
}

#[cfg(all(feature = "animation", feature = "animation_controller"))]
#[test]
fn a_client_animation_controller_is_found_by_its_name() {
    let set = set(&BEHAVIOR_ANIMATIONS, &CLIENT_ANIMATIONS);
    let cx = LinkCx::new(&set, ALL);
    assert_eq!(
        place(&cx.client_animation_controller(&id("controller.animation.demo.flap"))),
        Some((
            PackSide::Resource,
            "animation_controllers/flap.animation_controllers.json".to_owned()
        )),
        "{}",
        said(&set)
    );
    assert_eq!(
        cx.client_animation_controller(&id("controller.animation.demo.Flap")),
        case("controller.animation.demo.flap")
    );
    assert_eq!(
        cx.client_animation_controller(&id("controller.animation.demo.move")),
        MISSING
    );
    assert_eq!(
        cx.client_animation_controller(&id("animation.demo.fly")),
        MISSING
    );
}

/// The prefix of the name picks the collection of the side; a name with neither prefix
/// is nothing the game looks up.
#[cfg(all(feature = "animation", feature = "animation_controller"))]
#[test]
fn the_prefix_of_an_animation_name_picks_animations_or_controllers_of_the_side() {
    let set = set(&BEHAVIOR_ANIMATIONS, &CLIENT_ANIMATIONS);
    let cx = LinkCx::new(&set, ALL);
    let found = |side, name: &str| cx.animation(side, &id(name));
    let placed = |side, name: &str| found(side, name).as_ref().and_then(place);
    // `animation.`: the animations of the side.
    assert_eq!(
        placed(PackSide::Behavior, "animation.demo.walk"),
        Some((PackSide::Behavior, "animations/walk.json".to_owned())),
        "{}",
        said(&set)
    );
    assert_eq!(
        placed(PackSide::Resource, "animation.demo.fly"),
        Some((
            PackSide::Resource,
            "animations/fly.animation.json".to_owned()
        )),
        "{}",
        said(&set)
    );
    assert_eq!(
        found(PackSide::Resource, "animation.demo.walk"),
        Some(MISSING)
    );
    assert_eq!(
        found(PackSide::Behavior, "animation.demo.fly"),
        Some(MISSING)
    );
    assert_eq!(
        found(PackSide::Behavior, "animation.demo.Walk"),
        Some(case("animation.demo.walk"))
    );
    // `controller.animation.`: the controllers of the side.
    assert_eq!(
        placed(PackSide::Behavior, "controller.animation.demo.move"),
        Some((
            PackSide::Behavior,
            "animation_controllers/move.json".to_owned()
        )),
        "{}",
        said(&set)
    );
    assert_eq!(
        placed(PackSide::Resource, "controller.animation.demo.flap"),
        Some((
            PackSide::Resource,
            "animation_controllers/flap.animation_controllers.json".to_owned()
        )),
        "{}",
        said(&set)
    );
    assert_eq!(
        found(PackSide::Resource, "controller.animation.demo.move"),
        Some(MISSING)
    );
    assert_eq!(
        found(PackSide::Behavior, "controller.animation.demo.flap"),
        Some(MISSING)
    );
    // Any other prefix, on either side: the prefix is compared as written.
    for side in [PackSide::Behavior, PackSide::Resource] {
        for name in [
            "walk",
            "demo.walk",
            "controller.demo.move",
            "controller.render.demo",
            "Animation.demo.walk",
            "animation",
            "",
        ] {
            assert_eq!(found(side, name), None, "{side:?} {name}");
        }
    }
}

// ---- textures --------------------------------------------------------------------------------

#[cfg(feature = "texture_atlas")]
const ATLASES: [(&str, &str); 2] = [
    (
        "textures/terrain_texture.json",
        r#"{"texture_name": "atlas.terrain", "texture_data": {"demo_lamp": {"textures": "textures/blocks/lamp"}}}"#,
    ),
    (
        "textures/item_texture.json",
        r#"{"texture_name": "atlas.items", "texture_data": {"demo_gem": {"textures": "textures/items/demo_gem"}}}"#,
    ),
];

#[cfg(feature = "texture_atlas")]
#[test]
fn a_terrain_atlas_key_is_found_as_written() {
    let set = set(&[], &ATLASES);
    let cx = LinkCx::new(&set, ALL);
    assert_eq!(
        place(&cx.terrain_atlas_key(&id("demo_lamp"))),
        Some((
            PackSide::Resource,
            "textures/terrain_texture.json".to_owned()
        )),
        "{}",
        said(&set)
    );
    assert_eq!(cx.terrain_atlas_key(&id("Demo_Lamp")), case("demo_lamp"));
    assert_eq!(cx.terrain_atlas_key(&id("demo_lamps")), MISSING);
    // The item atlas's key is not the terrain atlas's.
    assert_eq!(cx.terrain_atlas_key(&id("demo_gem")), MISSING);
}

#[cfg(feature = "texture_atlas")]
#[test]
fn an_item_atlas_key_is_found_as_written() {
    let set = set(&[], &ATLASES);
    let cx = LinkCx::new(&set, ALL);
    assert_eq!(
        place(&cx.item_atlas_key(&id("demo_gem"))),
        Some((PackSide::Resource, "textures/item_texture.json".to_owned())),
        "{}",
        said(&set)
    );
    assert_eq!(cx.item_atlas_key(&id("DEMO_GEM")), case("demo_gem"));
    assert_eq!(cx.item_atlas_key(&id("demo_gems")), MISSING);
    // The terrain atlas's key is not the item atlas's.
    assert_eq!(cx.item_atlas_key(&id("demo_lamp")), MISSING);
}

#[cfg(all(feature = "texture_file", feature = "texture_set"))]
#[test]
fn a_texture_file_is_the_image_at_the_path_without_its_extension() {
    let set = set(&[], &[("textures/blocks/lamp.png", "not read by a load")]);
    let cx = LinkCx::new(&set, ALL);
    let found = cx
        .texture_file(&id("textures/blocks/lamp"))
        .unwrap_or_else(|| panic!("{}", said(&set)));
    assert!(!found.is_missing());
    assert_eq!(found.stack_index, 0);
    assert_eq!(found.pack.name(), "pack 2");
    assert!(found.set.is_none(), "a plain image has no texture set");
    assert!(cx.texture_file(&id("textures/blocks/lantern")).is_none());
    assert!(cx.texture_file(&id("textures/blocks")).is_none());
    assert!(cx.texture_file(&id("blocks/lamp")).is_none());
}

// ---- sounds ----------------------------------------------------------------------------------

#[cfg(any(
    feature = "sound_definitions",
    feature = "music",
    feature = "sounds_json"
))]
const SOUNDS: [(&str, &str); 3] = [
    (
        "sounds/sound_definitions.json",
        r#"{"format_version": "1.20.20", "sound_definitions": {"demo.ping": {"sounds": ["sounds/demo/ping"]}}}"#,
    ),
    (
        "sounds/music_definitions.json",
        r#"{"demo_game": {"event_name": "demo.ping", "min_delay": 60, "max_delay": 180}}"#,
    ),
    (
        "sounds.json",
        r#"{"block_sounds": {"demo_brass": {"events": {"hit": "demo.ping"}}}, "individual_named_sounds": {"sounds": {"Demo.Chime": "demo.ping"}}}"#,
    ),
];

#[cfg(feature = "sound_definitions")]
#[test]
fn a_sound_definition_is_found_as_written() {
    let set = set(&[], &SOUNDS);
    let cx = LinkCx::new(&set, ALL);
    assert_eq!(
        place(&cx.sound_definition(&id("demo.ping"))),
        Some((
            PackSide::Resource,
            "sounds/sound_definitions.json".to_owned()
        )),
        "{}",
        said(&set)
    );
    assert_eq!(cx.sound_definition(&id("Demo.Ping")), case("demo.ping"));
    assert_eq!(cx.sound_definition(&id("demo.pong")), MISSING);
    // Neither a music definition nor a name of `sounds.json`.
    assert_eq!(cx.sound_definition(&id("demo_game")), MISSING);
    assert_eq!(cx.sound_definition(&id("Demo.Chime")), MISSING);
}

#[cfg(feature = "music")]
#[test]
fn a_music_definition_is_found_by_its_name() {
    let set = set(&[], &SOUNDS);
    let cx = LinkCx::new(&set, ALL);
    assert_eq!(
        place(&cx.music_definition(&id("demo_game"))),
        Some((
            PackSide::Resource,
            "sounds/music_definitions.json".to_owned()
        )),
        "{}",
        said(&set)
    );
    assert_eq!(cx.music_definition(&id("Demo_Game")), case("demo_game"));
    assert_eq!(cx.music_definition(&id("demo_menu")), MISSING);
    assert_eq!(cx.music_definition(&id("demo.ping")), MISSING);
}

#[cfg(feature = "sounds_json")]
#[test]
fn a_block_sound_is_a_key_of_block_sounds_as_written() {
    let set = set(&[], &SOUNDS);
    let cx = LinkCx::new(&set, ALL);
    assert_eq!(
        place(&cx.block_sound("demo_brass")),
        Some((PackSide::Resource, "sounds.json".to_owned())),
        "{}",
        said(&set)
    );
    assert_eq!(cx.block_sound("Demo_Brass"), MISSING);
    assert_eq!(cx.block_sound("demo_tin"), MISSING);
    // Neither a sound played by name nor a sound event.
    assert_eq!(cx.block_sound("Demo.Chime"), MISSING);
    assert_eq!(cx.block_sound("demo.ping"), MISSING);
}

#[cfg(feature = "sounds_json")]
#[test]
fn a_named_sound_is_a_key_of_individual_named_sounds_with_case() {
    let set = set(&[], &SOUNDS);
    let cx = LinkCx::new(&set, ALL);
    assert_eq!(
        place(&cx.named_sound("Demo.Chime")),
        Some((PackSide::Resource, "sounds.json".to_owned())),
        "{}",
        said(&set)
    );
    assert_eq!(cx.named_sound("demo.chime"), MISSING);
    assert_eq!(cx.named_sound("DEMO.CHIME"), MISSING);
    assert_eq!(cx.named_sound("Demo.Bell"), MISSING);
    // Neither a block sound nor a sound event.
    assert_eq!(cx.named_sound("demo_brass"), MISSING);
    assert_eq!(cx.named_sound("demo.ping"), MISSING);
}

// ---- text ------------------------------------------------------------------------------------

/// A key some language of either stack defines, compared without ASCII case.
#[cfg(all(feature = "lang", feature = "languages"))]
#[test]
fn a_lang_key_of_either_stack_is_found_without_ascii_case() {
    let set = set(
        &[
            ("texts/languages.json", r#"["en_US"]"#),
            ("texts/en_US.lang", "demo.Server=Server\n"),
        ],
        &[
            ("texts/languages.json", r#"["en_US"]"#),
            ("texts/en_US.lang", "demo.Greeting=Hello\n"),
        ],
    );
    let cx = LinkCx::new(&set, ALL);
    for key in ["demo.Greeting", "demo.greeting", "DEMO.GREETING"] {
        assert_eq!(
            place(&cx.lang_key(key)),
            Some((PackSide::Resource, "texts/en_US.lang".to_owned())),
            "{key}: {}",
            said(&set)
        );
    }
    // A key only the behaviour stack has.
    for key in ["demo.Server", "demo.server"] {
        assert_eq!(
            place(&cx.lang_key(key)),
            Some((PackSide::Behavior, "texts/en_US.lang".to_owned())),
            "{key}: {}",
            said(&set)
        );
    }
    assert_eq!(cx.lang_key("demo.farewell"), MISSING);
    assert_eq!(cx.lang_key("demo.greetin"), MISSING);
    assert_eq!(cx.lang_key("Hello"), MISSING);
}

/// The game loads the languages some `texts/languages.json` of the stack lists, and its own
/// packs, which are on every stack of the game, list its 29 languages: a `.lang` file of
/// another code that no pack lists is never loaded.
#[cfg(all(feature = "lang", feature = "languages"))]
#[test]
fn a_lang_key_of_a_language_neither_a_pack_nor_the_game_lists_is_not_found() {
    let set = set(
        &[],
        &[
            ("texts/languages.json", r#"["en_US"]"#),
            ("texts/en_US.lang", "demo.greeting=Hello\n"),
            ("texts/xx_XX.lang", "demo.abschied=Tschüss\n"),
        ],
    );
    let cx = LinkCx::new(&set, ALL);
    assert_eq!(
        file(&cx.lang_key("demo.greeting")).as_deref(),
        Some("texts/en_US.lang"),
        "{}",
        said(&set)
    );
    assert_eq!(cx.lang_key("demo.abschied"), MISSING);
}

/// A pack without `texts/languages.json` (the usual add-on) has its `texts/en_US.lang`
/// loaded by the game all the same, because the game's own packs list `en_US` and a listed
/// language is read from every pack of the stack. The lookup reads the `.lang` files of the
/// game's languages that no `languages.json` of the stack lists.
#[cfg(all(feature = "lang", feature = "languages"))]
#[test]
fn a_lang_key_of_a_language_of_the_game_is_found_in_a_pack_without_languages_json() {
    let set = set(
        &[("texts/en_US.lang", "demo.Server=Server\n")],
        &[
            ("texts/en_US.lang", "demo.Greeting=Hello\n"),
            ("texts/de_DE.lang", "demo.abschied=Tschüss\n"),
            ("texts/xx_XX.lang", "demo.nirgends=Nowhere\n"),
        ],
    );
    let cx = LinkCx::new(&set, ALL);
    for key in ["demo.Greeting", "demo.greeting", "DEMO.GREETING"] {
        assert_eq!(
            place(&cx.lang_key(key)),
            Some((PackSide::Resource, "texts/en_US.lang".to_owned())),
            "{key}: {}",
            said(&set)
        );
    }
    assert_eq!(
        place(&cx.lang_key("demo.abschied")),
        Some((PackSide::Resource, "texts/de_DE.lang".to_owned()))
    );
    assert_eq!(
        place(&cx.lang_key("demo.server")),
        Some((PackSide::Behavior, "texts/en_US.lang".to_owned()))
    );
    // A key no file has, and the key of a language nobody lists.
    assert_eq!(cx.lang_key("demo.farewell"), MISSING);
    assert_eq!(cx.lang_key("demo.nirgends"), MISSING);
}

/// A pack that lists some languages still has the game's others read: the list of a
/// pack adds to the game's, it does not replace it.
#[cfg(all(feature = "lang", feature = "languages"))]
#[test]
fn a_lang_key_of_a_language_of_the_game_a_pack_does_not_list_is_found() {
    let set = set(
        &[],
        &[
            ("texts/languages.json", r#"["en_US"]"#),
            ("texts/en_US.lang", "demo.greeting=Hello\n"),
            ("texts/de_DE.lang", "demo.abschied=Tschüss\n"),
        ],
    );
    let cx = LinkCx::new(&set, ALL);
    assert_eq!(
        file(&cx.lang_key("demo.abschied")).as_deref(),
        Some("texts/de_DE.lang"),
        "{}",
        said(&set)
    );
}

/// Every language starts from the base language (`en_US`) and falls
/// back to it, so a key the base language's files lack is missing whatever another
/// language's unread file may hold: an unread file of another language leaves no key of the
/// pass unjudged.
#[cfg(all(feature = "lang", feature = "languages"))]
#[test]
fn a_lang_key_is_held_to_the_base_language_when_another_languages_file_is_not_resident() {
    let set = set_of(
        Vec::new(),
        vec![pack(
            2,
            "resources",
            &[
                ("texts/en_US.lang", Some("demo.greeting=Hello\n")),
                ("texts/de_DE.lang", None),
            ],
        )],
        &LoadOptions::default(),
    );
    let cx = LinkCx::new(&set, ALL);
    assert_eq!(
        file(&cx.lang_key("demo.greeting")).as_deref(),
        Some("texts/en_US.lang")
    );
    assert_eq!(cx.lang_key("demo.farewell"), MISSING);
}

/// The unlisted files of the game's languages are read once for the pass: `n` lookups
/// among the `n` keys of a `.lang` file no `languages.json` lists cost `n`, hit or miss.
#[cfg(all(feature = "lang", feature = "languages"))]
#[test]
fn n_lang_keys_among_the_n_keys_of_an_unlisted_lang_file_cost_n() {
    use std::fmt::Write as _;
    fn world(n: usize) -> (StackSet, usize) {
        let mut text = String::new();
        for i in 0..n {
            let _ = writeln!(text, "demo.key_{i}=Text {i}");
        }
        let text: &'static str = Box::leak(text.into_boxed_str());
        let set = set_of(
            Vec::new(),
            vec![pack(2, "resources", &[("texts/en_US.lang", Some(text))])],
            &LoadOptions::default(),
        );
        (set, n)
    }
    crate::test_cost::assert_linear_at(
        "link: n lang keys among the n keys of an unlisted .lang file",
        20_000,
        world,
        |(set, n): &(StackSet, usize)| {
            // A pass of its own: the file is read in it, once.
            let cx = LinkCx::new(set, ALL);
            let mut missing = 0usize;
            for i in 0..*n {
                let key = if i % 2 == 0 {
                    format!("demo.key_{i}")
                } else {
                    format!("demo.none_{i}")
                };
                missing += usize::from(cx.lang_key(&key).is_missing());
            }
            assert_eq!(missing, n / 2);
            missing
        },
    );
}

/// The list of the game's languages is `texts/languages.json` of the game's own resource
/// pack, in its order.
#[cfg(all(feature = "lang", feature = "languages"))]
#[test]
fn the_languages_of_the_game_are_the_vanilla_resource_packs_list() {
    use bedrock_addon_test_support::corpus;
    let Some(game_files) = std::env::var_os("BEDROCK_ADDON_CORPUS").filter(|v| !v.is_empty())
    else {
        corpus::skipped("the languages of the game", corpus::not_set(corpus::CORPUS));
        return;
    };
    let game_files = std::path::Path::new(&game_files);
    for file in [
        "bedrock-samples/resource_pack/texts/languages.json",
        "gdk-retail/resource_packs/vanilla/texts/languages.json",
        "bds-1.26.36.1/resource_packs/vanilla/texts/languages.json",
    ] {
        let text = std::fs::read_to_string(game_files.join(file)).expect(file);
        let codes: Vec<String> = crate::json::from_str_jsonc(&text).expect(file);
        assert_eq!(codes, crate::vanilla::names::VANILLA_LANGUAGES, "{file}");
    }
}

/// A `.lang` file of a language of the game that the pack lists by path and does not
/// hold the bytes of cannot be read: a key no other file has is not judged.
#[cfg(all(feature = "lang", feature = "languages"))]
#[test]
fn a_lang_key_is_not_judged_when_a_lang_file_of_the_game_s_languages_is_not_resident() {
    let set = set_of(
        Vec::new(),
        vec![pack(2, "resources", &[("texts/en_US.lang", None)])],
        &LoadOptions::default(),
    );
    let cx = LinkCx::new(&set, ALL);
    assert_eq!(cx.lang_key("demo.greeting"), Target::NotInView);
}
