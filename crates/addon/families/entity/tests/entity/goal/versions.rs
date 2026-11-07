//! Goals read by the document's format version (through an entity document): the typed → typed
//! conversions and member gates between a goal's typed version and the newest struct, and the
//! shared-shape defaults, bounds and member types every version reads.

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::molang::{self, MolangFailure, MolangFieldMut, ReaderKind};
use bedrock_addon_core::shared::EventTrigger;
use bedrock_addon_core::versioning::ConvertMode;
use bedrock_addon_family_entity::goal;

use super::{assert_finding, at_version, errors, fixture_member, goal, strict};

fn event(trigger: Option<&EventTrigger>) -> Option<&str> {
    trigger.and_then(|t| t.event.as_deref())
}

/// `e33_1_26_30_on_kill_overlap`, the typed path: a melee goal of a document at
/// or above its typed version and below 1.26.40 (read strictly) folds `on_kill` into
/// `on_attack`; at 1.26.40 the two are separate members.
#[test]
fn e33_a_strict_melee_goal_below_1_26_40_folds_on_kill() {
    let both = fixture_member("e33_on_kill_overlap", "both");
    for (key, typed) in [
        ("minecraft:behavior.melee_attack", (1, 26, 0)),
        ("minecraft:behavior.melee_box_attack", (1, 26, 0)),
        ("minecraft:behavior.delayed_attack", (1, 26, 0)),
        ("minecraft:behavior.stomp_attack", (1, 26, 0)),
        ("minecraft:behavior.use_kinetic_weapon", (1, 21, 120)),
    ] {
        for version in [typed, (1, 26, 30)] {
            let (value, d) = parse_at(key, &both, version);
            assert!(errors(&d).is_empty(), "{key} {version:?}: {d:?}");
            assert_eq!(
                value,
                Some(("x:killed".to_owned(), None)),
                "{key} {version:?}"
            );
        }
        let (value, _) = parse_at(key, &both, (1, 26, 40));
        assert_eq!(
            value,
            Some(("x:hit".to_owned(), Some("x:killed".to_owned()))),
            "{key} at 1.26.40"
        );
    }
}

/// The `(on_attack, on_kill)` events of a melee goal `key` read at `version`.
fn parse_at(
    key: &str,
    json: &str,
    version: (u16, u16, u16),
) -> (
    Option<(String, Option<String>)>,
    bedrock_addon_core::diagnostic::Diagnostics,
) {
    macro_rules! events {
        ($t:ty) => {{
            let (g, d) = at_version::<$t>(key, json, version);
            (
                g.map(|g| {
                    (
                        event(g.on_attack.as_ref()).unwrap_or_default().to_owned(),
                        event(g.on_kill.as_ref()).map(str::to_owned),
                    )
                }),
                d,
            )
        }};
    }
    match key {
        "minecraft:behavior.melee_attack" => events!(goal::MeleeAttack),
        "minecraft:behavior.melee_box_attack" => events!(goal::MeleeBoxAttack),
        "minecraft:behavior.delayed_attack" => events!(goal::DelayedAttack),
        "minecraft:behavior.stomp_attack" => events!(goal::StompAttack),
        _ => events!(goal::UseKineticWeapon),
    }
}

/// `e29c_1_21_130_typed_to_typed`, the typed path: `aquatic_charge_attack` is typed
/// from 1.21.120, where `charge_speed_multiplier` is the live name: a 1.21.130 document gets no
/// `DeprecatedMember` and the value becomes `charge_speed`; from 1.26.0 it is deprecated: an
/// error that rejects the document (in both masks).
#[test]
fn e29c_charge_speed_multiplier_is_live_below_1_26_0() {
    const KEY: &str = "minecraft:behavior.aquatic_charge_attack";
    let old = fixture_member("steps", "e29c_aquatic_old_name");
    let (g, d) = at_version::<goal::AquaticChargeAttack>(KEY, &old, (1, 21, 130));
    assert!(d.is_empty(), "{d:?}");
    let g = g.expect("kept");
    assert_eq!(g.charge_speed(), 0.5);
    assert!(
        d.unknown()
            .members_at(&bedrock_addon_core::diagnostic::JsonPath::new())
            .next()
            .is_none()
    );
    let (g, d) = at_version::<goal::AquaticChargeAttack>(KEY, &old, (1, 26, 0));
    assert_finding(
        &d,
        "core/deprecated_member_worded",
        "charge_speed_multiplier",
        Severity::Error,
    );
    assert!(g.is_none(), "the document is rejected");
}

