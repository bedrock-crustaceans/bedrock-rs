//! The two mob descriptors and the two control flags, the goals' and
//! `wither_target_highest_damage`'s, pinned side by side. They read and write differently
//! today; these tests hold each copy's behaviour until a capture from the game settles the
//! drift (`must_see_forget_duration`: the goal's is a float, as the game's schema types the
//! goal member; the component's an integer).

use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath, Severity};
use bedrock_addon_core::versioning::ConvertMode;
use bedrock_addon_family_entity::component::{self, WitherTargetHighestDamage};
use bedrock_addon_family_entity::goal::{self, AvoidMobType, Breed};

use super::component::{json, read_as, write, write_with};

const AVOID_MOB_TYPE: &str = "minecraft:behavior.avoid_mob_type";
const BREED: &str = "minecraft:behavior.breed";

/// `wither_target_highest_damage` read from `json` in `mode`.
fn wither(json: &str, mode: ConvertMode) -> (Option<WitherTargetHighestDamage>, Diagnostics) {
    read_as::<WitherTargetHighestDamage>(json, mode)
}

/// `avoid_mob_type` read from `json` in `mode`.
fn avoid(json: &str, mode: ConvertMode) -> (Option<AvoidMobType>, Diagnostics) {
    super::goal::goal::<AvoidMobType>(AVOID_MOB_TYPE, json, mode)
}

/// `breed` read from `json` in `mode`.
fn breed(json: &str, mode: ConvertMode) -> (Option<Breed>, Diagnostics) {
    super::goal::goal::<Breed>(BREED, json, mode)
}

/// Every finding as `(code, severity, path)`.
fn findings(d: &Diagnostics) -> Vec<(&'static str, Severity, String)> {
    d.iter()
        .map(|f| (f.code.id(), f.severity, f.location.json_path.to_string()))
        .collect()
}

fn finding(code: &'static str, severity: Severity, path: &str) -> (&'static str, Severity, String) {
    (code, severity, path.to_owned())
}

const EVERY_MEMBER: &str = r#"{"entity_types": [{"check_if_outnumbered": true, "cooldown": 2,
    "filters": {"test": "is_family", "subject": "other", "value": "monster"}, "max_dist": 8,
    "max_flee": 4, "max_height": 2, "must_see": true, "must_see_forget_duration": 5,
    "priority": 1, "reevaluate_description": true, "sprint_speed_multiplier": 1.5,
    "walk_speed_multiplier": 0.5}]}"#;

#[test]
fn the_component_descriptor_reads_every_member() {
    let (value, d) = wither(EVERY_MEMBER, ConvertMode::Strict);
    assert!(d.is_empty(), "{d:?}");
    let value = value.expect("kept");
    let [one] = value.entity_types.as_slice() else {
        panic!("one descriptor")
    };
    assert!(one.check_if_outnumbered && one.reevaluate_description);
    assert_eq!(
        (one.cooldown, one.priority, one.must_see_forget_duration),
        (2, 1, 5)
    );
    assert_eq!(
        (one.max_dist, one.max_flee, one.max_height),
        (8.0, 4.0, 2.0)
    );
    assert_eq!(
        (one.sprint_speed_multiplier, one.walk_speed_multiplier),
        (1.5, 0.5)
    );
    assert_eq!(one.must_see, Some(true));
    assert!(one.filters.is_some());
    assert_eq!(
        write(&value),
        json(
            r#"{"priority": 0, "control_flags": [], "entity_types": [{"check_if_outnumbered": true,
                "cooldown": 2, "filters": {"test": "is_family", "subject": "other", "value": "monster"},
                "max_dist": 8, "max_flee": 4, "max_height": 2, "must_see": true,
                "must_see_forget_duration": 5, "priority": 1, "reevaluate_description": true,
                "sprint_speed_multiplier": 1.5, "walk_speed_multiplier": 0.5}]}"#
        )
    );
}

#[test]
fn the_goal_descriptor_reads_every_member() {
    let (value, d) = avoid(EVERY_MEMBER, ConvertMode::Strict);
    assert!(d.is_empty(), "{d:?}");
    let value = value.expect("kept");
    let types = value.entity_types.as_ref().expect("entity_types");
    let [one] = types.as_slice() else {
        panic!("one descriptor")
    };
    assert_eq!(one.check_if_outnumbered, Some(true));
    assert_eq!(one.reevaluate_description, Some(true));
    assert_eq!((one.cooldown, one.priority), (Some(2), Some(1)));
    assert_eq!(one.must_see_forget_duration, Some(5.0));
    assert_eq!(
        (one.max_dist, one.max_flee, one.max_height),
        (Some(8.0), Some(4.0), Some(2.0))
    );
    assert_eq!(
        (one.sprint_speed_multiplier, one.walk_speed_multiplier),
        (Some(1.5), Some(0.5))
    );
    assert_eq!(one.must_see, Some(true));
    assert_eq!(write(&value), json(EVERY_MEMBER));
}

