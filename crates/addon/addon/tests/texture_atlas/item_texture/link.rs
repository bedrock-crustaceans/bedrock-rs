//! Fixture rows t14 to t21, t24, t26 and t27: a behaviour pack's `minecraft:icon` checked
//! against the merged item atlas (rules 9 to 12 and 21 to 24). The rules
//! need both stacks, so they run in the link pass (`StackSet::link`).

use bedrock_addon::diagnostic::{Diagnostics, Severity};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{PackOrigin, PackRepository, PackSelection, StackSet, WorldPacks};

use crate::common::{fixture_file, pack_with};

const ITEM: &str = "items/thing.json";
const ATLAS: &str = "textures/item_texture.json";

/// The vanilla item atlas (fixture t01).
fn vanilla_atlas() -> Vec<u8> {
    fixture_file(
        "item_texture",
        "t01_vanilla_shape",
        "p0/textures/item_texture.json",
    )
}

/// The atlas of fixture t02: vanilla's with `demo_gem` on top is what a stack of both gives;
/// here the one key is enough.
const DEMO_GEM: &[u8] =
    br#"{ "texture_name": "atlas.items", "texture_data": { "demo_gem": { "textures": "textures/items/demo_gem" } } }"#;
const UPPER_GEM: &[u8] =
    br#"{ "texture_name": "atlas.items", "texture_data": { "Demo_Gem": { "textures": "textures/items/demo_gem" } } }"#;

/// A `minecraft:item` document `demo:thing` of `format_version` with `components`.
fn item(format_version: &str, components: &str) -> Vec<u8> {
    format!(
        r#"{{ "format_version": "{format_version}", "minecraft:item": {{
            "description": {{ "identifier": "demo:thing", "menu_category": {{ "category": "items" }} }},
            "components": {{ {components} }} }} }}"#
    )
    .into_bytes()
}

/// The stacks of one behaviour pack holding `item` and one resource pack holding `atlas`.
fn stacks(item: &[u8], atlas: &[u8]) -> StackSet {
    let bp = pack_with(1, "data", &[], &[(ITEM, item)], PackOrigin::World);
    let rp = pack_with(2, "resources", &[], &[(ATLAS, atlas)], PackOrigin::World);
    let selection = |pack: &bedrock_addon::pack::Pack| PackSelection {
        pack_id: pack.id().uuid,
        version: pack.id().version.clone(),
        subpack: None,
    };
    let world = WorldPacks {
        behavior: vec![selection(&bp)],
        resource: vec![selection(&rp)],
    };
    let mut repository = PackRepository::new();
    repository.add_pack(bp, PackOrigin::World);
    repository.add_pack(rp, PackOrigin::Package); // the pack with the sources is one the game ships
    let set = repository.stacks(&world, LoadOptions::default());
    assert_eq!(set.behavior.items().iter().count(), 1, "the item loads");
    set
}

/// The findings at the item file above debug, as code, severity and text.
fn found(d: &Diagnostics) -> Vec<(&'static str, Severity, String)> {
    d.iter()
        .filter(|f| {
            f.severity > Severity::Debug
                && f.location.path.as_ref().is_some_and(|p| p.as_str() == ITEM)
        })
        .map(|f| (f.code.name(), f.severity, f.to_string()))
        .collect()
}

/// The one rule-9 kept error for `name`, with `hint` appended to vanilla's text (a level-3
/// line after which the item loads).
fn missing(d: &Diagnostics, name: &str, hint: &str) {
    let found = found(d);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(
        (found[0].0, found[0].1),
        ("UnresolvedReference", Severity::Error)
    );
    assert!(d.iter().any(|f| f.kept), "{d:#?}");
    let text = format!("-> components -> minecraft:icon:  Missing referenced asset {name}{hint}");
    assert!(found[0].2.contains(&text), "{}", found[0].2);
}

#[test]
fn t14_an_icon_that_names_an_atlas_key_is_no_finding() {
    let set = stacks(
        &item(
            "1.21.80",
            r#""minecraft:icon": { "textures": { "default": "demo_gem" } }"#,
        ),
        DEMO_GEM,
    );
    assert_eq!(found(&set.link()), []);
}

