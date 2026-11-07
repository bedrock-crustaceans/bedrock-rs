//! The components' values, read in a legacy component document, a 1.21.60 one, a 1.26.0 one
//! and the two modern formats from every JSON type at their scalar places, bare or in the
//! object form, and at both sides of each bound the family checks. Each line pins what the
//! load says (code, severity, path, span) and, when the item loads, the component the newest
//! struct writes back. The one-value components and the bounds were checked by hand; this table
//! is what they read, and the schema derive reads them the same.

use super::common::{allowlisted, item, load, newest_payload};

/// The component values the table reads, per component key.
const VALUES: &[(&str, &[&str])] = &[
    (
        "minecraft:allow_off_hand",
        &[
            "true",
            "1",
            "\"true\"",
            r#"{"value": true}"#,
            r#"{"value": 1}"#,
            r#"{"value": "x"}"#,
            "{}",
            "null",
        ],
    ),
    (
        "minecraft:can_destroy_in_creative",
        &["false", "5", r#"{"value": false}"#, "null"],
    ),
    (
        "minecraft:hand_equipped",
        &["true", "0.5", r#"{"value": true, "zz": 1}"#, "null"],
    ),
    ("minecraft:liquid_clipped", &["true", "[]", "null"]),
    (
        "minecraft:should_despawn",
        &["false", r#"{"value": "false"}"#, "1", "null"],
    ),
    ("minecraft:stacked_by_data", &["true", "{}", "1", "null"]),
    (
        "minecraft:fire_resistant",
        &[r#"{"value": true}"#, "true", r#"{"value": 1}"#, "{}"],
    ),
    (
        "minecraft:max_stack_size",
        &[
            "16",
            "0",
            "1",
            "64",
            "65",
            "1.5",
            "\"16\"",
            "true",
            r#"{"value": 16}"#,
            r#"{"value": 65}"#,
            r#"{"value": -1}"#,
            "{}",
            "null",
        ],
    ),
    (
        "minecraft:damage",
        &[
            "5",
            "-1",
            "32767",
            "40000",
            "1.5",
            r#"{"value": 5}"#,
            r#"{"value": -1}"#,
            "\"5\"",
            "true",
            "null",
        ],
    ),
    (
        "minecraft:display_name",
        &[r#"{"value": "x"}"#, "\"x\"", r#"{"value": 5}"#],
    ),
    (
        "minecraft:icon",
        &[
            "\"x\"",
            r#"{"textures": {"default": "x"}}"#,
            "5",
            r#"{"texture": "x"}"#,
        ],
    ),
    (
        "minecraft:fuel",
        &[
            "1",
            "0.05",
            "0.01",
            "-1",
            r#"{"duration": 2}"#,
            r#"{"duration": 0.01}"#,
            "\"1\"",
            "{}",
            "true",
            "null",
        ],
    ),
    (
        "minecraft:use_animation",
        &["\"eat\"", "\"x\"", "5", r#"{"value": "eat"}"#, "{}", "null"],
    ),
    (
        "minecraft:swing_duration",
        &[
            "0.5",
            "-1",
            r#"{"value": -1}"#,
            r#"{"value": 0}"#,
            "{}",
            "true",
            "null",
        ],
    ),
    (
        "minecraft:glint",
        &["true", "1", r#"{"value": false}"#, "null"],
    ),
    (
        "minecraft:hover_text_color",
        &["\"red\"", "5", r#"{"value": "red"}"#, "null"],
    ),
    (
        "minecraft:rarity",
        &["\"rare\"", "\"x\"", r#"{"value": "epic"}"#, "5", "null"],
    ),
    (
        "minecraft:durability",
        &[
            r#"{"max_durability": 5}"#,
            r#"{"max_durability": -1}"#,
            r#"{"max_durability": 5, "damage_chance": {"min": -1, "max": 5}}"#,
            r#"{"max_durability": 5, "damage_chance": {"min": 1, "max": 101}}"#,
            r#"{"max_durability": 1.5}"#,
        ],
    ),
    (
        "minecraft:enchantable",
        &[
            r#"{"value": 5, "slot": "sword"}"#,
            r#"{"value": -1, "slot": "sword"}"#,
            r#"{"value": 5, "slot": "x"}"#,
            r#"{"value": "5", "slot": "sword"}"#,
        ],
    ),
    (
        "minecraft:use_modifiers",
        &[
            r#"{"use_duration": 1}"#,
            r#"{"use_duration": -1}"#,
            r#"{"use_duration": 1, "movement_modifier": 2}"#,
            r#"{"use_duration": -1, "movement_modifier": 2}"#,
            r#"{"use_duration": 1, "movement_modifier": -0.5}"#,
        ],
    ),
    (
        "minecraft:wearable",
        &[
            r#"{"slot": "slot.armor.head", "protection": 2}"#,
            r#"{"slot": "slot.armor.head", "protection": -1}"#,
            r#"{"slot": "slot.armor.head", "protection": 1.5}"#,
        ],
    ),
    (
        "minecraft:kinetic_weapon",
        &[
            r#"{"damage_conditions": {"min_speed": 1}}"#,
            r#"{"damage_conditions": {"min_speed": -1}}"#,
            r#"{"knockback_conditions": {"min_relative_speed": -1}}"#,
            r#"{"dismount_conditions": {"max_duration": -1}}"#,
        ],
    ),
    (
        "minecraft:compostable",
        &[
            r#"{"composting_chance": 50}"#,
            r#"{"composting_chance": 0}"#,
            r#"{"composting_chance": 101}"#,
            r#"{"composting_chance": 100}"#,
        ],
    ),
    (
        "minecraft:storage_item",
        &[
            r#"{"max_slots": 64}"#,
            r#"{"max_slots": 65}"#,
            r#"{"max_slots": 5, "max_weight_limit": 65}"#,
            r#"{"max_slots": 5, "weight_in_storage_item": -1}"#,
            "{}",
        ],
    ),
    (
        "minecraft:storage_weight_limit",
        &[
            r#"{"max_weight_limit": 64}"#,
            r#"{"max_weight_limit": 65}"#,
            "{}",
        ],
    ),
    (
        "minecraft:storage_weight_modifier",
        &[
            r#"{"weight_in_storage_item": 0}"#,
            r#"{"weight_in_storage_item": -1}"#,
            "{}",
        ],
    ),
    (
        "minecraft:bundle_interaction",
        &[
            r#"{"num_viewable_slots": 0}"#,
            r#"{"num_viewable_slots": 1}"#,
            r#"{"num_viewable_slots": 65}"#,
            "{}",
        ],
    ),
    (
        "minecraft:cooldown",
        &[
            r#"{"category": "a", "duration": 1}"#,
            r#"{"category": "a", "duration": -1}"#,
            r#"{"category": 5, "duration": 1}"#,
        ],
    ),
    (
        "minecraft:use_duration",
        &["5", "-1", r#"{"value": -1}"#, r#"{"value": 2}"#],
    ),
];

/// The finding as the table pins it: the legacy `Debug` of its code, with the item's two
/// structured texts (the slot of `minecraft:enchantable`, the wearable's stack size) written in
/// the legacy payload's shape, which the shared rendering knows only for core codes.
fn pinned_debug(d: &bedrock_addon_core::diagnostic::Diagnostic) -> String {
    use bedrock_addon_family_item::codes;
    if let Some([slot]) = d.code.args_of(&codes::ENCHANT_SLOT) {
        return format!("InvalidValue {{ text: Some(ItemEnchantSlot {{ slot: {slot:?} }}) }}");
    }
    if let Some([component, slot]) = d.code.args_of(&codes::WEARABLE_STACK_SIZE) {
        return format!(
            "InvalidValue {{ text: Some(ItemWearableStackSize {{ component: {component:?}, slot: {slot:?} }}) }}"
        );
    }
    bedrock_addon_core::diagnostic::testing::legacy_debug(d)
}

/// One line of the table: the version, the component, the value, and what the load gave.
fn line(version: &str, key: &str, value: &str) -> String {
    let text = item(version, "demo:x", &format!(r#""{key}": {value}"#));
    let (loaded, diagnostics) = match load(&text, &allowlisted()) {
        Ok(document) => {
            let payload: bedrock_addon_core::json::Value =
                bedrock_addon_core::json::from_str(&newest_payload(&document)).expect("JSON");
            let written = payload
                .as_object()
                .and_then(|p| p.get("components"))
                .and_then(|c| c.as_object())
                .and_then(|c| c.get(key))
                .map(|v| bedrock_addon_core::json::to_facet_json(v).unwrap_or_default());
            (
                format!("loaded {}", written.unwrap_or_else(|| "-".to_owned())),
                document.diagnostics().clone(),
            )
        }
        Err(rejected) => ("rejected".to_owned(), rejected.diagnostics),
    };
    let findings: Vec<String> = diagnostics
        .iter()
        .map(|d| {
            let span = d
                .location
                .span
                .map_or("-".to_owned(), |s| format!("{}+{}", s.offset, s.len));
            format!(
                "{} {:?} {} {span}",
                pinned_debug(d),
                d.severity,
                d.location.json_path
            )
        })
        .collect();
    format!(
        "{version} {key} {value} => {loaded} [{}]",
        findings.join("; ")
    )
}

#[test]
fn the_components_read_every_value_as_pinned() {
    let mut table = String::new();
    for version in [
        "1.20.50", "1.21.30", "1.21.60", "1.26.0", "1.26.30", "1.26.50",
    ] {
        for (key, values) in VALUES {
            for value in *values {
                table.push_str(&line(version, key, value));
                table.push('\n');
            }
        }
    }
    let expected = include_str!("component_values.txt");
    assert!(table == expected, "--- actual ---\n{table}--- end ---");
}
