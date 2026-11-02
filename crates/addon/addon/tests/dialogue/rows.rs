//! Every row of the fixtures: the list ("Fixtures to write", `l1`–`l9`) and
//! the table (`d01`–`d27`), fixtures under `tests/fixtures/dialogue/dialogue/`.

use bedrock_addon::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon::document::{Content, Document};
use bedrock_addon::families::dialogue::{
    ActionMode, Button, DialogueFile, DialogueScene, DialogueText, NpcAction, StoredCommand,
};
use bedrock_addon::id::DialogueSceneTag;
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::MinEngineVersion;
use bedrock_addon::version::SemVersion;

use bedrock_addon::pack::{Behavior, PackStack};

use crate::common::*;

/// A plain text.
fn plain(text: &str) -> DialogueText {
    DialogueText::Plain(text.to_owned())
}

/// A command at the command version of `min_engine_version` 1.26.20 (46).
fn command(raw: &str) -> StoredCommand {
    StoredCommand {
        version: 46,
        raw: raw.to_owned(),
    }
}

// ---- the list -----------------------------------------------------------------------------

#[test]
fn l1_format_version_1_13_0_is_rejected() {
    load_fixture("l1_below_floor");
    let diagnostics = rejected(&fixture_text("l1_below_floor/dialogue/old.json"));
    assert!(matches!(
        diagnostics.first_error().map(|d| &d.code),
        Some(DiagnosticCode::UnsupportedVersion { text: Some(_) })
    ));
}

#[test]
fn l3_missing_format_version_is_rejected() {
    load_fixture("l3_missing_version");
    let diagnostics = rejected(&fixture_text(
        "l3_missing_version/dialogue/unversioned.json",
    ));
    assert!(matches!(
        diagnostics.first_error().map(|d| &d.code),
        Some(DiagnosticCode::MissingFormatVersion { text: Some(_) })
    ));
}

#[test]
fn l2_format_version_1_14_0_is_accepted() {
    let stack = load_fixture("l2_floor_accepted");
    let floor = scene(stack.dialogue(), "floor");
    assert_eq!(floor.text, plain("Loaded at the floor."));
    let file = loads(&fixture_text("l2_floor_accepted/dialogue/floor.json"));
    assert_eq!(file.format_version, SemVersion::new(1, 14, 0));
}

// ---- the table ----------------------------------------------------------------------------

#[test]
fn d01_vanilla_test_input_with_a_partial_version_loads_three_scenes() {
    let stack = load_fixture("d01_vanilla_test_1_14");
    let registry = stack.dialogue();
    let test_scene = scene(registry, "test_scene");
    assert_eq!(test_scene.npc_name, Some(plain("Test")));
    assert_eq!(test_scene.text, plain("This is a test scene."));
    assert_eq!(test_scene.buttons, None);
    let no_button = scene(registry, "no_button_test");
    assert_eq!(
        no_button.on_open_commands,
        Some(vec![command("/give @initiator emerald")])
    );
    assert_eq!(no_button.on_close_commands, None);
    let give = scene(registry, "give_emerald_scene");
    let buttons = give.buttons.as_ref().expect("buttons");
    assert_eq!(buttons.len(), 1);
    let Button {
        name,
        commands,
        url,
        ..
    } = &buttons[0];
    assert_eq!(name, &plain("Yes"));
    assert_eq!(commands, &Some(vec![command("/give @initiator emerald")]));
    assert_eq!(url, &None);
    let file = loads(&fixture_text(
        "d01_vanilla_test_1_14/dialogue/vanilla_test.json",
    ));
    assert_eq!(file.format_version, SemVersion::new(1, 14, 0));
    assert_eq!(file.scenes.len(), 3);
}

#[test]
fn d03_three_integer_array_version_loads_and_other_arrays_are_unreadable() {
    let stack = load_fixture("d03_array_version");
    assert_eq!(scene(stack.dialogue(), "array").text, plain("Hello."));
    let file = loads(&fixture_text("d03_array_version/dialogue/full.json"));
    assert_eq!(file.format_version, SemVersion::new(1, 14, 0));
    for name in ["short", "mixed"] {
        let diagnostics = rejected(&fixture_text(&format!(
            "d03_array_version/dialogue/{name}.json"
        )));
        assert!(matches!(
            diagnostics.first_error().map(|d| &d.code),
            Some(DiagnosticCode::MalformedFormatVersion { text: Some(_) })
        ));
    }
}