/// A component descriptor read from `{"max_dist": 24}`, as written back.
const DEFAULTS_AT_24: &str = r#"{"check_if_outnumbered": false, "cooldown": 0, "max_dist": 24,
    "max_flee": 10, "max_height": -1, "must_see_forget_duration": 3, "priority": 0,
    "reevaluate_description": false, "sprint_speed_multiplier": 1, "walk_speed_multiplier": 1}"#;

/// A component descriptor read from `{"zzz": 1}`, written back with the member kept.
const DEFAULTS_AND_ZZZ: &str = r#"{"check_if_outnumbered": false, "cooldown": 0, "max_dist": 16,
    "max_flee": 10, "max_height": -1, "must_see_forget_duration": 3, "priority": 0,
    "reevaluate_description": false, "sprint_speed_multiplier": 1, "walk_speed_multiplier": 1,
    "zzz": 1}"#;

/// The component's descriptor members are plain with defaults: every default is written back,
/// `must_see` (an `Option`) and `filters` aside.
#[test]
fn the_component_descriptor_writes_its_defaults_back() {
    let (value, _) = wither(
        r#"{"entity_types": [{"max_dist": 24}]}"#,
        ConvertMode::Strict,
    );
    assert_eq!(
        write(&value.expect("kept")),
        json(&format!(
            r#"{{"priority": 0, "control_flags": [], "entity_types": [{DEFAULTS_AT_24}]}}"#
        ))
    );
}

