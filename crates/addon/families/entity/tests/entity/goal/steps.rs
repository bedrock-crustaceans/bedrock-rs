//! The goal parts of the step rows of: what the lenient conversion
//! of a goal (a document below the goal's typed version) makes of the older shapes.

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon_core::shared::{EventTrigger, RangeForm};
use bedrock_addon_core::versioning::ConvertMode;
use bedrock_addon_family_entity::goal;

use super::{assert_finding, at_version, errors, fixture, fixture_member, goal};

/// The goal of type `T` the lenient conversion reads from `json`; no errors allowed.
fn convert<
    T: bedrock_addon_core::document::Component<
            Set = bedrock_addon_family_entity::component::EntityComponent,
        >,
>(
    key: &str,
    json: &str,
) -> (T, bedrock_addon_core::diagnostic::Diagnostics) {
    let (value, d) = goal::<T>(key, json, ConvertMode::Lenient);
    assert!(errors(&d).is_empty(), "{key}: {d:?}");
    (value.unwrap_or_else(|| panic!("{key} is kept: {d:?}")), d)
}

fn event(trigger: Option<&EventTrigger>) -> Option<&str> {
    trigger.and_then(|t| t.event.as_deref())
}

/// `e33_1_26_30_on_kill_overlap`, the lenient conversion: on each of the five melee
/// goals, `on_kill` alone or with `on_attack` becomes `on_attack`; `on_attack` alone is kept.
#[test]
fn e33_on_kill_folds_into_on_attack_on_the_five_melee_goals() {
    let cases = fixture("e33_on_kill_overlap");
    let expected = [
        ("on_kill_only", Some("x:killed")),
        ("both", Some("x:killed")),
        ("on_attack_only", Some("x:hit")),
    ];
    for (case, on_attack) in expected {
        let json = &cases.iter().find(|(k, _)| k == case).expect("case").1;
        let k = "minecraft:behavior.melee_attack";
        let (g, d) = convert::<goal::MeleeAttack>(k, json);
        assert_eq!(
            (event(g.on_attack.as_ref()), event(g.on_kill.as_ref())),
            (on_attack, None),
            "{k} {case}"
        );
        assert!(d.is_empty(), "no diagnostic: {d:?}");
        let k = "minecraft:behavior.melee_box_attack";
        let (g, _) = convert::<goal::MeleeBoxAttack>(k, json);
        assert_eq!(
            (event(g.on_attack.as_ref()), event(g.on_kill.as_ref())),
            (on_attack, None),
            "{k} {case}"
        );
        let k = "minecraft:behavior.delayed_attack";
        let (g, _) = convert::<goal::DelayedAttack>(k, json);
        assert_eq!(
            (event(g.on_attack.as_ref()), event(g.on_kill.as_ref())),
            (on_attack, None),
            "{k} {case}"
        );
        let k = "minecraft:behavior.stomp_attack";
        let (g, _) = convert::<goal::StompAttack>(k, json);
        assert_eq!(
            (event(g.on_attack.as_ref()), event(g.on_kill.as_ref())),
            (on_attack, None),
            "{k} {case}"
        );
        let k = "minecraft:behavior.use_kinetic_weapon";
        let (g, _) = convert::<goal::UseKineticWeapon>(k, json);
        assert_eq!(
            (event(g.on_attack.as_ref()), event(g.on_kill.as_ref())),
            (on_attack, None),
            "{k} {case}"
        );
    }
    // At or above 1.26.40 the two triggers are separate members.
    let json = fixture_member("e33_on_kill_overlap", "both");
    let (g, d) = goal::<goal::MeleeAttack>(
        "minecraft:behavior.melee_attack",
        &json,
        ConvertMode::Strict,
    );
    let g = g.expect("kept");
    assert!(d.is_empty());
    assert_eq!(
        (event(g.on_attack.as_ref()), event(g.on_kill.as_ref())),
        (Some("x:hit"), Some("x:killed"))
    );
}

