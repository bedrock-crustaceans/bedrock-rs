//! The writer with a document's `Unknown` table: each object's unknown members after its
//! known members, looked up by the object's path, in document order; a row whose object the
//! value no longer holds is left out and told (`DroppedField`, a warning of the write); a row
//! of a key the object holds replaces that member's value in place, untold.

#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "tests read fixtures and call facet-json directly"
)]
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, JsonPath, Phase, Severity};
use bedrock_addon_core::document::Unknown;
use bedrock_addon_core::json::{
    Indent, Value, WriteStyle, merge_unknown, write_json, write_json_with,
};
use facet::Facet;
use indexmap::IndexMap;

const COMPACT: WriteStyle = WriteStyle {
    indent: Indent::Compact,
    trailing_newline: false,
};

/// `value` written with `unknown`, and the findings of the write.
fn with<'f, T: Facet<'f>>(value: &T, unknown: &Unknown) -> (String, Vec<Diagnostic>) {
    let mut out = Vec::new();
    let told = write_json_with(value, unknown, &mut out, COMPACT).unwrap();
    (String::from_utf8(out).unwrap(), told)
}

fn path(keys: &[&str]) -> JsonPath {
    JsonPath::of_keys(keys)
}

#[derive(Debug, Default, Facet)]
struct Leaf {
    z: i32,
}

#[derive(Debug, Default, Facet)]
struct Tree {
    name: String,
    #[facet(default)]
    leaves: Vec<Leaf>,
    #[facet(default)]
    named: IndexMap<String, Leaf>,
    #[facet(default)]
    inner: Option<Leaf>,
}

fn tree() -> Tree {
    let mut named = IndexMap::new();
    named.insert("b".to_owned(), Leaf { z: 3 });
    Tree {
        name: "t".into(),
        leaves: vec![Leaf { z: 1 }, Leaf { z: 2 }],
        named,
        inner: Some(Leaf { z: 4 }),
    }
}

#[test]
fn an_empty_table_writes_what_write_json_writes() {
    let mut plain = Vec::new();
    write_json(&tree(), &mut plain, COMPACT).unwrap();
    let (written, told) = with(&tree(), &Unknown::new());
    assert_eq!(written.as_bytes(), plain);
    assert!(told.is_empty());
}

#[test]
fn each_object_has_its_unknown_members_after_its_known_ones() {
    let mut unknown = Unknown::new();
    unknown.put(&JsonPath::new(), "root_a", Value::from(1i64));
    unknown.put(&JsonPath::new(), "root_b", Value::from("x"));
    unknown.put(&path(&["leaves"]).index(1), "q", Value::from(true));
    unknown.put(&path(&["named", "b"]), "m", Value::from(5i64));
    unknown.put(&path(&["inner"]), "i", Value::from(1.5f64));
    let (written, told) = with(&tree(), &unknown);
    assert_eq!(
        written,
        concat!(
            r#"{"name":"t","leaves":[{"z":1},{"z":2,"q":true}],"named":{"b":{"z":3,"m":5}},"#,
            r#""inner":{"z":4,"i":1.5},"root_a":1,"root_b":"x"}"#
        )
    );
    assert!(told.is_empty(), "{told:?}");
}

#[test]
fn a_row_whose_object_is_gone_is_left_out_and_told() {
    let mut unknown = Unknown::new();
    unknown.put(&path(&["leaves"]).index(5), "q", Value::from(1i64));
    unknown.put(&path(&["name"]), "not_an_object", Value::from(1i64));
    let (written, told) = with(&tree(), &unknown);
    assert!(!written.contains("\"q\""), "{written}");
    let paths: Vec<String> = told
        .iter()
        .map(|finding| finding.location.json_path.to_string())
        .collect();
    assert_eq!(paths, ["leaves/5/q", "name/not_an_object"]);
    for finding in &told {
        assert!(
            matches!(finding.code, DiagnosticCode::DroppedField { .. }),
            "{finding:?}"
        );
        assert_eq!(finding.severity, Severity::Warning);
        assert_eq!(finding.phase, Phase::Write);
    }
}

