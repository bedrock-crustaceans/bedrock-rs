//! For the block family: a member the author did not write is not written back, and one
//! the author wrote at its default is.

#![allow(
    clippy::disallowed_methods,
    reason = "calls facet-json directly: parses a fixture or written text into a json Value to compare it and serializes a value to compare it with the written text; not yet moved to the door"
)]

use bedrock_addon_core::json::Value;
use bedrock_addon_core::load::{LoadContext, LoadOptions};
use bedrock_addon_core::pack_identity::PackType;
use bedrock_addon_core::version::MinEngineVersion;
use bedrock_addon_family_block::component::{
    BlockEntity, CollisionBox, CraftingTable, EntityFallOn, Flammable, Leashable, LiquidDetection,
    RandomOffset, RedstoneConductivity, RedstoneConsumer, RedstoneProducer, Tick,
};

use super::common::loads;

/// A base-game behaviour pack for engine 1.26.50 with Upcoming Creator Features on: every
/// component and `internal_vanilla_data` load.
fn everything() -> LoadContext {
    let options = LoadOptions {
        upcoming_creator_features: true,
        ..LoadOptions::default()
    };
    let mut cx = LoadContext::for_pack(
        &options,
        uuid::Uuid::from_u128(7),
        PackType::Behavior,
        Some(MinEngineVersion::new(1, 26, 50)),
        true,
    );
    cx.molang_version =
        bedrock_addon_core::molang::pack_molang_version(cx.min_engine_version.as_ref());
    cx
}

/// `value` as compact JSON text with every object's members sorted by key.
fn sorted(value: &Value) -> String {
    if let Some(object) = value.as_object() {
        let mut members: Vec<_> = object.iter().collect();
        members.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));
        let inner: Vec<String> = members
            .into_iter()
            .map(|(k, v)| format!("{:?}:{}", k.as_str(), sorted(v)))
            .collect();
        format!("{{{}}}", inner.join(","))
    } else if let Some(array) = value.as_array() {
        let inner: Vec<String> = array.iter().map(sorted).collect();
        format!("[{}]", inner.join(","))
    } else {
        facet_json::to_string(value).unwrap()
    }
}

/// A 1.26.50 block document with `description` members and `components`.
fn block(description: &str, components: &str) -> String {
    format!(
        r#"{{"format_version": "1.26.50", "minecraft:block": {{"description": {{"identifier": "minecraft:x"{description}}}, "components": {{{components}}}}}}}"#
    )
}

/// What the writer makes of `input`, and `input` itself, both with sorted members: equal when
/// the writer invents nothing and loses nothing.
fn rewritten(input: &str) -> (String, String) {
    let document = loads(input, &everything());
    let written = document.to_json().unwrap();
    let read = |text: &str| facet_json::from_str::<Value>(text).unwrap();
    (sorted(&read(&written)), sorted(&read(input)))
}

