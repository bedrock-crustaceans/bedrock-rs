//! Unit tests of the family's own functions: the `.lang` reader, and the cost of
//! each step that walks a pack or a file (on inputs built before the clock starts, at `n`
//! against four times `n`, by the crate's one protocol: `bedrock_addon_core::test_cost`).

#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "a cost test reads the clock"
)]

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::time::Duration;

use bedrock_addon_core::pack_identity::{PackIdVersion, PackPath, PackType};
use bytes::Bytes;
use uuid::Uuid;

use super::load::{
    as_emote, index, kept_animations, missing_images, missing_models, retarget_hip_parents,
};
use super::piece::PieceGeometry;
use super::read::{RootError, read_meta, read_root};
use super::text::lang_entries;
use super::{BUILTIN_PACK, PersonaPiece, PersonaPieces, PieceSource};
use bedrock_addon_core::document::Content;
use bedrock_addon_core::test_cost::{assert_each_linear_timed_at, time_of};
use bedrock_addon_core::version::SemVersion;

#[test]
fn a_lang_file_s_keys() {
    let entries = lang_entries(
        "\u{feff}## a comment\n## commented=out\n\nfirst=One\nsecond=Two\t## note\r\n  ## indented=comment\n \t## tabbed=comment\nno equals sign\nfirst=Again\nempty=\nwith=equals=inside\n"
            .as_bytes(),
    );
    let mut keys: Vec<(&str, &str)> = entries
        .iter()
        .map(|(key, value)| (key.as_str(), value.as_str()))
        .collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        [
            ("empty", ""),
            ("first", "One"),
            ("second", "Two"),
            ("with", "equals=inside")
        ]
    );
}

#[test]
fn the_index_groups_files_by_suffix_and_folder() {
    let paths: Vec<PackPath> = [
        "geometry.json",
        "pieces/a/a.geometry.json",
        "pieces/a/a.meta.json",
        "pieces/a/b.geometry.json",
        "pieces/a/b.meta.json",
        "pieces/a/b.store_info.json",
        "pieces/a/texts/de_DE.lang",
        "pieces/a/texts/en_US.lang",
        "pieces/b/texts/de_DE.lang",
        "pieces/c/lang/en_US.lang",
        "root.meta.json",
        "texts/de_DE.lang",
        "texts/en_US.lang",
    ]
    .iter()
    .map(|path| PackPath::from_static(path))
    .collect();
    let refs: Vec<&PackPath> = paths.iter().collect();
    let index = index(&refs);
    let folders: Vec<&str> = index.folders.keys().copied().collect();
    assert_eq!(folders, ["", "pieces/a"]);
    let a = &index.folders["pieces/a"];
    let names =
        |paths: &[&PackPath]| -> Vec<String> { paths.iter().map(ToString::to_string).collect() };
    assert_eq!(
        names(&a.metas),
        ["pieces/a/a.meta.json", "pieces/a/b.meta.json"]
    );
    // A later file of a suffix takes the slot.
    assert_eq!(
        a.geometry.map(PackPath::as_str),
        Some("pieces/a/b.geometry.json")
    );
    assert_eq!(
        a.store_info.map(PackPath::as_str),
        Some("pieces/a/b.store_info.json")
    );
    assert_eq!(
        a.texts.map(PackPath::as_str),
        Some("pieces/a/texts/en_US.lang")
    );
    assert_eq!(names(&index.folders[""].metas), ["root.meta.json"]);
    assert_eq!(index.folders[""].texts, None);
    assert_eq!(
        index.root_texts.map(PackPath::as_str),
        Some("texts/en_US.lang")
    );
    assert_eq!(
        names(&index.names),
        ["texts/de_DE.lang", "texts/en_US.lang"]
    );
    let mut with_texts: Vec<&str> = index.with_texts.iter().copied().collect();
    with_texts.sort_unstable();
    assert_eq!(with_texts, ["pieces/a", "pieces/b"]);
    assert_eq!(index.piece_files.len(), 6);
    assert_eq!(index.files.len(), paths.len());
}

// ---- cost ---------------------------------------------------------------------------------

/// A pack held in memory.
struct Files {
    id: PackIdVersion,
    files: BTreeMap<PackPath, Bytes>,
}

impl Files {
    fn built_in() -> Self {
        Files {
            id: PackIdVersion::new(BUILTIN_PACK, SemVersion::new(0, 0, 1), PackType::Resource),
            files: BTreeMap::new(),
        }
    }

    fn file(&mut self, path: &str, text: String) {
        self.files
            .insert(PackPath::new(path).expect("a pack path"), Bytes::from(text));
    }
}

impl PieceSource for Files {
    fn id(&self) -> &PackIdVersion {
        &self.id
    }

    fn paths(&self) -> impl Iterator<Item = &PackPath> + '_ {
        self.files.keys()
    }

    fn read(&self, path: &PackPath) -> Result<Bytes, std::io::ErrorKind> {
        self.files
            .get(path)
            .cloned()
            .ok_or(std::io::ErrorKind::NotFound)
    }
}

