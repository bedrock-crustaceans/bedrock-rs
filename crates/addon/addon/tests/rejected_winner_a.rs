#![cfg(feature = "full")]
//! Pins of present behaviour: a higher behaviour pack holds a *refused* definition of an
//! identifier (or, for loot tables, of a file path) and a lower pack a valid one. Either the
//! lower definition is effective ("fallback") or the identifier stays without an effective
//! definition ("taken"). Per family and kind of refusal there is a two-pack test (top refused,
//! low valid) and a three-pack test (top refused, middle valid, low valid) that says which
//! pack's definition is effective, by the document's origin (pack position, 0 = top, and file).
//!
//! Every test also requires the refusal to be reported: the error findings at the top pack's
//! file, by legacy code name.
//!
//! The comment above each family states what the game does and the present behaviour.

use bedrock_addon::diagnostic::{Diagnostic, Severity};
use bedrock_addon::document::{Document, Rejected};
use bedrock_addon::families::feature_rules::PlacementPass;
use bedrock_addon::families::recipe::RecipeNamespace;
use bedrock_addon::id::Id;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{Behavior, OpenOptions, Pack, PackOrigin, PackStack};
use bedrock_addon::storage::PackPath;
use uuid::Uuid;

/// A behaviour pack `n` of `origin` holding `files` besides its manifest.
fn pack_of(n: u128, origin: PackOrigin, files: &[(&str, &str)]) -> Pack {
    let manifest = format!(
        r#"{{"format_version": 2, "header": {{"name": "bp {n}", "uuid": "{}", "version": [1, 0, 0], "min_engine_version": [1, 26, 20]}}, "modules": [{{"type": "data", "uuid": "{}", "version": [1, 0, 0]}}]}}"#,
        Uuid::from_u128(n),
        Uuid::from_u128(n + 1000),
    );
    let mut all = vec![(
        PackPath::new("manifest.json").expect("a pack path"),
        manifest.into_bytes(),
    )];
    all.extend(files.iter().map(|(path, text)| {
        (
            PackPath::new(path).expect("a pack path"),
            text.as_bytes().to_vec(),
        )
    }));
    let options = OpenOptions {
        origin,
        ..OpenOptions::default()
    };
    Pack::from_files(all, &options).expect("the pack opens")
}

/// A user-installed behaviour pack `n` holding the one file `path`.
fn pack(n: u128, path: &str, text: &str) -> Pack {
    pack_of(n, PackOrigin::User, &[(path, text)])
}

/// A behaviour stack of `packs`, top first.
fn stack(packs: Vec<Pack>) -> PackStack<Behavior> {
    PackStack::<Behavior>::of(packs, LoadOptions::default()).expect("a behaviour stack")
}

/// Where a definition comes from: its pack's position in the stack (0 = top) and its file.
type Source = (Option<usize>, String);

/// The source of the effective definition, `None` when there is none.
fn source<T>(document: Option<&Document<T>>) -> Option<Source> {
    document.map(|d| (d.origin().pack.index(), d.origin().path.as_str().to_owned()))
}

/// The effective definition is the one of the pack at `position`, in `path`.
fn from_pack(position: usize, path: &str) -> Source {
    (Some(position), path.to_owned())
}

/// The sources of the rejected files.
fn refused<'a>(rejected: impl IntoIterator<Item = &'a Rejected>) -> Vec<Source> {
    rejected
        .into_iter()
        .map(|r| (r.origin.pack.index(), r.origin.path.as_str().to_owned()))
        .collect()
}

/// The variant names of the error findings at the top pack's file `path`.
fn errors_at_top<'a>(findings: impl Iterator<Item = &'a Diagnostic>, path: &str) -> Vec<String> {
    findings
        .filter(|d| d.severity == Severity::Error)
        .filter(|d| {
            d.location
                .pack
                .as_ref()
                .and_then(bedrock_addon::pack::PackRef::index)
                == Some(0)
        })
        .filter(|d| d.location.path.as_ref().map(PackPath::as_str) == Some(path))
        .map(|d| d.code.id().to_owned())
        .collect()
}

// ---- blocks ------------------------------------------------------------------------------
// The game says nothing about the stack here; for one file, a failure is an error and the block
// is skipped.
// Present behaviour: fallback. The registry is top-wins and a rejected file takes no part in
// it, so the highest *valid* definition is effective.

