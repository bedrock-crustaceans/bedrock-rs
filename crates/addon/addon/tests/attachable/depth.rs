//! How deep an attachable file may nest: to the JSON limit and no further. The description's
//! own types nest to a fixed depth; `min_engine_version` is kept as written, whatever it is,
//! and the `null` scan walks the whole value, so both are tried at the limit. Run once with
//! full debug info (COMMON "Build profiles differ").

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::json::MAX_JSON_DEPTH;

use super::common::*;
use super::stacks::*;

/// `demo:ghost` at 1.26.0 with `member` holding `levels` nested arrays around `inner`.
fn nested(member: &str, levels: usize, inner: &str) -> String {
    ghost(
        "1.26.0",
        &format!(
            r#""{member}": {}{inner}{}"#,
            "[".repeat(levels),
            "]".repeat(levels)
        ),
    )
}

/// The root, the payload and the description are three levels.
const OWN: usize = 3;

#[test]
fn a_document_nests_to_the_json_limit_and_no_further() {
    // `min_engine_version` is an array of anything: kept, and written back as it was.
    let deepest = nested("min_engine_version", MAX_JSON_DEPTH - OWN, "1");
    let document = loaded(attachable(&deepest, &cx()));
    clean(document.diagnostics());
    round_trip(&document, |text| attachable(text, &cx()));
    for levels in [MAX_JSON_DEPTH - OWN + 1, MAX_JSON_DEPTH, 100_000] {
        let d = rejected(attachable(
            &nested("min_engine_version", levels, "1"),
            &cx(),
        ));
        assert_eq!(d.len(), 1);
        let found = find(&d, "core/syntax", Severity::Error).unwrap();
        assert_eq!(
            found.code.to_string(),
            "the JSON nests too deeply",
            "{levels}"
        );
    }
}

#[test]
fn a_null_at_the_json_limit_is_found() {
    // In a declared member, as deep as a file may go: the scan reaches it.
    let text = nested("render_controllers", MAX_JSON_DEPTH - OWN, "null");
    let d = rejected(attachable(&text, &cx()));
    assert!(find(&d, "core/unknown_child_option", Severity::Error).is_some());
    // In `min_engine_version` it is part of the value as written.
    let text = nested("min_engine_version", MAX_JSON_DEPTH - OWN, "null");
    let document = loaded(attachable(&text, &cx()));
    round_trip(&document, |text| attachable(text, &cx()));
}

#[test]
fn a_file_nested_beyond_the_limit_is_rejected_in_a_stack_before_anything_reads_it() {
    // The walk peeks at every file's root to tell an attachable from a client entity: the
    // depth is checked before that peek, which would overflow the stack.
    let deep = nested("min_engine_version", 100_000, "1");
    let good = ghost("1.26.0", "");
    let set = stack(&[pack(&[
        ("attachables/deep.json", &deep),
        ("attachables/ghost.json", &good),
    ])]);
    let attachables = set.resource.attachables();
    assert_eq!(attachables.len(), 1);
    assert_eq!(attachables.rejected().len(), 1);
    let rejected = &attachables.rejected()[0];
    assert_eq!(rejected.origin.path.as_str(), "attachables/deep.json");
    let found = find(&rejected.diagnostics, "core/syntax", Severity::Error).unwrap();
    assert_eq!(found.code.to_string(), "the JSON nests too deeply");
    // An unclosed run of brackets is as deep.
    let open = "[".repeat(100_000);
    let d = rejected_text(&open);
    assert!(find(&d, "core/syntax", Severity::Error).is_some());
}

fn rejected_text(text: &str) -> bedrock_addon::diagnostic::Diagnostics {
    rejected(attachable(text, &cx()))
}

// ---- content after the root value ----

#[test]
fn content_after_the_root_is_warned_about_and_the_document_loads() {
    use bedrock_addon::diagnostic::Severity;
    for trailing in ["}", " 5", "\n{\"minecraft:attachable\": 1}", " garbage"] {
        let text = format!("{}{trailing}", ghost("1.26.0", r#""hide_armor": true"#));
        let document = loaded(attachable(&text, &cx()));
        assert!(document.hide_armor());
        let warnings: Vec<_> = document.diagnostics().at_least(Severity::Info).collect();
        assert_eq!(warnings.len(), 1, "{trailing:?}");
        let found = warnings[0];
        assert_eq!(found.code.id(), "core/syntax");
        assert_eq!(found.severity, Severity::Info);
        assert_eq!(
            found.code.to_string(),
            "content after the JSON value is ignored"
        );
        // The span is the content itself, in the file's bytes.
        let span = found.location.span.unwrap();
        assert_eq!(
            &text[span.offset as usize..][..span.len as usize],
            trailing.trim()
        );
        // It is not written back.
        let written = round_trip(&document, |text| attachable(text, &cx()));
        assert!(loaded(attachable(&written, &cx())).diagnostics().is_empty());
    }
    // White space and comments after the root are no content.
    let text = format!("{} \n // the end\n", ghost("1.26.0", ""));
    silent(loaded(attachable(&text, &cx())).diagnostics());
}

#[test]
fn content_after_the_root_does_not_change_which_payload_the_file_is() {
    // The peek that tells an attachable from a client entity reads the root as the load
    // does: a client entity followed by content is still a client entity.
    const BOTH: &str = r#"{"format_version": "1.10.0", "minecraft:client_entity": {"description": {"identifier": "example:ghost"}}, "minecraft:attachable": {"description": {"identifier": "example:sword"}}}"#;
    for trailing in ["", " }", " 5"] {
        let text = format!("{BOTH}{trailing}");
        let d = rejected(attachable(&text, &cx()));
        expect_at(
            &d,
            "core/ignored_payload",
            bedrock_addon::diagnostic::Severity::Error,
            "minecraft:attachable",
        );
        let set = stack(&[pack(&[("attachables/both.json", &text)])]);
        let attachables = set.resource.attachables();
        assert!(attachables.is_empty(), "{trailing:?}");
        assert!(attachables.rejected().is_empty(), "{trailing:?}");
        assert_eq!(
            attachables
                .diagnostics()
                .filter(|f| f.code.id() == "core/ignored_payload")
                .count(),
            1
        );
    }
    // And a version the gate refuses is refused with content after the root too.
    let text = format!("{} }}", ghost("0.9.0", ""));
    let d = rejected(attachable(&text, &cx()));
    assert!(
        find(
            &d,
            "visual/unknown_version",
            bedrock_addon::diagnostic::Severity::Error
        )
        .is_some()
    );
}
