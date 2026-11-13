//! The span of a finding: the bytes of the value its path names, in the text of the pack and
//! file the finding names. The copies of a material file are merged before they are read, so
//! the span is of the copy that wrote the member, not of the highest copy.

use bedrock_addon::diagnostic::{Diagnostic, JsonPathSegment};
use bedrock_addon::document::Content;
use bedrock_addon::families::material::{MaterialFile, MaterialList};
use bedrock_addon::json::Value;
use bedrock_addon::load::LoadContext;
use bedrock_addon::storage::PackPath;

use crate::common::{flat, pack_name, wrapped};
use crate::stacks::{Files, stack};

const COMMON: &str = "materials/common.json";
const A: &str = "materials/a.material";
const B: &str = "materials/b.material";

fn list(paths: &[&str]) -> String {
    let entries: Vec<String> = paths
        .iter()
        .map(|path| format!(r#"{{"path": "{path}"}}"#))
        .collect();
    format!("[{}]", entries.join(", "))
}

/// The bytes a finding's span covers in `text`.
fn text_of<'a>(finding: &Diagnostic, text: &'a str) -> Option<&'a str> {
    let span = finding.location.span?;
    let start = span.offset as usize;
    Some(&text[start..start + span.len as usize])
}

/// The value `finding`'s path names in `text`, when the text has one there.
fn named(finding: &Diagnostic, text: &str) -> Option<Value> {
    let text = text.trim_start_matches('\u{feff}');
    let mut value = bedrock_addon::json::from_str_jsonc::<Value>(text).ok()?;
    for segment in finding.location.json_path.segments() {
        value = match segment {
            JsonPathSegment::Key(key) => value.as_object()?.get(key)?.clone(),
            JsonPathSegment::Index(index) => value.as_array()?.get(index)?.clone(),
        };
    }
    Some(value)
}

