//! what the client's
//! loader does with a file before its struct is read.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::document::Content;
use bedrock_addon::families::ddui::{FORMAT_VERSION, UiComposition, UiRoot};
use bedrock_addon::load::LoadContext;
use bedrock_addon::storage::PackPath;

use super::common::*;

const ROOT_KEY: &str = "minecraft:ui-root";
const COMPOSITION_KEY: &str = "minecraft:ui-composition";
const MEMBER_COUNT: &str =
    "A Minecraft Document must have exactly two members: \"format_version\" and a payload member";

/// `R` with `version` (a JSON value) as its `format_version`.
fn versioned(version: &str) -> String {
    r().replace(r#""1.21.130""#, version)
}

fn no_parser(version: &str) -> Vec<Found> {
    vec![
        error(
            "UnsupportedVersion",
            "format_version",
            &format!("unsupported version: no parser available for version {version}"),
        ),
        failed_to_load(),
    ]
}

fn no_valid_version() -> Vec<Found> {
    vec![
        error(
            "UnsupportedVersion",
            "format_version",
            "no valid version passed by the caller or found in the input",
        ),
        failed_to_load(),
    ]
}

#[test]
fn d03_a_composition_payload_in_the_root_folder_is_a_missing_object() {
    let text = r().replace(ROOT_KEY, COMPOSITION_KEY);
    assert_eq!(found(&rejects(root(&text))), [missing_object(ROOT_KEY)]);
}

#[test]
fn d04_a_root_in_the_composition_folder_is_a_missing_object() {
    assert_eq!(
        found(&rejects(composition(&r()))),
        [missing_object(COMPOSITION_KEY)]
    );
}

#[test]
fn d05_truncated_json_is_a_missing_object_with_the_reader_s_text_as_a_note() {
    let text = r();
    let d = rejects(root(&text[..text.len() - 20]));
    let got = found(&d);
    assert_eq!(got.len(), 2, "{got:#?}");
    assert_eq!(got[0].0, Severity::Info);
    assert_eq!(got[0].1, "MalformedInput");
    assert!(
        got[0]
            .3
            .starts_with("Failed to parse the Minecraft Document JSON: "),
        "{}",
        got[0].3
    );
    assert_eq!(got[1], missing_object(ROOT_KEY));
}

#[test]
fn d06_a_third_root_member_is_a_missing_object_with_the_member_count_as_a_note() {
    let text = r().replacen('{', r#"{ "extra": 1, "#, 1);
    assert_eq!(
        found(&rejects(root(&text))),
        [
            info("MalformedInput", "", MEMBER_COUNT),
            missing_object(ROOT_KEY)
        ]
    );
}

/// The envelope reader's verdict comes first: with three members the payload key is never
/// looked at, so the note is there whichever key is wrong.
#[test]
fn d06_three_members_without_the_payload_key_still_carry_the_note() {
    let text = r()
        .replace(ROOT_KEY, COMPOSITION_KEY)
        .replacen('{', r#"{ "extra": 1, "#, 1);
    assert_eq!(
        found(&rejects(root(&text))),
        [
            info("MalformedInput", "", MEMBER_COUNT),
            missing_object(ROOT_KEY)
        ]
    );
}

#[test]
fn an_empty_file_and_null_are_missing_objects() {
    for text in ["", "  \n", "null", "// nothing\n"] {
        let got = found(&rejects(root(text)));
        assert_eq!(got.len(), 2, "{text:?}: {got:#?}");
        assert_eq!((got[0].0, got[0].1), (Severity::Info, "EmptyDocument"));
        assert_eq!(got[1], missing_object(ROOT_KEY));
    }
}

#[test]
fn a_root_that_is_no_object_is_a_missing_object() {
    for text in ["[]", "5", r#""text""#] {
        let got = found(&rejects(root(text)));
        assert_eq!(got.len(), 2, "{text:?}: {got:#?}");
        assert_eq!((got[0].0, got[0].1), (Severity::Info, "MalformedInput"));
        assert_eq!(got[1], missing_object(ROOT_KEY));
    }
}

#[test]
fn bytes_that_are_no_text_are_a_missing_object() {
    let path = PackPath::from_static("ddui/root/test.json");
    let rejected = UiRoot::parse(&path, &[0xff, 0xfe, b'{', b'}'], &LoadContext::detached())
        .expect_err("not UTF-8");
    let got = found(&rejected.diagnostics);
    assert_eq!((got[0].0, got[0].1), (Severity::Info, "MalformedInput"));
    assert_eq!(got[1], missing_object(ROOT_KEY));
    assert_eq!(got.len(), 2);
    // Nested past the reader's limit.
    let deep = format!("{}{}", "[".repeat(200), "]".repeat(200));
    let rejected =
        UiComposition::parse(&path, deep.as_bytes(), &LoadContext::detached()).expect_err("deep");
    assert_eq!(
        found(&rejected.diagnostics)[1],
        missing_object(COMPOSITION_KEY)
    );
}

#[test]
fn f01_the_exact_version_loads() {
    let document = loads(root(&r()));
    assert_eq!(document.description.identifier.as_str(), "demo:screen");
    assert_eq!(document.attribs.root_point, "minecraft:screen");
    assert_eq!(document.layout.markup.len(), 1);
    assert_eq!(document.version().written, Some(FORMAT_VERSION));
    assert_eq!(document.version().parsed_as, FORMAT_VERSION);
    assert!(!document.version().upgraded);
    assert!(document.diagnostics().is_empty());
}

#[test]
fn f02_a_version_below_has_no_parser() {
    assert_eq!(
        found(&rejects(root(&versioned(r#""1.21.120""#)))),
        no_parser("1.21.120")
    );
    // The boundary: the last version below.
    assert_eq!(
        found(&rejects(root(&versioned(r#""1.21.129""#)))),
        no_parser("1.21.129")
    );
}

/// The ceiling is the struct's version, not the engine's.
#[test]
fn f03_a_version_above_has_no_parser() {
    assert_eq!(
        found(&rejects(root(&versioned(r#""1.26.20""#)))),
        no_parser("1.26.20")
    );
    assert_eq!(
        found(&rejects(root(&versioned(r#""1.21.131""#)))),
        no_parser("1.21.131")
    );
    assert_eq!(
        found(&rejects(composition(
            &composition_json("demo:c", "", r#"{ "component": "Form.Divider" }"#)
                .replace(r#""1.21.130""#, r#""1.21.131""#)
        ))),
        no_parser("1.21.131")
    );
}

/// Not the warn-and-read-at-the-floor default: the envelope then has one member.
#[test]
fn f04_a_missing_version_is_a_missing_object_with_the_member_count_as_a_note() {
    let text = r().replace(r#""format_version": "1.21.130", "#, "");
    assert!(!text.contains("format_version"));
    assert_eq!(
        found(&rejects(root(&text))),
        [
            info("MalformedInput", "", MEMBER_COUNT),
            missing_object(ROOT_KEY)
        ]
    );
}

#[test]
fn f05_beta_has_no_parser_whoever_asks() {
    let text = versioned(r#""beta""#);
    assert_eq!(found(&rejects(root(&text))), no_parser("beta"));
    // Neither the Beta APIs toggle nor a pack the game ships changes it: no beta slice.
    let mut beta = cx(E20, false);
    beta.beta_apis = true;
    assert_eq!(found(&rejects(root_in(&text, &beta))), no_parser("beta"));
    let text = text.replace("demo:screen", "minecraft:screen");
    assert_eq!(
        found(&rejects(root_in(&text, &cx(E20, true)))),
        no_parser("beta")
    );
}

#[test]
fn f06_a_pre_release_of_the_version_sorts_below_it() {
    assert_eq!(
        found(&rejects(root(&versioned(r#""1.21.130-beta.1""#)))),
        no_parser("1.21.130-beta.1")
    );
}

#[test]
fn f07_a_version_string_that_is_no_version_is_a_missing_object() {
    assert_eq!(
        found(&rejects(root(&versioned(r#""abc""#)))),
        [
            info(
                "MalformedFormatVersion",
                "format_version",
                "Malformed format version"
            ),
            missing_object(ROOT_KEY)
        ]
    );
}

/// The version as the game's document input reads it: a number is read as its text (`1` is 1.0.0, `1.21` is 1.21.0) and has no parser;
/// three numbers are the version; a `null` gives none and is "Malformed format version".
#[test]
fn f08_a_version_that_is_no_string_is_read_as_the_document_input_reads_it() {
    assert_eq!(found(&rejects(root(&versioned("1")))), no_parser("1.0.0"));
    assert_eq!(
        found(&rejects(root(&versioned("1.21")))),
        no_parser("1.21.0")
    );
    let document = loads(root(&versioned("[1, 21, 130]")));
    assert_eq!(document.version().written, Some(FORMAT_VERSION));
    assert!(document.diagnostics().is_empty());
    assert_eq!(
        found(&rejects(root(&versioned("null")))),
        [
            info(
                "MalformedFormatVersion",
                "format_version",
                "Malformed format version"
            ),
            missing_object(ROOT_KEY)
        ]
    );
}

/// Two members, the payload among them, and no `format_version`: the load has no version.
#[test]
fn f08_two_members_without_a_version_member_reach_the_load_without_a_version() {
    let text = r().replace(r#""format_version": "1.21.130""#, r#""other": "1.21.130""#);
    assert_eq!(found(&rejects(root(&text))), no_valid_version());
    // Without the payload key the loader stops before the load.
    let text = text.replace(ROOT_KEY, COMPOSITION_KEY);
    assert_eq!(found(&rejects(root(&text))), [missing_object(ROOT_KEY)]);
}

/// Of a member written twice the last stands: the envelope still has two members.
#[test]
fn a_root_member_written_twice_counts_once_and_the_last_stands() {
    let text = r().replacen('{', r#"{ "format_version": "1.0.0", "#, 1);
    let document = loads(root(&text));
    assert_eq!(document.version().written, Some(FORMAT_VERSION));
    let text = r().replacen(
        '{',
        r#"{ "minecraft:ui-root": { "description": { "identifier": "demo:first" } }, "#,
        1,
    );
    assert_eq!(
        loads(root(&text)).description.identifier.as_str(),
        "demo:screen"
    );
}

#[test]
fn parse_as_written_gives_the_one_struct() {
    use bedrock_addon::document::JsonDocument;
    use bedrock_addon::versioning::Single;
    let path = PackPath::from_static("ddui/root/test.json");
    let cx = cx(E20, false);
    let written = UiRoot::parse_as_written(&path, r().as_bytes(), &cx).expect("parses");
    let Single(value) = written.value();
    assert_eq!(value, loads(root(&r())).value());
    let rejected = UiRoot::parse_as_written(&path, versioned(r#""1.26.20""#).as_bytes(), &cx)
        .expect_err("no parser");
    assert_eq!(found(&rejected.diagnostics), no_parser("1.26.20"));
    let rejected = UiRoot::parse_as_written(&path, b"{", &cx).expect_err("no JSON");
    assert_eq!(found(&rejected.diagnostics)[1], missing_object(ROOT_KEY));
}

/// Content after the root value refuses the document on every path: the data-driven UI files
/// are read through the game's document input, whose JSON parse stops at it, before the
/// envelope stage looks at the file.
#[test]
fn content_after_the_root_value_refuses_the_file_on_every_path() {
    let refusal = error(
        "MalformedInput",
        "",
        "Failed to parse the Minecraft Document JSON: The document root must not be followed by other values.",
    );
    let text = format!("{} trailing", r());
    let refused = rejects(root(&text));
    assert_eq!(found(&refused), [refusal.clone(), failed_to_load()]);
    assert_eq!(spanned(&text, refused.iter().next().unwrap()), "trailing");
    // Refused by the envelope stage (three members, a malformed version, the wrong payload
    // key, no usable version): the content after the root is told once, and the file is
    // refused.
    let three = r().replacen('{', r#"{ "extra": 1, "#, 1);
    let cases = [
        three,
        r().replace(r#""1.21.130""#, r#""abc""#),
        r().replace("minecraft:ui-root", "minecraft:ui-composition"),
        r().replace(r#""1.21.130""#, "1"),
        r().replace(r#""1.21.130""#, r#""1.26.20""#),
    ];
    for text in cases {
        let text = format!("{text} trailing");
        let got = found(&rejects(root(&text)));
        let told = got
            .iter()
            .filter(|f| {
                f.3.ends_with("The document root must not be followed by other values.")
                    || f.3 == "content after the JSON value is ignored"
            })
            .count();
        assert_eq!(told, 1, "{text}: {got:#?}");
    }
}