/// `pickup_items.stop_if_holding_item` exists from 1.26.30; a 1.26.20
/// document that writes it gets `UnrecognizedField`, an error of the strict slice that refuses
/// the document.
#[test]
fn e32_stop_if_holding_item_is_gated_at_1_26_30() {
    const KEY: &str = "minecraft:behavior.pickup_items";
    let json = r#"{"priority": 2, "stop_if_holding_item": true}"#;
    let (g, d) = at_version::<goal::PickupItems>(KEY, json, (1, 26, 20));
    assert_finding(
        &d,
        "core/unrecognized_field",
        "stop_if_holding_item",
        Severity::Error,
    );
    assert!(g.is_none());
    let (g, d) = at_version::<goal::PickupItems>(KEY, json, (1, 26, 30));
    assert!(d.is_empty(), "{d:?}");
    assert!(g.expect("kept").stop_if_holding_item());
    // Below the goal's typed version the conversion tells it, an information: the hand-written
    // legacy reader passes the member over.
    let (_, d) = at_version::<goal::PickupItems>(KEY, json, (1, 26, 10));
    assert_finding(
        &d,
        "core/unrecognized_field",
        "stop_if_holding_item",
        Severity::Info,
    );
}

/// `DragonBase`: `active_speed` = 1 for both dragon goals that derive from it.
#[test]
fn dragon_goals_take_the_dragon_base_defaults() {
    let (g, _) = goal::<goal::DragonChargePlayer>(
        "minecraft:behavior.dragonchargeplayer",
        "{}",
        ConvertMode::Strict,
    );
    let g = g.expect("kept");
    assert_eq!(
        (g.active_speed(), g.flight_speed(), g.turn_speed()),
        (1.0, 0.6, 0.7)
    );
    let (g, _) = goal::<goal::DragonStrafePlayer>(
        "minecraft:behavior.dragonstrafeplayer",
        "{}",
        ConvertMode::Strict,
    );
    assert_eq!(g.expect("kept").active_speed(), 1.0);
}

