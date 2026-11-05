//! `loading_messages.json`: reading the file, the highest pack's copy, the round trip and the
//! cost of a long list. The tips no language translates wait for the link stage; the game's
//! stage-list bug no file can cause.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::document::{Content, Document, Placed};
use bedrock_addon::families::loading_messages::{LoadingMessage, LoadingMessages, MessageList};
use bedrock_addon::load::LoadContext;
use bedrock_addon::storage::PackPath;

use super::common::{codes, json, located, pack_number, shown, stack_of};

const STAGES: &str = r#"{
  "beginner_loading_messages": ["b1", "b2"],
  "mid_game_loading_messages": ["m1"],
  "late_game_loading_messages": ["l1"],
  "realms_loading_messages": ["r1"],
  "addons_loading_messages": ["a1"],
  "creative_loading_messages": ["c1"],
  "editor_loading_messages": ["e1"]
}"#;

const BEGINNER_MISSING: &str =
    "Beginner messages are missing in ProgressTips::_parseLoadingMessages";

fn missing(key: &str) -> String {
    format!(
        "JSON passed to ProgressTips::_parseConditionalLoadingMessages does not contain messages of type: {key}"
    )
}

fn parse(text: &str) -> Document<LoadingMessages> {
    LoadingMessages::parse(
        &PackPath::from_static("loading_messages.json"),
        text.as_bytes(),
        &LoadContext::detached(),
    )
    .expect("a loading messages file")
}

fn texts(doc: &Document<LoadingMessages>) -> Vec<(&'static str, Severity, String)> {
    doc.diagnostics().iter().map(shown).collect()
}

#[test]
fn lm01_the_stage_lists() {
    let doc = parse(STAGES);
    assert_eq!(codes(doc.diagnostics()), []);
    assert_eq!(doc.tips(MessageList::Beginner), ["b1", "b2"]);
    assert_eq!(doc.tips(MessageList::MidGame), ["m1"]);
    assert_eq!(doc.tips(MessageList::LateGame), ["l1"]);
    assert!(!doc.uses_legacy_list());
    assert_eq!(
        doc.list(MessageList::Beginner).unwrap()[0],
        LoadingMessage::Text("b1".to_owned())
    );
}

#[test]
fn lm02_a_legacy_list_with_an_element_is_every_stage() {
    let doc = parse(
        r#"{ "loading_messages": ["a"], "beginner_loading_messages": ["b"],
             "mid_game_loading_messages": ["m"] }"#,
    );
    assert!(doc.uses_legacy_list());
    for stage in [
        MessageList::Beginner,
        MessageList::MidGame,
        MessageList::LateGame,
    ] {
        assert_eq!(doc.tips(stage), ["a"], "{stage:?}");
    }
    assert_eq!(
        doc.tips(MessageList::Realms),
        [] as [&str; 0],
        "a screen list is its own"
    );
    // Neither the stage lists nor the beginner list are asked for: only the four screens.
    assert_eq!(
        texts(&doc),
        ["realms", "addons", "creative", "editor"].map(|screen| (
            "MissingRequiredField",
            Severity::Info,
            missing(&format!("{screen}_loading_messages"))
        ))
    );
}