/// A valid `demo:b`.
const BLOCK: &str = r#"{"format_version": "1.26.20", "minecraft:block": {"description": {"identifier": "demo:b"}, "components": {"minecraft:friction": 0.4}}}"#;
/// `demo:b` refused after its identifier is read: `components` is not an object.
const BLOCK_REFUSED: &str = r#"{"format_version": "1.26.20", "minecraft:block": {"description": {"identifier": "demo:b"}, "components": []}}"#;

#[test]
fn blocks_two_packs_a_refused_top_block_leaves_the_lower_block_effective() {
    let stack = stack(vec![
        pack(1, "blocks/x.json", BLOCK_REFUSED),
        pack(2, "blocks/l.json", BLOCK),
    ]);
    let blocks = stack.blocks();
    assert_eq!(
        source(blocks.get(&Id::parse("demo:b"))),
        Some(from_pack(1, "blocks/l.json"))
    );
    assert_eq!(
        refused(blocks.rejected()),
        [(Some(0), "blocks/x.json".to_owned())]
    );
    assert_eq!(
        errors_at_top(blocks.diagnostics(), "blocks/x.json"),
        ["core/invalid_type"]
    );
}

#[test]
fn blocks_three_packs_a_refused_top_block_leaves_the_middle_block_effective() {
    let stack = stack(vec![
        pack(1, "blocks/x.json", BLOCK_REFUSED),
        pack(2, "blocks/m.json", BLOCK),
        pack(3, "blocks/l.json", BLOCK),
    ]);
    let blocks = stack.blocks();
    assert_eq!(
        source(blocks.get(&Id::parse("demo:b"))),
        Some(from_pack(1, "blocks/m.json"))
    );
    assert_eq!(
        refused(blocks.rejected()),
        [(Some(0), "blocks/x.json".to_owned())]
    );
    assert_eq!(
        errors_at_top(blocks.diagnostics(), "blocks/x.json"),
        ["core/invalid_type"]
    );
}

// ---- items, the stable route --------------------------------------------------------------
// The game says nothing about the stack here; a single file that fails leaves the item
// "failed to load" / "not registered".
// Present behaviour: fallback, as for blocks (top-wins registry, a rejected file takes no part).

/// A valid `demo:i`.
const ITEM: &str = r#"{"format_version": "1.26.30", "minecraft:item": {"description": {"identifier": "demo:i"}, "components": {"minecraft:max_stack_size": 8}}}"#;
/// `demo:i` refused after its identifier is read: `components` is not an object.
const ITEM_REFUSED: &str = r#"{"format_version": "1.26.30", "minecraft:item": {"description": {"identifier": "demo:i"}, "components": []}}"#;

#[test]
fn items_two_packs_a_refused_top_item_leaves_the_lower_item_effective() {
    let stack = stack(vec![
        pack(1, "items/x.json", ITEM_REFUSED),
        pack(2, "items/l.json", ITEM),
    ]);
    let items = stack.items();
    assert_eq!(
        source(items.get(&Id::parse("demo:i"))),
        Some(from_pack(1, "items/l.json"))
    );
    assert_eq!(
        refused(items.rejected()),
        [(Some(0), "items/x.json".to_owned())]
    );
    assert_eq!(
        errors_at_top(items.diagnostics(), "items/x.json"),
        ["item/components_not_object"]
    );
}

#[test]
fn items_three_packs_a_refused_top_item_leaves_the_middle_item_effective() {
    let stack = stack(vec![
        pack(1, "items/x.json", ITEM_REFUSED),
        pack(2, "items/m.json", ITEM),
        pack(3, "items/l.json", ITEM),
    ]);
    let items = stack.items();
    assert_eq!(
        source(items.get(&Id::parse("demo:i"))),
        Some(from_pack(1, "items/m.json"))
    );
    assert_eq!(
        refused(items.rejected()),
        [(Some(0), "items/x.json".to_owned())]
    );
    assert_eq!(
        errors_at_top(items.diagnostics(), "items/x.json"),
        ["item/components_not_object"]
    );
}

