//! `apply_knockback_rules` and `pushable_by_entity`, with their older shapes and the
//! component-side upgrades of and

use bedrock_addon_core::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon_core::json::JsonSlice;
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::versioning::ConvertMode;
use bedrock_addon_family_entity::component::older::{
    ApplyKnockbackRulesV1_8_0, PushableByEntityV1_26_20, PushableByEntityV1_26_30,
};
use bedrock_addon_family_entity::component::{
    ApplyKnockbackRules, ExtraKnockbackApproach, KnockbackMode, KnockbackPreset, PushMode,
    PushableByEntity,
};

use super::{has, json, read_as, round_trip, strict, write};

/// Reads an older shape strictly through its own parse.
fn older<T>(
    read: fn(
        JsonSlice<'_>,
        ConvertMode,
        &LoadContext,
        &mut bedrock_addon_core::diagnostic::Diagnostics,
    ) -> Option<T>,
    text: &str,
) -> (Option<T>, bedrock_addon_core::diagnostic::Diagnostics) {
    let mut d = bedrock_addon_core::diagnostic::Diagnostics::default();
    let value = read(
        JsonSlice { text, offset: 0 },
        ConvertMode::Strict,
        &LoadContext::detached(),
        &mut d,
    );
    (value, d)
}

#[expect(clippy::float_cmp, reason = "the reader gives exact values")]
#[test]
fn apply_knockback_rules_preset_reads_its_1_26_30_defaults() {
    let (rules, _) = strict::<ApplyKnockbackRules>(r#"{"presets": [{}]}"#);
    let preset = &rules.presets[0];
    assert_eq!(*preset, KnockbackPreset::default());
    assert_eq!(preset.horizontal_power.0, 1.0);
    assert_eq!(preset.vertical_power.0, 0.4);
    assert_eq!(preset.vertical_velocity_cap.0, 0.4);
    assert!(!preset.check_if_target_is_immersed_in_water.0);
    assert_eq!(
        preset.extra_knockback_approach,
        ExtraKnockbackApproach::ReapplyDefault
    );
    assert_eq!(preset.knockback_mode, KnockbackMode::RelativeHorizontal);
    assert!(!preset.scale_with_damage.0);
    assert_eq!(preset.slowdown_scale.0, 0.5);
}

#[expect(clippy::float_cmp, reason = "the reader gives exact values")]
#[test]
fn e31b_iron_golem_knockback_preset_round_trips() {
    // e31b: the rescaled 1.26.20 A-op values (iron golem ≥ 1.18.30: 1.3 × 0.4).
    let rules = round_trip::<ApplyKnockbackRules>(
        r#"{"presets": [{"horizontal_power": 0.52, "vertical_power": 0.39, "vertical_velocity_cap": 0.8,
                         "check_if_target_is_immersed_in_water": false}]}"#,
    );
    assert_eq!(rules.presets[0].horizontal_power.0, 0.52);
    let built = ApplyKnockbackRules::with_preset(KnockbackPreset::new(0.52, 0.39, 0.8, false));
    assert_eq!(built, rules);
}

#[test]
fn apply_knockback_rules_slowdown_scale_above_one_is_an_error() {
    let (value, d) = read_as::<ApplyKnockbackRules>(
        r#"{"presets": [{"slowdown_scale": 1.5}]}"#,
        ConvertMode::Strict,
    );
    assert!(value.is_none());
    assert!(has(
        &d,
        |c| matches!(c, DiagnosticCode::InvalidValue { .. }),
        Severity::Error,
        "presets/0/slowdown_scale"
    ));
}

#[test]
fn deviation_apply_knockback_rules_empty_presets_is_an_error() {
    // "`apply_knockback_rules.presets` must be non-empty (ours … vanilla
    // accepts an empty list, with no minimum).
    let (value, d) = read_as::<ApplyKnockbackRules>(r#"{"presets": []}"#, ConvertMode::Strict);
    assert!(value.is_none());
    assert!(has(
        &d,
        |c| matches!(c, DiagnosticCode::InvalidValue { .. }),
        Severity::Error,
        "presets"
    ));
}

#[expect(clippy::float_cmp, reason = "the reader gives exact values")]
#[test]
fn e32_floor_knockback_preset_upgrades_with_the_1_26_30_defaults() {
    // e32 "knockback preset defaults";: every preset gets
    // `extra_knockback_approach: reapply_default`, `knockback_mode: relative_horizontal`,
    // `scale_with_damage: false`, `slowdown_scale` at its default.
    let (floor, d) = older(
        ApplyKnockbackRulesV1_8_0::read,
        r#"{"presets": [{"horizontal_power": 0.6, "vertical_power": -0.6, "vertical_velocity_cap": -0.4,
                         "check_if_target_is_immersed_in_water": true, "filter": {"test": "is_family", "value": "player"}}]}"#,
    );
    assert!(d.is_empty(), "{d:?}");
    let upgraded = ApplyKnockbackRules::from(floor.expect("reads"));
    let preset = &upgraded.presets[0];
    assert_eq!(preset.horizontal_power.0, 0.6);
    assert!(preset.check_if_target_is_immersed_in_water.0);
    assert!(preset.filter.is_some());
    assert_eq!(
        preset.extra_knockback_approach,
        ExtraKnockbackApproach::ReapplyDefault
    );
    assert_eq!(preset.knockback_mode, KnockbackMode::RelativeHorizontal);
    assert_eq!(preset.slowdown_scale.0, 0.5);
}

