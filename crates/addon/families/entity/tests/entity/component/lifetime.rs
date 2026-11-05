//! `instant_despawn`, `tick_world`, `spawn_on_death`, `addrider`, `home`.

use bedrock_addon_core::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon_core::versioning::ConvertMode;
use bedrock_addon_family_entity::component::{
    Addrider, Home, InstantDespawn, RestrictionType, SpawnOnDeath, TickWorld,
};

use super::{has, json, lenient, read_as, round_trip, strict, write};

#[test]
fn e00f_instant_despawn_from_an_empty_despawn_round_trips() {
    // e00f: `despawn {}` becomes `instant_despawn {remove_child_entities: false}`.
    let value = round_trip::<InstantDespawn>(r#"{"remove_child_entities": false}"#);
    assert_eq!(value, InstantDespawn::default());
    assert_eq!(write(&value), json(r#"{"remove_child_entities": false}"#));
}

#[test]
fn tick_world_reads_its_schema_defaults() {
    let (value, _) = strict::<TickWorld>("{}");
    assert_eq!(value, TickWorld::default());
    assert_eq!(value.radius, 2);
    assert_eq!(value.distance_to_players, 128.0);
    assert!(value.never_despawn);
}

#[test]
fn tick_world_radius_outside_two_to_six_is_an_error_in_a_strict_read() {
    // the schema minimum / maximum of every typed field.
    for text in [
        r#"{"radius": 7}"#,
        r#"{"radius": 1}"#,
        r#"{"distance_to_players": 64}"#,
    ] {
        let (value, d) = read_as::<TickWorld>(text, ConvertMode::Strict);
        assert!(value.is_none(), "{text} is dropped");
        assert!(
            d.iter().any(|f| f.severity == Severity::Error
                && matches!(f.code, DiagnosticCode::InvalidValue { .. })),
            "{text}: {d:?}"
        );
    }
}

#[test]
fn tick_world_radius_outside_its_range_takes_the_default_in_the_lenient_conversion() {
    let (value, d) = lenient::<TickWorld>(r#"{"radius": 9, "never_despawn": false}"#);
    assert_eq!(value.radius, 2);
    assert!(!value.never_despawn);
    assert!(has(
        &d,
        |c| matches!(c, DiagnosticCode::InvalidValue { .. }),
        // The typed bound is the crate's own in the lenient conversion.
        Severity::Info,
        "radius"
    ));
}

#[test]
fn spawn_on_death_round_trips_with_its_range_and_filters() {
    let value = round_trip::<SpawnOnDeath>(
        r#"{"entity_to_spawn": "minecraft:slime", "spawn_amount": 2,
            "additional_spawn_range": {"min": 0, "max": 2}, "inherit_parent_name": false,
            "spawn_method": "born", "filters": {"test": "is_baby", "value": false}}"#,
    );
    assert_eq!(value.entity_to_spawn, "minecraft:slime");
    assert_eq!(value.additional_spawn_range.max, 2);
    assert!(value.filters.is_some());
    let (default, _) = strict::<SpawnOnDeath>("{}");
    assert_eq!(default, SpawnOnDeath::default());
    assert_eq!(default.entity_to_spawn, "self");
    assert_eq!(default.spawn_method, "spawned");
    assert!(default.inherit_parent_name);
}

#[test]
fn addrider_reads_its_riders() {
    let value = round_trip::<Addrider>(
        r#"{"riders": [{"entity_type": "minecraft:skeleton", "spawn_event": "minecraft:spawn_as_jockey"},
                       {"entity_type": "minecraft:chicken"}]}"#,
    );
    assert_eq!(value.riders.len(), 2);
    assert_eq!(value.riders[1].spawn_event, "");
}

#[test]
fn addrider_entry_without_entity_type_is_missing_a_required_member() {
    let (value, d) =
        read_as::<Addrider>(r#"{"riders": [{"spawn_event": "e"}]}"#, ConvertMode::Strict);
    assert!(value.is_none());
    assert!(
        d.iter()
            .any(|f| matches!(f.code, DiagnosticCode::MissingRequiredField { .. })),
        "{d:?}"
    );
}

#[test]
fn e01_guardian_home_round_trips() {
    // e01: `home { restriction_radius: 16 }`.
    let value = round_trip::<Home>(r#"{"restriction_radius": 16}"#);
    assert_eq!(value.restriction_radius, 16);
    assert_eq!(value.restriction_type, RestrictionType::None);
}

#[test]
fn e19_home_restriction_type_reads_each_value() {
    // e19: `restriction_type`.
    let value = round_trip::<Home>(
        r#"{"restriction_radius": 16, "restriction_type": "random_movement", "home_block_list": ["minecraft:bee_nest"]}"#,
    );
    assert_eq!(value.restriction_type, RestrictionType::RandomMovement);
    assert_eq!(value.home_block_list.len(), 1);
    let (all, _) = strict::<Home>(r#"{"restriction_type": "all_movement"}"#);
    assert_eq!(all.restriction_type, RestrictionType::AllMovement);
    let (bad, d) = read_as::<Home>(r#"{"restriction_type": "some"}"#, ConvertMode::Strict);
    assert!(bad.is_none());
    assert!(d.has_errors());
}

#[test]
fn the_lenient_conversion_reads_the_legacy_single_rider() {
    // Vanilla's 1.13.0 `spider.json`: `{"entity_type": "minecraft:skeleton"}` at the component
    // root; its 1.26.50 copy writes `riders: [{entity_type}]`.
    let (value, d) = lenient::<Addrider>(
        r#"{"entity_type": "minecraft:skeleton", "spawn_event": "minecraft:spawn_as_jockey"}"#,
    );
    assert!(d.is_empty(), "{d:?}");
    assert_eq!(value.riders.len(), 1);
    assert_eq!(value.riders[0].entity_type, "minecraft:skeleton");
    assert_eq!(value.riders[0].spawn_event, "minecraft:spawn_as_jockey");
    let (strict, d) = strict::<Addrider>(r#"{"entity_type": "minecraft:skeleton"}"#);
    assert!(strict.riders.is_empty());
    assert!(has(
        &d,
        |c| matches!(c, DiagnosticCode::UnrecognizedField { .. }),
        Severity::Warning,
        "entity_type"
    ));
}
