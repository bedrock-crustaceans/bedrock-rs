//! Write-back at the Value level: `parse(write(parse(x))) == parse(x)`, the
//! rewrite of the written text is identical, unknown members stay at their level, and no member
//! the author did not write is written.

use bedrock_addon::document::{Content, Placed};
use bedrock_addon::families::dialogue::{
    Button, DialogueFile, DialogueScene, DialogueText, StoredCommand,
};
use bedrock_addon::id::DialogueSceneTag;
use bedrock_addon::json::{Indent, WriteStyle};
use bedrock_addon::version::SemVersion;

use crate::common::*;

/// Parses, writes, parses again and writes again: the two values and the two texts are equal.
#[track_caller]
fn round_trip(text: &str) -> (DialogueFile, String) {
    let first = loads(text);
    let written = first.to_json().expect("writes");
    let second = loads(&written);
    assert_eq!(
        second.value(),
        first.value(),
        "the rewritten value\n{written}"
    );
    assert_eq!(
        second.to_json().expect("writes"),
        written,
        "the rewrite of the written text"
    );
    (first.into_value(), written)
}

/// Every fixture file that loads, by its path below the fixture directory.
fn loading_fixture_files() -> Vec<String> {
    fn walk(dir: &std::path::Path, base: &std::path::Path, out: &mut Vec<String>) {
        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                walk(&path, base, out);
            } else if path.extension().is_some_and(|e| e == "json")
                && path.components().any(|c| c.as_os_str() == "dialogue")
            {
                out.push(
                    path.strip_prefix(base)
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/"),
                );
            }
        }
    }
    let base = fixture_dir("");
    let mut out = Vec::new();
    walk(&base, &base, &mut out);
    out.retain(|rest| load("dialogue/a.json", &fixture_text(rest)).is_ok());
    out
}

#[test]
fn every_fixture_file_that_loads_round_trips() {
    let files = loading_fixture_files();
    assert!(files.len() >= 30, "{} files", files.len());
    for rest in files {
        let text = fixture_text(&rest);
        let first = loads(&text);
        let written = first.to_json().unwrap_or_else(|e| panic!("{rest}: {e}"));
        let second = match load("dialogue/a.json", &written) {
            Ok(second) => second,
            Err(rejected) => panic!("{rest}: {:#?}\n{written}", rejected.diagnostics),
        };
        assert_eq!(second.value(), first.value(), "{rest}\n{written}");
        assert_eq!(second.to_json().unwrap(), written, "{rest}");
        // Strict: what was dropped or coerced while loading is not written, so the rewritten
        // text loads without a warning.
        assert!(
            second.diagnostics().iter().all(|d| d.severity
                < bedrock_addon::diagnostic::Severity::Warning
                || rest.contains("d22_version_string_forms/dialogue/beta.json")),
            "{rest}: {:#?}",
            second.diagnostics()
        );
    }
}

#[test]
fn a_file_is_written_with_its_members_in_schema_order_and_unknown_ones_after() {
    let (_, written) = round_trip(
        r#"{
  "author": "someone",
  "minecraft:npc_dialogue": {
    "notes": 1,
    "scenes": [
      {
        "on_close_commands": ["  /say bye  "],
        "custom": true,
        "buttons": [
          {"icon": "x", "commands": ["/say a", " "], "name": "Say"},
          {"url": "https://example.com", "name": {"rawtext": [{"translate": "b", "with": ["c"]}]}}
        ],
        "text": {"rawtext": [{"text": "Hello"}]},
        "npc_name": "Guide",
        "on_open_commands": [],
        "scene_tag": "guide"
      }
    ]
  },
  "format_version": [1, 17, 0]
}"#,
    );
    assert_eq!(
        written,
        r#"{
  "format_version": "1.17.0",
  "minecraft:npc_dialogue": {
    "scenes": [
      {
        "scene_tag": "guide",
        "npc_name": "Guide",
        "text": {
          "rawtext": [
            {
              "text": "Hello"
            }
          ]
        },
        "buttons": [
          {
            "name": "Say",
            "commands": [
              "/say a"
            ],
            "icon": "x"
          },
          {
            "name": {
              "rawtext": [
                {
                  "translate": "b",
                  "with": [
                    "c"
                  ]
                }
              ]
            },
            "url": "https://example.com"
          }
        ],
        "on_open_commands": [],
        "on_close_commands": [
          "/say bye"
        ],
        "custom": true
      }
    ],
    "notes": 1
  },
  "author": "someone"
}
"#
    );
}