/// `e33b_1_26_30_shape_migrations`: `ranged_attack` interval pairs, `beg.look_time`
/// and `float_wander.float_duration` arrays become `{min, max}` ranges in a 1.26.30 document;
/// the same shapes in a 1.26.40 document are strict errors.
#[test]
fn e33b_the_1_26_40_shape_migrations() {
    const NAME: &str = "e33b_shape_migrations";
    const RANGED: &str = "minecraft:behavior.ranged_attack";
    let ranged = |case| convert::<goal::RangedAttack>(RANGED, &fixture_member(NAME, case));

    let (g, d) = ranged("ranged_attack_pair");
    assert!(d.is_empty());
    let interval = g.attack_interval.expect("attack_interval");
    assert_eq!(
        (interval.min, interval.max, interval.form),
        (1.0, 3.0, RangeForm::MinMax)
    );

    let (g, _) = ranged("ranged_attack_max_only");
    assert_eq!(g.attack_interval(), (0.0, 4.0), "a missing half takes 0");

    let (g, d) = ranged("ranged_attack_scalar");
    assert!(d.is_empty());
    let interval = g.attack_interval.expect("attack_interval");
    assert_eq!(
        (interval.min, interval.max, interval.form),
        (2.5, 2.5, RangeForm::Scalar)
    );

    let (g, d) = ranged("ranged_attack_pair_and_scalar");
    assert_eq!(g.attack_interval(), (1.0, 3.0), "the pair wins");
    assert_finding(&d, "DroppedField", "attack_interval", Severity::Info);

    let (g, d) = convert::<goal::Beg>("minecraft:behavior.beg", &fixture_member(NAME, "beg"));
    assert!(d.is_empty());
    let look_time = g.look_time.expect("look_time");
    assert_eq!(
        (look_time.min, look_time.max, look_time.form),
        (2, 4, RangeForm::MinMax)
    );

    let (g, d) = convert::<goal::FloatWander>(
        "minecraft:behavior.float_wander",
        &fixture_member(NAME, "float_wander"),
    );
    assert!(d.is_empty());
    let duration = g.float_duration.expect("float_duration");
    assert_eq!(
        (duration.min, duration.max, duration.form),
        (0.1, 0.35, RangeForm::MinMax)
    );

    for (key, case, member) in [
        (RANGED, "ranged_attack_pair", "attack_interval_min"),
        (RANGED, "ranged_attack_max_only", "attack_interval_max"),
        ("minecraft:behavior.beg", "beg", "look_time"),
        (
            "minecraft:behavior.float_wander",
            "float_wander",
            "float_duration",
        ),
    ] {
        let (value, d) = super::strict(key, &fixture_member(NAME, case));
        assert!(value.is_none(), "{key} {case}: dropped at 1.26.40");
        assert_finding(&d, "InvalidValue", member, Severity::Error);
    }
}

