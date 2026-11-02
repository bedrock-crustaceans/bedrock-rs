//! The list files read on their own.

use bedrock_addon::diagnostic::Severity::{Error, Info};
use bedrock_addon::document::Placed;
use bedrock_addon::families::material::{ListEntry, MaterialList, MaterialListKind};

use crate::common::{finding, fixture_text, parse_list, rendered};

/// The list in `text`, read as `materials/common.json`, which must load.
#[track_caller]
fn list(text: &str) -> MaterialList {
    let document = parse_list("materials/common.json", text).expect("a list");
    (*document).clone()
}

/// The findings of reading `text` as a list.
fn found(text: &str) -> Vec<crate::common::Rendered> {
    match parse_list("materials/common.json", text) {
        Ok(document) => rendered(document.diagnostics().iter()),
        Err(rejected) => rendered(rejected.diagnostics.iter()),
    }
}

fn owned(names: &[&str]) -> Option<Vec<String>> {
    Some(names.iter().map(|name| (*name).to_owned()).collect())
}

// The vanilla file is gdk-retail 1.26.45 `resource_packs/vanilla/materials/fancy.json`, copied
// unchanged.
#[test]
fn the_vanilla_fancy_list_is_read() {
    let text = fixture_text("vanilla/fancy.json");
    let document = parse_list("materials/fancy.json", &text).expect("a list");
    assert_eq!(rendered(document.diagnostics().iter()), []);
    assert_eq!(document.kind, MaterialListKind::Fancy);
    let paths: Vec<&str> = document.entries.iter().map(|e| e.path.as_str()).collect();
    assert_eq!(
        paths,
        [
            "materials/fancy.material",
            "materials/entity.material",
            "materials/terrain.material",
            "materials/hologram.material",
            "materials/portal.material",
            "materials/cameraFacingSprite.material",
            "materials/wireframe.material",
        ]
    );
    assert_eq!(document.entries[1].add_defines, owned(&["FANCY"]));
    assert_eq!(document.entries[1].defines, None);
    assert_eq!(document.entries[1].base_defines(), ["FANCY"]);
    assert_eq!(document.entries[3].add_defines, None);
    assert!(document.entries[3].base_defines().is_empty());
}

#[test]
fn the_path_says_which_list_a_file_is_and_the_kind_says_where_it_goes() {
    for kind in MaterialListKind::ALL {
        let document = parse_list(kind.path(), "[]").expect("a list");
        assert_eq!(document.kind, kind);
        assert_eq!(document.default_path().as_str(), kind.path());
        assert_eq!(MaterialListKind::from_path(kind.path()), Some(kind));
    }
    assert_eq!(
        MaterialListKind::ALL.map(MaterialListKind::path),
        [
            "materials/common.json",
            "materials/fancy.json",
            "materials/sad.json",
            "materials/gameface.json"
        ]
    );
    // Any other path is read as the common list.
    assert_eq!(MaterialListKind::from_path("materials/other.json"), None);
    assert_eq!(MaterialListKind::from_path("common.json"), None);
    let other = parse_list("materials/other.json", "[]").expect("a list");
    assert_eq!(other.kind, MaterialListKind::Common);
}

#[test]
fn m13_a_tag_is_reported_with_the_clients_text_and_the_entry_loads() {
    // the game logs it at level 3 as a deprecated schema keyword; the
    // entry still loads: a kept error.
    let text = r#"[{"path": "materials/a.material", "tag": "x"}]"#;
    assert_eq!(
        found(text),
        [finding(
            "DeprecatedMember",
            Error,
            "0/tag",
            "key \"tag\" is no longer supported, please remove"
        )]
    );
    let read = list(text);
    assert_eq!(read.entries.len(), 1);
    assert_eq!(read.entries[0].path, "materials/a.material");
    assert!(read.entries[0].extra.get("tag").is_some());
    // Whatever its value, and in each entry that has one.
    let more = r#"[{"path": "a.material"}, {"path": "b.material", "tag": null}, {"tag": 1, "path": "c.material"}]"#;
    let at: Vec<String> = found(more).into_iter().map(|f| f.2).collect();
    assert_eq!(at, ["1/tag", "2/tag"]);
}

#[test]
fn the_three_define_keys_are_read_and_combine_in_the_clients_order() {
    let text = r#"[{"path": "materials/a.material",
                    "-defines": ["B", "X"], "+defines": ["C", "A"], "defines": ["A", "B"]}]"#;
    assert_eq!(found(text), []);
    let entry = &list(text).entries[0];
    assert_eq!(entry.defines, owned(&["A", "B"]));
    assert_eq!(entry.add_defines, owned(&["C", "A"]));
    assert_eq!(entry.sub_defines, owned(&["B", "X"]));
    // `defines` first, then `+defines`, then `-defines` removes; each name once.
    assert_eq!(entry.base_defines(), ["A", "C"]);
}

