//! The kit's own rules, and four real readers written in its terms: each a translation of a
//! family's private reader, held to the findings that reader tells on the same fixtures
//! (captured from the family's reader as it is on main, before any family moves onto the kit).

mod crafting_catalog;
mod ddui;
mod pack_settings;
mod particle;

use super::*;
use crate::diagnostic::codes as core_codes;
use crate::json::{IsIntRule, WideInt};
use crate::versioning::{MemberFailure, UnknownMask};

/// A finding as the oracle captures were written: severity, path, code, span, kept. The code
/// is the legacy code's `Debug`, as the captures hold it.
fn rendered(found: &Diagnostics) -> Vec<String> {
    found
        .iter()
        .map(|f| {
            format!(
                "{:?}|{}|{}|{:?}|{}",
                f.severity,
                f.location.json_path,
                crate::diagnostic::testing::legacy_debug(f),
                f.location.span.map(|s| (s.offset, s.len)),
                f.kept
            )
        })
        .collect()
}

/// The untyped value of `text`.
fn value(text: &str) -> Value {
    crate::json::from_str_jsonc(text).unwrap_or_else(|error| panic!("{text}: {error}"))
}

/// Runs `read` on each fixture and requires what the family's reader gave: whether the value
/// was read, and every finding in order.
#[track_caller]
fn assert_reads_as(
    fixtures: &[&str],
    expected: &[(bool, &[&str])],
    read: impl Fn(&str, &mut Diagnostics) -> bool,
) {
    assert_eq!(
        fixtures.len(),
        expected.len(),
        "one expectation per fixture"
    );
    for (text, (loaded, findings)) in fixtures.iter().zip(expected) {
        let mut found = Diagnostics::default();
        let read = read(text, &mut found);
        assert_eq!(rendered(&found), *findings, "{text}");
        assert_eq!(read, *loaded, "{text}");
    }
}

const TOKEN: Coercing = Coercing::new(IsIntRule::Token, WideInt::LowWord);
const INTEGRAL: Coercing = Coercing::new(IsIntRule::Integral, WideInt::LowWord);

fn at(keys: &[&str]) -> JsonPath {
    JsonPath::of_keys(keys)
}

