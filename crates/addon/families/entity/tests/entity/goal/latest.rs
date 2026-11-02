//! The four goals typed at 1.26.60: `behavior.panic`, `behavior.tempt`, `behavior.float_tempt` and
//! `behavior.send_event`, read strictly from 1.26.60 and converted from older documents with
//! the legacy goal parser's rules.

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::document::Content;
use bedrock_addon_family_entity::goal::{FloatTempt, Panic, SendEvent, Tempt};

use super::{assert_finding, at_version, errors};
use crate::common::{codes, cx, stable, vanilla};

/// Every vanilla 1.26.60 file with these goals loads without a
/// finding at warning level, each goal read by its typed struct, and writes back what it read.
#[test]
fn the_vanilla_1_26_60_goals_load_typed() {
    for name in [
        "fox",
        "horse",
        "llama",
        "trader_llama",
        "ocelot",
        "sheep",
        "strider",
        "zombie_nautilus",
    ] {
        let text = vanilla(&format!("{name}.json"));
        let (entity, found) = stable(&text, &cx());
        // The files' own references (an event naming a group the file lacks) are told; no
        // goal is.
        let at_goals: Vec<String> = found
            .iter()
            .filter(|f| f.severity >= Severity::Warning)
            .map(|f| f.location.json_path.to_string())
            .filter(|path| path.contains("behavior."))
            .collect();
        assert!(at_goals.is_empty(), "{name}: {found:#?}");
        assert!(
            codes(&found, Severity::Error).is_empty(),
            "{name}: {found:#?}"
        );
        let sections = std::iter::once(&entity.components)
            .chain(entity.component_groups.values().map(|g| &g.components));
        let mut typed = 0;
        for map in sections {
            typed += usize::from(map.get::<Panic>().is_some());
            typed += usize::from(map.get::<Tempt>().is_some());
        }
        assert!(typed > 0, "{name}: no typed panic or tempt");
        let (again, _) = stable(&entity.to_json().unwrap(), &cx());
        assert_eq!(again, entity, "{name}: the written entity reads back");
    }
}

/// The fox's `tempt` (1.26.60) with its values, and the defaults of what it leaves out.
#[test]
fn the_fox_tempt_reads_its_members_and_defaults() {
    let (entity, _) = stable(&vanilla("fox.json"), &cx());
    let tempt = entity.components.get::<Tempt>().expect("a tempt");
    assert_eq!(tempt.priority(), 4);
    assert!((tempt.speed_multiplier() - 0.5).abs() < f32::EPSILON);
    assert!((tempt.within_radius() - 16.0).abs() < f32::EPSILON);
    assert!(tempt.can_get_scared());
    assert_eq!(tempt.items.as_ref().map(Vec::len), Some(2));
    assert!(!tempt.can_tempt_vertically());
    assert!(!tempt.can_tempt_while_ridden());
    assert_eq!(tempt.tempt_sound(), "undefined");
    assert_eq!(tempt.sound_interval(), (0.0, 0.0));
    assert_eq!(tempt.stop_distance(), None);
}