/// The paths of a pack of `count` piece folders, four files each.
fn piece_paths(count: usize) -> Vec<PackPath> {
    let mut paths = Vec::with_capacity(count * 4);
    for n in 0..count {
        for file in [
            "p.geometry.json",
            "p.meta.json",
            "p.store_info.json",
            "texts/en_US.lang",
        ] {
            paths.push(PackPath::new(&format!("pieces/p{n:06}/{file}")).expect("a pack path"));
        }
    }
    paths
}

#[test]
fn the_index_is_built_in_linear_time() {
    let run = |count: usize| {
        let paths = piece_paths(count);
        let refs: Vec<&PackPath> = paths.iter().collect();
        [time_of(|| {
            let index = index(&refs);
            assert_eq!(index.folders.len(), count);
            index.piece_files.len()
        })]
    };
    assert_each_linear_timed_at(["index"], 5_000, run);
}

/// A built-in pack of `count` pieces; every `same`-th piece repeats the id of the one before
/// it.
fn pack_of_pieces(count: usize) -> Files {
    let mut pack = Files::built_in();
    let mut names = String::new();
    for n in 0..count {
        let id = Uuid::from_u128(0x1000_0000_0000_4000_8000_0000_0000_0000 + (n / 2) as u128);
        pack.file(
            &format!("pieces/p{n:06}/p.meta.json"),
            format!(
                r#"{{ "piece_id": "{id}", "piece_name": "p{n}", "piece_type": "persona_hair", "texture_sources": [{{ "texture": "p" }}] }}"#
            ),
        );
        pack.file(&format!("pieces/p{n:06}/p.png"), String::new());
        let _ = writeln!(names, "persona.p{n}.title=Piece {n}");
    }
    pack.file("texts/en_US.lang", names);
    pack
}

#[test]
fn a_pack_of_many_pieces_loads_in_linear_time() {
    let run = |count: usize| {
        let pack = pack_of_pieces(count);
        [time_of(|| {
            let loaded = PersonaPiece::load_all(&pack);
            assert_eq!(loaded.pieces.len(), count);
            // Every second piece repeats an id; every name is translated.
            assert_eq!(loaded.diagnostics.len(), count / 2);
            assert_eq!(loaded.report().count(), count / 2);
            assert_eq!(loaded.kept().count(), count / 2);
            loaded
        })]
    };
    assert_each_linear_timed_at(["load_all"], 1_000, run);
}

/// A piece pack of one emote whose meta file has `count` entries in every list, each naming
/// a file or a member of its own.
fn pack_of_one_piece(count: usize) -> Files {
    let mut pack = Files::built_in();
    let mut textures = String::new();
    let mut geometries = String::new();
    let mut animations = String::new();
    let mut models = String::new();
    let mut clips = String::new();
    for n in 0..count {
        let comma = if n == 0 { "" } else { ", " };
        let _ = write!(
            textures,
            r#"{comma}{{ "texture": "t{n}", "tint_map": "m{n}" }}"#
        );
        let _ = write!(
            geometries,
            r#"{comma}{{ "geometry": "geometry.g{n}", "arm_size": "", "zone": ["z{n}"] }}"#
        );
        let _ = write!(
            animations,
            r#"{comma}{{ "name": "animation.a{n}", "animationFile": "p.animation.json" }}"#
        );
        let _ = write!(
            models,
            r#"{comma}"geometry.g{n}": {{ "bones": [{{ "name": "b", "parent": "hip" }}] }}"#
        );
        let _ = write!(
            clips,
            r#"{comma}"animation.a{n}": {{ "bones": {{ "hip": {{}} }} }}"#
        );
        pack.file(&format!("pieces/p/t{n}.png"), String::new());
        pack.file(&format!("pieces/p/m{n}.tga"), String::new());
    }
    pack.file(
        "pieces/p/p.meta.json",
        format!(
            r#"{{ "piece_id": "10000000-0000-4000-8000-000000000001", "piece_name": "p", "piece_type": "persona_emote", "texture_sources": [{textures}], "geometry_sources": [{geometries}], "animation_sources": [{animations}] }}"#
        ),
    );
    pack.file("pieces/p/p.geometry.json", format!("{{ {models} }}"));
    pack.file(
        "pieces/p/p.animation.json",
        format!(r#"{{ "animations": {{ {clips} }} }}"#),
    );
    pack.file("texts/en_US.lang", "persona.p.title=P\n".to_owned());
    pack
}

/// The one piece of `loaded`, which has no finding.
fn the_piece(loaded: &PersonaPieces) -> &PersonaPiece {
    assert_eq!(loaded.pieces.len(), 1);
    let piece = &loaded.pieces[0];
    let told: Vec<String> = loaded.report().map(ToString::to_string).collect();
    assert!(told.is_empty(), "{told:?}");
    piece
}

#[test]
fn a_piece_of_many_sources_loads_whole() {
    // The steps timed one by one below, wired together (small: the JSON parse dominates).
    let count = 2_000;
    let pack = pack_of_one_piece(count);
    let loaded = PersonaPiece::load_all(&pack);
    let piece = the_piece(&loaded);
    assert_eq!(piece.animations.len(), count);
    assert_eq!(piece.meta.value().textures().len(), count);
    assert!(piece.emote.is_some());
    assert!(piece.geometry.is_some());
}

/// The times of the per-entry steps of one piece with `count` entries in every list, each on
/// inputs read and parsed before its clock starts: the image lookups, the model lookups, the
/// hip retarget, the animation entries and the emote's rewrite.
fn steps_of_one_piece(count: usize) -> [Duration; 5] {
    const ID: &str = "10000000-0000-4000-8000-000000000001";
    let pack = pack_of_one_piece(count);
    let paths: Vec<&PackPath> = pack.files.keys().collect();
    let index = index(&paths);
    let root_of = |path: &'static str| {
        let bytes = pack.read(&PackPath::from_static(path)).expect("the file");
        read_root(&bytes, &mut Vec::new()).expect("JSON")
    };
    let meta = read_meta(&root_of("pieces/p/p.meta.json"), &mut Vec::new());
    let geometry_json = root_of("pieces/p/p.geometry.json");
    let animation = root_of("pieces/p/p.animation.json");

    let images = time_of(|| {
        let mut findings = Vec::new();
        missing_images(&index.files, "pieces/p", &meta, &mut findings);
        assert!(findings.is_empty());
    });
    let geometry = PieceGeometry {
        path: PackPath::from_static("pieces/p/p.geometry.json"),
        json: geometry_json.clone(),
    };
    let models = time_of(|| {
        let mut findings = Vec::new();
        missing_models(Some(&geometry), "pieces/p", &meta, &mut findings);
        assert!(findings.is_empty());
        // And every name missed, with its finding.
        missing_models(None, "pieces/p", &meta, &mut findings);
        assert_eq!(findings.len(), count);
    });
    let hips = time_of(|| {
        let mut json = geometry_json.clone();
        retarget_hip_parents(&mut json);
        json
    });
    let animations = time_of(|| {
        let mut findings = Vec::new();
        let mut opens = 0;
        let mut open = |_: &PackPath| {
            opens += 1;
            Ok(Some(animation.clone()))
        };
        let (kept, emote) = kept_animations(
            &index.files,
            "pieces/p",
            &meta,
            ID,
            &mut findings,
            &mut open,
        );
        assert_eq!(kept.len(), count);
        assert!(emote.is_some());
        assert!(findings.is_empty());
        // One file, however many entries name it.
        assert_eq!(opens, 1);
    });
    let emote = time_of(|| as_emote(&animation, ID, &mut Vec::new()));
    [images, models, hips, animations, emote]
}