#[test]
fn a_member_written_null_is_absent_only_under_the_absent_null_rule() {
    let json = value(r#"{"a": null}"#);
    let root = Node::new(&json, JsonPath::new());
    for (reading, absent) in [
        (Reading::NULL_ABSENT, true),
        (Reading::DOCUMENT, false),
        (Reading::OWN, false),
    ] {
        let mut found = Diagnostics::default();
        let mut read = Read::new(reading, &mut found);
        let members = read.object(&root).expect("an object");
        assert_eq!(read.opt(&members, "a").is_none(), absent, "{reading:?}");
        assert_eq!(
            read.required(&members, "a").is_none(),
            absent,
            "{reading:?}"
        );
        assert!(read.opt(&members, "b").is_none());
        // NULL_ABSENT refuses a failed member: the missing member is an error.
        let expected: &[&str] = if absent {
            &["Error|a|MissingRequiredField { text: None }|None|false"]
        } else {
            &[]
        };
        assert_eq!(rendered(&found), expected, "{reading:?}");
    }
}

#[test]
fn a_failed_read_is_an_error_that_fails_the_owner_unless_the_member_takes_its_default() {
    let json = value(r#"{"a": 1}"#);
    let root = Node::new(&json, at(&["p"]));
    for (failure, severity) in [
        (MemberFailure::Refuse, "Error"),
        (MemberFailure::DefaultAndTell, "Info"),
        (MemberFailure::DropOwner, "Error"),
        (MemberFailure::DropElement, "Error"),
    ] {
        let mut found = Diagnostics::default();
        let mut read = Read::new(
            Reading::DOCUMENT.with_on_member_failure(failure),
            &mut found,
        );
        let members = read.object(&root).expect("an object");
        let a = read.required(&members, "a").expect("written");
        assert_eq!(read.string(&a), None);
        assert!(read.required(&members, "b").is_none());
        assert_eq!(read.failed(), severity == "Error");
        assert_eq!(
            rendered(&found),
            [
                format!("{severity}|p/a|InvalidType {{ text: None }}|None|false"),
                format!("{severity}|p/b|MissingRequiredField {{ text: None }}|None|false"),
            ]
        );
    }
}

#[test]
fn an_unknown_member_is_told_under_its_disposition_at_the_mask_s_severity() {
    let json = value(r#"{"known": 1, "x": 2, "y": [3]}"#);
    let root = Node::new(&json, JsonPath::new());
    for (disposition, code) in [
        (Disposition::Keep, "UnrecognizedField"),
        (Disposition::Report, "MemberNotKept"),
        (Disposition::Reject, "DroppedField"),
        (Disposition::Ignore, "IgnoredMember"),
    ] {
        for (mask, severity) in [
            (UnknownMask::Error, "Error"),
            (UnknownMask::Warn, "Warning"),
        ] {
            let mut found = Diagnostics::default();
            let mut read = Read::new(Reading::DOCUMENT.with_unknown(mask), &mut found);
            let members = read.object(&root).expect("an object");
            let kept = read.unknown(&members, &["known"], disposition);
            let names: Vec<&str> = kept.iter().map(|(key, _)| *key).collect();
            let expected: &[&str] = if disposition == Disposition::Keep {
                &["x", "y"]
            } else {
                &[]
            };
            assert_eq!(names, expected);
            assert_eq!(
                rendered(&found),
                [
                    format!("{severity}|x|{code} {{ text: None }}|None|false"),
                    format!("{severity}|y|{code} {{ text: None }}|None|false"),
                ]
            );
        }
    }
    // A reader the game runs in silence tells them as information.
    let mut found = Diagnostics::default();
    let mut read = Read::new(Reading::OWN, &mut found).unknown_severity(Severity::Info);
    let members = read.object(&root).expect("an object");
    read.unknown(&members, &["known", "y"], Disposition::Keep);
    assert_eq!(
        rendered(&found),
        ["Info|x|UnrecognizedField { text: None }|None|false"]
    );
}

#[test]
fn texts_word_the_kit_s_findings_as_a_family_s_game_does() {
    let json = value(r#"{"a": 1, "z": 0}"#);
    let root = Node::new(&json, JsonPath::new());
    let mut found = Diagnostics::default();
    let mut read = Read::new(Reading::DOCUMENT, &mut found).texts(Texts {
        wrong_type: Some("not a string"),
        missing: Some(crate::diagnostic::cereal::MISSING_REQUIRED_FIELD_TEXT),
        unknown: Some(crate::diagnostic::cereal::MEMBER_NOT_IN_SCHEMA_TEXT),
    });
    let members = read.object(&root).expect("an object");
    read.unknown(&members, &["a"], Disposition::Keep);
    let a = read.required(&members, "a").expect("written");
    read.string(&a);
    read.required(&members, "b");
    assert_eq!(
        rendered(&found),
        [
            r#"Error|z|UnrecognizedField { text: Some(Fixed("this member was found in the input, but is not present in the Schema")) }|None|false"#,
            r#"Error|a|InvalidType { text: Some(Fixed("not a string")) }|None|false"#,
            r#"Error|b|MissingRequiredField { text: Some(Fixed("missing required field")) }|None|false"#,
        ]
    );
}

#[test]
fn findings_span_the_text_of_their_value_when_the_node_has_it() {
    let text = r#"{"list": [1, {"deep": "x"}], "flag": 0}"#;
    let json = value(text);
    let root = Node::with_text(&json, at(&["root"]), JsonSlice { text, offset: 100 });
    let mut found = Diagnostics::default();
    let mut read = Read::new(Reading::DOCUMENT, &mut found);
    let members = read.object(&root).expect("an object");
    let list = read.required(&members, "list").expect("written");
    let elements = read.array(&list).expect("an array");
    assert_eq!(elements.len(), 2);
    let inner = elements.element(1).expect("two elements");
    let inner = read.object(&inner).expect("an object");
    let deep = read.required(&inner, "deep").expect("written");
    assert_eq!(read.bool(&deep), None);
    read.required(&inner, "missing");
    let flag = read.required(&members, "flag").expect("written");
    assert_eq!(read.bool(&flag), None);
    assert_eq!(
        rendered(&found),
        [
            "Error|root/list/1/deep|InvalidType { text: None }|Some((122, 3))|false",
            "Error|root/list/1/missing|MissingRequiredField { text: None }|Some((113, 13))|false",
            "Error|root/flag|InvalidType { text: None }|Some((137, 1))|false",
        ]
    );
    // Without the text, the same findings carry no span.
    let bare = Node::new(&json, at(&["root"]));
    let mut found = Diagnostics::default();
    let mut read = Read::new(Reading::DOCUMENT, &mut found);
    let members = read.object(&bare).expect("an object");
    let flag = read.required(&members, "flag").expect("written");
    assert_eq!(flag.span(), None);
    read.bool(&flag);
    assert_eq!(
        rendered(&found),
        ["Error|root/flag|InvalidType { text: None }|None|false"]
    );
}

#[test]
fn a_strict_reading_takes_a_scalar_of_its_own_kind_alone() {
    let json = value(r#"[true, "s", 2, 2.0, 2.5, 4294967296, null]"#);
    let root = Node::new(&json, JsonPath::new());
    let mut found = Diagnostics::default();
    let mut read = Read::new(Reading::DOCUMENT, &mut found);
    let elements = read.array(&root).expect("an array");
    let nodes: Vec<Node<'_>> = elements.iter().collect();
    let bools: Vec<_> = nodes.iter().map(|n| read.bool(n)).collect();
    let strings: Vec<_> = nodes.iter().map(|n| read.string(n)).collect();
    let numbers: Vec<_> = nodes.iter().map(|n| read.number(n)).collect();
    let token: Vec<_> = nodes.iter().map(|n| read.int(n, TOKEN)).collect();
    let integral: Vec<_> = nodes.iter().map(|n| read.int(n, INTEGRAL)).collect();
    assert_eq!(bools, [Some(true), None, None, None, None, None, None]);
    assert_eq!(strings, [None, Some("s"), None, None, None, None, None]);
    assert_eq!(
        numbers,
        [
            None,
            None,
            Some(2.0),
            Some(2.0),
            Some(2.5),
            Some(4_294_967_296.0),
            None
        ]
    );
    assert_eq!(token, [None, None, Some(2), None, None, None, None]);
    assert_eq!(integral, [None, None, Some(2), Some(2), None, None, None]);
    // Every refusal is a wrong type at the element: 6 + 6 + 3 + 6 + 5.
    assert_eq!(found.len(), 26);
    assert!(found.iter().all(|f| f.is(&core_codes::INVALID_TYPE)));
}

#[test]
fn a_json_cpp_reading_takes_a_bool_as_a_number_and_any_number_as_an_int() {
    let json = value(r#"[true, "5", 2.7, -2.7, 4294967298, null]"#);
    let root = Node::new(&json, JsonPath::new());
    let mut found = Diagnostics::default();
    let mut read = Read::new(Reading::OWN.with_scalars(Scalars::JsonCpp), &mut found);
    let elements = read.array(&root).expect("an array");
    let nodes: Vec<Node<'_>> = elements.iter().collect();
    let numbers: Vec<_> = nodes.iter().map(|n| read.number(n)).collect();
    let ints: Vec<_> = nodes.iter().map(|n| read.int(n, TOKEN)).collect();
    assert_eq!(
        numbers,
        [
            Some(1.0),
            None,
            Some(2.7),
            Some(-2.7),
            Some(4_294_967_298.0),
            None
        ]
    );
    // `asInt`: a real cut toward zero, an integer's low word.
    assert_eq!(ints, [None, None, Some(2), Some(-2), Some(2), None]);
    // A bool and a string are no int; a string and `null` no number.
    assert_eq!(found.len(), 2 + 3);
}

#[test]
fn a_value_of_another_shape_is_a_wrong_type_at_it() {
    let json = value(r#"{"o": [], "a": {}}"#);
    let root = Node::new(&json, JsonPath::new());
    let mut found = Diagnostics::default();
    let mut read = Read::new(Reading::DOCUMENT, &mut found);
    let members = read.object(&root).expect("an object");
    let o = read.required(&members, "o").expect("written");
    let a = read.required(&members, "a").expect("written");
    assert!(read.object(&o).is_none());
    assert!(read.array(&a).is_none());
    assert_eq!(
        rendered(&found),
        [
            "Error|o|InvalidType { text: None }|None|false",
            "Error|a|InvalidType { text: None }|None|false",
        ]
    );
}

#[test]
fn members_are_walked_in_file_order_and_got_without_a_word() {
    let json = value(r#"{"b": 1, "a": null, "c": 3}"#);
    let root = Node::new(&json, at(&["r"]));
    let mut found = Diagnostics::default();
    let mut read = Read::new(Reading::NULL_ABSENT, &mut found);
    let members = read.object(&root).expect("an object");
    let walked: Vec<(&str, String)> = members
        .iter()
        .map(|(key, node)| (key, node.path().to_string()))
        .collect();
    assert_eq!(
        walked,
        [
            ("b", "r/b".to_owned()),
            ("a", "r/a".to_owned()),
            ("c", "r/c".to_owned())
        ]
    );
    // `get` is the member as written, `null` included; `opt` applies the null rule.
    assert!(members.get("a").is_some_and(|a| a.value().is_null()));
    assert!(read.opt(&members, "a").is_none());
    assert!(members.contains("a") && !members.contains("z"));
    assert_eq!((members.len(), members.is_empty()), (3, false));
    read.unknown_member(&members.get("c").expect("written"), Disposition::Report);
    assert_eq!(
        rendered(&found),
        ["Warning|r/c|MemberNotKept { text: None }|None|false"]
    );
}

#[test]
fn a_finding_pushed_as_it_is_counts_toward_the_failure_only_when_it_rejects() {
    let mut found = Diagnostics::default();
    let mut read = Read::new(Reading::DOCUMENT, &mut found);
    read.push(Diagnostic::of(&core_codes::INVALID_VALUE).severity(Severity::Info));
    assert!(!read.failed());
    read.push(Diagnostic::of(&core_codes::INVALID_VALUE).into_kept());
    assert!(!read.failed());
    read.push(Diagnostic::of(&core_codes::INVALID_VALUE));
    assert!(read.failed());
    assert_eq!(read.reading(), Reading::DOCUMENT);
    assert_eq!(found.len(), 3);
}
