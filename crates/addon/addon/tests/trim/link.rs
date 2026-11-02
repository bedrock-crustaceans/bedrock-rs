//! Link pass, `lk01`–`lk06`: the rules T1–T4 and T7, which run in the link pass over the loot and item documents against `TrimData`. Written
//! with their assertions, ignored until `StackSet::link` is implemented.

use bedrock_addon::diagnostic::{Diagnostics, Severity};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{PackOrigin, PackRepository, PackSelection, StackSet, WorldPacks};
use bedrock_addon::version::{BaseGameVersion, SemVersion};

use super::pack_common::{Spec, bp};

const LOOT: &str = "loot_tables/equipment/trim.json";

/// The stacks of one behaviour pack built from `spec`, for a world of `options`.
fn stacks(spec: &Spec, options: LoadOptions) -> StackSet {
    let id = spec.id();
    let mut repository = PackRepository::new();
    repository.add_pack(spec.open(), PackOrigin::World);
    let world = WorldPacks {
        behavior: vec![PackSelection {
            pack_id: id.uuid,
            version: id.version,
            subpack: None,
        }],
        resource: Vec::new(),
    };
    repository.stacks(&world, options)
}

/// A pack whose one loot table is `table`; the table loads.
fn loot_stacks(table: &'static [u8], options: LoadOptions) -> StackSet {
    let set = stacks(&bp(1).file(LOOT, table), options);
    assert_eq!(
        set.behavior.loot_tables().iter().count(),
        1,
        "the loot table loads"
    );
    set
}

/// The findings of code `name` at the file `path`, as `(severity, JSON path)`.
fn found(d: &Diagnostics, name: &str, path: &str) -> Vec<(Severity, String)> {
    d.iter()
        .filter(|f| {
            f.code.name() == name && f.location.path.as_ref().is_some_and(|p| p.as_str() == path)
        })
        .map(|f| (f.severity, f.location.json_path.to_string()))
        .collect()
}

/// The findings of the trim rules, whatever their code: every finding at `path` above debug.
fn all_at(d: &Diagnostics, path: &str) -> Vec<(&'static str, Severity, String)> {
    d.iter()
        .filter(|f| {
            f.severity > Severity::Debug
                && f.location.path.as_ref().is_some_and(|p| p.as_str() == path)
        })
        .map(|f| (f.code.name(), f.severity, f.location.json_path.to_string()))
        .collect()
}

/// The shape of vanilla's `loot_tables/equipment/trial_chamber_diamond.json`: `copper` with
/// `flow` and with `bolt`.
const VANILLA_SHAPE: &[u8] = br#"{"pools": [{"rolls": 1, "entries": [
    {"type": "item", "name": "minecraft:diamond_chestplate", "functions": [
        {"function": "set_armor_trim", "material": "copper", "pattern": "flow"}]},
    {"type": "item", "name": "minecraft:diamond_helmet", "functions": [
        {"function": "set_armor_trim", "material": "copper", "pattern": "bolt"}]}]}]}"#;

#[test]
fn lk01_an_unknown_pattern_is_an_unresolved_reference_deviation() {
    // `vanilla = "pattern: debug assertion, no trim"` (a deliberate deviation, V8).
    let set = loot_stacks(
        br#"{"pools": [{"rolls": 1, "entries": [
            {"type": "item", "name": "minecraft:diamond_chestplate", "functions": [
                {"function": "set_armor_trim", "material": "copper", "pattern": "spiral"}]}]}]}"#,
        LoadOptions::default(),
    );
    let d = set.link();
    // An information of ours: the game reads the name without a line.
    assert_eq!(
        found(&d, "UnresolvedReference", LOOT),
        [(
            Severity::Info,
            "pools/0/entries/0/functions/0/pattern".to_owned()
        )],
        "{d:#?}"
    );
}

#[test]
fn lk02_an_unknown_material_is_an_unresolved_reference_deviation() {
    // `vanilla = "material: silent, no trim"` (a deliberate deviation, V9).
    let set = loot_stacks(
        br#"{"pools": [{"rolls": 1, "entries": [
            {"type": "item", "name": "minecraft:diamond_chestplate", "functions": [
                {"function": "set_armor_trim", "material": "obsidian", "pattern": "flow"}]}]}]}"#,
        LoadOptions::default(),
    );
    let d = set.link();
    assert_eq!(
        found(&d, "UnresolvedReference", LOOT),
        [(
            Severity::Info,
            "pools/0/entries/0/functions/0/material".to_owned()
        )],
        "{d:#?}"
    );
}

#[test]
fn lk03_a_function_without_pattern_and_material_misses_both() {
    let set = loot_stacks(
        br#"{"pools": [{"rolls": 1, "entries": [
            {"type": "item", "name": "minecraft:diamond_chestplate", "functions": [
                {"function": "set_armor_trim"}]}]}]}"#,
        LoadOptions::default(),
    );
    let d = set.link();
    let mut missing = found(&d, "MissingRequiredField", LOOT);
    missing.sort();
    assert_eq!(
        missing,
        [
            (
                Severity::Info,
                "pools/0/entries/0/functions/0/material".to_owned()
            ),
            (
                Severity::Info,
                "pools/0/entries/0/functions/0/pattern".to_owned()
            ),
        ],
        "{d:#?}"
    );
    // An absent name is not also an unknown one.
    assert_eq!(found(&d, "UnresolvedReference", LOOT), [], "{d:#?}");
}

#[test]
fn lk04_the_vanilla_shape_has_no_finding() {
    let set = loot_stacks(VANILLA_SHAPE, LoadOptions::default());
    let d = set.link();
    assert_eq!(all_at(&d, LOOT), [], "{d:#?}");
}

#[test]
fn lk05_a_custom_template_item_has_no_effect_deviation() {
    // `vanilla = "accepted silently; never a template"` (a deliberate deviation).
    const ITEM: &str = "items/spiral_template.json";
    let spec = bp(1).file(
        ITEM,
        br#"{"format_version": "1.26.30", "minecraft:item": {"description": {"identifier": "demo:spiral_armor_trim_smithing_template"}, "components": {"minecraft:tags": {"tags": ["minecraft:trim_templates"]}}}}"#,
    );
    let set = stacks(&spec, LoadOptions::default());
    assert_eq!(set.behavior.items().len(), 1, "the item loads");
    let d = set.link();
    assert_eq!(
        found(&d, "NoEffect", ITEM),
        [(
            Severity::Info,
            "minecraft:item/components/minecraft:tags".to_owned()
        )],
        "{d:#?}"
    );
}

#[test]
fn lk06_trim_content_in_an_old_world_has_no_effect() {
    let options = LoadOptions {
        base_game_version: BaseGameVersion::new(SemVersion::new(1, 19, 0)),
        ..LoadOptions::default()
    };
    let set = loot_stacks(VANILLA_SHAPE, options);
    let d = set.link();
    // Trims are disabled for the world (G1): one info per function, and the names are not
    // looked up in registries that are empty.
    assert_eq!(
        all_at(&d, LOOT),
        [
            (
                "NoEffect",
                Severity::Info,
                "pools/0/entries/0/functions/0".to_owned()
            ),
            (
                "NoEffect",
                Severity::Info,
                "pools/0/entries/1/functions/0".to_owned()
            ),
        ],
        "{d:#?}"
    );
}
