//! The validation rules and the parse-time gates: fixtures `v01`–`v05`, `v07`–`v14`, `v16`–`v20`.

use bedrock_addon::diagnostic::{Severity, codes};
use bedrock_addon::document::ComponentRef;
use bedrock_addon::families::item::ItemFormat;
use bedrock_addon::families::item::component::{
    BlockPlacer, BundleInteraction, Cooldown, CooldownType, Damage, Fuel, UseModifiers, Wearable,
};
use bedrock_addon::json::Value;
use bedrock_addon::pack::PackType;
use bedrock_addon::shared::CreativeCategory;
use bedrock_addon::version::FormatVersion;

use bedrock_addon::diagnostic::Diagnostics;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{PackOrigin, PackRepository, PackSelection, StackSet, WorldPacks};

use super::common::*;
use super::pack_common::{Spec, bp, rp};

#[test]
fn v01_damage_ranges() {
    let at_110 = loads(
        &item("1.21.110", "demo:x", r#""minecraft:damage": 300"#),
        &third_party(),
    );
    assert_eq!(at_110.components.get::<Damage>().unwrap().value, 300);
    // `min(0)` is bound in the 1.26.0 and 1.20.50 slices: cereal's 0x40, the item
    // refused.
    let d = rejects(
        &item("1.21.110", "demo:x", r#""minecraft:damage": -1"#),
        &third_party(),
    );
    assert!(
        error_at(
            &d,
            "InvalidValue",
            "minecraft:item/components/minecraft:damage/value"
        ),
        "{d:#?}"
    );
    let top = loads(
        &item("1.26.0", "demo:x", r#""minecraft:damage": 32767"#),
        &third_party(),
    );
    assert_eq!(top.components.get::<Damage>().unwrap().value, 32767);
}

#[test]
fn v02_foil() {
    let foil = r#""minecraft:foil": true, "minecraft:max_stack_size": 1"#;
    let at_20 = loads(&item("1.20.20", "demo:x", foil), &third_party());
    assert!(has_text(
        at_20.diagnostics(),
        "minecraft:foil has been deprecated in format versions 1.20.20 and above. Use minecraft:glint instead."
    ));
    assert_eq!(at_20.legacy.is_glint, None);
    let allowlisted = loads(&item("1.20.10", "minecraft:x", foil), &allowlisted());
    assert_eq!(allowlisted.legacy.is_glint, Some(true));
    let third = loads(&item("1.20.10", "demo:x", foil), &third_party());
    assert_eq!(third.legacy.is_glint, None);
    assert_eq!(with_code(third.diagnostics(), "DroppedComponent").len(), 1);
    let both = loads(
        &item(
            "1.20.20",
            "demo:x",
            r#""minecraft:foil": true, "minecraft:glint": true"#,
        ),
        &third_party(),
    );
    assert!(has(both.diagnostics(), |c| c
        .args_of(&codes::DUPLICATE_COMPONENT_VERSION)
        .is_some_and(
            |args| args[0] == "minecraft:foil" && args[1] == "minecraft:glint"
        )));
}

#[test]
fn v03_the_custom_component_list() {
    let list = r#""minecraft:custom_components": ["demo:shake"]"#;
    // 1.21.90: deprecated by cereal's 0x200, an error in both
    // masks: the item is refused
    let at_90 = rejects(&item("1.21.90", "demo:x", list), &third_party());
    assert!(
        at_90.iter().any(|f| f.severity == Severity::Error
            && !f.kept
            && f.code.to_string() == "member \"minecraft:custom_components\" is deprecated"),
        "{at_90:#?}"
    );
    // 1.20.80 without the Beta APIs: the toggle text, not loaded
    let d = rejects(&item("1.20.80", "demo:x", list), &third_party());
    assert!(has(&d, |c| c.is(&codes::TURN_ON_TOGGLE_BEFORE)));
    let mut beta = third_party();
    beta.beta_apis = true;
    loads(&item("1.20.80", "demo:x", list), &beta);
}

/// A custom component of the `minecraft` namespace behind entries the reader of the list
/// leaves out (one that is neither a name nor an object with one) is told at the index the
/// file has it at.
#[test]
fn v03_a_refused_custom_component_behind_dropped_entries_is_at_the_index_of_the_file() {
    // Each entry of another shape (0x8) and each `minecraft` name (0x40) refuses the item;
    // every error is at the index the file has the entry at.
    let list = r#""minecraft:custom_components": [7, "minecraft:mine", {"data": 1}, "demo:shake", {"name": "minecraft:too"}]"#;
    let mut beta = third_party();
    beta.beta_apis = true;
    for version in ["1.21.60", "1.21.80"] {
        let d = rejects(&item(version, "demo:x", list), &beta);
        let at = "minecraft:item/components/minecraft:custom_components/";
        let mut told: Vec<(String, &'static str)> = d
            .iter()
            .filter_map(|finding| {
                let path = finding.location.json_path.to_string();
                Some((path.strip_prefix(at)?.to_owned(), finding.code.name()))
            })
            .collect();
        told.sort();
        assert_eq!(
            told,
            [
                ("0".to_owned(), "InvalidType"),
                ("1".to_owned(), "InvalidValue"),
                ("2".to_owned(), "InvalidType"),
                ("4".to_owned(), "InvalidValue"),
            ],
            "{version}: {:#?}",
            d.iter().map(ToString::to_string).collect::<Vec<_>>()
        );
        assert!(
            d.iter()
                .filter(|f| f.location.json_path.to_string().starts_with(at))
                .all(|f| f.severity == Severity::Error),
            "{version}: {d:#?}"
        );
    }
}

#[test]
fn v04_a_lowered_engine_version_refuses_cooldown_type() {
    let json = item(
        "1.26.0",
        "demo:x",
        r#""minecraft:cooldown": {"category": "x", "duration": 1.0, "type": "attack"}"#,
    );
    let normal = loads(&json, &third_party());
    assert_eq!(
        normal.components.get::<Cooldown>().unwrap().kind,
        CooldownType::Attack
    );
    let mut old = third_party();
    old.engine_version = FormatVersion::new(1, 21, 120);
    let lowered = loads(&json, &old);
    let cooldown = lowered.components.get::<Cooldown>().unwrap();
    assert_eq!(cooldown.kind, CooldownType::Use);
    let _ = cooldown;
    let at = bedrock_addon::diagnostic::JsonPath::of_keys([
        "minecraft:item",
        "components",
        "minecraft:cooldown",
    ]);
    assert!(lowered.unknown().get(&at, "type").is_some());
    // By legacy name: the variant and the definitions that were it.
    assert!(has(lowered.diagnostics(), |c| matches!(
        c.name(),
        "FieldSinceEngine"
    )));
}

/// The engine reads every member the crate predicts for an
/// older engine, so no line of the game stands behind `FieldSinceEngine`: an information,
/// while a member the version's struct does not bind stays 0x4's warning.
#[test]
fn v04b_a_member_above_the_engine_version_is_an_information_and_an_unbound_one_a_warning() {
    let json = item(
        "1.26.0",
        "demo:x",
        r#""minecraft:cooldown": {"category": "x", "duration": 1.0, "type": "attack"}"#,
    );
    let mut old = third_party();
    old.engine_version = FormatVersion::new(1, 21, 120);
    let lowered = loads(&json, &old);
    let told: Vec<_> = lowered
        .diagnostics()
        .iter()
        // By legacy name: the variant and the definitions that were it.
        .filter(|f| matches!(f.code.name(), "FieldSinceEngine"))
        .collect();
    assert_eq!(told.len(), 1);
    assert_eq!(told[0].severity, Severity::Info, "{told:#?}");
    let unbound = item(
        "1.21.30",
        "demo:y",
        r#""minecraft:block_placer": {"block": "demo:b", "replace_block_item": true}"#,
    );
    let loaded = loads(&unbound, &third_party());
    assert!(
        // By legacy name: the variant and the definitions that were it.
        loaded.diagnostics().iter().any(
            |f| matches!(f.code.name(), "UnrecognizedField") && f.severity == Severity::Warning
        ),
        "{:#?}",
        loaded.diagnostics()
    );
}

#[test]
fn v07_the_component_metadata_gates() {
    let food = r#""minecraft:food": {"nutrition": 1}"#;
    let d = rejects(&item("1.20.20", "demo:x", food), &third_party());
    assert!(has_text(
        &d,
        "To use item component 'minecraft:food', use json format version 1.20.30 or higher"
    ));
    loads(&item("1.20.20", "minecraft:x", food), &allowlisted());
    // an old third-party item fails on the item-level gate first
    let weapon = r#""minecraft:weapon": {}"#;
    let d = rejects(&item("1.16.100", "demo:x", weapon), &third_party());
    assert!(has_text(
        &d,
        "To use item 'demo:x', use json format version 1.20.0 or higher"
    ));
    assert!(!has_text(&d, "item component"));
    // render_offsets at its deprecation
    let offsets = loads(
        &item(
            "1.20.10",
            "demo:x",
            r#""minecraft:render_offsets": "tools""#,
        ),
        &third_party(),
    );
    assert!(has_text(
        offsets.diagnostics(),
        "minecraft:render_offsets has been deprecated and is not available in json format 1.20.10"
    ));
    let glint = r#""minecraft:glint": true"#;
    loads(&item("1.16.100", "minecraft:x", glint), &allowlisted());
    let d = rejects(&item("1.16.100", "demo:x", glint), &third_party());
    assert!(has_text(
        &d,
        "To use item 'demo:x', use json format version 1.20.0 or higher"
    ));
}

#[test]
fn v08_menu_category() {
    let menu = |version: &str, menu: &str| {
        format!(
            r#"{{"format_version": "{version}", "minecraft:item": {{"description": {{"identifier": "demo:x", "menu_category": {menu}}}, "components": {{"minecraft:hand_equipped": true}}}}}}"#
        )
    };
    let undefined = loads(&menu("1.21.60", "{}"), &third_party());
    assert_eq!(
        undefined.description.menu_category.category,
        Some(CreativeCategory::Undefined)
    );
    // Nothing of the game's. Ours: `Undefined` has no spelling in the format the item is
    // written in, which the upgrade says (`round_trip.rs`,
    // `an_undefined_category_the_newest_format_cannot_spell_is_told_at_the_load`).
    // An information.
    let told: Vec<&str> = undefined
        .diagnostics()
        .iter()
        .filter(|f| f.code.name() == "ValueRewritten")
        .map(|f| f.code.name())
        .collect();
    assert_eq!(told, ["ValueRewritten"]);
    assert!(above_info(undefined.diagnostics()).is_empty());
    let none = loads(&menu("1.21.110", "{}"), &third_party());
    assert_eq!(
        none.description.menu_category.category,
        Some(CreativeCategory::ItemCommandOnly)
    );
    let d = rejects(&menu("1.21.60", r#"{"group": "bundles"}"#), &third_party());
    assert!(has_text(
        &d,
        "menu_category -> group: string must be prefixed with a namespace (eg. namespace:value)"
    ));
}

#[test]
fn v09_the_validators() {
    let warned = |components: &str, text: &str| {
        let document = loads(&item("1.26.30", "demo:x", components), &third_party());
        let finding = document
            .diagnostics()
            .iter()
            .find(|f| f.code.to_string().contains(text))
            .unwrap_or_else(|| panic!("no `{text}`: {:#?}", document.diagnostics()));
        // The game writes it to the content log at level 2 and it fails nothing
        assert_eq!(finding.severity, Severity::Warning, "{text}");
        document
    };
    let errs = |components: &str, text: &str| {
        let d = rejects(&item("1.26.30", "demo:x", components), &third_party());
        let finding = d
            .iter()
            .find(|f| f.code.to_string().contains(text))
            .unwrap_or_else(|| panic!("no `{text}`: {d:#?}"));
        assert_eq!(finding.severity, Severity::Error, "{text}");
    };
    // rule 6
    warned(
        r#""minecraft:food": {"nutrition": 1}"#,
        "The \"minecraft:food\" component requires \"minecraft:use_modifiers > use_duration\"",
    );
    warned(
        r#""minecraft:shooter": {}"#,
        "The \"minecraft:shooter\" component requires",
    );
    // a legacy use_duration satisfies it
    let food = loads(
        &item(
            "1.21.0",
            "demo:x",
            r#""minecraft:food": {"nutrition": 1}, "minecraft:use_duration": 1.6"#,
        ),
        &third_party(),
    );
    assert!(!has_text(food.diagnostics(), "use_duration\" to be set"));
    // rule 7
    warned(
        r#""minecraft:damage_absorption": {"absorbable_causes": ["all"]}"#,
        "\"minecraft:damage_absorption\" component requires a \"minecraft:durability\"",
    );
    warned(
        r#""minecraft:durability_sensor": {"durability_thresholds": [{"durability": 5}]}"#,
        "\"minecraft:durability_sensor\" component requires a \"minecraft:durability\"",
    );
    // rule 8
    let bundle = warned(
        r#""minecraft:bundle_interaction": {}"#,
        "requires a \"minecraft:storage_item\" component.",
    );
    assert!(bundle.components.contains::<BundleInteraction>());
    errs(r#""minecraft:storage_item": {}"#, "set to 1.");
    loads(
        &item(
            "1.26.30",
            "demo:x",
            r#""minecraft:storage_item": {}, "minecraft:max_stack_size": 1"#,
        ),
        &third_party(),
    );
    // rule 9
    let kinetic = r#""minecraft:kinetic_weapon": {"damage_conditions": {"max_duration": 10}}"#;
    errs(kinetic, "requires a \"minecraft:use_modifiers\" component.");
    let with_use = |k: &str| format!(r#"{k}, "minecraft:use_modifiers": {{"use_duration": 1.0}}"#);
    errs(
        &with_use(r#""minecraft:kinetic_weapon": {}"#),
        "requires at least one of \"damage_conditions\"",
    );
    errs(
        &with_use(r#""minecraft:kinetic_weapon": {"knockback_conditions": {"max_duration": 0}}"#),
        "\"knockback_conditions\" cannot have a \"max_duration\" of zero",
    );
    errs(
        &with_use(
            r#""minecraft:kinetic_weapon": {"damage_conditions": {"min_speed": 1, "min_relative_speed": 1}}"#,
        ),
        "\"min_speed\" and \"min_relative_speed\" are mutually exclusive",
    );
    loads(
        &item("1.26.30", "demo:x", &with_use(kinetic)),
        &third_party(),
    );
    // rule 10
    errs(
        r#""minecraft:swing_sounds": {}"#,
        "requires at least one of its fields to be defined.",
    );
}

#[test]
fn v10_unknown_members_are_reported_from_1_20_60() {
    let json = |version: &str| {
        item(
            version,
            "demo:x",
            r#""minecraft:fuel": {"duration": 1.0, "smoke": true}, "minecraft:mystery": {}"#,
        )
    };
    // Quiet but for the load's note that the newest format refuses the unknown member, so
    // the item is not written in it.
    let quiet = loads(&json("1.20.50"), &third_party());
    let told: Vec<String> = quiet
        .diagnostics()
        .iter()
        .filter(|f| f.code.name() == "InvalidValue")
        .map(|f| format!("{} {:?} {}", f.code.name(), f.phase, f.location.json_path))
        .collect();
    assert_eq!(
        told,
        ["InvalidValue Validate minecraft:item/components/minecraft:fuel/smoke"],
        "{:#?}",
        quiet.diagnostics()
    );
    let loud = loads(&json("1.20.60"), &third_party());
    assert_eq!(with_code(loud.diagnostics(), "UnrecognizedField").len(), 1);
    // A name no component ever had is removed by the game's component validation without a
    // line: an information.
    let unknown: Vec<Severity> = with_code(loud.diagnostics(), "UnrecognizedComponent")
        .iter()
        .map(|f| f.severity)
        .collect();
    assert_eq!(unknown, [Severity::Info], "{:#?}", loud.diagnostics());
    // A name of a later format stays in the document and the slice does not bind it:
    // cereal's 0x4, a level-2 line under 0xfbf8: a warning.
    let later = loads(
        &item(
            "1.20.60",
            "demo:x",
            r#""minecraft:icon": "x", "minecraft:compostable": {"composting_chance": 50}"#,
        ),
        &third_party(),
    );
    let told: Vec<(Severity, String)> = with_code(later.diagnostics(), "UnrecognizedComponent")
        .iter()
        .map(|f| (f.severity, f.location.json_path.to_string()))
        .collect();
    assert_eq!(
        told,
        [(
            Severity::Warning,
            "minecraft:item/components/minecraft:compostable".to_owned()
        )],
        "{:#?}",
        later.diagnostics()
    );
    assert!(loud.components.get::<Fuel>().is_some());
    let at = bedrock_addon::diagnostic::JsonPath::of_keys([
        "minecraft:item",
        "components",
        "minecraft:fuel",
    ]);
    assert!(loud.unknown().get(&at, "smoke").is_some());
}

#[test]
fn v11_the_legacy_read() {
    let speed = r#""minecraft:mining_speed": 1.5, "minecraft:max_stack_size": 1"#;
    let kept = loads(&item("1.20.20", "demo:x", speed), &third_party());
    assert_eq!(kept.legacy.mining_speed, Some(1.5));
    let gone = loads(&item("1.20.30", "demo:x", speed), &third_party());
    assert_eq!(gone.legacy.mining_speed, None);
    assert!(has_text(
        gone.diagnostics(),
        "minecraft:mining_speed has been deprecated in format versions 1.20.30"
    ));
    let category = loads(
        &item(
            "1.20.20",
            "demo:x",
            r#""minecraft:creative_category": {"parent": "x"}, "minecraft:max_stack_size": 1"#,
        ),
        &third_party(),
    );
    assert!(has_text(
        category.diagnostics(),
        "minecraft:creative_category has been deprecated and is not available in json format 1.20.20"
    ));
    let tag = r#""tag:demo:old": {}"#;
    let upgraded = loads(&item("1.20.40", "demo:x", tag), &third_party());
    assert_eq!(
        upgraded
            .components
            .get::<bedrock_addon::families::item::component::Tags>()
            .unwrap()
            .tags,
        ["demo:old"]
    );
    let late = loads(&item("1.20.50", "demo:x", tag), &third_party());
    assert!(has_text(
        late.diagnostics(),
        "tag: has been deprecated in format versions 1.20.50"
    ));
    let modifiers = loads(
        &item(
            "1.20.40",
            "demo:x",
            r#""minecraft:use_modifiers": {"use_duration": 1.0}, "minecraft:max_stack_size": 1"#,
        ),
        &third_party(),
    );
    assert!(!modifiers.components.contains::<UseModifiers>());
    assert!(
        modifiers
            .legacy
            .shared
            .contains_key("minecraft:use_modifiers")
    );
    let button = loads(
        &item("1.21.0", "demo:x", r#""minecraft:interact_button": true"#),
        &third_party(),
    );
    assert_eq!(
        button.legacy.shared.get("minecraft:interact_button"),
        Some(&Value::from(true))
    );
}

#[test]
fn v12_the_item_level_gate() {
    let size = r#""minecraft:max_stack_size": 16"#;
    for version in ["1.16.100", "1.17.0", "1.19.80"] {
        let d = rejects(&item(version, "demo:x", size), &third_party());
        assert!(
            has_text(
                &d,
                "To use item 'demo:x', use json format version 1.20.0 or higher"
            ),
            "{version}"
        );
    }
    loads(&item("1.20.0", "demo:x", size), &third_party());
    loads(&item("1.16.100", "minecraft:x", size), &allowlisted());
}

#[test]
fn v13_resource_pack_items() {
    let mut rp = third_party();
    rp.pack_type = PackType::Resource;
    let d = rejects(
        &item("1.16.100", "demo:x", r#""minecraft:max_stack_size": 16"#),
        &rp,
    );
    assert!(has_text(
        &d,
        "Item 'demo:x' with format_version 1.16.100 or greater cannot be loaded from resource packs."
    ));
    // below 1.16.100: parsed (the resource-pack walk registers nothing)
    let legacy = loads(&item("1.10", "demo:x", r#""minecraft:icon": "x""#), &rp);
    assert_eq!(legacy.legacy.format, ItemFormat::Legacy);
}

#[test]
fn v13_a_resource_pack_item_without_a_behaviour_pack_twin_is_ignored() {
    // Rule 18: a legacy resource-pack item is parsed, not registered; with no behaviour-pack
    // definition of its identifier vanilla logs that it is ignored at level 3
    // and drops the definition: a kept error.
    const LONE: &[u8] = br#"{"format_version": "1.10", "minecraft:item": {"description": {"identifier": "demo:lone"}, "components": {"minecraft:icon": "lone"}}}"#;
    const TWIN_RP: &[u8] = br#"{"format_version": "1.10", "minecraft:item": {"description": {"identifier": "demo:twin"}, "components": {"minecraft:icon": "twin"}}}"#;
    const TWIN_BP: &[u8] = br#"{"format_version": "1.26.30", "minecraft:item": {"description": {"identifier": "demo:twin"}, "components": {"minecraft:max_stack_size": 16}}}"#;
    let set = stack_set(
        &bp(1).file("items/twin.json", TWIN_BP),
        &rp(2)
            .file("items/lone.json", LONE)
            .file("items/twin.json", TWIN_RP),
    );
    let d = set.link();
    assert_eq!(
        link_messages(
            &d,
            "UnresolvedReference",
            Severity::Error,
            "items/lone.json"
        ),
        [
            "Resource pack has item definitions not found in the behavior pack. Item 'demo:lone' is being ignored."
        ],
        "{d:#?}"
    );
    assert!(
        d.iter()
            .filter(|f| f.code.name() == "UnresolvedReference")
            .all(|f| f.kept),
        "{d:#?}"
    );
    // the twin has a behaviour-pack definition: no finding
    assert!(
        link_messages(
            &d,
            "UnresolvedReference",
            Severity::Error,
            "items/twin.json"
        )
        .is_empty(),
        "{d:#?}"
    );
}

#[test]
fn v14_use_beta_features() {
    let json = |version: &str, components: &str| {
        format!(
            r#"{{"format_version": "{version}", "use_beta_features": true, "minecraft:item": {{"description": {{"identifier": "demo:x"}}, "components": {{{components}}}}}}}"#
        )
    };
    // No gate in 1.26.60; the member is read by nothing and
    // told where it is, and the item loads without the toggle.
    for (version, components) in [("1.10", ""), ("1.21.0", r#""minecraft:max_stack_size": 1"#)] {
        let document = loads(&json(version, components), &third_party());
        // An information of ours: the game says nothing of it.
        let told: Vec<String> = unknown_members(document.diagnostics())
            .iter()
            .map(|f| {
                format!(
                    "{} {} {:?}",
                    f.location.json_path,
                    f.code.name(),
                    f.severity
                )
            })
            .collect();
        assert_eq!(
            told,
            ["use_beta_features UnrecognizedField Info"],
            "{version}"
        );
    }
}

#[test]
fn v16_a_wearable_defaults_to_a_stack_of_one() {
    let worn = loads(
        &item(
            "1.26.30",
            "demo:x",
            r#""minecraft:wearable": {"slot": "slot.armor.head"}"#,
        ),
        &third_party(),
    );
    assert!(has_text(
        worn.diagnostics(),
        "Items with the 'minecraft:wearable' component and 'slot' set to 'slot.armor.head' will default to a max stack size of 1. Use the 'minecraft:max_stack_size' component to override."
    ));
    // The game logs it at level 3 and the item loads: a kept error
    let told = worn
        .diagnostics()
        .iter()
        .find(|f| {
            f.code
                .to_string()
                .contains("will default to a max stack size of 1")
        })
        .unwrap();
    assert_eq!((told.severity, told.kept), (Severity::Error, true));
    assert_eq!(worn.max_stack_size(), 1);
    let sized = loads(
        &item(
            "1.26.30",
            "demo:x",
            r#""minecraft:wearable": {"slot": "slot.armor.head"}, "minecraft:max_stack_size": 16"#,
        ),
        &third_party(),
    );
    assert!(!has_text(
        sized.diagnostics(),
        "will default to a max stack size of 1"
    ));
    assert_eq!(sized.max_stack_size(), 16);
    assert_eq!(sized.components.get::<Wearable>().unwrap().protection, 0);
}

#[test]
fn v17_binding_constraints_refuse_the_item() {
    // A required member missing (0x80) or a constraint failed (0x40) in the legacy chain's
    // cereal slices: the parse fails and the item is refused with " Error Parsing Item", as the
    // strict slices do.
    let dropped = |components: &str, code: &str| {
        let d = rejects(&item("1.26.0", "demo:x", components), &third_party());
        assert!(
            d.iter()
                .any(|f| f.code.name() == code && f.severity == Severity::Error),
            "{components}: {d:#?}"
        );
    };
    for missing in [
        r#""minecraft:cooldown": {"duration": 1.0}"#,
        r#""minecraft:cooldown": {"category": "x"}"#,
        r#""minecraft:digger": {"destroy_speeds": [{"block": "minecraft:dirt"}]}"#,
        r#""minecraft:digger": {"destroy_speeds": [{"speed": 2}]}"#,
        r#""minecraft:wearable": {}"#,
        r#""minecraft:block_placer": {}"#,
        r#""minecraft:compostable": {}"#,
        r#""minecraft:damage_absorption": {}"#,
        r#""minecraft:durability": {}"#,
        r#""minecraft:durability_sensor": {}"#,
        r#""minecraft:enchantable": {"slot": "sword"}"#,
        r#""minecraft:enchantable": {"value": 1}"#,
        r#""minecraft:projectile": {}"#,
        r#""minecraft:repairable": {"repair_items": [{"repair_amount": 1}]}"#,
        r#""minecraft:shooter": {"ammunition": [{"use_offhand": true}]}"#,
    ] {
        dropped(missing, "MissingRequiredField");
    }
    // Numeric constraints (max_stack_size `range`, fuel `min`, use_modifiers `range` in the
    // 1.20.50 slice): 0x40, refused.
    for invalid in [
        r#""minecraft:max_stack_size": 65"#,
        r#""minecraft:fuel": 0.01"#,
        r#""minecraft:use_modifiers": {"use_duration": 1.0, "movement_modifier": 1.5}"#,
    ] {
        dropped(invalid, "InvalidValue");
    }
    // Checks of ours whose binding was not read: the component is dropped with an
    // information (open).
    let noted = |components: &str| {
        let document = loads(&item("1.26.0", "demo:x", components), &third_party());
        assert_eq!(
            document
                .components
                .iter()
                .filter(|e| matches!(e, ComponentRef::Typed { .. }))
                .count(),
            0,
            "{components}"
        );
        assert!(
            with_code(document.diagnostics(), "InvalidValue")
                .iter()
                .any(|f| f.severity == Severity::Info),
            "{components}: {:#?}",
            document.diagnostics()
        );
        // The crate's summary that the component goes whole: no game line.
        assert!(
            with_code(document.diagnostics(), "DroppedComponent")
                .iter()
                .all(|f| f.severity == Severity::Info),
            "{components}: {:#?}",
            document.diagnostics()
        );
    };
    for case in [
        r#""minecraft:compostable": {"composting_chance": 0}"#,
        r#""minecraft:damage_absorption": {"absorbable_causes": []}"#,
        r#""minecraft:durability_sensor": {"durability_thresholds": []}"#,
    ] {
        noted(case);
    }
    let many: Vec<String> = (0..257).map(|i| format!(r#""demo:b{i}""#)).collect();
    let long = format!(
        r#""minecraft:block_placer": {{"block": "minecraft:stone", "use_on": [{}]}}"#,
        many.join(", ")
    );
    let twice = r#""minecraft:block_placer": {"block": "minecraft:stone", "use_on": ["minecraft:dirt", "minecraft:dirt"]}"#;
    for case in [long.as_str(), twice] {
        noted(case);
        let older = loads(&item("1.21.110", "demo:x", case), &third_party());
        assert!(older.components.contains::<BlockPlacer>());
    }
}

#[test]
fn v18_the_root_keys_of_1_26_30() {
    let root = |version: &str, key: &str| {
        format!(
            r#"{{"format_version": "{version}", "minecraft:item": {{"description": {{"identifier": "demo:x"}}, "components": {{"minecraft:max_stack_size": 1}}, "{key}": {{"on_use": {{"on_use_trigger": {{"event": "e"}}}}}}}}}}"#
        )
    };
    // An unknown member of the 1.26.30 payload: an error in the strict slice.
    let unknown = rejects(&root("1.26.30", "__deprecated_components"), &third_party());
    assert!(
        unknown.iter().any(|f| f.severity == Severity::Error
            && f.code.name() == "UnrecognizedField"
            && f.location.json_path.to_string() == "minecraft:item/__deprecated_components"),
        "{unknown:#?}"
    );
    // Bound with `deprecateName`: cereal's 0x200, an error in both masks that rejects the
    // item.
    for key in ["deprecated_component_data", "legacy_data"] {
        let d = rejects(&root("1.26.30", key), &third_party());
        assert!(
            d.iter().any(|f| f.severity == Severity::Error
                && f.code.to_string() == format!("member \"{key}\" is deprecated")),
            "{key}: {d:#?}"
        );
    }
    // Bound with `deprecateName` in the 1.20.30–1.26.0 roots roots: cereal's 0x200
    // as above, the reader's two flags clear for a JSON object (`expectedType` is `Object`
    // and the loader config's first byte is zero for JSON): the item is refused.
    let older = rejects(&root("1.26.0", "__deprecated_components"), &third_party());
    assert!(
        older.iter().any(|f| f.severity == Severity::Error
            && f.code.to_string() == "member \"__deprecated_components\" is deprecated"),
        "{older:#?}"
    );
}

#[test]
fn v19_custom_component_names() {
    // The namespace constraint disallowing "minecraft" (1.20.80, 1.21.60) fails as cereal's
    // 0x40 ("string has invalid namespace"): an error under both masks, and the item is
    // refused.
    for version in ["1.21.10", "1.21.60"] {
        let d = rejects(
            &item(
                version,
                "demo:x",
                r#""minecraft:custom_components": ["minecraft:shake", "demo:shake"]"#,
            ),
            &third_party(),
        );
        assert!(
            has_text(
                &d,
                "custom component names must not use the minecraft namespace"
            ) && error_at(
                &d,
                "InvalidValue",
                "minecraft:item/components/minecraft:custom_components/0"
            ),
            "{version}: {d:#?}"
        );
    }
    let keyed = r#""demo:shake": {"strength": 2}, "minecraft:max_stack_size": 1"#;
    let early = loads(&item("1.21.80", "demo:x", keyed), &third_party());
    assert_eq!(
        with_code(early.diagnostics(), "UnrecognizedComponent").len(),
        1
    );
    assert!(
        !early
            .components
            .iter()
            .any(|e| matches!(e, ComponentRef::Custom(_)))
    );
    let on_time = loads(&item("1.21.90", "demo:x", keyed), &third_party());
    assert!(
        on_time
            .components
            .iter()
            .any(|e| matches!(e, ComponentRef::Custom(c) if c.name == "demo:shake"))
    );
}

#[test]
fn v20_wearable_protection_below_1_20_20_and_a_render_offsets_string() {
    let worn = loads(
        &item(
            "1.20.10",
            "minecraft:x",
            r#""minecraft:wearable": {"slot": "slot.armor.head", "protection": 3}"#,
        ),
        &allowlisted(),
    );
    assert_eq!(worn.components.get::<Wearable>().unwrap().protection, 3);
    assert!(
        above_info(worn.diagnostics()).is_empty(),
        "{:#?}",
        worn.diagnostics()
    );
    let offsets = loads(
        &item(
            "1.20.0",
            "minecraft:x",
            r#""minecraft:render_offsets": "tools""#,
        ),
        &allowlisted(),
    );
    assert!(
        !offsets
            .components
            .iter()
            .any(|e| matches!(e, ComponentRef::Unknown { .. }))
    );
    assert!(
        offsets
            .diagnostics()
            .iter()
            .all(|f| f.severity < Severity::Error)
    );
}

#[test]
fn shared_values_report_their_own_findings() {
    let json = item(
        "1.26.30",
        "demo:x",
        r#""minecraft:shooter": {"ammunition": [{"item": 5}]}, "minecraft:use_modifiers": {"use_duration": 1.0}, "minecraft:durability": {"max_durability": 5, "damage_chance": {"min": 50, "max": 10}}"#,
    );
    let document = loads(&json, &third_party());
    let d = document.diagnostics();
    let shape = d
        .iter()
        .find(|f| f.code.name() == "InvalidType")
        .unwrap_or_else(|| panic!("{d:#?}"));
    assert_eq!(
        shape.location.json_path.to_string(),
        "minecraft:item/components/minecraft:shooter/ammunition/0/item"
    );
    // a swapped range is an info
    assert!(d.iter().any(|f| f.code.name() == "InvalidValue"
        && f.severity == Severity::Info
        && f.location.json_path.to_string().ends_with("damage_chance")));
}

#[test]
fn rule_12_an_unknown_enchant_slot_refuses_the_item() {
    use bedrock_addon::families::item::component::Enchantable;
    // The slot is bound with a string regex constraint (1.20.50 enchantable
    // component), whose failure is cereal's 0x40 ("expected string to match the … regex"): an
    // error under both masks, and the item is refused. The game's own `none` text is reached
    // only by a
    // name the regex accepts and `enchantSlotFromString` does not know.
    for version in ["1.26.30", "1.21.0"] {
        let json = item(
            version,
            "demo:x",
            r#""minecraft:enchantable": {"value": 10, "slot": "elbow"}"#,
        );
        let d = rejects(&json, &third_party());
        assert!(
            error_at(
                &d,
                "InvalidValue",
                "minecraft:item/components/minecraft:enchantable/slot"
            ),
            "{version}: {d:#?}"
        );
    }
    let sword = item(
        "1.26.30",
        "demo:x",
        r#""minecraft:enchantable": {"value": 10, "slot": "sword"}"#,
    );
    assert_eq!(
        loads(&sword, &third_party())
            .components
            .get::<Enchantable>()
            .unwrap()
            .slot,
        "sword"
    );
}

#[test]
fn rule_1_an_identifier_needs_a_namespace() {
    for identifier in ["ruby", ":ruby", "demo:"] {
        let d = rejects(
            &item("1.26.30", identifier, r#""minecraft:max_stack_size": 1"#),
            &third_party(),
        );
        assert!(
            d.iter()
                // By legacy name: the variant and the definitions that were it.
                .any(|f| matches!(f.code.name(), "NamespaceRule")
                    && f.severity == Severity::Error
                    && f.code.to_string().contains("must include a namespace")),
            "{identifier}: {d:#?}"
        );
    }
    let legacy = rejects(&item("1.10", "ruby", ""), &third_party());
    // By legacy name: the variant and the definitions that were it.
    assert!(has(&legacy, |c| matches!(c.name(), "NamespaceRule")));
}

#[test]
fn rule_19a_legacy_components_must_be_an_object() {
    let json = r#"{"format_version": "1.10", "minecraft:item": {"description": {"identifier": "demo:x"}, "components": [1, 2]}}"#;
    let d = rejects(json, &third_party());
    assert!(
        has_text(&d, "Item 'components' should be a json object"),
        "{d:#?}"
    );
    // a missing components object is fine
    loads(
        r#"{"format_version": "1.10", "minecraft:item": {"description": {"identifier": "demo:x"}}}"#,
        &third_party(),
    );
}

// The link reads the crop against the block registry: with `item` alone there is none.
#[cfg(feature = "block")]
#[test]
fn rule_19d_an_unknown_crop_result_is_an_information() {
    // A legacy seed's `crop_result` names a block of the block registry
    // or the built-ins; an unknown name is vanilla's warning and the item still loads.
    const BAD: &[u8] = br#"{"format_version": "1.10", "minecraft:item": {"description": {"identifier": "demo:bad_seeds"}, "components": {"minecraft:seed": {"crop_result": "demo:not_a_block"}}}}"#;
    const BUILT_IN: &[u8] = br#"{"format_version": "1.10", "minecraft:item": {"description": {"identifier": "demo:wheat_seeds"}, "components": {"minecraft:seed": {"crop_result": "wheat"}}}}"#;
    const CUSTOM: &[u8] = br#"{"format_version": "1.10", "minecraft:item": {"description": {"identifier": "demo:crop_seeds"}, "components": {"minecraft:seed": {"crop_result": "demo:crop"}}}}"#;
    const CROP: &[u8] = br#"{"format_version": "1.21.0", "minecraft:block": {"description": {"identifier": "demo:crop"}, "components": {}}}"#;
    let set = stack_set(
        &bp(1)
            .file("items/bad_seeds.json", BAD)
            .file("items/wheat_seeds.json", BUILT_IN)
            .file("items/crop_seeds.json", CUSTOM)
            .file("blocks/crop.json", CROP),
        &rp(2),
    );
    // the item loads: the warning does not reject it
    assert_eq!(set.behavior.items().len(), 3);
    let d = set.link();
    // The game's text goes to the developer log alone: an information of ours.
    assert_eq!(
        link_messages(
            &d,
            "UnresolvedReference",
            Severity::Info,
            "items/bad_seeds.json"
        ),
        [
            "Item[demo:bad_seeds] Seed Component : Failed to load crop_result[demo:not_a_block]. Are you using valid block names?"
        ],
        "{d:#?}"
    );
    for resolved in ["items/wheat_seeds.json", "items/crop_seeds.json"] {
        assert!(
            link_messages(&d, "UnresolvedReference", Severity::Info, resolved).is_empty(),
            "{resolved}: {d:#?}"
        );
    }
}

#[test]
fn a_legacy_creator_component_without_a_release_has_our_text() {
    let d = rejects(
        &item(
            "1.20.0",
            "demo:x",
            r#""minecraft:chargeable": {"movement_modifier": 0.5}"#,
        ),
        &third_party(),
    );
    assert!(
        has_text(
            &d,
            "Item component 'minecraft:chargeable' is only available to legacy vanilla packs"
        ),
        "{d:#?}"
    );
}

/// The same document in the band 1.21.60 to 1.21.109, where an item without a category is
/// told of it: the doubled members say nothing of their own, and the category is told once.
#[test]
fn a_member_written_twice_in_the_undefined_category_band_is_told_of_the_category_alone() {
    let text = r#"{"format_version": "1.21.60", "minecraft:item": {"description": {"identifier": "demo:first", "identifier": "demo:last"}, "components": {"minecraft:max_stack_size": 16}, "components": {"minecraft:max_stack_size": 32}}}"#;
    let item = super::common::loads(text, &super::common::third_party());
    assert_eq!(item.description.identifier.as_str(), "demo:last");
    let told: Vec<String> = super::common::above_info(item.diagnostics())
        .iter()
        .map(|f| {
            format!(
                "{} {:?} at {}",
                f.code.name(),
                f.severity,
                f.location.json_path
            )
        })
        .collect();
    // The category's finding is an information: nothing above it.
    assert_eq!(told, Vec::<String>::new());
}

// Of a member written twice the last value stands, whole, at every level: a doubled identifier is no "duplicate field" error.
#[test]
fn a_member_written_twice_keeps_its_last_value() {
    // (1.21.110: below it an item without a category is told of that, `v08_menu_category`.)
    let text = r#"{"format_version": "1.21.110", "minecraft:item": {"description": {"identifier": "demo:first", "identifier": "demo:last"}, "components": {"minecraft:max_stack_size": 16}, "components": {"minecraft:max_stack_size": 32}}}"#;
    let item = super::common::loads(text, &super::common::third_party());
    assert_eq!(item.description.identifier.as_str(), "demo:last");
    let written = bedrock_addon::document::Content::to_json(item.value()).unwrap();
    assert!(
        written.contains("32") && !written.contains("16"),
        "{written}"
    );
    assert!(
        super::common::above_info(item.diagnostics()).is_empty(),
        "{:#?}",
        item.diagnostics()
    );
}

// A doubled root
// `use_beta_features` is one root member nothing reads, told once.
#[test]
fn a_doubled_use_beta_features_is_read_as_its_last_copy() {
    let text = r#"{"format_version": "1.21.60", "use_beta_features": true, "use_beta_features": false, "minecraft:item": {"description": {"identifier": "demo:x"}, "components": {}}}"#;
    let item = super::common::loads(text, &super::common::third_party());
    // An information of ours: the game says nothing of it.
    let told: Vec<String> = super::common::unknown_members(item.diagnostics())
        .iter()
        .map(|f| {
            format!(
                "{} {} {:?}",
                f.location.json_path,
                f.code.name(),
                f.severity
            )
        })
        .collect();
    assert_eq!(told, ["use_beta_features UnrecognizedField Info"]);
}

/// The stacks of behaviour pack `behavior` and resource pack `resource`, both enabled.
fn stack_set(behavior: &Spec, resource: &Spec) -> StackSet {
    let mut repository = PackRepository::new();
    repository.add_pack(behavior.open(), PackOrigin::World);
    repository.add_pack(resource.open(), PackOrigin::World);
    let selection = |spec: &Spec| {
        let id = spec.id();
        PackSelection {
            pack_id: id.uuid,
            version: id.version,
            subpack: None,
        }
    };
    let world = WorldPacks {
        behavior: vec![selection(behavior)],
        resource: vec![selection(resource)],
    };
    repository.stacks(&world, LoadOptions::default())
}

/// The messages of the link findings of code `name` and `severity` at the file `path`.
fn link_messages(d: &Diagnostics, name: &str, severity: Severity, path: &str) -> Vec<String> {
    d.iter()
        .filter(|f| {
            f.code.name() == name
                && f.severity == severity
                && f.location.path.as_ref().is_some_and(|p| p.as_str() == path)
        })
        .map(|f| f.code.to_string())
        .collect()
}

// --- plain scalars: unverified, lenient ----------------------------------------------

/// The item components hold plain `String`, integer, float and bool members, which the typed
/// reader fills from any scalar it can convert ("typed coercions"): `"5"` into an
/// integer, `1e1` into an integer, `5` into a string. The item folder gives each member's
/// type and says nothing about a value of another JSON type, so the read stays lenient;
/// what is read as a value of another JSON type than the file writes is told at
/// the member. The typed read fails a value of another JSON type
/// and the item is refused at the member; the spelling `1e1` of an integer is no other type.
/// This pins what is read and what is said.
#[test]
fn a_scalar_of_another_json_type_refuses_the_item_at_the_member() {
    let cx = third_party();
    for (components, written, told_at) in [
        (
            r#""minecraft:durability": {"max_durability": "5"}"#,
            r#""max_durability": 5"#,
            Some("minecraft:durability/max_durability"),
        ),
        (
            // The number the token spells, in the member's type: nothing is changed.
            r#""minecraft:durability": {"max_durability": 1e1}"#,
            r#""max_durability": 10"#,
            None,
        ),
        (
            r#""minecraft:display_name": {"value": 5}"#,
            r#""value": "5""#,
            Some("minecraft:display_name/value"),
        ),
        (
            r#""minecraft:icon": {"textures": {"default": 5}}"#,
            r#""default": "5""#,
            Some("minecraft:icon/textures/default"),
        ),
        (
            r#""minecraft:cooldown": {"category": 5, "duration": 1.0}"#,
            r#""category": "5""#,
            Some("minecraft:cooldown/category"),
        ),
        (
            r#""minecraft:compostable": {"composting_chance": "5"}"#,
            r#""composting_chance": 5"#,
            Some("minecraft:compostable/composting_chance"),
        ),
    ] {
        let text = item("1.21.60", "demo:x", components);
        if let Some(at) = told_at {
            // The typed read fails a value of another type (`0x8`, an error in both masks):
            // the item is refused whole (converted with a warning before).
            let d = rejects(&text, &cx);
            let wrong_type: Vec<(String, Severity)> = with_code(&d, "InvalidType")
                .iter()
                .map(|d| (d.location.json_path.to_string(), d.severity))
                .collect();
            assert_eq!(
                wrong_type,
                [(format!("minecraft:item/components/{at}"), Severity::Error)],
                "{components}"
            );
            continue;
        }
        let document = loads(&text, &cx);
        assert!(
            with_code(document.diagnostics(), "InvalidType").is_empty(),
            "{components}"
        );
        let text = bedrock_addon::document::Content::to_json(&*document).unwrap();
        assert!(text.contains(written), "{components}: {text}");
    }
    // What no conversion exists for is still refused: a fraction into an integer.
    let fraction = item(
        "1.21.60",
        "demo:x",
        r#""minecraft:durability": {"max_durability": 1.5}"#,
    );
    let found = findings(&fraction, &cx);
    assert!(
        !found.iter().all(|d| d.severity < Severity::Warning),
        "{found:?}"
    );
}

// --- upgrade steps never fall back in silence -----------------------------------------

/// The 1.18 step retypes a legacy `repair_items` list. An entry it cannot convert (a number,
/// an object without `items`, `items` of another shape) is dropped with an `InvalidValue`
/// warning at the entry and the other entries are kept; a `repair_amount` beyond a float is
/// the largest finite float with the warning at the number. The whole list was dropped
/// without a finding (`retype(items).unwrap_or_default()`).
#[test]
fn a_legacy_repair_items_entry_the_step_cannot_convert_is_told_and_the_others_are_kept() {
    let cx = allowlisted();
    let list = "minecraft:item/components/minecraft:repairable/repair_items";
    for version in ["1.16.100", "1.17.0"] {
        for (entry, told_at, kept) in [
            (
                r#"{"items": ["minecraft:stick"], "repair_amount": 1e39}"#,
                "0/repair_amount",
                true,
            ),
            (
                r#"{"items": ["minecraft:stick"], "repair_amount": -1e39}"#,
                "0/repair_amount",
                true,
            ),
            (r#"{"items": 7, "repair_amount": 1}"#, "0", false),
            (r#"{"repair_amount": 1}"#, "0", false),
            ("7", "0", false),
            (r#"{"a": 1}"#, "0", false),
        ] {
            let components = format!(
                r#""minecraft:repairable": {{"repair_items": [{entry}, {{"items": ["minecraft:planks"], "repair_amount": 2}}]}}"#
            );
            if !kept {
                // An entry the 1.19.83 slice cannot read: `items` is required (0x80), a value of another
                // type 0x8; both refuse the item.
                let d = rejects(&item(version, "demo:x", &components), &cx);
                assert!(
                    error_at(&d, "InvalidValue", &format!("{list}/{told_at}")),
                    "{version} {entry}: {d:#?}"
                );
                continue;
            }
            let document = loads(&item(version, "demo:x", &components), &cx);
            let told: Vec<(String, Severity)> = with_code(document.diagnostics(), "InvalidValue")
                .iter()
                .map(|d| (d.location.json_path.to_string(), d.severity))
                .collect();
            // Beyond a double the core reader clamps and says so; beyond a float
            // alone the step's own note is an information.
            let severity = if entry.contains("1e999") {
                Severity::Warning
            } else {
                Severity::Info
            };
            assert_eq!(
                told,
                [(format!("{list}/{told_at}"), severity)],
                "{version} {entry}"
            );
            let text = bedrock_addon::document::Content::to_json(&*document).unwrap();
            assert!(
                text.contains("minecraft:planks"),
                "{version} {entry}: {text}"
            );
            assert_eq!(
                text.contains("minecraft:stick"),
                kept,
                "{version} {entry}: {text}"
            );
            if kept {
                assert!(text.contains("3.4028235e38"), "{version} {entry}: {text}");
            }
        }
        // A list that is no array fails the slice's read too (0x8): the item is refused.
        let components =
            r#""minecraft:repairable": {"repair_items": {"items": ["minecraft:stick"]}}"#;
        let d = rejects(&item(version, "demo:x", components), &cx);
        assert!(error_at(&d, "InvalidValue", list), "{version}: {d:#?}");
    }
}

/// the 1.20.20 step moves `description.category` to `menu_category.category`
/// A name that is no creative category gives none, as before, and is told
/// at the member: the value is dropped. A category the set holds says nothing.
#[test]
fn a_legacy_category_that_is_no_creative_category_is_dropped_and_told() {
    use bedrock_addon::diagnostic::Phase;
    let with_category = |category: &str| {
        item(
            "1.20.0",
            "demo:x",
            r#""minecraft:icon": {"texture": "stick"}"#,
        )
        .replace(
            r#""identifier": "demo:x""#,
            &format!(r#""identifier": "demo:x", "category": "{category}""#),
        )
    };
    let told = |category: &str| -> Vec<(String, Severity)> {
        loads(
            &with_category(category),
            &bedrock_addon::load::LoadContext::detached(),
        )
        .diagnostics()
        .iter()
        .filter(|f| {
            // By legacy name: the variant and the definitions that were it.
            f.phase == Phase::Upgrade && matches!(f.code.name(), "InvalidValue")
        })
        .map(|f| (f.location.json_path.to_string(), f.severity))
        .collect()
    };
    for category in ["Items", "tools"] {
        assert_eq!(
            told(category),
            [(
                "minecraft:item/description/category".to_owned(),
                Severity::Info
            )],
            "{category}"
        );
    }
    for category in ["items", "equipment", "nature", "construction"] {
        assert!(told(category).is_empty(), "{category}");
    }
}

/// what a step or the legacy reader of an allow-listed pack's item reads as another
/// value, or drops, is told at the member the file has; the value stays what the step made.
#[test]
fn a_legacy_member_a_step_drops_or_reads_as_another_value_is_told() {
    // (version, components, the member told, whether the code is InvalidType)
    let rows: [(&str, &str, &str, &str); 8] = [
        (
            "1.20.0",
            r#""minecraft:food": {"nutrition": 4, "saturation_modifier": 1e39}"#,
            "components/minecraft:food/saturation_modifier",
            "InvalidValue",
        ),
        (
            "1.20.0",
            r#""minecraft:food": {"nutrition": 4, "saturation_modifier": true}"#,
            "components/minecraft:food/saturation_modifier",
            "InvalidType",
        ),
        (
            "1.16.100",
            r#""minecraft:durability": {"damage_chance": {"min": 10, "max": 50}}"#,
            "components/minecraft:durability",
            "MissingRequiredField",
        ),
        (
            "1.20.40",
            r#""minecraft:use_duration": 4294967296, "minecraft:icon": {"texture": "stick"}"#,
            "components/minecraft:use_duration",
            "InvalidValue",
        ),
        (
            "1.21.50",
            r#""minecraft:custom_components": ["demo:shake", 7], "minecraft:icon": "stick""#,
            "components/minecraft:custom_components/1",
            "InvalidType",
        ),
        (
            "1.20.0",
            r#""minecraft:foil": 7, "minecraft:icon": {"texture": "stick"}"#,
            "components/minecraft:foil",
            "InvalidType",
        ),
        (
            "1.20.0",
            r#""minecraft:block": 7, "minecraft:icon": {"texture": "stick"}"#,
            "components/minecraft:block",
            "InvalidType",
        ),
        (
            "1.20.0",
            r#""minecraft:mining_speed": 1e39, "minecraft:icon": {"texture": "stick"}"#,
            "components/minecraft:mining_speed",
            "InvalidValue",
        ),
    ];
    for (version, components, member, code) in rows {
        let text = item(version, "demo:x", components);
        let found = findings(&text, &allowlisted());
        let at = format!("minecraft:item/{member}");
        let told: Vec<String> = found
            .iter()
            .filter(|f| f.location.json_path.to_string() == at && f.severity >= Severity::Info)
            .map(bedrock_addon::diagnostic::testing::legacy_debug)
            .collect();
        assert!(
            told.iter().any(|name| name.starts_with(code)),
            "{version} {components}: {told:?} at {at}; all: {found:#?}"
        );
        // A document the game refuses (the 1.18 durability without a maximum)
        // has nothing written.
        let Ok(document) = load(&text, &allowlisted()) else {
            continue;
        };
        // Nothing infinite is held: the item is written, or it is left without a component
        // and refused at `components`, or it holds a member the strict 1.26.50 format
        // does not bind and is refused there (the list entry `7`, which no
        // keyed custom component can name, stays in the side-car).
        if let Err(error) = document.to_json() {
            let at_components = document.components.iter().count() == 0
                && format!("{error:?}").contains(r#"Key("components")] }"#);
            assert!(
                at_components
                    || matches!(&error, bedrock_addon::json::EncodeError::NotRepresentable { at, .. }
                        if at.to_string() == "minecraft:item/__deprecated_components"),
                "{version} {components}: {error:?}"
            );
        }
    }
}

#[test]
fn a_root_use_beta_features_that_is_no_bool_is_a_member_nothing_reads() {
    // A root `use_beta_features` that is no bool is a root member nothing reads.
    let text = item("1.21.60", "demo:x", r#""minecraft:icon": "stick""#).replacen(
        '{',
        r#"{"use_beta_features": 7, "#,
        1,
    );
    let document = loads(&text, &allowlisted());
    assert!(
        document.diagnostics().iter().any(|f| {
            f.location.json_path.to_string() == "use_beta_features"
                // By legacy name: the variant and the definitions that were it.
                && matches!(f.code.name(), "UnrecognizedField")
        }),
        "{:#?}",
        document.diagnostics()
    );
}

#[test]
fn the_value_of_a_legacy_tag_key_with_members_is_told_as_a_note() {
    // The value of a legacy tag key is not read: an object with members is told as a note.
    let document = loads(
        &item(
            "1.20.40",
            "demo:x",
            r#""tag:demo:blunt": {"a": 1}, "tag:demo:plain": {}, "minecraft:icon": {"texture": "stick"}"#,
        ),
        &allowlisted(),
    );
    let notes: Vec<String> = document
        .diagnostics()
        .iter()
        .filter(|f| f.severity == Severity::Info)
        .map(|f| f.location.json_path.to_string())
        .collect();
    assert_eq!(notes, ["minecraft:item/components/tag:demo:blunt"]);
}

/// A component-based item whose every component the load drops holds none, and a text whose
/// `components` name no registered component is the legacy item format to the game
/// (`isComponentBasedItemSchema`): the document that was read has
/// no text. The load says so at `components`, and the writer refuses it there.
#[test]
fn a_component_based_item_whose_every_component_the_load_drops_is_told_and_not_written() {
    use bedrock_addon::document::Content;
    use bedrock_addon::json::EncodeError;
    let cx = third_party();
    let item = |components: &str| {
        format!(
            r#"{{"format_version": "1.21.10", "minecraft:item": {{"description": {{"identifier": "demo:x"}}, "components": {{{components}}}}}}}"#
        )
    };
    let told =
        |document: &bedrock_addon::document::Document<bedrock_addon::families::item::Item>| {
            document
                .diagnostics()
                .iter()
                .filter(|f| f.location.json_path.to_string() == "minecraft:item/components")
                .map(|f| format!("{} {:?}", f.code.name(), f.severity))
                .collect::<Vec<_>>()
        };
    // Its one component is dropped (an empty `absorbable_causes`, a check of ours whose
    // binding was not read: an information): component-based, and empty.
    let lone = loads(
        &item(r#""minecraft:damage_absorption": {"absorbable_causes": []}"#),
        &cx,
    );
    assert_eq!(lone.legacy.format, ItemFormat::DataDriven);
    assert_eq!(lone.components.iter().count(), 0);
    // the load's note of ours, an information
    assert_eq!(told(&lone), ["InvalidValue Info"]);
    match lone.to_json() {
        Err(EncodeError::NotRepresentable { at, .. }) => {
            assert_eq!(at.to_string(), "minecraft:item/components");
        }
        other => panic!("{other:?}"),
    }
    // A second component that is kept: nothing to tell, written, and read back as it is.
    let kept = loads(
        &item(
            r#""minecraft:damage_absorption": {"absorbable_causes": []}, "minecraft:max_stack_size": 16"#,
        ),
        &cx,
    );
    assert_eq!(told(&kept), Vec::<String>::new());
    let written = kept.to_json().expect("an item with a component is written");
    let again = loads(&written, &cx);
    assert_eq!(again.legacy.format, ItemFormat::DataDriven);
    assert_eq!(again.to_json().ok(), Some(written));
    // An item built in code without a component is another case: no key made it
    // component-based, and it is written as the legacy document it is.
    let new = bedrock_addon::families::item::Item::new(bedrock_addon::id::ItemId::parse("demo:x"));
    assert!(new.to_json().is_ok());
}

/// A root `use_beta_features` with comments around it (between its value and the comma that
/// follows, between the comma before it and its key, between its key and its colon) is one
/// root member nothing reads, told at its key; the item loads without the toggle.
#[test]
fn a_root_use_beta_features_is_taken_out_with_comments_around_it() {
    let rest = r#""format_version": "1.21.60", "minecraft:item": {"description": {"identifier": "demo:x"}, "components": {}}"#;
    for text in [
        format!(r#"{{"use_beta_features": true /* the toggle */, {rest}}}"#),
        format!("{{\"use_beta_features\": true // the toggle\n, {rest}}}"),
        format!(r#"{{{rest}, /* a, b */ "use_beta_features": true}}"#),
        format!("{{{rest}, // a, b\n \"use_beta_features\": true /* c */}}"),
        format!(r#"{{"use_beta_features" /* "use_beta_features" */ : true, {rest}}}"#),
    ] {
        let item = loads(&text, &third_party());
        // An information of ours: the game says nothing of it.
        let told: Vec<String> = unknown_members(item.diagnostics())
            .iter()
            .map(|f| {
                format!(
                    "{} {} {:?}",
                    f.location.json_path,
                    f.code.name(),
                    f.severity
                )
            })
            .collect();
        assert_eq!(told, ["use_beta_features UnrecognizedField Info"], "{text}");
    }
}

/// A root `use_beta_features` whose key is written with an escape is that member: read by
/// nothing, told under its name, and the item loads without the toggle.
#[test]
fn a_root_use_beta_features_written_with_an_escape_is_taken_out() {
    let text = r#"{"format_version": "1.21.60", "use_beta\u005ffeatures": true, "minecraft:item": {"description": {"identifier": "demo:x"}, "components": {}}}"#;
    let item = loads(text, &third_party());
    // An information of ours: the game says nothing of it.
    let told: Vec<String> = unknown_members(item.diagnostics())
        .iter()
        .map(|f| {
            format!(
                "{} {} {:?}",
                f.location.json_path,
                f.code.name(),
                f.severity
            )
        })
        .collect();
    assert_eq!(told, ["use_beta_features UnrecognizedField Info"]);
}

/// A modern item (1.26.30 here; 1.26.50 and beta in
/// `a_null_holds_in_every_modern_slice`) is read from a `MinecraftDocumentInput`, so a
/// member written as `null` fails its type and the item is rejected.
#[test]
fn a_null_member_of_a_modern_item_rejects_it() {
    let d = rejects(
        &item(
            "1.26.30",
            "demo:x",
            r#""minecraft:use_modifiers": {"use_duration": 1.0, "start_sound": null}"#,
        ),
        &third_party(),
    );
    assert!(
        d.iter().any(|f| f.severity == Severity::Error
            // By legacy name: the variant and the definitions that were it.
            && matches!(f.code.name(), "InvalidType")
            && f.location.json_path.to_string()
                == "minecraft:item/components/minecraft:use_modifiers/start_sound"),
        "{d:#?}"
    );
}

/// Below 1.26.30 the item's components are read through
/// the lenient JSON reader, where a member written as `null` is absent (a pin: unchanged).
#[test]
fn a_null_member_of_a_legacy_item_reads_as_absent() {
    let document = loads(
        &item(
            "1.20.50",
            "demo:x",
            r#""minecraft:use_modifiers": {"use_duration": 1.0, "start_sound": null}"#,
        ),
        &third_party(),
    );
    assert!(
        above_info(document.diagnostics()).is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    let modifiers = document.components.get::<UseModifiers>().unwrap();
    assert_eq!(modifiers.start_sound, None);
}

/// The legacy chain's components are read through the lenient JSON reader; a component
/// written as `null` is absent, with no word.
#[test]
fn a_null_component_of_a_legacy_item_is_absent() {
    let document = loads(
        &item(
            "1.20.50",
            "demo:x",
            r#""minecraft:tags": null, "minecraft:icon": "stick""#,
        ),
        &third_party(),
    );
    assert!(
        above_info(document.diagnostics()).is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    assert!(
        document
            .components
            .get::<bedrock_addon::families::item::component::Tags>()
            .is_none()
    );
}

/// Whether `d` has an error with code `name` at `path`.
fn error_at(d: &bedrock_addon::diagnostic::Diagnostics, name: &str, path: &str) -> bool {
    d.iter().any(|f| {
        f.severity == Severity::Error
            && f.code.name() == name
            && f.location.json_path.to_string() == path
    })
}

/// Every modern slice (1.26.30, 1.26.50, beta) reads
/// through the strict document input: a `null` where the item reads a value is a type error
/// that rejects it, in any component, in the description and as a component's whole value.
#[test]
fn a_null_holds_in_every_modern_slice() {
    let mut beta = third_party();
    beta.beta_apis = true;
    for version in ["1.26.30", "1.26.50", "beta"] {
        for (components, path) in [
            (
                r#""minecraft:use_modifiers": {"use_duration": 1.0, "start_sound": null}"#,
                "minecraft:item/components/minecraft:use_modifiers/start_sound",
            ),
            (
                r#""minecraft:display_name": {"value": null}, "minecraft:icon": "stick""#,
                "minecraft:item/components/minecraft:display_name/value",
            ),
            (
                r#""minecraft:tags": null, "minecraft:icon": "stick""#,
                "minecraft:item/components/minecraft:tags",
            ),
        ] {
            // (`Item::parse` refuses a loaded beta document too, without a type error)
            let d = rejects(&item(version, "demo:x", components), &beta);
            assert!(
                error_at(&d, "InvalidType", path),
                "{version} {path}: {d:#?}"
            );
        }
        let text = item(version, "demo:x", r#""minecraft:icon": "stick""#).replace(
            r#""identifier": "demo:x""#,
            r#""identifier": "demo:x", "menu_category": null"#,
        );
        let d = rejects(&text, &beta);
        assert!(
            error_at(
                &d,
                "InvalidType",
                "minecraft:item/description/menu_category"
            ),
            "{version}: {d:#?}"
        );
    }
}

/// The modern slices are registered with the strict error codes
/// 0xfffc, so an unknown member or component of the payload is an error that rejects the item;
/// the legacy slice reads them as before (a warning from 1.20.60, nothing below).
#[test]
fn o5_an_unknown_member_rejects_a_modern_item_and_not_a_legacy_one() {
    for (components, path, name) in [
        (
            r#""minecraft:tags": {"tags": ["demo:a"], "note": 1}"#,
            "minecraft:item/components/minecraft:tags/note",
            "UnrecognizedField",
        ),
        (
            r#""minecraft:icon": "stick", "minecraft:sparkle": {}"#,
            "minecraft:item/components/minecraft:sparkle",
            "UnrecognizedComponent",
        ),
    ] {
        for version in ["1.26.30", "1.26.50"] {
            let d = rejects(&item(version, "demo:x", components), &third_party());
            assert!(error_at(&d, name, path), "{version} {path}: {d:#?}");
        }
        let document = loads(&item("1.21.90", "demo:x", components), &third_party());
        // A component name no format ever had is removed without a game line: an information;
        // an unknown member is cereal's 0x4, a level-2 line: a warning.
        let legacy = if name == "UnrecognizedComponent" {
            Severity::Info
        } else {
            Severity::Warning
        };
        assert!(
            document.diagnostics().iter().any(|f| f.severity == legacy
                && f.code.name() == name
                && f.location.json_path.to_string() == path),
            "{path}: {:#?}",
            document.diagnostics()
        );
    }
    // a root member is no member of the payload: still a warning
    let text = item("1.26.50", "demo:x", r#""minecraft:icon": "stick""#).replacen(
        '{',
        r#"{"note": 1, "#,
        1,
    );
    loads(&text, &third_party());
}

/// `events` is no member of the modern documents, an unknown
/// member that the strict slice refuses, at 1.26.30 and at 1.26.50; at 1.26.0 the legacy chain
/// reads it with no parse finding; the one word is the load's note that the newest format
/// does not hold it (a `Validate` warning).
#[test]
fn events_is_an_unknown_member_of_the_modern_documents_only() {
    let with_events = |version: &str| {
        item(version, "demo:x", r#""minecraft:icon": "x""#).replacen(
            r#""components""#,
            r#""events": {"demo:e": {}}, "components""#,
            1,
        )
    };
    for version in ["1.26.30", "1.26.50"] {
        let d = rejects(&with_events(version), &third_party());
        assert!(
            error_at(&d, "UnrecognizedField", "minecraft:item/events"),
            "{version}: {d:#?}"
        );
    }
    let legacy = loads(&with_events("1.26.0"), &third_party());
    let told: Vec<String> = legacy
        .diagnostics()
        .iter()
        .filter(|f| f.code.name() == "InvalidValue")
        .map(|f| {
            format!(
                "{} {:?} {:?} {}",
                f.code.name(),
                f.severity,
                f.phase,
                f.location.json_path
            )
        })
        .collect();
    assert_eq!(told, ["InvalidValue Info Validate minecraft:item/events"]);
    assert!(legacy.events.is_some());
}

/// The bounds of the slices the null rule and the strict mask start at 1.26.29 →
/// 1.26.30 (`FIRST_MODERN_SLICE`); the sound members take any name from 1.26.50, not at
/// 1.26.49.
#[test]
fn the_slice_rules_change_at_their_bounds() {
    let null = r#""minecraft:use_modifiers": {"use_duration": 1.0, "start_sound": null}"#;
    loads(&item("1.26.29", "demo:x", null), &third_party());
    let d = rejects(&item("1.26.30", "demo:x", null), &third_party());
    assert!(
        error_at(
            &d,
            "InvalidType",
            "minecraft:item/components/minecraft:use_modifiers/start_sound"
        ),
        "{d:#?}"
    );
    let unknown = r#""minecraft:tags": {"tags": ["demo:a"], "note": 1}"#;
    let older = loads(&item("1.26.29", "demo:x", unknown), &third_party());
    assert!(
        older
            .diagnostics()
            .iter()
            .any(|f| f.severity == Severity::Warning && f.code.name() == "UnrecognizedField")
    );
    let d = rejects(&item("1.26.30", "demo:x", unknown), &third_party());
    assert!(
        error_at(
            &d,
            "UnrecognizedField",
            "minecraft:item/components/minecraft:tags/note"
        ),
        "{d:#?}"
    );
    let sound = r#""minecraft:use_modifiers": {"use_duration": 1.0, "start_sound": "demo.custom"}"#;
    let d = rejects(&item("1.26.49", "demo:x", sound), &third_party());
    assert!(
        error_at(
            &d,
            "InvalidEnumValue",
            "minecraft:item/components/minecraft:use_modifiers/start_sound"
        ),
        "{d:#?}"
    );
    loads(&item("1.26.50", "demo:x", sound), &third_party());
}

/// The legacy chain (1.16.100 … 1.26.0) reads its components through cereal
/// slices with mask 0xfbf8, under which a value
/// of the wrong type (0x8) and a missing required member (0x80) are errors like the strict
/// slices'; the loader then fails and logs " Error Parsing Item". The item
/// is refused, whatever the slice.
#[test]
fn e1_a_component_of_the_wrong_shape_refuses_the_item_in_every_slice() {
    for (version, components, code, at) in [
        // `minecraft:interact_button`: a bool or a string (document.rs and look.rs)
        (
            "1.21.0",
            r#""minecraft:interact_button": 7"#,
            "InvalidType",
            "minecraft:interact_button",
        ),
        (
            "1.26.30",
            r#""minecraft:interact_button": 7"#,
            "InvalidType",
            "minecraft:interact_button",
        ),
        // a bare scalar of the wrong type (document.rs `value` and component/value.rs)
        (
            "1.21.0",
            r#""minecraft:max_stack_size": "x""#,
            "InvalidType",
            "minecraft:max_stack_size",
        ),
        (
            "1.26.30",
            r#""minecraft:max_stack_size": "x""#,
            "InvalidType",
            "minecraft:max_stack_size",
        ),
        // the custom component list of 1.20.80–1.21.50, and the entries of 1.21.60–1.26.0
        (
            "1.21.30",
            r#""minecraft:custom_components": 7"#,
            "InvalidType",
            "minecraft:custom_components",
        ),
        (
            "1.21.60",
            r#""minecraft:custom_components": 7"#,
            "InvalidType",
            "minecraft:custom_components",
        ),
        (
            "1.21.60",
            r#""minecraft:custom_components": [7]"#,
            "InvalidType",
            "minecraft:custom_components/0",
        ),
    ] {
        let d = rejects(&item(version, "demo:x", components), &third_party());
        assert!(
            error_at(&d, code, &format!("minecraft:item/components/{at}")),
            "{version} {components}: {d:#?}"
        );
    }
    // A 1.18-shaped durability without a maximum: `max_durability` is required by every
    // slice 0x80, refused; the
    // 1.18 fixer gives a pre-1.18 document nothing to rename (the step's own refusal).
    for version in ["1.16.100", "1.20.0"] {
        let d = rejects(
            &item(
                version,
                "minecraft:x",
                r#""minecraft:durability": {"damage_chance": {"min": 1, "max": 2}}"#,
            ),
            &allowlisted(),
        );
        assert!(
            d.iter().any(|f| f.code.name() == "MissingRequiredField"
                && f.severity == Severity::Error
                && f.location
                    .json_path
                    .to_string()
                    .starts_with("minecraft:item/components/minecraft:durability")),
            "{version}: {d:#?}"
        );
    }
    // A list element that is no string reads as the name facet makes of it (ours: told as
    // an information, the slice's coercion of a number to a string not read).
    let document = loads(
        &item(
            "1.21.30",
            "demo:x",
            r#""minecraft:custom_components": ["demo:a", 7]"#,
        ),
        &third_party(),
    );
    assert!(
        document
            .diagnostics()
            .iter()
            .any(|f| f.code.name() == "InvalidType"
                && f.severity == Severity::Info
                && f.location.json_path.to_string()
                    == "minecraft:item/components/minecraft:custom_components/1"),
        "{:#?}",
        document.diagnostics()
    );
}