#[test]
fn a_floor_knockback_preset_does_not_know_the_1_26_30_members() {
    // a document below the gate that writes a newer field gets `UnrecognizedField`.
    let (floor, d) = older(
        ApplyKnockbackRulesV1_8_0::read,
        r#"{"presets": [{"knockback_mode": "hit_direction"}]}"#,
    );
    assert!(floor.is_some());
    assert!(has(
        &d,
        |c| matches!(c, DiagnosticCode::UnrecognizedField { .. }),
        Severity::Warning,
        "presets/0/knockback_mode"
    ));
}

#[expect(clippy::float_cmp, reason = "the reader gives exact values")]
#[test]
fn pushable_by_entity_preset_reads_its_1_26_40_defaults() {
    let (value, _) = strict::<PushableByEntity>(r#"{"presets": [{}]}"#);
    let preset = &value.presets[0];
    assert_eq!(preset.push_mode, PushMode::Default);
    assert_eq!(preset.strength_multiplier.0, 0.05);
    assert_eq!(preset.min_distance.0, 0.01);
    assert_eq!(preset.max_distance.0, f32::MAX);
    assert_eq!(preset.kick_speed_scale.0, 2.0);
    assert_eq!(preset.max_kick_speed.0, 0.5);
    assert_eq!(preset.vertical_kick_multiplier.0, 0.3);
    assert!(preset.require_collision_overlap.0);
    assert_eq!(value.use_default_with_overlap_check, None);
    assert!(value.uses_default_with_overlap_check());
}

#[test]
fn sulfur_cube_ball_preset_round_trips() {
    // retail 1.26.45 `sulfur_cube.json` uses `filters` (plural) and `ball`.
    let value = round_trip::<PushableByEntity>(
        r#"{"presets": [{"filters": {"test": "is_family", "subject": "other", "value": "player"},
                         "push_mode": "ball", "play_sound": true, "require_collision_overlap": false}]}"#,
    );
    assert_eq!(value.presets[0].push_mode, PushMode::Ball);
    assert!(!value.presets[0].require_collision_overlap.0);
}

#[test]
fn pushable_by_entity_negative_distance_is_an_error() {
    let (value, d) = read_as::<PushableByEntity>(
        r#"{"presets": [{"min_distance": -1}]}"#,
        ConvertMode::Strict,
    );
    assert!(value.is_none());
    assert!(d.has_errors());
}

#[test]
fn e32_pushable_by_entity_1_26_20_becomes_an_empty_preset_list() {
    // "the empty 1.26.20 struct becomes `{ presets: [] }`".
    let (marker, d) = older(PushableByEntityV1_26_20::read, "{}");
    assert!(d.is_empty());
    let next = PushableByEntityV1_26_30::from(marker.expect("reads"));
    assert!(next.presets.is_empty());
}

#[test]
fn e33_pushable_by_entity_presets_carried_to_1_26_40_do_not_require_overlap() {
    // e33 `require_collision_overlap: false`;
    let (old, d) = older(
        PushableByEntityV1_26_30::read,
        r#"{"presets": [{"push_mode": "ball"}, {"strength_multiplier": 0.1}]}"#,
    );
    assert!(d.is_empty(), "{d:?}");
    let new = PushableByEntity::from(old.expect("reads"));
    assert_eq!(new.presets.len(), 2);
    assert!(new.presets.iter().all(|p| !p.require_collision_overlap.0));
    assert_eq!(new.presets[0].push_mode, PushMode::Ball);
    assert_eq!(new.use_default_with_overlap_check, None);
    // No presets: the hidden flag is set to `false`.
    let (empty, _) = older(PushableByEntityV1_26_30::read, r#"{"presets": []}"#);
    let new = PushableByEntity::from(empty.expect("reads"));
    assert_eq!(new.use_default_with_overlap_check.map(|b| b.0), Some(false));
    assert_eq!(
        write(&new),
        json(r#"{"presets": [], "use_default_with_overlap_check": false}"#)
    );
}

#[test]
fn a_1_26_30_pushable_preset_does_not_know_require_collision_overlap() {
    let (old, d) = older(
        PushableByEntityV1_26_30::read,
        r#"{"presets": [{"require_collision_overlap": true}]}"#,
    );
    assert!(old.is_some());
    assert!(has(
        &d,
        |c| matches!(c, DiagnosticCode::UnrecognizedField { .. }),
        Severity::Warning,
        "presets/0/require_collision_overlap"
    ));
}

#[test]
fn e30_pushable_split_writes_an_empty_pushable_by_entity() {
    // e30 "pushable split": the step writes `pushable_by_entity {}`, the memberless
    // 1.26.20 component, which carries to `{presets: []}`.
    let (marker, d) = older(PushableByEntityV1_26_20::read, "{}");
    assert!(d.is_empty(), "{d:?}");
    let latest = PushableByEntity::from(PushableByEntityV1_26_30::from(marker.expect("reads")));
    assert!(latest.presets.is_empty());
    assert!(!latest.uses_default_with_overlap_check());
}
