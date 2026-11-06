//! `Filter`: the grammar `group := test | [group] | {key: group | [group]}` with the
//! eight group keys kept apart, test members with the interim typing, the unknown-test-name
//! warning, and unknown members kept at every level.

#[path = "shared/common.rs"]
mod common;

use bedrock_addon_core::diagnostic::{DiagnosticCode, JsonPath, Severity, codes};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::shared::{Filter, FilterGroup, FilterScalar, FilterSubject, FilterTest};
use common::{compact, read, round_trip};

/// The test of a `Filter::Test`.
fn test_of(filter: &Filter) -> &FilterTest {
    match filter {
        Filter::Test(test) => test,
        other => panic!("not a test: {other:?}"),
    }
}

#[test]
fn a_test_reads_its_five_members() {
    let f: Filter = round_trip(
        r#"{"test": "has_equipment", "subject": "other", "operator": "!=", "domain": "hand", "value": "minecraft:shield"}"#,
    );
    let t = test_of(&f);
    assert_eq!(t.test, "has_equipment");
    assert_eq!(t.subject, Some(FilterSubject::Other));
    assert_eq!(t.operator.as_deref(), Some("!="));
    assert_eq!(t.domain, Some(FilterScalar::String("hand".into())));
    assert_eq!(
        t.value,
        Some(FilterScalar::String("minecraft:shield".into()))
    );
    assert!(t.extra.is_empty());
}

