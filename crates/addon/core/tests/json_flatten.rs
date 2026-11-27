//! The canary of `#[facet(flatten)]` through the reader: a struct that flattens a struct of
//! plain scalars, a Molang member, an untyped value and a nested struct is read, judged,
//! walked for unknown and `null` members and placed in its errors as if the flattened
//! members were its own. No JSON level belongs to the flattened field.

use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
use bedrock_addon_core::json::kit::{Node, Profile, Read};
use bedrock_addon_core::json::parse::{Options, parse};
use bedrock_addon_core::json::{Value, from_str_jsonc, null_members_of_shape};
use bedrock_addon_core::molang::Molang;
use bedrock_addon_core::schema::{Schema, read_document};
use bedrock_addon_core::versioning::Reading;
use facet::Facet;

#[derive(Debug, PartialEq, Facet, Schema)]
struct Outer {
    identifier: String,
    #[facet(flatten)]
    slots: Slots,
    tail: Option<bool>,
}

#[derive(Debug, PartialEq, Facet, Schema)]
struct Slots {
    flag: Option<bool>,
    count: Option<u8>,
    script: Option<Molang>,
    mev: Option<Value>,
    inner: Option<Inner>,
}

#[derive(Debug, PartialEq, Facet, Schema)]
struct Inner {
    a: Option<u8>,
}

/// `text` read as `Outer` under the strict reading: the value and each finding as `path code`.
fn read(text: &str) -> (Option<Outer>, Vec<String>) {
    let parsed = parse(text, Options::default()).expect("JSON");
    let mut d = Diagnostics::default();
    let value = {
        let mut read = Read::new(Reading::OWN.strict(), &mut d).profile(Profile::FIRST_FAILURE);
        read_document::<Outer>(&Node::parsed(&parsed, JsonPath::new()), &mut read)
    };
    let told = d
        .iter()
        .map(|f| format!("{} {}", f.location.json_path, f.code.id()))
        .collect();
    (value, told)
}

#[test]
fn a_flattened_struct_reads_its_members_at_the_parent_level() {
    let text = r#"{"identifier": "x", "flag": true, "count": 3, "script": "q.a", "mev": [1, 2], "inner": {"a": 1}, "tail": false}"#;
    let (outer, told) = read(text);
    assert!(told.is_empty(), "{told:?}");
    let outer = outer.expect("reads");
    assert_eq!(outer.identifier, "x");
    assert_eq!(outer.tail, Some(false));
    assert_eq!(outer.slots.flag, Some(true));
    assert_eq!(outer.slots.count, Some(3));
    assert!(matches!(outer.slots.script, Some(Molang::Expr(_))));
    assert_eq!(outer.slots.mev, Some(from_str_jsonc("[1, 2]").unwrap()));
    assert_eq!(outer.slots.inner, Some(Inner { a: Some(1) }));
}

#[test]
fn a_flattened_structs_scalars_are_judged_under_the_strict_rule() {
    for (member, text) in [
        ("flag", r#"{"identifier": "x", "flag": "true"}"#),
        ("count", r#"{"identifier": "x", "count": "3"}"#),
        ("inner/a", r#"{"identifier": "x", "inner": {"a": "1"}}"#),
    ] {
        let (outer, told) = read(text);
        assert_eq!(outer, None, "{text}");
        assert_eq!(told, [format!("{member} core/invalid_type")], "{text}");
    }
}

#[test]
fn a_flattened_structs_unknown_members_are_told_after_the_owners() {
    let (outer, told) =
        read(r#"{"identifier": "x", "inner": {"a": 1, "bogus": 2}, "flag": true, "nope": 1}"#);
    assert!(outer.is_some());
    assert_eq!(
        told,
        // A flattened struct's members are read after its owner's declared members, so its
        // findings follow the owner's wherever the file has them: `nope` before `inner/bogus`.
        [
            "nope core/unrecognized_field",
            "inner/bogus core/unrecognized_field"
        ]
    );
}

#[test]
fn a_flattened_structs_null_members_are_found() {
    let json: Value =
        from_str_jsonc(r#"{"identifier": "x", "count": null, "inner": {"a": null}, "tail": null}"#)
            .unwrap();
    let nulls: Vec<String> = null_members_of_shape(Outer::SHAPE, &json)
        .iter()
        .map(ToString::to_string)
        .collect();
    assert_eq!(nulls, ["count", "inner/a", "tail"]);
}