// ---- items, the override gate -------------------------------------------------------------
// Vanilla-item override gates are errors and the item is not registered: a `minecraft:` item
// whose document is below 1.16.100 (legacy format), where the vanilla item exists only as a
// 1.16.100+ (component-format) definition. Base-game packs define the vanilla set. The game does
// not say what stays effective.
// Present behaviour: fallback. The gated override is rejected at the combine step and the
// definition below it stays effective: the base-game pack's, or a valid override between.

/// `minecraft:apple` in the component format.
const APPLE: &str = r#"{"format_version": "1.26.30", "minecraft:item": {"description": {"identifier": "minecraft:apple"}, "components": {"minecraft:max_stack_size": 8}}}"#;
/// `minecraft:apple` in the legacy format: valid alone, refused by the gate above a
/// component-format vanilla apple.
const APPLE_LEGACY: &str = r#"{"format_version": "1.10", "minecraft:item": {"description": {"identifier": "minecraft:apple"}, "components": {"minecraft:max_stack_size": 4}}}"#;

#[test]
fn items_two_packs_a_gated_top_override_leaves_the_base_game_item_effective() {
    let stack = stack(vec![
        pack(1, "items/x.json", APPLE_LEGACY),
        pack_of(2, PackOrigin::Package, &[("items/l.json", APPLE)]),
    ]);
    let items = stack.items();
    assert_eq!(
        source(items.get(&Id::parse("minecraft:apple"))),
        Some(from_pack(1, "items/l.json"))
    );
    assert_eq!(
        refused(items.rejected()),
        [(Some(0), "items/x.json".to_owned())]
    );
    assert_eq!(
        errors_at_top(items.diagnostics(), "items/x.json"),
        ["item/vanilla_is_component_based"]
    );
}

#[test]
fn items_three_packs_a_gated_top_override_leaves_the_middle_override_effective() {
    let stack = stack(vec![
        pack(1, "items/x.json", APPLE_LEGACY),
        pack(2, "items/m.json", APPLE),
        pack_of(3, PackOrigin::Package, &[("items/l.json", APPLE)]),
    ]);
    let items = stack.items();
    assert_eq!(
        source(items.get(&Id::parse("minecraft:apple"))),
        Some(from_pack(1, "items/m.json"))
    );
    assert_eq!(
        refused(items.rejected()),
        [(Some(0), "items/x.json".to_owned())]
    );
    assert_eq!(
        errors_at_top(items.diagnostics(), "items/x.json"),
        ["item/vanilla_is_component_based"]
    );
}

// ---- entities -----------------------------------------------------------------------------
// The game says nothing about the stack here; for one file, the entity is skipped.
// Present behaviour: fallback (top-wins registry, a rejected file takes no part). The refused
// file closes with the game's `ERROR: Entity '{}' failed to load from JSON: {}`
// (`DocumentRejected`) after its own error.

/// A valid `demo:e`.
const ENTITY: &str = r#"{"format_version": "1.26.50", "minecraft:entity": {"description": {"identifier": "demo:e"}, "components": {"minecraft:health": {"value": 3}}}}"#;
/// `demo:e` refused after its identifier is read: `components` is not an object.
const ENTITY_REFUSED: &str = r#"{"format_version": "1.26.50", "minecraft:entity": {"description": {"identifier": "demo:e"}, "components": []}}"#;

#[test]
fn entities_two_packs_a_refused_top_entity_leaves_the_lower_entity_effective() {
    let stack = stack(vec![
        pack(1, "entities/x.json", ENTITY_REFUSED),
        pack(2, "entities/l.json", ENTITY),
    ]);
    let entities = stack.entities();
    assert_eq!(
        source(entities.get(&Id::parse("demo:e"))),
        Some(from_pack(1, "entities/l.json"))
    );
    assert_eq!(
        refused(entities.rejected()),
        [(Some(0), "entities/x.json".to_owned())]
    );
    assert_eq!(
        errors_at_top(entities.diagnostics(), "entities/x.json"),
        ["core/invalid_type", "entity/failed_to_load"]
    );
}

#[test]
fn entities_three_packs_a_refused_top_entity_leaves_the_middle_entity_effective() {
    let stack = stack(vec![
        pack(1, "entities/x.json", ENTITY_REFUSED),
        pack(2, "entities/m.json", ENTITY),
        pack(3, "entities/l.json", ENTITY),
    ]);
    let entities = stack.entities();
    assert_eq!(
        source(entities.get(&Id::parse("demo:e"))),
        Some(from_pack(1, "entities/m.json"))
    );
    assert_eq!(
        refused(entities.rejected()),
        [(Some(0), "entities/x.json".to_owned())]
    );
    assert_eq!(
        errors_at_top(entities.diagnostics(), "entities/x.json"),
        ["core/invalid_type", "entity/failed_to_load"]
    );
}

