#![allow(clippy::pedantic, missing_docs)]
//! Of a member written twice the last value stands, whole, at every level:
//! every family on the one rule, its own reader included. One test per family with
//! a reader of its own: a member doubled at the root, and one doubled inside. `FIRST` marks
//! the copy that must vanish, `LAST` the one that stands.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use bedrock_addon::document::{Content, Document, Rejected};
use bedrock_addon::load::LoadContext;
use bedrock_addon::storage::PackPath;

/// The `Debug` of what `text` loads to (the rejection's, when it is rejected).
#[allow(dead_code, reason = "used by the tests of the compiled families")]
fn loaded<T: Content + std::fmt::Debug>(path: &'static str, text: &str) -> String {
    let outcome: Result<Document<T>, Rejected> = T::parse(
        &PackPath::from_static(path),
        text.as_bytes(),
        &LoadContext::detached(),
    );
    match outcome {
        Ok(document) => format!("{:?}", document.value()),
        Err(rejected) => panic!("{text}\n{:#?}", rejected.diagnostics),
    }
}

/// Each text loads with its `last` marker and without its `first` one.
#[track_caller]
#[allow(dead_code, reason = "used by the tests of the compiled families")]
fn last_stands<T: Content + std::fmt::Debug>(path: &'static str, texts: [&str; 2]) {
    for text in texts {
        let value = loaded::<T>(path, text);
        assert!(value.contains("last"), "{text}\n{value}");
        assert!(!value.contains("first"), "{text}\n{value}");
    }
}