#[test]
fn d04_a_number_version_is_unreadable() {
    load_fixture("d04_number_version");
}

#[test]
fn d05_a_version_above_the_engine_loads_without_a_finding() {
    let stack = load_fixture("d05_no_upper_bound");
    assert_eq!(stack.dialogue().diagnostics().count(), 0);
    let file = loads(&fixture_text("d05_no_upper_bound/dialogue/future.json"));
    assert_eq!(file.format_version, SemVersion::new(99, 0, 0));
}

#[test]
fn d22_version_strings_follow_vanilla_s_grammar() {
    let stack = load_fixture("d22_version_string_forms");
    assert_eq!(scene(stack.dialogue(), "beta").text, plain("Hello."));
}

///: ours is an
/// information.
#[test]
fn d22_beta_loads_as_9999_with_an_information_deviation() {
    let file = loads(&fixture_text("d22_version_string_forms/dialogue/beta.json"));
    assert_eq!(
        (
            file.format_version.major,
            file.format_version.minor,
            file.format_version.patch
        ),
        (9999, 9999, 9999)
    );
    assert_eq!(names(file.diagnostics(), Severity::Info), ["InvalidValue"]);
}

#[test]
fn d07_a_truncated_file_is_skipped_with_the_reader_s_message_after_the_prefix() {
    let stack = load_fixture("d07_invalid_json");
    let finding = stack
        .dialogue()
        .diagnostics()
        .next()
        .expect("the syntax error");
    let DiagnosticCode::MalformedInput { message, .. } = &finding.code else {
        panic!("{:?}", finding.code);
    };
    assert!(!message.is_empty());
    assert_eq!(
        finding.code.to_string(),
        format!("JSON: truncated.json has an error: \n {message}")
    );
}

#[test]
fn d08_scenes_that_are_not_an_array_skip_the_file() {
    load_fixture("d08_scenes_not_array");
}

#[test]
fn d26_comments_anywhere_in_the_file_load() {
    let stack = load_fixture("d26_comments");
    assert_eq!(
        scene(stack.dialogue(), "commented").text,
        plain("Comments are fine.")
    );
}

#[test]
fn l7_rawtext_object_text_and_npc_name_are_kept_as_json_text() {
    let stack = load_fixture("l7_rawtext_objects");
    let rawtext = scene(stack.dialogue(), "rawtext");
    assert!(matches!(rawtext.text, DialogueText::Raw(_)));
    assert_eq!(
        rawtext.text.stored(),
        "{\"rawtext\":[{\"text\":\"Hello \"},{\"translate\":\"npc.greeting\",\"with\":[\"Steve\"]}]}\n"
    );
    assert_eq!(
        rawtext
            .npc_name
            .as_ref()
            .map(DialogueText::stored)
            .as_deref(),
        Some("{\"rawtext\":[{\"translate\":\"npc.name\"}]}\n")
    );
}

#[test]
fn d09_an_empty_scene_tag_drops_the_scene_silently_and_a_number_tag_with_the_text() {
    let stack = load_fixture("d09_empty_scene_tag");
    let registry = stack.dialogue();
    // The empty tag: no scene, no rejection, no finding at any level.
    assert_eq!(registry.diagnostics().count(), 1);
    assert_eq!(registry.rejected().len(), 1);
    assert_eq!(registry.rejected()[0].origin.key, None);
    assert_eq!(scene(registry, "kept").text, plain("Kept."));
}

#[test]
fn d10_text_of_another_type_is_coerced_as_as_string_does() {
    let stack = load_fixture("d10_text_coercion");
    let registry = stack.dialogue();
    assert_eq!(scene(registry, "true").text, plain("true"));
    assert_eq!(scene(registry, "false").text, plain("false"));
    assert_eq!(scene(registry, "number").text, plain(""));
    assert_eq!(scene(registry, "array").text, plain(""));
}

/// Ours is an information.
#[test]
fn d10_coerced_text_is_an_invalid_type_information_deviation() {
    let stack = load_fixture("d10_text_coercion");
    let coerced = scene(stack.dialogue(), "number");
    assert_eq!(
        names(coerced.diagnostics(), Severity::Info),
        ["InvalidType"]
    );
    assert!(matches!(
        coerced.diagnostics().iter().next().map(|d| &d.code),
        Some(DiagnosticCode::InvalidType { text: None })
    ));
}

