//! 20 R32 (and I6): a behaviour pack item's `minecraft:icon` against the merged item atlas,
//! with the rules of the item_texture folder that travel with it: `textures` without
//! `default` (rule 10), a `textures` key the game never reads (rule 11), a tile the atlas
//! dropped (rule 12), trimmable armour without an icon, without `minecraft:wearable` or worn
//! in no armour slot (rules 21 to 23) and the palette values that are no atlas keys (rule 24).

use bedrock_addon::diagnostic::{
    Diagnostic, DiagnosticDetail, Diagnostics, Phase, ReferenceHint, Severity,
};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::LinkOptions;

// The material and atlas files of these worlds are a pack the game ships: the rows
// are tested at their own severity. `tests/link/view.rs` has the other views.
use crate::common::{
    behavior_pack, game_link as link, game_link_with as link_with, hosted, in_file, stacks_of,
};

const ITEM: &str = "items/thing.json";
const ATLAS: &str = "textures/item_texture.json";
const ICON: &str = "minecraft:item/components/minecraft:icon";

/// An item atlas with the tiles `gem` and `Ruby`.
const GEMS: &str = r#"{ "texture_name": "atlas.items", "texture_data": {
    "gem": { "textures": "textures/items/gem" },
    "Ruby": { "textures": "textures/items/ruby" } } }"#;

/// A `minecraft:item` document `demo:thing` of `format_version` with `components`.
fn item(format_version: &str, components: &str) -> String {
    format!(
        r#"{{ "format_version": "{format_version}", "minecraft:item": {{
            "description": {{ "identifier": "demo:thing" }},
            "components": {{ {components} }} }} }}"#
    )
}

/// The link findings at the item file of a world with one item of the newest format with
/// `components` and the atlas `atlas`.
fn linked(components: &str, atlas: &str) -> Vec<Diagnostic> {
    linked_at("1.21.80", components, atlas)
}

/// [`linked`] for an item of `format_version`.
fn linked_at(format_version: &str, components: &str, atlas: &str) -> Vec<Diagnostic> {
    let found = link(
        &[(ITEM, &item(format_version, components))],
        &[(ATLAS, atlas)],
    );
    at_item(&found)
}

/// The findings at the item file.
fn at_item(found: &Diagnostics) -> Vec<Diagnostic> {
    in_file(found, ITEM).into_iter().cloned().collect()
}

/// What a finding says: its code, severity and JSON path.
fn said(finding: &Diagnostic) -> (&'static str, Severity, String) {
    (
        finding.code.name(),
        finding.severity,
        finding.location.json_path.to_string(),
    )
}

/// The name and the hint of a reference finding.
fn named(finding: &Diagnostic) -> (&'static str, &str, Option<&ReferenceHint>) {
    match finding.detail.as_deref() {
        Some(DiagnosticDetail::Reference { kind, id, hint }) => (kind, id, hint.as_ref()),
        other => panic!("no reference: {other:?}"),
    }
}

#[test]
fn r32_an_icon_key_a_pack_defines_resolves() {
    for components in [
        r#""minecraft:icon": "gem""#,
        r#""minecraft:icon": { "textures": { "default": "gem" } }"#,
        r#""minecraft:icon": { "textures": { "default": "gem", "dyed": "Ruby" } }"#,
    ] {
        assert_eq!(linked(components, GEMS), [], "{components}");
    }
}