#[test]
fn absent_members_stay_absent() {
    let f: Filter = round_trip(r#"{"test": "is_daytime"}"#);
    let t = test_of(&f);
    assert_eq!(
        (t.subject, &t.operator, &t.domain, &t.value),
        (None, &None, &None, &None)
    );
}

#[test]
fn every_subject_and_operator_spelling_is_kept() {
    for subject in [
        "self", "other", "player", "target", "parent", "baby", "block", "damager", "holder", "item",
    ] {
        round_trip::<Filter>(&format!(
            r#"{{"test": "is_family", "subject": "{subject}"}}"#
        ));
    }
    for operator in ["=", "==", "equals", "!=", "<>", "not", "<", "<=", ">", ">="] {
        let f: Filter = round_trip(&format!(
            r#"{{"test": "rider_count", "operator": "{operator}", "value": 1}}"#
        ));
        assert_eq!(test_of(&f).operator.as_deref(), Some(operator));
    }
}

#[test]
fn value_and_domain_keep_their_json_type() {
    let f: Filter = round_trip(r#"{"test": "is_daytime", "value": true}"#);
    assert_eq!(test_of(&f).value, Some(FilterScalar::Bool(true)));
    let f: Filter = round_trip(r#"{"test": "rider_count", "value": 2}"#);
    assert_eq!(test_of(&f).value, Some(FilterScalar::Int(2)));
    let f: Filter = round_trip(r#"{"test": "random_chance", "value": 2.0}"#);
    assert_eq!(test_of(&f).value, Some(FilterScalar::Float(2.0)));
    let f: Filter = round_trip(r#"{"test": "is_brightness", "value": 0.1}"#);
    assert_eq!(test_of(&f).value, Some(FilterScalar::Float(0.1)));
    let f: Filter = round_trip(r#"{"test": "int_property", "domain": 3, "value": "1"}"#);
    assert_eq!(test_of(&f).domain, Some(FilterScalar::Int(3)));
    assert_eq!(test_of(&f).value, Some(FilterScalar::String("1".into())));
}

#[test]
fn a_list_and_every_group_key_are_read_and_kept_apart() {
    let f: Filter = round_trip(r#"[{"test": "is_daytime"}, {"test": "is_sneaking"}]"#);
    assert!(matches!(&f, Filter::List(list) if list.len() == 2));
    for key in [
        "all_of", "all", "AND", "any_of", "any", "OR", "none_of", "NOT",
    ] {
        let json = format!(r#"{{"{key}": [{{"test": "is_daytime"}}]}}"#);
        let f: Filter = round_trip(&json);
        let Filter::Group(group) = &f else {
            panic!("not a group")
        };
        let members = [
            &group.all_of,
            &group.all,
            &group.and,
            &group.any_of,
            &group.any,
            &group.or,
            &group.none_of,
            &group.not,
        ];
        assert_eq!(members.iter().filter(|m| m.is_some()).count(), 1, "{key}");
    }
}

#[test]
fn a_group_member_is_a_test_a_list_or_a_group() {
    let f: Filter = round_trip(
        r#"{"all_of": {"test": "is_daytime"}, "any_of": [{"none_of": [{"test": "is_sneaking"}]}, {"test": "on_ground"}]}"#,
    );
    let Filter::Group(group) = &f else {
        panic!("not a group")
    };
    assert!(matches!(group.all_of.as_deref(), Some(Filter::Test(_))));
    let Some(Filter::List(any)) = group.any_of.as_deref() else {
        panic!("any_of is not a list")
    };
    assert!(matches!(&any[0], Filter::Group(inner) if inner.none_of.is_some()));
}

#[test]
fn the_empty_group_is_the_default_and_unfiltered() {
    let f: Filter = round_trip("{}");
    assert_eq!(f, Filter::default());
    assert_eq!(f, Filter::Group(FilterGroup::default()));
    assert_eq!(compact(&Filter::default()), "{}");
    // Jigsaw samples write `null` members; they are absent.
    let f: Filter = read(r#"{"AND": null, "OR": null}"#).unwrap();
    assert_eq!(f, Filter::default());
}

#[test]
fn unknown_members_are_kept_at_every_level() {
    let json = r#"{"any_of": [{"test": "is_family", "value": "pig", "note": 1}, {"all_of": [], "x": [2]}], "y": true}"#;
    let f: Filter = round_trip(json);
    let Filter::Group(group) = &f else {
        panic!("not a group")
    };
    assert_eq!(group.extra.get("y"), Some(&Value::from(true)));
    let Some(Filter::List(any)) = group.any_of.as_deref() else {
        panic!("any_of is not a list")
    };
    assert_eq!(test_of(&any[0]).extra.get("note"), Some(&Value::from(1i64)));
    assert!(matches!(&any[1], Filter::Group(g) if g.extra.get("x").is_some()));
}

#[test]
fn an_unknown_test_name_is_a_kept_error_with_vanillas_text() {
    let f: Filter =
        read(r#"{"all_of": [{"test": "is_family", "value": "pig"}, {"test": "is_purple"}]}"#)
            .unwrap();
    let mut at = JsonPath::new();
    at.push("filters");
    let found = f.check(&at, Severity::Warning);
    assert_eq!(found.len(), 1);
    let finding = found.iter().next().unwrap();
    // The game logs an error at level 3 and the test evaluates false, so the document loads on:
    // a kept error.
    assert_eq!(finding.severity, Severity::Error);
    assert!(finding.kept);
    assert_eq!(
        finding.code,
        DiagnosticCode::from(codes::FILTER_TEST_NOT_FOUND.with(["is_purple"]))
    );
    let mut expected = at.clone();
    expected.push("all_of");
    expected.push_index(1);
    assert_eq!(finding.location.json_path, expected);
}

#[test]
fn the_tests_vanilla_registers_have_no_finding() {
    for name in [
        "is_family",
        "has_biome_tag",
        "has_component",
        "is_biome",
        "has_tag",
        "redstone_strength_at_position",
        "is_mark_variant",
        "in_lava",
        "has_memory",
    ] {
        let f = Filter::Test(FilterTest::new(name));
        assert!(
            f.check(&JsonPath::new(), Severity::Warning).is_empty(),
            "{name}"
        );
    }
}

#[test]
fn o5_numeric_subject_and_operator_read_by_vanillas_numbering_deviation() {
    // Vanilla 1.26 entity files (re-serialised by vanilla's cereal writer) carry
    // `"subject": 1, "operator": 0`; they are read and written back as names.
    let f: Filter =
        read(r#"{"test": "is_family", "subject": 1, "operator": 0, "value": "player"}"#).unwrap();
    let t = test_of(&f);
    assert_eq!(t.subject, Some(FilterSubject::Other));
    assert_eq!(t.operator.as_deref(), Some("=="));
    assert_eq!(
        compact(&f),
        r#"{"test":"is_family","subject":"other","operator":"==","value":"player"}"#
    );
    let f: Filter = read(r#"{"test": "has_component", "subject": 0, "operator": 1}"#).unwrap();
    assert_eq!(test_of(&f).subject, Some(FilterSubject::Self_));
    assert_eq!(test_of(&f).operator.as_deref(), Some("!="));
    assert!(read::<Filter>(r#"{"test": "is_family", "subject": 10}"#).is_err());
    assert!(read::<Filter>(r#"{"test": "is_family", "operator": 7}"#).is_err());
}

#[test]
fn malformed_tests_are_read_errors() {
    for json in [
        r#"{"test": 5}"#,
        r#"{"subject": "self", "value": 1}"#,
        r#"{"test": "is_family", "subject": "nobody"}"#,
        r#"{"test": "is_family", "subject": true}"#,
        r#"{"test": "is_family", "operator": true}"#,
        r#"{"test": "is_family", "value": [1]}"#,
        r#"{"test": "is_family", "value": {}}"#,
        r#"{"test": "rider_count", "value": 3000000000}"#,
        "5",
        r#""is_family""#,
        r#"{"all_of": 5}"#,
    ] {
        assert!(read::<Filter>(json).is_err(), "{json} must be rejected");
    }
}

#[test]
fn filter_test_and_group_read_on_their_own() {
    let t: FilterTest = round_trip(r#"{"test": "is_daytime", "value": false}"#);
    assert_eq!(t.value, Some(FilterScalar::Bool(false)));
    let g: FilterGroup = round_trip(r#"{"none_of": {"test": "is_daytime"}}"#);
    assert!(g.none_of.is_some());
    assert!(read::<FilterGroup>("[]").is_err());
    assert_eq!(
        round_trip::<FilterSubject>(r#""damager""#),
        FilterSubject::Damager
    );
    assert_eq!(round_trip::<FilterScalar>("-1"), FilterScalar::Int(-1));
}

#[test]
fn constructed_values_write_their_members() {
    let mut t = FilterTest::new("is_family");
    t.subject = Some(FilterSubject::Self_);
    t.value = Some(FilterScalar::String("monster".into()));
    let f = Filter::Group(FilterGroup {
        any_of: Some(Box::new(Filter::List(vec![Filter::Test(t)]))),
        ..FilterGroup::default()
    });
    assert_eq!(
        compact(&f),
        r#"{"any_of":[{"test":"is_family","subject":"self","value":"monster"}]}"#
    );
    assert_eq!(FilterSubject::default(), FilterSubject::Self_);
}
