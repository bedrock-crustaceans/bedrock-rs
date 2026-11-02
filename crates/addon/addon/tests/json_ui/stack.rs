//! `rp.ui()` on a stack: the list, merge, modification, reference and global-variable rows
//! of the fixtures (rules L1–L22).

use bedrock_addon::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon::families::json_ui::{COLOR_VARIABLES, ROOT_NAMESPACE, UiPackErrorKind};

use super::common::*;

// ---- defs lists and file discovery ----------------------------------------------------------

#[test]
fn d01_union_sorted() {
    let stack = check("d01_union_sorted");
    let ui = stack.ui();
    // the united list is sorted, each path once, whatever order the packs list them in
    let paths: Vec<&str> = ui.files.keys().map(String::as_str).collect();
    assert_eq!(paths, ["ui/a.json", "ui/b.json", "ui/c.json"]);
    // a file is the pack's that has it; the layers are top first
    assert_eq!(
        ui.origin(&"ui/c.json".to_owned()).unwrap().pack.index(),
        Some(0)
    );
    assert_eq!(
        ui.origin(&"ui/a.json".to_owned()).unwrap().pack.index(),
        Some(1)
    );
    assert_eq!(ui.origin(&"ui/zz.json".to_owned()), None);
    let layers: Vec<(Option<usize>, &str)> = ui
        .layers()
        .iter()
        .map(|layer| (layer.origin.pack.index(), layer.origin.path.as_str()))
        .collect();
    assert_eq!(layers.len(), 5);
    assert!(layers[..2].iter().all(|(pack, _)| *pack == Some(0)));
    assert!(layers[2..].iter().all(|(pack, _)| *pack == Some(1)));
    assert!(layers.contains(&(Some(0), "ui/_ui_defs.json")));
    assert!(ui.layers()[0].json.is_object());
}

#[test]
fn d02_defs_parse_error() {
    let stack = check("d02_defs_parse_error");
    let text = &texts(&stack, "MalformedInput")[0];
    assert!(
        text.starts_with("JSON Parse error in file: ui/_ui_defs.json\n\n") && text.ends_with('\n'),
        "{text:?}"
    );
    // the broken list's own file is not loaded: only the other pack's list counts
    assert!(!stack.ui().files.contains_key("ui/b.json"));
}

#[test]
fn d03_missing_file() {
    let stack = check("d03_missing_file");
    let missing = stack
        .ui()
        .diagnostics()
        .find(|d| d.code.name() == "UnresolvedReference")
        .unwrap();
    // reported against the pack that lists the file, at its list
    assert_eq!(missing.location.pack.as_ref().unwrap().index(), Some(0));
    assert_eq!(
        missing.location.path.as_ref().unwrap().as_str(),
        "ui/_ui_defs.json"
    );
}

/// A finding about a listed file is at the element of the list that names it: of several
/// lists the highest pack's, of several elements the last.
#[test]
fn a_finding_about_a_listed_file_is_at_its_element_of_the_list() {
    let stack = stack_of(&[
        &[
            (
                "ui/_ui_defs.json",
                r#"{ "ui_defs": ["ui/zz.json", "ui/a.json", "hud/b.json"] }"#,
            ),
            ("ui/a.json", "{}"),
        ],
        &[(
            "ui/_ui_defs.json",
            r#"{ "ui_defs": ["ui/a.json", "ui/yy.json", "ui/c.txt", "ui/yy.json", 5] }"#,
        )],
    ]);
    let mut at: Vec<(&str, Option<usize>, String, String)> = stack
        .ui()
        .diagnostics()
        .filter(|d| matches!(d.code.name(), "UnresolvedReference" | "FileNotLoaded"))
        .map(|d| {
            assert_eq!(d.severity, Severity::Info);
            assert_eq!(
                d.location.path.as_ref().unwrap().as_str(),
                "ui/_ui_defs.json"
            );
            (
                d.code.name(),
                d.location.pack.as_ref().unwrap().index(),
                d.location.json_path.to_string(),
                d.code.to_string(),
            )
        })
        .collect();
    at.sort();
    let not_loaded = |path: &str| {
        format!(
            "The following file could not be loaded: {path}. Make sure the file exists or remove the reference in '_ui_defs.json'"
        )
    };
    let unread = "file indexed but not loaded".to_owned();
    let row = |code: &'static str, pack: usize, at: &str, text: String| {
        (code, Some(pack), at.to_owned(), text)
    };
    assert_eq!(
        at,
        [
            row("FileNotLoaded", 0, "ui_defs/2", unread.clone()),
            row("FileNotLoaded", 1, "ui_defs/2", unread),
            row(
                "UnresolvedReference",
                0,
                "ui_defs/3",
                not_loaded("ui/yy.json")
            ),
            // the non-string element reads as the empty path (d05)
            row("UnresolvedReference", 0, "ui_defs/4", not_loaded("")),
            row(
                "UnresolvedReference",
                1,
                "ui_defs/0",
                not_loaded("ui/zz.json")
            ),
        ]
    );
    // the index is an index segment, not a member named "3"
    let finding = stack
        .ui()
        .diagnostics()
        .find(|d| d.location.json_path.to_string() == "ui_defs/3")
        .unwrap();
    let mut expected = bedrock_addon::diagnostic::JsonPath::new();
    expected.push("ui_defs");
    expected.push_index(3);
    assert_eq!(finding.location.json_path, expected);
}