#[test]
fn d11_a_number_npc_name_drops_the_scene_and_an_object_is_kept_as_json_text() {
    let stack = load_fixture("d11_npc_name_invalid");
    let registry = stack.dialogue();
    assert_eq!(
        scene(registry, "object_name")
            .npc_name
            .as_ref()
            .map(DialogueText::stored)
            .as_deref(),
        Some("{\"foo\":1}\n")
    );
    assert_eq!(
        registry.rejected()[0].origin.key.as_deref(),
        None,
        "the name is checked before the tag is read"
    );
}

#[test]
fn d12_the_npc_name_is_checked_before_the_scene_tag() {
    let stack = load_fixture("d12_check_order");
    assert_eq!(stack.dialogue().diagnostics().count(), 1);
}

#[test]
fn d25_a_rawtext_text_is_stored_with_the_fast_writer_s_newline() {
    let stack = load_fixture("d25_rawtext_trailing_newline");
    assert_eq!(
        scene(stack.dialogue(), "newline").text.stored(),
        "{\"rawtext\":[{\"text\":\"x\"}]}\n"
    );
}

#[test]
fn d27_a_scene_that_is_not_an_object_fails_the_scene_tag_rule_only() {
    let stack = load_fixture("d27_non_object_scene");
    assert_eq!(stack.dialogue().diagnostics().count(), 1);
}

/// The buttons of the scene `tag`.
#[track_caller]
fn buttons_of<'s>(stack: &'s PackStack<Behavior>, tag: &str) -> &'s [Button] {
    scene(stack.dialogue(), tag)
        .buttons
        .as_deref()
        .expect("buttons")
}

#[test]
fn l5_seven_buttons_drop_the_scene() {
    let stack = load_fixture("l5_seven_buttons");
    assert_eq!(
        stack.dialogue().rejected()[0].origin.key.as_deref(),
        Some("seven")
    );
}

#[test]
fn l6_a_button_with_commands_and_a_url_is_dropped() {
    let stack = load_fixture("l6_commands_and_url");
    assert!(buttons_of(&stack, "both").is_empty());
}

#[test]
fn l8_a_number_in_on_open_commands_skips_that_element_only() {
    let stack = load_fixture("l8_on_open_number");
    assert_eq!(
        scene(stack.dialogue(), "open").on_open_commands,
        Some(vec![command("/say a"), command("/say b")])
    );
}

#[test]
fn d13_only_valid_buttons_count_towards_the_maximum_of_six() {
    let stack = load_fixture("d13_button_count_valid_only");
    let kept = buttons_of(&stack, "eight_with_two_nameless");
    let names: Vec<String> = kept.iter().map(|b| b.name.stored()).collect();
    assert_eq!(
        names,
        [
            "Button 1", "Button 2", "Button 3", "Button 4", "Button 5", "Button 6"
        ]
    );
    assert_eq!(
        stack.dialogue().rejected()[0].origin.key.as_deref(),
        Some("seven_valid")
    );
}

#[test]
fn d14_a_button_name_is_a_string_or_an_object_and_an_empty_one_drops_silently() {
    let stack = load_fixture("d14_button_name_forms");
    let kept = buttons_of(&stack, "names");
    assert_eq!(kept.len(), 2);
    assert_eq!(kept[0].name.stored(), "{\"rawtext\":[{\"text\":\"Hi\"}]}\n");
    assert_eq!(kept[1].name, plain("Plain"));
}

#[test]
fn d15_commands_that_are_not_an_array_are_ignored_silently() {
    let stack = load_fixture("d15_button_commands_not_array");
    let kept = buttons_of(&stack, "string_commands");
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].commands, None);
    assert_eq!(stack.dialogue().diagnostics().count(), 0);
}

#[test]
fn d16_a_button_with_only_a_name_is_a_command_action_with_empty_text() {
    let stack = load_fixture("d16_button_neither");
    let actions = scene(stack.dialogue(), "neither").actions();
    assert_eq!(
        actions,
        [NpcAction::Command {
            mode: ActionMode::Button,
            button_name: plain("Only a name"),
            commands: vec![],
        }]
    );
    assert_eq!(actions[0].text(), "");
}

