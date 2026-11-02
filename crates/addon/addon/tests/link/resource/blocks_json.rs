//! 20 R27, R28 and R29: what `blocks.json` says of blocks, of the terrain atlas and of the
//! block sound types; and the rules of the `blocks_json` folder that need both stacks: an
//! `ambient_occlusion_exponent` beside `minecraft:material_instances` (V11) and the
//! tessellation fallback of Upcoming Creator Features (V12, V13, L13).

use bedrock_addon::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticDetail, Diagnostics, Phase, ReferenceHint, Severity,
};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::LinkOptions;

// The material and atlas files of these worlds are a pack the game ships: the rows
// are tested at their own severity. `tests/link/view.rs` has the other views.
use crate::common::{
    behavior_pack, game_link as link, game_link_with as link_with, hosted, resource_pack, stacks_of,
};

const BLOCKS: &str = "blocks.json";

/// A `blocks.json` at 1.21.40 with `entries`.
fn file(entries: &str) -> String {
    format!(r#"{{"format_version": "1.21.40", {entries}}}"#)
}

/// A custom block `id` with `components`.
fn block(id: &str, components: &str) -> String {
    format!(
        r#"{{"format_version": "1.26.20", "minecraft:block": {{"description": {{"identifier": "{id}"}}, "components": {{{components}}}}}}}"#
    )
}

/// The components of a block drawn from a geometry: from 1.21.80 a block has both or neither.
const DRAWN: &str = r#""minecraft:geometry": "minecraft:geometry.full_block", "minecraft:material_instances": {"*": {"texture": "lamp"}}"#;

/// Everything a finding says: code, severity, phase, pack, file, JSON path, and the kind and
/// the name of its reference (empty without one).
type Said = (
    &'static str,
    Severity,
    Phase,
    String,
    String,
    String,
    &'static str,
    String,
);

fn said(finding: &Diagnostic) -> Said {
    let (kind, id) = match finding.detail.as_deref() {
        Some(DiagnosticDetail::Reference { kind, id, .. }) => (*kind, id.clone()),
        _ => ("", String::new()),
    };
    (
        finding.code.name(),
        finding.severity,
        finding.phase,
        finding
            .location
            .pack
            .as_ref()
            .map(|pack| pack.name().to_owned())
            .unwrap_or_default(),
        finding
            .location
            .path
            .as_ref()
            .map(|path| path.as_str().to_owned())
            .unwrap_or_default(),
        finding.location.json_path.to_string(),
        kind,
        id,
    )
}

fn hint(finding: &Diagnostic) -> Option<&ReferenceHint> {
    match finding.detail.as_deref()? {
        DiagnosticDetail::Reference { hint, .. } => hint.as_ref(),
        _ => None,
    }
}

/// The findings of `blocks.json` whose code is called `code`.
fn of<'a>(found: &'a Diagnostics, code: &str) -> Vec<&'a Diagnostic> {
    found
        .iter()
        .filter(|finding| {
            finding.code.name() == code
                && finding
                    .location
                    .path
                    .as_ref()
                    .is_some_and(|path| path.as_str() == BLOCKS)
        })
        .collect()
}

/// V5's text for `key`.
fn no_such_block(key: &str) -> String {
    format!("The block named {key} used in a \"blocks.json\" file does not exist in the registry")
}

// ---- R27 --------------------------------------------------------------------------------------

