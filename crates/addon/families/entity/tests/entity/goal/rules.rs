//! The goal rules of: (strict typed parse), 16
//! (bounds), 17 (component value messages), and (triggers, filters, Molang).

use bedrock_addon_core::diagnostic::{DiagnosticCode, Severity, VanillaText};
use bedrock_addon_core::molang::{self, MolangFieldMut, MolangVersion, MolangVersionSource};
use bedrock_addon_core::versioning::ConvertMode;
use bedrock_addon_family_entity::goal;

use super::{assert_finding, errors, finding, goal, lenient, strict};

/// In a document at or above a goal's typed version, a member of the wrong JSON type
/// (including the integer-or-real distinction) is an `InvalidType` error that drops the goal.
#[test]
fn rule15_wrong_json_types_fail_the_strict_parse() {
    for (key, json, member) in [
        (
            "minecraft:behavior.float",
            r#"{"sink_with_passengers": 1}"#,
            "sink_with_passengers",
        ),
        (
            "minecraft:behavior.random_stroll",
            r#"{"interval": 1.5}"#,
            "interval",
        ),
        (
            "minecraft:behavior.random_stroll",
            r#"{"interval": "120"}"#,
            "interval",
        ),
        (
            "minecraft:behavior.breed",
            r#"{"priority": "2"}"#,
            "priority",
        ),
        (
            "minecraft:behavior.mingle",
            r#"{"mingle_partner_type": 3}"#,
            "mingle_partner_type",
        ),
        (
            "minecraft:behavior.drop_item_for",
            r#"{"target_range": [1, 2]}"#,
            "target_range",
        ),
    ] {
        let (value, d) = strict(key, json);
        assert!(value.is_none(), "{key} {json}: dropped");
        assert_finding(&d, "InvalidType", member, Severity::Error);
    }
}

/// For the named members, a name outside a member's set is a typed parse error.
#[test]
fn rule15_unknown_names_fail_the_strict_parse() {
    for (key, json, member) in [
        (
            "minecraft:behavior.move_to_poi",
            r#"{"poi_type": "kitchen"}"#,
            "poi_type",
        ),
        (
            "minecraft:behavior.avoid_block",
            r#"{"target_selection_method": "farthest"}"#,
            "target_selection_method",
        ),
        (
            "minecraft:behavior.play_dead",
            r#"{"damage_sources": ["fire", "boredom"]}"#,
            "damage_sources",
        ),
        (
            "minecraft:behavior.breed",
            r#"{"control_flags": ["move", "fly"]}"#,
            "control_flags",
        ),
        (
            "minecraft:behavior.transport_items",
            r#"{"place_strategy": "anywhere"}"#,
            "place_strategy",
        ),
    ] {
        let (value, d) = strict(key, json);
        assert!(value.is_none(), "{key} {json}: dropped");
        let found = d
            .iter()
            .find(|f| f.location.json_path.to_string().starts_with(member))
            .unwrap_or_else(|| panic!("{member}: {d:?}"));
        assert_eq!(found.severity, Severity::Error);
        assert!(
            matches!(
                found.code,
                DiagnosticCode::InvalidType { .. } | DiagnosticCode::InvalidEnumValue { .. }
            ),
            "{found:?}"
        );
    }
    let (g, d) = goal::<goal::MoveToPoi>(
        "minecraft:behavior.move_to_poi",
        r#"{"poi_type": "jobsite"}"#,
        ConvertMode::Strict,
    );
    assert!(d.is_empty());
    assert_eq!(g.expect("kept").poi_type(), Some(goal::PoiType::Jobsite));
    let (g, d) = goal::<goal::Celebrate>(
        "minecraft:behavior.celebrate",
        r#"{"celebration_sound": "celebrate"}"#,
        ConvertMode::Strict,
    );
    assert!(d.is_empty());
    assert_eq!(g.expect("kept").celebration_sound(), "celebrate");
}