/// The documents whose optional members are all left out: `(what, description, components)`.
const SPARSE: [(&str, &str, &str); 19] = [
    (
        "redstone_conductivity",
        "",
        r#""minecraft:redstone_conductivity": {}"#,
    ),
    // One member written: the other is still left out.
    (
        "redstone_conductivity with redstone_conductor",
        "",
        r#""minecraft:redstone_conductivity": {"redstone_conductor": true}"#,
    ),
    (
        "redstone_consumer",
        "",
        r#""minecraft:redstone_consumer": {}"#,
    ),
    (
        "redstone_producer",
        "",
        r#""minecraft:redstone_producer": {"power": 5}"#,
    ),
    ("flammable", "", r#""minecraft:flammable": {}"#),
    (
        "random_offset",
        "",
        r#""minecraft:random_offset": {"x": {"range": {"min": 0, "max": 0}}}"#,
    ),
    (
        "tick",
        "",
        r#""minecraft:tick": {"interval_range": [10, 20]}"#,
    ),
    ("entity_fall_on", "", r#""minecraft:entity_fall_on": {}"#),
    ("leashable", "", r#""minecraft:leashable": {}"#),
    (
        "crafting_table",
        "",
        r#""minecraft:crafting_table": {"crafting_tags": ["demo"]}"#,
    ),
    (
        "placement_filter",
        "",
        r#""minecraft:placement_filter": {}"#,
    ),
    (
        "placement_filter condition",
        "",
        r#""minecraft:placement_filter": {"conditions": [{}]}"#,
    ),
    (
        "liquid_detection",
        "",
        r#""minecraft:liquid_detection": {"detection_rules": [{}]}"#,
    ),
    ("block_entity", "", r#""minecraft:block_entity": {}"#),
    ("selection_box", "", r#""minecraft:selection_box": {}"#),
    (
        "multi_block",
        r#", "traits": {"minecraft:multi_block": {"enabled_states": ["minecraft:multi_block_part"], "direction": "up"}}"#,
        r#""minecraft:movable": {"movement_type": "popped"}"#,
    ),
    (
        "internal_vanilla_data",
        r#", "internal_vanilla_data": {}"#,
        "",
    ),
    (
        "placement_direction",
        r#", "traits": {"minecraft:placement_direction": {"enabled_states": ["minecraft:cardinal_direction"]}}"#,
        "",
    ),
    (
        "geometry",
        "",
        r#""minecraft:geometry": {"identifier": "geometry.demo"}, "minecraft:material_instances": {"*": {"texture": "demo"}}"#,
    ),
];

#[test]
fn a_member_the_author_did_not_write_is_not_written_back() {
    let mut invented = Vec::new();
    for (what, description, components) in SPARSE {
        let (written, input) = rewritten(&block(description, components));
        if written != input {
            invented.push(format!("{what}:\n  input   {input}\n  written {written}"));
        }
    }
    assert!(
        invented.is_empty(),
        "the writer invented members:\n{}",
        invented.join("\n")
    );
}

#[test]
fn a_member_written_at_its_default_is_written_back() {
    let inputs = [
        block(
            "",
            r#""minecraft:redstone_conductivity": {"redstone_conductor": false, "allows_wire_to_step_down": true}"#,
        ),
        block(
            "",
            r#""minecraft:redstone_consumer": {"min_power": 0, "propagates_power": false}"#,
        ),
        block(
            "",
            r#""minecraft:redstone_producer": {"power": 5, "transform_relative": true}"#,
        ),
        block(
            "",
            r#""minecraft:flammable": {"catch_chance_modifier": 5, "destroy_chance_modifier": 20}"#,
        ),
        block(
            "",
            r#""minecraft:random_offset": {"x": {"range": {"min": 0, "max": 0}, "steps": 0}}"#,
        ),
        block(
            "",
            r#""minecraft:tick": {"interval_range": [10, 20], "looping": true}"#,
        ),
        block(
            "",
            r#""minecraft:entity_fall_on": {"min_fall_distance": 0}"#,
        ),
        block("", r#""minecraft:leashable": {"offset": [0, 0.25, 0]}"#),
        block(
            "",
            r#""minecraft:crafting_table": {"crafting_tags": ["demo"], "table_name": ""}"#,
        ),
        block(
            "",
            r#""minecraft:liquid_detection": {"detection_rules": [{"can_contain_liquid": false, "use_liquid_clipping": false}]}"#,
        ),
        block(
            "",
            r#""minecraft:block_entity": {"dynamic_properties": false}"#,
        ),
        block(
            r#", "traits": {"minecraft:multi_block": {"enabled_states": ["minecraft:multi_block_part"], "direction": "up", "parts": 2}}"#,
            r#""minecraft:movable": {"movement_type": "popped"}"#,
        ),
        block(r#", "internal_vanilla_data": {"block_id": 0}"#, ""),
    ];
    for input in inputs {
        let (written, input) = rewritten(&input);
        assert_eq!(written, input);
    }
}

/// The defaults vanilla applies to an absent member are the accessors'.
#[test]
fn an_absent_member_reads_as_its_default_through_the_accessor() {
    let all: String = SPARSE[..15]
        .iter()
        .filter(|(what, ..)| {
            !what.starts_with("placement_filter")
                && *what != "redstone_producer"
                && *what != "redstone_conductivity with redstone_conductor"
        })
        .map(|(_, _, component)| *component)
        .chain([r#""minecraft:collision_box": {}"#])
        .collect::<Vec<_>>()
        .join(", ");
    let document = loads(&block("", &all), &everything());
    let components = &document.value().components;

    let conductivity = components.get::<RedstoneConductivity>().unwrap();
    assert_eq!(conductivity.redstone_conductor, None);
    assert_eq!(conductivity.allows_wire_to_step_down, None);
    assert!(!conductivity.redstone_conductor());
    assert!(conductivity.allows_wire_to_step_down());

    let consumer = components.get::<RedstoneConsumer>().unwrap();
    assert_eq!(
        (consumer.min_power(), consumer.propagates_power()),
        (0, false)
    );
    let producer = loads(&block("", SPARSE[3].2), &everything());
    let producer = producer
        .value()
        .components
        .get::<RedstoneProducer>()
        .unwrap();
    assert_eq!(producer.transform_relative, None);
    assert!(producer.transform_relative());

    let Some(Flammable::Params(flammable)) = components.get::<Flammable>() else {
        panic!("the object form");
    };
    assert_eq!(flammable.catch_chance_modifier(), 5);
    assert_eq!(flammable.destroy_chance_modifier(), 20);

    let offset = components.get::<RandomOffset>().unwrap();
    assert_eq!(offset.x.as_ref().unwrap().steps(), 0);
    assert!(components.get::<Tick>().unwrap().looping());
    assert_eq!(
        components
            .get::<EntityFallOn>()
            .unwrap()
            .min_fall_distance(),
        0.0
    );
    assert_eq!(
        components.get::<Leashable>().unwrap().offset(),
        [0.0, 0.25, 0.0]
    );
    assert_eq!(components.get::<CraftingTable>().unwrap().table_name(), "");
    let detection = components.get::<LiquidDetection>().unwrap();
    assert!(!detection.detection_rules[0].can_contain_liquid());
    assert!(!detection.detection_rules[0].use_liquid_clipping());
    assert!(
        !components
            .get::<BlockEntity>()
            .unwrap()
            .dynamic_properties()
    );

    // The 1.26.20 → 1.26.60 step gives the omitted box its corners, the full block.
    let Some(CollisionBox::Boxes(collision)) = components.get::<CollisionBox>() else {
        panic!("the list form");
    };
    assert_eq!(collision.len(), 1);
    assert_eq!(collision[0].corners(), ([0.0; 3], [16.0; 3]));
}

/// A collision box of the newest format is written back as written, an omitted member left
/// out and a default one kept; one of an older format holds what the 1.26.20 → 1.26.60 step
/// made of it, every member written (the step keeps no trace of what the file left out).
#[test]
fn a_collision_box_member_is_written_back_as_the_newest_format_holds_it() {
    let newest = |components: &str| {
        format!(
            r#"{{"format_version": "1.26.60", "minecraft:block": {{"description": {{"identifier": "minecraft:x"}}, "components": {{{components}}}}}}}"#
        )
    };
    for components in [
        r#""minecraft:collision_box": [{}]"#,
        r#""minecraft:collision_box": [{"origin": [-8, 0, -8], "size": [16, 16, 16]}]"#,
    ] {
        let (written, input) = rewritten(&newest(components));
        assert_eq!(written, input);
    }
    let (written, _) = rewritten(&block("", r#""minecraft:collision_box": {}"#));
    assert!(
        written.contains(r#""origin""#) && written.contains(r#""size""#),
        "{written}"
    );
    let (written, input) = rewritten(&block(
        "",
        r#""minecraft:collision_box": {"origin": [-8, 0, -8], "size": [16, 16, 16]}"#,
    ));
    assert_eq!(written, input);
}