#[test]
fn members_the_author_did_not_write_are_not_written() {
    let (file, written) = round_trip(
        r#"{"format_version": "1.14.0", "minecraft:npc_dialogue": {"scenes": [
            {"scene_tag": "bare", "text": "x"},
            {"scene_tag": "empty", "text": "", "npc_name": "", "buttons": [{"name": "b"}, {"name": "u", "url": ""}, {"name": "c", "commands": []}]}]}}"#,
    );
    assert_eq!(file.scenes[0].npc_name, None);
    assert_eq!(file.scenes[0].buttons, None);
    let style = WriteStyle {
        indent: Indent::Compact,
        trailing_newline: false,
    };
    let mut compact = Vec::new();
    file.write_json(&mut compact, style).unwrap();
    assert_eq!(
        String::from_utf8(compact).unwrap(),
        r#"{"format_version":"1.14.0","minecraft:npc_dialogue":{"scenes":[{"scene_tag":"bare","text":"x"},{"scene_tag":"empty","npc_name":"","text":"","buttons":[{"name":"b"},{"name":"u","url":""},{"name":"c","commands":[]}]}]}}"#
    );
    assert!(written.ends_with("}\n"));
}

#[test]
fn the_version_is_written_as_read_and_beta_as_the_literal() {
    for (written, expected) in [
        (r#""1.14""#, r#""1.14.0""#),
        (r"[1, 20, 30]", r#""1.20.30""#),
        (r#""1.21.0-rc.1+build.5""#, r#""1.21.0-rc.1+build.5""#),
        (r#""beta""#, r#""beta""#),
    ] {
        let text = format!(
            r#"{{"format_version": {written}, "minecraft:npc_dialogue": {{"scenes": []}}}}"#
        );
        let (_, out) = round_trip(&text);
        assert!(
            out.contains(&format!(r#""format_version": {expected}"#)),
            "{written}: {out}"
        );
    }
}

#[test]
fn a_file_built_by_hand_writes_and_loads_back_equal() {
    let mut scene = DialogueScene::new(
        DialogueSceneTag::parse("greeting"),
        DialogueText::Plain("Hello there.".to_owned()),
    );
    scene.npc_name = Some(DialogueText::Plain("Guide".to_owned()));
    scene.buttons = Some(vec![
        Button {
            commands: Some(vec![StoredCommand::new("/say hi")]),
            ..Button::new(DialogueText::Plain("Hi".to_owned()))
        },
        Button {
            url: Some("https://example.com".to_owned()),
            ..Button::new(DialogueText::Plain("Web".to_owned()))
        },
    ]);
    scene.on_open_commands = Some(vec![StoredCommand::new("/say open")]);
    let file = DialogueFile::new(vec![scene]);
    assert_eq!(file.format_version, SemVersion::new(1, 14, 0));
    assert_eq!(file.default_path().as_str(), "dialogue/greeting.json");
    assert_eq!(
        DialogueFile::new(Vec::new()).default_path().as_str(),
        "dialogue/dialogue.json"
    );
    let written = file.to_json().expect("writes");
    let loaded = loads(&written);
    assert!(
        loaded.diagnostics().is_empty(),
        "{:#?}",
        loaded.diagnostics()
    );
    // `StoredCommand::new` is at the current command version, as the 1.26.20 test pack's is.
    assert_eq!(loaded.value(), &file);
    assert_eq!(StoredCommand::new("  /x ").raw, "  /x ");
    assert_eq!(StoredCommand::new("/x").version, 46);
}

#[test]
fn an_object_text_with_a_number_json_cannot_spell_is_an_encode_error() {
    use bedrock_addon::json::{EncodeError, Value};
    let mut object = facet_value::VObject::new();
    object.insert("x", Value::from(f64::NAN));
    let scene = DialogueScene::new(
        DialogueSceneTag::parse("a"),
        DialogueText::Raw(Value::from(object)),
    );
    let file = DialogueFile::new(vec![scene]);
    assert!(matches!(
        file.to_json(),
        Err(EncodeError::NotRepresentable { .. })
    ));
    let mut out = b"kept".to_vec();
    assert!(file.write_json(&mut out, WriteStyle::default()).is_err());
    assert_eq!(out, b"kept");
}

#[test]
fn a_known_member_among_the_unknown_ones_never_replaces_the_typed_one() {
    use bedrock_addon::diagnostic::JsonPath;
    use bedrock_addon::json::Value;
    let mut scene = DialogueScene::new(
        DialogueSceneTag::parse("typed"),
        DialogueText::Plain("typed text".to_owned()),
    );
    scene.buttons = Some(vec![Button::new(DialogueText::Plain("typed".to_owned()))]);
    let mut document = loads(&DialogueFile::new(vec![scene]).to_json().unwrap());
    let root = JsonPath::new();
    let payload = root.child("minecraft:npc_dialogue");
    let scene_at = payload.child("scenes").index(0);
    let button_at = scene_at.child("buttons").index(0);
    let unknown = document.unknown_mut();
    unknown.put(&scene_at, "scene_tag", Value::from("from extra"));
    unknown.put(&scene_at, "text", Value::from("from extra"));
    unknown.put(&scene_at, "kept", Value::from(true));
    unknown.put(&button_at, "name", Value::from("from extra"));
    unknown.put(&root, "format_version", Value::from("9.9.9"));
    unknown.put(&root, "minecraft:npc_dialogue", Value::from(1_i64));
    unknown.put(&payload, "scenes", Value::from(1_i64));
    let loaded = loads(&document.to_json().unwrap());
    assert_eq!(loaded.format_version, SemVersion::new(1, 14, 0));
    assert_eq!(loaded.scenes.len(), 1);
    let scene = &loaded.scenes[0];
    assert_eq!(scene.scene_tag.as_str(), "typed");
    assert_eq!(scene.text, DialogueText::Plain("typed text".to_owned()));
    assert_eq!(
        loaded
            .unknown()
            .members_at(&scene_at)
            .map(|(k, _)| k)
            .collect::<Vec<_>>(),
        ["kept"]
    );
    assert_eq!(
        scene.buttons.as_ref().unwrap()[0].name,
        DialogueText::Plain("typed".to_owned())
    );
}

#[test]
fn a_dropped_button_s_unknown_member_is_not_written_and_a_kept_one_s_is_at_its_new_index() {
    let text = r#"{"format_version": "1.14.0", "minecraft:npc_dialogue": {"scenes": [{"scene_tag": "t", "text": "x", "buttons": [{"commands": [], "dropped": 1}, {"name": "kept", "commands": [], "kept": 2}]}]}}"#;
    let document = loads(text);
    let buttons = document.scenes[0].buttons.as_ref().expect("buttons");
    assert_eq!(buttons.len(), 1);
    assert_eq!(buttons[0].name, DialogueText::Plain("kept".to_owned()));
    let written = document.to_json().expect("writes");
    let compact: String = written.split_whitespace().collect();
    assert!(
        compact.contains(r#""buttons":[{"name":"kept","commands":[],"kept":2}]"#),
        "{written}"
    );
    assert!(!written.contains("dropped"), "{written}");
}