/// The typed defaults: an empty goal of each kind at 1.26.60.
#[test]
fn an_empty_goal_takes_the_typed_defaults() {
    let (panic, d) = at_version::<Panic>("minecraft:behavior.panic", "{}", (1, 26, 60));
    assert!(d.is_empty(), "{d:#?}");
    let panic = panic.expect("a panic");
    assert!((panic.speed_multiplier() - 1.0).abs() < f32::EPSILON);
    assert!(!panic.ignore_mob_damage() && !panic.prefer_water() && !panic.force());
    assert!(panic.damage_sources().is_empty());
    assert_eq!(panic.panic_sound(), "undefined");
    assert_eq!(panic.sound_interval(), (0.0, 0.0));
    let (float_tempt, d) =
        at_version::<FloatTempt>("minecraft:behavior.float_tempt", "{}", (1, 26, 60));
    assert!(d.is_empty(), "{d:#?}");
    assert_eq!(
        float_tempt.expect("a float_tempt").tempt_sound(),
        "undefined"
    );
    let json = r#"{"event_choices": [{"sequence": [{}]}]}"#;
    let (send, d) = at_version::<SendEvent>("minecraft:behavior.send_event", json, (1, 26, 60));
    assert!(d.is_empty(), "{d:#?}");
    let send = send.expect("a send_event");
    let choice = &send.event_choices.as_ref().unwrap()[0];
    assert!((choice.min_activation_range() - 1.0).abs() < f32::EPSILON);
    assert!((choice.max_activation_range() - 32.0).abs() < f32::EPSILON);
    assert!(
        choice.cooldown_time() == 0.0 && choice.cast_duration() == 0.0 && choice.weight() == 0.0
    );
    assert!(choice.look_at_target() && choice.do_casting());
    assert_eq!(choice.start_sound_event(), "");
    let stage = &choice.sequence.as_ref().unwrap()[0];
    assert!(stage.base_delay() == 0.0);
    assert_eq!((stage.event(), stage.sound_event()), ("", ""));
}