#[test]
fn the_steps_of_a_piece_of_many_sources_take_linear_time() {
    assert_each_linear_timed_at(
        ["images", "models", "hip parents", "animations", "emote"],
        15_000,
        steps_of_one_piece,
    );
}

#[test]
fn a_meta_file_is_read_and_written_in_linear_time() {
    // The member readers and the writer on their own: the root is parsed before the clock.
    let run = |count: usize| {
        let pack = pack_of_one_piece(count);
        let bytes = pack
            .read(&PackPath::from_static("pieces/p/p.meta.json"))
            .expect("the meta file");
        let mut findings = Vec::new();
        let root = read_root(&bytes, &mut findings).expect("JSON");
        let read = time_of(|| {
            let mut findings = Vec::new();
            let meta = read_meta(&root, &mut findings);
            assert!(findings.is_empty());
            meta
        });
        let meta = read_meta(&root, &mut findings);
        let written = time_of(|| meta.to_value());
        let text = time_of(|| meta.to_json().expect("a text").len());
        [read, written, text]
    };
    assert_each_linear_timed_at(["read_meta", "to_value", "to_json"], 5_000, run);
}

#[test]
fn a_file_of_many_unknown_members_is_read_and_written_in_linear_time() {
    let run = |count: usize| {
        let mut text =
            String::from(r#"{ "piece_id": "x", "piece_name": "p", "piece_type": "persona_hair""#);
        for n in 0..count {
            let _ = write!(text, r#", "unknown{n}": {n}"#);
        }
        text.push_str(" }");
        let mut findings = Vec::new();
        let root = read_root(text.as_bytes(), &mut findings).expect("JSON");
        let read = time_of(|| {
            let mut findings = Vec::new();
            let meta = read_meta(&root, &mut findings);
            assert_eq!(findings.len(), count);
            meta
        });
        let meta = read_meta(&root, &mut findings);
        let written = time_of(|| meta.to_value());
        [read, written]
    };
    assert_each_linear_timed_at(["read_meta", "to_value"], 25_000, run);
}

#[test]
fn read_root_reports_bad_bytes_as_typed_errors() {
    assert!(matches!(
        read_root(&[0xff, 0xfe, b'{'], &mut Vec::new()),
        Err(RootError::NotUtf8(_))
    ));
    assert!(matches!(
        read_root(b"{\"a\":", &mut Vec::new()),
        Err(RootError::Json(_))
    ));
    let deep = "[".repeat(2000);
    assert!(matches!(
        read_root(deep.as_bytes(), &mut Vec::new()),
        Err(RootError::TooDeep { .. })
    ));
}
