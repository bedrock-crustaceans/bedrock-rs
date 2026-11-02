//! The `on_*` trigger components and `on_equipment_changed`.

use bedrock_addon_core::shared::FilterSubject;
use bedrock_addon_core::versioning::ConvertMode;
use bedrock_addon_family_entity::component::{
    EquipmentSlot, OnDeath, OnEquipmentChanged, OnFriendlyAnger, OnHurt, OnHurtByPlayer, OnIgnite,
    OnStartLanding, OnStartTakeoff, OnTargetAcquired, OnTargetEscape, OnWakeWithOwner,
};

use super::{json, read, read_as, round_trip, strict, write};

const TRIGGERS: [&str; 10] = [
    "on_death",
    "on_friendly_anger",
    "on_hurt",
    "on_hurt_by_player",
    "on_ignite",
    "on_start_landing",
    "on_start_takeoff",
    "on_target_acquired",
    "on_target_escape",
    "on_wake_with_owner",
];

#[test]
fn every_trigger_component_reads_event_target_and_filters() {
    let text = r#"{"event": "minecraft:become_aggressive", "target": "self",
                   "filters": {"test": "is_family", "subject": "other", "value": "player"}}"#;
    for key in TRIGGERS {
        let key = format!("minecraft:{key}");
        let (value, d) = read(&key, text, ConvertMode::Strict);
        assert!(d.is_empty(), "{key}: {d:?}");
        assert!(value.is_some(), "{key}");
    }
}

#[test]
fn on_target_acquired_round_trips_with_its_values() {
    let value = round_trip::<OnTargetAcquired>(
        r#"{"event": "minecraft:become_aggressive", "target": "self"}"#,
    );
    assert_eq!(value.event.0, "minecraft:become_aggressive");
    assert_eq!(value.target, FilterSubject::Self_);
    assert!(value.filters.is_none());
}

#[test]
fn trigger_components_default_to_no_event_on_self() {
    let (death, _) = strict::<OnDeath>("{}");
    assert_eq!(death, OnDeath::default());
    assert_eq!(death.event.0, "");
    assert_eq!(death.target, FilterSubject::Self_);
    let (hurt, _) = strict::<OnHurt>(r#"{"event": "x:hurt", "target": "other"}"#);
    assert_eq!(hurt.target, FilterSubject::Other);
    let _ = (
        strict::<OnFriendlyAnger>("{}"),
        strict::<OnHurtByPlayer>("{}"),
        strict::<OnIgnite>("{}"),
        strict::<OnStartLanding>("{}"),
        strict::<OnStartTakeoff>("{}"),
        strict::<OnTargetEscape>("{}"),
        strict::<OnWakeWithOwner>("{}"),
    );
}

#[test]
fn a_trigger_with_an_unknown_target_is_an_error_in_a_strict_read() {
    let (value, d) = read_as::<OnHurt>(
        r#"{"event": "x:e", "target": "nobody"}"#,
        ConvertMode::Strict,
    );
    assert!(value.is_none());
    assert!(d.has_errors(), "{d:?}");
}

#[test]
fn on_equipment_changed_reads_its_slots() {
    let value = round_trip::<OnEquipmentChanged>(
        r#"{"slots": [{"slot": "slot.armor.head", "on_equip": "x:helmet_on", "on_unequip": "x:helmet_off"},
                      {"on_equip": "x:held"}]}"#,
    );
    assert_eq!(value.slots.len(), 2);
    assert_eq!(value.slots[0].slot, EquipmentSlot::ArmorHead);
    assert_eq!(value.slots[1].slot, EquipmentSlot::WeaponMainhand);
    assert_eq!(value.slots[1].on_unequip.0, "");
    assert_eq!(
        write(&value),
        json(
            r#"{"slots": [{"slot": "slot.armor.head", "on_equip": "x:helmet_on", "on_unequip": "x:helmet_off"},
                          {"slot": "slot.weapon.mainhand", "on_equip": "x:held", "on_unequip": ""}]}"#
        )
    );
}