// ---- spawn rules --------------------------------------------------------------------------
// A later document with the same identifier replaces the earlier one; with the pack stack read
// bottom to top the top pack wins (inferred). Nothing is known about a top document that fails.
// Present behaviour: fallback (top-wins registry, a rejected file takes no part). The only
// error-level refusal the family has is the version gate; its other "not registered" case
// (an empty or unknown `population_control`) is a warning on a
// document that loads and is not pinned here.

/// Valid rules of `x:pig`.
const SPAWN: &str = r#"{"format_version": "1.8.0", "minecraft:spawn_rules": {"description": {"identifier": "x:pig", "population_control": "animal"}, "conditions": [{"minecraft:weight": {"default": 9}}]}}"#;
/// Rules of `x:pig` refused by the version gate: no parser for 9.9.9.
const SPAWN_REFUSED: &str = r#"{"format_version": "9.9.9", "minecraft:spawn_rules": {"description": {"identifier": "x:pig", "population_control": "animal"}, "conditions": [{"minecraft:weight": {"default": 9}}]}}"#;

#[test]
fn spawn_rules_two_packs_a_refused_top_file_leaves_the_lower_rule_effective() {
    let stack = stack(vec![
        pack(1, "spawn_rules/x.json", SPAWN_REFUSED),
        pack(2, "spawn_rules/l.json", SPAWN),
    ]);
    let rules = stack.spawn_rules();
    assert_eq!(
        source(rules.get(&Id::parse("x:pig"))),
        Some(from_pack(1, "spawn_rules/l.json"))
    );
    assert_eq!(
        refused(rules.rejected()),
        [(Some(0), "spawn_rules/x.json".to_owned())]
    );
    assert_eq!(
        errors_at_top(rules.diagnostics(), "spawn_rules/x.json"),
        ["core/no_parser_for_version", "spawn_rules/failed_rules"]
    );
}

#[test]
fn spawn_rules_three_packs_a_refused_top_file_leaves_the_middle_rule_effective() {
    let stack = stack(vec![
        pack(1, "spawn_rules/x.json", SPAWN_REFUSED),
        pack(2, "spawn_rules/m.json", SPAWN),
        pack(3, "spawn_rules/l.json", SPAWN),
    ]);
    let rules = stack.spawn_rules();
    assert_eq!(
        source(rules.get(&Id::parse("x:pig"))),
        Some(from_pack(1, "spawn_rules/m.json"))
    );
    assert_eq!(
        refused(rules.rejected()),
        [(Some(0), "spawn_rules/x.json".to_owned())]
    );
    assert_eq!(
        errors_at_top(rules.diagnostics(), "spawn_rules/x.json"),
        ["core/no_parser_for_version", "spawn_rules/failed_rules"]
    );
}

// ---- feature rules ------------------------------------------------------------------------
// A parse failure skips the file; the pack and the other files continue. The insert is
// first-wins. A skipped file inserts nothing, so fallback follows.
// Present behaviour: fallback. A rejected rule never names its identifier to the registry.

/// A valid `demo:r` in the surface pass.
const FEATURE_RULE: &str = r#"{"format_version": "1.13.0", "minecraft:feature_rules": {"description": {"identifier": "demo:r", "places_feature": "demo:f"}, "conditions": {"placement_pass": "surface_pass"}}}"#;
/// `demo:r` refused after its identifier is read: the required `places_feature` is missing.
const FEATURE_RULE_REFUSED: &str = r#"{"format_version": "1.13.0", "minecraft:feature_rules": {"description": {"identifier": "demo:r"}, "conditions": {"placement_pass": "surface_pass"}}}"#;