/// `BaseMoveToGoal` / `BaseMoveToBlockGoal`: `speed_multiplier` = 1 (≥ 0) and
/// `search_count` = 10 (≥ 0) for the goals deriving from them (1.26.51 headers), and the bases'
/// bounds for every one of them. Three derived goals construct their definition with values of
/// their own, which the game's own 1.26.10 schema prints:
/// `harvest_farm_block` 0.5 and 0, `fertilize_farm_block` 0.5 and 9, `drop_item_for` 1 and 0.
#[test]
fn move_to_goals_take_the_base_defaults_and_bounds() {
    let (g, _) = goal::<goal::HarvestFarmBlock>(
        "minecraft:behavior.harvest_farm_block",
        "{}",
        ConvertMode::Strict,
    );
    let g = g.expect("kept");
    assert_eq!((g.speed_multiplier(), g.search_count()), (0.5, 0));
    let (g, _) = goal::<goal::FertilizeFarmBlock>(
        "minecraft:behavior.fertilize_farm_block",
        "{}",
        ConvertMode::Strict,
    );
    let g = g.expect("kept");
    assert_eq!((g.speed_multiplier(), g.search_count()), (0.5, 9));
    let (g, _) = goal::<goal::DropItemFor>(
        "minecraft:behavior.drop_item_for",
        "{}",
        ConvertMode::Strict,
    );
    let g = g.expect("kept");
    assert_eq!((g.speed_multiplier(), g.search_count()), (1.0, 0));
    // A derived goal whose schema prints the bases' values keeps them.
    let (g, _) = goal::<goal::InspectBookshelf>(
        "minecraft:behavior.inspect_bookshelf",
        "{}",
        ConvertMode::Strict,
    );
    let g = g.expect("kept");
    assert_eq!((g.speed_multiplier(), g.search_count()), (1.0, 10));
    // Not derived from the bases: the samples default stays.
    let (g, _) = goal::<goal::Work>("minecraft:behavior.work", "{}", ConvertMode::Strict);
    assert_eq!(g.expect("kept").speed_multiplier(), 0.5);
    for (member, json) in [
        ("speed_multiplier", r#"{"speed_multiplier": -1}"#),
        ("search_count", r#"{"search_count": -1}"#),
    ] {
        let (g, d) = strict("minecraft:behavior.harvest_farm_block", json);
        assert!(g.is_none());
        assert_finding(&d, "core/invalid_value", member, Severity::Error);
    }
}

/// `behavior.move_outdoors`: "`speed_multiplier` / `search_count` deprecated
/// and re-bound". The goal's own binding replaces the base's: its own defaults
/// (0.5 and 10) and no base bound.
#[test]
fn move_outdoors_rebinds_speed_multiplier_and_search_count() {
    let (g, d) = goal::<goal::MoveOutdoors>(
        "minecraft:behavior.move_outdoors",
        "{}",
        ConvertMode::Strict,
    );
    assert!(d.is_empty(), "{d:?}");
    let g = g.expect("kept");
    assert_eq!((g.speed_multiplier(), g.search_count()), (0.5, 10));
    let (g, d) = goal::<goal::MoveOutdoors>(
        "minecraft:behavior.move_outdoors",
        r#"{"speed_multiplier": -1, "search_count": -1}"#,
        ConvertMode::Strict,
    );
    assert!(d.is_empty(), "{d:?}");
    let g = g.expect("kept");
    assert_eq!((g.speed_multiplier(), g.search_count()), (-1.0, -1));
}

/// for the `TargetGoal` bounds of the goals deriving from
/// `nearest_attackable_target`, and their `attack_interval` range.
#[test]
fn target_goal_bounds_hold_for_derived_goals() {
    for key in [
        "minecraft:behavior.nearest_prioritized_attackable_target",
        "minecraft:behavior.defend_trusted_target",
    ] {
        for member in ["within_radius", "must_see_forget_duration"] {
            let json = format!(r#"{{"{member}": -1}}"#);
            let (g, d) = strict(key, &json);
            assert!(g.is_none(), "{key} {member}");
            assert_finding(&d, "core/invalid_value", member, Severity::Error);
        }
        let (g, d) = strict(key, r#"{"attack_interval": [2, 4]}"#);
        assert!(g.is_some() && d.is_empty(), "{key}: {d:?}");
    }
    let (g, d) = strict(
        "minecraft:behavior.nearest_prioritized_attackable_target",
        r#"{"persist_time": -1}"#,
    );
    assert!(g.is_none());
    assert_finding(&d, "core/invalid_value", "persist_time", Severity::Error);
    let (g, _) = goal::<goal::NearestPrioritizedAttackableTarget>(
        "minecraft:behavior.nearest_prioritized_attackable_target",
        r#"{"attack_interval": [2, 4]}"#,
        ConvertMode::Strict,
    );
    assert_eq!(g.expect("kept").attack_interval(), (2.0, 4.0));
}

/// `swim_with_entity.entity_types` is `EntityTypes`: one descriptor or a list.
#[test]
fn swim_with_entity_takes_one_descriptor_or_a_list() {
    const KEY: &str = "minecraft:behavior.swim_with_entity";
    let (g, d) = goal::<goal::SwimWithEntity>(
        KEY,
        r#"{"entity_types": {"filters": {"test": "is_family", "subject": "other", "value": "player"}}}"#,
        ConvertMode::Strict,
    );
    assert!(d.is_empty(), "{d:?}");
    assert!(matches!(
        g.expect("kept").entity_types,
        Some(goal::EntityTypes::One(_))
    ));
    let (g, _) = goal::<goal::SwimWithEntity>(
        KEY,
        r#"{"entity_types": [{"max_dist": 4}]}"#,
        ConvertMode::Strict,
    );
    assert!(matches!(
        g.expect("kept").entity_types,
        Some(goal::EntityTypes::Many(_))
    ));
}

/// `MobDescriptor.must_see_forget_duration` is a float; `avoid_block.target_selection_method`
/// is the closed enum.
#[test]
fn member_types_settled_in_review() {
    let (g, d) = goal::<goal::AvoidMobType>(
        "minecraft:behavior.avoid_mob_type",
        r#"{"entity_types": [{"must_see_forget_duration": 2.5}]}"#,
        ConvertMode::Strict,
    );
    assert!(d.is_empty(), "{d:?}");
    let types = g.expect("kept").entity_types.expect("entity_types");
    assert_eq!(types.as_slice()[0].must_see_forget_duration, Some(2.5));
    let (g, _) = goal::<goal::AvoidBlock>(
        "minecraft:behavior.avoid_block",
        r#"{"target_selection_method": "random"}"#,
        ConvertMode::Strict,
    );
    assert_eq!(
        g.expect("kept").target_selection_method(),
        goal::TargetSelectionMethod::Random
    );
}

/// A vector's object form keeps the members it does not define, reports them and writes them
/// back.
#[test]
fn vector_objects_keep_unknown_members() {
    const KEY: &str = "minecraft:behavior.drop_item_for";
    let json = r#"{"target_range": {"x": 6, "y": 6, "z": 6, "w": 1}}"#;
    let d = super::round_trip::<goal::DropItemFor>(KEY, json, ConvertMode::Strict).expect("kept");
    assert_finding(
        &d,
        "core/unrecognized_field",
        "target_range/w",
        Severity::Warning,
    );
    let (g, _) = goal::<goal::DropItemFor>(KEY, json, ConvertMode::Strict);
    let range = g.expect("kept").target_range.expect("target_range");
    assert!(range.extra.get("w").is_some());
}

/// A vector of the wrong shape is refused (`InvalidType` at the member, the reader's message
/// not carried), and both forms write back as read: pins the findings across the move of
/// `Vec3`/`Vec2` into shapes.
#[test]
fn vectors_of_the_wrong_shape_are_refused_and_both_forms_write_back() {
    const KEY: &str = "minecraft:behavior.drop_item_for";
    const LOOK: &str = "minecraft:memory_behavior.look_at_entity";
    let told = |key: &str, json: &str| -> Vec<String> {
        let (g, d) = strict(key, json);
        assert!(g.is_none(), "{key} {json}: refused");
        d.iter()
            .map(|d| {
                format!(
                    "{:?} {} {} {:?}",
                    d.severity,
                    d.location.json_path,
                    bedrock_addon_core::diagnostic::testing::shown(d),
                    d.detail
                )
            })
            .collect()
    };
    assert_eq!(
        told(KEY, r#"{"target_range": [1, 2]}"#),
        ["Error target_range core/invalid_type None"]
    );
    assert_eq!(
        told(KEY, r#"{"target_range": "far"}"#),
        ["Error target_range core/invalid_type None"]
    );
    assert_eq!(
        told(KEY, r#"{"target_range": {"x": "a"}}"#),
        ["Error target_range core/invalid_type None"]
    );
    assert_eq!(
        told(KEY, r#"{"target_range": [1, 2, 1e39]}"#),
        ["Error target_range core/invalid_type None"]
    );
    assert_eq!(
        told(
            LOOK,
            r#"{"look_distance": 1, "look_angle": [1, 2, 3], "target_entity": "t"}"#
        ),
        ["Error look_angle core/invalid_type None"]
    );
    assert_eq!(
        written::<goal::DropItemFor>(KEY, r#"{"target_range": [1, 2.5, 3]}"#),
        r#"{ "target_range": [ 1.0, 2.5, 3.0 ] }"#
    );
    assert_eq!(
        written::<goal::DropItemFor>(KEY, r#"{"target_range": {"z": 0.1, "x": 2}}"#),
        r#"{ "target_range": { "x": 2.0, "y": 0.0, "z": 0.1 } }"#
    );
    assert_eq!(
        written::<goal::MemoryLookAtEntity>(
            LOOK,
            r#"{"look_distance": 1, "look_angle": {"y": 30}, "target_entity": "t"}"#
        ),
        r#"{ "look_distance": 1.0, "look_angle": { "x": 0.0, "y": 30.0 }, "target_entity": "t" }"#
    );
}

/// The goal `T` read strictly from `json` under `key`, written back on one line.
fn written<T>(key: &str, json: &str) -> String
where
    T: bedrock_addon_core::document::Component<
            Set = bedrock_addon_family_entity::component::EntityComponent,
        > + for<'a> facet::Facet<'a>,
{
    let (g, d) = goal::<T>(key, json, ConvertMode::Strict);
    assert!(errors(&d).is_empty(), "{d:?}");
    let text = bedrock_addon_core::json::to_json(&g.expect("kept")).expect("writes");
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// `memory_sensor.game_event` `event_type` is the closed `GameEventType`, read in any case.
#[test]
fn game_event_type_is_a_closed_set() {
    const KEY: &str = "minecraft:memory_sensor.game_event";
    let (g, d) = goal::<goal::MemorySensorGameEvent>(
        KEY,
        r#"{"sensor_entries": [{"event_type": "instrument"}]}"#,
        ConvertMode::Strict,
    );
    assert!(d.is_empty(), "{d:?}");
    let entries = g.expect("kept").sensor_entries.expect("entries");
    assert_eq!(entries[0].event_type, Some(goal::GameEventType::Instrument));
    let (g, d) = strict(KEY, r#"{"sensor_entries": [{"event_type": "thunder"}]}"#);
    assert!(g.is_none());
    assert!(!errors(&d).is_empty());
}

/// The Molang members found by the pass-1 walk: `(path, reader, severity, failure)`.
fn molang_specs<T: molang::MolangFields>(
    value: &mut T,
) -> Vec<(String, ReaderKind, Severity, MolangFailure, bool)> {
    let mut found = Vec::new();
    molang::visit_fields(
        value,
        &bedrock_addon_core::diagnostic::JsonPath::new(),
        |field| {
            let source = matches!(field.value, MolangFieldMut::Source(_));
            found.push((
                field.path.to_string(),
                field.spec.reader,
                field.spec.error_severity,
                field.spec.on_failure,
                source,
            ));
        },
    );
    found
}

/// a legacy trigger `condition` (kept in the trigger's `extra`) is a Molang
/// member: `LenientVersioned`, a rejected expression only a warning.
#[test]
fn d5_legacy_trigger_conditions_are_walked_as_molang() {
    let json = r#"{"on_start": {"event": "x:dig", "condition": "query.is_baby"},
        "on_escape": [{"event": "x:a"}, {"event": "x:b", "condition": 1}]}"#;
    let (g, _) = goal::<goal::Dig>(
        "minecraft:behavior.dig",
        r#"{"on_start": {"event": "x:dig", "condition": "query.is_baby"}}"#,
        ConvertMode::Lenient,
    );
    let mut g = g.expect("kept");
    let fields = molang_specs(&mut g);
    assert_eq!(
        fields,
        [(
            "on_start/condition".to_owned(),
            ReaderKind::LenientVersioned,
            Severity::Error,
            MolangFailure::KeepLogged,
            false
        )]
    );
    // A second walk (pass 2) sees the same value.
    assert_eq!(molang_specs(&mut g).len(), 1);
    let (g, _) =
        goal::<goal::AvoidBlock>("minecraft:behavior.avoid_block", json, ConvertMode::Lenient);
    let paths: Vec<String> = molang_specs(&mut g.expect("kept"))
        .into_iter()
        .map(|f| f.0)
        .collect();
    assert_eq!(paths, ["on_escape/1/condition"]);
    // The component enum forwards the walk to every goal row with triggers (`molang` flag).
    let (component, _) = super::lenient(
        "minecraft:behavior.dig",
        r#"{"on_start": {"event": "x:dig", "condition": "query.is_baby"}}"#,
    );
    let mut component = component.expect("kept");
    assert_eq!(molang_specs(&mut component).len(), 1);
}

/// `eat_block.success_chance` read by the lenient conversion is
/// schema-validated, and a rejected expression is a warning that gives the default.
#[test]
fn d5_eat_block_success_chance_in_the_lenient_conversion() {
    const KEY: &str = "minecraft:behavior.eat_block";
    let json = r#"{"success_chance": "query.is_baby ? 0.1 : 0.01"}"#;
    let (g, _) = goal::<goal::EatBlock>(KEY, json, ConvertMode::Lenient);
    let mut g = g.expect("kept");
    assert!(g.converted);
    assert_eq!(
        molang_specs(&mut g),
        [(
            "success_chance".to_owned(),
            ReaderKind::SchemaValidated,
            Severity::Error,
            MolangFailure::ReplaceWith(0.02),
            false
        )]
    );
    let (g, _) = goal::<goal::EatBlock>(KEY, json, ConvertMode::Strict);
    let mut g = g.expect("kept");
    assert!(!g.converted);
    assert_eq!(molang_specs(&mut g)[0].1, ReaderKind::StrictVersioned);
}