/// `e34_1_26_40_ranged_projectile`, the goal part: `attack_radius_min` /
/// `attack_radius` become `attack_range` and `in_range_movement_mode` is `hold_position`, in the
/// lenient conversion of an older document and in the strict read of a 1.26.40 document; a
/// mode written before 1.26.50 is not a member yet (`UnrecognizedField`); the new shape reads
/// strictly at 1.26.50, where the old members are unknown.
#[test]
fn e34_ranged_attack_range_and_movement_mode() {
    const NAME: &str = "e34_ranged_attack";
    const RANGED: &str = "minecraft:behavior.ranged_attack";
    let (g, d) = convert::<goal::RangedAttack>(RANGED, &fixture_member(NAME, "radius"));
    assert!(d.is_empty());
    assert_eq!(g.attack_range(), (4.0, 15.0));
    assert_eq!(g.attack_interval(), (1.0, 3.0));
    assert_eq!(
        g.in_range_movement_mode,
        Some(goal::InRangeMovementMode::HoldPosition)
    );

    let (g, d) = convert::<goal::RangedAttack>(RANGED, &fixture_member(NAME, "follow"));
    assert_eq!(g.attack_range(), (0.0, 10.0));
    assert_finding(
        &d,
        "UnrecognizedField",
        "in_range_movement_mode",
        Severity::Info,
    );
    assert_eq!(
        g.in_range_movement_mode(),
        goal::InRangeMovementMode::HoldPosition
    );

    // A 1.26.40 document: the 1.26.40 struct, read strictly, then the 1.26.50 conversion.
    let (g, d) =
        at_version::<goal::RangedAttack>(RANGED, &fixture_member(NAME, "radius"), (1, 26, 40));
    assert!(d.is_empty(), "{d:?}");
    let g = g.expect("kept");
    assert_eq!(g.attack_range(), (4.0, 15.0));
    assert_eq!(g.attack_interval(), (1.0, 3.0));
    assert_eq!(
        g.in_range_movement_mode,
        Some(goal::InRangeMovementMode::HoldPosition)
    );
    assert!(
        d.unknown()
            .members_at(&bedrock_addon_core::diagnostic::JsonPath::new())
            .next()
            .is_none(),
        "the old members are taken"
    );
    let (g, d) =
        at_version::<goal::RangedAttack>(RANGED, &fixture_member(NAME, "new_shape"), (1, 26, 40));
    // The 1.26.40 struct has neither member: a strict slice refuses them.
    assert_finding(&d, "UnrecognizedField", "attack_range", Severity::Error);
    assert_finding(
        &d,
        "UnrecognizedField",
        "in_range_movement_mode",
        Severity::Error,
    );
    assert!(g.is_none(), "the document is refused");
    // The interval pair is an error at 1.26.40 (e33b), not converted.
    let (g, d) = at_version::<goal::RangedAttack>(
        RANGED,
        r#"{"attack_interval_min": 1, "attack_interval_max": 3, "attack_radius": 9}"#,
        (1, 26, 40),
    );
    assert!(g.is_none());
    assert_finding(&d, "InvalidValue", "attack_interval_min", Severity::Error);

    // A 1.26.50 document: the new shape, the old members unknown.
    let (g, d) =
        at_version::<goal::RangedAttack>(RANGED, &fixture_member(NAME, "new_shape"), (1, 26, 50));
    assert!(d.is_empty(), "{d:?}");
    assert_eq!(g.expect("kept").attack_range(), (4.0, 15.0));
    let (g, d) = at_version::<goal::RangedAttack>(RANGED, r#"{"attack_radius": 9}"#, (1, 26, 50));
    assert_finding(&d, "UnrecognizedField", "attack_radius", Severity::Error);
    assert!(g.is_none(), "the document is refused");
    let (g, d) = goal::<goal::RangedAttack>(
        RANGED,
        &fixture_member(NAME, "new_shape"),
        ConvertMode::Strict,
    );
    assert!(d.is_empty(), "{d:?}");
    assert_eq!(g.expect("kept").attack_range(), (4.0, 15.0));
}

/// `e24_1_21_89_leashable_breed`, the goal part: `breed`'s lenient conversion
/// with a wrong-typed member warns and uses the default.
#[test]
fn e24_breed_lenient_conversion_defaults_a_wrong_typed_member() {
    let json = fixture_member("steps", "e24_breed_wrong_type");
    let (g, d) = convert::<goal::Breed>("minecraft:behavior.breed", &json);
    assert_eq!(g.speed_multiplier, None);
    assert_eq!(g.speed_multiplier(), 1.0);
    assert_eq!(g.priority(), 3);
    assert_finding(&d, "InvalidValue", "speed_multiplier", Severity::Info);
}

/// `e29b_1_21_130_look_at_interact_jump`, the goal part: `min_look_time` /
/// `max_look_time` become `look_time` (a missing end 4), and `target_distance` is removed with
/// `DroppedField` (the crate's text; an information).
#[test]
fn e29b_look_at_goals_take_a_look_time_range() {
    let json = fixture_member("steps", "e29b_look_at_player");
    let (g, d) = convert::<goal::LookAtPlayer>("minecraft:behavior.look_at_player", &json);
    assert_eq!(g.look_time(), (3.0, 4.0));
    assert!(
        d.unknown()
            .members_at(&bedrock_addon_core::diagnostic::JsonPath::new())
            .next()
            .is_none(),
        "target_distance is gone"
    );
    let dropped = super::finding(&d, "DroppedField", "target_distance").expect("reported");
    // The game drops it without a word: an information.
    assert_eq!(dropped.severity, Severity::Info);
    assert_eq!(
        dropped.code,
        DiagnosticCode::from(
            core_codes::DROPPED_FIELD_WORDED
                .with(["`target_distance` was removed by the 1.26.0 upgrade"])
        )
    );
    assert_eq!(d.len(), 1, "{d:?}");
    for key in [
        "minecraft:behavior.look_at_target",
        "minecraft:behavior.look_at_entity",
        "minecraft:behavior.look_at_trading_player",
    ] {
        let (value, d) = super::lenient(key, &json);
        assert!(value.is_some() && d.len() == 1, "{key}: {d:?}");
    }
}

/// `e29c_1_21_130_typed_to_typed`, the goal part: `charge_speed_multiplier` is
/// `charge_speed` in an older document; at or above 1.26.0 it is a deprecated member, copied
/// into `charge_speed` only when that is absent.
#[test]
fn e29c_charge_speed_multiplier_becomes_charge_speed() {
    const KEY: &str = "minecraft:behavior.aquatic_charge_attack";
    let old = fixture_member("steps", "e29c_aquatic_old_name");
    let both = fixture_member("steps", "e29c_aquatic_both");

    let (g, d) = convert::<goal::AquaticChargeAttack>(KEY, &old);
    assert!(d.is_empty(), "the conversion renames silently: {d:?}");
    assert_eq!(g.charge_speed(), 0.5);

    // A typed read: the deprecated member is an error that rejects the document (in both
    // masks); the goal value itself is still read.
    let (g, d) = goal::<goal::AquaticChargeAttack>(KEY, &old, ConvertMode::Strict);
    let g = g.expect("kept");
    assert_finding(
        &d,
        "DeprecatedMember",
        "charge_speed_multiplier",
        Severity::Error,
    );
    assert_eq!(g.charge_speed(), 0.5, "copied when charge_speed is absent");
    assert!(
        d.unknown()
            .members_at(&bedrock_addon_core::diagnostic::JsonPath::new())
            .next()
            .is_none()
    );

    let (g, _) = goal::<goal::AquaticChargeAttack>(KEY, &both, ConvertMode::Strict);
    assert_eq!(
        g.expect("kept").charge_speed(),
        0.2,
        "an explicit charge_speed wins"
    );
}

/// `e25_1_21_90_enderman_renames`, the goal part: the renamed enderman goals with
/// the values the step writes (the 26-block list included) read as `place_block` and
/// `take_block`.
#[test]
fn e25_the_renamed_enderman_goals_read_as_typed_goals() {
    const NAME: &str = "e25_enderman_renames";
    let (place, d) = convert::<goal::PlaceBlock>(
        "minecraft:behavior.place_block",
        &fixture_member(NAME, "minecraft:behavior.place_block"),
    );
    assert!(d.is_empty(), "{d:?}");
    assert_eq!(place.xz_range(), (-1, 1));
    assert_eq!(place.y_range(), (0, 2));
    assert_eq!(place.chance(), 0.0005);

    let (take, d) = convert::<goal::TakeBlock>(
        "minecraft:behavior.take_block",
        &fixture_member(NAME, "minecraft:behavior.take_block"),
    );
    assert!(d.is_empty(), "{d:?}");
    assert_eq!(take.blocks.as_ref().map(Vec::len), Some(26));
    assert_eq!(take.y_range(), (0, 3));
    assert!(!take.requires_line_of_sight());
}

/// `e31c_1_26_10_follow_target_leader`, the goal part: the renamed goal with both
/// inserted defaults reads as `follow_target_leader`.
#[test]
fn e31c_follow_target_leader_reads_its_inserted_defaults() {
    let json = fixture_member("steps", "e31c_follow_target_leader");
    let (g, d) =
        convert::<goal::FollowTargetLeader>("minecraft:behavior.follow_target_leader", &json);
    assert!(d.is_empty(), "{d:?}");
    assert_eq!(g.search_cooldown(), 1);
    assert!(g.leader_filters.is_some());
}

/// `e32_1_26_20_to_1_26_30`, the goal part: `pickup_items.stop_if_holding_item`
/// (struct default `false`) reads as written.
#[test]
fn e32_pickup_items_stop_if_holding_item() {
    let json = fixture_member("steps", "e32_pickup_items");
    let (g, d) = goal::<goal::PickupItems>(
        "minecraft:behavior.pickup_items",
        &json,
        ConvertMode::Strict,
    );
    let g = g.expect("kept");
    assert!(d.is_empty(), "{d:?}");
    assert_eq!(g.stop_if_holding_item, Some(false));
    assert!(!goal::PickupItems::default().stop_if_holding_item());
}

/// e00e, e00f, e01, e02, e04, e07, e09, e10, e11, e11b, e18, e21, e28, the goal
/// side: what the entity chain's A/B steps write into a goal reads as the typed goal with no
/// finding (the steps themselves are the chain's, F-ENTITY-DOC).
#[test]
fn the_goals_the_steps_write_read_as_typed_goals() {
    let outputs = fixture("step_outputs");
    assert_eq!(outputs.len(), 19);
    for (name, json) in &outputs {
        let key = name.rsplit(' ').next().expect("a key");
        let (value, d) = super::lenient(key, json);
        assert!(value.is_some(), "{name}: kept");
        if name.starts_with("e00f") {
            // The 1.16.0 A-op writes `set_persistent` into `melee_attack`, whose typed struct
            // (`MeleeAttackBase`) has no such member: kept and reported.
            assert_finding(&d, "UnrecognizedField", "set_persistent", Severity::Warning);
            assert_eq!(d.len(), 1, "{name}: {d:?}");
        } else {
            assert!(d.is_empty(), "{name}: {d:?}");
        }
    }
    let json = fixture_member(
        "step_outputs",
        "e28 minecraft:behavior.nearest_attackable_target",
    );
    let (g, _) = convert::<goal::NearestAttackableTarget>(
        "minecraft:behavior.nearest_attackable_target",
        &json,
    );
    let interval = g.attack_interval.expect("attack_interval");
    assert_eq!(
        (interval.min, interval.max, interval.form),
        // Read from the legacy spelling, written in the typed one.
        (2.0, 10.0, RangeForm::MinMax)
    );
    let json = fixture_member(
        "step_outputs",
        "e28 lone minecraft:behavior.nearest_attackable_target",
    );
    let (g, _) = convert::<goal::NearestAttackableTarget>(
        "minecraft:behavior.nearest_attackable_target",
        &json,
    );
    assert_eq!(
        g.attack_interval(),
        (10.0, 10.0),
        "a lone number is both ends"
    );
    let json = fixture_member("step_outputs", "e11b minecraft:behavior.drink_potion");
    let (g, _) = convert::<goal::DrinkPotion>("minecraft:behavior.drink_potion", &json);
    assert_eq!(g.speed_modifier(), -0.25);
    let potions = g.potions.expect("potions");
    let ids: Vec<i32> = potions.iter().map(goal::Potion::id).collect();
    assert_eq!(ids, [19, 12, 21, 14]);
}

/// `e00d_1_12_0_triggers_pushable`, the goal part: the iron golem's
/// `target_when_pushed` as the 1.13.0 A-op writes it, with vanilla's `minecraft:filters` /
/// `minecraft:all_of` key quirk: the descriptor's quirk key is an unknown member, kept and
/// reported.
#[test]
fn e00d_target_when_pushed_keeps_the_quirk_keys_as_unknown_members() {
    let json = r#"{"priority": 1, "percent_chance": 5.0, "entity_types": [{"minecraft:filters": {"minecraft:all_of": [
        {"test": "is_family", "subject": "other", "value": "monster"},
        {"test": "is_family", "subject": "other", "operator": "!=", "value": "creeper"}]}}]}"#;
    let (g, d) = convert::<goal::TargetWhenPushed>("minecraft:behavior.target_when_pushed", json);
    assert_finding(
        &d,
        "UnrecognizedField",
        "entity_types/0/minecraft:filters",
        Severity::Warning,
    );
    assert_eq!(d.len(), 1, "{d:?}");
    assert_eq!(g.percent_chance(), 5.0);
    let types = g.entity_types.expect("entity_types");
    let _ = types;
    let first = bedrock_addon_core::diagnostic::JsonPath::of_keys(["entity_types"]).index(0);
    assert!(d.unknown().get(&first, "minecraft:filters").is_some());
}