#[test]
fn feature_rules_two_packs_a_refused_top_rule_leaves_the_lower_rule_effective() {
    let stack = stack(vec![
        pack(1, "feature_rules/x.json", FEATURE_RULE_REFUSED),
        pack(2, "feature_rules/l.json", FEATURE_RULE),
    ]);
    let rules = stack.feature_rules();
    assert_eq!(
        source(rules.get(PlacementPass::Surface, &Id::parse("demo:r"))),
        Some(from_pack(1, "feature_rules/l.json"))
    );
    assert_eq!(
        refused(rules.rejected()),
        [(Some(0), "feature_rules/x.json".to_owned())]
    );
    assert_eq!(
        errors_at_top(rules.diagnostics(), "feature_rules/x.json"),
        ["core/missing_required_field"]
    );
}

#[test]
fn feature_rules_three_packs_a_refused_top_rule_leaves_the_middle_rule_effective() {
    let stack = stack(vec![
        pack(1, "feature_rules/x.json", FEATURE_RULE_REFUSED),
        pack(2, "feature_rules/m.json", FEATURE_RULE),
        pack(3, "feature_rules/l.json", FEATURE_RULE),
    ]);
    let rules = stack.feature_rules();
    assert_eq!(
        source(rules.get(PlacementPass::Surface, &Id::parse("demo:r"))),
        Some(from_pack(1, "feature_rules/m.json"))
    );
    assert_eq!(
        refused(rules.rejected()),
        [(Some(0), "feature_rules/x.json".to_owned())]
    );
    assert_eq!(
        errors_at_top(rules.diagnostics(), "feature_rules/x.json"),
        ["core/missing_required_field"]
    );
}

// ---- features, a schema failure -----------------------------------------------------------
// If the identifier is already present, the file is dropped silently, so the first
// (highest-priority) pack wins per identifier. A failure leaves the feature not registered;
// references to it then log "No definition found for feature '%s'". The identifier enters the
// map before the schema is checked.
// Present behaviour: taken (`ClaimBeforeValidate`): the failed top definition keeps its
// identifier and no lower definition is effective.

/// A valid `demo:vein`.
const FEATURE: &str = r#"{"format_version": "1.13.0", "minecraft:ore_feature": {"description": {"identifier": "demo:vein"}, "count": 2}}"#;
/// `demo:vein` refused after its identifier is read: `count` is below its range.
const FEATURE_REFUSED: &str = r#"{"format_version": "1.13.0", "minecraft:ore_feature": {"description": {"identifier": "demo:vein"}, "count": 0}}"#;
/// A file that is not JSON: its identifier is never read.
const FEATURE_UNPARSABLE: &str = r#"{"format_version": "1.13.0", "minecraft:ore_feature": {"description": {"identifier": "demo:vein"}, "count": "#;

#[test]
fn features_two_packs_a_schema_refused_top_feature_keeps_its_identifier_taken() {
    let stack = stack(vec![
        pack(1, "features/x.json", FEATURE_REFUSED),
        pack(2, "features/l.json", FEATURE),
    ]);
    let features = stack.features();
    assert_eq!(source(features.get(&Id::parse("demo:vein"))), None);
    assert_eq!(
        refused(features.rejected()),
        [(Some(0), "features/x.json".to_owned())]
    );
    assert_eq!(
        errors_at_top(features.diagnostics(), "features/x.json"),
        ["core/outside_range"]
    );
}

#[test]
fn features_three_packs_a_schema_refused_top_feature_keeps_its_identifier_taken_from_both_lower_packs()
 {
    let stack = stack(vec![
        pack(1, "features/x.json", FEATURE_REFUSED),
        pack(2, "features/m.json", FEATURE),
        pack(3, "features/l.json", FEATURE),
    ]);
    let features = stack.features();
    assert_eq!(source(features.get(&Id::parse("demo:vein"))), None);
    assert_eq!(
        refused(features.rejected()),
        [(Some(0), "features/x.json".to_owned())]
    );
    assert_eq!(
        errors_at_top(features.diagnostics(), "features/x.json"),
        ["core/outside_range"]
    );
}

// ---- features, unparsable JSON ------------------------------------------------------------
// Invalid JSON logs "[Feature Registry] Passed in contents could not be evaluated as valid JSON
// in the following file: {}" and skips the file, before its identifier is looked up.
// Present behaviour: fallback. Unlike a schema failure, a file that is not JSON claims nothing.