/// The goal's descriptor members are `Option`s: only what was written is written back.
#[test]
fn the_goal_descriptor_writes_back_what_it_read() {
    let (value, _) = avoid(
        r#"{"entity_types": [{"max_dist": 24}]}"#,
        ConvertMode::Strict,
    );
    assert_eq!(
        write(&value.expect("kept")),
        json(r#"{"entity_types": [{"max_dist": 24}]}"#)
    );
}

/// The drift: `must_see_forget_duration` is an `f32` in the goal's descriptor and an `i32` in
/// the component's. 2.5 reads in a goal; in the component it fails the strict read and, in
/// the lenient one, drops the whole descriptor. (The only fixture with the member,
/// `zombie_nautilus.json`, has it as a `nearest_attackable_target` member, `17`, not inside a
/// descriptor; the game's
/// schema types the goal member as a float.)
#[test]
fn a_fractional_forget_duration_reads_in_a_goal_and_not_in_the_component() {
    const TEXT: &str = r#"{"entity_types": [{"must_see_forget_duration": 2.5}]}"#;
    for mode in [ConvertMode::Strict, ConvertMode::Lenient] {
        let (value, d) = avoid(TEXT, mode);
        assert!(d.is_empty(), "{mode:?}: {d:?}");
        let value = value.expect("kept");
        let types = value.entity_types.as_ref().expect("entity_types");
        assert_eq!(types.as_slice()[0].must_see_forget_duration, Some(2.5));
        assert_eq!(write(&value), json(TEXT));
    }

    let (value, d) = wither(TEXT, ConvertMode::Strict);
    assert!(value.is_none());
    assert_eq!(
        findings(&d),
        [finding(
            "core/invalid_type",
            Severity::Error,
            "entity_types/0/must_see_forget_duration"
        )]
    );
    let (value, d) = wither(TEXT, ConvertMode::Lenient);
    assert!(value.expect("kept").entity_types.as_slice().is_empty());
    assert_eq!(
        findings(&d),
        [
            finding(
                "core/invalid_value",
                Severity::Info,
                "entity_types/0/must_see_forget_duration"
            ),
            finding("core/dropped_field", Severity::Info, "entity_types/0"),
        ]
    );
}

/// `cooldown` is an integer in both, but a lenient read that refuses it drops the member in a
/// goal and the whole descriptor in the component.
#[test]
fn a_fractional_cooldown_drops_the_member_in_a_goal_and_the_descriptor_in_the_component() {
    const TEXT: &str = r#"{"entity_types": [{"cooldown": 2.5}]}"#;
    let strict = [finding(
        "core/invalid_type",
        Severity::Error,
        "entity_types/0/cooldown",
    )];
    let (value, d) = avoid(TEXT, ConvertMode::Strict);
    assert!(value.is_none());
    assert_eq!(findings(&d), strict);
    let (value, d) = wither(TEXT, ConvertMode::Strict);
    assert!(value.is_none());
    assert_eq!(findings(&d), strict);

    let (value, d) = avoid(TEXT, ConvertMode::Lenient);
    assert_eq!(
        findings(&d),
        [finding(
            "core/invalid_value",
            Severity::Info,
            "entity_types/0/cooldown"
        )]
    );
    assert_eq!(
        write(&value.expect("kept")),
        json(r#"{"entity_types": [{}]}"#)
    );
    let (value, d) = wither(TEXT, ConvertMode::Lenient);
    assert_eq!(
        findings(&d),
        [
            finding(
                "core/invalid_value",
                Severity::Info,
                "entity_types/0/cooldown"
            ),
            finding("core/dropped_field", Severity::Info, "entity_types/0"),
        ]
    );
    assert!(value.expect("kept").entity_types.as_slice().is_empty());
}

/// A member a descriptor does not define is reported and kept, and written back, in the
/// component as in a goal (`goal/rules.rs` pins the goal's).
#[test]
fn the_component_descriptor_keeps_an_unknown_member() {
    let (value, d) = wither(r#"{"entity_types": [{"zzz": 1}]}"#, ConvertMode::Strict);
    assert_eq!(
        findings(&d),
        [finding(
            "core/unrecognized_field",
            Severity::Warning,
            "entity_types/0/zzz"
        )]
    );
    let first = JsonPath::of_keys(["entity_types"]).index(0);
    assert!(d.unknown().get(&first, "zzz").is_some());
    assert_eq!(
        write_with(&value.expect("kept"), d.unknown()),
        json(&format!(
            r#"{{"priority": 0, "control_flags": [], "entity_types": [{DEFAULTS_AND_ZZZ}]}}"#
        ))
    );
}

/// One descriptor is written back as an object, a list as a list, in both.
#[test]
fn one_descriptor_or_a_list_is_written_in_the_form_read() {
    let (value, _) = avoid(r#"{"entity_types": {"max_dist": 24}}"#, ConvertMode::Strict);
    let value = value.expect("kept");
    assert!(matches!(
        value.entity_types,
        Some(goal::EntityTypes::One(_))
    ));
    assert_eq!(write(&value), json(r#"{"entity_types": {"max_dist": 24}}"#));

    let (value, _) = wither(r#"{"entity_types": {"max_dist": 24}}"#, ConvertMode::Strict);
    let value = value.expect("kept");
    assert!(matches!(value.entity_types, component::EntityTypes::One(_)));
    assert_eq!(
        write(&value),
        json(&format!(
            r#"{{"priority": 0, "control_flags": [], "entity_types": {DEFAULTS_AT_24}}}"#
        ))
    );
    let (value, _) = wither(
        r#"{"entity_types": [{"max_dist": 24}]}"#,
        ConvertMode::Strict,
    );
    assert_eq!(
        write(&value.expect("kept")),
        json(&format!(
            r#"{{"priority": 0, "control_flags": [], "entity_types": [{DEFAULTS_AT_24}]}}"#
        ))
    );
}

/// An unknown control flag fails a strict read in both, but not with the same finding: the
/// goal's flag is a closed name set that tells `InvalidType` at the element; the component's
/// is a plain enum whose refusal is `InvalidEnumValue` at the list, or at the element before
/// the bad one.
#[test]
fn an_unknown_control_flag_is_told_differently_by_a_goal_and_the_component() {
    let (value, d) = breed(r#"{"control_flags": ["fly"]}"#, ConvertMode::Strict);
    assert!(value.is_none());
    assert_eq!(
        findings(&d),
        [finding(
            "core/invalid_type",
            Severity::Error,
            "control_flags/0"
        )]
    );
    let (value, d) = breed(r#"{"control_flags": ["move", "fly"]}"#, ConvertMode::Strict);
    assert!(value.is_none());
    assert_eq!(
        findings(&d),
        [finding(
            "core/invalid_type",
            Severity::Error,
            "control_flags/1"
        )]
    );

    let (value, d) = wither(r#"{"control_flags": ["fly"]}"#, ConvertMode::Strict);
    assert!(value.is_none());
    assert_eq!(
        findings(&d),
        [finding(
            "core/invalid_enum_value",
            Severity::Error,
            "control_flags"
        )]
    );
    let (value, d) = wither(r#"{"control_flags": ["move", "fly"]}"#, ConvertMode::Strict);
    assert!(value.is_none());
    assert_eq!(
        findings(&d),
        [finding(
            "core/invalid_enum_value",
            Severity::Error,
            "control_flags/0"
        )]
    );
}

/// The lenient read drops an unknown control flag and keeps the rest, in both, with the same
/// finding.
#[test]
fn the_lenient_read_drops_an_unknown_control_flag_in_both() {
    const TEXT: &str = r#"{"control_flags": ["move", "fly"]}"#;
    let told = [finding(
        "core/invalid_value",
        Severity::Info,
        "control_flags/1",
    )];
    let (value, d) = breed(TEXT, ConvertMode::Lenient);
    assert_eq!(findings(&d), told);
    assert_eq!(
        value.expect("kept").control_flags,
        Some(vec![goal::ControlFlag::Move])
    );
    let (value, d) = wither(TEXT, ConvertMode::Lenient);
    assert_eq!(findings(&d), told);
    assert_eq!(
        value.expect("kept").control_flags,
        vec![component::ControlFlag::Move]
    );
}