/// One finding of a stack as these tests compare it: code, JSON path, pack, file, and the
/// text its span covers in that pack's file.
type Spanned = (&'static str, String, String, String, Option<String>);

/// Every finding of the materials of `packs` with the text its span covers, each checked: a
/// finding whose path names a value of its file has the span of exactly that value, and one
/// whose path names none (or that has no path) has no span.
fn spanned(packs: &[Files<'_>]) -> Vec<Spanned> {
    let loaded = stack(packs);
    loaded
        .materials()
        .diagnostics()
        .map(|finding| {
            let pack = pack_name(finding).expect("a pack");
            let file = finding.location.path.as_ref().expect("a file").to_string();
            let number: usize = pack.trim_start_matches("pack ").parse().unwrap();
            let text = packs[number - 1]
                .iter()
                .find(|(path, _)| *path == file)
                .map_or_else(|| panic!("{pack} has no {file}"), |(_, text)| *text);
            let covered = text_of(finding, text);
            let path = finding.location.json_path.to_string();
            match named(finding, text).filter(|_| !path.is_empty()) {
                Some(value) => {
                    let covered =
                        covered.unwrap_or_else(|| panic!("{finding:#?} has no span in {text}"));
                    // A number past the doubles has no value to compare: its token is pinned
                    // by the tests themselves.
                    if let Ok(read) = bedrock_addon::json::from_str_jsonc::<Value>(covered)
                        && !covered.contains("e999")
                    {
                        assert_eq!(read, value, "{path} in {text}");
                    }
                }
                None => assert_eq!(covered, None, "{finding:#?}"),
            }
            (
                finding.code.id(),
                path,
                pack,
                file,
                covered.map(str::to_owned),
            )
        })
        .collect()
}

fn at(code: &'static str, path: &str, pack: &str, file: &str, covered: &str) -> Spanned {
    (
        code,
        path.to_owned(),
        pack.to_owned(),
        file.to_owned(),
        Some(covered.to_owned()),
    )
}

#[test]
fn a_finding_of_the_merge_has_the_span_of_its_value_in_the_copy_that_wrote_it() {
    // Three copies of one file: each finding is in the pack that wrote the member, and its
    // span is of that pack's text.
    let low = wrapped(r#""x": {"blendSrc": "Bad", "+vertexShader": "s", "states": ["Nope"]}"#);
    let middle = wrapped(r#""x": {"blendSrc": "Worse"}"#);
    let top = wrapped(r#""x": {"depthFunc": "Sometimes"}"#);
    let packs: &[Files<'_>] = &[
        &[(COMMON, &list(&[A])), (A, &low)],
        &[(A, &middle)],
        &[(A, &top)],
    ];
    assert_eq!(
        spanned(packs),
        [
            at(
                "core/invalid_value",
                "materials/x/depthFunc",
                "pack 3",
                A,
                r#""Sometimes""#
            ),
            at(
                "core/unrecognized_field",
                "materials/x/+vertexShader",
                "pack 1",
                A,
                r#""s""#
            ),
            at(
                "core/invalid_value",
                "materials/x/blendSrc",
                "pack 2",
                A,
                r#""Worse""#
            ),
            at(
                "material/state_unknown",
                "materials/x/states/0",
                "pack 1",
                A,
                r#""Nope""#
            ),
        ]
    );
}

#[test]
fn a_span_is_found_under_the_key_and_the_wrapper_of_its_own_copy() {
    // The low copy is flat and spells the key with a parent; the high one is wrapped and
    // spells it without.
    let low = flat(r#""p": {}, "x:p": {"blendSrc": "Bad", "frontFace": {"stencilFunc": "Low"}}"#);
    let high = wrapped(r#""x": {"depthFunc": "Bad", "frontFace": {"stencilPassOp": "High"}}"#);
    let packs: &[Files<'_>] = &[&[(COMMON, &list(&[A])), (A, &low)], &[(A, &high)]];
    let found = spanned(packs);
    let covered: Vec<(&str, &str, Option<&str>)> = found
        .iter()
        .filter(|f| f.0 == "core/invalid_value")
        .map(|f| (f.1.as_str(), f.2.as_str(), f.4.as_deref()))
        .collect();
    assert_eq!(
        covered,
        [
            ("materials/x/depthFunc", "pack 2", Some(r#""Bad""#)),
            (
                "materials/x/frontFace/stencilPassOp",
                "pack 2",
                Some(r#""High""#)
            ),
            ("x:p/frontFace/stencilFunc", "pack 1", Some(r#""Low""#)),
            ("x:p/blendSrc", "pack 1", Some(r#""Bad""#)),
        ],
        "{found:#?}"
    );
    // The key that takes over a lower key under another spelling (V20) has its own value.
    let shadowed: Vec<&Spanned> = found
        .iter()
        .filter(|f| f.0 == "core/shadowed_definition")
        .collect();
    assert_eq!(shadowed.len(), 1, "{found:#?}");
    assert_eq!(
        shadowed[0].4.as_deref(),
        Some(r#"{"depthFunc": "Bad", "frontFace": {"stencilPassOp": "High"}}"#)
    );
}

#[test]
fn the_findings_of_one_copy_and_of_a_list_have_their_spans() {
    // A number past the doubles (the copy's own finding), entries the readers drop, and a
    // listing that has no effect (a finding of the group, placed at the list's entry).
    let file = flat(r#""x": {"depthBias": 1e999, "defines": ["A", 3, "B", {}]}"#);
    let listing = format!(r#"[{{"path": "{A}"}}, {{"path": "{A}"}}, 7]"#);
    let packs: &[Files<'_>] = &[&[(COMMON, &listing), (A, &file)]];
    let found = spanned(packs);
    let covered: Vec<(&str, &str, &str, Option<&str>)> = found
        .iter()
        .map(|f| (f.0, f.1.as_str(), f.3.as_str(), f.4.as_deref()))
        .collect();
    assert!(
        covered.contains(&("core/invalid_value", "x/depthBias", A, Some("1e999"))),
        "{covered:#?}"
    );
    assert!(
        covered.contains(&("core/invalid_type", "x/defines/1", A, Some("3"))),
        "{covered:#?}"
    );
    assert!(
        covered.contains(&("core/invalid_type", "x/defines/3", A, Some("{}"))),
        "{covered:#?}"
    );
    let again = format!(r#"{{"path": "{A}"}}"#);
    let of_list: Vec<_> = covered.iter().filter(|f| f.2 == COMMON).collect();
    assert_eq!(
        of_list,
        [
            &("core/invalid_type", "2", COMMON, Some("7")),
            &("core/no_effect", "1", COMMON, Some(again.as_str())),
        ]
    );
    // Every finding with a path has a span here.
    assert!(
        found.iter().all(|f| f.1.is_empty() || f.4.is_some()),
        "{found:#?}"
    );
}

#[test]
fn a_finding_of_the_creation_has_the_span_of_the_definition_in_its_copy() {
    // `c` names a parent no file defines: the creation refuses it, in the pack that wrote it.
    let low = flat(r#""a": {}"#);
    let high = flat(r#""c:nowhere": {"depthFunc": "Always"}"#);
    let packs: &[Files<'_>] = &[&[(COMMON, &list(&[A, B])), (A, &low)], &[(B, &high)]];
    let found = spanned(packs);
    let of_b: Vec<_> = found.iter().filter(|f| f.3 == B).collect();
    assert_eq!(of_b.len(), 1, "{found:#?}");
    assert_eq!(of_b[0].1, "c:nowhere");
    assert_eq!(of_b[0].2, "pack 2");
    assert_eq!(of_b[0].4.as_deref(), Some(r#"{"depthFunc": "Always"}"#));
}

#[test]
fn of_a_member_written_twice_the_span_is_of_the_value_that_stands() {
    let file = flat(r#""x": {"blendSrc": "First"}, "y": {}, "x": {"blendSrc": "Second"}"#);
    let packs: &[Files<'_>] = &[&[(COMMON, &list(&[A])), (A, &file)]];
    let found = spanned(packs);
    let bad: Vec<_> = found.iter().filter(|f| f.1 == "x/blendSrc").collect();
    assert_eq!(bad.len(), 1, "{found:#?}");
    assert_eq!(bad[0].4.as_deref(), Some(r#""Second""#));
}

#[test]
fn a_file_read_on_its_own_has_its_spans_counted_from_the_start_of_the_file() {
    // A byte-order mark is three bytes of the file: the span counts them.
    let inner = wrapped(r#""x": {"blendSrc": "Bad", "depthBias": -1e999}"#);
    let text = format!("\u{feff}{inner}");
    let path = PackPath::new(A).unwrap();
    // The file is refused for its keyword: the findings of a refusal have their spans too.
    let refused = MaterialFile::parse(&path, text.as_bytes(), &LoadContext::detached())
        .expect_err("a keyword the client cannot load");
    let covered: Vec<(String, Option<&str>)> = refused
        .diagnostics
        .iter()
        .map(|finding| {
            (
                finding.location.json_path.to_string(),
                text_of(finding, &text),
            )
        })
        .collect();
    assert_eq!(
        covered,
        [
            ("materials/x/depthBias".to_owned(), Some("-1e999")),
            ("materials/x/blendSrc".to_owned(), Some(r#""Bad""#)),
        ]
    );
    // A list read on its own.
    let text = format!(r#"[{{"path": "{A}", "+defines": [1]}}, []]"#);
    let path = PackPath::new(COMMON).unwrap();
    let document =
        MaterialList::parse(&path, text.as_bytes(), &LoadContext::detached()).expect("it loads");
    let found: Vec<_> = document.diagnostics().iter().collect();
    assert!(!found.is_empty());
    for finding in found {
        let value = named(finding, &text).expect("the path names a value");
        let covered = text_of(finding, &text).expect("a span");
        assert_eq!(
            bedrock_addon::json::from_str_jsonc::<Value>(covered).unwrap(),
            value,
            "{finding:#?}"
        );
    }
}

#[test]
fn a_finding_of_a_group_names_a_list_entry_by_its_index_in_the_file() {
    // The list reader drops entry 1 (no object) and entry 2 (no path), so the entries the
    // list holds are not at their places in the file: what a group finds about one of them
    // is at the index the entry has in the file, and its span is that entry's.
    let listing =
        format!(r#"[{{"path": "{A}"}}, 7, {{}}, {{"path": "notes.txt"}}, {{"path": "{A}"}}]"#);
    let file = flat(r#""x": {}"#);
    let packs: &[Files<'_>] = &[&[(COMMON, &listing), (A, &file), ("notes.txt", "{}")]];
    let found = spanned(packs);
    let of_groups: Vec<(&str, &str, Option<&str>)> = found
        .iter()
        .filter(|f| matches!(f.0, "core/no_effect" | "core/ignored_file"))
        .map(|f| (f.0, f.1.as_str(), f.4.as_deref()))
        .collect();
    let again = format!(r#"{{"path": "{A}"}}"#);
    assert_eq!(
        of_groups,
        [
            ("core/ignored_file", "3/path", Some(r#""notes.txt""#)),
            ("core/no_effect", "4", Some(again.as_str())),
        ],
        "{found:#?}"
    );
}
