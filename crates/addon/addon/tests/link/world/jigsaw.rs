//! 20 W16 to W22: the references of the four jigsaw documents. W23 (generation) and W24 (block
//! tags) are audited in `audit.rs`.

#![allow(unused_imports, reason = "which rows compile depends on the features")]

use bedrock_addon::pack::LinkOptions;

use super::super::common::{hosted, link, link_with};
use super::{said, texts, unresolved_error, unresolved_info};

const POOL: &str = "worldgen/template_pools/pool.json";
const STRUCTURE: &str = "worldgen/structures/ruin.json";
const SET: &str = "worldgen/structure_sets/ruins.json";
const PROCESSORS: &str = "worldgen/processors/weather.json";

/// A template pool `id` with `members` after its description and the elements `elements`.
fn pool(id: &str, members: &str, elements: &str) -> String {
    format!(
        r#"{{"format_version": "1.21.20", "minecraft:template_pool": {{"description": {{"identifier": "{id}"}}{members}, "elements": [{elements}]}}}}"#
    )
}

/// A jigsaw structure `id` that starts in `start_pool`, with `members` after it.
fn structure(id: &str, start_pool: &str, members: &str) -> String {
    format!(
        r#"{{"format_version": "1.21.20", "minecraft:jigsaw": {{"description": {{"identifier": "{id}"}}, "step": "surface_structures", "start_pool": "{start_pool}", "max_depth": 4, "start_height": {{"type": "constant", "value": {{"absolute": 0}}}}{members}}}}}"#
    )
}

/// A structure set `demo:ruins` that places `structure`.
fn set(structure: &str) -> String {
    format!(
        r#"{{"format_version": "1.21.20", "minecraft:structure_set": {{"description": {{"identifier": "demo:ruins"}}, "placement": {{"type": "minecraft:random_spread", "salt": 1, "separation": 8, "spacing": 34, "spread_type": "triangular"}}, "structures": [{{"structure": "{structure}", "weight": 1}}]}}}}"#
    )
}

/// A processor list `demo:weather` with the processors `processors`.
fn processors(processors: &str) -> String {
    format!(
        r#"{{"format_version": "1.21.20", "minecraft:processor_list": {{"description": {{"identifier": "demo:weather"}}, "processors": [{processors}]}}}}"#
    )
}

/// A feature pool element.
fn feature_element(feature: &str) -> String {
    format!(
        r#"{{"element": {{"element_type": "minecraft:feature_pool_element", "feature": "{feature}"}}}}"#
    )
}

// ---- W16: template pools ---------------------------------------------------------------------