#[test]
fn r27_a_key_of_a_block_a_pack_defines_is_no_finding() {
    let found = link(
        &[("blocks/lamp.json", &block("demo:lamp", DRAWN))],
        &[(BLOCKS, &file(r#""demo:lamp": {"sound": "glass"}"#))],
    );
    assert!(of(&found, "UnresolvedReference").is_empty(), "{found:#?}");
}

#[test]
fn r27_a_key_of_a_block_of_the_game_is_no_finding_with_a_host_and_without() {
    let files = [(BLOCKS, file(r#""dirt": {"sound": "gravel"}"#))];
    let files: Vec<(&'static str, &str)> = files.iter().map(|(p, t)| (*p, t.as_str())).collect();
    // Without a host the crate's own list of the game's blocks answers.
    let found = link(&[], &files);
    assert!(of(&found, "UnresolvedReference").is_empty(), "{found:#?}");
    // A host is asked by the block's full name.
    let found = link_with(
        &[],
        &files,
        hosted(&[("block", "minecraft:dirt")]),
        LinkOptions::default(),
    );
    assert!(of(&found, "UnresolvedReference").is_empty(), "{found:#?}");
    // And it is believed when it says no.
    let found = link_with(&[], &files, hosted(&[]), LinkOptions::default());
    let missing = of(&found, "UnresolvedReference");
    assert_eq!(missing.len(), 1, "{found:#?}");
    assert_eq!(missing[0].code.to_string(), no_such_block("dirt"));
}

#[test]
fn r27_a_key_nothing_defines_is_a_warning_at_the_key() {
    let found = link(
        &[("blocks/lamp.json", &block("demo:lamp", DRAWN))],
        &[(BLOCKS, &file(r#""demo:ghost": {}, "demo:lamp": {}"#))],
    );
    let missing = of(&found, "UnresolvedReference");
    assert_eq!(missing.len(), 1, "{found:#?}");
    assert_eq!(
        said(missing[0]),
        (
            "UnresolvedReference",
            Severity::Warning,
            Phase::Link,
            "rp 2".to_owned(),
            BLOCKS.to_owned(),
            "demo:ghost".to_owned(),
            "block",
            "demo:ghost".to_owned()
        )
    );
    assert_eq!(missing[0].code.to_string(), no_such_block("demo:ghost"));
    assert_eq!(hint(missing[0]), None);
    // A warning is no lint: it stays without the lints.
    let found = link_with(
        &[],
        &[(BLOCKS, &file(r#""demo:ghost": {}"#))],
        LoadOptions::default(),
        LinkOptions {
            lints: false,
            ..Default::default()
        },
    );
    assert_eq!(of(&found, "UnresolvedReference").len(), 1, "{found:#?}");
}

#[test]
fn r27_a_bare_key_is_a_block_of_the_game_in_lower_case() {
    // `minecraft:` is added to a key without a namespace, and the key is lower-cased; a key
    // in the `minecraft` namespace is lower-cased whole.
    let found = link(
        &[],
        &[(
            BLOCKS,
            &file(r#""Dirt": {}, "minecraft:Gravel": {}, "minecraft:sand": {}, "Nope": {}"#),
        )],
    );
    let missing = of(&found, "UnresolvedReference");
    assert_eq!(missing.len(), 1, "{found:#?}");
    // The key is told as the file has it.
    assert_eq!(missing[0].location.json_path.to_string(), "Nope");
    assert_eq!(missing[0].code.to_string(), no_such_block("Nope"));
    assert_eq!(said(missing[0]).6, "block");
    assert_eq!(said(missing[0]).7, "Nope");
}

#[test]
fn r27_a_key_of_another_namespace_is_compared_as_written() {
    // A pack's block is found by its name as written: another case misses, with the hint;
    // and a bare key is the game's, never the pack's.
    let found = link(
        &[("blocks/lamp.json", &block("demo:lamp", DRAWN))],
        &[(
            BLOCKS,
            &file(r#""demo:Lamp": {}, "lamp": {}, "demo:lamp": {}"#),
        )],
    );
    let missing = of(&found, "UnresolvedReference");
    let paths: Vec<String> = missing
        .iter()
        .map(|finding| finding.location.json_path.to_string())
        .collect();
    assert_eq!(paths, ["demo:Lamp", "lamp"], "{found:#?}");
    assert_eq!(
        hint(missing[0]),
        Some(&ReferenceHint::CaseMismatch("demo:lamp".to_owned()))
    );
    assert_eq!(hint(missing[1]), None);
}

#[test]
fn r27_a_name_the_client_still_answers_to_is_a_block_with_any_host() {
    // `seaLantern` is a key of the game's own file: the client resolves it, the list of the
    // game's blocks has `sea_lantern` alone.
    let entries = file(r#""seaLantern": {"sound": "glass"}"#);
    let found = link(&[], &[(BLOCKS, &entries)]);
    assert!(of(&found, "UnresolvedReference").is_empty(), "{found:#?}");
    let found = link_with(
        &[],
        &[(BLOCKS, &entries)],
        hosted(&[]),
        LinkOptions::default(),
    );
    assert!(of(&found, "UnresolvedReference").is_empty(), "{found:#?}");
    // The name is the game's alone.
    let found = link(&[], &[(BLOCKS, &file(r#""demo:seaLantern": {}"#))]);
    assert_eq!(of(&found, "UnresolvedReference").len(), 1, "{found:#?}");
}

#[test]
fn r27_a_key_two_packs_write_is_told_once_at_the_higher_pack() {
    let entries = file(r#""demo:ghost": {"sound": "glass"}"#);
    let set = stacks_of(
        Vec::new(),
        vec![
            resource_pack(3, &[(BLOCKS, &entries)]),
            resource_pack(2, &[(BLOCKS, &entries)]),
        ],
        LoadOptions::default(),
    );
    let found = set.link();
    let missing = of(&found, "UnresolvedReference");
    assert_eq!(missing.len(), 1, "{found:#?}");
    assert_eq!(said(missing[0]).3, "rp 3");
}

// ---- R28 --------------------------------------------------------------------------------------

#[cfg(feature = "texture_atlas")]
mod textures {
    use super::*;

    const ATLAS: &str = "textures/terrain_texture.json";
    /// A terrain atlas: `one` has one element, `two` two, `none` none, `plain` is no array.
    const TERRAIN: &str = r#"{"resource_pack_name": "t", "texture_name": "atlas.terrain", "texture_data": {
        "one": {"textures": ["textures/a"]}, "two": {"textures": ["textures/a", "textures/b"]},
        "none": {"textures": []}, "plain": {"textures": "textures/p"}}}"#;

    fn no_elements(name: &str) -> String {
        format!("Terrain texture <{name}> doesn't have any elements. Using the missing texture.")
    }

    /// The atlas's developer-log line for a tile it lacks.
    fn no_such_item(tile: &str) -> String {
        format!("This texture item doesn't exist: Texture [{tile}]")
    }

    /// The findings about a tile of the terrain atlas.
    fn tiles(found: &Diagnostics) -> Vec<&Diagnostic> {
        of(found, "UnresolvedReference")
            .into_iter()
            .filter(|finding| said(finding).6 == "atlas key")
            .collect()
    }

    /// The custom blocks the entries of these tests are written for: a `blocks.json` entry of
    /// a block the registry lacks is dropped before its textures are read.
    const DEMO_BLOCKS: [(&str, &str); 6] = [
        ("blocks/a.json", "demo:a"),
        ("blocks/b.json", "demo:b"),
        ("blocks/c.json", "demo:c"),
        ("blocks/in.json", "demo:in"),
        ("blocks/out.json", "demo:out"),
        ("blocks/far.json", "demo:far"),
    ];

    /// The behaviour files of [`DEMO_BLOCKS`].
    fn demo_blocks() -> Vec<(&'static str, String)> {
        DEMO_BLOCKS
            .iter()
            .map(|(path, id)| (*path, block(id, "")))
            .collect()
    }

    fn linked(entries: &str) -> Diagnostics {
        let behavior = demo_blocks();
        let behavior: Vec<(&'static str, &str)> = behavior
            .iter()
            .map(|(path, text)| (*path, text.as_str()))
            .collect();
        link(&behavior, &[(BLOCKS, &file(entries)), (ATLAS, TERRAIN)])
    }

    #[test]
    fn r28_an_entry_of_a_block_the_registry_lacks_has_no_texture_finding() {
        // The game drops the entry at V5's message, before any texture is
        // looked up (1.26.60): V5 alone is told.
        let found = link(
            &[],
            &[
                (BLOCKS, &file(r#""demo:nowhere": {"textures": "ghost"}"#)),
                (ATLAS, TERRAIN),
            ],
        );
        assert!(tiles(&found).is_empty(), "{found:#?}");
        assert_eq!(of(&found, "UnresolvedReference").len(), 1, "{found:#?}");
    }

    #[test]
    fn r28_a_miss_is_graded_by_its_kind_whatever_the_block() {
        // Loading a block's graphics resolves its textures: a tile the
        // atlas lacks is the atlas's developer-log line, an information; a tile without
        // elements is the content-log warning, for a
        // game block as for a block of the stack.
        for (entries, behavior) in [
            (
                r#""minecraft:stone": {"textures": "ghost"}, "minecraft:dirt": {"textures": "none"}"#,
                false,
            ),
            (
                r#""demo:a": {"textures": "ghost"}, "demo:b": {"textures": "none"}"#,
                true,
            ),
        ] {
            let found = if behavior {
                linked(entries)
            } else {
                link(&[], &[(BLOCKS, &file(entries)), (ATLAS, TERRAIN)])
            };
            let mut told: Vec<(String, Severity)> = tiles(&found)
                .iter()
                .map(|f| (f.code.to_string(), f.severity))
                .collect();
            told.sort();
            assert_eq!(
                told,
                [
                    (no_elements("none"), Severity::Warning),
                    (no_such_item("ghost"), Severity::Info),
                ],
                "{entries}"
            );
        }
    }

    #[test]
    fn r28_a_tile_of_the_atlas_is_no_finding() {
        let found = linked(
            r#""demo:a": {"textures": "one", "carried_textures": "plain"},
               "demo:b": {"textures": {"up": "two:1", "down": "two:0", "side": "one:0"}},
               "demo:c": {"textures": "plain:0"}"#,
        );
        let told: Vec<Said> = found
            .iter()
            .filter(|f| f.location.json_path.to_string().contains("textures"))
            .map(said)
            .collect();
        assert!(told.is_empty(), "{told:#?}");
    }

    #[test]
    fn r28_a_tile_the_atlas_lacks_is_an_information_at_the_member() {
        // The game logs it to the developer log and hands back the
        // `missing` item.
        let found = linked(r#""demo:a": {"textures": "ghost"}"#);
        let missing = tiles(&found);
        assert_eq!(missing.len(), 1, "{found:#?}");
        assert_eq!(
            said(missing[0]),
            (
                "UnresolvedReference",
                Severity::Info,
                Phase::Link,
                "rp 2".to_owned(),
                BLOCKS.to_owned(),
                "demo:a/textures".to_owned(),
                "atlas key",
                "ghost".to_owned()
            )
        );
        assert_eq!(missing[0].code.to_string(), no_such_item("ghost"));
    }

    #[test]
    fn r28_carried_textures_are_checked_too_and_each_tile_is_told_once_per_member() {
        let found = linked(
            r#""demo:a": {"textures": {"up": "ghost", "down": "ghost", "side": "one"},
                          "carried_textures": {"up": "one", "down": "none", "north": "ghost",
                                               "south": "ghost", "west": "one", "east": "one"}}"#,
        );
        let told: Vec<(String, String)> = tiles(&found)
            .iter()
            .map(|f| (said(f).5, f.code.to_string()))
            .collect();
        assert_eq!(
            told,
            [
                ("demo:a/textures".to_owned(), no_such_item("ghost")),
                ("demo:a/carried_textures".to_owned(), no_elements("none")),
                ("demo:a/carried_textures".to_owned(), no_such_item("ghost")),
            ],
            "{found:#?}"
        );
    }

    #[test]
    fn r28_an_element_is_inside_its_tile_or_the_first_is_drawn() {
        // `two` has the elements 0 and 1: `two:1` is the last one inside, `two:2` the first
        // one beyond. The finding is ours, an information: the client draws element 0
        // without a word.
        let found = linked(
            r#""demo:in": {"textures": "two:1"}, "demo:out": {"textures": "two:2"},
               "demo:far": {"textures": "one:99999999999999999999999999"}"#,
        );
        let told: Vec<Said> = tiles(&found).iter().map(|f| said(f)).collect();
        let paths: Vec<(&str, &str)> = told
            .iter()
            .map(|said| (said.5.as_str(), said.7.as_str()))
            .collect();
        assert_eq!(
            paths,
            [
                ("demo:far/textures", "one:99999999999999999999999999"),
                ("demo:out/textures", "two:2")
            ],
            "{found:#?}"
        );
        for finding in tiles(&found) {
            assert_eq!(finding.severity, Severity::Info, "{finding:#?}");
            // Ours: no text of vanilla's.
            assert!(
                matches!(
                    finding.code,
                    DiagnosticCode::UnresolvedReference { text: None }
                ),
                "{finding:#?}"
            );
        }
        // An element of a tile without elements, or of no tile, is the tile's miss.
        let found =
            linked(r#""demo:a": {"textures": "none:0"}, "demo:b": {"textures": "ghost:1"}"#);
        let texts: Vec<String> = tiles(&found).iter().map(|f| f.code.to_string()).collect();
        assert_eq!(
            texts,
            [no_elements("none:0"), no_such_item("ghost")],
            "{found:#?}"
        );
    }

    #[test]
    fn r28_a_name_is_compared_as_written_and_only_a_number_selects_an_element() {
        // The atlas key is exact: another case misses, with the tile it nearly is. A name
        // with a `:` that is not `<tile>:<digits>` is a tile's name whole.
        let found = linked(r#""demo:a": {"textures": "One"}, "demo:b": {"textures": "one:x"}"#);
        let missing = tiles(&found);
        assert_eq!(missing.len(), 2, "{found:#?}");
        assert_eq!(said(missing[0]).7, "One");
        assert_eq!(
            hint(missing[0]),
            Some(&ReferenceHint::CaseMismatch("one".to_owned()))
        );
        assert_eq!(missing[1].code.to_string(), no_such_item("one:x"));
    }

    #[test]
    fn r28_a_tile_of_the_game_is_a_host_s_to_know() {
        let entries = file(r#""demo:a": {"textures": "stone"}"#);
        let demo = block("demo:a", "");
        let behavior = [("blocks/a.json", demo.as_str())];
        // A host that has the tile: no finding, with the stack's own atlas or without one.
        let host = || hosted(&[("atlas key", "stone")]);
        for resource in [
            vec![(BLOCKS, entries.as_str()), (ATLAS, TERRAIN)],
            vec![(BLOCKS, entries.as_str())],
        ] {
            let found = link_with(&behavior, &resource, host(), LinkOptions::default());
            assert!(tiles(&found).is_empty(), "{found:#?}");
        }
        // A host that lacks it is believed.
        let found = link_with(
            &behavior,
            &[(BLOCKS, &entries)],
            hosted(&[]),
            LinkOptions::default(),
        );
        assert_eq!(tiles(&found).len(), 1, "{found:#?}");
        // Without a host the stack's atlas is all there is: the tile misses.
        let found = link(&behavior, &[(BLOCKS, &entries), (ATLAS, TERRAIN)]);
        assert_eq!(tiles(&found).len(), 1, "{found:#?}");
    }

    #[test]
    fn r28_without_a_terrain_atlas_and_without_a_host_nothing_is_compared() {
        let found = link(
            &[],
            &[(BLOCKS, &file(r#""demo:a": {"textures": "ghost"}"#))],
        );
        assert!(tiles(&found).is_empty(), "{found:#?}");
    }

    #[test]
    fn r28_a_legacy_key_split_onto_its_blocks_is_told_once_per_tile() {
        // Read as 1.1.0, `wool` is sixteen blocks with the textures `ghost:0` to `ghost:15`.
        let found = link(
            &[],
            &[
                (BLOCKS, r#"{"wool": {"textures": "ghost"}}"#),
                (ATLAS, TERRAIN),
            ],
        );
        let missing = tiles(&found);
        assert_eq!(missing.len(), 1, "{found:#?}");
        assert_eq!(said(missing[0]).5, "wool/textures");
    }
}

/// Without the `texture` feature there is no atlas in view and the rule does not run.
#[cfg(not(feature = "texture_atlas"))]
#[test]
fn r28_without_the_texture_feature_nothing_is_reported() {
    let found = link(
        &[],
        &[
            (BLOCKS, &file(r#""dirt": {"textures": "ghost"}"#)),
            (
                "textures/terrain_texture.json",
                r#"{"texture_name": "atlas.terrain", "texture_data": {}}"#,
            ),
        ],
    );
    let tiles = of(&found, "UnresolvedReference")
        .into_iter()
        .filter(|finding| said(finding).6 == "atlas key")
        .count();
    assert_eq!(tiles, 0, "{found:#?}");
}

// ---- R29 --------------------------------------------------------------------------------------

#[cfg(feature = "sounds_json")]
mod sounds {
    use super::*;

    const SOUNDS: &str = r#"{"block_sounds": {"normal": {"events": {}}, "Stone": {"events": {}}}}"#;

    fn unknown(entries: &str, sounds: &str) -> Vec<Said> {
        let found = link(&[], &[(BLOCKS, &file(entries)), ("sounds.json", sounds)]);
        of(&found, "UnknownSound").into_iter().map(said).collect()
    }

    #[test]
    fn r29_a_block_sound_type_of_sounds_json_is_no_finding() {
        assert_eq!(
            unknown(r#""dirt": {"sound": "Stone"}, "sand": {}"#, SOUNDS),
            []
        );
    }

    #[test]
    fn r29_a_sound_no_block_sound_type_has_is_an_information_at_the_member() {
        // The name is compared as written: `stone` is not `Stone`. The client's only line
        // for it is at level 0 before it plays
        // the `normal` sounds: ours, an information.
        assert_eq!(
            unknown(r#""dirt": {"sound": "stone"}"#, SOUNDS),
            [(
                "UnknownSound",
                Severity::Info,
                Phase::Link,
                "rp 2".to_owned(),
                BLOCKS.to_owned(),
                "dirt/sound".to_owned(),
                "block sound",
                "stone".to_owned()
            )]
        );
    }

    #[test]
    fn r29_without_block_sounds_nothing_is_compared_with_a_host_or_without() {
        let entries = file(r#""dirt": {"sound": "squeaky"}"#);
        // A `sounds.json` without the table, and no `sounds.json` at all.
        for resource in [
            vec![
                (BLOCKS, entries.as_str()),
                ("sounds.json", r#"{"entity_sounds": {}}"#),
            ],
            vec![(BLOCKS, entries.as_str())],
        ] {
            let found = link(&[], &resource);
            assert!(of(&found, "UnknownSound").is_empty(), "{found:#?}");
            let found = link_with(&[], &resource, hosted(&[]), LinkOptions::default());
            assert!(of(&found, "UnknownSound").is_empty(), "{found:#?}");
        }
        // The game's sound types are in its own `sounds.json`: with a table on the stack a
        // host is not asked.
        let found = link_with(
            &[],
            &[(BLOCKS, &entries), ("sounds.json", SOUNDS)],
            hosted(&[("block sound", "squeaky")]),
            LinkOptions::default(),
        );
        assert_eq!(of(&found, "UnknownSound").len(), 1, "{found:#?}");
    }

    #[test]
    fn r29_a_sound_that_is_no_string_is_the_load_s_finding_alone() {
        // A number is read as `""`, a boolean as its word; an empty string is a name.
        assert_eq!(
            unknown(r#""dirt": {"sound": 5}, "sand": {"sound": true}"#, SOUNDS),
            []
        );
        let told = unknown(
            r#""dirt": {"sound": ""}, "sand": {"sound": "true"}"#,
            SOUNDS,
        );
        let paths: Vec<&str> = told.iter().map(|said| said.5.as_str()).collect();
        assert_eq!(paths, ["dirt/sound", "sand/sound"]);
    }

    #[test]
    fn r29_a_legacy_key_split_onto_its_blocks_is_told_once() {
        let found = link(
            &[],
            &[
                (BLOCKS, r#"{"wool": {"sound": "squeaky"}}"#),
                ("sounds.json", SOUNDS),
            ],
        );
        let told: Vec<Said> = of(&found, "UnknownSound").into_iter().map(said).collect();
        assert_eq!(told.len(), 1, "{found:#?}");
        assert_eq!(told[0].5, "wool/sound");
    }
}

/// Without the `sound` feature there is no `sounds.json` in view and the rule does not run.
#[cfg(not(feature = "sounds_json"))]
#[test]
fn r29_without_the_sound_feature_nothing_is_reported() {
    let found = link(
        &[],
        &[
            (BLOCKS, &file(r#""dirt": {"sound": "squeaky"}"#)),
            (
                "sounds.json",
                r#"{"block_sounds": {"normal": {"events": {}}}}"#,
            ),
        ],
    );
    assert!(of(&found, "UnknownSound").is_empty(), "{found:#?}");
}

// ---- V11 --------------------------------------------------------------------------------------

#[test]
fn v11_an_unused_exponent_is_told_at_the_member_of_the_entry() {
    let found = link(
        &[("blocks/lamp.json", &block("demo:lamp", DRAWN))],
        &[(
            BLOCKS,
            &file(r#""demo:lamp": {"ambient_occlusion_exponent": 0.5}"#),
        )],
    );
    let unused = of(&found, "NoEffect");
    assert_eq!(unused.len(), 1, "{found:#?}");
    assert_eq!(
        said(unused[0]),
        (
            "NoEffect",
            Severity::Warning,
            Phase::Link,
            "rp 2".to_owned(),
            BLOCKS.to_owned(),
            "demo:lamp/ambient_occlusion_exponent".to_owned(),
            "",
            String::new()
        )
    );
    assert_eq!(
        unused[0].code.to_string(),
        "Block demo:lamp has an \"ambient_occlusion_exponent\" in 'blocks.json' file, this value is unused because the value in \"minecraft:material_instances\" component is used instead"
    );
}

#[test]
fn v11_the_exponent_is_the_merged_one_and_the_component_a_base_component() {
    // The higher pack's 1.0 replaces the lower pack's 0.5: nothing is unused.
    let set = stacks_of(
        vec![behavior_pack(
            1,
            &[("blocks/lamp.json", &block("demo:lamp", DRAWN))],
        )],
        vec![
            resource_pack(
                3,
                &[(
                    BLOCKS,
                    &file(r#""demo:lamp": {"ambient_occlusion_exponent": 1.0}"#),
                )],
            ),
            resource_pack(
                2,
                &[(
                    BLOCKS,
                    &file(r#""demo:lamp": {"ambient_occlusion_exponent": 0.5}"#),
                )],
            ),
        ],
        LoadOptions::default(),
    );
    let found = set.link();
    assert!(of(&found, "NoEffect").is_empty(), "{found:#?}");
    // A block without the component, and a block of the game no pack defines: nothing.
    let found = link(
        &[(
            "blocks/lamp.json",
            &block("demo:lamp", r#""minecraft:friction": 0.4"#),
        )],
        &[(
            BLOCKS,
            &file(
                r#""demo:lamp": {"ambient_occlusion_exponent": 0.5}, "dirt": {"ambient_occlusion_exponent": 0.5}"#,
            ),
        )],
    );
    assert!(of(&found, "NoEffect").is_empty(), "{found:#?}");
}

// ---- R28, R29, V11: the pack that wrote the member ---------------------------------------------

/// The tiles the atlas lacks, told at `blocks.json`, as `(member, tile)`: a pack whose
/// `blocks.json` holds `entries` beside an atlas of one other tile.
#[cfg(feature = "texture_atlas")]
fn mushroom_tiles_told(entries: &str) -> Vec<(String, String)> {
    const TERRAIN: (&str, &str) = (
        "textures/terrain_texture.json",
        r#"{"resource_pack_name": "t", "texture_name": "atlas.terrain", "texture_data": {"one": {"textures": ["textures/a"]}}}"#,
    );
    let set = stacks_of(
        Vec::new(),
        vec![resource_pack(2, &[(BLOCKS, &file(entries)), TERRAIN])],
        LoadOptions::default(),
    );
    let found = set.link();
    let mut told: Vec<(String, String)> = found
        .iter()
        .filter(|finding| finding.code.name() == "UnresolvedReference")
        .map(said)
        .filter(|said| said.4 == BLOCKS && said.7.starts_with("ghost_"))
        .map(|said| (said.5, said.7))
        .collect();
    told.sort();
    told
}

/// `(member, tile)` pairs as `mushroom_tiles_told` gives them.
#[cfg(feature = "texture_atlas")]
fn tiles(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
    pairs
        .iter()
        .map(|(member, tile)| ((*member).to_owned(), (*tile).to_owned()))
        .collect()
}

/// Both mushroom blocks' entries also go to the stem, the later entry over the earlier: the
/// stem's tiles are the red block's. A tile of the stem that the atlas lacks is told at the
/// entry that gave it, where the red block's own finding already stands, and never at the
/// brown block's entry with the red block's tile.
#[cfg(feature = "texture_atlas")]
#[test]
fn r28_a_tile_of_the_mushroom_stem_is_told_at_the_entry_that_gave_it() {
    assert_eq!(
        mushroom_tiles_told(
            r#""brown_mushroom_block": {"textures": "ghost_brown"}, "red_mushroom_block": {"textures": "ghost_red"}"#
        ),
        tiles(&[
            ("brown_mushroom_block/textures", "ghost_brown"),
            ("red_mushroom_block/textures", "ghost_red"),
        ])
    );
}

/// "Later" is the order the client applies the entries in (bytewise by key), not the order
/// the file writes them in: the red block's entry written first still gives the stem its
/// tiles.
#[cfg(feature = "texture_atlas")]
#[test]
fn r28_the_stem_s_tiles_are_the_red_block_s_whatever_order_the_file_writes() {
    assert_eq!(
        mushroom_tiles_told(
            r#""red_mushroom_block": {"textures": "ghost_red"}, "brown_mushroom_block": {"textures": "ghost_brown"}"#
        ),
        tiles(&[
            ("brown_mushroom_block/textures", "ghost_brown"),
            ("red_mushroom_block/textures", "ghost_red"),
        ])
    );
}

/// The stem's own entry comes after the brown block's (`mushroom_stem` sorts after
/// `brown_mushroom_block`) and over it: the stem's tile is its own and is told at its own
/// entry.
#[cfg(feature = "texture_atlas")]
#[test]
fn r28_the_stem_s_own_entry_is_the_place_over_the_brown_block_s() {
    assert_eq!(
        mushroom_tiles_told(
            r#""brown_mushroom_block": {"textures": "ghost_brown"}, "mushroom_stem": {"textures": "ghost_stem"}"#
        ),
        tiles(&[
            ("brown_mushroom_block/textures", "ghost_brown"),
            ("mushroom_stem/textures", "ghost_stem"),
        ])
    );
}

/// The red block's entry comes after the stem's own (`red_mushroom_block` sorts after
/// `mushroom_stem`) and over it: the stem's tile is the red block's, told once, at the red
/// block's entry; the stem's own tile is never used and not told.
#[cfg(feature = "texture_atlas")]
#[test]
fn r28_the_red_block_s_entry_is_the_place_over_the_stem_s_own() {
    assert_eq!(
        mushroom_tiles_told(
            r#""mushroom_stem": {"textures": "ghost_stem"}, "red_mushroom_block": {"textures": "ghost_red"}"#
        ),
        tiles(&[("red_mushroom_block/textures", "ghost_red")])
    );
}

/// A red block's entry without `textures` gives the stem no tile: the brown block's entry
/// stays the place of the stem's tiles, and they are the brown block's.
#[cfg(feature = "texture_atlas")]
#[test]
fn r28_a_red_block_entry_without_textures_leaves_the_brown_block_s_the_place() {
    assert_eq!(
        mushroom_tiles_told(
            r#""brown_mushroom_block": {"textures": "ghost_brown"}, "red_mushroom_block": {"ambient_occlusion_exponent": 0.5}"#
        ),
        tiles(&[("brown_mushroom_block/textures", "ghost_brown")])
    );
}

/// K122: the merged entry of a block is placed at the highest pack that has an entry for it,
/// whichever pack wrote each member. A finding about a member is told at the pack and file
/// that wrote it: the ordinary shape of an add-on over another pack's `blocks.json`.
#[cfg(all(feature = "texture_atlas", feature = "sounds_json"))]
#[test]
fn r28_r29_v11_a_member_a_lower_pack_wrote_is_told_at_that_packs_file() {
    const TERRAIN: (&str, &str) = (
        "textures/terrain_texture.json",
        r#"{"resource_pack_name": "t", "texture_name": "atlas.terrain", "texture_data": {"one": {"textures": ["textures/a"]}}}"#,
    );
    const SOUNDS: (&str, &str) = (
        "sounds.json",
        r#"{"block_sounds": {"normal": {"events": {}}}}"#,
    );
    let lower = file(
        r#""demo:lamp": {"textures": "ghost", "carried_textures": "ghost", "sound": "squeaky", "ambient_occlusion_exponent": 0.5}"#,
    );
    let told = |higher: &str| -> Vec<(String, String, &'static str)> {
        let set = stacks_of(
            vec![behavior_pack(
                1,
                &[("blocks/lamp.json", &block("demo:lamp", DRAWN))],
            )],
            vec![
                resource_pack(3, &[(BLOCKS, &file(higher))]),
                resource_pack(2, &[(BLOCKS, &lower), TERRAIN, SOUNDS]),
            ],
            LoadOptions::default(),
        );
        let found = set.link();
        let mut told: Vec<_> = found
            .iter()
            .filter(|finding| {
                matches!(
                    finding.code.name(),
                    "UnresolvedReference" | "UnknownSound" | "NoEffect"
                )
            })
            .map(said)
            .filter(|said| said.4 == BLOCKS)
            .map(|said| (said.3, said.5, said.0))
            .collect();
        told.sort();
        told
    };
    let at = |pack: &str, member: &str, code: &'static str| {
        (pack.to_owned(), format!("demo:lamp/{member}"), code)
    };
    // The higher pack has an entry for the block and none of the four members.
    assert_eq!(
        told(r#""demo:lamp": {"isotropic": true}"#),
        [
            at("rp 2", "ambient_occlusion_exponent", "NoEffect"),
            at("rp 2", "carried_textures", "UnresolvedReference"),
            at("rp 2", "sound", "UnknownSound"),
            at("rp 2", "textures", "UnresolvedReference"),
        ]
    );
    // A member the higher pack writes too is the higher pack's.
    assert_eq!(
        told(r#""demo:lamp": {"sound": "creaky", "textures": "phantom"}"#),
        [
            at("rp 2", "ambient_occlusion_exponent", "NoEffect"),
            at("rp 2", "carried_textures", "UnresolvedReference"),
            at("rp 3", "sound", "UnknownSound"),
            at("rp 3", "textures", "UnresolvedReference"),
        ]
    );
}

// ---- V12, V13, L13 ----------------------------------------------------------------------------

fn creator_features() -> LoadOptions {
    LoadOptions {
        upcoming_creator_features: true,
        ..LoadOptions::default()
    }
}

const CUSTOM: &str = "demo:lamp: trying to override the Geometry component with blocks.json settings for a custom block. This isn't supported. Please remove any legacy texture definition or block shape specification for this block.";

#[test]
fn v12_each_of_the_four_drawing_members_counts_and_no_other() {
    let lamp = block("demo:lamp", DRAWN);
    let told = |entry: &str| -> Vec<Said> {
        let found = link_with(
            &[("blocks/lamp.json", &lamp)],
            &[(BLOCKS, &file(&format!(r#""demo:lamp": {entry}"#)))],
            creator_features(),
            LinkOptions::default(),
        );
        found
            .iter()
            .filter(|finding| finding.code.to_string() == CUSTOM)
            .map(said)
            .collect()
    };
    for entry in [
        r#"{"textures": "lamp"}"#,
        r#"{"isotropic": true}"#,
        r#"{"ambient_occlusion_exponent": 0.5}"#,
        // A member counts whatever its value.
        r#"{"textures": null}"#,
    ] {
        assert_eq!(
            told(entry),
            [(
                "NoEffect",
                Severity::Warning,
                Phase::Link,
                "rp 2".to_owned(),
                BLOCKS.to_owned(),
                "demo:lamp".to_owned(),
                "",
                String::new()
            )],
            "{entry}"
        );
    }
    assert_eq!(
        told(r#"{"sound": "glass"}"#),
        [],
        "a member of another kind"
    );
    assert_eq!(told("{}"), []);
}

#[test]
fn v12_only_a_block_drawn_from_its_geometry_is_told() {
    let found = link_with(
        &[(
            "blocks/lamp.json",
            &block("demo:lamp", r#""minecraft:friction": 0.4"#),
        )],
        &[(BLOCKS, &file(r#""demo:lamp": {"textures": "lamp"}"#))],
        creator_features(),
        LinkOptions::default(),
    );
    assert!(
        found
            .iter()
            .all(|finding| finding.code.to_string() != CUSTOM),
        "{found:#?}"
    );
}

#[test]
fn v12_is_told_at_the_highest_pack_that_sets_a_member() {
    let lamp = block("demo:lamp", DRAWN);
    let set = stacks_of(
        vec![behavior_pack(1, &[("blocks/lamp.json", &lamp)])],
        vec![
            resource_pack(4, &[(BLOCKS, &file(r#""demo:lamp": {"sound": "glass"}"#))]),
            resource_pack(3, &[(BLOCKS, &file(r#""demo:lamp": {"isotropic": true}"#))]),
            resource_pack(
                2,
                &[(BLOCKS, &file(r#""demo:lamp": {"textures": "lamp"}"#))],
            ),
        ],
        creator_features(),
    );
    let found = set.link();
    let told: Vec<Said> = found
        .iter()
        .filter(|finding| finding.code.to_string() == CUSTOM)
        .map(said)
        .collect();
    assert_eq!(told.len(), 1, "{found:#?}");
    assert_eq!(told[0].3, "rp 3");
}
