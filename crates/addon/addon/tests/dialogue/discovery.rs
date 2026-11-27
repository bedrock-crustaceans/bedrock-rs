//! Which files the dialogue loader visits:
//! `dialogue/**/*.json`, extension exactly `json`; packs shipped inside the app are walked flat.

use bedrock_addon::pack::PackOrigin;

use crate::common::*;

/// A one-scene file.
fn one(tag: &str) -> Vec<u8> {
    format!(
        r#"{{"format_version": "1.14.0", "minecraft:npc_dialogue": {{"scenes": [{{"scene_tag": "{tag}", "text": "x"}}]}}}}"#
    )
    .into_bytes()
}

/// The tags a pack of `origin` with `files` loads, sorted.
fn tags(origin: PackOrigin, files: &[(&str, &str)]) -> Vec<String> {
    let files = files
        .iter()
        .map(|(path, tag)| ((*path).to_owned(), one(tag)))
        .collect();
    let stack = stack(vec![pack_of(10, files, origin)]);
    let registry = stack.dialogue();
    assert_eq!(registry.diagnostics().count(), 0);
    let mut tags: Vec<String> = registry
        .iter()
        .map(|(tag, _)| tag.as_str().to_owned())
        .collect();
    tags.sort();
    tags
}

#[test]
fn only_files_below_dialogue_with_the_extension_json_are_visited() {
    let files = [
        ("dialogue/a.json", "a"),
        ("dialogue/deep/er/b.json", "b"),
        ("dialogue/c.d.json", "c"),
        ("dialogue/upper.JSON", "upper"),
        ("dialogue/mixed.Json", "mixed"),
        ("dialogue/backup.json.bak", "backup"),
        ("dialogue/json", "bare"),
        ("dialogue/jsonish.xjson", "jsonish"),
        ("dialogue.json", "root"),
        ("dialogues/e.json", "plural"),
        ("other/dialogue/f.json", "elsewhere"),
    ];
    assert_eq!(tags(PackOrigin::User, &files), ["a", "b", "c"]);
}

#[test]
fn sub_folders_are_walked_in_every_pack_outside_the_app_package() {
    let files = [
        ("dialogue/flat.json", "flat"),
        ("dialogue/sub/nested.json", "nested"),
    ];
    for origin in [PackOrigin::User, PackOrigin::Dev, PackOrigin::World] {
        assert_eq!(tags(origin, &files), ["flat", "nested"], "{origin:?}");
    }
}

#[test]
fn a_pack_shipped_inside_the_app_is_walked_flat() {
    let files = [
        ("dialogue/flat.json", "flat"),
        ("dialogue/sub/nested.json", "nested"),
        ("dialogue/sub/deep/er.json", "deeper"),
    ];
    assert_eq!(tags(PackOrigin::Package, &files), ["flat"]);
}

#[test]
fn vanilla_s_texts_name_the_stem_without_folders_or_the_last_extension() {
    let stack = stack(vec![pack(
        10,
        vec![
            ("dialogue/sub/deep/x.y.json".to_owned(), b"[]".to_vec()),
            ("dialogue/plain.json".to_owned(), b"[]".to_vec()),
        ],
    )]);
    let mut texts: Vec<String> = stack
        .dialogue()
        .diagnostics()
        .map(|d| d.code.to_string())
        .collect();
    texts.sort();
    assert_eq!(
        texts,
        [
            "JSON: plain.json has an error: \n Root value is not a object",
            "JSON: x.y.json has an error: \n Root value is not a object",
        ]
    );
}