#[test]
fn d17_a_url_makes_a_url_action_a_number_drops_the_button_and_an_empty_url_is_a_command_button() {
    let stack = load_fixture("d17_url_forms");
    let urls = scene(stack.dialogue(), "urls");
    assert_eq!(
        urls.actions(),
        [
            NpcAction::Url {
                button_name: plain("Link"),
                url: "https://x".to_owned(),
            },
            NpcAction::Command {
                mode: ActionMode::Button,
                button_name: plain("Empty"),
                commands: vec![],
            },
        ]
    );
    assert_eq!(buttons_of(&stack, "urls")[1].url.as_deref(), Some(""));
}

#[test]
fn d18_commands_are_trimmed_and_blank_ones_dropped() {
    let stack = load_fixture("d18_command_trim");
    let trim = scene(stack.dialogue(), "trim");
    let actions = trim.actions();
    assert_eq!(actions[0].text(), "/say a\n/say b\n");
    assert_eq!(
        buttons_of(&stack, "trim")[0].commands,
        Some(vec![command("/say a"), command("/say b")])
    );
    assert_eq!(trim.on_open_commands, Some(vec![command("/say open")]));
    assert_eq!(actions[1].text(), "/say open\n");
}

#[test]
fn d19_a_non_array_open_list_is_ignored_and_an_empty_close_list_is_one_action() {
    let stack = load_fixture("d19_on_open_not_array");
    let lists = scene(stack.dialogue(), "lists");
    assert_eq!(lists.on_open_commands, None);
    assert_eq!(lists.on_close_commands, Some(vec![]));
    assert_eq!(
        lists.actions(),
        [NpcAction::Command {
            mode: ActionMode::OnClose,
            button_name: plain(""),
            commands: vec![],
        }]
    );
    assert_eq!(stack.dialogue().diagnostics().count(), 0);
}

#[test]
fn d23_a_long_button_name_is_kept_and_its_label_cut_to_16_code_points() {
    let stack = load_fixture("d23_button_label_cut");
    let kept = buttons_of(&stack, "labels");
    assert_eq!(kept[0].name, plain("ABCDEFGHIJKLMNOPQRST"));
    assert_eq!(kept[0].label(), "ABCDEFGHIJKLMNOP");
    assert_eq!(kept[1].name, plain("ABCDEFGHIJKLMNOP"));
    assert_eq!(kept[1].label(), "ABCDEFGHIJKLMNOP");
    assert_eq!(stack.dialogue().diagnostics().count(), 1);
}

///: ours is a `ClampedValue` info.
#[test]
fn d23_a_cut_label_is_a_clamped_value_info_deviation() {
    let stack = load_fixture("d23_button_label_cut");
    let labels = scene(stack.dialogue(), "labels");
    let found: Vec<_> = labels
        .diagnostics()
        .iter()
        .map(|d| (d.code.clone(), d.severity))
        .collect();
    assert_eq!(
        found,
        [(DiagnosticCode::ClampedValue { text: None }, Severity::Info)]
    );
}

#[test]
fn d24_a_rawtext_button_name_keeps_its_json_and_shows_the_evaluated_text() {
    let stack = load_fixture("d24_button_rawtext_evaluated");
    let kept = buttons_of(&stack, "evaluated");
    assert_eq!(kept[0].name.stored(), "{\"rawtext\":[{\"text\":\"Hi\"}]}\n");
    assert_eq!(kept[0].label(), "Hi");
    assert_eq!(kept[1].label(), "not json");
}

#[test]
fn l4_a_scene_tag_in_two_packs_keeps_the_lower_priority_pack_s_scene() {
    let stack = load_fixture("l4_duplicate_across_packs");
    let registry = stack.dialogue();
    let shared = scene(registry, "shared");
    assert_eq!(shared.text, plain("From the bottom pack."));
    assert_eq!(shared.origin().pack.index(), Some(1));
    // The other scene of the top pack's file still loads.
    assert_eq!(scene(registry, "top_only").origin().pack.index(), Some(0));
    // "already exists" is about the top pack's scene.
    let error = registry
        .diagnostics()
        .find(|d| d.severity == Severity::Error)
        .expect("the error");
    assert!(matches!(
        error.code,
        DiagnosticCode::DuplicateIdentifier { text: Some(_) }
    ));
    assert_eq!(
        error.location.pack.as_ref().and_then(|p| p.index()),
        Some(0)
    );
    // The dropped scene is the shadowed layer.
    let layers: Vec<_> = registry
        .layers(&DialogueSceneTag::parse("shared"))
        .map(|d| d.text.clone())
        .collect();
    assert_eq!(
        layers,
        [plain("From the bottom pack."), plain("From the top pack.")]
    );
}