/// `e01c_1_16_0_component_rewrites`, the goal part: `drop_item_for.target_range`
/// `[6, 6, 6]` (from `max_dist: 6`) and `explore_outskirts.max_wait_time` 0.25 (from
/// `wait_time: 5` ticks) read as the typed goals.
#[test]
fn e01c_the_rewritten_members_read_as_typed_goals() {
    let (g, d) = convert::<goal::DropItemFor>(
        "minecraft:behavior.drop_item_for",
        r#"{"priority": 1, "target_range": [6, 6, 6]}"#,
    );
    assert!(d.is_empty(), "{d:?}");
    assert_eq!(g.target_range(), [6.0, 6.0, 6.0]);
    let (g, d) = convert::<goal::ExploreOutskirts>(
        "minecraft:behavior.explore_outskirts",
        r#"{"max_wait_time": 0.25}"#,
    );
    assert!(d.is_empty(), "{d:?}");
    assert_eq!(g.max_wait_time(), 0.25);
}

/// in the lenient conversion a failing value inside a list
/// takes its default by itself; its neighbours are kept.
#[test]
fn a_nested_lenient_failure_drops_only_the_failing_value() {
    let (g, d) = goal::<goal::DrinkPotion>(
        "minecraft:behavior.drink_potion",
        r#"{"potions": [{"id": 3, "chance": "x"}, {"id": 21}]}"#,
        ConvertMode::Lenient,
    );
    let potions = g.expect("kept").potions.expect("potions kept");
    assert_eq!(potions.len(), 2);
    assert_eq!(
        potions[0].chance(),
        1.0,
        "the failing member takes its default"
    );
    assert_eq!(potions[0].id(), 3);
    assert_eq!(potions[1].id(), 21);
    assert_finding(&d, "InvalidValue", "potions/0/chance", Severity::Info);
    // An `id` that does not read is −1: the legacy reader drops the potion with its
    // content-log error, a kept error.
    let (g, d) = goal::<goal::DrinkPotion>(
        "minecraft:behavior.drink_potion",
        r#"{"potions": [{"id": "x", "chance": 0.5}, {"id": 21}]}"#,
        ConvertMode::Lenient,
    );
    let potions = g.expect("kept").potions.expect("potions kept");
    assert_eq!(potions.len(), 1);
    assert_eq!(potions[0].id(), 21);
    assert!(
        d.iter().any(|f| f.kept
            && f.location.json_path.to_string() == "potions/0"
            && f.code.to_string() == "Drinkable potions must specify a valid potion id"),
        "{d:?}"
    );
}

