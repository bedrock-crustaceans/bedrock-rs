//! `interact`: the interactions, their sub-objects and the lenient conversion's spellings.

use bedrock_addon_core::versioning::ConvertMode;
use bedrock_addon_family_entity::component::{CopperEvent, Interact, InteractVibration};

use super::{fixture, lenient, read_as, round_trip, strict};

#[test]
fn interact_reads_a_vanilla_saddle_interaction() {
    let value = round_trip::<Interact>(&fixture("interact_saddle.json"));
    assert_eq!(value.interactions.len(), 2);
    let saddle = &value.interactions[0];
    assert_eq!(
        saddle.equip_item_slot.as_ref().map(|s| s.0.as_str()),
        Some("0")
    );
    assert_eq!(saddle.play_sounds.len(), 1);
    assert!(saddle.on_interact.is_some());
    assert!(saddle.use_item.0 && !saddle.swing.0);
    let unsaddle = &value.interactions[1];
    assert_eq!(unsaddle.vibration, InteractVibration::Shear);
    assert_eq!(
        unsaddle.particle_on_start.as_ref().map(|p| p.copper_event),
        Some(CopperEvent::None)
    );
}

#[expect(clippy::float_cmp, reason = "the reader gives exact values")]
#[test]
fn an_interaction_reads_its_schema_defaults() {
    let (value, _) = strict::<Interact>(r#"{"interactions": [{}]}"#);
    let one = &value.interactions[0];
    assert!(one.swing.0, "the 1.26.0 default is `true`");
    assert_eq!(one.vibration, InteractVibration::EntityInteract);
    assert_eq!(one.cooldown.0, 0.0);
    assert_eq!(one.health_amount.0, 0);
    assert!(
        one.add_items.is_none() && one.spawn_items.is_none() && one.repair_entity_item.is_none()
    );
    let (repair, _) = strict::<Interact>(
        r#"{"interactions": [{"repair_entity_item": {"slot": "slot.armor.body"}, "spawn_items": {"table": "loot_tables/x.json"}}]}"#,
    );
    let one = &repair.interactions[0];
    assert_eq!(one.repair_entity_item.as_ref().map(|r| r.amount.0), Some(1));
    assert_eq!(one.spawn_items.as_ref().map(|s| s.y_offset.0), Some(1.0));
}

#[test]
fn a_string_play_sounds_is_an_error_in_a_strict_read() {
    let (value, d) = read_as::<Interact>(
        r#"{"interactions": [{"play_sounds": "eat"}]}"#,
        ConvertMode::Strict,
    );
    assert!(value.is_none() && d.has_errors());
}

#[test]
fn the_lenient_conversion_reads_a_string_play_sounds_as_one_sound() {
    // The legacy parser takes a single string; vanilla's 1.26.50 files write a one-element
    // array where older ones wrote the string.
    let (value, d) = lenient::<Interact>(
        r#"{"interactions": [{"play_sounds": "eat", "spawn_entities": "minecraft:cow"}]}"#,
    );
    assert!(d.is_empty(), "{d:?}");
    assert_eq!(value.interactions[0].play_sounds[0].0, "eat");
    assert_eq!(value.interactions[0].spawn_entities[0].0, "minecraft:cow");
}

#[test]
fn an_on_interact_without_event_is_a_condition_only() {
    // Vanilla's 1.26.50 `horse.json` writes `on_interact` with `filters` and `target` only.
    let value = round_trip::<Interact>(
        r#"{"interactions": [{"on_interact": {"filters": {"test": "is_sneak_held", "subject": "other", "value": false}, "target": "self"}, "equip_item_slot": "1"}]}"#,
    );
    let trigger = value.interactions[0]
        .on_interact
        .as_ref()
        .expect("on_interact");
    assert!(trigger.event.is_none() && trigger.filters.is_some());
}

#[test]
fn the_lenient_conversion_reads_a_legacy_trigger_string_and_a_single_interaction() {
    let (value, d) =
        lenient::<Interact>(r#"{"interactions": {"on_interact": "minecraft:on_sheared"}}"#);
    assert!(d.is_empty(), "{d:?}");
    let trigger = value.interactions[0]
        .on_interact
        .as_ref()
        .expect("on_interact");
    assert_eq!(trigger.event.as_deref(), Some("minecraft:on_sheared"));
}