#[test]
fn base_defines_of_each_key_alone() {
    let entry = |defines: &[&str], add: &[&str], sub: &[&str]| ListEntry {
        path: "x".to_owned(),
        defines: (!defines.is_empty()).then(|| owned(defines).unwrap()),
        add_defines: (!add.is_empty()).then(|| owned(add).unwrap()),
        sub_defines: (!sub.is_empty()).then(|| owned(sub).unwrap()),
        ..ListEntry::default()
    };
    assert_eq!(entry(&["A", "B", "A"], &[], &[]).base_defines(), ["A", "B"]);
    assert_eq!(entry(&[], &["B", "A"], &[]).base_defines(), ["B", "A"]);
    assert!(entry(&[], &[], &["A"]).base_defines().is_empty());
    assert!(entry(&["A"], &["A"], &["A"]).base_defines().is_empty());
    assert_eq!(entry(&["A"], &["B"], &["C"]).base_defines(), ["A", "B"]);
}

#[test]
fn an_entry_the_client_cannot_use_is_dropped_with_a_warning() {
    let text = r#"[5, {"path": "materials/a.material"}, {}, {"path": 7}, {"path": null},
                   "materials/b.material", {"path": ""}]"#;
    let wrong = |at: &str| finding("InvalidType", Info, at, "wrong JSON type");
    let missing = |at: &str| finding("MissingRequiredField", Info, at, "missing required field");
    assert_eq!(
        found(text),
        [
            wrong("0"),
            missing("2/path"),
            wrong("3/path"),
            missing("4/path"),
            wrong("5"),
        ]
    );
    let paths: Vec<String> = list(text).entries.into_iter().map(|e| e.path).collect();
    // An empty path is a path: the client asks the packs for it and finds nothing.
    assert_eq!(paths, ["materials/a.material", ""]);
}

#[test]
fn defines_of_the_wrong_type_are_kept_and_entries_that_are_no_string_dropped() {
    let text =
        r#"[{"path": "a", "defines": "FANCY", "+defines": ["A", 5, null, "B"], "-defines": {}}]"#;
    let wrong = |at: &str| finding("InvalidType", Info, at, "wrong JSON type");
    assert_eq!(
        found(text),
        [
            wrong("0/defines"),
            wrong("0/+defines/1"),
            wrong("0/+defines/2"),
            wrong("0/-defines"),
        ]
    );
    let entry = &list(text).entries[0];
    assert_eq!(entry.defines, None);
    assert_eq!(entry.add_defines, owned(&["A", "B"]));
    assert_eq!(entry.sub_defines, None);
    assert!(entry.extra.get("defines").is_some());
    assert!(entry.extra.get("-defines").is_some());
    assert_eq!(entry.base_defines(), ["A", "B"]);
}

#[test]
fn other_members_of_an_entry_are_kept_and_say_nothing() {
    let text = r#"[{"path": "a", "comment": "x", "defines": null, "extra": 1}]"#;
    assert_eq!(found(text), []);
    let entry = &list(text).entries[0];
    let extra: Vec<&str> = entry.extra.iter().map(|(key, _)| key).collect();
    assert_eq!(extra, ["comment", "defines", "extra"]);
}

#[test]
fn a_root_that_is_no_array_is_rejected_and_an_empty_array_is_a_list() {
    for text in ["{}", "5", "\"materials/a.material\"", "null"] {
        let rejected = parse_list("materials/common.json", text).expect_err("no list");
        assert_eq!(
            rendered(rejected.diagnostics.iter()),
            [finding("InvalidType", Error, "", "wrong JSON type")],
            "{text}"
        );
    }
    // vanilla_base ships `gameface.json` as `[\n]`.
    assert!(list("[\n]").entries.is_empty());
    assert_eq!(found("[\n]"), []);
}

#[test]
fn a_list_that_is_no_json_is_rejected_with_the_clients_text() {
    let rejected = parse_list("materials/sad.json", "[{").expect_err("no JSON");
    let findings = rendered(rejected.diagnostics.iter());
    assert_eq!(findings.len(), 1);
    assert_eq!((findings[0].0, findings[0].1), ("MalformedInput", Error));
    assert!(
        findings[0]
            .3
            .starts_with("Error parsing materials/sad.json :\n "),
        "{}",
        findings[0].3
    );
}