///: ours adds a `ShadowedDefinition` info at
/// the higher-priority pack's file.
#[test]
fn l4_the_dropped_higher_scene_is_a_shadowed_definition_info_deviation() {
    let stack = load_fixture("l4_duplicate_across_packs");
    let shadowed: Vec<_> = stack
        .dialogue()
        .diagnostics()
        .filter(|d| d.code.name() == "ShadowedDefinition")
        .collect();
    assert_eq!(shadowed.len(), 1);
    assert_eq!(shadowed[0].severity, Severity::Info);
    assert_eq!(
        shadowed[0].location.pack.as_ref().and_then(|p| p.index()),
        Some(0)
    );
    assert_eq!(
        shadowed[0].location.path.as_ref().map(|p| p.as_str()),
        Some("dialogue/shared.json")
    );
}

#[test]
fn d02_vanilla_test_input_1_17_loads_without_actions() {
    let stack = load_fixture("d02_vanilla_test_1_17");
    let ducky = scene(stack.dialogue(), "ducky_scene");
    assert_eq!(ducky.npc_name, Some(plain("Ducky")));
    assert_eq!(ducky.buttons, Some(vec![]));
    assert!(ducky.actions().is_empty());
    let file = loads(&fixture_text("d02_vanilla_test_1_17/dialogue/ducky.json"));
    assert_eq!(file.format_version, SemVersion::new(1, 17, 0));
}

#[test]
fn d20_each_command_carries_its_pack_s_command_version_not_the_file_s() {
    let stack = load_fixture("d20_command_version_from_pack");
    let registry = stack.dialogue();
    let versions = |document: &Document<DialogueScene>| -> Vec<u8> {
        document
            .actions()
            .iter()
            .flat_map(|action| match action {
                NpcAction::Command { commands, .. } => commands.clone(),
                _ => Vec::new(),
            })
            .map(|command| command.version)
            .collect()
    };
    let layers: Vec<_> = registry
        .layers(&DialogueSceneTag::parse("versioned"))
        .collect();
    assert_eq!(layers.len(), 2);
    // The bottom pack (1.21.0 → 39) is effective; the top pack (1.19.0 → 20) is shadowed.
    assert_eq!(versions(layers[0]), [39, 39, 39]);
    assert_eq!(versions(layers[1]), [20, 20, 20]);
    // The file says 1.17.0 in both; a pack without `min_engine_version` gets the current one.
    let text =
        fixture_text("d20_command_version_from_pack/stack/0_engine_1_19_0/dialogue/versioned.json");
    let path = PackPath::new("dialogue/versioned.json").unwrap();
    let parse = |cx| DialogueFile::parse(&path, text.as_bytes(), &cx).unwrap();
    let unset = parse(context_at(None));
    assert_eq!(
        unset.scenes[0].on_open_commands.as_ref().unwrap()[0].version,
        46
    );
    let old = parse(context_at(Some(MinEngineVersion::new(1, 17, 0))));
    assert_eq!(
        old.scenes[0].on_open_commands.as_ref().unwrap()[0].version,
        15
    );
}

#[test]
fn d21_the_same_tag_twice_in_one_file_keeps_the_first() {
    let stack = load_fixture("d21_duplicate_within_file");
    let registry = stack.dialogue();
    assert_eq!(scene(registry, "a").text, plain("First."));
    assert_eq!(registry.rejected()[0].origin.key.as_deref(), Some("a"));
    assert!(
        registry
            .diagnostics()
            .all(|d| d.code.name() != "ShadowedDefinition"),
        "one pack: nothing is shadowed"
    );
}

#[test]
fn l9_a_file_in_a_sub_folder_loads_and_its_texts_name_the_stem_only() {
    let stack = load_fixture("l9_sub_folder");
    let nested = scene(stack.dialogue(), "nested");
    assert_eq!(nested.origin().path.as_str(), "dialogue/sub/x.json");
    assert_eq!(nested.origin().key.as_deref(), Some("nested"));
}

#[test]
fn d06_only_the_exact_extension_json_is_visited() {
    let stack = load_fixture("d06_extension_case");
    assert_eq!(scene(stack.dialogue(), "json").text, plain("Loaded."));
    assert_eq!(stack.dialogue().len(), 1);
    assert_eq!(stack.dialogue().rejected().len(), 0);
}