#[test]
fn r32_an_icon_key_nothing_defines_is_a_kept_error_at_the_icon() {
    let found = linked(r#""minecraft:icon": "nothing""#, GEMS);
    assert_eq!(found.len(), 1, "{found:#?}");
    let finding = &found[0];
    // The validator's line is logged at level 3 and nothing is dropped: a kept error.
    assert!(finding.kept);
    assert_eq!(
        said(finding),
        ("UnresolvedReference", Severity::Error, ICON.to_owned())
    );
    assert_eq!(finding.phase, Phase::Link);
    let pack = finding.location.pack.as_ref().expect("the pack");
    assert_eq!(pack.name(), "bp 1");
    assert_eq!(
        finding.location.path.as_ref().expect("the file").as_str(),
        ITEM
    );
    assert_eq!(named(finding), ("atlas key", "nothing", None));
    // The linked-asset validator's text, with the item's name for its breadcrumb.
    assert_eq!(
        finding.code.to_string(),
        "demo:thing -> components -> minecraft:icon:  Missing referenced asset nothing"
    );
}

#[test]
fn r32_a_key_other_than_default_is_told_at_its_member() {
    let found = linked(
        r#""minecraft:icon": { "textures": { "default": "gem", "dyed": "none", "icon_trim": "nope" } }"#,
        GEMS,
    );
    assert_eq!(
        found.iter().map(said).collect::<Vec<_>>(),
        [
            (
                "UnresolvedReference",
                Severity::Error,
                format!("{ICON}/textures/dyed")
            ),
            (
                "UnresolvedReference",
                Severity::Error,
                format!("{ICON}/textures/icon_trim")
            ),
        ]
    );
    assert_eq!(named(&found[0]).1, "none");
    assert_eq!(named(&found[1]).1, "nope");
}

// X9: the client's lookup is exact; vanilla's validator compares lower-cased names.
#[test]
fn r32_a_key_is_compared_as_written_and_a_miss_by_case_names_the_tile() {
    // Exact, on both sides of the case.
    assert_eq!(linked(r#""minecraft:icon": "Ruby""#, GEMS), []);
    assert_eq!(
        linked(
            r#""minecraft:icon": { "textures": { "default": "Ruby" } }"#,
            GEMS
        ),
        []
    );
    for (written, tile) in [("ruby", "Ruby"), ("RUBY", "Ruby"), ("Gem", "gem")] {
        let found = linked(&format!(r#""minecraft:icon": "{written}""#), GEMS);
        assert_eq!(found.len(), 1, "{written}: {found:#?}");
        assert_eq!(said(&found[0]).0, "UnresolvedReference");
        let hint = ReferenceHint::CaseMismatch(tile.to_owned());
        assert_eq!(named(&found[0]), ("atlas key", written, Some(&hint)));
        assert_eq!(
            found[0].code.to_string(),
            format!(
                "demo:thing -> components -> minecraft:icon:  Missing referenced asset {written} (an atlas key differs only in case: '{tile}')"
            )
        );
    }
}

#[test]
fn r32_a_key_only_the_game_has_resolves_when_the_host_says_so() {
    let world = |options: LoadOptions| {
        let found = link_with(
            &[(ITEM, &item("1.21.80", r#""minecraft:icon": "apple""#))],
            &[(ATLAS, GEMS)],
            options,
            &LinkOptions::default(),
        );
        at_item(&found).iter().map(said).collect::<Vec<_>>()
    };
    assert_eq!(world(hosted(&[("atlas key", "apple")])), []);
    // A host is believed when it says no.
    assert_eq!(
        world(hosted(&[("atlas key", "pear")])),
        [("UnresolvedReference", Severity::Error, ICON.to_owned())]
    );
    // Without a host the crate has no list of the game's tiles: the game's own resource pack
    // on the stack is what defines them.
    assert_eq!(
        world(LoadOptions::default()),
        [("UnresolvedReference", Severity::Error, ICON.to_owned())]
    );
}

#[test]
fn r32_the_1_20_50_texture_member_and_the_legacy_icon_are_checked() {
    assert_eq!(
        linked_at("1.20.50", r#""minecraft:icon": { "texture": "gem" }"#, GEMS),
        []
    );
    let found = linked_at(
        "1.20.50",
        r#""minecraft:icon": { "texture": "none" }"#,
        GEMS,
    );
    assert_eq!(
        found.iter().map(said).collect::<Vec<_>>(),
        [("UnresolvedReference", Severity::Error, ICON.to_owned())]
    );
    assert_eq!(named(&found[0]).1, "none");
    // The item format of 1.10: the icon is a client key of the legacy reader.
    assert_eq!(linked_at("1.10", r#""minecraft:icon": "gem""#, GEMS), []);
    for components in [
        r#""minecraft:icon": "none""#,
        r#""minecraft:icon": { "texture": "none" }"#,
    ] {
        let found = linked_at("1.10", components, GEMS);
        assert_eq!(
            found.iter().map(said).collect::<Vec<_>>(),
            [("UnresolvedReference", Severity::Error, ICON.to_owned())],
            "{components}"
        );
        assert_eq!(named(&found[0]).1, "none");
    }
}

#[test]
fn r32_an_empty_icon_name_is_no_reference() {
    assert_eq!(linked(r#""minecraft:icon": """#, GEMS), []);
    assert_eq!(
        linked(
            r#""minecraft:icon": { "textures": { "default": "gem", "dyed": "" } }"#,
            GEMS
        ),
        []
    );
}

#[test]
fn r32_an_item_without_an_icon_is_no_finding() {
    assert_eq!(linked(r#""minecraft:max_stack_size": 1"#, GEMS), []);
}

// Rule 2 of the folder: an atlas the stack does not build has no tile.
// The atlas is the add-on's own here: an atlas file of the game's pack that builds nothing
// is no view at all (`tests/link/view.rs`), and with an add-on's own a miss is an
// information.
#[test]
fn r32_every_icon_misses_an_atlas_that_was_not_built() {
    let found = crate::common::link(
        &[(ITEM, &item("1.21.80", r#""minecraft:icon": "gem""#))],
        &[(
            ATLAS,
            r#"{ "texture_data": { "gem": { "textures": "textures/items/gem" } } }"#,
        )],
    );
    assert_eq!(
        at_item(&found).iter().map(said).collect::<Vec<_>>(),
        [("UnresolvedReference", Severity::Info, ICON.to_owned())]
    );
}

// The icon names need the client's view of the item atlas: without a pack that has the atlas
// file and without a host, no name is compared. The rules about the item alone still run.
#[test]
fn r32_without_an_item_atlas_and_without_a_host_no_name_is_compared() {
    let components = r#""minecraft:icon": { "textures": { "dyed": "none", "spare": "nope" } }"#;
    let alone = [
        (
            "MissingRequiredField",
            Severity::Error,
            format!("{ICON}/textures"),
        ),
        ("NoEffect", Severity::Info, format!("{ICON}/textures/spare")),
    ];
    // No resource pack.
    let set = stacks_of(
        vec![behavior_pack(1, &[(ITEM, &item("1.21.80", components))])],
        Vec::new(),
        LoadOptions::default(),
    );
    assert_eq!(set.behavior.items().iter().count(), 1);
    assert_eq!(
        at_item(&set.link()).iter().map(said).collect::<Vec<_>>(),
        alone
    );
    // A resource pack without the atlas file.
    let other = [("textures/terrain_texture.json", GEMS)];
    let found = link(&[(ITEM, &item("1.21.80", components))], &other);
    assert_eq!(at_item(&found).iter().map(said).collect::<Vec<_>>(), alone);
    // With a host the game's tiles are in view.
    let found = link_with(
        &[(ITEM, &item("1.21.80", components))],
        &other,
        hosted(&[("atlas key", "none")]),
        &LinkOptions::default(),
    );
    assert_eq!(
        at_item(&found).iter().map(said).collect::<Vec<_>>(),
        [
            alone[0].clone(),
            alone[1].clone(),
            (
                "UnresolvedReference",
                Severity::Error,
                format!("{ICON}/textures/spare")
            ),
        ]
    );
}

#[test]
fn rule_10_textures_without_default_is_an_error_at_the_map() {
    let found = linked(
        r#""minecraft:icon": { "textures": { "dyed": "gem" } }"#,
        GEMS,
    );
    assert_eq!(
        found.iter().map(said).collect::<Vec<_>>(),
        [(
            "MissingRequiredField",
            Severity::Error,
            format!("{ICON}/textures")
        )]
    );
    assert_eq!(found[0].phase, Phase::Link);
    assert_eq!(
        found[0].code.to_string(),
        "demo:thing : minecraft:icon component should include default key in textures map to set icon texture"
    );
    // An empty map sets no icon and is not this rule's.
    assert_eq!(linked(r#""minecraft:icon": { "textures": {} }"#, GEMS), []);
    assert_eq!(linked(r#""minecraft:icon": {}"#, GEMS), []);
}

#[test]
fn rule_11_a_key_the_game_never_reads_is_a_lint_at_the_key() {
    let components = r#""minecraft:icon": { "textures": { "default": "gem", "dyed": "gem",
        "icon_trim": "gem", "bundle_open_front": "gem", "bundle_open_back": "gem",
        "spare": "gem", "trim_palette": "gem", "palette": "gem" } }"#;
    let found = linked(components, GEMS);
    assert_eq!(
        found.iter().map(said).collect::<Vec<_>>(),
        [
            ("NoEffect", Severity::Info, format!("{ICON}/textures/spare")),
            (
                "NoEffect",
                Severity::Info,
                format!("{ICON}/textures/trim_palette")
            ),
            (
                "NoEffect",
                Severity::Info,
                format!("{ICON}/textures/palette")
            ),
        ]
    );
    assert!(found.iter().all(|finding| finding.phase == Phase::Link));
    // A lint: left out with the lints off.
    let quiet = link_with(
        &[(ITEM, &item("1.21.80", components))],
        &[(ATLAS, GEMS)],
        LoadOptions::default(),
        &LinkOptions {
            lints: false,
            ..Default::default()
        },
    );
    assert_eq!(at_item(&quiet), []);
}

// Rule 9 names "each `textures.<key>` value except `<material>_palette` values": the value of
// a key the game never reads is still a recorded reference.
#[test]
fn rule_11_the_value_of_an_unread_key_is_still_an_atlas_key() {
    let found = linked(
        r#""minecraft:icon": { "textures": { "default": "gem", "spare": "none" } }"#,
        GEMS,
    );
    assert_eq!(
        found.iter().map(said).collect::<Vec<_>>(),
        [
            ("NoEffect", Severity::Info, format!("{ICON}/textures/spare")),
            (
                "UnresolvedReference",
                Severity::Error,
                format!("{ICON}/textures/spare")
            ),
        ]
    );
}

// Rule 24; trim T5: the prefix is the text before the first `_palette` and must be one of the
// eleven trim materials.
#[test]
fn rule_24_a_material_palette_value_is_no_atlas_key() {
    for material in [
        "quartz",
        "iron",
        "netherite",
        "redstone",
        "copper",
        "gold",
        "emerald",
        "diamond",
        "lapis",
        "amethyst",
        "resin",
    ] {
        let components = format!(
            r#""minecraft:icon": {{ "textures": {{ "default": "gem", "{material}_palette": "no_tile" }} }}"#
        );
        assert_eq!(linked(&components, GEMS), [], "{material}");
    }
    // The text after `_palette` is not read.
    assert_eq!(
        linked(
            r#""minecraft:icon": { "textures": { "default": "gem", "iron_palette_two": "no_tile" } }"#,
            GEMS
        ),
        []
    );
    // No material: an unread key whose value is a reference like any other.
    let found = linked(
        r#""minecraft:icon": { "textures": { "default": "gem", "steel_palette": "no_tile" } }"#,
        GEMS,
    );
    assert_eq!(
        found.iter().map(said).collect::<Vec<_>>(),
        [
            (
                "NoEffect",
                Severity::Info,
                format!("{ICON}/textures/steel_palette")
            ),
            (
                "UnresolvedReference",
                Severity::Error,
                format!("{ICON}/textures/steel_palette")
            ),
        ]
    );
}

#[test]
fn rule_12_a_tile_the_atlas_dropped_is_no_tile_and_no_near_miss() {
    // `Gem` has no `textures`: the load drops it (rule 4).
    let atlas = r#"{ "texture_name": "atlas.items", "texture_data": { "Gem": { "quad": true } } }"#;
    for written in ["Gem", "gem"] {
        let found = linked(&format!(r#""minecraft:icon": "{written}""#), atlas);
        assert_eq!(found.len(), 1, "{found:#?}");
        assert_eq!(
            said(&found[0]),
            ("UnresolvedReference", Severity::Error, ICON.to_owned())
        );
        assert_eq!(named(&found[0]), ("atlas key", written, None));
    }
}

const TRIMMABLE: &str = r#""minecraft:tags": { "tags": ["minecraft:trimmable_armors"] }"#;
const GEM_ICON: &str = r#""minecraft:icon": "gem""#;

#[test]
fn rule_22_trimmable_armour_without_wearable_is_a_warning_at_the_components() {
    let found = linked(&format!("{TRIMMABLE}, {GEM_ICON}"), GEMS);
    assert_eq!(
        found.iter().map(said).collect::<Vec<_>>(),
        [(
            "MissingRequiredField",
            Severity::Warning,
            "minecraft:item/components".to_owned()
        )]
    );
    assert_eq!(found[0].phase, Phase::Link);
    assert_eq!(
        found[0].code.to_string(),
        "Custom armor item 'demo:thing' needs a wearable component"
    );
}

#[test]
fn rule_22_is_about_tagged_items_with_an_icon_and_no_wearable_only() {
    let wearable = r#""minecraft:wearable": { "slot": "slot.armor.chest" }"#;
    // With the component.
    assert_eq!(
        linked(&format!("{TRIMMABLE}, {GEM_ICON}, {wearable}"), GEMS),
        []
    );
    // Without the tag.
    assert_eq!(linked(GEM_ICON, GEMS), []);
    assert_eq!(
        linked(
            &format!(r#""minecraft:tags": {{ "tags": ["minecraft:is_armor"] }}, {GEM_ICON}"#),
            GEMS
        ),
        []
    );
    // Without an icon the game stops at the rule before this one (rule 21).
    let found = linked(TRIMMABLE, GEMS);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(
        found[0].code.to_string(),
        "Custom armor item 'demo:thing' needs an icon component"
    );
}

// K119, rule 21: the game asks a trimmable armour for its icon first; without one it says so
// and makes no trimmed icons, whatever else the item has.
#[test]
fn rule_21_trimmable_armour_without_an_icon_is_a_warning_at_the_components() {
    let wearable = r#""minecraft:wearable": { "slot": "slot.armor.chest" }"#;
    for components in [TRIMMABLE.to_owned(), format!("{TRIMMABLE}, {wearable}")] {
        let found = linked(&components, GEMS);
        assert_eq!(
            found.iter().map(said).collect::<Vec<_>>(),
            [(
                "MissingRequiredField",
                Severity::Warning,
                "minecraft:item/components".to_owned()
            )],
            "{components}"
        );
        assert_eq!(found[0].phase, Phase::Link);
        assert_eq!(
            found[0].code.to_string(),
            "Custom armor item 'demo:thing' needs an icon component"
        );
    }
    // Without the tag an item needs no icon.
    assert_eq!(linked(wearable, GEMS), []);
    assert_eq!(
        linked(
            r#""minecraft:tags": { "tags": ["minecraft:is_armor"] }"#,
            GEMS
        ),
        []
    );
}

// K119, rule 23: with an icon and a wearable, the wearable must be armour: one of the four
// armour slots. The off hand is the one slot of the component that is none.
#[test]
fn rule_23_trimmable_armour_worn_in_the_off_hand_is_a_warning_at_the_slot() {
    let offhand = r#""minecraft:wearable": { "slot": "slot.weapon.offhand" }"#;
    let found = linked(&format!("{TRIMMABLE}, {GEM_ICON}, {offhand}"), GEMS);
    assert_eq!(
        found.iter().map(said).collect::<Vec<_>>(),
        [(
            "InvalidValue",
            Severity::Warning,
            "minecraft:item/components/minecraft:wearable/slot".to_owned()
        )]
    );
    assert_eq!(found[0].phase, Phase::Link);
    assert_eq!(
        found[0].code.to_string(),
        "Only armor wearable components are trimmable; 'demo:thing' is not armor."
    );
    // Each of the four armour slots is armour.
    for slot in ["head", "chest", "legs", "feet"] {
        let wearable = format!(r#""minecraft:wearable": {{ "slot": "slot.armor.{slot}" }}"#);
        assert_eq!(
            linked(&format!("{TRIMMABLE}, {GEM_ICON}, {wearable}"), GEMS),
            [],
            "{slot}"
        );
    }
    // Without the tag the slot is nobody's business, and without an icon rule 21 is the one
    // finding.
    assert_eq!(linked(&format!("{GEM_ICON}, {offhand}"), GEMS), []);
    let found = linked(&format!("{TRIMMABLE}, {offhand}"), GEMS);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(
        found[0].code.to_string(),
        "Custom armor item 'demo:thing' needs an icon component"
    );
}