/// A sound event name the crate does not know (vanilla adds names over time) is kept with an
/// `UnknownSound` warning in both modes; a wrong JSON type is still a parse error. The
/// string-typed sound members of the samples schema are sound events too.
#[test]
fn unknown_sound_names_are_warnings() {
    for mode in [ConvertMode::Strict, ConvertMode::Lenient] {
        let (g, d) = goal::<goal::Celebrate>(
            "minecraft:behavior.celebrate",
            r#"{"celebration_sound": "party"}"#,
            mode,
        );
        assert_eq!(g.expect("kept").celebration_sound(), "party");
        assert_finding(&d, "UnknownSound", "celebration_sound", Severity::Info);
        assert!(errors(&d).is_empty());
    }
    let (g, d) = strict(
        "minecraft:behavior.celebrate",
        r#"{"celebration_sound": 3}"#,
    );
    assert!(g.is_none());
    assert_finding(&d, "InvalidType", "celebration_sound", Severity::Error);
    for (key, json, member) in [
        (
            "minecraft:behavior.admire_item",
            r#"{"admire_item_sound": "nope"}"#,
            "admire_item_sound",
        ),
        (
            "minecraft:behavior.eat_mob",
            r#"{"eat_mob_sound": "nope"}"#,
            "eat_mob_sound",
        ),
        (
            "minecraft:behavior.sonic_boom",
            r#"{"attack_sound": "nope"}"#,
            "attack_sound",
        ),
        (
            "minecraft:behavior.sonic_boom",
            r#"{"charge_sound": "nope"}"#,
            "charge_sound",
        ),
        (
            "minecraft:behavior.sneeze",
            r#"{"prepare_sound": "nope"}"#,
            "prepare_sound",
        ),
        ("minecraft:behavior.sneeze", r#"{"sound": "nope"}"#, "sound"),
    ] {
        let (g, d) = strict(key, json);
        assert!(g.is_some(), "{key}");
        assert_finding(&d, "UnknownSound", member, Severity::Info);
    }
    let (g, d) = goal::<goal::Sneeze>(
        "minecraft:behavior.sneeze",
        r#"{"sound": "sneeze", "prepare_sound": "prepare.sneeze"}"#,
        ConvertMode::Strict,
    );
    let g = g.expect("kept");
    assert!(
        d.iter().all(|f| f.code.name() != "UnknownSound") || g.sound() == "sneeze",
        "{d:?}"
    );
}

/// `move_to_liquid.material_type` is bound capitalised and written lower-case by vanilla's own
/// files: either case reads.
#[test]
fn liquid_material_reads_either_case() {
    for (text, material) in [
        ("any", goal::LiquidMaterial::Any),
        ("Water", goal::LiquidMaterial::Water),
        ("LAVA", goal::LiquidMaterial::Lava),
    ] {
        let json = format!(r#"{{"material_type": "{text}"}}"#);
        let (g, d) = goal::<goal::MoveToLiquid>(
            "minecraft:behavior.move_to_liquid",
            &json,
            ConvertMode::Strict,
        );
        assert!(d.is_empty(), "{text}: {d:?}");
        assert_eq!(g.expect("kept").material_type(), material);
        // The spelling read is written back.
        let d = super::round_trip::<goal::MoveToLiquid>(
            "minecraft:behavior.move_to_liquid",
            &json,
            ConvertMode::Strict,
        );
        assert!(d.is_some_and(|d| d.is_empty()));
        let (g, _) = goal::<goal::MoveToLiquid>(
            "minecraft:behavior.move_to_liquid",
            &json,
            ConvertMode::Strict,
        );
        let written = bedrock_addon_core::json::to_json(&g.expect("kept")).expect("writes");
        assert!(written.contains(&format!("\"{text}\"")), "{written}");
    }
}

/// `swim_up_for_breath.material_type` names the same liquid materials as `move_to_liquid`
/// Either case reads, the spelling read is written back, and
/// a name that is not a liquid material fails the strict parse.
#[test]
fn swim_up_for_breath_material_is_a_liquid_material() {
    const KEY: &str = "minecraft:behavior.swim_up_for_breath";
    let (g, _) = goal::<goal::SwimUpForBreath>(KEY, "{}", ConvertMode::Strict);
    assert_eq!(
        g.expect("kept").material_type(),
        goal::LiquidMaterial::Water
    );
    for (text, material) in [
        ("water", goal::LiquidMaterial::Water),
        ("Water", goal::LiquidMaterial::Water),
        ("LAVA", goal::LiquidMaterial::Lava),
        ("any", goal::LiquidMaterial::Any),
    ] {
        let json = format!(r#"{{"material_type": "{text}"}}"#);
        let (g, _) = goal::<goal::SwimUpForBreath>(KEY, &json, ConvertMode::Strict);
        assert_eq!(g.expect("kept").material_type(), material, "{text}");
        let d = super::round_trip::<goal::SwimUpForBreath>(KEY, &json, ConvertMode::Strict);
        assert!(d.is_some_and(|d| d.is_empty()), "{text}");
        let (g, _) = goal::<goal::SwimUpForBreath>(KEY, &json, ConvertMode::Strict);
        let written = bedrock_addon_core::json::to_json(&g.expect("kept")).expect("writes");
        assert!(written.contains(&format!("\"{text}\"")), "{written}");
    }
    let (value, d) = strict(KEY, r#"{"material_type": "mud"}"#);
    assert!(value.is_none(), "an unknown material drops the goal: {d:?}");
    let found = d
        .iter()
        .find(|f| {
            f.location
                .json_path
                .to_string()
                .starts_with("material_type")
        })
        .unwrap_or_else(|| panic!("material_type: {d:?}"));
    assert_eq!(found.severity, Severity::Error);
}

/// In a lenient conversion a `material_type` that
/// is no liquid material is a warning, the goal is kept and the member reads as its default.
#[test]
fn swim_up_for_breath_material_that_is_no_liquid_resets_leniently() {
    const KEY: &str = "minecraft:behavior.swim_up_for_breath";
    let (g, d) = goal::<goal::SwimUpForBreath>(
        KEY,
        r#"{"material_type": "mud", "search_height": 9}"#,
        ConvertMode::Lenient,
    );
    let g = g.unwrap_or_else(|| panic!("kept leniently: {d:?}"));
    assert_eq!(
        g.material_type(),
        goal::LiquidMaterial::Water,
        "the default"
    );
    assert_eq!(g.search_height(), 9, "the other members are read");
    let found: Vec<_> = d
        .iter()
        .filter(|f| {
            f.location
                .json_path
                .to_string()
                .starts_with("material_type")
        })
        .collect();
    assert_eq!(found.len(), 1, "{d:?}");
    assert_eq!(found[0].severity, Severity::Info);
    assert!(errors(&d).is_empty(), "{d:?}");
    // The reset member is not written back as the bad name.
    let written = bedrock_addon_core::json::to_json(&g).expect("writes");
    assert!(!written.contains("mud"), "{written}");
}

/// The schema bounds of every typed member. A violation is an error that drops the goal
/// in a strict parse, and a warning that resets the member to its default in a lenient one.
#[test]
fn rule16_bounds_are_errors_strictly_and_defaults_leniently() {
    for (key, json, member) in [
        (
            "minecraft:behavior.breed",
            r#"{"priority": -1}"#,
            "priority",
        ),
        (
            "minecraft:behavior.aquatic_charge_attack",
            r#"{"charge_speed": 0.01}"#,
            "charge_speed",
        ),
        (
            "minecraft:behavior.transport_items",
            r#"{"max_stack_size": 65}"#,
            "max_stack_size",
        ),
        (
            "minecraft:behavior.transport_items",
            r#"{"max_stack_size": 0}"#,
            "max_stack_size",
        ),
        (
            "minecraft:behavior.transport_items",
            r#"{"interaction_time": -1}"#,
            "interaction_time",
        ),
        (
            "minecraft:behavior.look_at_player",
            r#"{"angle_of_view_horizontal": 361}"#,
            "angle_of_view_horizontal",
        ),
        (
            "minecraft:behavior.look_at_player",
            r#"{"probability": 1.5}"#,
            "probability",
        ),
        (
            "minecraft:behavior.dragonstrafeplayer",
            r#"{"view_angle": 0}"#,
            "view_angle",
        ),
        (
            "minecraft:behavior.nearest_attackable_target",
            r#"{"scan_interval": 0}"#,
            "scan_interval",
        ),
        (
            "minecraft:behavior.jump_to_block",
            r#"{"search_width": 16}"#,
            "search_width",
        ),
    ] {
        let (value, d) = strict(key, json);
        assert!(value.is_none(), "{key} {json}: dropped");
        assert_finding(&d, "InvalidValue", member, Severity::Error);
        let (value, d) = lenient(key, json);
        assert!(value.is_some(), "{key} {json}: kept leniently");
        // The typed bound is ours in the lenient conversion: an information.
        assert_finding(&d, "InvalidValue", member, Severity::Info);
        assert!(errors(&d).is_empty());
    }
    let (g, _) = goal::<goal::TransportItems>(
        "minecraft:behavior.transport_items",
        r#"{"max_stack_size": 65}"#,
        ConvertMode::Lenient,
    );
    assert_eq!(g.expect("kept").max_stack_size(), 64, "the default");
    // Bounds apply to written values only: `work.work_in_rain_tolerance` defaults to −1, below
    // its minimum 0.
    let (g, d) = goal::<goal::Work>("minecraft:behavior.work", "{}", ConvertMode::Strict);
    assert!(d.is_empty());
    assert_eq!(g.expect("kept").work_in_rain_tolerance(), -1);
}

/// For `ranged_attack.attack_range` the typed range
/// binds `min` / `max` with no order constraint and no
/// swap, so a range written out of order is held as written, with no finding.
#[test]
fn a_ranged_attack_range_written_out_of_order_is_held_as_written() {
    let (g, d) = goal::<goal::RangedAttack>(
        "minecraft:behavior.ranged_attack",
        r#"{"attack_range": {"min": 15, "max": 4}}"#,
        ConvertMode::Strict,
    );
    assert!(d.is_empty(), "{d:?}");
    assert_eq!(g.expect("kept").attack_range(), (15.0, 4.0));
}

/// `charge_held_item.items` is required and holds at least one item.
#[test]
fn charge_held_item_needs_items() {
    let key = "minecraft:behavior.charge_held_item";
    let (value, d) = strict(key, r#"{"priority": 3}"#);
    assert!(value.is_none());
    assert_finding(&d, "MissingRequiredField", "items", Severity::Error);
    let (value, d) = strict(key, r#"{"items": []}"#);
    assert!(value.is_none());
    assert_finding(&d, "InvalidValue", "items", Severity::Error);
    let (value, d) = strict(key, r#"{"items": ["minecraft:fire_charge"]}"#);
    assert!(value.is_some() && d.is_empty(), "{d:?}");
}

/// `v12_component_value_messages`, `transport_items.search_distance: 0`: vanilla's
/// retail text; the default is used. An error in a strict parse, a warning in a lenient one.
#[expect(clippy::float_cmp, reason = "the reader gives exact values")]
#[test]
fn v12_transport_items_search_distance_must_be_positive() {
    let key = "minecraft:behavior.transport_items";
    let json = r#"{"search_distance": [0, 32]}"#;
    for (mode, severity) in [
        (ConvertMode::Strict, Severity::Error),
        (ConvertMode::Lenient, Severity::Error),
    ] {
        let (g, d) = goal::<goal::TransportItems>(key, json, mode);
        let found = finding(&d, "InvalidValue", "search_distance").expect("reported");
        assert_eq!(found.severity, severity);
        assert_eq!(
            found.code,
            DiagnosticCode::InvalidValue {
                text: Some(VanillaText::Fixed(
                    "behavior.transport_items: \"search_distance\" may only contain positive values"
                ))
            }
        );
        let g = g.expect("the goal is kept");
        assert_eq!(g.search_distance(), [64.0, 32.0], "the default is used");
    }
}

/// An entity document of `version` whose `components` hold `components` (JSON members).
fn load_entity(
    version: &str,
    components: &str,
) -> bedrock_addon_core::versioning::LoadOutcome<bedrock_addon_family_entity::Entity> {
    let text = format!(
        r#"{{"format_version": "{version}", "minecraft:entity": {{"description": {{"identifier": "demo:goal"}}, "components": {{{components}}}}}}}"#
    );
    bedrock_addon_core::versioning::load::<bedrock_addon_family_entity::Entity>(
        bedrock_addon_core::json::JsonSlice::new(&text),
        &bedrock_addon_core::load::LoadContext::detached(),
    )
}

/// `v06_generic_goal_suffix` through a document load: a missing suffix and a
/// 33-character suffix are `GenericGoalSuffixMissing` informations and the goal is dropped; a valid
/// suffix loads the goal. Needs the component map's key routing (`goal::versioned`,
/// `goal::GenericGoal::finding`, unit-tested in `goal/memory.rs`).
#[test]
fn v06_generic_goal_suffix_through_the_component_map() {
    let long = "a".repeat(33);
    let outcome = load_entity(
        "1.26.20",
        &format!(
            r#""minecraft:memory_behavior.move_to_position": {{"priority": 1}},
               "minecraft:memory_behavior.move_to_position.{long}": {{"priority": 1}},
               "minecraft:memory_behavior.move_to_position.gold": {{"priority": 1}}"#
        ),
    );
    let missing: Vec<String> = outcome
        .diagnostics
        .iter()
        .filter(|f| f.code == DiagnosticCode::GenericGoalSuffixMissing)
        // an information of the crate's own (no game line), the entity loads
        .inspect(|f| assert_eq!(f.severity, Severity::Info))
        .map(|f| f.location.json_path.to_string())
        .collect();
    assert_eq!(
        missing,
        [
            "minecraft:entity/components/minecraft:memory_behavior.move_to_position".to_owned(),
            format!(
                "minecraft:entity/components/minecraft:memory_behavior.move_to_position.{long}"
            ),
        ]
    );
    let Some(bedrock_addon_core::versioning::Loaded::Stable(entity)) = outcome.value else {
        panic!("the entity loads: {:?}", outcome.diagnostics);
    };
    let goals = entity
        .components
        .iter()
        .filter(|c| {
            matches!(
                c,
                bedrock_addon_core::document::ComponentRef::Typed {
                    value: bedrock_addon_family_entity::component::EntityComponent::MemoryBehaviorMoveToPosition(_),
                    ..
                }
            )
        })
        .count();
    assert_eq!(goals, 1, "only the suffixed goal loads");
}

/// triggers: a goal trigger is `EventTrigger`; a legacy `condition` is kept in its
/// `extra` and reported (the typed trigger has no condition).
#[test]
fn d5_goal_triggers_keep_and_report_a_legacy_condition() {
    let json =
        r#"{"on_start": {"event": "x:dig", "target": "self", "condition": "query.is_baby"}}"#;
    let (g, d) = goal::<goal::Dig>("minecraft:behavior.dig", json, ConvertMode::Lenient);
    let g = g.expect("kept");
    assert_finding(
        &d,
        "UnrecognizedField",
        "on_start/condition",
        Severity::Warning,
    );
    let trigger = g.on_start.expect("on_start");
    assert_eq!(trigger.event.as_deref(), Some("x:dig"));
    assert!(
        trigger.extra.get("condition").is_some(),
        "kept for write-back"
    );
    // The string form is the event on self.
    let (g, d) = goal::<goal::Dig>(
        "minecraft:behavior.dig",
        r#"{"on_start": "x:dig"}"#,
        ConvertMode::Strict,
    );
    assert!(d.is_empty());
    assert!(g.expect("kept").on_start.expect("on_start").bare);
}

/// The legacy `condition` of a goal trigger stays in the trigger's `extra` as it was
/// written; the Molang walk reads a copy of it (`molang::read`, where a number beyond `f32`
/// becomes the largest finite float) and that copy is never written. So such a number is
/// kept as written, and the one finding is the unknown member's: nothing is clamped in the
/// document, and there is no clamp to report.
#[test]
fn d5_a_legacy_condition_beyond_f32_is_kept_as_written_deviation() {
    let json = r#"{"on_start": {"event": "x:dig", "target": "self", "condition": 1e39}}"#;
    let (g, d) = goal::<goal::Dig>("minecraft:behavior.dig", json, ConvertMode::Lenient);
    let g = g.expect("kept");
    assert_eq!(d.iter().count(), 1, "{d:#?}");
    assert_finding(
        &d,
        "UnrecognizedField",
        "on_start/condition",
        Severity::Warning,
    );
    let trigger = g.on_start.expect("on_start");
    let kept = trigger.extra.get("condition").expect("kept for write-back");
    assert_eq!(
        kept.as_number().map(facet_value::VNumber::to_f64_lossy),
        Some(1e39)
    );
}

/// filters: an unknown filter test name in a goal's filters is a warning
/// ("Filter test not found -> %s"); the goal is kept.
#[test]
fn d5_an_unknown_filter_test_in_a_goal_is_a_warning() {
    let json = r#"{"filters": {"all_of": [{"test": "is_teapot", "value": true}]}}"#;
    let (value, d) = strict("minecraft:behavior.drink_milk", json);
    assert!(value.is_some());
    assert!(errors(&d).is_empty(), "{d:?}");
    let found = d
        .iter()
        .find(|f| {
            f.location
                .json_path
                .to_string()
                .starts_with("filters/all_of/0")
        })
        .unwrap_or_else(|| panic!("{d:?}"));
    assert_eq!(found.severity, Severity::Error);
    assert!(found.kept, "the goal is kept: a kept error");
    assert_eq!(
        found.code,
        DiagnosticCode::UnresolvedReference {
            text: Some(VanillaText::FilterTestNotFound {
                name: "is_teapot".to_owned()
            })
        }
    );
}

/// `entity_types` takes one descriptor or a list; members of a descriptor it does not define
/// are kept and reported at their path.
#[test]
fn entity_types_take_one_descriptor_or_a_list() {
    let key = "minecraft:behavior.avoid_mob_type";
    let one = r#"{"entity_types": {"filters": {"test": "is_family", "subject": "other", "value": "wolf"}, "max_dist": 6}}"#;
    let (g, d) = goal::<goal::AvoidMobType>(key, one, ConvertMode::Strict);
    assert!(d.is_empty(), "{d:?}");
    let types = g.expect("kept").entity_types.expect("entity_types");
    assert!(matches!(types, goal::EntityTypes::One(_)));
    assert_eq!(types.as_slice()[0].max_dist.map(|v| v.0), Some(6.0));

    let list = r#"{"entity_types": [{"max_dist": 6}, {"max_flee": 3, "zzz": 1}]}"#;
    let (g, d) = goal::<goal::AvoidMobType>(key, list, ConvertMode::Strict);
    assert_finding(
        &d,
        "UnrecognizedField",
        "entity_types/1/zzz",
        Severity::Warning,
    );
    let types = g.expect("kept").entity_types.expect("entity_types");
    assert_eq!(types.as_slice().len(), 2);
    assert!(types.as_slice()[1].extra.get("zzz").is_some());
}

/// Unknown members are kept and written back (the value round trip of ).
#[test]
fn unknown_goal_members_are_kept_and_written_back() {
    let json = r#"{"priority": 1, "speed_multiplier": 1.25, "demo_member": [1, 2]}"#;
    let d = super::round_trip::<goal::Breed>("minecraft:behavior.breed", json, ConvertMode::Strict)
        .expect("kept");
    assert_finding(&d, "UnrecognizedField", "demo_member", Severity::Warning);
}

/// Vectors take `[x, y, z]` and `{x, y, z}` and are written in the form read.
#[expect(clippy::float_cmp, reason = "the reader gives exact values")]
#[test]
fn vectors_take_both_forms() {
    let key = "minecraft:behavior.offer_flower";
    for json in [
        r#"{"search_area": [6, 2, 6]}"#,
        r#"{"search_area": {"x": 6, "y": 2}}"#,
    ] {
        let d =
            super::round_trip::<goal::OfferFlower>(key, json, ConvertMode::Strict).expect("kept");
        assert!(d.is_empty(), "{json}: {d:?}");
    }
    let (g, _) = goal::<goal::OfferFlower>(
        key,
        r#"{"search_area": {"x": 6, "y": 2}}"#,
        ConvertMode::Strict,
    );
    assert_eq!(g.expect("kept").search_area(), [6.0, 2.0, 0.0]);
}

/// The Molang members found by the pass-1 walk: `(path, version source, string-only)`.
fn molang_fields<T: molang::MolangFields>(
    value: &mut T,
) -> Vec<(String, MolangVersionSource, bool)> {
    let mut found = Vec::new();
    molang::visit_fields(
        value,
        &bedrock_addon_core::diagnostic::JsonPath::new(),
        |field| {
            let source = matches!(field.value, MolangFieldMut::Source(_));
            found.push((field.path.to_string(), field.spec.version, source));
        },
    );
    found
}

/// M1: `eat_block.success_chance` is a Molang value at the pack's version; in the lenient
/// conversion its object form is an `InvalidType` warning and the member takes its default.
#[test]
fn d5_m1_eat_block_success_chance_is_molang() {
    let key = "minecraft:behavior.eat_block";
    let (g, d) = goal::<goal::EatBlock>(
        key,
        r#"{"success_chance": "query.is_baby ? 0.02 : 0.001"}"#,
        ConvertMode::Strict,
    );
    assert!(d.is_empty(), "{d:?}");
    let mut g = g.expect("kept");
    assert_eq!(
        molang_fields(&mut g),
        [(
            "success_chance".to_owned(),
            MolangVersionSource::Pack,
            false
        )]
    );
    let object = r#"{"success_chance": {"expression": "0.5", "version": 12}}"#;
    let (g, d) = goal::<goal::EatBlock>(key, object, ConvertMode::Strict);
    assert!(
        d.is_empty(),
        "the object form reads at or above 1.26.40: {d:?}"
    );
    assert!(g.expect("kept").success_chance.is_some());
    let (g, d) = goal::<goal::EatBlock>(key, object, ConvertMode::Lenient);
    assert_finding(&d, "InvalidType", "success_chance", Severity::Info);
    assert!(g.expect("kept").success_chance.is_none());
}

/// M3: a descriptor's `tags` in a typed
/// goal is a string-only Molang member at the constant version 13.
#[test]
fn d5_m3_descriptor_tags_in_goals_are_molang_at_13() {
    let key = "minecraft:behavior.avoid_block";
    let json = r#"{"target_blocks": ["minecraft:magma", {"tags": "query.any_tag('stone')"}]}"#;
    let (g, d) = goal::<goal::AvoidBlock>(key, json, ConvertMode::Strict);
    assert!(d.is_empty(), "{d:?}");
    let mut g = g.expect("kept");
    assert_eq!(
        molang_fields(&mut g),
        [(
            "target_blocks/1/tags".to_owned(),
            MolangVersionSource::Constant(MolangVersion::V13),
            true
        )]
    );
    let key = "minecraft:behavior.place_block";
    let json = r#"{"randomly_placeable_blocks": [{"block": {"tags": "query.any_tag('dirt')"}, "weight": 2}]}"#;
    let (g, d) = goal::<goal::PlaceBlock>(key, json, ConvertMode::Strict);
    assert!(d.is_empty(), "{d:?}");
    let fields = molang_fields(&mut g.expect("kept"));
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].0, "randomly_placeable_blocks/0/block/tags");
}
