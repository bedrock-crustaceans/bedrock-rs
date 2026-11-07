//! The validation rules and the upgrade steps that no fixture row covers on
//! its own, and both ends of every rule's range.

use bedrock_addon::diagnostic::{Code, Diagnostics, JsonPath, Severity, codes};
use bedrock_addon::families::dialogue::codes as dialogue_codes;
use bedrock_addon::families::dialogue::{ActionMode, DialogueScene, DialogueText, NpcAction};
use bedrock_addon::id::DialogueSceneTag;

use crate::common::*;

/// A file of version 1.14.0 with these scenes.
fn file(scenes: &str) -> String {
    format!(r#"{{"format_version": "1.14.0", "minecraft:npc_dialogue": {{"scenes": [{scenes}]}}}}"#)
}

/// The texts of the error-level findings.
fn errors(diagnostics: &Diagnostics) -> Vec<String> {
    diagnostics
        .at_least(Severity::Error)
        .map(|d| d.code.to_string())
        .collect()
}

// ---- file level -----------------------------------------------------------------------------

#[test]
fn a_root_that_is_not_an_object_skips_the_file() {
    load_fixture("v_root_not_object");
    let diagnostics = rejected("[]");
    assert!(
        diagnostics
            .first_error()
            .is_some_and(|d| d.code.is(&dialogue_codes::ROOT_NOT_OBJECT))
    );
}

#[test]
fn a_missing_or_null_payload_skips_the_file() {
    load_fixture("v_missing_payload");
    let diagnostics = rejected(r#"{"format_version": "1.14.0"}"#);
    assert!(
        diagnostics
            .first_error()
            .is_some_and(|d| d.code.is(&dialogue_codes::NO_PAYLOAD))
    );
}

#[test]
fn file_rules_are_checked_in_vanilla_s_order() {
    // The version before the payload, the payload before `scenes`; one text each.
    let no_version = rejected("{}");
    assert_eq!(
        errors(&no_version),
        ["JSON: a.json has an error: \n Couldn't find 'format_version' property."]
    );
    let low_version = rejected(r#"{"format_version": "1.13.9"}"#);
    assert_eq!(
        errors(&low_version),
        [
            "JSON: a.json has an error: \n 'format_version' was out of range of expected values. Must be greater than 1.14.0"
        ]
    );
    let no_payload = rejected(r#"{"format_version": "1.14.0"}"#);
    assert_eq!(
        errors(&no_payload),
        ["JSON: a.json has an error: \n Couldn't find 'minecraft:npc_dialogue' object."]
    );
}

#[test]
fn an_empty_file_is_a_syntax_error() {
    for text in ["", "   \n", "// only a comment\n"] {
        let diagnostics = rejected(text);
        let Some(finding) = diagnostics.first_error() else {
            panic!("{text:?}: {diagnostics:#?}");
        };
        let message = finding.syntax_message().expect("the reader's message");
        assert_eq!(
            finding.code.args_of(&dialogue_codes::READER),
            Some(&["a".to_owned(), message.to_owned()][..]),
            "{text:?}"
        );
    }
}

// ---- scene level ----------------------------------------------------------------------------

/// The one loaded scene of a one-scene file.
#[track_caller]
fn one_scene(scene: &str) -> (DialogueScene, Diagnostics) {
    let document = loads(&file(scene));
    assert_eq!(document.scenes.len(), 1, "{:#?}", document.diagnostics());
    (document.scenes[0].clone(), document.diagnostics().clone())
}

#[test]
fn a_missing_or_null_text_drops_the_scene() {
    let stack = load_fixture("v_missing_text");
    let rejected = stack.dialogue().rejected();
    assert_eq!(rejected[0].origin.key.as_deref(), Some("absent"));
    assert_eq!(rejected[1].origin.key.as_deref(), Some("null"));
}

#[test]
fn an_absent_or_null_scene_tag_is_a_missing_field_and_another_type_an_invalid_type() {
    for (scene, missing) in [
        (r#"{"text": "x"}"#, true),
        (r#"{"scene_tag": null, "text": "x"}"#, true),
        (r#"{"scene_tag": true, "text": "x"}"#, false),
        (r#"{"scene_tag": ["a"], "text": "x"}"#, false),
        (r#"{"scene_tag": {"a": 1}, "text": "x"}"#, false),
    ] {
        let stack = stack(vec![pack(
            10,
            vec![("dialogue/a.json".to_owned(), file(scene).into_bytes())],
        )]);
        let registry = stack.dialogue();
        assert!(registry.is_empty(), "{scene}");
        let error = registry.diagnostics().next().expect("the tag rule");
        assert_eq!(error.severity, Severity::Error, "{scene}");
        assert_eq!(
            error
                .code
                .is(&dialogue_codes::NO_SCENE_TAG_MISSING_REQUIRED_FIELD),
            missing,
            "{scene}: {:?}",
            error.code
        );
        assert_eq!(
            error.code.is(&dialogue_codes::NO_SCENE_TAG_INVALID_TYPE),
            !missing,
            "{scene}: {:?}",
            error.code
        );
    }
}

#[test]
fn npc_name_is_a_string_or_any_object_and_null_is_absent() {
    assert_eq!(
        one_scene(r#"{"scene_tag": "a", "text": "x"}"#).0.npc_name,
        None
    );
    assert_eq!(
        one_scene(r#"{"scene_tag": "a", "npc_name": null, "text": "x"}"#)
            .0
            .npc_name,
        None
    );
    assert_eq!(
        one_scene(r#"{"scene_tag": "a", "npc_name": "", "text": "x"}"#)
            .0
            .npc_name,
        Some(DialogueText::Plain(String::new()))
    );
    assert!(matches!(
        one_scene(r#"{"scene_tag": "a", "npc_name": {}, "text": "x"}"#)
            .0
            .npc_name,
        Some(DialogueText::Raw(_))
    ));
    for name in ["5", "1.5", "true", "false", "[]", r#"["a"]"#] {
        let document = loads(&file(&format!(
            r#"{{"scene_tag": "a", "npc_name": {name}, "text": "x"}}"#
        )));
        assert!(document.scenes.is_empty(), "{name}");
        assert_eq!(
            document
                .diagnostics()
                .iter()
                .map(|d| d.code.to_string())
                .collect::<Vec<_>>(),
            [
                "JSON: a.json has an error: \n The 'name' of an NPC must be a string or rawtext object. Skipping scene..."
            ],
            "{name}"
        );
    }
}

#[test]
fn a_scene_with_an_empty_tag_still_runs_the_later_rules() {
    // The scene parser accepts the empty tag; `parseFile` drops the scene afterwards, so a
    // missing text is still reported.
    let document = loads(&file(r#"{"scene_tag": ""}"#));
    assert!(document.scenes.is_empty());
    assert_eq!(
        document
            .diagnostics()
            .iter()
            .map(|d| d.code.to_string())
            .collect::<Vec<_>>(),
        [
            "JSON: a.json has an error: \n Each scene needs a 'text' value to display to the reader, was not found. Skipping scene..."
        ]
    );
    // With a text it parses and is dropped without a word.
    let document = loads(&file(r#"{"scene_tag": "", "text": "x"}"#));
    assert!(document.scenes.is_empty());
    assert!(document.diagnostics().is_empty());
}

#[test]
fn text_forms_are_read_as_vanilla_reads_them() {
    let text = |value: &str| one_scene(&format!(r#"{{"scene_tag": "a", "text": {value}}}"#));
    let (scene, diagnostics) = text(r#""plain""#);
    assert_eq!(scene.text, DialogueText::Plain("plain".to_owned()));
    assert!(diagnostics.is_empty());
    let (scene, diagnostics) = text(r#""""#);
    assert_eq!(scene.text, DialogueText::Plain(String::new()));
    assert!(diagnostics.is_empty());
    let (scene, diagnostics) = text(r#"{"anything": [1, 2]}"#);
    assert_eq!(scene.text.stored(), "{\"anything\":[1,2]}\n");
    assert!(diagnostics.is_empty());
    for (value, coerced) in [
        ("true", "true"),
        ("false", "false"),
        ("0", ""),
        ("1.5", ""),
        (r#"["a"]"#, ""),
    ] {
        let (scene, diagnostics) = text(value);
        assert_eq!(
            scene.text,
            DialogueText::Plain(coerced.to_owned()),
            "{value}"
        );
        // ours, an information
        assert_eq!(
            names(&diagnostics, Severity::Info),
            ["InvalidType"],
            "{value}"
        );
    }
}

#[test]
fn the_stored_text_of_an_object_has_its_members_in_key_order() {
    // `Json::Value` keeps object members in a sorted map, so `FastWriter` writes them by key;
    // nested objects too, arrays in their own order.
    let (scene, _) = one_scene(
        r#"{"scene_tag": "a", "text": {"with": ["b", "a"], "translate": "k", "nested": {"z": 1, "a": {"y": true, "x": null}}}}"#,
    );
    assert_eq!(
        scene.text.stored(),
        "{\"nested\":{\"a\":{\"x\":null,\"y\":true},\"z\":1},\"translate\":\"k\",\"with\":[\"b\",\"a\"]}\n"
    );
    // The value itself keeps the author's order for write-back.
    let DialogueText::Raw(value) = &scene.text else {
        panic!("{:?}", scene.text);
    };
    let keys: Vec<&str> = value
        .as_object()
        .unwrap()
        .keys()
        .map(facet_value::VString::as_str)
        .collect();
    assert_eq!(keys, ["with", "translate", "nested"]);
}

/// the scene is the registry's document, so a rule that drops it is an error there and
/// the scene is listed in `rejected()`; read as a file, the file loads without the scene and
/// the same text is a kept error (`NpcDialogueStorage::parseFile` skips the
/// scene and goes on).
#[test]
fn a_dropped_scene_is_an_error_in_the_registry_and_a_kept_error_in_the_file() {
    let text = file(r#"{"scene_tag": "a"}, {"scene_tag": "b", "text": "kept"}"#);
    let document = loads(&text);
    assert_eq!(document.scenes.len(), 1);
    assert!(!document.diagnostics().has_errors());
    let kept: Vec<_> = document
        .diagnostics()
        .iter()
        .filter(|d| d.severity == Severity::Error && d.kept)
        .map(|d| d.code.name())
        .collect();
    assert_eq!(kept, ["MissingRequiredField"]);
    let stack = stack(vec![pack(
        10,
        vec![("dialogue/a.json".to_owned(), text.into_bytes())],
    )]);
    let registry = stack.dialogue();
    assert_eq!(registry.len(), 1);
    assert_eq!(registry.rejected().len(), 1);
    assert!(registry.rejected()[0].diagnostics.has_errors());
    let errors: Vec<_> = registry
        .diagnostics()
        .filter(|d| d.severity == Severity::Error)
        .map(|d| d.code.name())
        .collect();
    assert_eq!(errors, ["MissingRequiredField"]);
}

// ---- buttons and commands -------------------------------------------------------------------

/// The texts of the kept errors (the game's lines after which the scene loads).
fn warnings(diagnostics: &Diagnostics) -> Vec<String> {
    diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error && d.kept)
        .map(|d| d.code.to_string())
        .collect()
}

/// A scene `a` with these buttons.
fn with_buttons(buttons: &str) -> (DialogueScene, Diagnostics) {
    one_scene(&format!(
        r#"{{"scene_tag": "a", "text": "x", "buttons": [{buttons}]}}"#
    ))
}

const NO_NAME: &str = "JSON: a.json has an error: \n Each button needs a 'name' field, was not found. Skipping button...";
const BOTH: &str = "JSON: a.json has an error: \n Each button supports either a 'commands' array or a URL to give it functionality, not both. Skipping button...";

#[test]
fn a_non_string_command_drops_the_whole_button() {
    let stack = load_fixture("v_button_command_type");
    let typed = scene(stack.dialogue(), "typed");
    let kept = typed.buttons.as_deref().unwrap();
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].name, DialogueText::Plain("Fine".to_owned()));
}

#[test]
fn a_button_name_that_is_absent_or_null_is_missing_and_another_type_invalid() {
    for (button, missing) in [
        (r"{}", true),
        (r#"{"name": null}"#, true),
        (r"5", true),
        (r#"{"name": true}"#, false),
        (r#"{"name": 1.5}"#, false),
        (r#"{"name": ["a"]}"#, false),
    ] {
        let (scene, diagnostics) = with_buttons(button);
        assert_eq!(scene.buttons.as_deref(), Some(&[][..]), "{button}");
        assert_eq!(warnings(&diagnostics), [NO_NAME], "{button}");
        let code = &diagnostics.iter().next().unwrap().code;
        assert_eq!(
            // By legacy name: the variant and the definitions that were it.
            matches!(code.name(), "MissingRequiredField"),
            missing,
            "{button}: {code:?}"
        );
        assert_eq!(
            // By legacy name: the variant and the definitions that were it.
            matches!(code.name(), "InvalidType"),
            !missing,
            "{button}: {code:?}"
        );
    }
}

#[test]
fn commands_and_url_exclude_each_other_whatever_their_types() {
    // Both non-null: dropped before either type is looked at.
    for button in [
        r#"{"name": "b", "commands": [], "url": ""}"#,
        r#"{"name": "b", "commands": "x", "url": 5}"#,
        r#"{"name": "b", "commands": 1, "url": "https://x"}"#,
    ] {
        let (scene, diagnostics) = with_buttons(button);
        assert_eq!(scene.buttons.as_deref(), Some(&[][..]), "{button}");
        assert_eq!(warnings(&diagnostics), [BOTH], "{button}");
    }
    // A null member is absent.
    for button in [
        r#"{"name": "b", "commands": null, "url": "https://x"}"#,
        r#"{"name": "b", "commands": ["/say"], "url": null}"#,
    ] {
        let (scene, diagnostics) = with_buttons(button);
        assert_eq!(
            scene.buttons.as_deref().map(<[_]>::len),
            Some(1),
            "{button}"
        );
        assert!(diagnostics.is_empty(), "{button}");
    }
    // The name is checked first.
    let (_, diagnostics) = with_buttons(r#"{"commands": [], "url": ""}"#);
    assert_eq!(warnings(&diagnostics), [NO_NAME]);
}

#[test]
fn the_both_check_comes_before_the_command_and_url_types_and_commands_before_the_url() {
    let (_, diagnostics) = with_buttons(r#"{"name": "b", "commands": [5]}"#);
    assert_eq!(
        warnings(&diagnostics),
        [
            "JSON: a.json has an error: \n Each command needs to be a string type. Skipping button..."
        ]
    );
    let (_, diagnostics) = with_buttons(r#"{"name": "b", "url": []}"#);
    assert_eq!(
        warnings(&diagnostics),
        ["JSON: a.json has an error: \n The URL needs to be a string type. Skipping button..."]
    );
    // An empty name is looked at only after the button parsed: its other rules still speak.
    let (scene, diagnostics) = with_buttons(r#"{"name": "", "url": 5}"#);
    assert_eq!(scene.buttons.as_deref(), Some(&[][..]));
    assert_eq!(
        warnings(&diagnostics),
        ["JSON: a.json has an error: \n The URL needs to be a string type. Skipping button..."]
    );
}

#[test]
fn buttons_that_are_not_an_array_are_ignored_silently() {
    for buttons in [r#""x""#, "5", "{}", "null", "true"] {
        let (scene, diagnostics) = one_scene(&format!(
            r#"{{"scene_tag": "a", "text": "x", "buttons": {buttons}}}"#
        ));
        assert_eq!(scene.buttons, None, "{buttons}");
        assert!(diagnostics.is_empty(), "{buttons}");
    }
    let (scene, _) = with_buttons("");
    assert_eq!(scene.buttons, Some(vec![]));
}

#[test]
fn six_valid_buttons_load_and_seven_drop_the_scene() {
    let buttons = |n: usize| {
        (0..n)
            .map(|i| format!(r#"{{"name": "b{i}"}}"#))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let (scene, diagnostics) = with_buttons(&buttons(6));
    assert_eq!(scene.buttons.as_deref().map(<[_]>::len), Some(6));
    assert!(diagnostics.is_empty());
    let document = loads(&file(&format!(
        r#"{{"scene_tag": "a", "text": "x", "buttons": [{}]}}"#,
        buttons(7)
    )));
    assert!(document.scenes.is_empty());
    assert_eq!(
        warnings(document.diagnostics()),
        [
            "JSON: a.json has an error: \n Only a maximum number of 6 buttons are currently supported. Skipping scene..."
        ]
    );
    // Silently dropped empty names do not count either.
    let seven_with_an_empty = format!(r#"{}, {{"name": ""}}"#, buttons(6));
    let (scene, diagnostics) = with_buttons(&seven_with_an_empty);
    assert_eq!(scene.buttons.as_deref().map(<[_]>::len), Some(6));
    assert!(diagnostics.is_empty());
}

#[test]
fn the_button_count_is_checked_before_the_open_and_close_commands() {
    let buttons = (0..7)
        .map(|i| format!(r#"{{"name": "b{i}"}}"#))
        .collect::<Vec<_>>()
        .join(", ");
    let document = loads(&file(&format!(
        r#"{{"scene_tag": "a", "text": "x", "buttons": [{buttons}], "on_open_commands": [5]}}"#
    )));
    assert_eq!(document.diagnostics().len(), 1);
}

#[test]
fn a_non_string_close_command_is_skipped_with_its_own_key_in_the_text() {
    let (scene, diagnostics) = one_scene(
        r#"{"scene_tag": "a", "text": "x", "on_open_commands": [true, "/open"], "on_close_commands": ["/close", {}, []]}"#,
    );
    assert_eq!(
        warnings(&diagnostics),
        [
            "JSON: a.json has an error: \n Each command needs to be a string type. Skipping this on_open_command command...",
            "JSON: a.json has an error: \n Each command needs to be a string type. Skipping this on_close_command command...",
            "JSON: a.json has an error: \n Each command needs to be a string type. Skipping this on_close_command command...",
        ]
    );
    assert_eq!(scene.on_open_commands.as_deref().map(<[_]>::len), Some(1));
    assert_eq!(scene.on_close_commands.as_deref().map(<[_]>::len), Some(1));
    for list in ["5", r#""say""#, "{}", "null"] {
        let (scene, diagnostics) = one_scene(&format!(
            r#"{{"scene_tag": "a", "text": "x", "on_open_commands": {list}, "on_close_commands": {list}}}"#
        ));
        assert_eq!(scene.on_open_commands, None, "{list}");
        assert_eq!(scene.on_close_commands, None, "{list}");
        assert!(diagnostics.is_empty(), "{list}");
    }
}

#[test]
fn actions_are_the_buttons_then_the_open_then_the_close_commands() {
    let (scene, _) = one_scene(
        r#"{"scene_tag": "a", "text": "x",
            "on_close_commands": ["/close"], "on_open_commands": ["/open 1", "/open 2"],
            "buttons": [{"name": "Go", "commands": ["/go"]}, {"name": "Web", "url": "https://x"}]}"#,
    );
    let actions = scene.actions();
    let shape: Vec<(Option<ActionMode>, String, String)> = actions
        .iter()
        .map(|action| match action {
            NpcAction::Command {
                mode, button_name, ..
            } => (Some(*mode), button_name.stored(), action.text()),
            NpcAction::Url { button_name, .. } => (None, button_name.stored(), action.text()),
            _ => unreachable!(),
        })
        .collect();
    assert_eq!(
        shape,
        [
            (
                Some(ActionMode::Button),
                "Go".to_owned(),
                "/go\n".to_owned()
            ),
            (None, "Web".to_owned(), "https://x".to_owned()),
            (
                Some(ActionMode::OnOpen),
                String::new(),
                "/open 1\n/open 2\n".to_owned()
            ),
            (
                Some(ActionMode::OnClose),
                String::new(),
                "/close\n".to_owned()
            ),
        ]
    );
    assert_eq!(ActionMode::Button as u8, 0);
    assert_eq!(ActionMode::OnClose as u8, 1);
    assert_eq!(ActionMode::OnOpen as u8, 2);
}

#[test]
fn a_leading_slash_is_kept_and_inner_whitespace_too() {
    let (scene, _) =
        with_buttons(r#"{"name": "b", "commands": ["/say  a b ", "say c", "\r\n\t "]}"#);
    let raw: Vec<&str> = scene.buttons.as_deref().unwrap()[0]
        .commands
        .as_deref()
        .unwrap()
        .iter()
        .map(|c| c.raw.as_str())
        .collect();
    assert_eq!(raw, ["/say  a b", "say c"]);
}

#[test]
fn the_label_is_cut_at_17_code_points_not_bytes_and_long_scene_texts_are_not() {
    let label = |name: &str| {
        let (scene, diagnostics) = with_buttons(&format!(r#"{{"name": {name}}}"#));
        (
            scene.buttons.as_deref().unwrap()[0].label(),
            diagnostics
                .iter()
                .map(|d| d.code.name())
                .collect::<Vec<_>>(),
        )
    };
    let e16 = "é".repeat(16);
    let e17 = "é".repeat(17);
    assert_eq!(label(&format!("\"{e16}\"")), (e16.clone(), vec![]));
    assert_eq!(
        label(&format!("\"{e17}\"")),
        (e16.clone(), vec!["ClampedValue"])
    );
    // The evaluated text is what is measured: a short text in a long object is not cut.
    assert_eq!(
        label(r#"{"rawtext": [{"text": "Short"}, {"text": " label"}]}"#),
        ("Short label".to_owned(), vec![])
    );
    assert_eq!(
        label(r#"{"rawtext": [{"text": "ABCDEFGHIJKLMNOPQ"}]}"#),
        ("ABCDEFGHIJKLMNOP".to_owned(), vec!["ClampedValue"])
    );
    // An object that is not a raw-text object is its own JSON text.
    assert_eq!(label(r#"{"foo": 1}"#), ("{\"foo\":1}\n".to_owned(), vec![]));
    // `npc::MAX_NAME_LENGTH` (32) plays no part at load: names and texts of any length load.
    let long = "x".repeat(200);
    let (scene, diagnostics) = one_scene(&format!(
        r#"{{"scene_tag": "{long}", "npc_name": "{long}", "text": "{long}"}}"#
    ));
    assert_eq!(scene.text, DialogueText::Plain(long.clone()));
    assert_eq!(scene.npc_name, Some(DialogueText::Plain(long)));
    assert!(diagnostics.is_empty());
}

// ---- duplicates -----------------------------------------------------------------------------

/// A user pack `uuid` with these dialogue files.
fn dialogue_pack(uuid: u128, files: &[(&str, String)]) -> bedrock_addon::pack::Pack {
    pack(
        uuid,
        files
            .iter()
            .map(|(path, text)| ((*path).to_owned(), text.clone().into_bytes()))
            .collect(),
    )
}

const TAG_A: &str = r#"{"scene_tag": "a", "text": "x"}"#;

#[test]
fn the_same_tag_in_two_files_of_one_pack_keeps_the_first_file_s() {
    let stack = stack(vec![dialogue_pack(
        10,
        &[
            (
                "dialogue/z.json",
                file(r#"{"scene_tag": "a", "text": "from z"}"#),
            ),
            (
                "dialogue/b.json",
                file(r#"{"scene_tag": "a", "text": "from b"}"#),
            ),
        ],
    )]);
    let registry = stack.dialogue();
    assert_eq!(
        scene(registry, "a").text,
        DialogueText::Plain("from b".to_owned())
    );
    let found: Vec<_> = registry
        .diagnostics()
        .map(|d| (d.code.to_string(), d.severity))
        .collect();
    assert_eq!(
        found,
        [(
            "JSON: z.json has an error: \n A scene with a scene name of 'a' already exists. Skipping scene..."
                .to_owned(),
            Severity::Error
        )]
    );
    assert_eq!(registry.rejected().len(), 1);
}

#[test]
fn three_packs_with_one_tag_keep_the_bottom_one_and_report_the_two_above() {
    let packs = (0..3)
        .map(|i| dialogue_pack(10 * (i + 1), &[("dialogue/a.json", file(TAG_A))]))
        .collect();
    let stack = stack(packs);
    let registry = stack.dialogue();
    assert_eq!(scene(registry, "a").origin().pack.index(), Some(2));
    let mut found: Vec<_> = registry
        .diagnostics()
        .map(|d| {
            (
                d.code.name(),
                d.location
                    .pack
                    .as_ref()
                    .and_then(bedrock_addon::pack::PackRef::index),
            )
        })
        .collect();
    found.sort();
    assert_eq!(
        found,
        [
            ("DuplicateIdentifier", Some(0)),
            ("DuplicateIdentifier", Some(1)),
            ("ShadowedDefinition", Some(0)),
            ("ShadowedDefinition", Some(1)),
        ]
    );
    // Without `keep_shadowed` only the effective scene is listed.
    assert_eq!(registry.layers(&DialogueSceneTag::parse("a")).count(), 1);
}

#[test]
fn scene_tags_are_case_sensitive_and_a_dropped_scene_does_not_reserve_its_tag() {
    let document = loads(&file(
        r#"{"scene_tag": "a", "text": "x"}, {"scene_tag": "A", "text": "x"}"#,
    ));
    assert_eq!(document.scenes.len(), 2);
    assert!(document.diagnostics().is_empty());
    // The first `a` has no text and is dropped; the second one is the first stored.
    let text = file(r#"{"scene_tag": "a"}, {"scene_tag": "a", "text": "second"}"#);
    let stack = stack(vec![dialogue_pack(10, &[("dialogue/a.json", text)])]);
    let registry = stack.dialogue();
    assert_eq!(
        scene(registry, "a").text,
        DialogueText::Plain("second".to_owned())
    );
    assert!(
        registry
            .diagnostics()
            .all(|d| d.code.name() != "DuplicateIdentifier")
    );
}

#[test]
fn a_duplicate_within_a_file_is_dropped_from_the_file_view_with_a_warning() {
    let document = loads(&file(
        r#"{"scene_tag": "a", "text": "first"}, {"scene_tag": "a", "text": "second"}"#,
    ));
    assert_eq!(document.scenes.len(), 1);
    assert_eq!(
        document.scenes[0].text,
        DialogueText::Plain("first".to_owned())
    );
    let found: Vec<_> = document
        .diagnostics()
        .iter()
        .map(|d| {
            (
                d.code.to_string(),
                d.severity,
                d.location.json_path.to_string(),
            )
        })
        .collect();
    assert_eq!(
        found,
        [(
            "JSON: a.json has an error: \n A scene with a scene name of 'a' already exists. Skipping scene..."
                .to_owned(),
            // a kept error
            Severity::Error,
            "minecraft:npc_dialogue/scenes/1".to_owned()
        )]
    );
}

// ---- unknown members ------------------------------------------------------------------------

const UNKNOWN_MEMBERS: &str = r#"{
  "format_version": "1.14.0",
  "extra": 1,
  "author": "someone",
  "minecraft:npc_dialogue": {
    "notes": ["a", "b"],
    "scenes": [
      {
        "scene_tag": "a",
        "text": "x",
        "custom": {"nested": true},
        "buttons": [{"name": "b", "icon": "textures/ui/icon", "commands": []}]
      }
    ]
  }
}"#;

///: vanilla never reads other members; ours reports each as an
/// `UnrecognizedField` info and keeps it at its level; the same holds for scenes and buttons.
#[test]
fn unknown_members_are_reported_as_infos_and_kept_deviation() {
    let document = loads(UNKNOWN_MEMBERS);
    let found: Vec<_> = document
        .diagnostics()
        .iter()
        .map(|d| (d.code.clone(), d.severity, d.location.json_path.to_string()))
        .collect();
    let info = |at: &str| {
        (
            Code::from(&codes::UNRECOGNIZED_FIELD),
            Severity::Info,
            at.to_owned(),
        )
    };
    assert_eq!(
        found,
        [
            info("extra"),
            info("author"),
            info("minecraft:npc_dialogue/notes"),
            info("minecraft:npc_dialogue/scenes/0/buttons/0/icon"),
            info("minecraft:npc_dialogue/scenes/0/custom"),
        ]
    );
    // A member literally named "extra" is an unknown member like any other.
    let unknown = document.unknown();
    let root = JsonPath::new();
    assert_eq!(
        unknown
            .get(&root, "extra")
            .and_then(|v| v.as_number()?.to_i64()),
        Some(1)
    );
    assert!(unknown.get(&root, "author").is_some());
    assert_eq!(unknown.members_at(&root).count(), 2);
    let payload = root.child("minecraft:npc_dialogue");
    assert_eq!(
        unknown
            .members_at(&payload)
            .map(|(k, _)| k)
            .collect::<Vec<_>>(),
        ["notes"]
    );
    let scene = payload.child("scenes").index(0);
    assert_eq!(
        unknown
            .members_at(&scene)
            .map(|(k, _)| k)
            .collect::<Vec<_>>(),
        ["custom"]
    );
    let button = scene.child("buttons").index(0);
    assert_eq!(
        unknown
            .members_at(&button)
            .map(|(k, _)| k)
            .collect::<Vec<_>>(),
        ["icon"]
    );
}

#[test]
fn known_members_are_never_unknown_and_a_non_object_payload_has_none() {
    let document = loads(
        r#"{"format_version": "1.14.0", "minecraft:npc_dialogue": {"scenes": [
            {"scene_tag": "a", "npc_name": "n", "text": "x", "buttons": [{"name": "b", "url": "u"}, {"name": "c", "commands": []}],
             "on_open_commands": [], "on_close_commands": []}]}}"#,
    );
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    // No level of the document, scenes and buttons included, keeps a member.
    assert!(document.unknown().is_empty());
}

#[test]
fn a_skipped_file_and_a_dropped_scene_report_no_unknown_members() {
    let diagnostics = rejected(r#"{"format_version": "1.14.0", "author": "x"}"#);
    assert_eq!(diagnostics.len(), 1);
    let document = loads(&file(r#"{"scene_tag": "a", "custom": 1}"#));
    assert_eq!(document.diagnostics().len(), 1);
    // a kept error
    assert_eq!(
        names(document.diagnostics(), Severity::Error),
        ["MissingRequiredField"]
    );
}

// ---- both ends of the version rule ----------------------------------------------------------

/// What a file of this `format_version` JSON does: `Ok(version)` or the error's code name.
fn version_outcome(written: &str) -> Result<String, &'static str> {
    let text =
        format!(r#"{{"format_version": {written}, "minecraft:npc_dialogue": {{"scenes": []}}}}"#);
    match load("dialogue/a.json", &text) {
        Ok(document) => Ok(document.format_version.to_string()),
        Err(rejected) => {
            assert_eq!(rejected.diagnostics.len(), 1, "{written}");
            let error = rejected.diagnostics.first_error().expect("an error");
            assert_eq!(
                error.location.json_path.to_string(),
                "format_version",
                "{written}"
            );
            Err(error.code.name())
        }
    }
}

#[test]
fn the_floor_is_1_14_0_inclusive_and_there_is_no_ceiling() {
    for (written, version) in [
        (r#""1.14.0""#, "1.14.0"),
        (r#""1.14""#, "1.14.0"),
        (r#""1.14.1""#, "1.14.1"),
        (r#""1.15""#, "1.15.0"),
        (r#""2""#, "2.0.0"),
        (r#""1.14.0+build""#, "1.14.0+build"),
        (r#""1.14.1-beta""#, "1.14.1-beta"),
        (r#""1.15.0-rc.1+build.5""#, "1.15.0-rc.1+build.5"),
        (r#""2.0.0+x""#, "2.0.0+x"),
        (r#""65535.65535.65535""#, "65535.65535.65535"),
        (r#""10000.0.0""#, "10000.0.0"),
        (r"[1, 14, 0]", "1.14.0"),
        (r"[65535, 0, 0]", "65535.0.0"),
    ] {
        assert_eq!(
            version_outcome(written),
            Ok(version.to_owned()),
            "{written}"
        );
    }
    for written in [
        r#""1.13.65535""#,
        r#""1.13""#,
        r#""1""#,
        r#""0""#,
        r#""0.14.0""#,
        r#""0.0.0""#,
        r#""1.14.0-beta""#,
        r"[1, 13, 9]",
        r"[0, 99, 0]",
    ] {
        assert_eq!(
            version_outcome(written),
            Err("UnsupportedVersion"),
            "{written}"
        );
    }
}

#[test]
fn a_version_vanilla_cannot_read_is_malformed_and_an_absent_or_null_one_missing() {
    for written in [
        r#""""#,
        r#""*""#,
        r#""abc""#,
        r#""01.14.0""#,
        r#""1.014.0""#,
        r#""1.14.00""#,
        r#""1.14.0.0""#,
        r#""1..0""#,
        r#""1.14.""#,
        r#""100000.0.0""#,
        r#""65536.0.0""#,
        r#""v1.14.0""#,
        r#""1.14.0 ""#,
        // `SemVerRegex` takes `-<pre>` and `+<build>` only after the third number.
        r#""1.15-beta""#,
        r#""2-rc""#,
        r#""1.14+b""#,
        r#""2+x""#,
        r#""1.14-beta""#,
        r#""1-rc""#,
        r#""1.14.0-""#,
        r#""1.14.0+""#,
        r#""Beta""#,
        "1.14",
        "2",
        "true",
        "false",
        "{}",
        "[]",
        "[1, 14]",
        "[1, 14, 0, 0]",
        r#"[1, "14", 0]"#,
        "[1, 14, -1]",
        "[65536, 0, 0]",
    ] {
        assert_eq!(
            version_outcome(written),
            Err("MalformedFormatVersion"),
            "{written}"
        );
    }
    assert_eq!(version_outcome("null"), Err("MissingFormatVersion"));
    let diagnostics = rejected(r#"{"minecraft:npc_dialogue": {"scenes": []}}"#);
    assert_eq!(
        names(&diagnostics, Severity::Error),
        ["MissingFormatVersion"]
    );
}

// ---- the file and its provenance ------------------------------------------------------------

#[test]
fn a_byte_order_mark_is_skipped_and_bad_bytes_or_depth_are_syntax_errors() {
    use bedrock_addon::document::Content;
    use bedrock_addon::families::dialogue::DialogueFile;
    use bedrock_addon::storage::PackPath;
    let path = PackPath::new("dialogue/a.json").unwrap();
    let mut bom = vec![0xEF, 0xBB, 0xBF];
    bom.extend_from_slice(file(TAG_A).as_bytes());
    let document = DialogueFile::parse(&path, &bom, &context()).expect("loads");
    assert_eq!(document.scenes.len(), 1);
    let syntax = |bytes: &[u8]| {
        let rejected = DialogueFile::parse(&path, bytes, &context()).expect_err("rejected");
        assert_eq!(rejected.diagnostics.len(), 1);
        let error = rejected.diagnostics.first_error().unwrap();
        assert!(error.code.is(&dialogue_codes::READER), "{:?}", error.code);
        assert!(
            error
                .code
                .to_string()
                .starts_with("JSON: a.json has an error: \n ")
        );
    };
    syntax(b"{\"format_version\": \"\xFF\"}");
    let deep = format!("{}{}", "[".repeat(200), "]".repeat(200));
    syntax(deep.as_bytes());
    // 128 levels are within the limit: the root is then simply not an object.
    let nested = format!("{}{}", "[".repeat(128), "]".repeat(128));
    let rejected = DialogueFile::parse(&path, nested.as_bytes(), &context()).expect_err("rejected");
    assert_eq!(
        rejected.diagnostics.first_error().unwrap().code.to_string(),
        "JSON: a.json has an error: \n Root value is not a object"
    );
}

#[test]
fn documents_say_where_they_came_from_and_what_version_they_were_written_in() {
    use bedrock_addon::document::Content;
    use bedrock_addon::families::dialogue::DialogueFile;
    use bedrock_addon::pack::PackSide;
    use bedrock_addon::version::FormatVersion;
    assert_eq!(DialogueFile::NAME, "dialogue");
    assert_eq!(DialogueFile::SIDES, [PackSide::Behavior]);
    let text = r#"{"format_version": "1.20.30", "minecraft:npc_dialogue": {"scenes": [
        {"scene_tag": "a", "text": 5, "buttons": [{"name": 1}]}]}}"#;
    let document = loads(text);
    assert_eq!(document.origin().path.as_str(), "dialogue/a.json");
    assert_eq!(document.origin().key, None);
    assert_eq!(document.origin().side, PackSide::Behavior);
    let version = document.version();
    assert_eq!(version.written, Some(FormatVersion::new(1, 20, 30)));
    assert_eq!(version.effective, FormatVersion::new(1, 20, 30));
    assert_eq!(version.parsed_as, FormatVersion::new(1, 14, 0));
    assert!(!version.beta && !version.upgraded);
    for finding in document.diagnostics() {
        assert_eq!(
            finding
                .location
                .path
                .as_ref()
                .map(bedrock_addon::storage::PackPath::as_str),
            Some("dialogue/a.json")
        );
        assert!(finding.location.pack.is_some());
    }
    let stack = stack(vec![dialogue_pack(
        10,
        &[("dialogue/sub/a.json", text.to_owned())],
    )]);
    let registry = stack.dialogue();
    let a = scene(registry, "a");
    assert_eq!(a.origin().key.as_deref(), Some("a"));
    assert_eq!(a.origin().side, PackSide::Behavior);
    assert_eq!(a.origin().pack.index(), Some(0));
    assert_eq!(a.version(), version);
    assert_eq!(a.diagnostics().len(), 2);
    for finding in registry.diagnostics() {
        assert_eq!(
            finding
                .location
                .path
                .as_ref()
                .map(bedrock_addon::storage::PackPath::as_str),
            Some("dialogue/sub/a.json")
        );
        assert_eq!(
            finding
                .location
                .pack
                .as_ref()
                .and_then(bedrock_addon::pack::PackRef::index),
            Some(0)
        );
    }
    // "beta" is a version like any other here: the one shape reads it, not a beta struct.
    let beta = loads(r#"{"format_version": "beta", "minecraft:npc_dialogue": {"scenes": []}}"#);
    assert_eq!(
        beta.version().written,
        Some(FormatVersion::new(9999, 9999, 9999))
    );
    assert!(!beta.version().beta);
}

#[test]
fn the_registry_reports_the_findings_of_a_scene_dropped_for_its_empty_tag_and_of_the_file() {
    let text = r#"{"format_version": "beta", "author": "x", "minecraft:npc_dialogue": {"scenes": [
        {"scene_tag": "", "text": "x", "buttons": [{"name": 1}], "on_open_commands": [2]}]}}"#;
    let stack = stack(vec![dialogue_pack(
        10,
        &[("dialogue/a.json", text.to_owned())],
    )]);
    let registry = stack.dialogue();
    assert!(registry.is_empty());
    assert!(registry.rejected().is_empty());
    let found: Vec<_> = registry
        .diagnostics()
        .map(|d| (d.code.name(), d.severity, d.location.path.is_some()))
        .collect();
    assert_eq!(
        found,
        [
            // the beta version is ours (an information), the two command lines the game's
            // (kept errors)
            ("InvalidValue", Severity::Info, true),
            ("UnrecognizedField", Severity::Info, true),
            ("InvalidType", Severity::Error, true),
            ("InvalidType", Severity::Error, true),
        ]
    );
}

// ---- the label's evaluation -----------------------------------------------------------------

#[test]
fn a_raw_text_name_is_evaluated_part_by_part_and_anything_else_is_shown_as_written() {
    use bedrock_addon::families::dialogue::Button;
    let label = |name: DialogueText| Button::new(name).label();
    let plain = |text: &str| label(DialogueText::Plain(text.to_owned()));
    // A string that spells a raw-text object is evaluated like an object name.
    assert_eq!(plain(r#"{"rawtext":[{"text":"A"},{"text":"B"}]}"#), "AB");
    // A `translate` part shows its key; a part that is neither shows nothing; `text` wins.
    assert_eq!(
        plain(
            r#"{"rawtext":[{"translate":"k.e.y"},{"score":{}},{"text":"!","translate":"t"},5,{"text":7}]}"#
        ),
        "k.e.y!"
    );
    assert_eq!(plain(r#"{"rawtext":[]}"#), "");
    // Not a raw-text object: the string itself (cut to 16).
    assert_eq!(plain(r#"{"rawtext":5}"#), r#"{"rawtext":5}"#);
    assert_eq!(plain(r#"["rawtext"]"#), r#"["rawtext"]"#);
    assert_eq!(plain(r#"{"text":"x"}"#), r#"{"text":"x"}"#);
    assert_eq!(plain("{not json"), "{not json");
    assert_eq!(plain(""), "");
    assert_eq!(plain("12345678901234567"), "1234567890123456");
    // An object that JSON cannot spell has no stored text.
    let mut object = facet_value::VObject::new();
    object.insert("x", bedrock_addon::json::Value::from(f64::INFINITY));
    let unspellable = DialogueText::Raw(bedrock_addon::json::Value::from(object));
    assert_eq!(unspellable.stored(), "");
    assert_eq!(label(unspellable), "");
}

#[test]
fn a_button_s_action_is_a_url_action_only_for_a_non_empty_url() {
    use bedrock_addon::families::dialogue::{Button, StoredCommand};
    let name = DialogueText::Plain("b".to_owned());
    let mut button = Button::new(name.clone());
    assert!(matches!(
        button.action(),
        NpcAction::Command { mode: ActionMode::Button, ref commands, .. } if commands.is_empty()
    ));
    button.url = Some(String::new());
    button.commands = Some(vec![StoredCommand::new("/a"), StoredCommand::new("/b")]);
    let action = button.action();
    assert!(matches!(action, NpcAction::Command { .. }));
    assert_eq!(action.text(), "/a\n/b\n");
    button.url = Some("u".to_owned());
    assert_eq!(
        button.action(),
        NpcAction::Url {
            button_name: name,
            url: "u".to_owned()
        }
    );
    assert_eq!(button.action().text(), "u");
}

// ---- what loading does not check ------------------------------------------------------------

/// is stored raw and compiled when its action runs, so no command, selector, URL or raw-text
/// `translate` key is a load finding.
#[test]
fn commands_urls_and_lang_keys_are_not_checked_at_load() {
    let text = file(
        r#"{"scene_tag": "a",
            "npc_name": {"rawtext": [{"translate": "no.such.key"}]},
            "text": {"rawtext": [{"translate": "no.such.key", "with": ["x"]}, {"selector": "@nobody[bad"}]},
            "buttons": [
                {"name": "bad", "commands": ["/nosuchcommand 1 2 3", "not even a slash", "/say @e[type="]},
                {"name": "url", "url": "not a url"}],
            "on_open_commands": ["/dialogue open @s @initiator no_such_scene"],
            "on_close_commands": ["/function no/such/function"]}"#,
    );
    let document = loads(&text);
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    let scene = &document.scenes[0];
    let raw: Vec<&str> = scene.buttons.as_ref().unwrap()[0]
        .commands
        .as_ref()
        .unwrap()
        .iter()
        .map(|c| c.raw.as_str())
        .collect();
    assert_eq!(
        raw,
        ["/nosuchcommand 1 2 3", "not even a slash", "/say @e[type="]
    );
    let stack = stack(vec![dialogue_pack(10, &[("dialogue/a.json", text)])]);
    assert_eq!(stack.dialogue().diagnostics().count(), 0);
    assert_eq!(stack.dialogue().len(), 1);
}

/// The upgrade steps: an array of "exactly 3 elements, each convertible to uint". jsoncpp's
/// `isConvertibleTo(uintValue)` admits `null` (0), booleans (0 / 1) and non-negative reals
/// (truncated), as the manifest's reader of the same `SemVersion::fromJsonArray` does;
/// core's `SemVersion::from_json` takes numbers only, so dialogue reads the array
/// itself.
#[test]
fn version_array_elements_convert_to_unsigned_as_jsoncpp_does() {
    for (written, version) in [
        ("[1, 14, 0]", "1.14.0"),
        ("[1.5, 14, 0]", "1.14.0"),
        ("[1, 14, 0.0]", "1.14.0"),
        ("[1, 14.9, 0.9]", "1.14.0"),
        ("[true, 14, 0]", "1.14.0"),
        ("[1, 14, null]", "1.14.0"),
        ("[1, 14, false]", "1.14.0"),
        ("[2, null, null]", "2.0.0"),
        ("[1, 14, true]", "1.14.1"),
        ("[65535.9, 0, 0]", "65535.0.0"),
        ("[1, 14, -0.0]", "1.14.0"),
    ] {
        assert_eq!(
            version_outcome(written),
            Ok(version.to_owned()),
            "{written}"
        );
    }
    // Read, and below the floor.
    for written in [
        "[null, 14, 0]",
        "[false, 99, 0]",
        "[true, 13, 9]",
        "[0.9, 99, 0]",
    ] {
        assert_eq!(
            version_outcome(written),
            Err("UnsupportedVersion"),
            "{written}"
        );
    }
    // Not convertible, out of range, or not three elements.
    for written in [
        r#"["1", 14, 0]"#,
        r#"[1, 14, ""]"#,
        "[[1], 14, 0]",
        "[1, {}, 0]",
        "[-0.5, 14, 0]",
        "[1, 14, -1]",
        "[1, -1.0, 0]",
        "[65536, 0, 0]",
        "[65536.0, 0, 0]",
        "[1, 14, 1e9]",
        "[null, null]",
        "[true, true, true, true]",
        "[]",
    ] {
        assert_eq!(
            version_outcome(written),
            Err("MalformedFormatVersion"),
            "{written}"
        );
    }
}

// ---- the JSON dialect -----------------------------------------------------------

/// Deviation (parse error `)`):
/// every pack file is read by the one jsonc reader, so a dialogue file with trailing commas
/// loads without a finding where vanilla's `Json::Reader` skips the file.
#[test]
fn trailing_commas_load_silently_deviation() {
    let document = loads(
        r#"{
  "format_version": "1.14.0",
  "minecraft:npc_dialogue": {
    "scenes": [
      {
        "scene_tag": "a",
        "text": "x",
        "buttons": [ { "name": "b", "commands": [ "/say a", ], }, ],
        "on_open_commands": [ "/say open", ],
      },
    ],
  },
}"#,
    );
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    assert_eq!(document.scenes.len(), 1);
    let scene = &document.scenes[0];
    assert_eq!(
        scene.buttons.as_ref().unwrap()[0]
            .commands
            .as_ref()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(scene.on_open_commands.as_ref().unwrap().len(), 1);
    // A doubled or leading comma is a syntax error for both readers.
    for text in [
        r#"{"format_version": "1.14.0",, "minecraft:npc_dialogue": {"scenes": []}}"#,
        r#"{"format_version": "1.14.0", "minecraft:npc_dialogue": {"scenes": [,]}}"#,
        r#"{, "format_version": "1.14.0", "minecraft:npc_dialogue": {"scenes": []}}"#,
    ] {
        let diagnostics = rejected(text);
        assert!(
            diagnostics
                .first_error()
                .is_some_and(|d| d.code.is(&dialogue_codes::READER)),
            "{text}"
        );
    }
}

/// As vanilla: content after the root is ignored, and of a member written twice the
/// last value stands.
#[test]
fn content_after_the_root_is_ignored_and_a_repeated_member_takes_its_last_value() {
    let document = loads(
        r#"{"format_version": "1.13.0", "format_version": "1.14.0", "minecraft:npc_dialogue": {"scenes": [
            {"scene_tag": "first", "text": "x", "scene_tag": "last"}]}} trailing words"#,
    );
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    assert_eq!(document.scenes[0].scene_tag.as_str(), "last");
}

/// The registry reports the infos too, and `expected.json` lists every finding at every
/// severity (an unlisted info fails the fixture).
#[test]
fn unknown_members_are_infos_in_the_registry_as_well() {
    let stack = load_fixture("v_unknown_members");
    let registry = stack.dialogue();
    assert_eq!(registry.diagnostics().count(), 5);
    assert!(registry.diagnostics().all(|d| d.severity == Severity::Info));
    let a = scene(registry, "a");
    assert_eq!(
        a.unknown()
            .members_at(&JsonPath::new())
            .map(|(k, _)| k)
            .collect::<Vec<_>>(),
        ["custom"]
    );
}