/// A list may name the list itself and the global variables: both files exist, so neither is
/// "could not be loaded". The game loads every listed path as a UI file; these two are that
/// besides what they always are, and each has one layer.
#[test]
fn a_list_that_names_itself_or_the_global_variables_finds_both() {
    let files: &[(&str, &str)] = &[
        (
            "ui/_ui_defs.json",
            r#"{ "ui_defs": ["ui/_ui_defs.json", "ui/_global_variables.json", "ui/a.json"] }"#,
        ),
        ("ui/_global_variables.json", r#"{ "$x": 1 }"#),
        ("ui/a.json", r#"{ "j": { "type": "panel" } }"#),
    ];
    for packs in [&[files][..], &[files, files][..]] {
        let stack = stack_of(packs);
        assert_eq!(texts(&stack, "UnresolvedReference"), Vec::<String>::new());
        assert_eq!(texts(&stack, "FileNotLoaded"), Vec::<String>::new());
        let ui = stack.ui();
        // what they always are
        assert!(ui.get(ROOT_NAMESPACE, "j").is_some());
        assert!(ui.global_variables.get("$x").is_some());
        // and listed files like any other
        let paths: Vec<&str> = ui.files.keys().map(String::as_str).collect();
        assert_eq!(
            paths,
            ["ui/_global_variables.json", "ui/_ui_defs.json", "ui/a.json"]
        );
        // one layer per file of each pack
        assert_eq!(ui.layers().len(), 3 * packs.len());
        assert_eq!(
            ui.origin(&"ui/_global_variables.json".to_owned())
                .unwrap()
                .pack
                .index(),
            Some(0)
        );
    }
    // A list that names the global variables no pack has: that file is missing.
    let stack = stack_of(&[&[(
        "ui/_ui_defs.json",
        r#"{ "ui_defs": ["ui/_global_variables.json"] }"#,
    )]]);
    assert_eq!(texts(&stack, "UnresolvedReference").len(), 1);
}

#[test]
fn d04_unlisted_file_ignored() {
    let stack = check("d04_unlisted_file_ignored");
    assert!(stack.ui().get("x", "one").is_none());
    assert!(
        stack
            .ui()
            .layers()
            .iter()
            .all(|layer| layer.origin.path.as_str() != "ui/x.json")
    );
}

#[test]
fn d05_non_string_entry() {
    check("d05_non_string_entry");
}

#[test]
fn d06_comments() {
    check("d06_comments");
}

#[test]
fn an_empty_combined_list_is_an_info_deviation() {
    // L2: vanilla's debug assertion "Didn't find any files!"
    for packs in [
        &[&[
            ("ui/_ui_defs.json", r#"{ "ui_defs": [] }"#),
            ("ui/a.json", "{}"),
        ][..]][..],
        &[&[("ui/a.json", "{}")][..]][..],
        &[&[("ui/_ui_defs.json", r#"{ "other": ["ui/a.json"] }"#)][..]][..],
    ] {
        let stack = stack_of(packs);
        assert_eq!(
            findings(&stack),
            vec![(
                "NoUiDefinitions",
                Severity::Info,
                "no UI definition files".to_owned()
            )]
        );
        assert!(stack.ui().files.is_empty());
    }
    // a stack without any file below `ui/` has no UI to report on
    let stack = stack_of(&[&[("texts/en_US.lang", "a=b\n")]]);
    assert!(findings(&stack).is_empty());
    assert!(stack.ui().layers().is_empty());
    // one listed path is enough for no info, even when the file is missing
    let stack = stack_of(&[&[("ui/_ui_defs.json", r#"{ "ui_defs": ["ui/a.json"] }"#)]]);
    assert!(texts(&stack, "NoUiDefinitions").is_empty());
}

#[test]
fn a_listed_path_outside_ui_is_not_read_deviation() {
    // vanilla reads any pack-relative path; only `ui/` is walked here
    let stack = stack_of(&[&[
        (
            "ui/_ui_defs.json",
            r#"{ "ui_defs": ["hud/a.json", "ui/a.json"] }"#,
        ),
        ("hud/a.json", r#"{ "k": { "type": "panel" } }"#),
        ("ui/a.json", r#"{ "j": { "type": "panel" } }"#),
    ]]);
    assert_eq!(
        findings(&stack),
        vec![(
            "FileNotLoaded",
            Severity::Info,
            "file indexed but not loaded".to_owned()
        )]
    );
    assert!(stack.ui().get(ROOT_NAMESPACE, "k").is_none());
    assert!(stack.ui().get(ROOT_NAMESPACE, "j").is_some());
}

#[test]
fn a_listed_file_that_is_no_json_file_is_not_read_deviation() {
    // vanilla reads whatever path a list names; only the `.json` files below `ui/` are asked
    // for here, so that no picture or archive an author left there is read
    let stack = stack_of(&[&[
        (
            "ui/_ui_defs.json",
            r#"{ "ui_defs": ["ui/a.txt", "ui/b.JSON", "ui/gone.txt"] }"#,
        ),
        ("ui/a.txt", r#"{ "k": { "type": "panel" } }"#),
        ("ui/b.JSON", r#"{ "j": { "type": "panel" } }"#),
    ]]);
    let found: Vec<&str> = findings(&stack).iter().map(|(name, _, _)| *name).collect();
    assert_eq!(found, ["FileNotLoaded", "FileNotLoaded"]);
    assert!(stack.ui().get(ROOT_NAMESPACE, "k").is_none());
    assert!(stack.ui().get(ROOT_NAMESPACE, "j").is_some());
}

// ---- cross-pack merge -----------------------------------------------------------------------

#[test]
fn m01_short_name_override() {
    let stack = check("m01_short_name_override");
    let ui = stack.ui();
    // the key keeps its `@a.base`
    assert!(
        ui.files["ui/a.json"]
            .defs
            .keys()
            .any(|key| key.to_string() == "root@a.base")
    );
    // the merged file is the top pack's, with the base pack as a contributor
    let origin = ui.origin(&"ui/a.json".to_owned()).unwrap();
    assert_eq!(origin.pack.index(), Some(0));
    assert_eq!(origin.contributors.len(), 1);
    assert_eq!(origin.contributors[0].index(), Some(1));
}

#[test]
fn m02_deep_object_merge() {
    check("m02_deep_object_merge");
}

#[test]
fn m03_array_replaced() {
    check("m03_array_replaced");
}

#[test]
fn m04_path_key() {
    check("m04_path_key");
}

#[test]
fn m05_path_key_missing() {
    let stack = check("m05_path_key_missing");
    let ui = stack.ui();
    assert_eq!(ui.reports.len(), 1);
    assert_eq!(ui.reports[0].pack.index(), Some(0));
    let error = &ui.reports[0].errors[0];
    assert_eq!(error.kind, UiPackErrorKind::MissingControl);
    assert_eq!(
        error.to_string(),
        "ui/a.json: Can't find the control: root/nope"
    );
    // the rolled-back pack is no part of the merged file
    let origin = ui.origin(&"ui/a.json".to_owned()).unwrap();
    assert_eq!(origin.pack.index(), Some(1));
    assert!(origin.contributors.is_empty());
}

#[test]
fn a_path_key_fails_at_each_step() {
    // L7: the first segment is not a base short name; a node without `controls`; a missing
    // child deeper down
    for key in ["nope/x", "leaf/x", "root/x/deeper"] {
        let overlay = format!(r#"{{ "{key}": {{ "layer": 1 }} }}"#);
        let stack = stack_of(&[
            &[
                ("ui/_ui_defs.json", r#"{ "ui_defs": ["ui/a.json"] }"#),
                (
                    "ui/a.json",
                    r#"{ "namespace": "a", "root": { "type": "panel", "controls": [ { "x@a.leaf": {} } ] },
                        "leaf": { "type": "label", "text": "hi" } }"#,
                ),
            ],
            &[("ui/a.json", &overlay)],
        ]);
        assert_eq!(
            texts(&stack, "UnresolvedReference"),
            vec![format!("ui/a.json: Can't find the control: {key}")]
        );
    }
}

#[test]
fn m06_new_control() {
    check("m06_new_control");
}

#[test]
fn m07_rebased_override_deviation() {
    // vanilla: a second member, silently; ours adds `RebasedOverride` and `DuplicateIdentifier`
    check("m07_rebased_override");
    // the same key as the base's is no rebase
    let stack = stack_of(&[
        &[
            ("ui/_ui_defs.json", r#"{ "ui_defs": ["ui/a.json"] }"#),
            (
                "ui/a.json",
                r#"{ "base": {}, "root@base": { "type": "panel" } }"#,
            ),
        ],
        &[(
            "ui/a.json",
            r#"{ "root@base": { "layer": 2 }, "new@base": { "type": "panel" } }"#,
        )],
    ]);
    assert!(findings(&stack).is_empty(), "{:?}", findings(&stack));
}

#[test]
fn m08_rollback_whole_copy() {
    check("m08_rollback_whole_copy");
}

#[test]
fn m09_three_packs_order() {
    check("m09_three_packs_order");
    check("m09b_three_packs_error");
}

#[test]
fn m10_overlay_parse_error() {
    let stack = check("m10_overlay_parse_error");
    let text = &texts(&stack, "MalformedInput")[0];
    assert!(
        text.starts_with("ui/a.json: Unable to parse UI JSON file with stack: "),
        "{text:?}"
    );
}

#[test]
fn m11_base_parse_error() {
    let stack = check("m11_base_parse_error");
    let text = &texts(&stack, "MalformedInput")[0];
    assert!(
        text.starts_with("JSON Parse error in file: ui/a.json\n\n"),
        "{text:?}"
    );
    assert_eq!(stack.ui().origin(&"ui/a.json".to_owned()), None);
}

#[test]
fn a_base_parse_error_keeps_no_partial_result_deviation() {
    // vanilla: jsoncpp may leave a partial result behind; ours loads nothing of the file
    let stack = stack_of(&[&[
        ("ui/_ui_defs.json", r#"{ "ui_defs": ["ui/a.json"] }"#),
        ("ui/a.json", r#"{ "first": { "type": "panel" }, "second": "#),
    ]]);
    assert!(stack.ui().files.is_empty());
    assert_eq!(stack.ui().controls().count(), 0);
}

#[test]
fn m12_texture_pack_index() {
    let stack = check("m12_texture_pack_index");
    let img = control(&stack, "a", "img");
    assert_eq!(
        img.get("texture").unwrap().as_string().unwrap().as_str(),
        "t1"
    );
    assert_eq!(img.texture(), Some("t2"));
}

#[test]
fn a_texture_with_uv_size_alone_is_paired_too() {
    let stack = stack_of(&[
        &[
            ("ui/_ui_defs.json", r#"{ "ui_defs": ["ui/a.json"] }"#),
            (
                "ui/a.json",
                r#"{ "img": { "type": "image", "texture": "t1", "uv_size": [8, 8] } }"#,
            ),
        ],
        &[(
            "ui/a.json",
            r#"{ "img": { "texture": "t2", "uv_size": [4, 4] } }"#,
        )],
        &[(
            "ui/a.json",
            r#"{ "img": { "texture": "t3", "uv": [1, 1] } }"#,
        )],
    ]);
    let img = control(&stack, ROOT_NAMESPACE, "img");
    assert!(img.get("texture:1").is_some() && img.get("texture:2").is_some());
    // the highest pack's texture is the one read
    assert_eq!(img.texture(), Some("t3"));
}

#[test]
fn m13_texture_without_uv() {
    let stack = check("m13_texture_without_uv");
    assert_eq!(control(&stack, "a", "img").texture(), Some("t2"));
}

#[test]
fn m14_null_member_skipped() {
    check("m14_null_member_skipped");
}

#[test]
fn m15_invalid_child_names_base() {
    let stack = check("m15_invalid_child_names_base");
    // L5: the game logs it at the content log's error level and merges the file all the same
    // (`UIResourcePackMergeStrategy::mergeFiles`, `_contentLog(true, 3, 33)`):
    // a kept error, and the file's definitions load.
    let finding = stack
        .ui()
        .diagnostics()
        .find(|d| d.code.name() == "InvalidValue")
        .expect("the L5 finding");
    assert_eq!(finding.severity, Severity::Error);
    assert!(finding.kept, "{finding}");
    assert!(!finding.rejects());
    assert!(stack.ui().get("a", "root").is_some());
}

#[test]
fn an_empty_child_name_is_invalid_too() {
    let stack = one_file(
        r#"{ "namespace": "a", "leaf": { "type": "label", "text": "t" },
            "root": { "type": "panel", "controls": [ { "@a.leaf": {} }, { "x@a.leaf": {} } ] } }"#,
    );
    let texts = texts(&stack, "InvalidValue");
    assert_eq!(texts.len(), 1);
    assert!(
        texts[0].contains("Controls:\n\nFile name:ui/a.json"),
        "{:?}",
        texts[0]
    );
    // unique, non-empty names are fine
    let stack = one_file(
        r#"{ "namespace": "a", "leaf": { "type": "label", "text": "t" },
            "root": { "type": "panel", "controls": [ { "y@a.leaf": {} }, { "x@a.leaf": {} } ] } }"#,
    );
    assert!(findings(&stack).is_empty());
}

#[test]
fn m16_invalid_child_names_overlay() {
    let stack = check("m16_invalid_child_names_overlay");
    let error = &stack.ui().reports[0].errors[0];
    assert_eq!(error.kind, UiPackErrorKind::InvalidChildNames);
    assert_eq!(error.severity, Severity::Warning);
}

#[test]
fn names_already_invalid_in_the_base_are_not_reported_again() {
    let stack = stack_of(&[
        &[
            ("ui/_ui_defs.json", r#"{ "ui_defs": ["ui/a.json"] }"#),
            (
                "ui/a.json",
                r#"{ "namespace": "a", "leaf": { "type": "label", "text": "t" },
                    "root": { "type": "panel", "controls": [ { "x@a.leaf": {} }, { "x@a.leaf": {} } ] } }"#,
            ),
        ],
        &[("ui/a.json", r#"{ "root": { "layer": 1 } }"#)],
    ]);
    assert!(stack.ui().reports.is_empty());
    assert_eq!(texts(&stack, "InvalidValue").len(), 1);
}

#[test]
fn m17_non_object_value_ignored_deviation() {
    // vanilla ignores the two members silently; the copy is committed
    let stack = check("m17_non_object_value_ignored");
    assert!(stack.ui().get("a", "newkey").is_none());
    assert_eq!(
        stack
            .ui()
            .origin(&"ui/a.json".to_owned())
            .unwrap()
            .pack
            .index(),
        Some(0)
    );
}

#[test]
fn m18_short_name_map_base_only() {
    check("m18_short_name_map_base_only");
}

#[test]
fn an_empty_overlay_copy_is_skipped() {
    let stack = stack_of(&[
        &[
            ("ui/_ui_defs.json", r#"{ "ui_defs": ["ui/a.json"] }"#),
            ("ui/a.json", r#"{ "k": { "type": "panel" } }"#),
        ],
        &[("ui/a.json", "")],
    ]);
    assert!(findings(&stack).is_empty(), "{:?}", findings(&stack));
    assert!(stack.ui().reports.is_empty());
    assert_eq!(
        stack
            .ui()
            .origin(&"ui/a.json".to_owned())
            .unwrap()
            .pack
            .index(),
        Some(1)
    );
}

// ---- modifications --------------------------------------------------------------------------

#[test]
fn o01_insert_back_default_array() {
    check("o01_insert_back_default_array");
}

#[test]
fn o02_insert_after_where_bindings() {
    check("o02_insert_after_where_bindings");
}

#[test]
fn o03_move_before_target() {
    check("o03_move_before_target");
}

#[test]
fn o04_swap() {
    check("o04_swap");
}

#[test]
fn o05_remove() {
    check("o05_remove");
}

#[test]
fn o06_replace() {
    check("o06_replace");
}

#[test]
fn o07_value_not_array() {
    check("o07_value_not_array");
}

#[test]
fn o08_missing_array_name() {
    check("o08_missing_array_name");
}

#[test]
fn o09_missing_operation() {
    check("o09_missing_operation");
}

#[test]
fn o10_invalid_operation() {
    check("o10_invalid_operation");
}

#[test]
fn o11_missing_condition() {
    check("o11_missing_condition");
}

#[test]
fn o12_missing_target() {
    check("o12_missing_target");
}

#[test]
fn o13_missing_value() {
    check("o13_missing_value");
}

#[test]
fn o14_nested_modifications_ignored_deviation() {
    // vanilla drops the nested list silently
    check("o14_nested_modifications_ignored");
}

#[test]
fn o15_several_ops_one_array() {
    check("o15_several_ops_one_array");
}

#[test]
fn o16_where_any_member() {
    check("o16_where_any_member");
}

#[test]
fn o17_where_array_never_matches_deviation() {
    // vanilla: a silent no-op (its inner loop never reports a match)
    check("o17_where_array_never_matches");
}

#[test]
fn o18_where_miss_silent_deviation() {
    // vanilla: a silent no-op without a `uiPackError`
    check("o18_where_miss_silent");
}

#[test]
fn o19_where_scalar_first_element() {
    check("o19_where_scalar_first_element");
}

#[test]
fn o20_order_deepest_first() {
    check("o20_order_deepest_first");
}

#[test]
fn o21_move_without_target() {
    check("o21_move_without_target");
}

#[test]
fn o22_insert_back_without_value() {
    check("o22_insert_back_without_value");
}

#[test]
fn o23_empty_control_name() {
    check("o23_empty_control_name");
}

#[test]
fn o24_control_name_not_string() {
    check("o24_control_name_not_string");
}

/// The children of `root` after `pack_1` applies `modifications` to a base with the children
/// `x`, `y`, `z`, and the `uiPackError` keys raised.
fn modified(modifications: &str) -> (Vec<String>, Vec<&'static str>) {
    let overlay = format!(r#"{{ "root": {{ "modifications": {modifications} }} }}"#);
    let stack = stack_of(&[
        &[
            ("ui/_ui_defs.json", r#"{ "ui_defs": ["ui/a.json"] }"#),
            (
                "ui/a.json",
                r#"{ "namespace": "a", "leaf": { "type": "label", "text": "t" },
                    "root": { "type": "panel", "controls": [ { "x@a.leaf": {} }, { "y@a.leaf": {} }, { "z@a.leaf": {} } ] } }"#,
            ),
        ],
        &[("ui/a.json", &overlay)],
    ]);
    let ui = stack.ui();
    let names = control(&stack, "a", "root")
        .children()
        .into_iter()
        .map(|(key, _)| key.name)
        .collect();
    let errors = ui
        .reports
        .iter()
        .flat_map(|report| &report.errors)
        .map(|error| error.kind.key())
        .collect();
    (names, errors)
}

fn names(modifications: &str) -> String {
    let (names, errors) = modified(modifications);
    assert!(errors.is_empty(), "{modifications}: {errors:?}");
    names.join(",")
}

const NEW: &str = r#"{ "n@a.leaf": {} }"#;

#[test]
fn every_operation_places_its_element() {
    let op = |operation: &str, rest: &str| {
        names(&format!(
            r#"[ {{ "control_name": "y", "operation": "{operation}"{rest} }} ]"#
        ))
    };
    let value = format!(r#", "value": {NEW}"#);
    assert_eq!(op("insert_back", &value), "x,y,z,n");
    assert_eq!(op("insert_front", &value), "n,x,y,z");
    assert_eq!(op("insert_after", &value), "x,y,n,z");
    assert_eq!(op("insert_before", &value), "x,n,y,z");
    assert_eq!(op("move_back", ""), "x,z,y");
    assert_eq!(op("move_front", ""), "y,x,z");
    assert_eq!(op("move_after", r#", "target_control": "z""#), "x,z,y");
    assert_eq!(op("move_before", r#", "target_control": "x""#), "y,x,z");
    assert_eq!(op("swap", r#", "target_control": "x""#), "y,x,z");
    assert_eq!(op("remove", ""), "x,z");
    assert_eq!(op("replace", &value), "x,n,z");
    // a `target` is a where-style condition
    assert_eq!(
        names(r#"[ { "control_name": "x", "operation": "move_after", "target": "any" } ]"#),
        "x,y,z"
    );
    // moving next to itself changes nothing
    assert_eq!(op("move_after", r#", "target_control": "y""#), "x,y,z");
}

#[test]
fn a_list_that_is_no_array_is_ignored() {
    assert_eq!(
        names(r#"{ "control_name": "x", "operation": "remove" }"#),
        "x,y,z"
    );
}

#[test]
fn an_explicit_array_name_is_used_with_a_control_name() {
    // `array_name` given beside `control_name` is used as written: no `x` in `bindings`
    let (names, errors) =
        modified(r#"[ { "control_name": "x", "array_name": "bindings", "operation": "remove" } ]"#);
    assert_eq!(names.join(","), "x,y,z");
    assert_eq!(errors, ["uiPackError.missingControlTarget"]);
    // an empty one is the default, `controls`
    assert_eq!(
        self::names(r#"[ { "control_name": "x", "array_name": "", "operation": "remove" } ]"#),
        "y,z"
    );
}

#[test]
fn each_missing_part_of_a_modification_is_its_own_error() {
    let kind = |entry: &str| modified(&format!("[ {entry} ]")).1;
    // L11: the condition, for every operation that needs one
    for operation in [
        "insert_after",
        "insert_before",
        "move_back",
        "move_front",
        "remove",
        "replace",
    ] {
        let entry = format!(
            r#"{{ "array_name": "controls", "operation": "{operation}", "value": {NEW} }}"#
        );
        assert_eq!(
            kind(&entry),
            ["uiPackError.missingCondition"],
            "{operation}"
        );
    }
    // L11: a null `where` is no condition; L11: the second condition of a move or a swap
    assert_eq!(
        kind(r#"{ "array_name": "controls", "where": null, "operation": "remove" }"#),
        ["uiPackError.missingCondition"]
    );
    for operation in ["move_after", "move_before", "swap"] {
        let entry = format!(r#"{{ "control_name": "x", "operation": "{operation}" }}"#);
        assert_eq!(
            kind(&entry),
            ["uiPackError.missingCondition"],
            "{operation}"
        );
        let entry = format!(
            r#"{{ "control_name": "x", "operation": "{operation}", "target_control": "" }}"#
        );
        assert_eq!(
            kind(&entry),
            ["uiPackError.missingCondition"],
            "{operation}"
        );
        // L12: a target name that matches nothing
        let entry = format!(
            r#"{{ "control_name": "x", "operation": "{operation}", "target_control": "nope" }}"#
        );
        assert_eq!(
            kind(&entry),
            ["uiPackError.missingControlTarget"],
            "{operation}"
        );
    }
    // L13: the value, for every insert and for replace; a null value is none
    for operation in [
        "insert_back",
        "insert_front",
        "insert_after",
        "insert_before",
        "replace",
    ] {
        let entry = format!(r#"{{ "control_name": "x", "operation": "{operation}" }}"#);
        assert_eq!(kind(&entry), ["uiPackError.missingValue"], "{operation}");
        let entry =
            format!(r#"{{ "control_name": "x", "operation": "{operation}", "value": null }}"#);
        assert_eq!(kind(&entry), ["uiPackError.missingValue"], "{operation}");
    }
    // operations that need no value do not ask for one
    for operation in ["move_back", "move_front", "remove"] {
        let entry = format!(r#"{{ "control_name": "x", "operation": "{operation}" }}"#);
        assert!(kind(&entry).is_empty(), "{operation}");
    }
    // L9, L8: an empty operation and an empty array name are missing
    assert_eq!(
        kind(r#"{ "control_name": "x", "operation": "" }"#),
        ["uiPackError.missingOperation"]
    );
    assert_eq!(
        kind(r#"{ "where": { "a": 1 }, "array_name": "", "operation": "remove" }"#),
        ["uiPackError.missingArrayName"]
    );
    // every failing modification of a list is reported
    assert_eq!(
        kind(r#"{ "control_name": "x" }, { "control_name": "nope", "operation": "remove" }"#),
        [
            "uiPackError.missingOperation",
            "uiPackError.missingControlTarget"
        ]
    );
}

#[test]
fn a_where_object_matches_on_one_equal_member_only() {
    let removed = |condition: &str| {
        let overlay = format!(
            r#"{{ "root": {{ "modifications": [ {{ "array_name": "bindings", "operation": "remove", "where": {condition} }} ] }} }}"#
        );
        let stack = stack_of(&[
            &[
                ("ui/_ui_defs.json", r#"{ "ui_defs": ["ui/a.json"] }"#),
                (
                    "ui/a.json",
                    r##"{ "root": { "type": "panel", "bindings": [ { "binding_name": "#a" }, null, { "binding_name": "#b", "binding_condition": "once" } ] } }"##,
                ),
            ],
            &[("ui/a.json", &overlay)],
        ]);
        let left = control(&stack, ROOT_NAMESPACE, "root")
            .get("bindings")
            .unwrap()
            .as_array()
            .unwrap()
            .len();
        (left, texts(&stack, "UnmatchedCondition").len())
    };
    assert_eq!(removed(r##"{ "binding_name": "#b" }"##), (2, 0));
    // a member with another value, or one the element lacks, is no match
    assert_eq!(removed(r##"{ "binding_name": "#c" }"##), (3, 1));
    assert_eq!(removed(r#"{ "missing": 1 }"#), (3, 1));
    assert_eq!(removed("{}"), (3, 1));
    // any other non-null type takes the first element; an empty array has none
    assert_eq!(removed("7"), (2, 0));
    assert_eq!(removed("[]"), (3, 1));
}

#[test]
fn a_scalar_where_on_an_empty_array_matches_nothing() {
    let stack = stack_of(&[
        &[
            ("ui/_ui_defs.json", r#"{ "ui_defs": ["ui/a.json"] }"#),
            ("ui/a.json", r#"{ "root": { "type": "panel" } }"#),
        ],
        &[(
            "ui/a.json",
            r#"{ "root": { "modifications": [ { "array_name": "bindings", "operation": "remove", "where": true } ] } }"#,
        )],
    ]);
    assert_eq!(texts(&stack, "UnmatchedCondition").len(), 1);
    // no array is invented where nothing was inserted
    assert!(
        control(&stack, ROOT_NAMESPACE, "root")
            .get("bindings")
            .is_none()
    );
}

#[test]
fn an_insert_creates_the_array() {
    let stack = stack_of(&[
        &[
            ("ui/_ui_defs.json", r#"{ "ui_defs": ["ui/a.json"] }"#),
            ("ui/a.json", r#"{ "root": { "type": "panel" } }"#),
        ],
        &[(
            "ui/a.json",
            r##"{ "root": { "modifications": [ { "array_name": "bindings", "operation": "insert_back", "value": { "binding_name": "#n" } } ] } }"##,
        )],
    ]);
    let bindings = control(&stack, ROOT_NAMESPACE, "root")
        .get("bindings")
        .unwrap();
    assert_eq!(bindings.as_array().unwrap().len(), 1);
}

// ---- definitions and references -------------------------------------------------------------

#[test]
fn r01_default_namespace() {
    let stack = check("r01_default_namespace");
    assert_eq!(ROOT_NAMESPACE, "_root");
    assert_eq!(control(&stack, "_root", "k").namespace, "_root");
}

#[test]
fn no_definition_has_an_empty_namespace() {
    // L19: vanilla asserts "Namespace should not be empty at this point."; an empty
    // `namespace` is `_root` here
    let stack = one_file(r#"{ "namespace": "", "k": { "type": "panel" } }"#);
    assert!(stack.ui().get("_root", "k").is_some());
    assert!(
        stack
            .ui()
            .controls()
            .all(|control| !control.namespace.is_empty())
    );
    assert!(
        stack
            .ui()
            .namespaces
            .keys()
            .all(|namespace| !namespace.is_empty())
    );
}

#[test]
fn the_namespace_member_is_no_definition_deviation() {
    // vanilla keeps an inert definition named `namespace`; ours leaves it out
    let stack = one_file(r#"{ "namespace": "a", "k": { "type": "panel" } }"#);
    assert!(stack.ui().get("a", "namespace").is_none());
    assert_eq!(stack.ui().controls().count(), 1);
}

#[test]
fn a_namespace_that_is_no_string_leaves_the_file_in_root() {
    let stack = one_file(r#"{ "namespace": 5, "k": { "type": "panel" } }"#);
    assert!(stack.ui().get("_root", "k").is_some());
    assert_eq!(
        findings(&stack),
        vec![("InvalidType", Severity::Info, "wrong JSON type".to_owned())]
    );
}

#[test]
fn r02_same_namespace_ref() {
    check("r02_same_namespace_ref");
}

#[test]
fn r03_shallow_inheritance() {
    let stack = check("r03_shallow_inheritance");
    let derived = control(&stack, "a", "d");
    // `raw` is the definition before flattening
    assert!(derived.raw.as_object().unwrap().get("size").is_none());
    assert!(derived.get("size").is_some());
    assert_eq!(derived.source_file, "ui/a.json");
    assert_eq!(derived.key.to_string(), "d@a.b");
}

#[test]
fn r04_chain() {
    check("r04_chain");
}

#[test]
fn r05_cycle_deviation() {
    // vanilla: the walk stops silently
    check("r05_cycle");
}

#[test]
fn a_definition_that_names_itself_is_a_cycle() {
    let stack = one_file(r#"{ "namespace": "a", "k@a.k": { "type": "panel" } }"#);
    assert_eq!(texts(&stack, "ReferenceCycle").len(), 1);
}

#[test]
fn r06_missing_base_deviation() {
    // vanilla: silent at load; "Type not specified (or @-base not found)" at creation
    let stack = check("r06_missing_base");
    let missing = stack
        .ui()
        .diagnostics()
        .find(|d| matches!(d.code, DiagnosticCode::UnresolvedReference { .. }))
        .unwrap();
    assert!(
        missing.to_string().ends_with(": ui definition 'a.nope'"),
        "{missing}"
    );
}

#[test]
fn r07_duplicate_across_files_deviation() {
    // vanilla: the first `emplace` wins, silently
    let stack = check("r07_duplicate_across_files");
    assert_eq!(control(&stack, "a", "leaf").source_file, "ui/a.json");
    let duplicate = stack.ui().diagnostics().next().unwrap();
    assert_eq!(
        duplicate.location.path.as_ref().unwrap().as_str(),
        "ui/b.json"
    );
}

#[test]
fn r08_var_in_reference() {
    check("r08_var_in_reference");
}

#[test]
fn a_reference_variable_comes_from_the_definition_then_the_globals() {
    let files = |globals: &'static str, body: &'static str| {
        let file = format!(
            r#"{{ "namespace": "a", "one": {{ "type": "panel" }}, "two": {{ "type": "label", "text": "t" }}, "k@a.$which": {body} }}"#
        );
        let stack = stack_of(&[&[
            ("ui/_ui_defs.json", r#"{ "ui_defs": ["ui/a.json"] }"#),
            ("ui/_global_variables.json", globals),
            ("ui/a.json", &file),
        ]]);
        let k = control(&stack, "a", "k");
        (
            k.get("type")
                .map(|t| t.as_string().unwrap().as_str().to_owned()),
            findings(&stack).len(),
        )
    };
    assert_eq!(
        files(r#"{ "$which": "one" }"#, "{}"),
        (Some("panel".to_owned()), 0)
    );
    assert_eq!(
        files(r#"{ "$which": "one" }"#, r#"{ "$which": "two" }"#),
        (Some("label".to_owned()), 0)
    );
    assert_eq!(
        files("{}", r#"{ "$which|default": "two" }"#),
        (Some("label".to_owned()), 0)
    );
    // a variable nothing here defines is the game's to resolve: no finding
    assert_eq!(files("{}", "{}"), (None, 0));
}

// ---- globals --------------------------------------------------------------------------------

#[test]
fn g01_global_vars_merge() {
    let stack = check("g01_global_vars_merge");
    let ui = stack.ui();
    let origin = ui.origin(&"ui/_global_variables.json".to_owned()).unwrap();
    assert_eq!(origin.pack.index(), Some(0));
    assert_eq!(origin.contributors.len(), 1);
    assert!(ui.global_variables.get("$new").is_some());
}

#[test]
fn g02_color_format() {
    let stack = check("g02_color_format");
    let globals = &stack.ui().global_variables;
    assert_eq!(COLOR_VARIABLES.len(), 28);
    let table: Vec<_> = globals.color_format().collect();
    assert_eq!(
        table,
        vec![
            ("$a_color_format", [1.0, 0.0, 0.0, 1.0]),
            ("$coin_color", [1.0, 1.0, 0.0, 1.0]),
        ]
    );
    assert_eq!(globals.color("$b_color_format"), None);
}

#[test]
fn g03_global_vars_base_parse_error() {
    let stack = check("g03_global_vars_base_parse_error");
    assert_eq!(
        stack.ui().origin(&"ui/_global_variables.json".to_owned()),
        None
    );
}

#[test]
fn g04_global_vars_overlay_parse_error_deviation() {
    // vanilla skips the copy silently
    check("g04_global_vars_overlay_parse_error");
}

// ---- depth ----------------------------------------------------------------------------------

#[test]
fn nesting_up_to_the_json_depth_limit_loads_and_deeper_is_no_json() {
    // Three JSON levels per child (`controls`, the element, its body): 40 children nest to
    // 122 levels, under the reader's limit of 128; the merge, the flattening and the checks
    // all walk that deep. Run once with full debug info (COMMON "Build profiles differ").
    let nested = |levels: usize| {
        let mut body = r#"{ "type": "panel" }"#.to_owned();
        for _ in 0..levels {
            body = format!(r#"{{ "type": "panel", "controls": [ {{ "c": {body} }} ] }}"#);
        }
        format!(r#"{{ "root": {body} }}"#)
    };
    let deep = nested(40);
    let overlay = r#"{ "root": { "layer": 1 } }"#;
    let stack = stack_of(&[
        &[
            ("ui/_ui_defs.json", r#"{ "ui_defs": ["ui/a.json"] }"#),
            ("ui/a.json", &deep),
        ],
        &[("ui/a.json", overlay)],
    ]);
    assert!(findings(&stack).is_empty(), "{:?}", findings(&stack));
    let mut depth = 0;
    let mut node = control(&stack, ROOT_NAMESPACE, "root").body();
    while let Some(child) = node
        .as_object()
        .and_then(|o| o.get("controls")?.as_array()?.get(0)?.as_object()?.get("c"))
        .cloned()
    {
        depth += 1;
        node = child;
    }
    assert_eq!(depth, 40);
    // past the limit the copy is no JSON: the file contributes nothing
    let stack = one_file(&nested(60));
    assert_eq!(texts(&stack, "MalformedInput").len(), 1);
    assert!(stack.ui().files.is_empty());
}

#[test]
fn a_reference_names_another_namespace_or_its_own() {
    let stack = stack_of(&[&[
        (
            "ui/_ui_defs.json",
            r#"{ "ui_defs": ["ui/a.json", "ui/b.json"] }"#,
        ),
        (
            "ui/a.json",
            r#"{ "namespace": "a", "base": { "type": "panel" }, "k@b.base": {}, "own@.base": {}, "plain@base": {} }"#,
        ),
        (
            "ui/b.json",
            r#"{ "namespace": "b", "base": { "type": "label", "text": "t" } }"#,
        ),
    ]]);
    let type_of = |name: &str| {
        control(&stack, "a", name)
            .get("type")
            .and_then(|t| t.as_string())
            .map(|t| t.as_str().to_owned())
    };
    assert_eq!(type_of("k").as_deref(), Some("label"));
    assert_eq!(type_of("plain").as_deref(), Some("panel"));
    // an empty namespace before the dot is the file's own
    assert_eq!(type_of("own").as_deref(), Some("panel"));
    assert!(findings(&stack).is_empty(), "{:?}", findings(&stack));
}

#[test]
fn an_overlay_value_of_another_shape_replaces_the_base_value() {
    let stack = stack_of(&[
        &[
            ("ui/_ui_defs.json", r#"{ "ui_defs": ["ui/a.json"] }"#),
            (
                "ui/a.json",
                r#"{ "k": { "type": "panel", "$a": { "x": 1 }, "$b": 2, "$c": { "x": 1 } } }"#,
            ),
        ],
        &[(
            "ui/a.json",
            r#"{ "k": { "$a": 7, "$b": { "y": 1 }, "$c": { "y": 2 } } }"#,
        )],
    ]);
    let k = control(&stack, ROOT_NAMESPACE, "k");
    // only two objects merge; a scalar over an object, or an object over a scalar, replaces
    assert_eq!(k.get("$a"), Some(&json("7")));
    assert_eq!(k.get("$b"), Some(&json(r#"{ "y": 1 }"#)));
    assert_eq!(k.get("$c"), Some(&json(r#"{ "x": 1, "y": 2 }"#)));
}

#[test]
fn the_pack_reports_are_top_first() {
    let base = [
        ("ui/_ui_defs.json", r#"{ "ui_defs": ["ui/a.json"] }"#),
        ("ui/a.json", r#"{ "k": { "type": "panel" } }"#),
    ];
    let stack = stack_of(&[
        &base,
        &[("ui/a.json", r#"{ "k/one": {} }"#)],
        &[("ui/a.json", r#"{ "k/two": {} }"#)],
    ]);
    let reports: Vec<(Option<usize>, &str)> = stack
        .ui()
        .reports
        .iter()
        .map(|report| (report.pack.index(), report.errors[0].args[1].as_str()))
        .collect();
    assert_eq!(reports, [(Some(0), "k/two"), (Some(1), "k/one")]);
}

// ---- long chains ----------------------------------------------------------------------------

/// A file of `count` definitions `d<i>@d<next(i)>`, with `d0` as given.
fn chained(count: usize, first: &str, next: impl Fn(usize) -> Option<usize>) -> String {
    let mut text = String::from("{ \"namespace\": \"a\"");
    for index in 0..count {
        match next(index) {
            Some(base) => text.push_str(&format!(", \"d{index}@d{base}\": {{}}")),
            None => text.push_str(&format!(", \"d{index}\": {first}")),
        }
    }
    text.push_str(" }");
    text
}

// The tests of this section hold the load of one file to the crate's one protocol:
// the text is built before the clock starts; the stack of the one file and its UI group are
// loaded on the clock; what was loaded is checked after it stops. The walk was 19 s for a
// chain of 4 000; materialised per definition, a growing chain of 6 000 took longer than
// 10 s, and one of 20 000 more memory than the machine has.

/// The smaller size of the long chains; the larger is four times it (20 000).
const LONG_CHAIN: usize = 5_000;

/// The stack of the one file `text` with its UI group loaded, and how long both took.
fn timed_load(
    text: &str,
) -> (
    bedrock_addon::pack::PackStack<bedrock_addon::pack::Resource>,
    std::time::Duration,
) {
    crate::cost::timed(|| {
        let stack = one_file(text);
        stack.ui();
        stack
    })
}

#[test]
fn a_chain_of_twenty_thousand_bases_resolves_in_linear_time() {
    crate::cost::assert_linear_timed_at("ui chain of bases", LONG_CHAIN, |count| {
        let text = chained(count, r#"{ "type": "panel", "layer": 3 }"#, |i| {
            i.checked_sub(1)
        });
        let (stack, took) = timed_load(&text);
        assert_eq!(stack.ui().controls().count(), count);
        let last = control(&stack, "a", &format!("d{}", count - 1));
        assert_eq!(last.get("layer"), Some(&json("3")));
        assert_eq!(last.get("type"), Some(&json(r#""panel""#)));
        assert!(findings(&stack).is_empty());
        took
    });
}

#[test]
fn a_cycle_of_twenty_thousand_bases_resolves_in_linear_time() {
    crate::cost::assert_linear_timed_at("ui cycle of bases", LONG_CHAIN, |count| {
        let text = chained(count, "{}", |i| Some((i + 1) % count));
        let (stack, took) = timed_load(&text);
        assert_eq!(stack.ui().controls().count(), count);
        // every member of the cycle is reported, and nothing is inherited from an empty cycle
        assert_eq!(texts(&stack, "ReferenceCycle").len(), count);
        assert_eq!(control(&stack, "a", "d7").body(), json("{}"));
        took
    });
}

/// A file of `count` definitions in namespace `a`: `d0` has a type, every other one names
/// the one before it as its base, and each adds a variable of its own (`$m<i>`), so the
/// flattened definitions together hold `count`² / 2 members.
fn growing_chain(count: usize) -> String {
    let mut text =
        String::from("{ \"namespace\": \"a\", \"d0\": { \"type\": \"panel\", \"$m0\": 0 }");
    for index in 1..count {
        text.push_str(&format!(
            ", \"d{index}@d{}\": {{ \"$m{index}\": {index} }}",
            index - 1
        ));
    }
    text.push_str(" }");
    text
}

/// How many definitions the growing chains have at the larger size.
const GROWING_CHAIN: usize = 4 * LONG_CHAIN;

/// The most memory this process has held, in bytes (`VmHWM`); `None` where the system does
/// not say.
fn peak_memory() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let line = status.lines().find(|line| line.starts_with("VmHWM:"))?;
    let kilobytes: u64 = line.split_whitespace().nth(1)?.parse().ok()?;
    Some(kilobytes * 1024)
}

#[test]
fn a_chain_where_every_definition_adds_a_member_loads_in_linear_time_and_memory() {
    crate::cost::assert_linear_timed_at("ui growing chain", LONG_CHAIN, |count| {
        let text = growing_chain(count);
        let (stack, took) = timed_load(&text);
        assert_eq!(stack.ui().controls().count(), count);
        assert!(findings(&stack).is_empty());
        took
    });
    let stack = one_file(&growing_chain(GROWING_CHAIN));
    // The whole test binary, every other test of it included, stays far below what the
    // flattened table would take (200 million members at 20 000).
    let peak = peak_memory();
    if std::env::var_os("BEDROCK_ADDON_SMOKE_VERBOSE").is_some() {
        eprintln!("growing chain of {GROWING_CHAIN}: peak memory {peak:?} bytes");
    }
    assert!(peak.is_none_or(|peak| peak < 2 << 30), "{peak:?}");

    // a member is found wherever in the chain it is written
    let last = control(&stack, "a", &format!("d{}", GROWING_CHAIN - 1));
    assert_eq!(last.get("type"), Some(&json(r#""panel""#)));
    assert_eq!(last.get("$m0"), Some(&json("0")));
    assert_eq!(last.get("$m7"), Some(&json("7")));
    assert_eq!(last.get("$nothing"), None);
    // and the flattened definition is built when it is asked for, its members by key
    let body = last.body();
    let body = body.as_object().unwrap();
    assert_eq!(body.len(), GROWING_CHAIN + 1);
    assert_eq!(body.keys().next().unwrap().as_str(), "$m0");
    assert_eq!(body.keys().last().unwrap().as_str(), "type");
}

#[test]
fn a_cycle_where_every_definition_adds_a_member_loads_in_linear_time() {
    crate::cost::assert_linear_timed_at("ui growing cycle", LONG_CHAIN, |count| {
        // d<i>@d<i+1>, the last one naming the first: each inherits around the cycle once
        let mut text = String::from("{ \"namespace\": \"a\"");
        for index in 0..count {
            text.push_str(&format!(
                ", \"d{index}@d{}\": {{ \"$m{index}\": {index} }}",
                (index + 1) % count
            ));
        }
        text.push_str(" }");
        let (stack, took) = timed_load(&text);
        assert_eq!(texts(&stack, "ReferenceCycle").len(), count);
        let seventh = control(&stack, "a", "d7");
        assert_eq!(seventh.get("$m7"), Some(&json("7")));
        assert_eq!(seventh.get("$m6"), Some(&json("6")));
        assert_eq!(seventh.body().as_object().unwrap().len(), count);
        took
    });
}

#[test]
fn children_naming_the_definitions_of_a_long_chain_are_checked_in_linear_time() {
    crate::cost::assert_linear_timed_at("ui children along a chain", LONG_CHAIN, |count| {
        // every definition of the chain writes a child that names the definition before it:
        // the child's type is the one the first definition writes, at the far end of the chain
        let mut text = String::from(
            "{ \"namespace\": \"a\", \"d0\": { \"type\": \"panel\", \"from\": [0, 0] }",
        );
        for index in 1..count {
            let base = index - 1;
            text.push_str(&format!(
                ", \"d{index}@d{base}\": {{ \"$m{index}\": 1, \"controls\": [{{ \"c@a.d{base}\": {{ \"layer\": 1 }} }}] }}"
            ));
        }
        // an animation at the near end of a chain of its own: its endpoints are inherited
        text.push_str(
            ", \"s0\": { \"anim_type\": \"size\", \"from\": [0, 0], \"to\": [\"100%\", 0] }",
        );
        for index in 1..count {
            text.push_str(&format!(
                ", \"s{index}@s{}\": {{ \"duration\": {index} }}",
                index - 1
            ));
        }
        text.push_str(&format!(
            ", \"bad@s{}\": {{ \"to\": [\"100% + 1px\", 0] }}",
            count - 1
        ));
        text.push_str(" }");
        let (stack, took) = timed_load(&text);
        assert_eq!(stack.ui().controls().count(), 2 * count + 1);
        // the one finding is the animation whose own `to` does not pair with the inherited
        // `from`
        assert_eq!(
            texts(&stack, "InvalidValue"),
            ["The animation needs to have the same number x values."]
        );
        assert_eq!(findings(&stack).len(), 1, "{:?}", findings(&stack));
        took
    });
}

#[test]
fn a_definition_names_its_base_and_never_prints_or_compares_the_chain() {
    let stack = one_file(&growing_chain(GROWING_CHAIN));
    let last = control(&stack, "a", &format!("d{}", GROWING_CHAIN - 1));
    // `Debug` names the base; nothing of the 19 999 definitions behind it is printed
    let printed = format!("{last:?}");
    assert!(
        printed.contains(r#"inherited: From("a", "d19998")"#),
        "{printed}"
    );
    assert!(printed.len() < 600, "{}", printed.len());
    assert!(!printed.contains("$m19998"), "{printed}");
    assert!(format!("{:?}", control(&stack, "a", "d0")).contains("inherited: Nothing"));
    // the whole table prints, compares and is copied in the time its size takes
    crate::cost::assert_linear_at(
        "ui table printed, copied and compared",
        LONG_CHAIN,
        |count| {
            let stack = one_file(&growing_chain(count));
            stack.ui();
            (stack, count)
        },
        |(stack, count)| {
            let ui = stack.ui();
            let table = format!("{:?}", ui.namespaces);
            assert!(table.len() < count * 600, "{}", table.len());
            let copy = ui.namespaces.clone();
            assert!(copy == ui.namespaces);
        },
    );
    // a copy of one definition keeps the chain alive by itself, and letting go of it at the
    // far end of the chain does not recurse through every base
    let alone = last.clone();
    drop(stack);
    assert_eq!(alone.get("$m0"), Some(&json("0")));
    drop(alone);
}

#[test]
fn definitions_written_alike_with_different_bases_are_not_equal() {
    let files = |base: &'static str| {
        stack_of(&[&[
            ("ui/_ui_defs.json", r#"{ "ui_defs": ["ui/a.json"] }"#),
            ("ui/_global_variables.json", base),
            (
                "ui/a.json",
                r#"{ "namespace": "a", "x": { "layer": 1 }, "y": { "layer": 2 }, "k@$base": {} }"#,
            ),
        ]])
    };
    let (on_x, again, on_y) = (
        files(r#"{ "$base": "x" }"#),
        files(r#"{ "$base": "x" }"#),
        files(r#"{ "$base": "y" }"#),
    );
    let k = |stack| control(stack, "a", "k");
    assert_eq!(k(&on_x).get("layer"), Some(&json("1")));
    assert_eq!(k(&on_y).get("layer"), Some(&json("2")));
    assert_eq!(k(&on_x).raw, k(&on_y).raw);
    assert!(k(&on_x) == k(&again));
    assert!(k(&on_x) != k(&on_y));
}

#[test]
fn one_member_read_through_the_chain_is_the_flattened_definitions_member() {
    // chains, a cycle, a definition leading into it and nulls at every distance
    let stack = one_file(
        r#"{ "namespace": "a",
            "far": { "type": "panel", "a": 1, "b": 2, "c": 3, "gone": null },
            "near@far": { "a": null, "c": 30, "n": [1] },
            "k@near": { "b": null, "own": true },
            "d@p": { "own": 1, "x": null },
            "p@q": { "x": "p", "only_p": 1, "hole": null },
            "q@p": { "x": "q", "only_q": 2, "hole": 5 },
            "text": "no object",
            "t@text": { "z": 1 } }"#,
    );
    let keys = [
        "type", "a", "b", "c", "gone", "n", "own", "x", "only_p", "only_q", "hole", "z", "none",
    ];
    for control in stack.ui().controls() {
        let body = control.body();
        for key in keys {
            let flattened = body
                .as_object()
                .and_then(|body| body.get(key))
                .filter(|member| !member.is_null());
            assert_eq!(control.get(key), flattened, "{}: {key}", control.key);
        }
    }
    assert_eq!(control(&stack, "a", "text").body(), json(r#""no object""#));
    assert_eq!(control(&stack, "a", "t").body(), json(r#"{ "z": 1 }"#));
    assert_eq!(
        control(&stack, "a", "d").body(),
        json(r#"{ "own": 1, "x": null, "only_p": 1, "only_q": 2, "hole": 5 }"#)
    );
}

#[test]
fn a_definition_leading_into_a_cycle_inherits_around_it_once() {
    // `d` inherits from `p`; `p` and `q` name each other. Each takes the nearest definition's
    // value of a member, a null member of a base is not inherited, and all three are reported.
    let stack = one_file(
        r#"{ "namespace": "a",
            "d@p": { "own": 1 },
            "p@q": { "x": "p", "only_p": 1, "hole": null },
            "q@p": { "x": "q", "only_q": 2, "hole": 5 } }"#,
    );
    assert_eq!(
        control(&stack, "a", "d").body(),
        json(r#"{ "own": 1, "x": "p", "only_p": 1, "only_q": 2, "hole": 5 }"#)
    );
    assert_eq!(
        control(&stack, "a", "p").body(),
        json(r#"{ "x": "p", "only_p": 1, "hole": null, "only_q": 2 }"#)
    );
    assert_eq!(
        control(&stack, "a", "q").body(),
        json(r#"{ "x": "q", "only_q": 2, "hole": 5, "only_p": 1 }"#)
    );
    assert_eq!(texts(&stack, "ReferenceCycle").len(), 3);
}

#[test]
fn around_a_cycle_the_nearest_definition_after_one_wins() {
    // a -> b -> c -> a. `x` is written by `a` and `c`, `y` by `a` and `b`: each definition
    // takes the first value on its way around, and `o` takes what `b` hands on
    let stack = one_file(
        r#"{ "namespace": "n",
            "a@b": { "x": "a", "y": "a" },
            "b@c": { "y": "b" },
            "c@a": { "x": "c" },
            "o@b": {} }"#,
    );
    let value = |name: &str, key: &str| control(&stack, "n", name).get(key).cloned();
    assert_eq!(value("b", "x"), Some(json(r#""c""#)));
    assert_eq!(value("c", "y"), Some(json(r#""a""#)));
    assert_eq!(value("a", "y"), Some(json(r#""a""#)));
    assert_eq!(value("o", "x"), Some(json(r#""c""#)));
    assert_eq!(value("o", "y"), Some(json(r#""b""#)));
    assert_eq!(
        control(&stack, "n", "o").body(),
        json(r#"{ "x": "c", "y": "b" }"#)
    );
    assert_eq!(
        control(&stack, "n", "b").body(),
        json(r#"{ "y": "b", "x": "c" }"#)
    );
}

#[test]
fn the_flattened_definition_lists_its_members_by_key() {
    // the game's flattened definition is a jsoncpp object, which iterates by key: neither the
    // order a file writes its members in nor where in a chain or a cycle a member comes from
    // shows in `body()`
    let stack = one_file(
        r#"{ "namespace": "n",
            "far": { "type": "panel", "m": 1, "b": 2 },
            "k@far": { "z": 1, "a": null, "c": 3 },
            "p@q": { "y": 1, "d": 1 },
            "q@r": { "x": 1, "e": 1 },
            "r@p": { "w": 1, "a": 1 },
            "o@q": { "f": 1 } }"#,
    );
    let keys = |name: &str| -> Vec<String> {
        let body = control(&stack, "n", name).body();
        let body = body.as_object().unwrap();
        body.keys().map(|key| key.as_str().to_owned()).collect()
    };
    assert_eq!(keys("k"), ["a", "b", "c", "m", "type", "z"]);
    for name in ["p", "q", "r"] {
        assert_eq!(keys(name), ["a", "d", "e", "w", "x", "y"], "{name}");
    }
    assert_eq!(keys("o"), ["a", "d", "e", "f", "w", "x", "y"]);
}

#[test]
fn a_null_member_of_a_base_is_not_inherited() {
    // "every member of a base that is non-null and absent": the null of the nearer base is
    // skipped and the farther base's value is taken; an own null stays
    let stack = one_file(
        r#"{ "namespace": "a",
            "far": { "type": "panel", "a": 1, "b": 2, "c": 3 },
            "near@far": { "a": null, "c": 30 },
            "k@near": { "b": null } }"#,
    );
    assert_eq!(
        control(&stack, "a", "k").body(),
        json(r#"{ "b": null, "c": 30, "a": 1, "type": "panel" }"#)
    );
    assert_eq!(
        control(&stack, "a", "near").body(),
        json(r#"{ "a": null, "c": 30, "b": 2, "type": "panel" }"#)
    );
    assert!(findings(&stack).is_empty());
}

#[test]
fn a_missing_base_is_reported_along_the_whole_chain() {
    let stack =
        one_file(r#"{ "namespace": "a", "k@j": { "type": "panel" }, "j@gone": { "layer": 1 } }"#);
    assert_eq!(texts(&stack, "UnresolvedReference").len(), 2);
    assert_eq!(control(&stack, "a", "k").get("layer"), Some(&json("1")));
}

#[test]
fn of_two_base_members_with_one_short_name_an_overlay_edits_the_first() {
    // The folder says "record shortName → fullKey" and not which of two keys stays; the
    // first in jsoncpp's member order is kept (inferred): it is also the definition
    // that wins the table.
    let stack = stack_of(&[
        &[
            ("ui/_ui_defs.json", r#"{ "ui_defs": ["ui/a.json"] }"#),
            (
                "ui/a.json",
                r#"{ "namespace": "a", "x": { "type": "panel" }, "y": { "type": "panel" },
                    "k@a.y": { "layer": 2 }, "k@a.x": { "layer": 1 } }"#,
            ),
        ],
        &[("ui/a.json", r#"{ "k": { "visible": false } }"#)],
    ]);
    let merged = file_json(&stack.ui().files["ui/a.json"]);
    let merged = merged.as_object().unwrap();
    assert_eq!(
        merged.get("k@a.x"),
        Some(&json(r#"{ "layer": 1, "visible": false }"#))
    );
    assert_eq!(merged.get("k@a.y"), Some(&json(r#"{ "layer": 2 }"#)));
    assert!(merged.get("k").is_none());
}

#[test]
fn the_name_of_an_element_with_several_members_is_its_first_in_key_order() {
    // jsoncpp iterates an object by key: of `{ "b@…", "a@…" }` the first member is `a`
    let base = [
        ("ui/_ui_defs.json", r#"{ "ui_defs": ["ui/a.json"] }"#),
        (
            "ui/a.json",
            r#"{ "namespace": "a", "leaf": { "type": "label", "text": "t" },
                "root": { "type": "panel", "controls": [ { "b@a.leaf": {}, "a@a.leaf": {} }, { "z@a.leaf": {} } ] } }"#,
        ),
    ];
    let removing = |name: &str| {
        let overlay = format!(
            r#"{{ "root": {{ "modifications": [ {{ "control_name": "{name}", "operation": "remove" }} ] }} }}"#
        );
        let stack = stack_of(&[&base, &[("ui/a.json", &overlay)]]);
        (
            control(&stack, "a", "root").children().len(),
            stack.ui().reports.len(),
        )
    };
    assert_eq!(removing("a"), (1, 0));
    assert_eq!(removing("b"), (2, 1));
    // a path key descends by the same name, into that member
    let stack = stack_of(&[&base, &[("ui/a.json", r#"{ "root/a": { "layer": 4 } }"#)]]);
    let children = control(&stack, "a", "root").children();
    assert_eq!(children[0].0.name, "a");
    assert_eq!(children[0].1, &json(r#"{ "layer": 4 }"#));
    assert!(stack.ui().reports.is_empty());
}

#[test]
fn a_merged_file_may_be_exactly_as_deep_as_one_copy_and_no_deeper_deviation() {
    // n children below `root` are 3n + 2 JSON levels. The base has 30; a path key grafts
    // `grafted` more below the deepest: 42 children are 128 levels, the limit of one copy,
    // and are accepted; 43 are 131 and the overlay is refused
    let nested = |levels: usize| {
        let mut body = r#"{ "type": "panel" }"#.to_owned();
        for _ in 0..levels {
            body = format!(r#"{{ "type": "panel", "controls": [ {{ "c": {body} }} ] }}"#);
        }
        body
    };
    let merged = |grafted: usize| {
        let base = format!(r#"{{ "root": {} }}"#, nested(30));
        let overlay = format!(r#"{{ "root{}": {} }}"#, "/c".repeat(30), nested(grafted));
        let stack = stack_of(&[
            &[
                ("ui/_ui_defs.json", r#"{ "ui_defs": ["ui/a.json"] }"#),
                ("ui/a.json", &base),
            ],
            &[("ui/a.json", &overlay)],
        ]);
        let mut children = 0;
        let mut node = control(&stack, ROOT_NAMESPACE, "root").body();
        while let Some(child) = node
            .as_object()
            .and_then(|o| o.get("controls")?.as_array()?.get(0)?.as_object()?.get("c"))
            .cloned()
        {
            children += 1;
            node = child;
        }
        let errors: Vec<String> = stack
            .ui()
            .reports
            .iter()
            .flat_map(|report| &report.errors)
            .map(|error| error.args[1].clone())
            .collect();
        (children, errors)
    };
    assert_eq!(merged(12), (42, vec![]));
    assert_eq!(
        merged(13),
        (
            30,
            vec!["nesting too deep after the merge: 131 levels, at most 128".to_owned()]
        )
    );
}

#[test]
fn an_overlay_that_nests_the_merged_file_past_the_limit_is_refused_deviation() {
    // Each pack grafts ten more children below the deepest one through a path key. A copy is
    // bounded on its own; the merged file must be too, or sixty packs nest it 1 800 levels
    // deep and every walk over it overflows the stack. Run once with full debug info.
    let nested = |levels: usize| {
        let mut body = r#"{ "type": "panel" }"#.to_owned();
        for _ in 0..levels {
            body = format!(r#"{{ "type": "panel", "controls": [ {{ "c": {body} }} ] }}"#);
        }
        body
    };
    let mut texts: Vec<Vec<(String, String)>> = vec![vec![
        (
            "ui/_ui_defs.json".to_owned(),
            r#"{ "ui_defs": ["ui/a.json"] }"#.to_owned(),
        ),
        (
            "ui/a.json".to_owned(),
            format!(r#"{{ "root": {} }}"#, nested(10)),
        ),
    ]];
    for pack in 1..=60 {
        let path = format!("root{}", "/c".repeat(10 * pack));
        texts.push(vec![(
            "ui/a.json".to_owned(),
            format!(r#"{{ "{path}": {} }}"#, nested(10)),
        )]);
    }
    let packs: Vec<Vec<(&str, &str)>> = texts
        .iter()
        .map(|files| {
            files
                .iter()
                .map(|(p, t)| (p.as_str(), t.as_str()))
                .collect()
        })
        .collect();
    let packs: Vec<&[(&str, &str)]> = packs.iter().map(Vec::as_slice).collect();
    let stack = stack_of(&packs);
    let ui = stack.ui();
    // 10 children are 32 JSON levels; packs 1 to 3 bring the file to 40 children (122
    // levels), pack 4 would make it 152 and is refused like a copy that is too deep itself
    let mut depth = 0;
    let mut node = control(&stack, ROOT_NAMESPACE, "root").body();
    while let Some(child) = node
        .as_object()
        .and_then(|o| o.get("controls")?.as_array()?.get(0)?.as_object()?.get("c"))
        .cloned()
    {
        depth += 1;
        node = child;
    }
    assert_eq!(depth, 40);
    assert_eq!(ui.reports.len(), 57);
    let refused = ui
        .reports
        .iter()
        .find(|r| r.pack.index() == Some(56))
        .unwrap();
    assert_eq!(refused.errors[0].kind, UiPackErrorKind::ParseError);
    assert_eq!(
        refused.errors[0].args[1],
        "nesting too deep after the merge: 152 levels, at most 128"
    );
    // the packs above it name a child that was never grafted
    assert!(
        ui.reports
            .iter()
            .filter(|r| r.pack.index() != Some(56))
            .all(|r| r.errors[0].kind == UiPackErrorKind::MissingControl)
    );
    assert_eq!(
        ui.origin(&"ui/a.json".to_owned()).unwrap().pack.index(),
        Some(57)
    );
}

#[test]
fn a_namespace_member_that_is_no_string_defines_nothing() {
    let stack = one_file(r#"{ "namespace": { "type": "panel" }, "k": { "type": "panel" } }"#);
    assert!(stack.ui().get("_root", "namespace").is_none());
    assert_eq!(stack.ui().controls().count(), 1);
    // it is still a member of the file, for write-back
    assert_eq!(stack.ui().files["ui/a.json"].defs.len(), 2);
}

// ---- severities --------------------------------------------------------------

/// The definition table's own findings (L17 a duplicate, L18 a missing base, a cycle) are the
/// crate's (vanilla is silent at load), and the control-creation texts are what the game logs
/// only when a screen builds the control: every one is an information, and a creation-time
/// text carries the note that says when the game would log it.
#[test]
fn the_table_and_control_creation_findings_are_informations() {
    let stack = stack_of(&[&[
        (
            "ui/_ui_defs.json",
            r#"{ "ui_defs": ["ui/a.json", "ui/b.json"] }"#,
        ),
        (
            "ui/a.json",
            r#"{ "namespace": "a", "leaf": { "type": "label", "text": "1" },
                "k@a.nope": {}, "c@a.c": { "type": "panel" } }"#,
        ),
        (
            "ui/b.json",
            r#"{ "namespace": "a", "leaf": { "type": "label", "text": "2" } }"#,
        ),
    ]]);
    let found: Vec<(&str, Severity, Option<String>)> = stack
        .ui()
        .diagnostics()
        .map(|d| {
            (
                d.code.name(),
                d.severity,
                d.detail.as_deref().map(ToString::to_string),
            )
        })
        .collect();
    let names: Vec<&str> = found.iter().map(|(name, _, _)| *name).collect();
    for code in [
        "DuplicateIdentifier",
        "UnresolvedReference",
        "ReferenceCycle",
        "MissingRequiredField",
    ] {
        assert!(names.contains(&code), "{code}: {found:#?}");
    }
    assert!(
        found
            .iter()
            .all(|(_, severity, _)| *severity == Severity::Info),
        "{found:#?}"
    );
    let creation = found
        .iter()
        .find(|(name, _, _)| *name == "MissingRequiredField")
        .unwrap();
    assert_eq!(
        creation.2.as_deref(),
        Some(": the game logs this only when a screen creates the control")
    );
}