#[test]
fn features_two_packs_an_unparsable_top_feature_leaves_the_lower_feature_effective() {
    let stack = stack(vec![
        pack(1, "features/x.json", FEATURE_UNPARSABLE),
        pack(2, "features/l.json", FEATURE),
    ]);
    let features = stack.features();
    assert_eq!(
        source(features.get(&Id::parse("demo:vein"))),
        Some(from_pack(1, "features/l.json"))
    );
    assert_eq!(
        refused(features.rejected()),
        [(Some(0), "features/x.json".to_owned())]
    );
    assert_eq!(
        errors_at_top(features.diagnostics(), "features/x.json"),
        ["feature/invalid_json"]
    );
}

#[test]
fn features_three_packs_an_unparsable_top_feature_leaves_the_middle_feature_effective() {
    let stack = stack(vec![
        pack(1, "features/x.json", FEATURE_UNPARSABLE),
        pack(2, "features/m.json", FEATURE),
        pack(3, "features/l.json", FEATURE),
    ]);
    let features = stack.features();
    assert_eq!(
        source(features.get(&Id::parse("demo:vein"))),
        Some(from_pack(1, "features/m.json"))
    );
    assert_eq!(
        refused(features.rejected()),
        [(Some(0), "features/x.json".to_owned())]
    );
    assert_eq!(
        errors_at_top(features.diagnostics(), "features/x.json"),
        ["feature/invalid_json"]
    );
}

// ---- biomes -------------------------------------------------------------------------------
// A file that fails anywhere (world gate, parse, upgrade, glue validation) is skipped and nothing
// else is affected; the first document with an identifier wins, a later one is discarded.
// Present behaviour: fallback. The highest valid definition is effective; in the three-pack
// form the low one is the discarded duplicate.

/// A valid `test:biome`.
const BIOME: &str = r#"{"format_version": "1.26.30", "minecraft:biome": {"description": {"identifier": "test:biome"}, "components": {"minecraft:climate": {"temperature": 1.0}}}}"#;
/// `test:biome` refused after its identifier is read: `components` is not an object.
const BIOME_REFUSED: &str = r#"{"format_version": "1.26.30", "minecraft:biome": {"description": {"identifier": "test:biome"}, "components": []}}"#;

#[test]
fn biomes_two_packs_a_refused_top_biome_leaves_the_lower_biome_effective() {
    let stack = stack(vec![
        pack(1, "biomes/x.json", BIOME_REFUSED),
        pack(2, "biomes/l.json", BIOME),
    ]);
    let biomes = stack.biomes();
    assert_eq!(
        source(biomes.get(&Id::parse("test:biome"))),
        Some(from_pack(1, "biomes/l.json"))
    );
    assert_eq!(
        refused(biomes.rejected()),
        [(Some(0), "biomes/x.json".to_owned())]
    );
    assert_eq!(
        errors_at_top(biomes.diagnostics(), "biomes/x.json"),
        ["core/invalid_type"]
    );
}

#[test]
fn biomes_three_packs_a_refused_top_biome_leaves_the_middle_biome_effective() {
    let stack = stack(vec![
        pack(1, "biomes/x.json", BIOME_REFUSED),
        pack(2, "biomes/m.json", BIOME),
        pack(3, "biomes/l.json", BIOME),
    ]);
    let biomes = stack.biomes();
    assert_eq!(
        source(biomes.get(&Id::parse("test:biome"))),
        Some(from_pack(1, "biomes/m.json"))
    );
    assert_eq!(
        refused(biomes.rejected()),
        [(Some(0), "biomes/x.json".to_owned())]
    );
    assert_eq!(
        errors_at_top(biomes.diagnostics(), "biomes/x.json"),
        ["core/invalid_type"]
    );
}

// ---- recipes ------------------------------------------------------------------------------
// A file that fails to load does not reserve its identifier across packs, so a lower pack's
// recipe with the same identifier still loads.
// Present behaviour: fallback (`TopWinsFallback`, silent). The other case, a recipe
// without tags that loads and reserves its identifier ("No tags specified"), is not a
// refusal and is pinned by `recipe::stack::tags_missing_override_reserves_the_identifier`.

/// A valid shaped `x:y`.
const RECIPE: &str = r##"{"format_version": "1.12", "minecraft:recipe_shaped": {"description": {"identifier": "x:y"}, "tags": ["crafting_table"], "pattern": ["#"], "key": {"#": {"item": "minecraft:stick"}}, "result": {"item": "minecraft:torch"}}}"##;
/// `x:y` refused after its identifier is read: the required `pattern` is missing.
const RECIPE_REFUSED: &str = r##"{"format_version": "1.12", "minecraft:recipe_shaped": {"description": {"identifier": "x:y"}, "tags": ["crafting_table"], "key": {"#": {"item": "minecraft:stick"}}, "result": {"item": "minecraft:torch"}}}"##;

