//! The legacy members' Molang: event and trigger conditions and `shoot`
//! responses, kept as JSON, are compiled with the item's other fields.

use super::common::*;

/// A 1.21.0 item with `events` and `components` (JSON object members).
fn with_events(events: &str, components: &str) -> String {
    format!(
        r#"{{"format_version": "1.21.0", "minecraft:item": {{"description": {{"identifier": "demo:x"}}, "components": {{"minecraft:max_stack_size": 1{components}}}, "events": {{{events}}}}}}}"#
    )
}

///. The legacy expressions are a check-only cache (`item/expressions.rs`): the event
/// conditions and the `shoot` members are read through `molang::read_list` / `molang::read`,
/// where a number beyond `f32` becomes the largest finite float, but only that copy holds it.
/// The document keeps the JSON and writes the number as authored, so nothing is clamped in
/// what is written and there is no clamp to report. A legacy trigger's `condition` is read by
/// the strict reader, which refuses such a number (`InvalidType`, the member kept as written).
#[test]
fn a_legacy_expression_beyond_f32_is_written_as_authored_deviation() {
    use bedrock_addon_core::diagnostic::{DiagnosticCode, Severity};
    use bedrock_addon_core::document::Content;
    use bedrock_addon_core::json::Value;

    // The document with `@` where the number stands, the members down to the number from the
    // item, and whether the strict reader reads it.
    let cases: [(String, &[&str], bool); 4] = [
        (
            with_events(r#""demo:e": {"condition": @}"#, ""),
            &["events", "demo:e", "condition"],
            false,
        ),
        (
            with_events(r#""demo:e": {"condition": [@]}"#, ""),
            &["events", "demo:e", "condition", "0"],
            false,
        ),
        (
            with_events(
                r#""demo:fire": {"shoot": {"projectile": "minecraft:arrow", "launch_power": @}}"#,
                "",
            ),
            &["events", "demo:fire", "shoot", "launch_power"],
            false,
        ),
        (
            with_events(
                r#""demo:fire": {"shoot": {"projectile": "minecraft:arrow", "angle_offset": @}}"#,
                "",
            ),
            &["events", "demo:fire", "shoot", "angle_offset"],
            false,
        ),
    ];
    let listed = |d: &bedrock_addon_core::diagnostic::Diagnostics| {
        let mut all: Vec<String> = d.iter().map(|f| format!("{f:?}")).collect();
        all.sort();
        all
    };
    for (text, members, strict) in &cases {
        for (wide, number) in [("1e39", 1e39_f64), ("-1e39", -1e39), ("4e38", 4e38)] {
            let case = format!("{} = {wide}", members.join("/"));
            let document = loads(&text.replace('@', wide), &third_party());
            let plain = loads(&text.replace('@', "1"), &third_party());

            // The findings: none for the number where the cache alone holds the clamp; the
            // strict reader's refusal, at the number, where it reads.
            let path = format!("minecraft:item/{}", members.join("/"));
            let mut expected = listed(plain.diagnostics());
            if *strict {
                let refusals: Vec<_> = document
                    .diagnostics()
                    .iter()
                    .filter(|f| {
                        f.code == DiagnosticCode::InvalidType { text: None }
                            && f.severity == Severity::Warning
                            && f.location.json_path.to_string() == path
                    })
                    .collect();
                assert_eq!(refusals.len(), 1, "{case}: {:#?}", document.diagnostics());
                expected.push(format!("{:?}", refusals[0]));
                expected.sort();
            }
            assert_eq!(listed(document.diagnostics()), expected, "{case}");

            // `events` and the side-car are no members of the 1.26.50 format: the
            // newest writer writes the item at 1.26.0. The number is written as
            // authored, and reads back so.
            let written = written_at_1_26_0(&document.value().to_json());
            let read = |text: &str| {
                let root: Value = facet_json::from_str(text).unwrap();
                let mut value = root
                    .as_object()
                    .and_then(|root| root.get("minecraft:item"))
                    .cloned()
                    .expect("the item");
                for member in *members {
                    value = match member.parse::<usize>() {
                        Ok(index) => value.as_array().and_then(|a| a.get(index)).cloned(),
                        Err(_) => value.as_object().and_then(|o| o.get(member)).cloned(),
                    }
                    .unwrap_or_else(|| panic!("{case}: no {member} in\n{text}"));
                }
                value.as_number().map(facet_value::VNumber::to_f64_lossy)
            };
            assert_eq!(read(&written), Some(number), "{case}\n{written}");
            assert!(!written.contains("3.4028235e38"), "{case}\n{written}");
            assert_eq!(written_as_read(&written, &third_party()), written, "{case}");
        }
    }
}

/// `loadEvent` logs "Item event has both randomize and sequence tags.
/// Sequence will be ignored." at content-log level 2, area 16 when an event holds both, and
/// loads only the `randomize` (the
/// `sequence` is loaded only without it): a warning at the event's `sequence`, the
/// event kept as written.
#[test]
fn an_item_event_with_randomize_and_sequence_warns_and_ignores_the_sequence() {
    use bedrock_addon_core::diagnostic::{DiagnosticCode, Severity};

    let both = with_events(
        r#""demo:e": {"randomize": [{"weight": 1, "swing": {}}], "sequence": [{"swing": {}}]},
           "demo:ok": {"sequence": [{"swing": {}}]},
           "demo:nested": {"sequence": [{"randomize": [{"swing": {}}], "sequence": [{"swing": {}}]}]}"#,
        "",
    );
    let document = loads(&both, &third_party());
    let found: Vec<(String, Severity, String)> = document
        .diagnostics()
        .iter()
        .filter(|f| matches!(f.code, DiagnosticCode::IgnoredMember { .. }))
        .map(|f| {
            (
                f.location.json_path.to_string(),
                f.severity,
                f.code.to_string(),
            )
        })
        .collect();
    let text = "Item event has both randomize and sequence tags. Sequence will be ignored.";
    assert_eq!(
        found,
        [
            (
                "minecraft:item/events/demo:e/sequence".to_owned(),
                Severity::Warning,
                text.to_owned()
            ),
            (
                "minecraft:item/events/demo:nested/sequence/0/sequence".to_owned(),
                Severity::Warning,
                text.to_owned()
            ),
        ]
    );
    // the event is kept as written
    assert!(written_as_read(&both, &third_party()).contains(r#""randomize""#));
}

/// The `InvalidType` findings of a load, as (path, severity, kept).
fn wrong_typed(
    d: &bedrock_addon_core::diagnostic::Diagnostics,
) -> Vec<(String, bedrock_addon_core::diagnostic::Severity, bool)> {
    d.iter()
        .filter(|f| {
            f.code == bedrock_addon_core::diagnostic::DiagnosticCode::InvalidType { text: None }
        })
        .map(|f| (f.location.json_path.to_string(), f.severity, f.kept))
        .collect()
}

/// An event `condition` that is no expression. `ExpressionNode::parse(Json)`
/// reads a number or a bool as a constant, `null` in silence, and logs
/// "Type not convertible to Molang expression" at level 3 for an object; the
/// constructor's result is dropped, so the event and the item stay: an object is a
/// kept error, a `null` element (the crate's own refusal, no slot for it) an information.
#[test]
fn an_event_condition_that_is_no_expression_is_graded_by_the_game_line() {
    use bedrock_addon_core::diagnostic::Severity;
    let at = "minecraft:item/events/demo:e/condition";
    for (condition, expected) in [
        (r#"{"a": 1}"#, (Severity::Error, true)),
        (r#"[{"a": 1}]"#, (Severity::Error, true)),
        (
            r#"["query.is_sneaking", [{"a": 1}]]"#,
            (Severity::Error, true),
        ),
        (r"[null]", (Severity::Info, false)),
        // The object form is no exception, and it is not compiled
        (
            r#"{"expression": "query.is_sneaking", "version": 2}"#,
            (Severity::Error, true),
        ),
    ] {
        let json = with_events(
            &format!(r#""demo:e": {{"condition": {condition}, "swing": {{}}}}"#),
            "",
        );
        let document = loads(&json, &third_party());
        assert_eq!(
            wrong_typed(document.diagnostics()),
            [(at.to_owned(), expected.0, expected.1)],
            "{condition}"
        );
        let events = document.events.as_ref().unwrap().as_object().unwrap();
        let event = events.get("demo:e").unwrap().as_object().unwrap();
        assert!(event.get("swing").is_some(), "{condition}: the event stays");
    }
}

/// A legacy trigger's `condition` of another type than a string or the object form
/// is a cereal read error (`JSONCppSchemaReaderBase::asString` has no other type;
/// "invalid string" at code 0x8, a parse code of the item's mask 0xfbf8), so the item
/// is refused: an error that rejects.
#[test]
fn a_legacy_trigger_condition_of_another_type_refuses_the_item() {
    use bedrock_addon_core::diagnostic::Severity;
    // (A number or a bool is no string either.)
    for condition in [
        "5",
        "true",
        "null",
        r#"["query.is_sneaking"]"#,
        r#"{"a": 1}"#,
    ] {
        let json = item(
            "1.26.0",
            "demo:x",
            &format!(
                r#""minecraft:digger": {{"destroy_speeds": [{{"block": "minecraft:dirt", "speed": 2, "__deprecated_on_dig": {{"event": "demo:e", "condition": {condition}}}}}]}}"#
            ),
        );
        let at = "minecraft:item/components/minecraft:digger/destroy_speeds/0/__deprecated_on_dig/condition";
        let d = rejects(&json, &third_party());
        assert!(
            wrong_typed(&d).contains(&(at.to_owned(), Severity::Error, false)),
            "{condition}: {d:#?}"
        );
    }
}

/// A `shoot` `launch_power` / `angle_offset` of a type the typed node cannot parse
/// (`canParse` takes a number, a bool and a string) is "child '%s' not valid here."
/// at level 3 (`LogUnknownChildSchemaOption`, from `_validate` with the
/// loader's `false`) and fails the validation, so the leaf event is
/// inert: a kept error, the item loads, the siblings stay.
#[test]
fn a_wrong_typed_shoot_member_is_a_kept_error_and_makes_its_event_inert() {
    use bedrock_addon_core::diagnostic::Severity;
    for (field, value) in [
        ("launch_power", "null"),
        ("launch_power", "[1]"),
        ("angle_offset", r#"{"a": 1}"#),
    ] {
        let json = with_events(
            &format!(
                r#""demo:fire": {{"shoot": {{"projectile": "minecraft:arrow", "{field}": {value}}}, "swing": {{}}}},
                   "demo:other": {{"swing": {{}}}}"#
            ),
            "",
        );
        let document = loads(&json, &third_party());
        assert_eq!(
            wrong_typed(document.diagnostics()),
            [(
                format!("minecraft:item/events/demo:fire/shoot/{field}"),
                Severity::Error,
                true
            )],
            "{field} = {value}"
        );
        let events = document.events.as_ref().unwrap().as_object().unwrap();
        assert!(
            events
                .get("demo:fire")
                .unwrap()
                .as_object()
                .unwrap()
                .is_empty(),
            "{field} = {value}: the leaf event is inert"
        );
        assert!(
            events
                .get("demo:other")
                .unwrap()
                .as_object()
                .unwrap()
                .get("swing")
                .is_some(),
            "a sibling event stays"
        );
    }
}

#[cfg(not(feature = "molang"))]
#[test]
fn legacy_expressions_are_counted_without_the_molang_feature() {
    let json = with_events(r#""demo:e": {"condition": "query.is_sneaking"}"#, "");
    let d = findings(&json, &third_party());
    assert!(
        d.iter().any(|f| matches!(
            f.code,
            bedrock_addon_core::diagnostic::DiagnosticCode::MolangNotChecked { fields: 1 }
        )),
        "{d:#?}"
    );
}

#[cfg(feature = "molang")]
mod compiled {
    use std::sync::Arc;

    use bedrock_addon_core::document::Content as _;

    use bedrock_addon_core::diagnostic::{DiagnosticCode, Severity};
    use bedrock_addon_core::load::LoadContext;
    use bedrock_addon_core::molang::MolangCheck;

    use super::*;

    fn checking() -> LoadContext {
        let mut cx = third_party();
        cx.molang = Some(Arc::new(MolangCheck::new()));
        cx
    }

    /// The paths of the `InvalidMolang` findings, with their severities.
    fn invalid(d: &bedrock_addon_core::diagnostic::Diagnostics) -> Vec<(String, Severity)> {
        d.iter()
            .filter(|f| matches!(f.code, DiagnosticCode::InvalidMolang { .. }))
            .map(|f| (f.location.json_path.to_string(), f.severity))
            .collect()
    }

    #[test]
    fn an_event_condition_is_compiled_and_kept() {
        let json = with_events(
            r#""demo:e": {"sequence": [{"condition": "'a' + 1", "run_command": {"command": ["say hi"]}}]}"#,
            "",
        );
        let document = loads(&json, &checking());
        let found = invalid(document.diagnostics());
        assert!(
            found.contains(&(
                "minecraft:item/events/demo:e/sequence/0/condition".to_owned(),
                Severity::Error
            )),
            "{found:?}"
        );
        // the parser's line is level 3 and the item loads on: a kept error
        assert!(
            document
                .diagnostics()
                .iter()
                .filter(|f| matches!(f.code, DiagnosticCode::InvalidMolang { .. }))
                .all(|f| f.kept),
            "{:#?}",
            document.diagnostics()
        );
        // the value is kept as written (`events` is no member of the newest format, whose
        // writer refuses it; the as-written writer keeps it)
        assert!(refused_at(
            &document.value().to_json(),
            "minecraft:item/events"
        ));
        let written = written_as_read(&json, &checking());
        assert!(written.contains("'a' + 1"), "{written}");
        let valid = with_events(r#""demo:e": {"condition": "query.is_sneaking"}"#, "");
        assert!(invalid(loads(&valid, &checking()).diagnostics()).is_empty());
    }

    #[test]
    fn a_legacy_trigger_condition_is_compiled() {
        // a side-car trigger of a 1.21.0 document, and a digger trigger at 1.26.0 (the 1.26.30
        // format does not bind `__deprecated_on_dig`: an unknown member, which its strict slice
        // refuses)
        let side_car = r#"{"format_version": "1.21.0", "minecraft:item": {"description": {"identifier": "demo:x"}, "components": {"minecraft:max_stack_size": 1}, "__deprecated_components": {"on_use": {"on_use_trigger": {"event": "demo:e", "condition": "'a' + 1"}}}}}"#;
        // (the item is refused for the deprecated side-car; the Molang compile
        // check still tells of its trigger)
        let all = findings(side_car, &checking());
        assert!(
            all.iter()
                .any(|f| f.severity == Severity::Error && f.code.name() == "DeprecatedMember"),
            "{all:#?}"
        );
        // (nothing of the trigger is compiled: the load stops at the refusal)
        assert!(invalid(&all).is_empty(), "{all:#?}");
        let digger = item(
            "1.26.0",
            "demo:x",
            r#""minecraft:digger": {"destroy_speeds": [{"block": "minecraft:dirt", "speed": 2, "__deprecated_on_dig": {"event": "demo:e", "condition": "'a' + 1"}}]}"#,
        );
        let found = invalid(loads(&digger, &checking()).diagnostics());
        assert!(
            found.iter().any(|(p, severity)| p
                == "minecraft:item/components/minecraft:digger/destroy_speeds/0/__deprecated_on_dig/condition"
                && *severity == Severity::Error),
            "{found:?}"
        );
        assert!(
            loads(&digger, &checking())
                .diagnostics()
                .iter()
                .filter(|f| matches!(f.code, DiagnosticCode::InvalidMolang { .. }))
                .all(|f| f.kept),
            "a kept error"
        );
    }

    /// A rejected `launch_power` / `angle_offset` fails the event's response
    /// validation (`EventResponseCollection::validate`), the event gets status 3
    /// and `DefinitionEvent::evaluateEvent` does nothing for it: the
    /// LEAF event is inert, with all its responses; its siblings and its parent stay.
    /// The game parses no `condition` inside an ignored `sequence` (it loads the
    /// `randomize` alone), so only the `randomize` child's condition is told.
    #[test]
    fn the_conditions_of_an_ignored_sequence_are_not_compiled() {
        let json = with_events(
            r#""demo:e": {"randomize": [{"condition": "'a' + 1", "swing": {}}], "sequence": [{"condition": "'a' + 1", "swing": {}}]}"#,
            "",
        );
        let found = invalid(loads(&json, &checking()).diagnostics());
        assert_eq!(
            found,
            [(
                "minecraft:item/events/demo:e/randomize/0/condition".to_owned(),
                Severity::Error
            )]
        );
    }

    #[test]
    fn a_rejected_shoot_drops_the_event() {
        let json = with_events(
            r#""demo:fire": {"shoot": {"projectile": "minecraft:arrow", "launch_power": "'a' + 1"}, "swing": {}, "run_command": {"command": ["say hi"]}},
               "demo:other": {"swing": {}},
               "demo:seq": {"sequence": [{"shoot": {"projectile": "minecraft:arrow", "angle_offset": "'a' + 1"}, "swing": {}}, {"swing": {}}]}"#,
            "",
        );
        let document = loads(&json, &checking());
        let found = invalid(document.diagnostics());
        for path in [
            "minecraft:item/events/demo:fire/shoot/launch_power",
            "minecraft:item/events/demo:seq/sequence/0/shoot/angle_offset",
        ] {
            assert!(
                found.contains(&(path.to_owned(), Severity::Error)),
                "{path}: {found:?}"
            );
        }
        assert!(
            document
                .diagnostics()
                .iter()
                .filter(|f| matches!(f.code, DiagnosticCode::InvalidMolang { .. }))
                .all(|f| f.kept),
            "kept errors: the item loads"
        );
        let events = document.events.as_ref().unwrap().as_object().unwrap();
        let empty = |event: &bedrock_addon_core::json::Value| event.as_object().unwrap().is_empty();
        let fire = events.get("demo:fire").expect("the event stays registered");
        assert!(empty(fire), "the leaf event is inert, all responses gone");
        let other = events.get("demo:other").unwrap().as_object().unwrap();
        assert!(other.get("swing").is_some(), "a sibling event stays");
        let seq = events.get("demo:seq").unwrap().as_object().unwrap();
        let children = seq.get("sequence").unwrap().as_array().unwrap();
        assert_eq!(children.len(), 2, "the container keeps its children");
        assert!(empty(&children[0]), "the leaf child is inert");
        assert!(children[1].as_object().unwrap().get("swing").is_some());
        // a valid shoot stays
        let valid = with_events(
            r#""demo:fire": {"shoot": {"projectile": "minecraft:arrow", "launch_power": 2.0, "angle_offset": "query.is_sneaking"}}"#,
            "",
        );
        let document = loads(&valid, &checking());
        let events = document.events.as_ref().unwrap().as_object().unwrap();
        assert!(
            events
                .get("demo:fire")
                .unwrap()
                .as_object()
                .unwrap()
                .get("shoot")
                .is_some()
        );
    }
}
