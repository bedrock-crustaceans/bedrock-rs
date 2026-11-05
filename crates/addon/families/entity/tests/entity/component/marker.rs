//! Marker components: `{}`, a stray member kept and reported.

use bedrock_addon_core::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon_core::versioning::ConvertMode;
use bedrock_addon_family_entity::component::{
    EntityComponent, InputGroundControlled, IsBaby, IsHiddenWhenInvisible, RendersWhenInvisible,
    RotationAxisAligned, RotationLockedToVehicle, UsesLegacyAmbientSounds, UsesLegacyFriction,
};

use super::{fixture, has, json, read, read_as, round_trip, strict, write, write_with};

/// The 49 marker keys of
const MARKERS: [&str; 49] = [
    "body_rotation_always_follows_head",
    "body_rotation_blocked",
    "can_climb",
    "can_fly",
    "can_power_jump",
    "fire_immune",
    "input_ground_controlled",
    "is_baby",
    "is_charged",
    "is_chested",
    "is_hidden_when_invisible",
    "is_ignited",
    "is_illager_captain",
    "is_pregnant",
    "is_saddled",
    "is_shaking",
    "is_sheared",
    "is_stackable",
    "is_stunned",
    "is_tamed",
    "renders_when_invisible",
    "rotation_axis_aligned",
    "rotation_locked_to_vehicle",
    "block_climber",
    "can_join_raid",
    "dimension_bound",
    "item_hopper",
    "out_of_control",
    "suspect_tracking",
    "transient",
    "uses_legacy_friction",
    "uses_uniform_air_drag",
    "vibration_damper",
    "vibration_listener",
    "annotation.open_door",
    "bucketable",
    "cannot_be_attacked",
    "floats_in_liquid",
    "hide",
    "managed_wandering_trader",
    "not_pickable_from_inside",
    "persistent",
    "trade_resupply",
    "trust",
    "wants_jockey",
    "can_stand_on_powder_snow",
    "freezing_immune",
    "freezing_vulnerable",
    "uses_legacy_ambient_sounds",
];

#[test]
fn every_marker_reads_an_empty_object_as_a_typed_value() {
    for key in MARKERS {
        let key = format!("minecraft:{key}");
        let (value, d) = read(&key, "{}", ConvertMode::Strict);
        assert!(d.is_empty(), "{key}: {d:?}");
        let value = value.unwrap_or_else(|| panic!("{key} dropped"));
        assert!(
            !matches!(value, EntityComponent::Dynamic(_)),
            "{key} is typed"
        );
    }
}

#[test]
fn e27_marker_with_a_stray_member_keeps_it_and_reports_unrecognized_field() {
    // e27: "a marker with a stray member (`UnrecognizedField`)".
    let (marker, d) = strict::<RotationLockedToVehicle>(&fixture("e27_marker_stray_member.json"));
    assert!(has(
        &d,
        |c| matches!(c, DiagnosticCode::UnrecognizedField { .. }),
        Severity::Warning,
        "stray"
    ));
    assert_eq!(
        d.unknown()
            .get(&bedrock_addon_core::diagnostic::JsonPath::new(), "stray"),
        Some(&json("1"))
    );
    assert_eq!(write_with(&marker, d.unknown()), json(r#"{"stray": 1}"#));
}

#[test]
fn e20_renders_when_invisible_round_trips_as_an_empty_object() {
    // e20: the spider marker `renders_when_invisible {}`.
    let marker = round_trip::<RendersWhenInvisible>("{}");
    assert_eq!(write(&marker), json("{}"));
}

#[test]
fn a_marker_that_is_not_an_object_is_an_error_in_a_strict_read() {
    let (value, d) = read_as::<IsBaby>("true", ConvertMode::Strict);
    assert!(value.is_none());
    assert!(d.has_errors(), "{d:?}");
}

#[test]
fn a_marker_that_is_not_an_object_is_a_warning_in_the_lenient_conversion() {
    // "A value that is not an object … gets `InvalidValue` (an information of the crate's own) and the
    // component is dropped".
    let (value, d) = read_as::<UsesLegacyAmbientSounds>("[]", ConvertMode::Lenient);
    assert!(value.is_none(), "dropped");
    assert_eq!(d.len(), 1, "{d:?}");
    assert!(has(
        &d,
        |c| matches!(c, DiagnosticCode::InvalidValue { .. }),
        Severity::Info,
        ""
    ));
}

/// A marker the step of row `row` inserts reads as written there: `{}`.
fn inserted_marker<T>()
where
    T: bedrock_addon_core::document::Component<Set = EntityComponent>
        + for<'a> facet::Facet<'a>
        + PartialEq
        + std::fmt::Debug,
{
    let marker = round_trip::<T>("{}");
    assert_eq!(write(&marker), json("{}"));
}

#[test]
fn e00b_is_hidden_when_invisible_inserted_for_the_player_reads() {
    // e00b: `is_hidden_when_invisible {}` for `minecraft:player`.
    inserted_marker::<IsHiddenWhenInvisible>();
}

#[test]
fn e10_input_ground_controlled_marker_reads() {
    // e10: the `input_ground_controlled` ops read the marker (`WASDControlledDefinition`).
    inserted_marker::<InputGroundControlled>();
}

#[test]
fn e25_rotation_axis_aligned_renamed_marker_reads() {
    // e25: `body_rotation_axis_aligned` renamed to `rotation_axis_aligned`.
    inserted_marker::<RotationAxisAligned>();
}

#[test]
fn e31_uses_legacy_friction_inserted_for_a_custom_entity_reads() {
    // e31: `uses_legacy_friction {}` inserted.
    inserted_marker::<UsesLegacyFriction>();
}