#[test]
fn t15_an_icon_that_names_no_atlas_key_is_an_unresolved_reference_deviation() {
    // `vanilla = "Error; client-hosted worlds only, once, on the validation packet;
    // lowercased union; 1.20.60+ shorthand not lowercased"`.
    let set = stacks(
        &item("1.21.80", r#""minecraft:icon": "test_icon""#),
        &vanilla_atlas(),
    );
    missing(&set.link(), "test_icon", "");
}

#[test]
fn t16_an_icon_that_misses_by_case_gets_the_hint_deviation() {
    // Vanilla's validator compares lower-cased and would pass it; the client's lookup is exact.
    let set = stacks(
        &item(
            "1.21.80",
            r#""minecraft:icon": { "textures": { "default": "demo_gem" } }"#,
        ),
        UPPER_GEM,
    );
    missing(
        &set.link(),
        "demo_gem",
        " (an atlas key differs only in case: 'Demo_Gem')",
    );
}

#[test]
fn t17_the_1_20_50_texture_member_is_checked() {
    let set = stacks(
        &item("1.20.50", r#""minecraft:icon": { "texture": "apple" }"#),
        &vanilla_atlas(),
    );
    assert_eq!(found(&set.link()), []);
}

#[test]
fn t18_the_bundle_keys_are_checked_without_a_version_finding() {
    let set = stacks(
        &item(
            "1.20.60",
            r#""minecraft:icon": { "textures": { "default": "bundle",
                "bundle_open_front": "bundle_open_front", "bundle_open_back": "missing_key" } }"#,
        ),
        &vanilla_atlas(),
    );
    missing(&set.link(), "missing_key", "");
}

#[test]
fn t19_textures_without_default_is_an_error() {
    let set = stacks(
        &item(
            "1.21.80",
            r#""minecraft:icon": { "textures": { "dyed": "apple" } }"#,
        ),
        &vanilla_atlas(),
    );
    let found = found(&set.link());
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(found[0].1, Severity::Error);
    assert!(found[0].2.contains(
        "demo:thing : minecraft:icon component should include default key in textures map to set icon texture"
    ));
}

#[test]
fn t20_a_key_the_game_does_not_read_is_an_info() {
    // `trim_palette`: its prefix `trim` is no trim material.
    let set = stacks(
        &item(
            "1.21.80",
            r#""minecraft:icon": { "textures": { "default": "apple", "trim_palette": "apple" } }"#,
        ),
        &vanilla_atlas(),
    );
    let found = found(&set.link());
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(found[0].1, Severity::Info);
}

#[test]
fn t21_a_reference_to_a_dropped_tile_is_unresolved() {
    let set = stacks(
        &item(
            "1.21.80",
            r#""minecraft:icon": { "textures": { "default": "a" } }"#,
        ),
        br#"{ "texture_name": "atlas.items", "texture_data": { "a": { "quad": true } } }"#,
    );
    // The atlas reports the dropped tile at load.
    let atlas: Vec<_> = set
        .resource
        .texture_atlases()
        .item_atlas
        .diagnostics()
        .collect();
    assert_eq!(atlas[0].code.name(), "MissingRequiredField");
    assert_eq!(atlas[0].code.to_string(), "Missing textures \"a\".");
    missing(&set.link(), "a", "");
}

#[test]
fn t24_an_upper_case_shorthand_matches_exactly_deviation() {
    // Vanilla's validator would report it: the key is stored lower-cased and the 1.20.60+
    // shorthand is not.
    let set = stacks(
        &item("1.21.80", r#""minecraft:icon": "Demo_Gem""#),
        UPPER_GEM,
    );
    assert_eq!(found(&set.link()), []);
}

#[test]
fn t26_a_palette_value_is_not_resolved_against_the_atlas_deviation() {
    // `vanilla = "recorded as an ItemTexture reference; reported by the client-hosted
    // validator when not an atlas key"`.
    let set = stacks(
        &item(
            "1.21.80",
            r#""minecraft:tags": { "tags": ["minecraft:trimmable_armors"] },
               "minecraft:wearable": { "slot": "slot.armor.chest" },
               "minecraft:icon": { "textures": { "default": "apple", "icon_trim": "apple",
                   "iron_palette": "iron_darker" } }"#,
        ),
        &vanilla_atlas(),
    );
    assert_eq!(found(&set.link()), []);
}

#[test]
fn t27_custom_armor_without_wearable_is_a_warning() {
    let set = stacks(
        &item(
            "1.21.80",
            r#""minecraft:tags": { "tags": ["minecraft:trimmable_armors"] },
               "minecraft:icon": { "textures": { "default": "apple" } }"#,
        ),
        &vanilla_atlas(),
    );
    let found = found(&set.link());
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(found[0].1, Severity::Warning);
    assert!(
        found[0]
            .2
            .contains("Custom armor item 'demo:thing' needs a wearable component")
    );
}
