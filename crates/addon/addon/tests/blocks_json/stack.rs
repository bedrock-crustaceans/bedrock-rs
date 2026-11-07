//! `rp.client_blocks()` and the walk.

use std::collections::BTreeMap;

use bedrock_addon::diagnostic::{Kind, Severity};
use bedrock_addon::document::{Content, Placed};
use bedrock_addon::families::blocks_json::codes::NOT_JSON;
use bedrock_addon::families::blocks_json::{BlocksJson, ClientBlock};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{PackStack, Resource};

use super::common::{
    E, Found, I, W, base, faces, file, file_at, fixture, leak, load, load_one, ours, six, user,
    vanilla,
};
use super::pack_common::rp;

/// How many findings of each `(code, severity)`.
fn tally(found: &[Found]) -> BTreeMap<(&'static str, String), usize> {
    let mut counts = BTreeMap::new();
    for f in found {
        *counts.entry((f.code, f.severity.to_string())).or_insert(0) += 1;
    }
    counts
}

#[test]
fn vanilla_s_1_26_50_file_as_a_base_game_pack() {
    // The 1.26.50 vanilla resource pack's `blocks.json`, copied.
    let text = fixture("bj01_vanilla_12650/packs/00_vanilla/blocks.json");
    let loaded = load(&[base(&text)]);
    let found = loaded.found();
    let block = loaded.block("stone");
    assert_eq!(block.version().effective.to_string(), "1.21.40");
    // The texture-list warning for the two seven-key texture lists.
    let v7: Vec<&str> = found
        .iter()
        .filter(|f| f.code == "blocks_json/bad_textures" && !f.text.is_empty())
        .map(|f| f.text.as_str())
        .collect();
    assert_eq!(
        v7,
        [
            "'minecraft:azalea' | Invalid or incomplete texture list.",
            "'minecraft:flowering_azalea' | Invalid or incomplete texture list."
        ]
    );
    // The unread-member warning for the typo `sounds` of `coral_block`, the only unread member of the file.
    let l3: Vec<&Found> = found
        .iter()
        .filter(|f| f.code == "core/ignored_member" && f.at.contains('/'))
        .collect();
    assert_eq!(l3.len(), 1, "{l3:#?}");
    assert_eq!(
        (l3[0].at.as_str(), l3[0].severity),
        ("coral_block/sounds", I)
    );
    // The legacy-name warning for the 31 legacy keys that are not their own block, suppressed (debug) in a
    // base-game pack; `lava_cauldron` dropped; nothing at error level.
    let suppressed: Vec<&Found> = found
        .iter()
        .filter(|f| f.code == "blocks_json/old_name")
        .collect();
    assert_eq!(suppressed.len(), 31);
    assert!(
        suppressed
            .iter()
            .all(|f| f.severity == Severity::Debug && !f.text.is_empty())
    );
    assert!(found.contains(&ours("core/ignored_member", I, "lava_cauldron")));
    assert!(found.iter().all(|f| f.severity < E), "{:#?}", tally(&found));
    // The 13 legacy keys that are still blocks resolve directly.
    for name in [
        "stone",
        "prismarine",
        "anvil",
        "dirt",
        "quartz_block",
        "red_sandstone",
        "sand",
        "sandstone",
        "cobblestone_wall",
        "purpur_block",
        "sponge",
        "structure_void",
        "tnt",
    ] {
        assert!(loaded.get(name).is_some(), "{name}");
    }
    assert!(
        loaded.get("granite").is_some(),
        "the flattened names are in the file"
    );
    assert!(loaded.get("wool").is_none());
    // `brown_mushroom_block` < `mushroom_stem` < `red_mushroom_block`: the stem ends with the
    // red block's textures.
    assert_eq!(
        loaded.block("mushroom_stem").textures,
        loaded.block("red_mushroom_block").textures
    );
    assert_ne!(
        loaded.block("mushroom_stem").textures,
        loaded.block("brown_mushroom_block").textures
    );
    // 1,352 entries: the 31 old names and `lava_cauldron` apply to nothing; every other entry
    // is one block (the stem is fed by its own entry and by both mushroom blocks).
    assert_eq!(loaded.registry().len(), 1352 - 31 - 1);
    assert_eq!(found.len(), 31 + 2 + 1 + 1 + 2, "{:#?}", tally(&found));
}

