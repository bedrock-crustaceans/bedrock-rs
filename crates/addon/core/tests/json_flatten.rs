//! The canary of `#[facet(flatten)]` through the door: a struct that flattens a struct of
//! plain scalars, a Molang member, an untyped value and a nested struct is read, judged,
//! walked for unknown and `null` members and placed in its errors as if the flattened
//! members were its own. No JSON level belongs to the flattened field.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use bedrock_addon_core::id::MaterialId;
use bedrock_addon_core::json::{
    BeyondWidth, JsonSlice, ScalarFailure, ScalarRules, Scalars, Value, from_str, null_members,
    parse_at, parse_with_rules, unknown_members,
};
use bedrock_addon_core::molang::Molang;
use facet::Facet;
use indexmap::IndexMap;

#[derive(Debug, PartialEq, Facet)]
struct Outer {
    identifier: String,
    #[facet(flatten)]
    slots: Slots,
    #[facet(default)]
    tail: Option<bool>,
}

#[derive(Debug, Default, PartialEq, Facet)]
struct Slots {
    #[facet(default)]
    flag: Option<bool>,
    #[facet(default)]
    count: Option<u8>,
    #[facet(default)]
    script: Option<Molang>,
    #[facet(default)]
    mev: Option<Value>,
    #[facet(default)]
    inner: Option<Inner>,
}

#[derive(Debug, Default, PartialEq, Facet)]
struct Inner {
    #[facet(default)]
    a: Option<u8>,
}

const STRICT: ScalarRules = ScalarRules::PLAIN
    .with_scalars(Scalars::Strict)
    .with_beyond_width(BeyondWidth::Error);

#[test]
fn a_flattened_struct_reads_its_members_at_the_parent_level() {
    let text = r#"{"identifier": "x", "flag": true, "count": 3, "script": "q.a", "mev": [1, 2], "inner": {"a": 1}, "tail": false}"#;
    let read = parse_with_rules::<Outer>(JsonSlice::new(text), None, STRICT).expect("reads");
    assert!(read.told.is_empty(), "{:?}", read.told);
    let outer = read.value;
    assert_eq!(outer.identifier, "x");
    assert_eq!(outer.tail, Some(false));
    assert_eq!(outer.slots.flag, Some(true));
    assert_eq!(outer.slots.count, Some(3));
    assert!(matches!(outer.slots.script, Some(Molang::Expr(_))));
    assert_eq!(outer.slots.mev, Some(from_str::<Value>("[1, 2]").unwrap()));
    assert_eq!(outer.slots.inner, Some(Inner { a: Some(1) }));
}