#[test]
fn recipes_two_packs_a_refused_top_recipe_leaves_the_lower_recipe_effective() {
    let stack = stack(vec![
        pack(1, "recipes/x.json", RECIPE_REFUSED),
        pack(2, "recipes/l.json", RECIPE),
    ]);
    let recipes = stack.recipes();
    assert_eq!(
        source(recipes.get(RecipeNamespace::Crafting, &Id::parse("x:y"))),
        Some(from_pack(1, "recipes/l.json"))
    );
    assert_eq!(
        refused(recipes.rejected()),
        [(Some(0), "recipes/x.json".to_owned())]
    );
    assert_eq!(
        errors_at_top(recipes.diagnostics(), "recipes/x.json"),
        ["core/missing_required_field"]
    );
}

#[test]
fn recipes_three_packs_a_refused_top_recipe_leaves_the_middle_recipe_effective() {
    let stack = stack(vec![
        pack(1, "recipes/x.json", RECIPE_REFUSED),
        pack(2, "recipes/m.json", RECIPE),
        pack(3, "recipes/l.json", RECIPE),
    ]);
    let recipes = stack.recipes();
    assert_eq!(
        source(recipes.get(RecipeNamespace::Crafting, &Id::parse("x:y"))),
        Some(from_pack(1, "recipes/m.json"))
    );
    assert_eq!(
        refused(recipes.rejected()),
        [(Some(0), "recipes/x.json".to_owned())]
    );
    assert_eq!(
        errors_at_top(recipes.diagnostics(), "recipes/x.json"),
        ["core/missing_required_field"]
    );
}

// ---- loot tables --------------------------------------------------------------------------
// The path is loaded from the pack stack. If the parse fails or the root is not an object the
// game logs "Loot table for %s is invalid." and caches nothing.
// Present behaviour: taken. The path belongs to the top pack's copy; when that copy is refused
// the path has no table and no lower copy stands in. Unparsable JSON and a root that is not an
// object are one refusal (`MalformedInput`); the family has no other.

/// The path every pack defines.
const LOOT_PATH: &str = "loot_tables/chests/x.json";
/// A valid table.
const LOOT: &str = r#"{"pools": [{"rolls": 1, "entries": [{"type": "empty"}]}]}"#;
/// A refused table: JSON whose root is not an object.
const LOOT_REFUSED: &str = r#"[{"pools": []}]"#;

#[test]
fn loot_tables_two_packs_a_refused_top_copy_leaves_the_path_without_a_table() {
    let stack = stack(vec![
        pack(1, LOOT_PATH, LOOT_REFUSED),
        pack(2, LOOT_PATH, LOOT),
    ]);
    let tables = stack.loot_tables();
    let path = PackPath::new(LOOT_PATH).expect("a pack path");
    assert_eq!(source(tables.get(&path)), None);
    assert_eq!(tables.iter().count(), 0);
    assert_eq!(
        refused(tables.rejected()),
        [(Some(0), LOOT_PATH.to_owned())]
    );
    assert_eq!(
        errors_at_top(tables.diagnostics(), LOOT_PATH),
        ["loot/invalid_table"]
    );
}

#[test]
fn loot_tables_three_packs_a_refused_top_copy_leaves_the_path_without_a_table() {
    let stack = stack(vec![
        pack(1, LOOT_PATH, LOOT_REFUSED),
        pack(2, LOOT_PATH, LOOT),
        pack(3, LOOT_PATH, LOOT),
    ]);
    let tables = stack.loot_tables();
    let path = PackPath::new(LOOT_PATH).expect("a pack path");
    assert_eq!(source(tables.get(&path)), None);
    assert_eq!(tables.iter().count(), 0);
    assert_eq!(
        refused(tables.rejected()),
        [(Some(0), LOOT_PATH.to_owned())]
    );
    assert_eq!(
        errors_at_top(tables.diagnostics(), LOOT_PATH),
        ["loot/invalid_table"]
    );
}