#[cfg(feature = "loot")]
#[test]
fn loot_keeps_the_last_copy() {
    let pool = |item: &str| {
        format!(r#"{{"rolls": 1, "entries": [{{"type": "item", "name": "minecraft:{item}"}}]}}"#)
    };
    let root = format!(
        r#"{{"pools": [{}], "pools": [{}]}}"#,
        pool("first"),
        pool("last")
    );
    let nested = r#"{"pools": [{"rolls": 1, "entries": [{"type": "item", "name": "minecraft:first", "name": "minecraft:last"}]}]}"#;
    last_stands::<bedrock_addon::families::loot::LootTable>("loot_tables/x.json", [&root, nested]);
}

#[cfg(feature = "recipe")]
#[test]
fn recipe_keeps_the_last_copy() {
    let payload = |id: &str| {
        format!(
            r#"{{"description": {{"identifier": "demo:{id}"}}, "tags": ["crafting_table"], "ingredients": ["minecraft:stone"], "result": "minecraft:stone", "{id}_member": 1}}"#
        )
    };
    let root = format!(
        r#"{{"format_version": "1.12.0", "minecraft:recipe_shapeless": {}, "minecraft:recipe_shapeless": {}}}"#,
        payload("first"),
        payload("last")
    );
    let nested = r#"{"format_version": "1.12.0", "minecraft:recipe_shapeless": {"description": {"identifier": "demo:first", "identifier": "demo:last"}, "tags": ["crafting_table"], "ingredients": ["minecraft:stone"], "result": "minecraft:stone"}}"#;
    last_stands::<bedrock_addon::families::recipe::Recipe>("recipes/x.json", [&root, nested]);
}

#[cfg(feature = "dialogue")]
#[test]
fn dialogue_keeps_the_last_copy() {
    let scenes = |tag: &str| format!(r#"{{"scenes": [{{"scene_tag": "{tag}", "text": "Hi."}}]}}"#);
    let root = format!(
        r#"{{"format_version": "1.17.0", "minecraft:npc_dialogue": {}, "minecraft:npc_dialogue": {}}}"#,
        scenes("first"),
        scenes("last")
    );
    let nested = r#"{"format_version": "1.17.0", "minecraft:npc_dialogue": {"scenes": [{"scene_tag": "first", "scene_tag": "last", "text": "Hi."}]}}"#;
    last_stands::<bedrock_addon::families::dialogue::DialogueFile>(
        "dialogue/x.json",
        [&root, nested],
    );
}

#[cfg(feature = "behavior_tree")]
#[test]
fn behavior_tree_keeps_the_last_copy() {
    let tree = |name: &str| {
        format!(
            r#"{{"minecraft:name": "{name}", "{name}_member": 1, "minecraft:root": {{"minecraft:wait_ticks": {{"num_of_ticks": 1}}}}}}"#
        )
    };
    let root = format!(
        r#"{{"minecraft:behavior_tree": {}, "minecraft:behavior_tree": {}}}"#,
        tree("first"),
        tree("last")
    );
    let nested = r#"{"minecraft:behavior_tree": {"minecraft:name": "first", "minecraft:name": "last", "minecraft:root": {"minecraft:wait_ticks": {"num_of_ticks": 1}}}}"#;
    // The tree keeps its file's text as written (`raw`, for its own write-back), so the
    // name read is what is compared.
    for text in [root.as_str(), nested] {
        let value = loaded::<bedrock_addon::families::behavior_tree::BehaviorTree>(
            "behavior_trees/x.json",
            text,
        );
        assert!(
            value.contains(r#"name: Id<behavior tree>("last")"#),
            "{value}"
        );
        // Whole: no member of the first copy survives in the kept tree.
        assert!(!value.contains(r#"Extra({"first"#), "{value}");
    }
}

#[cfg(feature = "function")]
#[test]
fn the_tick_list_keeps_the_last_copy() {
    // A tick list's members are strings: the root is its one object level.
    let root = r#"{"values": ["first"], "values": ["last"]}"#;
    let comment = r#"{"values": ["first"], /* again */ "values": ["last"]}"#;
    last_stands::<bedrock_addon::families::function::TickList>(
        "functions/tick.json",
        [root, comment],
    );
}

#[cfg(feature = "crafting_catalog")]
#[test]
fn the_catalogue_keeps_the_last_copy() {
    let catalog = |name: &str| {
        format!(
            r#"{{"categories": [{{"category_name": "construction", "groups": [{{"group_identifier": {{"name": "demo:{name}"}}, "items": []}}]}}]}}"#
        )
    };
    let root = format!(
        r#"{{"format_version": "1.21.60", "minecraft:crafting_items_catalog": {}, "minecraft:crafting_items_catalog": {}}}"#,
        catalog("first"),
        catalog("last")
    );
    let nested = r#"{"format_version": "1.21.60", "minecraft:crafting_items_catalog": {"categories": [{"category_name": "construction", "groups": [{"group_identifier": {"name": "demo:first", "name": "demo:last"}, "items": []}]}]}}"#;
    last_stands::<bedrock_addon::families::crafting_catalog::CraftingCatalog>(
        "item_catalog/crafting_item_catalog.json",
        [&root, nested],
    );
}

#[test]
fn the_manifest_keeps_the_last_copy() {
    use bedrock_addon::manifest::{CapabilityRegistry, Manifest};
    let header = |name: &str| {
        format!(
            r#"{{"name": "{name}", "uuid": "00000000-0000-0000-0000-000000000001", "version": [1, 0, 0], "min_engine_version": [1, 21, 0]}}"#
        )
    };
    let modules = r#"[{"type": "data", "uuid": "00000000-0000-0000-0000-000000000002", "version": [1, 0, 0]}]"#;
    let root = format!(
        r#"{{"format_version": 2, "header": {}, "header": {}, "modules": {modules}}}"#,
        header("first"),
        header("last")
    );
    let nested = format!(
        r#"{{"format_version": 2, "header": {{"name": "first", "name": "last", "uuid": "00000000-0000-0000-0000-000000000001", "version": [1, 0, 0], "min_engine_version": [1, 21, 0]}}, "modules": {modules}}}"#
    );
    for text in [root, nested] {
        let manifest = Manifest::parse(
            &PackPath::from_static("manifest.json"),
            text.as_bytes(),
            &CapabilityRegistry::default(),
        )
        .unwrap_or_else(|r| panic!("{text}\n{:#?}", r.diagnostics));
        let value = format!("{:?}", manifest.value());
        assert!(
            value.contains("last") && !value.contains("first"),
            "{value}"
        );
    }
}

// Text's two JSON files. Vanilla iterates an array root, or the
// member values of an object root (a warning of ours); in an object root a key written twice
// is the one level where a duplicate can occur, and its last value stands.
#[cfg(feature = "languages")]
#[test]
fn the_language_lists_keep_the_last_copy() {
    last_stands::<bedrock_addon::families::languages::Languages>(
        "texts/languages.json",
        [
            r#"{"a": "first_US", "a": "last_US"}"#,
            r#"{"a": "first_US", "b": "x_US", "a": "last_US"}"#,
        ],
    );
    last_stands::<bedrock_addon::families::languages::LanguageNames>(
        "texts/language_names.json",
        [
            r#"{"a": ["en_US", "first"], "a": ["en_US", "last"]}"#,
            r#"{"b": ["de_DE", "x"], "a": ["en_US", "first"], "a": ["en_US", "last"]}"#,
        ],
    );
}

/// The deferred settings read their payloads with a reader of their own, over core's text:
/// inside the payload the last copy stands. At the root the game counts the members as
/// written, so a root member written twice is a third member.
#[cfg(feature = "deferred_settings")]
#[test]
fn the_deferred_settings_keep_the_last_copy_inside_the_payload() {
    use bedrock_addon::families::deferred_settings::CubemapSettings;
    last_stands::<CubemapSettings>(
        "cubemaps/x.json",
        [
            r#"{"format_version": "1.21.130", "minecraft:cubemap_settings": {"description": {"identifier": "t:first"}, "description": {"identifier": "t:last"}}}"#,
            r#"{"format_version": "1.21.130", "minecraft:cubemap_settings": {"description": {"identifier": "t:first", "identifier": "t:last"}}}"#,
        ],
    );
    let doubled_root = r#"{"format_version": "1.21.130", "minecraft:cubemap_settings": {"description": {"identifier": "t:first"}}, "minecraft:cubemap_settings": {"description": {"identifier": "t:last"}}}"#;
    let rejected = CubemapSettings::parse(
        &PackPath::from_static("cubemaps/x.json"),
        doubled_root.as_bytes(),
        &LoadContext::detached(),
    )
    .unwrap_err();
    let codes: Vec<&str> = rejected.diagnostics.iter().map(|f| f.code.name()).collect();
    assert_eq!(codes, ["MalformedInput"]);
}

#[cfg(feature = "geometry")]
#[test]
fn geometry_keeps_the_last_copy() {
    last_stands::<bedrock_addon::families::geometry::GeometryFile>(
        "models/entity/x.json",
        [
            r#"{"format_version": "1.21.0", "minecraft:geometry": [{"description": {"identifier": "geometry.first"}}], "minecraft:geometry": [{"description": {"identifier": "geometry.last"}}]}"#,
            r#"{"format_version": "1.21.0", "minecraft:geometry": [{"description": {"identifier": "geometry.x"}, "bones": [{"name": "first", "name": "last"}]}]}"#,
        ],
    );
}