#[test]
fn a_row_of_a_key_the_object_holds_replaces_its_value_in_place_untold() {
    let mut unknown = Unknown::new();
    unknown.put(&JsonPath::new(), "name", Value::from("again"));
    unknown.put(&JsonPath::new(), "other", Value::from(2i64));
    let (written, told) = with(&tree(), &unknown);
    assert!(written.starts_with(r#"{"name":"again","#), "{written}");
    assert!(written.ends_with(r#","other":2}"#), "{written}");
    assert_eq!(written.matches("\"name\"").count(), 1, "{written}");
    assert!(told.is_empty(), "{told:?}");
}

#[test]
fn merge_puts_the_rows_into_a_value_as_the_writer_writes_them() {
    let mut value: Value = facet_json::from_str(r#"{"a":{"b":[{"c":1}]},"d":2}"#).unwrap();
    let mut unknown = Unknown::new();
    unknown.put(&path(&["a", "b"]).index(0), "e", Value::from(3i64));
    unknown.put(&path(&["gone"]), "f", Value::from(4i64));
    let told = merge_unknown(&mut value, &unknown);
    let expected: Value = facet_json::from_str(r#"{"a":{"b":[{"c":1,"e":3}]},"d":2}"#).unwrap();
    assert_eq!(value, expected);
    assert_eq!(told.len(), 1);
    assert_eq!(told[0].location.json_path.to_string(), "gone/f");
}

#[derive(Debug, Facet)]
#[repr(u8)]
#[allow(dead_code)]
enum External {
    Named { name: String },
    Unit,
}

#[derive(Debug, Facet)]
#[facet(tag = "type")]
#[repr(u8)]
#[allow(dead_code)]
enum Internal {
    Named { name: String },
}

#[derive(Debug, Facet)]
#[facet(untagged)]
#[repr(u8)]
#[allow(dead_code)]
enum Untagged {
    Named { name: String },
    Number(u8),
}

#[test]
fn a_struct_variant_has_its_members_at_its_payload_object() {
    let mut unknown = Unknown::new();
    // An externally tagged payload is the object below its variant's key.
    unknown.put(&path(&["Named"]), "u", Value::from(1i64));
    let external = External::Named { name: "a".into() };
    assert_eq!(
        with(&external, &unknown).0,
        r#"{"Named":{"name":"a","u":1}}"#
    );
    // An internally tagged or untagged one is the object itself.
    let mut unknown = Unknown::new();
    unknown.put(&JsonPath::new(), "u", Value::from(1i64));
    let internal = Internal::Named { name: "a".into() };
    assert_eq!(
        with(&internal, &unknown).0,
        r#"{"type":"Named","name":"a","u":1}"#
    );
    let untagged = Untagged::Named { name: "a".into() };
    assert_eq!(with(&untagged, &unknown).0, r#"{"name":"a","u":1}"#);
}

#[test]
fn a_value_has_the_members_of_its_objects_by_path() {
    let value: Value = facet_json::from_str(r#"{"a":{"b":[{"c":1}]},"d":2}"#).unwrap();
    let mut unknown = Unknown::new();
    unknown.put(&path(&["a", "b"]).index(0), "e", Value::from(3i64));
    unknown.put(&JsonPath::new(), "f", Value::from(4i64));
    assert_eq!(
        with(&value, &unknown).0,
        r#"{"a":{"b":[{"c":1,"e":3}]},"d":2,"f":4}"#
    );
}

#[derive(Debug, Facet)]
struct Flattened {
    name: String,
    #[facet(flatten)]
    leaf: Leaf,
}

/// A flattened struct's members are its parent's: the rows of the object are at the parent's
/// path, after the flattened members; a row at the flattened field's own name has no object.
#[test]
fn a_flattened_struct_has_its_members_at_its_parent_object() {
    let value = Flattened {
        name: "f".into(),
        leaf: Leaf { z: 7 },
    };
    let mut unknown = Unknown::new();
    unknown.put(&JsonPath::new(), "a", Value::from(1i64));
    unknown.put(&path(&["leaf"]), "b", Value::from(2i64));
    let (written, told) = with(&value, &unknown);
    assert_eq!(written, r#"{"name":"f","z":7,"a":1}"#);
    let paths: Vec<String> = told
        .iter()
        .map(|finding| finding.location.json_path.to_string())
        .collect();
    assert_eq!(paths, ["leaf/b"]);
}

#[test]
fn under_known_wins_a_row_of_a_key_the_object_holds_is_left_out_untold() {
    use bedrock_addon_core::json::{Precedence, merge_unknown_with};
    let text = r#"{"name":"tree","leaf":{"size":1}}"#;
    let mut unknown = Unknown::new();
    unknown.put(&JsonPath::new(), "name", Value::from("again"));
    unknown.put(&JsonPath::new(), "other", Value::from(2i64));
    unknown.put(&path(&["leaf"]), "size", Value::from(9i64));
    unknown.put(&path(&["gone"]), "f", Value::from(4i64));
    let mut known: Value = facet_json::from_str(text).unwrap();
    let told = merge_unknown_with(&mut known, &unknown, Precedence::KnownWins);
    let expected: Value =
        facet_json::from_str(r#"{"name":"tree","leaf":{"size":1},"other":2}"#).unwrap();
    assert_eq!(known, expected);
    // A row whose object is gone is told under either precedence.
    assert_eq!(told.len(), 1, "{told:?}");
    assert_eq!(told[0].location.json_path.to_string(), "gone/f");
    // Today's rule, the default, is the unknown member's.
    assert_eq!(Precedence::default(), Precedence::UnknownWins);
    let mut unknown_wins: Value = facet_json::from_str(text).unwrap();
    let again = merge_unknown_with(&mut unknown_wins, &unknown, Precedence::UnknownWins);
    let mut plain: Value = facet_json::from_str(text).unwrap();
    assert_eq!(again, merge_unknown(&mut plain, &unknown));
    assert_eq!(unknown_wins, plain);
    let expected: Value =
        facet_json::from_str(r#"{"name":"again","leaf":{"size":9},"other":2}"#).unwrap();
    assert_eq!(plain, expected);
}

/// A whole-file kind that writes `{"name": <its name>}`, stating which side of a shared key
/// wins.
#[derive(Clone, Debug, PartialEq)]
struct Named<const KNOWN_WINS: bool>(&'static str);

impl<const KNOWN_WINS: bool> bedrock_addon_core::__private::Sealed for Named<KNOWN_WINS> {}

impl<const KNOWN_WINS: bool> bedrock_addon_core::document::Content for Named<KNOWN_WINS> {
    const NAME: &'static str = "named";
    const SIDES: &'static [bedrock_addon_core::pack_identity::PackSide] =
        &[bedrock_addon_core::pack_identity::PackSide::Behavior];
    const UNKNOWN_PRECEDENCE: bedrock_addon_core::json::Precedence = if KNOWN_WINS {
        bedrock_addon_core::json::Precedence::KnownWins
    } else {
        bedrock_addon_core::json::Precedence::UnknownWins
    };

    fn parse(
        _: &bedrock_addon_core::pack_identity::PackPath,
        _: &[u8],
        _: &bedrock_addon_core::load::LoadContext,
    ) -> Result<bedrock_addon_core::document::Document<Self>, bedrock_addon_core::document::Rejected>
    {
        unimplemented!("the test builds the value by hand")
    }

    fn write_json(
        &self,
        out: &mut Vec<u8>,
        style: WriteStyle,
    ) -> Result<(), bedrock_addon_core::json::EncodeError> {
        let mut root = IndexMap::new();
        root.insert("name".to_owned(), self.0.to_owned());
        write_json(&root, out, style)
    }
}

#[test]
fn a_content_states_whether_its_own_member_or_the_unknown_one_is_written() {
    use bedrock_addon_core::document::Content;
    let mut unknown = Unknown::new();
    unknown.put(&JsonPath::new(), "name", Value::from("unknown"));
    unknown.put(&JsonPath::new(), "other", Value::from(2i64));
    let written = |content: &dyn Fn(&mut Vec<u8>) -> Vec<Diagnostic>| {
        let mut out = Vec::new();
        let told = content(&mut out);
        assert!(told.is_empty(), "{told:?}");
        String::from_utf8(out).unwrap()
    };
    let known = written(&|out| {
        Named::<true>("known")
            .write_json_with(&unknown, out, COMPACT)
            .unwrap()
    });
    assert_eq!(known, r#"{"name":"known","other":2}"#);
    let unknown_wins = written(&|out| {
        Named::<false>("known")
            .write_json_with(&unknown, out, COMPACT)
            .unwrap()
    });
    assert_eq!(unknown_wins, r#"{"name":"unknown","other":2}"#);
}