#[test]
fn w16_a_pool_a_pack_defines_or_the_game_has_resolves() {
    let start = pool("demo:start", r#", "fallback": "demo:end""#, "");
    let end = pool("demo:end", r#", "fallback": "minecraft:empty""#, "");
    let ruin = structure(
        "demo:ruin",
        "demo:start",
        r#", "pool_aliases": [{"alias": "demo:a", "target": "demo:end"}, {"alias": "demo:b", "targets": [{"weight": 1, "data": "demo:start"}]}, {"groups": [{"weight": 1, "data": [{"alias": "demo:c", "target": "demo:end"}]}]}]"#,
    );
    let found = link(
        &[
            ("worldgen/template_pools/start.json", start.as_str()),
            ("worldgen/template_pools/end.json", end.as_str()),
            (STRUCTURE, ruin.as_str()),
        ],
        &[],
    );
    assert_eq!(said(&found, &["template pool"]), []);
    // A host answers for a pool of the game.
    let ruin = structure("demo:ruin", "minecraft:village/plains/town_centers", "");
    let found = link_with(
        &[(STRUCTURE, ruin.as_str())],
        &[],
        hosted(&[("template pool", "minecraft:village/plains/town_centers")]),
        &LinkOptions::default(),
    );
    assert_eq!(said(&found, &["template pool"]), []);
}

#[test]
fn w16_a_pool_nothing_defines_is_an_error_at_each_member() {
    let start = pool("demo:start", r#", "fallback": "demo:gone""#, "");
    let ruin = structure(
        "demo:ruin",
        "demo:nowhere",
        r#", "pool_aliases": [{"alias": "demo:a", "target": "demo:t"}, {"alias": "demo:b", "targets": [{"weight": 1, "data": "demo:start"}, {"weight": 1, "data": "demo:u"}]}, {"groups": [{"weight": 1, "data": [{"alias": "demo:c", "target": "demo:v"}]}]}]"#,
    );
    let found = link(&[(POOL, start.as_str()), (STRUCTURE, ruin.as_str())], &[]);
    let jigsaw = |member: &str| format!("minecraft:jigsaw/{member}");
    assert_eq!(
        said(&found, &["template pool"]),
        [
            unresolved_error(
                STRUCTURE,
                &jigsaw("start_pool"),
                "template pool",
                "demo:nowhere"
            ),
            unresolved_error(
                STRUCTURE,
                &jigsaw("pool_aliases/0/target"),
                "template pool",
                "demo:t"
            ),
            unresolved_error(
                STRUCTURE,
                &jigsaw("pool_aliases/1/targets/1/data"),
                "template pool",
                "demo:u"
            ),
            unresolved_error(
                STRUCTURE,
                &jigsaw("pool_aliases/2/groups/0/data/0/target"),
                "template pool",
                "demo:v"
            ),
            unresolved_error(
                POOL,
                "minecraft:template_pool/fallback",
                "template pool",
                "demo:gone"
            ),
        ]
    );
    assert_eq!(
        texts(&found, "template pool")[0],
        "-> start_pool:  Missing referenced asset demo:nowhere"
    );
}

// ---- W17: structure templates ----------------------------------------------------------------

#[cfg(feature = "structure")]
#[test]
fn w17_a_template_a_pack_has_or_the_game_has_resolves_without_its_namespace() {
    use bedrock_addon::families::structure::Structure;
    use bedrock_addon::load::LoadOptions;

    use super::super::common::{behavior_pack_of, leak, stacks_of};
    let template: &'static [u8] = Box::leak(
        Structure::new([1, 1, 1])
            .to_bytes()
            .expect("a structure encodes")
            .into_boxed_slice(),
    );
    let elements = r#"{"element": {"element_type": "minecraft:single_pool_element", "location": "demo:ruin/hall"}}, {"element": {"element_type": "minecraft:single_pool_element", "location": "ruin/hall"}}, {"element": {"element_type": "minecraft:single_pool_element", "location": "demo:ruin/gone"}}"#;
    let text = pool("demo:start", "", elements);
    let pack = || {
        behavior_pack_of(
            1,
            &[
                ("structures/ruin/hall.mcstructure", template),
                (POOL, leak(&text)),
            ],
        )
    };
    // The game's validator line beside each miss is `linked_asset.rs`'s.
    let found = crate::common::family_findings(
        &stacks_of(vec![pack()], Vec::new(), LoadOptions::default()).link(),
    );
    assert_eq!(
        said(&found, &["structure"]),
        [unresolved_info(
            POOL,
            "minecraft:template_pool/elements/2/element/location",
            "structure",
            "demo:ruin/gone"
        )]
    );
    assert_eq!(
        texts(&found, "structure"),
        [
            "[JIGSAW GENERATION FAILURE] Structure name [demo:ruin/gone] referenced in template pool [demo:start] does not exist."
        ]
    );
    // A host answers for a template of the game.
    let found = stacks_of(
        vec![pack()],
        Vec::new(),
        hosted(&[("structure", "demo:ruin/gone")]),
    )
    .link();
    assert_eq!(said(&found, &["structure"]), []);
}

/// The game's own templates are `structures/<location>.nbt` files of the packs it ships: the
/// game reads such a file when no pack has the `.mcstructure`, and only from its own packs.
#[cfg(feature = "structure")]
#[test]
fn w17_a_legacy_template_resolves_in_a_pack_of_the_game_and_nowhere_else() {
    use bedrock_addon::load::LoadOptions;

    use super::stacks_with_game;
    let element = |location: &str| {
        format!(
            r#"{{"element": {{"element_type": "minecraft:single_pool_element", "location": "{location}"}}}}"#
        )
    };
    let elements = [
        element("trail_ruins/tower/tower_1"),
        // The namespace is no part of the file's name.
        element("minecraft:trail_ruins/tower/tower_1"),
        element("trail_ruins/tower/tower_2"),
    ]
    .join(", ");
    let text = pool("demo:start", "", &elements);
    // The bytes of a legacy template are never read: the listing answers.
    let template = [("structures/trail_ruins/tower/tower_1.nbt", "not read")];
    let set = stacks_with_game(&[(POOL, text.as_str())], &template, LoadOptions::default());
    // The game's validator line beside each miss is `linked_asset.rs`'s.
    assert_eq!(
        said(&crate::common::family_findings(&set.link()), &["structure"]),
        [unresolved_info(
            POOL,
            "minecraft:template_pool/elements/2/element/location",
            "structure",
            "trail_ruins/tower/tower_2"
        )]
    );
    // The same file in a pack of the world is no template: the game never reads it there.
    let mut world = vec![(POOL, text.as_str())];
    world.extend(template);
    let set = stacks_with_game(&world, &[], LoadOptions::default());
    let found = set.link();
    // The game's validator finds the file in the world's pack: no line for the first
    // template; the missing one has its line beside rule W17's finding.
    let lines: Vec<String> = found
        .iter()
        .map(|f| f.code.to_string())
        .filter(|t| t.contains(":  Invalid asset path "))
        .collect();
    assert_eq!(
        lines,
        ["-> elements[2] -> element -> location:  Invalid asset path trail_ruins/tower/tower_2"]
    );
    assert_eq!(
        said(&crate::common::family_findings(&found), &["structure"])
            .iter()
            .map(|said| said.5.as_str())
            .collect::<Vec<_>>(),
        [
            "trail_ruins/tower/tower_1",
            "minecraft:trail_ruins/tower/tower_1",
            "trail_ruins/tower/tower_2"
        ]
    );
}

// ---- W18: processor lists --------------------------------------------------------------------

#[test]
fn w18_a_processor_list_resolves_and_the_empty_name_is_none() {
    let element = |list: &str| {
        format!(
            r#"{{"element": {{"element_type": "minecraft:single_pool_element", "location": "demo:hall", "processors": "{list}"}}}}"#
        )
    };
    let elements = [
        element("demo:weather"),
        element(""),
        element("demo:gone"),
        element("minecraft:mossify"),
    ]
    .join(", ");
    let (start, weather) = (pool("demo:start", "", &elements), processors(""));
    let files = [(POOL, start.as_str()), (PROCESSORS, weather.as_str())];
    let found = link(&files, &[]);
    let at = |index: usize| format!("minecraft:template_pool/elements/{index}/element/processors");
    assert_eq!(
        said(&found, &["processor list"]),
        [
            unresolved_error(POOL, &at(2), "processor list", "demo:gone"),
            unresolved_error(POOL, &at(3), "processor list", "minecraft:mossify"),
        ]
    );
    assert_eq!(
        texts(&found, "processor list")[0],
        "-> elements[2] -> element -> processors:  Missing referenced asset demo:gone"
    );
    // A host answers for a list of the game.
    let found = link_with(
        &files,
        &[],
        hosted(&[("processor list", "minecraft:mossify")]),
        &LinkOptions::default(),
    );
    assert_eq!(said(&found, &["processor list"]).len(), 1);
}

// ---- W19: features of a pool -----------------------------------------------------------------

#[test]
fn w19_a_feature_of_a_pool_resolves_or_is_a_warning_with_the_rows_text() {
    let vein = r#"{"format_version": "1.13.0", "minecraft:ore_feature": {"description": {"identifier": "demo:vein"}, "count": 16}}"#;
    let elements = [feature_element("demo:vein"), feature_element("demo:gone")].join(", ");
    let start = pool("demo:start", "", &elements);
    let files = [(POOL, start.as_str()), ("features/vein.json", vein)];
    let found = link(&files, &[]);
    assert_eq!(
        said(&found, &["feature"]),
        [unresolved_info(
            POOL,
            "minecraft:template_pool/elements/1/element/feature",
            "feature",
            "demo:gone"
        )]
    );
    assert_eq!(
        texts(&found, "feature"),
        [
            "Cannot find 'feature' 'demo:gone' for 'feature_pool_element'. Make sure to include the 'namespace_name:' prefix."
        ]
    );
    // A host answers for a feature of the game.
    let found = link_with(
        &files,
        &[],
        hosted(&[("feature", "demo:gone")]),
        &LinkOptions::default(),
    );
    assert_eq!(said(&found, &["feature"]), []);
}

// ---- W20: the structures of a set ------------------------------------------------------------

#[test]
fn w20_a_structure_of_a_set_resolves_or_is_an_error() {
    let start = pool("demo:start", "", "");
    let ruin = structure("demo:ruin", "demo:start", "");
    // A jigsaw structure of the pack, and a structure feature of the game by the crate's list.
    for known in ["demo:ruin", "minecraft:village"] {
        let ruins = set(known);
        let found = link(
            &[
                (POOL, start.as_str()),
                (STRUCTURE, ruin.as_str()),
                (SET, ruins.as_str()),
            ],
            &[],
        );
        assert_eq!(said(&found, &["jigsaw structure"]), [], "{known}");
    }
    let ruins = set("demo:gone");
    let files = [(SET, ruins.as_str())];
    let found = link(&files, &[]);
    assert_eq!(
        said(&found, &["jigsaw structure"]),
        [unresolved_error(
            SET,
            "minecraft:structure_set/structures/0/structure",
            "jigsaw structure",
            "demo:gone"
        )]
    );
    let found = link_with(
        &files,
        &[],
        hosted(&[("jigsaw structure", "demo:gone")]),
        &LinkOptions::default(),
    );
    assert_eq!(said(&found, &["jigsaw structure"]), []);
}

// ---- W21: the blocks of a processor ------------------------------------------------------------

/// A processor list that names `block` in every member of the row.
#[cfg(feature = "block")]
fn naming(block: &str) -> String {
    processors(&format!(
        r#"{{"processor_type": "minecraft:block_ignore", "blocks": ["{block}"]}}, {{"processor_type": "minecraft:rule", "rules": [{{"input_predicate": {{"predicate_type": "minecraft:block_match", "block": "{block}"}}, "location_predicate": {{"predicate_type": "minecraft:random_block_match", "block": "{block}", "probability": 0.5}}, "output_state": {{"name": "{block}"}}}}, {{"input_predicate": {{"predicate_type": "minecraft:blockstate_match", "block_state": {{"name": "{block}"}}}}, "output_state": {{"name": "minecraft:stone"}}}}]}}"#
    ))
}

#[cfg(feature = "block")]
#[test]
fn w21_a_block_of_a_processor_a_pack_defines_or_the_game_has_resolves() {
    let lamp = super::block("demo:lamp");
    for known in ["demo:lamp", "minecraft:gravel"] {
        let weather = naming(known);
        let found = link(
            &[("blocks/lamp.json", lamp.as_str()), (PROCESSORS, &weather)],
            &[],
        );
        assert_eq!(said(&found, &["block"]), [], "{known}");
    }
}

#[cfg(feature = "block")]
#[test]
fn w21_a_block_nothing_defines_is_one_warning_per_member() {
    let weather = naming("demo:lamp");
    let files = [(PROCESSORS, weather.as_str())];
    // Rule 12's findings; the game's validator lines of the rules' blocks are
    // `linked_asset.rs`'s.
    let all = link(&files, &[]);
    let found = crate::common::without_validator_lines(&all);
    let at = |member: &str| format!("minecraft:processor_list/processors/{member}");
    assert_eq!(
        said(&found, &["block"]),
        [
            unresolved_error(PROCESSORS, &at("0/blocks/0"), "block", "demo:lamp"),
            unresolved_error(
                PROCESSORS,
                &at("1/rules/0/input_predicate/block"),
                "block",
                "demo:lamp"
            ),
            unresolved_error(
                PROCESSORS,
                &at("1/rules/0/location_predicate/block"),
                "block",
                "demo:lamp"
            ),
            unresolved_error(
                PROCESSORS,
                &at("1/rules/0/output_state"),
                "block",
                "demo:lamp"
            ),
            unresolved_error(
                PROCESSORS,
                &at("1/rules/1/input_predicate/block_state"),
                "block",
                "demo:lamp"
            ),
        ]
    );
    assert_eq!(texts(&found, "block")[0], "Cannot find block 'demo:lamp'");
    let found = link_with(
        &files,
        &[],
        hosted(&[("block", "demo:lamp"), ("block", "minecraft:stone")]),
        &LinkOptions::default(),
    );
    assert_eq!(said(&found, &["block"]), []);
}

/// Without the `block` module nobody knows a pack's blocks.
#[cfg(not(feature = "block"))]
#[test]
fn w21_without_the_block_module_no_block_of_a_processor_is_reported() {
    let weather =
        processors(r#"{"processor_type": "minecraft:block_ignore", "blocks": ["demo:lamp"]}"#);
    let found = link(&[(PROCESSORS, weather.as_str())], &[]);
    assert_eq!(said(&found, &["block"]), []);
}

// ---- W22: the loot table of a processor rule ---------------------------------------------------

#[cfg(feature = "loot")]
#[test]
fn w22_a_loot_table_resolves_with_or_without_its_extension_or_is_an_error() {
    let rule = |table: &str| {
        format!(
            r#"{{"output_state": {{"name": "minecraft:stone"}}, "block_entity_modifier": {{"type": "minecraft:append_loot", "loot_table": "{table}"}}}}"#
        )
    };
    let rules = [
        rule("loot_tables/chests/ruin.json"),
        // The game's own lists write a table without its extension.
        rule("loot_tables/chests/ruin"),
        rule("Loot_Tables/Chests/Ruin.json"),
        rule("loot_tables/chests/gone.json"),
    ]
    .join(", ");
    let weather = processors(&format!(
        r#"{{"processor_type": "minecraft:rule", "rules": [{rules}]}}"#
    ));
    let files = [
        (PROCESSORS, weather.as_str()),
        ("loot_tables/chests/ruin.json", r#"{"pools": []}"#),
    ];
    let found = link(&files, &[]);
    assert_eq!(
        said(&found, &["loot table"]),
        [unresolved_error(
            PROCESSORS,
            "minecraft:processor_list/processors/0/rules/3/block_entity_modifier/loot_table",
            "loot table",
            "loot_tables/chests/gone.json"
        )]
    );
    assert_eq!(
        texts(&found, "loot table"),
        [
            "-> processors[0] -> rules[3] -> block_entity_modifier -> loot_table:  Invalid asset path loot_tables/chests/gone.json"
        ]
    );
    let found = link_with(
        &files,
        &[],
        hosted(&[("loot table", "loot_tables/chests/gone.json")]),
        &LinkOptions::default(),
    );
    assert_eq!(said(&found, &["loot table"]), []);
}