/// At the typed version, for goals as for the
/// components: a wrong type and an unknown member refuse the entity; a member written as
/// `null` is absent, with the information of a member written back as its default.
#[test]
fn a_typed_goal_refuses_a_wrong_type_and_an_unknown_member_and_reads_null_as_absent() {
    for (json, code, member) in [
        (
            r#"{"speed_multiplier": "fast"}"#,
            "InvalidType",
            "speed_multiplier",
        ),
        (r#"{"made_up": 1}"#, "UnrecognizedField", "made_up"),
    ] {
        let (goal, d) = at_version::<Panic>("minecraft:behavior.panic", json, (1, 26, 60));
        assert!(goal.is_none(), "{json}");
        let told: Vec<String> = errors(&d)
            .iter()
            .map(|f| {
                format!(
                    "{} {:?} {}",
                    f.code.name(),
                    f.severity,
                    f.location.json_path
                )
            })
            .collect();
        assert_eq!(told, [format!("{code} Error {member}")], "{json}: {d:#?}");
    }
    let (goal, d) = at_version::<Panic>(
        "minecraft:behavior.panic",
        r#"{"speed_multiplier": null, "force": null}"#,
        (1, 26, 60),
    );
    let goal = goal.expect("a panic");
    assert_eq!(goal.speed_multiplier, None);
    assert!(errors(&d).is_empty(), "{d:#?}");
    assert_finding(&d, "ValueRewritten", "speed_multiplier", Severity::Info);
    assert_finding(&d, "ValueRewritten", "force", Severity::Info);
}

/// Below 1.26.60 the legacy goal parser reads the goal: a wrong type keeps the default with a
/// warning, and the entity loads.
#[test]
fn an_older_document_s_goal_is_converted_leniently() {
    let (goal, d) = at_version::<Tempt>(
        "minecraft:behavior.tempt",
        r#"{"speed_multiplier": "fast", "within_radius": 4}"#,
        (1, 26, 50),
    );
    let goal = goal.expect("a tempt");
    assert_eq!(goal.speed_multiplier, None);
    assert!((goal.within_radius() - 4.0).abs() < f32::EPSILON);
    assert!(errors(&d).is_empty(), "{d:#?}");
    // The hand-written legacy reader takes the default without a word.
    assert_finding(&d, "InvalidValue", "speed_multiplier", Severity::Info);
}

/// The panic goal's damage sources in an older document: an empty list and `all`
/// beside other causes are told with the legacy texts, and the goal loads.
#[test]
fn the_legacy_panic_damage_sources_messages() {
    // An empty list is a level-3 line, a kept error; `all` beside other
    // causes a level-2 line, a warning.
    for (json, severity, text) in [
        (
            r#"{"damage_sources": []}"#,
            "Error",
            "No damage source provided in \"damage_sources\".",
        ),
        (
            r#"{"damage_sources": ["fire", "all"]}"#,
            "Warning",
            "\"all\" is not the only damage source in the list. The other damage sources can be removed because \"all\" covers every damage source.",
        ),
    ] {
        let (goal, d) = at_version::<Panic>("minecraft:behavior.panic", json, (1, 26, 50));
        assert!(goal.is_some(), "{json}");
        let found: Vec<String> = d
            .iter()
            .filter(|f| f.severity >= Severity::Warning)
            .map(|f| {
                format!(
                    "{} {:?} {} {}",
                    f.code.name(),
                    f.severity,
                    f.location.json_path,
                    f.code
                )
            })
            .collect();
        assert_eq!(
            found,
            [format!("InvalidValue {severity} damage_sources {text}")],
            "{json}"
        );
        // At the typed version the reader has no such rule.
        let (goal, d) = at_version::<Panic>("minecraft:behavior.panic", json, (1, 26, 60));
        assert!(goal.is_some() && d.is_empty(), "{json}: {d:#?}");
    }
}

/// The legacy `send_event` reader and its transform: a
/// negative minimum becomes 1, a negative maximum 512, a minimum above the maximum is
/// swapped, each with its text; the times go through whole ticks (truncated); the vanilla
/// evoker (1.26.0) converts without a warning.
#[test]
fn the_legacy_send_event_conversions() {
    let json = r#"{"event_choices": [
        {"min_activation_range": -2, "max_activation_range": -3, "cooldown_time": 2.03, "cast_duration": 1.5, "sequence": [{"base_delay": 0.07}]},
        {"min_activation_range": 9, "max_activation_range": 4}]}"#;
    let (goal, d) = at_version::<SendEvent>("minecraft:behavior.send_event", json, (1, 26, 50));
    let goal = goal.expect("a send_event");
    let choices = goal.event_choices.as_ref().unwrap();
    assert!((choices[0].min_activation_range() - 1.0).abs() < f32::EPSILON);
    assert!((choices[0].max_activation_range() - 512.0).abs() < f32::EPSILON);
    assert_eq!(choices[0].cooldown_time(), 40.0_f32 / 20.0);
    assert_eq!(choices[0].cast_duration(), 1.5);
    assert_eq!(
        choices[0].sequence.as_ref().unwrap()[0].base_delay(),
        1.0_f32 / 20.0
    );
    assert!((choices[1].min_activation_range() - 4.0).abs() < f32::EPSILON);
    assert!((choices[1].max_activation_range() - 9.0).abs() < f32::EPSILON);
    let texts: Vec<String> = d
        .iter()
        .filter(|f| f.severity >= Severity::Warning)
        .map(|f| {
            format!(
                "{} {:?} {} {}",
                f.code.name(),
                f.severity,
                f.location.json_path,
                f.code
            )
        })
        .collect();
    assert_eq!(
        texts,
        [
            "InvalidValue Warning event_choices/0/min_activation_range 'min_activation_range' must not be negative, defaulting to 1.0",
            "InvalidValue Warning event_choices/0/max_activation_range 'max_activation_range' should not be negative, setting to 512.",
            "InvalidValue Warning event_choices/1/min_activation_range 'min_activation_range' must be less than or equal to 'max_activation_range', swapping values.",
        ]
    );
    assert_finding(
        &d,
        "ValueRewritten",
        "event_choices/0/cooldown_time",
        Severity::Info,
    );
    assert_finding(
        &d,
        "ValueRewritten",
        "event_choices/0/sequence/0/base_delay",
        Severity::Info,
    );
    // The vanilla evoker of the 1.21.50 pack (format 1.26.0).
    let (entity, found) = stable(&vanilla("evocation_illager_1_26_0.json"), &cx());
    assert!(codes(&found, Severity::Warning).is_empty(), "{found:#?}");
    let sections = std::iter::once(&entity.components)
        .chain(entity.component_groups.values().map(|g| &g.components));
    let send = sections
        .filter_map(|map| map.get::<SendEvent>())
        .next()
        .expect("a typed send_event");
    let choice = &send.event_choices.as_ref().unwrap()[0];
    assert_eq!(choice.cast_duration(), 3.0);
    assert_eq!(choice.start_sound_event(), "cast.spell");
}