/// When `leader_filters` is present and does not read, the legacy goal reader
/// logs its warning and returns: the members it reads after `leader_filters` (of this goal,
/// `search_cooldown`) keep their defaults; those read before (`speed_multiplier`,
/// `within_radius`, `follow_distance`) are read.
#[test]
fn a_bad_leader_filters_ends_the_legacy_read_of_follow_target_leader() {
    let (g, d) = goal::<goal::FollowTargetLeader>(
        "minecraft:behavior.follow_target_leader",
        r#"{"priority": 2, "speed_multiplier": 1.5, "within_radius": 9, "follow_distance": 4, "leader_filters": {"test": 5}, "search_cooldown": 40}"#,
        ConvertMode::Lenient,
    );
    let g = g.expect("kept");
    assert_eq!(
        g.search_cooldown, None,
        "read after leader_filters: its default"
    );
    assert!(
        g.speed_multiplier.is_some() && g.within_radius.is_some() && g.follow_distance.is_some()
    );
    let at_filters: Vec<_> = d
        .iter()
        .filter(|f| f.location.json_path.to_string() == "leader_filters")
        .collect();
    assert_eq!(at_filters.len(), 1, "{d:?}");
    assert_eq!(
        at_filters[0].code.to_string(),
        "'minecraft:follow_target_leader' | 'leader_filters' must specify filters to identify leader to follow."
    );
}

/// The `search_cooldown` the legacy reader never reaches after a failed
/// `leader_filters` is told as dropped (an information: the game is silent), so the
/// author's value is not lost without a word; an absent one is not told.
#[test]
fn the_search_cooldown_a_bad_leader_filters_leaves_unread_is_told_as_dropped() {
    let (_, d) = goal::<goal::FollowTargetLeader>(
        "minecraft:behavior.follow_target_leader",
        r#"{"priority": 2, "leader_filters": {"test": 5}, "search_cooldown": 40}"#,
        ConvertMode::Lenient,
    );
    let at: Vec<_> = d
        .iter()
        .filter(|f| f.location.json_path.to_string() == "search_cooldown")
        .map(|f| (f.code.name(), f.severity))
        .collect();
    assert_eq!(at, [("DroppedField", Severity::Info)], "{d:?}");
    let (_, d) = goal::<goal::FollowTargetLeader>(
        "minecraft:behavior.follow_target_leader",
        r#"{"priority": 2, "leader_filters": {"test": 5}}"#,
        ConvertMode::Lenient,
    );
    assert!(
        d.iter()
            .all(|f| f.location.json_path.to_string() != "search_cooldown"),
        "{d:?}"
    );
}
