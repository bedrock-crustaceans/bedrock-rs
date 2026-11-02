//! `boss`, `burns_in_daylight`, `damage_over_time`, `exhaustion_values`, `insomnia`,
//! `ignore_cannot_be_attacked`: defaults, round trips, strict and lenient wrong types.

use bedrock_addon_core::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon_core::shared::Filter;
use bedrock_addon_core::versioning::ConvertMode;
use bedrock_addon_family_entity::component::{
    Boss, BurnsInDaylight, DamageOverTime, EquipmentSlot, ExhaustionValues, IgnoreCannotBeAttacked,
    Insomnia,
};

use super::{has, json, lenient, read_as, round_trip, strict, write};

#[test]
fn boss_reads_its_schema_defaults() {
    let (boss, d) = strict::<Boss>("{}");
    assert!(d.is_empty(), "{d:?}");
    assert_eq!(boss, Boss::default());
    assert_eq!(boss.name.0, "");
    assert!(!boss.should_darken_sky.0);
    assert_eq!(boss.hud_range.0, 55);
}

#[test]
fn e00a_boss_values_of_the_wither_and_the_dragon_round_trip() {
    // e00a: the 1.10.0 A-op writes these values; author-written members stay.
    let wither = round_trip::<Boss>(
        r#"{"name": "entity.wither.name", "should_darken_sky": true, "hud_range": 55}"#,
    );
    assert!(wither.should_darken_sky.0);
    assert_eq!(wither.name.0, "entity.wither.name");
    let dragon = round_trip::<Boss>(r#"{"should_darken_sky": false, "hud_range": 125}"#);
    assert_eq!(dragon.hud_range.0, 125);
}

#[test]
fn boss_with_a_string_range_is_an_error_in_a_strict_read() {
    let (boss, d) = read_as::<Boss>(r#"{"hud_range": "55"}"#, ConvertMode::Strict);
    assert!(boss.is_none(), "dropped");
    assert!(has(
        &d,
        |c| matches!(c, DiagnosticCode::InvalidType { .. }),
        Severity::Error,
        "hud_range"
    ));
}

#[test]
fn boss_with_a_string_range_takes_the_default_in_the_lenient_conversion() {
    let (boss, d) = lenient::<Boss>(r#"{"hud_range": "40", "should_darken_sky": true}"#);
    assert_eq!(boss.hud_range.0, 55);
    assert!(boss.should_darken_sky.0, "the other members are kept");
    assert!(has(
        &d,
        |c| matches!(c, DiagnosticCode::InvalidValue { .. }),
        Severity::Info,
        "hud_range"
    ));
}

#[test]
fn burns_in_daylight_reads_its_protection_slot() {
    let (default, _) = strict::<BurnsInDaylight>("{}");
    assert_eq!(default.protection_slot, EquipmentSlot::ArmorHead);
    let chest = round_trip::<BurnsInDaylight>(r#"{"protection_slot": "slot.armor.chest"}"#);
    assert_eq!(chest.protection_slot, EquipmentSlot::ArmorChest);
    let (bad, d) = read_as::<BurnsInDaylight>(
        r#"{"protection_slot": "slot.armor.hat"}"#,
        ConvertMode::Strict,
    );
    assert!(bad.is_none());
    assert!(d.has_errors());
}

#[test]
fn damage_over_time_and_insomnia_round_trip() {
    let dot = round_trip::<DamageOverTime>(r#"{"damage_per_hurt": 2, "time_between_hurt": 0.5}"#);
    assert_eq!(dot.damage_per_hurt.0, 2);
    assert_eq!(dot.time_between_hurt.0, 0.5);
    assert_eq!(DamageOverTime::default().damage_per_hurt.0, 1);
    let insomnia = round_trip::<Insomnia>(r#"{"days_until_insomnia": 2.5}"#);
    assert_eq!(insomnia.days_until_insomnia.0, 2.5);
    assert_eq!(Insomnia::default().days_until_insomnia.0, 3.0);
}

#[test]
fn e08_exhaustion_values_from_an_empty_object_are_the_vanilla_defaults() {
    // e08: the 1.18.30 A-op writes `{}` for the player; the typed read fills the
    // defaults vanilla's legacy parser used.
    let (values, _) = strict::<ExhaustionValues>("{}");
    assert_eq!(values.heal.0, 6.0);
    assert_eq!(values.jump.0, 0.05);
    assert_eq!(values.sprint_jump.0, 0.2);
    assert_eq!(values.mine.0, 0.005);
    assert_eq!(values.attack.0, 0.1);
    assert_eq!(values.damage.0, 0.1);
    assert_eq!(values.walk.0, 0.0);
    assert_eq!(values.sprint.0, 0.01);
    assert_eq!(values.swim.0, 0.01);
    assert_eq!(values.lunge.0, 4.0);
    let custom = round_trip::<ExhaustionValues>(r#"{"heal": 3, "walk": 0.25}"#);
    assert_eq!(custom.heal.0, 3.0);
}

#[test]
fn ignore_cannot_be_attacked_keeps_its_filters() {
    let value = round_trip::<IgnoreCannotBeAttacked>(
        r#"{"filters": {"test": "is_family", "subject": "other", "value": "player"}}"#,
    );
    assert!(matches!(value.filters, Some(Filter::Test(_))));
    assert_eq!(
        write(&value),
        json(r#"{"filters": {"test": "is_family", "subject": "other", "value": "player"}}"#)
    );
}

#[test]
fn ignore_cannot_be_attacked_reports_an_unknown_filter_test() {
    let (_, d) = strict::<IgnoreCannotBeAttacked>(r#"{"filters": {"test": "is_bogus"}}"#);
    assert!(
        d.iter()
            .any(|f| f.location.json_path.to_string().starts_with("filters")),
        "{d:?}"
    );
}

#[test]
fn exhaustion_values_empty_string_is_the_defaults_in_the_lenient_conversion() {
    // vanilla's player file writes `""`, which the legacy parser reads as all
    // defaults.
    let (values, d) = lenient::<ExhaustionValues>(r#""""#);
    assert!(d.is_empty(), "{d:?}");
    assert_eq!(values, ExhaustionValues::default());
    let (strict, d) = read_as::<ExhaustionValues>(r#""""#, ConvertMode::Strict);
    assert!(strict.is_none() && d.has_errors());
}

/// At 1.26.50:
/// the member finding carries the game's text `days_until_insomnia: invalid numeric value`
/// is an `InvalidType` error at `days_until_insomnia`.
#[test]
fn a_wrong_typed_insomnia_threshold_carries_the_game_s_text() {
    let (value, d) =
        read_as::<Insomnia>(r#"{ "days_until_insomnia": "three" }"#, ConvertMode::Strict);
    assert!(value.is_none(), "{d:?}");
    let found: Vec<_> = d
        .iter()
        .filter(|f| {
            matches!(f.code, DiagnosticCode::InvalidType { .. })
                && f.severity == Severity::Error
                && f.location.json_path.to_string() == "days_until_insomnia"
        })
        .collect();
    assert_eq!(found.len(), 1, "{d:?}");
    let DiagnosticCode::InvalidType { text: Some(text) } = &found[0].code else {
        panic!("no text: {d:?}")
    };
    assert_eq!(
        text.to_string(),
        "days_until_insomnia: invalid numeric value"
    );
}