#[test]
fn a_flattened_structs_scalars_are_judged_under_the_strict_rule() {
    for (member, text) in [
        ("flag", r#"{"identifier": "x", "flag": "true"}"#),
        ("count", r#"{"identifier": "x", "count": "3"}"#),
        ("inner/a", r#"{"identifier": "x", "inner": {"a": "1"}}"#),
    ] {
        match parse_with_rules::<Outer>(JsonSlice::new(text), None, STRICT) {
            Err(ScalarFailure::Read(error)) => {
                assert_eq!(error.relative_path().to_string(), member, "{text}");
            }
            other => panic!("{text}: {other:?}"),
        }
    }
}

#[test]
fn a_flattened_structs_unknown_members_are_found_below_it() {
    let json: Value =
        from_str(r#"{"identifier": "x", "inner": {"a": 1, "bogus": 2}, "flag": true, "nope": 1}"#)
            .unwrap();
    let unknown: Vec<String> = unknown_members(Outer::SHAPE, &json)
        .iter()
        .map(|(path, _)| path.to_string())
        .collect();
    assert_eq!(unknown, ["inner/bogus", "nope"]);
}

#[test]
fn a_flattened_structs_null_members_are_found() {
    let json: Value =
        from_str(r#"{"identifier": "x", "count": null, "inner": {"a": null}, "tail": null}"#)
            .unwrap();
    let nulls: Vec<String> = null_members(Outer::SHAPE, &json)
        .iter()
        .map(ToString::to_string)
        .collect();
    assert_eq!(nulls, ["count", "inner/a", "tail"]);
}

#[test]
fn an_error_inside_a_flattened_struct_has_no_step_for_the_field() {
    // facet's path names the flattened field as a step; the walk takes no member for it.
    let text = r#"{"identifier": "x", "inner": {"a": true}}"#;
    let error = parse_at::<Outer>(JsonSlice::new(text)).expect_err("a bool is no u8");
    let steps = error.error.path.as_ref().expect("a path").steps().len();
    assert_eq!(steps, 3, "slots, inner, a");
    assert_eq!(error.path.to_string(), "inner/a");
    // An integer out of range has neither a span nor a path from facet, flattened or not:
    // the error is at the slice either way.
    for text in [
        r#"{"identifier": "x", "inner": {"a": 300}}"#,
        r#"{"identifier": "x", "count": 300}"#,
    ] {
        let error = parse_at::<Outer>(JsonSlice::new(text)).expect_err("no u8");
        assert!(error.error.path.is_none(), "{text}");
        assert_eq!(error.path.to_string(), "", "{text}");
    }
    let error = parse_at::<Slots>(JsonSlice::new(r#"{"inner": {"a": 300}}"#)).expect_err("no u8");
    assert_eq!(error.path.to_string(), "");
}

/// A struct that flattens `Names`, with and without `default` on the field.
#[derive(Debug, Facet)]
struct Named {
    #[facet(flatten)]
    names: Names,
}

#[derive(Debug, Facet)]
struct NamedOrNot {
    #[facet(flatten, default)]
    names: Names,
}

#[derive(Debug, Default, Facet)]
struct Names {
    #[facet(default)]
    flags: IndexMap<String, bool>,
    #[facet(default)]
    ids: IndexMap<String, MaterialId>,
    #[facet(default)]
    list: Vec<MaterialId>,
}

/// facet's flatten reader (0.50.0-rc.7), pinned as it is: a flattened struct none of whose
/// members is written is a missing field unless the field is `default`; and where a proxied
/// value inside a flattened member refuses its JSON (an identifier given a number), the read
/// fails with "not initialized" at the end of the object, at the member, without the key or
/// the index below it. A plain value keeps its place. A family that needs the place reads a
/// struct with the members written out (the client entity's and the attachable's
/// `DescriptionRead`); the place is the walk's to recover once facet keeps it.
#[test]
fn facets_flatten_reader_needs_default_and_loses_a_refused_proxys_place() {
    assert!(parse_at::<Named>(JsonSlice::new("{}")).is_err());
    assert!(parse_at::<NamedOrNot>(JsonSlice::new("{}")).is_ok());
    let text = r#"{"flags": {"k": "x"}}"#;
    let error = parse_at::<NamedOrNot>(JsonSlice::new(text)).expect_err("a string is no bool");
    assert_eq!(error.path.to_string(), "flags/k");
    for (text, member, place) in [
        (r#"{"ids": {"k": 5}}"#, "ids", "ids/k"),
        (r#"{"list": ["a", 5]}"#, "list", "list/1"),
    ] {
        let error = parse_at::<NamedOrNot>(JsonSlice::new(text)).expect_err("a number is no id");
        assert_eq!(error.path.to_string(), member, "{text}");
        assert_eq!(error.span.len, 1, "{text}");
        assert_eq!(error.span.offset as usize, text.len() - 1, "{text}");
        assert!(
            error.error.to_string().contains("not initialized"),
            "{error}"
        );
        // Not flattened, the place is the value's.
        let error = parse_at::<Names>(JsonSlice::new(text)).expect_err("a number is no id");
        assert_eq!(error.path.to_string(), place, "{text}");
    }
}