#[test]
fn vanilla_s_legacy_file_is_split() {
    const VERSION_FINDINGS: &[&str] = &[
        "animation_controller/ill_formatted_missing_format_version",
        "block_culling/missing_object_malformed_format_version",
        "block_culling/missing_object_missing_format_version",
        "blocks_json/version_invalid",
        "blocks_json/version_not_given",
        "client_biome/no_valid_version",
        "core/format_version_required",
        "core/malformed_format_version",
        "core/malformed_format_version_worded",
        "core/missing_format_version",
        "core/missing_format_version_worded",
        "core/no_format_version_field",
        "core/no_valid_version",
        "core/version_not_given_legacy_fields",
        "core/version_not_processable",
        "core/version_not_readable",
        "ddui/malformed_version",
        "deferred_settings/malformed_version",
        "deferred_settings/member_count_without_version",
        "dialogue/no_format_version_malformed_format_version",
        "dialogue/no_format_version_missing_format_version",
        "fog/invalid_version",
        "geometry/version_not_valid",
        "item/version_not_readable",
        "jigsaw/bad_format_version_string",
        "recipe/missing_version_tag",
        "render_controller/required_child",
        "texture_set/set_bad_version",
    ];
    // The retail game's `resource_packs/vanilla/blocks.json`, `"format_version": [1, 1, 0]`
    let text = fixture("bj02_vanilla_legacy/packs/00_vanilla/blocks.json");
    let loaded = load(&[base(&text)]);
    let found = loaded.found();
    assert_eq!(
        loaded.block("stone").version().effective.to_string(),
        "1.1.0"
    );
    assert_eq!(
        loaded
            .block("stone")
            .version()
            .written
            .map(|v| v.to_string())
            .as_deref(),
        Some("1.1.0")
    );
    // The version is given, as an array: no version warnings, hence no legacy-name finding, and no legacy-name warning below the gates.
    assert!(
        found.iter().all(|f| !VERSION_FINDINGS.contains(&f.code)),
        "{:#?}",
        tally(&found)
    );
    for kind in [Kind::UnsupportedVersion, Kind::Deprecated] {
        assert!(
            loaded.registry().diagnostics().all(|f| f.kind() != kind),
            "{kind:?}"
        );
    }
    // `wool` → `white_wool: "wool:0"` … `black_wool: "wool:15"`.
    assert_eq!(loaded.textures("white_wool"), six("wool:0"));
    assert_eq!(loaded.textures("black_wool"), six("wool:15"));
    assert!(loaded.get("wool").is_none());
    assert!(found.iter().all(|f| f.severity < E), "{:#?}", tally(&found));
    // The typo `sounds` of `coral_block`, `lava_cauldron` dropped, and the two
    // mushroom blocks feeding the stem.
    assert_eq!(
        tally(&found),
        [
            (("core/ignored_member", "info".to_owned()), 2),
            (("core/shadowed_definition", "info".to_owned()), 2),
        ]
        .into_iter()
        .collect()
    );
    assert_eq!(loaded.registry().len(), 607);
}