#[test]
fn lm03_an_empty_legacy_list_leaves_the_stage_lists_in_use() {
    let doc = parse(r#"{ "loading_messages": [], "beginner_loading_messages": ["b"] }"#);
    assert!(!doc.uses_legacy_list());
    assert_eq!(doc.tips(MessageList::Beginner), ["b"]);
    assert_eq!(doc.tips(MessageList::MidGame), [] as [&str; 0]);
    assert_eq!(
        texts(&doc).len(),
        6,
        "mid, late and the four screens are missing"
    );
}

#[test]
fn lm04_a_file_without_beginner_messages() {
    let doc = parse(r#"{ "mid_game_loading_messages": ["m"] }"#);
    let found = texts(&doc);
    assert_eq!(
        found[0],
        (
            "MissingRequiredField",
            Severity::Info,
            BEGINNER_MISSING.to_owned()
        )
    );
    assert_eq!(
        found[1..],
        ["late_game", "realms", "addons", "creative", "editor"].map(|list| (
            "MissingRequiredField",
            Severity::Info,
            missing(&format!("{list}_loading_messages"))
        ))
    );
}

#[test]
fn l3_an_empty_beginner_list() {
    let doc = parse(&STAGES.replace(r#"["b1", "b2"]"#, "[]"));
    assert_eq!(
        texts(&doc),
        [(
            "InvalidValue",
            Severity::Info,
            "ProgressTips::getProgressTipStage called before loading the tips!".to_owned()
        )]
    );
    assert_eq!(located(doc.diagnostics())[0].2, "beginner_loading_messages");
    // With the legacy list in use the beginner list is not read.
    let doc = parse(&STAGES.replace(r#"["b1", "b2"]"#, r#"[], "loading_messages": ["x"]"#));
    assert_eq!(codes(doc.diagnostics()), []);
}

#[test]
fn lm05_conditions_deviation() {
    let doc = parse(&STAGES.replace(
        r#"["m1"]"#,
        r#"[{ "message": "m1", "condition": "$marketplace_not_disabled" },
            { "message": "m2", "condition": "$other" }, { "condition": "$x" }, 7,
            { "message": "m5" }]"#,
    ));
    assert_eq!(doc.tips(MessageList::MidGame), ["m1", "m5"]);
    assert_eq!(
        located(doc.diagnostics()),
        [
            (
                "InvalidValue",
                Severity::Info,
                "mid_game_loading_messages/1/condition".to_owned()
            ),
            (
                "MissingRequiredField",
                Severity::Info,
                "mid_game_loading_messages/2/message".to_owned()
            ),
            (
                "InvalidValue",
                Severity::Info,
                "mid_game_loading_messages/3".to_owned()
            ),
        ]
    );
    let list = doc.list(MessageList::MidGame).unwrap();
    assert_eq!(list.len(), 5, "skipped elements are kept as written");
    assert_eq!(list[3], LoadingMessage::Other(json("7")));
    assert_eq!(list[1].shown(), None);
}

#[test]
fn message_and_condition_are_read_as_the_game_reads_strings() {
    let doc = parse(&STAGES.replace(
        r#"["m1"]"#,
        r#"[{ "message": true }, { "message": 4, "condition": null }, { "message": null }]"#,
    ));
    let LoadingMessage::Conditional(first) = &doc.list(MessageList::MidGame).unwrap()[0] else {
        panic!("an object with a message")
    };
    assert_eq!(
        (first.message.as_str(), first.condition.as_deref()),
        ("true", None)
    );
    // A null condition is the condition "", which the game does not know.
    assert_eq!(doc.tips(MessageList::MidGame), ["true", ""]);
    assert_eq!(
        located(doc.diagnostics()),
        [
            (
                "InvalidType",
                Severity::Info,
                "mid_game_loading_messages/0/message".to_owned()
            ),
            (
                "InvalidType",
                Severity::Info,
                "mid_game_loading_messages/1/message".to_owned()
            ),
            (
                "InvalidType",
                Severity::Info,
                "mid_game_loading_messages/1/condition".to_owned()
            ),
            (
                "InvalidValue",
                Severity::Info,
                "mid_game_loading_messages/1/condition".to_owned()
            ),
            (
                "InvalidType",
                Severity::Info,
                "mid_game_loading_messages/2/message".to_owned()
            ),
        ]
    );
}

#[test]
fn lm06_a_missing_screen_list() {
    let doc = parse(
        &STAGES
            .replace(r#""creative_loading_messages": ["c1"],"#, "")
            .replace(
                r#""editor_loading_messages": ["e1"]"#,
                r#""creative_loading_messages": ["c1"]"#,
            ),
    );
    assert_eq!(doc.tips(MessageList::Realms), ["r1"]);
    assert_eq!(doc.tips(MessageList::Addons), ["a1"]);
    assert_eq!(doc.tips(MessageList::Creative), ["c1"]);
    assert_eq!(doc.list(MessageList::Editor), None);
    assert_eq!(
        texts(&doc),
        [(
            "MissingRequiredField",
            Severity::Info,
            missing("editor_loading_messages")
        )]
    );
}

#[test]
fn l4_every_list_the_game_asks_for_is_named_when_missing() {
    for list in [
        MessageList::MidGame,
        MessageList::LateGame,
        MessageList::Realms,
        MessageList::Addons,
        MessageList::Creative,
        MessageList::Editor,
    ] {
        let mut root = json(STAGES);
        root.as_object_mut().unwrap().remove(list.key());
        let doc = parse(&facet_json::to_string(&root).unwrap());
        assert_eq!(
            texts(&doc),
            [("MissingRequiredField", Severity::Info, missing(list.key()))],
            "{list:?}"
        );
    }
}

#[test]
fn lm07_the_top_pack_s_file_is_read_whole() {
    let stack = stack_of(&[
        &[("loading_messages.json", STAGES)],
        &[(
            "loading_messages.json",
            r#"{ "store_progress_tooltips": ["t"] }"#,
        )],
    ]);
    let merged = stack.loading_messages();
    assert_eq!(merged.tooltips(), ["t"]);
    assert_eq!(merged.list(MessageList::Beginner), None);
    let found: Vec<_> = merged.diagnostics().collect();
    assert_eq!(shown(found[0]).2, BEGINNER_MISSING);
    assert_eq!(found.len(), 7);
    assert!(found.iter().all(|d| pack_number(d, 2) == Some(1)));
    assert_eq!(merged.layers().len(), 2);
    assert!(
        merged.layers()[1].list(MessageList::Beginner).is_some(),
        "top first"
    );
    let origin = merged
        .origin(&"store_progress_tooltips".to_owned())
        .unwrap();
    assert_eq!(origin.pack.index(), Some(0));
}

#[test]
fn three_packs_the_highest_file_wins_and_the_lower_ones_report_nothing() {
    let stack = stack_of(&[
        &[("loading_messages.json", "{ nope")],
        &[(
            "loading_messages.json",
            r#"{ "mid_game_loading_messages": [7] }"#,
        )],
        &[("loading_messages.json", STAGES)],
    ]);
    let merged = stack.loading_messages();
    assert_eq!(merged.tips(MessageList::Beginner), ["b1", "b2"]);
    assert_eq!(codes(merged.diagnostics()), []);
    assert_eq!(
        merged
            .origin(&"beginner_loading_messages".to_owned())
            .unwrap()
            .pack
            .index(),
        Some(0)
    );
}

#[test]
fn lm08_a_file_that_is_not_json() {
    let rejected = LoadingMessages::parse(
        &PackPath::from_static("loading_messages.json"),
        b"{ nope",
        &LoadContext::detached(),
    )
    .unwrap_err();
    let found: Vec<_> = rejected.diagnostics.iter().map(shown).collect();
    assert_eq!(
        found,
        [(
            "MalformedInput",
            Severity::Error,
            "Failed to read loading messages file".to_owned()
        )]
    );
}

#[test]
fn l1_a_broken_highest_copy_leaves_no_tips_and_the_lower_file_is_not_used() {
    for lower in [
        &[&[("loading_messages.json", STAGES)][..]][..],
        &[
            &[("loading_messages.json", STAGES)][..],
            &[("loading_messages.json", STAGES)][..],
        ][..],
    ] {
        let mut packs = lower.to_vec();
        packs.push(&[("loading_messages.json", "[")]);
        let stack = stack_of(&packs);
        let merged = stack.loading_messages();
        assert_eq!(merged.value(), &LoadingMessages::default());
        assert_eq!(
            codes(merged.diagnostics()),
            [("MalformedInput", Severity::Error)]
        );
        assert_eq!(merged.layers().len(), lower.len());
    }
}

#[test]
fn a_stack_without_the_file_has_no_tips_and_no_finding() {
    let stack = stack_of(&[&[("other.txt", "")]]);
    assert_eq!(
        stack.loading_messages().value(),
        &LoadingMessages::default()
    );
    assert_eq!(codes(stack.loading_messages().diagnostics()), []);
}

#[test]
fn l8_tooltips_are_converted() {
    let doc = parse(&STAGES.replace(
        r#""editor_loading_messages""#,
        r#""store_progress_tooltips": ["a", true, 3, null], "editor_loading_messages""#,
    ));
    assert_eq!(doc.tooltips(), ["a", "true", "", ""]);
    assert_eq!(
        located(doc.diagnostics()),
        [
            (
                "InvalidValue",
                Severity::Info,
                "store_progress_tooltips/1".to_owned()
            ),
            (
                "InvalidValue",
                Severity::Info,
                "store_progress_tooltips/2".to_owned()
            ),
            (
                "InvalidValue",
                Severity::Info,
                "store_progress_tooltips/3".to_owned()
            ),
        ]
    );
}

#[test]
fn a_list_that_is_no_array_is_iterated_as_the_game_does() {
    let doc = parse(
        &STAGES
            .replace(r#"["m1"]"#, r#"{ "z": "last", "a": "first" }"#)
            .replace(r#"["l1"]"#, "5"),
    );
    assert_eq!(doc.tips(MessageList::MidGame), ["first", "last"]);
    assert_eq!(
        doc.list(MessageList::LateGame),
        Some(&[][..]),
        "present, nothing iterated"
    );
    assert_eq!(
        located(doc.diagnostics()),
        [
            (
                "InvalidType",
                Severity::Info,
                "mid_game_loading_messages".to_owned()
            ),
            (
                "InvalidType",
                Severity::Info,
                "late_game_loading_messages".to_owned()
            ),
        ]
    );
    let doc = parse("[1]");
    assert_eq!(
        codes(doc.diagnostics())[0],
        ("InvalidType", Severity::Info),
        "a root that is no object"
    );
    // No list; the root is kept as written beside them.
    let kept = LoadingMessages {
        written: Some(json("[1]")).into(),
        ..LoadingMessages::default()
    };
    assert_eq!(doc.value(), &kept);
    assert_eq!(*doc.written, kept.written.0);
    assert_eq!(json(&doc.to_json().unwrap()), json("[1]"));
}

#[test]
fn unknown_members_are_kept_and_reported() {
    let text = STAGES.replace(
        r#""realms_loading_messages""#,
        r#""extra": [1], "note": "n", "realms_loading_messages""#,
    );
    let doc = parse(&text);
    assert_eq!(
        located(doc.diagnostics()),
        [
            ("UnrecognizedField", Severity::Info, "extra".to_owned()),
            ("UnrecognizedField", Severity::Info, "note".to_owned()),
        ]
    );
    assert_eq!(json(&doc.to_json().unwrap()), json(&text));
}

#[test]
fn a_member_written_twice_keeps_its_last_value() {
    let doc = parse(
        &STAGES
            .replace(
                r#""realms_loading_messages""#,
                r#""beginner_loading_messages": ["first"], "realms_loading_messages""#,
            )
            .replace("\"b1\", \"b2\"", "\"last\""),
    );
    // Whichever comes last in the text stands.
    let last = if STAGES.find("beginner").unwrap() < STAGES.find("realms").unwrap() {
        "first"
    } else {
        "last"
    };
    assert_eq!(doc.tips(MessageList::Beginner), [last]);
}

#[test]
fn loading_messages_are_placed_at_the_pack_root() {
    assert_eq!(
        LoadingMessages::default().default_path().as_str(),
        "loading_messages.json"
    );
    assert_eq!(LoadingMessages::default().to_json().unwrap().trim(), "{}");
}

/// A few thousand generated files: each loads, writes text that loads back to an equal value,
/// and writes the same text again.
#[test]
fn generated_files_round_trip() {
    let elements = [
        "\"t\"",
        "7",
        "null",
        "{}",
        "[]",
        "true",
        "{\"message\": \"m\"}",
        "{\"message\": \"m\", \"condition\": \"$marketplace_not_disabled\"}",
        "{\"message\": 3, \"condition\": \"$x\", \"k\": [1]}",
        "{\"condition\": \"$x\"}",
        "{\"message\": null, \"condition\": false}",
    ];
    let lists: Vec<String> = ["null", "5", "\"s\"", "[]", "{\"k\": \"v\", \"a\": 1}"]
        .iter()
        .map(|s| (*s).to_owned())
        .chain(elements.iter().map(|e| format!("[{e}]")))
        .chain(elements.iter().map(|e| format!("[\"x\", {e}, {e}]")))
        .collect();
    let mut count = 0;
    for key in MessageList::ALL
        .map(MessageList::key)
        .into_iter()
        .chain(["store_progress_tooltips"])
    {
        for list in &lists {
            for other in [
                None,
                Some("loading_messages"),
                Some("beginner_loading_messages"),
                Some("zz"),
            ] {
                for other_list in ["[]", "[\"o\"]", "3"] {
                    let mut members = vec![format!("\"{key}\": {list}")];
                    if let Some(other) = other.filter(|other| *other != key) {
                        members.push(format!("\"{other}\": {other_list}"));
                    }
                    let text = format!("{{{}}}", members.join(", "));
                    let first = parse(&text);
                    let written = first.to_json().unwrap();
                    // As authored: no generated document is rewritten.
                    assert_eq!(json(&written), json(&text), "{text}");
                    let second = parse(&written);
                    assert_eq!(first.value(), second.value(), "{text}");
                    assert_eq!(second.to_json().unwrap(), written, "{text}");
                    for list in MessageList::ALL {
                        assert_eq!(first.tips(list), second.tips(list), "{text}");
                    }
                    assert_eq!(first.tooltips(), second.tooltips(), "{text}");
                    count += 1;
                }
            }
        }
    }
    assert!(count >= 2000, "{count} documents");
    println!("{count} generated documents written as authored");
}