/// The four keys are typed entries of the component map from 1.26.60 (they were dynamic rows
/// kept as JSON, `component/mod.rs` `keep_json`), and the typed table has them at 1.26.60.
#[test]
fn the_1_26_60_goal_keys_are_typed_entries() {
    use bedrock_addon_core::document::ComponentRef;
    let (entity, _) = stable(&vanilla("fox.json"), &cx());
    let tempt = entity
        .components
        .iter()
        .find(|entry| matches!(entry, ComponentRef::Typed { name, .. } if *name == "minecraft:behavior.tempt"));
    assert!(tempt.is_some(), "the fox's tempt is no typed entry");
}

/// A goal written as `null` is absent (as a component is read), with the information of a place the write-back changes.
#[test]
fn a_goal_written_null_is_absent() {
    for version in ["1.26.60", "1.26.40"] {
        let text = format!(
            r#"{{"format_version": "{version}", "minecraft:entity": {{"description": {{"identifier": "demo:x"}}, "components": {{"minecraft:behavior.panic": null, "minecraft:behavior.float": null}}}}}}"#
        );
        let (entity, found) = stable(&text, &cx());
        assert!(entity.components.get::<Panic>().is_none(), "{version}");
        let told: Vec<String> = found
            .iter()
            .filter(|f| f.location.json_path.to_string().contains("behavior."))
            .map(|f| {
                format!(
                    "{:?} {} {}",
                    f.severity,
                    f.code.name(),
                    f.location.json_path
                )
            })
            .collect();
        assert_eq!(
            told,
            [
                "Info ValueRewritten minecraft:entity/components/minecraft:behavior.panic",
                "Info ValueRewritten minecraft:entity/components/minecraft:behavior.float",
            ],
            "{version}"
        );
    }
}

/// The adjacent versions of the four goals' typed slice: at 1.26.59 the legacy goal parser
/// reads each (a wrong type keeps the default with a warning), at 1.26.60 the typed struct
/// does (the same member refuses the entity).
#[test]
fn the_four_goals_are_typed_at_1_26_60_and_not_at_1_26_59() {
    fn bound<
        T: bedrock_addon_core::document::Component<
                Set = bedrock_addon_family_entity::component::EntityComponent,
            >,
    >(
        key: &str,
        json: &str,
        member: &str,
    ) {
        let (goal, d) = at_version::<T>(key, json, (1, 26, 59));
        assert!(goal.is_some(), "{key} 1.26.59: {d:#?}");
        assert!(errors(&d).is_empty(), "{key} 1.26.59: {d:#?}");
        assert!(
            // An information: the four goals' legacy reader is hand-written.
            d.iter()
                .any(|f| f.severity == Severity::Info
                    && f.location.json_path.to_string() == member),
            "{key} 1.26.59: {d:#?}"
        );
        let (goal, d) = at_version::<T>(key, json, (1, 26, 60));
        assert!(goal.is_none(), "{key} 1.26.60");
        let told: Vec<String> = errors(&d)
            .iter()
            .map(|f| format!("{} {}", f.code.name(), f.location.json_path))
            .collect();
        assert_eq!(
            told,
            [format!("InvalidType {member}")],
            "{key} 1.26.60: {d:#?}"
        );
    }
    bound::<Panic>("minecraft:behavior.panic", r#"{"force": "yes"}"#, "force");
    bound::<Tempt>(
        "minecraft:behavior.tempt",
        r#"{"within_radius": "near"}"#,
        "within_radius",
    );
    bound::<FloatTempt>(
        "minecraft:behavior.float_tempt",
        r#"{"within_radius": "near"}"#,
        "within_radius",
    );
    bound::<SendEvent>(
        "minecraft:behavior.send_event",
        r#"{"event_choices": [{"weight": "heavy"}]}"#,
        "event_choices/0/weight",
    );
}