#[test]
fn an_old_name_in_a_base_game_pack_is_kept_at_debug_level_deviation() {
    // `vanilla = "logged"`: the client logs the legacy-name warning for its own files too; there it is noise. The
    // entry is skipped either way.
    let entry = file(r#""wool": {"textures": "w"}"#);
    let in_base = load(&[base(&entry)]);
    assert!(in_base.registry().is_empty());
    let found = in_base.found();
    assert_eq!(found.len(), 1);
    assert_eq!(
        (found[0].code, found[0].severity),
        ("blocks_json/old_name", Severity::Debug)
    );
    assert!(
        found[0]
            .text
            .starts_with("blocks.json file contains a block using an old name")
    );
    assert_eq!(in_base.problems(), []);
    let in_user = load(&[user(&entry)]);
    assert!(in_user.registry().is_empty());
    let found = in_user.found();
    assert_eq!(
        (found[0].code, found[0].severity),
        ("blocks_json/old_name", E)
    );
    assert_eq!(found[0].text, in_base.found()[0].text);
}

#[test]
fn a_higher_pack_overrides_field_by_field() {
    let loaded = load(&[
        base(&file(
            r#""obsidian": {"textures": "a", "sound": "stone", "isotropic": true}"#,
        )),
        user(&file(
            r#""obsidian": {"textures": {"up": "b", "down": "b", "side": "c"}}"#,
        )),
        user(&file(r#""obsidian": {"sound": "wood", "textures": 5}"#)),
    ]);
    let obsidian = loaded.block("obsidian");
    // Pack 2's invalid `textures` leaves pack 1's in place.
    assert_eq!(
        obsidian.textures,
        Some(faces(["b", "b", "c", "c", "c", "c"]))
    );
    assert_eq!(obsidian.sound.as_deref(), Some("wood"));
    assert_eq!(obsidian.isotropic_mask(), 0xff, "from pack 0");
    assert_eq!(
        loaded.found(),
        [vanilla(
            "blocks_json/bad_textures",
            W,
            "obsidian/textures",
            "'minecraft:obsidian' | Invalid or incomplete texture list."
        )
        .in_pack(2)]
    );
    // The provenance: the top pack's entry, the lower ones as contributors.
    let origin = obsidian.origin();
    assert_eq!(origin.pack.name(), "rp 2");
    assert_eq!(origin.path.as_str(), "blocks.json");
    assert_eq!(origin.key.as_deref(), Some("obsidian"));
    let mut contributors: Vec<&str> = origin
        .contributors
        .iter()
        .map(bedrock_addon::pack::PackRef::name)
        .collect();
    contributors.sort_unstable();
    assert_eq!(contributors, ["rp 0", "rp 1"]);
    assert_eq!(loaded.registry().len(), 1);
    assert!(loaded.registry().rejected().is_empty());
}

#[test]
fn every_field_merges_on_its_own_over_three_packs() {
    type Replaced = fn(&ClientBlock) -> bool;

    let bottom = file(
        r#""obsidian": {"textures": "t0", "carried_textures": "c0", "isotropic": true,
            "blockshape": "cross_texture", "ambient_occlusion_exponent": 0.25, "sound": "s0",
            "brightness_gamma": 1}"#,
    );
    // Each field alone in the top pack replaces only itself.
    let cases: [(&str, Replaced); 6] = [
        (r#""textures": "t2""#, |b| b.face_textures() == ["t2"; 6]),
        (r#""carried_textures": "c2""#, |b| {
            b.carried_face_textures() == ["c2"; 6]
        }),
        (r#""isotropic": false"#, |b| b.isotropic_mask() == 0),
        (r#""blockshape": "torch""#, |b| {
            b.blockshape.as_deref() == Some("torch")
        }),
        (r#""ambient_occlusion_exponent": 4"#, |b| {
            b.ambient_occlusion_exponent == Some(4.0)
        }),
        (r#""sound": "s2""#, |b| b.sound.as_deref() == Some("s2")),
    ];
    for (index, (member, replaced)) in cases.iter().enumerate() {
        let loaded = load(&[
            user(&bottom),
            user(&file(r#""glass": {"textures": "unrelated"}"#)),
            user(&file(&format!(r#""obsidian": {{{member}}}"#))),
        ]);
        let block = loaded.block("obsidian");
        assert!(replaced(block), "{member}: {block:#?}");
        let unchanged = [
            block.face_textures() == ["t0"; 6],
            block.carried_face_textures() == ["c0"; 6],
            block.isotropic_mask() == 0xff,
            block.blockshape.as_deref() == Some("cross_texture"),
            block.ambient_occlusion_exponent == Some(0.25),
            block.sound.as_deref() == Some("s0"),
        ];
        for (other, same) in unchanged.iter().enumerate() {
            assert_eq!(*same, other != index, "{member} and field {other}");
        }
        assert!(
            block.extra.get("brightness_gamma").is_some(),
            "unread members are kept"
        );
        assert_eq!(block.origin().pack.name(), "rp 2");
    }
    // `textures` replaces all six faces at once, and `isotropic` the whole mask.
    let loaded = load(&[
        user(&file(
            r#""obsidian": {"textures": {"up": "u", "down": "d", "side": "s"}, "isotropic": {"up": true, "down": true}}"#,
        )),
        user(&file(
            r#""obsidian": {"textures": "all", "isotropic": {"north": true}}"#,
        )),
    ]);
    assert_eq!(loaded.textures("obsidian"), six("all"));
    assert_eq!(loaded.block("obsidian").isotropic_mask(), 0x04);
    // A `null` field in a higher pack changes nothing.
    let loaded = load(&[
        user(&bottom),
        user(&file(
            r#""obsidian": {"textures": null, "carried_textures": null, "isotropic": null, "blockshape": null, "ambient_occlusion_exponent": null, "sound": null}"#,
        )),
    ]);
    let block = loaded.block("obsidian");
    assert_eq!(block.face_textures(), ["t0"; 6]);
    assert_eq!(block.sound(), "s0");
    assert_eq!(block.blockshape.as_deref(), Some("cross_texture"));
    assert_eq!(block.isotropic_mask(), 0xff);
}

#[test]
fn a_pack_without_the_block_or_without_the_file_takes_no_part() {
    let loaded = load(&[
        user(&file(r#""obsidian": {"textures": "low"}"#)),
        user(&file(r#""glass": {"textures": "g"}"#)),
    ]);
    assert_eq!(loaded.textures("obsidian"), six("low"));
    assert_eq!(loaded.block("obsidian").origin().pack.name(), "rp 0");
    assert!(loaded.block("obsidian").origin().contributors.is_empty());
    // A stack without any `blocks.json` has no client blocks: a custom block without an
    // entry gets no graphics from this loader.
    let empty = PackStack::<Resource>::of(
        [rp(1).file("textures/terrain_texture.json", b"{}").open()],
        LoadOptions::default(),
    )
    .unwrap();
    assert!(empty.client_blocks().is_empty());
    assert_eq!(empty.client_blocks().diagnostics().count(), 0);
}

#[test]
fn a_file_that_is_not_json_is_refused_with_an_error_deviation() {
    // `vanilla = "silent"`: the lower pack's values survive unchanged. The file is
    // refused, so the finding is an error.
    let valid = file(r#""obsidian": {"textures": "a", "sound": "stone"}"#);
    for broken in [
        r#"{"format_version": "1.21.40", "obsidian": {"textures": "b""#,
        "",
        "[]",
        "5",
        r#""blocks""#,
        "{\"a\": }",
    ] {
        let loaded = load(&[user(&valid), user(broken)]);
        assert_eq!(loaded.textures("obsidian"), six("a"), "{broken:?}");
        assert_eq!(loaded.block("obsidian").sound(), "stone");
        let found = loaded.found();
        assert_eq!(found.len(), 1, "{broken:?}: {found:#?}");
        assert_eq!(
            (
                found[0].code,
                found[0].severity,
                found[0].pack.as_str(),
                found[0].at.as_str()
            ),
            ("blocks_json/not_json", I, "rp 1", ""),
            "{broken:?}"
        );
        // The text is ours, with the reason as its one argument: which reason a file is
        // refused for is the reader's own variant (`resolve::tests`), not a wording.
        let finding = loaded.registry().diagnostics().next().cloned();
        assert!(
            finding.as_ref().is_some_and(|finding| {
                let message = finding.syntax_message().unwrap_or_default();
                finding.is(&NOT_JSON)
                    && NOT_JSON.template()
                        == "blocks.json is not valid JSON ({}); clients ignore this file"
                    && finding.code.args_of(&NOT_JSON) == Some(&[message.to_owned()][..])
                    && !message.is_empty()
            }),
            "{broken:?}: {finding:?}"
        );
        assert_eq!(loaded.registry().rejected().len(), 1);
        assert_eq!(loaded.registry().rejected()[0].origin.pack.name(), "rp 1");
    }
    // The other way round: a broken lower file takes nothing from the higher one.
    let loaded = load(&[user("{"), user(&valid)]);
    assert_eq!(loaded.textures("obsidian"), six("a"));
    // Bytes that are not UTF-8, and nesting beyond the limit.
    let deep = format!(r#"{{"a": {}{}}}"#, "[".repeat(200), "]".repeat(200));
    let loaded = load(&[user(&valid), user(&deep)]);
    assert_eq!(
        (loaded.found()[0].code, loaded.found()[0].severity),
        ("blocks_json/not_json", I)
    );
    assert_eq!(
        loaded.found()[0].text,
        "blocks.json is not valid JSON (nested too deeply); clients ignore this file"
    );
    assert_eq!(loaded.textures("obsidian"), six("a"));
    // The file type refuses the same file the same way.
    let refused = BlocksJson::parse(
        &bedrock_addon::storage::PackPath::from_static("blocks.json"),
        b"[]",
        &bedrock_addon::load::LoadContext::detached(),
    )
    .unwrap_err();
    // refused, with the crate's information (the client is silent)
    assert!(refused.diagnostics.iter().next().is_some());
}

#[test]
fn a_byte_order_mark_and_comments_are_read_through() {
    let text = format!(
        "\u{feff}// a comment\n{}",
        file(r#""obsidian": /* inline */ {"textures": "a"}"#)
    );
    let loaded = load_one(&text);
    assert_eq!(loaded.textures("obsidian"), six("a"));
    assert_eq!(loaded.found(), []);
}

#[test]
fn only_blocks_json_at_the_pack_root_is_read() {
    let entry = leak(&file(r#""obsidian": {"textures": "a"}"#));
    let stack = PackStack::<Resource>::of(
        [rp(1)
            .file("Blocks.json", entry)
            .file("blocks.JSON", entry)
            .file("textures/blocks.json", entry)
            .file("blocks/blocks.json", entry)
            .file("blocks.json5", entry)
            .open()],
        LoadOptions::default(),
    )
    .unwrap();
    assert!(stack.client_blocks().is_empty());
    assert!(std::ptr::eq(stack.client_blocks(), stack.client_blocks()));
    assert_eq!(BlocksJson::new().default_path().as_str(), "blocks.json");
}

#[test]
fn a_subpack_s_file_replaces_its_pack_s() {
    // One file per stack entry: the active subpack's copy shadows the pack's own (inferred; the loader's walk keys a subpack file by its path in the pack).
    use bedrock_addon::pack::{PackOrigin, PackRepository, PackSelection, WorldPacks};
    let spec = rp(1)
        .subpack("hd", "HD", 1)
        .file(
            "blocks.json",
            leak(&file(
                r#""obsidian": {"textures": "pack", "sound": "pack"}"#,
            )),
        )
        .file(
            "subpacks/hd/blocks.json",
            leak(&file(r#""obsidian": {"textures": "subpack"}"#)),
        );
    let mut repository = PackRepository::new();
    repository.add_pack(spec.open(), PackOrigin::User);
    let id = spec.id();
    let world = WorldPacks {
        behavior: Vec::new(),
        resource: vec![PackSelection {
            pack_id: id.uuid,
            version: id.version,
            subpack: Some("hd".to_owned()),
        }],
    };
    let set = repository.stacks(&world, LoadOptions::default());
    let registry = set.resource.client_blocks();
    let block = registry.get(&super::common::block_id("obsidian")).unwrap();
    assert_eq!(block.face_textures(), ["subpack"; 6]);
    assert_eq!(block.sound(), "normal", "the pack's own file is not read");
    assert_eq!(block.origin().path.as_str(), "subpacks/hd/blocks.json");
}

/// A file with `count` entries: custom blocks, vanilla names, and one legacy key.
fn many_entries(count: usize) -> String {
    let mut entries: Vec<String> = (0..count)
        .map(|n| {
            format!(
                r#""demo:block_{n}": {{"textures": {{"up": "u{n}", "down": "d{n}", "side": "s{n}"}}, "sound": "stone", "isotropic": {{"up": true, "down": true}}, "ambient_occlusion_exponent": 0.5}}"#
            )
        })
        .collect();
    entries.push(r#""obsidian": {"textures": "o", "blockshape": "cross_texture"}"#.to_owned());
    file(&entries.join(", "))
}

/// A stack of two packs with many entries each: every block is merged from both, in order.
/// (What a load costs is measured where it could be more than linear, on the functions
/// themselves: `src/block/client/cost.rs`.)
#[test]
fn two_packs_of_many_entries_merge_block_by_block() {
    let count = 300;
    let text = many_entries(count);
    let loaded = load(&[user(&text), user(&text)]);
    assert_eq!(loaded.registry().len(), count + 1);
    for n in [0, count / 2, count - 1] {
        let block = loaded.block(&format!("demo:block_{n}"));
        assert_eq!(block.origin().contributors.len(), 1);
        assert_eq!(block.face_textures()[0], format!("u{n}"));
    }
    assert_eq!(loaded.registry().diagnostics().count(), 0);
}

/// A file whose every entry is refused in some field, and one key repeated many times: one
/// finding per refused field, one per earlier value of the key.
#[test]
fn many_refused_entries_and_a_key_repeated_many_times() {
    let count = 300;
    let refused: Vec<String> = (0..count)
        .map(|n| format!(r#""demo:b{n}": {{"textures": 5, "sound": 5, "brightness_gamma": 1}}"#))
        .collect();
    let loaded = load_one(&file(&refused.join(", ")));
    assert_eq!(loaded.registry().diagnostics().count(), 3 * count);
    assert_eq!(loaded.registry().len(), count);
    let repeated = vec![r#""obsidian": {"textures": "a"}"#; count];
    let loaded = load_one(&file(&repeated.join(", ")));
    assert_eq!(loaded.registry().diagnostics().count(), count - 1);
    assert_eq!(loaded.textures("obsidian"), six("a"));
}

#[test]
fn a_file_version_is_each_file_s_own() {
    // The gates compare each file's own version: a 1.1.0 file below a 1.21.40 file.
    let loaded = load(&[
        user(&file_at("[1, 1, 0]", r#""wool": {"textures": "old"}"#)),
        user(&file(
            r#""white_wool": {"sound": "cloth"}, "wool": {"textures": "new"}"#,
        )),
    ]);
    let white = loaded.block("white_wool");
    assert_eq!(white.face_textures(), ["old:0"; 6]);
    assert_eq!(white.sound(), "cloth");
    assert_eq!(
        white.version().effective.to_string(),
        "1.21.40",
        "the top contribution's"
    );
    assert_eq!(loaded.textures("black_wool"), six("old:15"));
    let found = loaded.problems();
    assert_eq!(found.len(), 1);
    assert_eq!(
        (found[0].code, found[0].severity, found[0].pack.as_str()),
        ("blocks_json/old_name", E, "rp 1")
    );
}

#[test]
fn content_after_the_root_object_is_ignored_with_a_warning_at_its_span() {
    // Vanilla reads the first value and stops; the file loads, and the content is reported
    // once, where it is.
    let valid = file(r#""obsidian": {"textures": "a"}"#);
    let text = format!("{valid}\n garbage\n");
    let loaded = load_one(&text);
    assert_eq!(loaded.textures("obsidian"), six("a"));
    let found: Vec<_> = loaded.registry().diagnostics().collect();
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(found[0].code.id(), "core/syntax");
    // Ours: a JsonCpp reader stops at the root value in silence.
    assert_eq!(found[0].severity, Severity::Info);
    assert!(found[0].location.pack.is_some() && found[0].location.path.is_some());
    let span = found[0].location.span.expect("the content's span");
    let at = span.offset as usize;
    assert_eq!(&text[at..at + span.len as usize], "garbage");
    // A comment after the root is no content.
    let loaded = load_one(&format!("{valid}\n// the end\n"));
    assert_eq!(loaded.registry().diagnostics().count(), 0);
    // The file type reports it the same way.
    let document = BlocksJson::parse(
        &bedrock_addon::storage::PackPath::from_static("blocks.json"),
        text.as_bytes(),
        &bedrock_addon::load::LoadContext::detached(),
    )
    .unwrap();
    assert_eq!(document.diagnostics().len(), 1);
}

#[test]
fn a_second_object_after_the_root_is_content_after_it_and_no_repeated_key() {
    // The client's reader stops at the end of the root object: what follows is reported once,
    // as content after the root, and its keys are not keys of the file.
    for text in [
        r#"{"obsidian":{}} {"obsidian":{}}"#,
        r#"{"obsidian":{"textures":"a"}} // one
 {"obsidian":{}, "obsidian":{}}"#,
    ] {
        let loaded = load_one(text);
        let found: Vec<(&str, String)> = loaded
            .registry()
            .diagnostics()
            .filter(|f| f.severity >= Severity::Info)
            .map(|f| (f.code.id(), f.location.json_path.to_string()))
            .collect();
        assert_eq!(
            found,
            [
                ("core/syntax", String::new()),
                ("blocks_json/version_not_given", String::new())
            ],
            "{text}"
        );
    }
    // A key repeated inside the root is still counted, with content after the root or not.
    let loaded = load_one(r#"{"obsidian":{}, "obsidian":{}} {"obsidian":{}}"#);
    let repeated = loaded
        .registry()
        .diagnostics()
        .filter(|f| f.code.id() == "core/duplicate_identifier")
        .count();
    assert_eq!(repeated, 1);
}
