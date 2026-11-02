//! The files beside a meta file: geometry (pg01 to pg04, pg08), emotes (pe01 to
//! pe07), store info and texts (ps01 to ps06), pm14 and pm15, and rules M13, G1 to G3,
//! A1 to A5, S1 to S5.

use std::cell::RefCell;

use bedrock_addon::diagnostic::{DiagnosticCode, DiagnosticDetail, Severity};
use bedrock_addon::families::persona::{
    BUILTIN_PACK, PersonaPiece, PersonaPieces, PieceSource, name_key,
};
use bedrock_addon::pack::PackIdVersion;
use bedrock_addon::storage::PackPath;
use bytes::Bytes;

use crate::common::{OfPack, PIECE_PACK, builtin_pack, hat, lines, message, only, piece_pack};
use crate::packs::meta;

const META: &str = "pieces/hat/hat.meta.json";

/// The findings of `pieces`, as lines.
fn found(pieces: &PersonaPieces) -> Vec<String> {
    lines(pieces).into_iter().collect()
}

/// The message of the first finding whose line contains `wanted`.
fn text_of(pieces: &PersonaPieces, wanted: &str) -> String {
    pieces
        .report()
        .find(|found| crate::common::line(found).contains(wanted))
        .map_or_else(|| panic!("no {wanted} in {:?}", lines(pieces)), message)
}

/// A hair piece whose one geometry source names `geometry.hat`, with `files` beside it.
fn with_geometry(files: &[(&str, &str)]) -> PersonaPieces {
    hat(
        &meta(r#""geometry_sources": [{ "geometry": "geometry.hat" }]"#),
        files,
    )
}

// ---- geometry -----------------------------------------------------------------------------

#[test]
fn pg01_a_keyed_model_is_found_by_its_name() {
    let pieces = with_geometry(&[(
        "hat.geometry.json",
        r#"{ "format_version": "1.12.0", "geometry.hat": { "bones": [] } }"#,
    )]);
    assert_eq!(found(&pieces), Vec::<String>::new());
    let geometry = only(&pieces).geometry.as_ref().expect("the geometry file");
    assert_eq!(geometry.path.as_str(), "pieces/hat/hat.geometry.json");
    assert_eq!(geometry.format_version(), "1.12.0");
    assert!(geometry.model("geometry.hat").is_some());
    assert_eq!(geometry.models().collect::<Vec<_>>(), ["geometry.hat"]);
    let file = geometry.model_file("geometry.hat").expect("the model");
    let members = file.as_object().unwrap();
    assert_eq!(
        members
            .get("format_version")
            .and_then(|v| v.as_string())
            .map(facet_value::VString::as_str),
        Some("1.12.0")
    );
    assert!(members.get("geometry.hat").is_some());
    assert_eq!(geometry.model_file("geometry.other"), None);
}

#[cfg(feature = "geometry")]
#[test]
fn pg01_the_model_goes_to_the_geometry_reader_with_the_file_s_version() {
    use bedrock_addon::document::Content;
    use bedrock_addon::families::geometry::GeometryFile;
    use bedrock_addon::load::LoadContext;

    let pieces = with_geometry(&[(
        "hat.geometry.json",
        r#"{ "format_version": "1.12.0", "geometry.hat": { "bones": [{ "name": "head", "parent": "hip" }], "texturewidth": 64 }, "geometry.other": { "bones": [] } }"#,
    )]);
    let geometry = only(&pieces).geometry.as_ref().unwrap();
    assert_eq!(geometry.format_version(), "1.12.0");
    let file = geometry.model_file("geometry.hat").unwrap();
    // The client puts the character's model together in memory and registers it with the
    // file's version; a geometry *file* of 1.12.0 has no keyed models, so the geometry
    // family's file reader refuses the piece file as it stands...
    let text = bedrock_addon::json::to_json(&file).unwrap();
    let cx = LoadContext::detached();
    assert!(GeometryFile::parse(&geometry.path, text.as_bytes(), &cx).is_err());
    // ... and its keyed reader takes the model with the file's version (pg01, G7),
    // the bone already on `waist`.
    let document =
        bedrock_addon::families::geometry::load_keyed_file(&geometry.path, text.as_bytes(), &cx)
            .unwrap_or_else(|rejected| panic!("{rejected:?}"));
    assert_eq!(
        document.value().format_version,
        bedrock_addon::version::FormatVersion::new(1, 12, 0)
    );
    let models = &document.value().geometries;
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].description.identifier.as_str(), "geometry.hat");
    assert_eq!(
        models[0].bones.as_ref().unwrap()[0].parent.as_deref(),
        Some("waist")
    );
}

#[test]
fn a_geometry_file_without_a_version_is_1_0() {
    let pieces = with_geometry(&[("hat.geometry.json", r#"{ "geometry.hat": {} }"#)]);
    let geometry = only(&pieces).geometry.as_ref().unwrap();
    assert_eq!(geometry.format_version(), "1.0");
    let pieces = with_geometry(&[(
        "hat.geometry.json",
        r#"{ "format_version": 2, "geometry.hat": {} }"#,
    )]);
    assert_eq!(
        only(&pieces).geometry.as_ref().unwrap().format_version(),
        "1.0"
    );
}

const NOT_ON: &str = "Couldn't find geometry name geometry.hat on pieces/hat/hat.geometry.json.";

/// G1, G2 and G3 are the game's level-3 lines after which the piece stays: kept errors.
#[test]
fn g1_g2_g3_are_kept_errors() {
    for files in [
        &[][..],
        &[("hat.geometry.json", "[")][..],
        &[("hat.geometry.json", r#"{ "format_version": "1.12.0" }"#)][..],
    ] {
        let pieces = with_geometry(files);
        assert_eq!(pieces.pieces.len(), 1);
        let errors: Vec<_> = pieces
            .report()
            .filter(|found| found.severity == Severity::Error)
            .collect();
        assert!(
            !errors.is_empty() || files.is_empty(),
            "{:#?}",
            lines(&pieces)
        );
        assert!(errors.iter().all(|found| found.kept), "{errors:#?}");
    }
}

#[test]
fn pg02_g3_a_model_of_the_array_layout_is_not_found() {
    let pieces = with_geometry(&[(
        "hat.geometry.json",
        r#"{ "format_version": "1.16.0", "minecraft:geometry": [{ "description": { "identifier": "geometry.hat" } }] }"#,
    )]);
    assert_eq!(
        found(&pieces),
        [format!(
            "error UnresolvedReference {META} geometry_sources/0/geometry"
        )]
    );
    assert_eq!(text_of(&pieces, "UnresolvedReference"), NOT_ON);
    let finding = pieces.report().next().unwrap();
    assert!(matches!(
        finding.code,
        DiagnosticCode::UnresolvedReference { .. }
    ));
    assert!(matches!(
        finding.detail.as_deref(),
        Some(DiagnosticDetail::Reference { kind: "geometry", id, .. }) if id == "geometry.hat"
    ));
    // The piece is kept: the client skips the entry when a character wears the piece.
    assert_eq!(pieces.pieces.len(), 1);
}

#[test]
fn pg03_g3_geometry_sources_without_a_geometry_file() {
    let pieces = with_geometry(&[]);
    assert_eq!(
        found(&pieces),
        [format!(
            "error UnresolvedReference {META} geometry_sources/0/geometry"
        )]
    );
    assert!(only(&pieces).geometry.is_none());
    assert_eq!(
        text_of(&pieces, "UnresolvedReference"),
        "Couldn't find geometry name geometry.hat on pieces/hat."
    );
}

#[test]
fn g3_each_source_is_looked_up() {
    let pieces = hat(
        &meta(
            r#""geometry_sources": [{ "geometry": "geometry.a" }, { "geometry": "geometry.b" }, { "geometry": "geometry.A" }, { "geometry": "geometry.null" }, { "geometry": "" }, { "geometry": "geometry.a" }]"#,
        ),
        &[(
            "hat.geometry.json",
            r#"{ "geometry.a": {}, "geometry.null": null }"#,
        )],
    );
    assert_eq!(
        found(&pieces),
        [
            format!("error UnresolvedReference {META} geometry_sources/1/geometry"),
            format!("error UnresolvedReference {META} geometry_sources/2/geometry"),
            format!("error UnresolvedReference {META} geometry_sources/3/geometry"),
            // The empty name is looked up like any other (a source without one is refused).
            format!("error UnresolvedReference {META} geometry_sources/4/geometry"),
        ]
    );
    // A geometry file whose root is no object has no models.
    let pieces = with_geometry(&[("hat.geometry.json", "[]")]);
    assert_eq!(
        found(&pieces),
        [format!(
            "error UnresolvedReference {META} geometry_sources/0/geometry"
        )]
    );
}

#[test]
fn pg04_g2_a_geometry_file_that_is_no_json() {
    let pieces = with_geometry(&[("hat.geometry.json", r#"{ "geometry.hat": "#)]);
    // The broken file is what is reported: nobody could look a name up in it.
    assert_eq!(
        found(&pieces),
        ["error MalformedInput pieces/hat/hat.geometry.json"]
    );
    assert_eq!(
        text_of(&pieces, "MalformedInput"),
        "Could not parse geometry file pieces/hat/hat.geometry.json"
    );
    let piece = only(&pieces);
    assert!(piece.geometry.is_none());
    assert!(pieces.rejected.is_empty());
}

#[test]
fn pg08_a_bone_whose_parent_is_hip_hangs_on_waist() {
    let pieces = with_geometry(&[(
        "hat.geometry.json",
        r#"{ "format_version": "1.12.0", "geometry.hat": { "bones": [{ "name": "a", "parent": "hip" }, { "name": "b", "parent": "waist" }, { "name": "hip" }, { "name": "c", "parent": "Hip" }, 3] }, "geometry.b": { "bones": [{ "name": "d", "parent": "hip" }] }, "geometry.c": { "bones": 3 }, "geometry.d": 4 }"#,
    )]);
    assert_eq!(found(&pieces), Vec::<String>::new());
    let geometry = only(&pieces).geometry.as_ref().unwrap();
    let bones = |model: &str| -> Vec<(String, Option<String>)> {
        let bones = geometry
            .model(model)
            .unwrap()
            .as_object()
            .unwrap()
            .get("bones");
        bones
            .and_then(|v| v.as_array())
            .unwrap()
            .as_slice()
            .iter()
            .filter_map(|bone| bone.as_object())
            .map(|bone| {
                let text = |key: &str| {
                    bone.get(key)
                        .and_then(|v| v.as_string())
                        .map(|v| v.as_str().to_owned())
                };
                (text("name").unwrap(), text("parent"))
            })
            .collect()
    };
    assert_eq!(
        bones("geometry.hat"),
        [
            ("a".to_owned(), Some("waist".to_owned())),
            ("b".to_owned(), Some("waist".to_owned())),
            ("hip".to_owned(), None),
            ("c".to_owned(), Some("Hip".to_owned())),
        ]
    );
    assert_eq!(
        bones("geometry.b"),
        [("d".to_owned(), Some("waist".to_owned()))]
    );
}

#[test]
fn the_geometry_file_held_is_the_file_after_the_retarget_not_as_authored() {
    use bedrock_addon::json::{Value, from_str};

    let authored = r#"{ "format_version": "1.12.0", "geometry.hat": { "bones": [{ "name": "a", "parent": "hip", "pivot": [0, 1, 0] }, { "name": "hip", "parent": "root" }] } }"#;
    let pieces = with_geometry(&[("hat.geometry.json", authored)]);
    assert_eq!(found(&pieces), Vec::<String>::new());
    let held = &only(&pieces).geometry.as_ref().unwrap().json;
    // Not the author's file: the one bone parent `hip` is `waist`, and nothing else differs.
    assert_ne!(*held, from_str::<Value>(authored).unwrap());
    let retargeted = authored.replace(r#""parent": "hip""#, r#""parent": "waist""#);
    assert_eq!(*held, from_str::<Value>(&retargeted).unwrap());
    // A file without such a parent is held as it was written.
    let pieces = with_geometry(&[("hat.geometry.json", &retargeted)]);
    assert_eq!(
        only(&pieces).geometry.as_ref().unwrap().json,
        from_str::<Value>(&retargeted).unwrap()
    );
}

// ---- a source that fails some reads, and counts all ---------------------------------------

/// A pack that refuses to read `unreadable` and records every read.
struct Watched<'a> {
    pack: OfPack<'a>,
    unreadable: &'a [&'a str],
    read: RefCell<Vec<String>>,
}

impl<'a> Watched<'a> {
    fn new(pack: &'a bedrock_addon::pack::Pack, unreadable: &'a [&'a str]) -> Self {
        Watched {
            pack: OfPack(pack),
            unreadable,
            read: RefCell::new(Vec::new()),
        }
    }
}

impl PieceSource for Watched<'_> {
    fn id(&self) -> &PackIdVersion {
        self.pack.id()
    }

    fn paths(&self) -> impl Iterator<Item = &PackPath> + '_ {
        self.pack.paths()
    }

    fn read(&self, path: &PackPath) -> Result<Bytes, std::io::ErrorKind> {
        self.read.borrow_mut().push(path.as_str().to_owned());
        if self.unreadable.contains(&path.as_str()) {
            Err(std::io::ErrorKind::PermissionDenied)
        } else {
            self.pack.read(path)
        }
    }
}

#[test]
fn g1_a_geometry_file_that_cannot_be_read() {
    let pack = piece_pack(&[
        (
            META,
            &meta(r#""geometry_sources": [{ "geometry": "geometry.hat" }]"#),
        ),
        ("pieces/hat/hat.geometry.json", r#"{ "geometry.hat": {} }"#),
        ("texts/en_US.lang", "persona.hat.title=Hat\n"),
    ]);
    let watched = Watched::new(&pack, &["pieces/hat/hat.geometry.json"]);
    let pieces = PersonaPiece::load_all(&watched);
    assert_eq!(
        found(&pieces),
        ["error ReadError pieces/hat/hat.geometry.json"]
    );
    assert_eq!(
        text_of(&pieces, "ReadError"),
        "Missing expected geometry file pieces/hat/hat.geometry.json"
    );
    assert!(pieces.report().any(|found| matches!(
        found.code,
        DiagnosticCode::ReadError {
            kind: std::io::ErrorKind::PermissionDenied,
            ..
        }
    )));
    assert!(only(&pieces).geometry.is_none());
}

// ---- textures -----------------------------------------------------------------------------

#[test]
fn pm14_m13_a_texture_that_is_no_file_of_the_folder() {
    let pieces = hat(
        &meta(r#""texture_sources": [{ "texture": "nope.png" }]"#),
        &[("hat.png", "png")],
    );
    assert_eq!(
        found(&pieces),
        [format!(
            "info UnresolvedReference {META} texture_sources/0/texture"
        )]
    );
    let finding = pieces.report().next().unwrap();
    assert!(matches!(
        finding.detail.as_deref(),
        Some(DiagnosticDetail::Reference { kind: "texture", id, .. }) if id == "nope.png"
    ));
    // The game only asserts in a developer build: an information of ours.
    assert_eq!(finding.severity, Severity::Info);
}

#[test]
fn pm15_m13_a_texture_resolves_with_or_without_its_extension() {
    for file in ["hat", "hat.tga", "hat.png", "hat.jpg", "hat.jpeg"] {
        let pieces = hat(
            &meta(r#""texture_sources": [{ "texture": "hat" }]"#),
            &[(file, "image")],
        );
        assert_eq!(found(&pieces), Vec::<String>::new(), "{file}");
    }
    for file in [
        "hat.gif",
        "hat.PNG",
        "Hat.png",
        "sub/hat.png",
        "hat.png.png",
    ] {
        let pieces = hat(
            &meta(r#""texture_sources": [{ "texture": "hat" }]"#),
            &[(file, "image")],
        );
        assert_eq!(
            found(&pieces),
            [format!(
                "info UnresolvedReference {META} texture_sources/0/texture"
            )],
            "{file}"
        );
    }
    // The name is relative to the piece's folder: a file of that name elsewhere is another.
    let pack = piece_pack(&[
        (META, &meta(r#""texture_sources": [{ "texture": "hat" }]"#)),
        ("pieces/other/hat.png", "png"),
        ("hat.png", "png"),
        ("texts/en_US.lang", "persona.hat.title=Hat\n"),
    ]);
    assert_eq!(found(&crate::common::load(&pack)).len(), 1);
    // A name in a folder below the piece's.
    let pieces = hat(
        &meta(r#""texture_sources": [{ "texture": "sub/hat" }]"#),
        &[("sub/hat.png", "png")],
    );
    assert_eq!(found(&pieces), Vec::<String>::new());
}

#[test]
fn m13_every_image_member_is_looked_up() {
    let members = r#""clothing_map": "c", "face_clothing_map": "f", "texture_sources": [{ "texture": "t0", "tint_map": "m0" }, { "texture": "t1", "tint_map": "m1" }], "geometry_sources": [{ "geometry": "g", "texture": "t2", "tint_map": "m2" }]"#;
    let geometry = ("hat.geometry.json", r#"{ "g": {} }"#);
    let none = hat(&meta(members), &[geometry]);
    let at = |member: &str| format!("info UnresolvedReference {META} {member}");
    assert_eq!(
        found(&none),
        [
            at("clothing_map"),
            at("face_clothing_map"),
            at("texture_sources/0/texture"),
            at("texture_sources/0/tint_map"),
            at("texture_sources/1/texture"),
            at("texture_sources/1/tint_map"),
            at("geometry_sources/0/texture"),
            at("geometry_sources/0/tint_map"),
        ]
    );
    let all = hat(
        &meta(members),
        &[
            geometry,
            ("c.png", ""),
            ("f.tga", ""),
            ("t0.png", ""),
            ("m0.tga", ""),
            ("t1.jpg", ""),
            ("m1.jpeg", ""),
            ("t2", ""),
            ("m2.png", ""),
        ],
    );
    assert_eq!(found(&all), Vec::<String>::new());
}

#[test]
fn m13_an_entry_the_game_skips_has_nothing_to_look_up() {
    // M17 already says the entry is skipped: its tint map is never read.
    let pieces = hat(
        &meta(r#""texture_sources": [{ "texture": "", "tint_map": "nope" }]"#),
        &[],
    );
    assert_eq!(
        found(&pieces),
        [format!(
            "info MissingRequiredField {META} texture_sources/0"
        )]
    );
}

#[test]
fn a_load_reads_only_the_files_it_parses() {
    let pack = piece_pack(&[
        (
            META,
            r#"{ "piece_id": "aa8d3c95-f189-cf08-0ae3-5cc1e090343b", "piece_name": "hat", "piece_type": "persona_emote", "texture_sources": [{ "texture": "hat", "tint_map": "hat_tintmap" }], "geometry_sources": [{ "geometry": "g" }], "animation_sources": [{ "name": "animation.wave", "animationFile": "wave.animation.json" }, { "name": "animation.wave", "animationFile": "wave.animation.json" }] }"#,
        ),
        ("pieces/hat/hat.png", "png"),
        ("pieces/hat/hat_tintmap.tga", "tga"),
        ("pieces/hat/hat.geometry.json", r#"{ "g": {} }"#),
        (
            "pieces/hat/hat.store_info.json",
            r#"{ "piece_store_name": "hat", "piece_thumbnail": "textures/thumb" }"#,
        ),
        (
            "pieces/hat/wave.animation.json",
            r#"{ "animations": { "animation.wave": {} } }"#,
        ),
        ("pieces/hat/texts/en_US.lang", "persona.offer.title=Wave\n"),
        (
            "pieces/hat/texts/de_DE.lang",
            "persona.offer.title=Winken\n",
        ),
        ("pieces/hat/unused.json", "{}"),
        ("textures/thumb.png", "png"),
        ("texts/en_US.lang", "a=b\n"),
        ("texts/de_DE.lang", "a=b\n"),
        ("contents.json", "{}"),
    ]);
    let watched = Watched::new(&pack, &[]);
    let pieces = PersonaPiece::load_all(&watched);
    assert_eq!(found(&pieces), Vec::<String>::new());
    let mut read = watched.read.borrow().clone();
    read.sort();
    assert_eq!(
        read,
        [
            "pieces/hat/hat.geometry.json",
            "pieces/hat/hat.meta.json",
            "pieces/hat/hat.store_info.json",
            "pieces/hat/texts/en_US.lang",
            // Named by two entries, read once.
            "pieces/hat/wave.animation.json",
        ]
    );
}

#[test]
fn the_pack_root_geometry_file_is_named_and_not_read() {
    // the client loads the pack-root `geometry.json` as a model pack. No
    // rule of a piece looks a name up in it, so the load names the file and leaves its
    // bytes (here: no JSON at all) to the geometry reader.
    let pack = piece_pack(&[
        (
            META,
            &meta(r#""geometry_sources": [{ "geometry": "geometry.shared" }]"#),
        ),
        ("geometry.json", "not read"),
        ("texts/en_US.lang", "persona.hat.title=Hat\n"),
    ]);
    let watched = Watched::new(&pack, &[]);
    let pieces = PersonaPiece::load_all(&watched);
    assert_eq!(
        pieces.shared_geometry.as_ref().map(PackPath::as_str),
        Some("geometry.json")
    );
    // A source's model is looked up in the piece's own geometry file alone (G3).
    assert_eq!(
        found(&pieces),
        [format!(
            "error UnresolvedReference {META} geometry_sources/0/geometry"
        )]
    );
    assert!(only(&pieces).geometry.is_none());
    let mut read = watched.read.borrow().clone();
    read.sort();
    assert_eq!(read, [META, "texts/en_US.lang"]);

    // Only that name, only at the root, only in a persona pack.
    let elsewhere = crate::common::load(&piece_pack(&[
        (META, &meta("")),
        ("Geometry.json", "{}"),
        ("geometry.JSON", "{}"),
        ("pieces/geometry.json", "{}"),
        ("pieces/hat/geometry.json", "{}"),
        ("texts/en_US.lang", "persona.hat.title=Hat\n"),
    ]));
    assert_eq!(elsewhere.shared_geometry, None);
    assert_eq!(found(&elsewhere), Vec::<String>::new());
    let built_in = crate::common::load(&builtin_pack(&[("geometry.json", "{}")]));
    assert_eq!(
        built_in.shared_geometry.as_ref().map(PackPath::as_str),
        Some("geometry.json")
    );
    let ordinary = crate::common::load(&crate::common::ordinary_pack(&[("geometry.json", "{}")]));
    assert_eq!(ordinary.shared_geometry, None);
    assert_eq!(found(&ordinary), Vec::<String>::new());
}

#[test]
fn the_pack_s_names_are_read_only_when_a_name_needs_them() {
    let pack = piece_pack(&[
        (META, &meta("")),
        ("texts/en_US.lang", "persona.hat.title=Hat\n"),
        ("texts/de_DE.lang", "a=b\n"),
    ]);
    let watched = Watched::new(&pack, &[]);
    let pieces = PersonaPiece::load_all(&watched);
    assert_eq!(found(&pieces), Vec::<String>::new());
    let mut read = watched.read.borrow().clone();
    read.sort();
    assert_eq!(read, [META, "texts/de_DE.lang", "texts/en_US.lang"]);
}

// ---- emotes -------------------------------------------------------------------------------

/// An emote piece with `sources` as its `animation_sources` and `files` beside it.
fn emote(sources: &str, files: &[(&str, &str)]) -> PersonaPieces {
    hat(
        &format!(
            r#"{{ "piece_id": "aa8d3c95-f189-cf08-0ae3-5cc1e090343b", "piece_name": "hat", "piece_type": "persona_emote", "animation_sources": {sources} }}"#
        ),
        files,
    )
}

const WAVE: &str = r#"[{ "name": "animation.wave", "animationFile": "wave.animation.json" }]"#;
/// A5: the game's level-3 line, after which the piece is kept: a kept error.
const A5: &str = "error MissingRequiredField pieces/hat/hat.meta.json animation_sources";

#[test]
fn pe01_an_emote_s_animation_is_stored_under_the_piece_s_id() {
    let pieces = emote(
        WAVE,
        &[(
            "wave.animation.json",
            r#"{ "format_version": "1.8.0", "animations": { "animation.wave": { "loop": true, "bones": { "hip": { "rotation": [0, 10, 0] }, "head": { "rotation": [1, 2, 3] } } }, "animation.other": { "bones": { "HIP": {} } } } }"#,
        )],
    );
    assert_eq!(found(&pieces), Vec::<String>::new());
    let piece = only(&pieces);
    assert_eq!(piece.animations.len(), 1);
    assert_eq!(piece.animations[0].name, "animation.wave");
    assert_eq!(
        piece.animations[0].stored_name,
        format!("animation.wave.{PIECE_PACK}")
    );
    assert_eq!(
        piece.animations[0].stored_name,
        "animation.wave.00000000-0000-4000-8000-000000000051"
    );
    assert_eq!(
        piece.animations[0].file.as_str(),
        "pieces/hat/wave.animation.json"
    );
    let emote = piece.emote.as_ref().expect("the emote's file");
    assert_eq!(emote.path.as_str(), "pieces/hat/wave.animation.json");
    let animations = emote
        .json
        .as_object()
        .unwrap()
        .get("animations")
        .and_then(|v| v.as_object())
        .unwrap();
    let keys: Vec<&str> = animations
        .keys()
        .map(facet_value::VString::as_str)
        .collect();
    assert_eq!(
        keys,
        [
            format!("animation.wave.{PIECE_PACK}"),
            format!("animation.other.{PIECE_PACK}")
        ]
    );
    let bones = |animation: &str| -> Vec<String> {
        animations
            .get(&format!("{animation}.{PIECE_PACK}"))
            .and_then(|v| v.as_object())
            .and_then(|v| v.get("bones"))
            .and_then(|v| v.as_object())
            .unwrap()
            .keys()
            .map(|key| key.as_str().to_owned())
            .collect()
    };
    assert_eq!(bones("animation.wave"), ["waist", "head"]);
    // Only the name `hip`, as written.
    assert_eq!(bones("animation.other"), ["HIP"]);
}

/// The channels of the bone `waist` of `animation.wave` in the emote file of `pieces`, as JSON
/// text, and the names of the animation's bones.
fn waist_of(pieces: &PersonaPieces) -> (String, Vec<String>) {
    let emote = only(pieces).emote.as_ref().expect("the emote's file");
    let bones = emote
        .json
        .as_object()
        .unwrap()
        .get("animations")
        .and_then(|v| v.as_object())
        .and_then(|v| v.get(&format!("animation.wave.{PIECE_PACK}")))
        .and_then(|v| v.as_object())
        .and_then(|v| v.get("bones"))
        .and_then(|v| v.as_object())
        .unwrap();
    let names = bones.keys().map(|key| key.as_str().to_owned()).collect();
    let waist = bedrock_addon::json::to_json(bones.get("waist").expect("a bone waist")).unwrap();
    (waist.split_whitespace().collect(), names)
}

#[test]
fn an_emote_with_a_hip_and_a_waist_bone_keeps_the_later_and_says_so() {
    // The folder says the bone `hip` is named `waist`, not what happens when the animation
    // has a `waist` already: the later of the two stands and the other is reported where it
    // was written.
    let pieces = emote(
        WAVE,
        &[(
            "wave.animation.json",
            r#"{ "animations": { "animation.wave": { "bones": { "hip": { "rotation": [1, 0, 0] }, "head": {}, "waist": { "rotation": [2, 0, 0] } } }, "animation.other": { "bones": { "waist": {}, "arm": {} } } } }"#,
        )],
    );
    assert_eq!(
        found(&pieces),
        ["info DuplicateBone pieces/hat/wave.animation.json animations/animation.wave/bones/hip"]
    );
    let (waist, names) = waist_of(&pieces);
    assert_eq!(waist, r#"{"rotation":[2,0,0]}"#);
    assert_eq!(names, ["waist", "head"]);
    let lost = pieces.report().next().unwrap();
    assert_eq!(lost.location.pack.as_ref().unwrap().id().uuid, PIECE_PACK);

    // The other order: `hip`, written later, is the bone `waist`.
    let pieces = emote(
        WAVE,
        &[(
            "wave.animation.json",
            r#"{ "animations": { "animation.wave": { "bones": { "waist": { "rotation": [2, 0, 0] }, "head": {}, "hip": { "rotation": [1, 0, 0] } } } } }"#,
        )],
    );
    assert_eq!(
        found(&pieces),
        ["info DuplicateBone pieces/hat/wave.animation.json animations/animation.wave/bones/waist"]
    );
    let (waist, names) = waist_of(&pieces);
    assert_eq!(waist, r#"{"rotation":[1,0,0]}"#);
    assert_eq!(names, ["waist", "head"]);
}

#[test]
fn every_animation_of_an_emote_s_file_is_renamed_and_says_what_it_lost() {
    // The rewrite is of the file (the fixed rewrites: bone `hip` → `waist`, every
    // `animations` key renamed), not of the one animation the meta file names: an animation
    // beside the emote's is stored with it and loses a bone the same way, with its finding.
    let pieces = emote(
        WAVE,
        &[(
            "wave.animation.json",
            r#"{ "animations": { "animation.wave": { "bones": { "hip": {}, "waist": {} } }, "animation.other": { "bones": { "waist": { "rotation": [2, 0, 0] }, "hip": { "rotation": [1, 0, 0] } } }, "animation.third": { "bones": { "hip": {} } } } }"#,
        )],
    );
    assert_eq!(
        found(&pieces),
        [
            "info DuplicateBone pieces/hat/wave.animation.json animations/animation.wave/bones/hip",
            "info DuplicateBone pieces/hat/wave.animation.json animations/animation.other/bones/waist",
        ]
    );
}

#[test]
fn pe01_a_built_in_emote_is_stored_under_its_piece_id() {
    let id = "4c8ae710-df2e-47cd-814d-cc7bf21a3d67";
    let pack = builtin_pack(&[
        (
            "pieces/emote_wave/overhead_wave.meta.json",
            &format!(
                r#"{{ "piece_id": "{}", "piece_name": "overhead_wave", "piece_type": "persona_emote", "animation_sources": [{{ "name": "animation.overhead_wave", "animationFile": "overhead_wave.animation.json" }}] }}"#,
                id.to_uppercase()
            ),
        ),
        (
            "pieces/emote_wave/overhead_wave.animation.json",
            r#"{ "format_version": "1.8.0", "animations": { "animation.overhead_wave": {} } }"#,
        ),
        (
            "pieces/emote_wave/texts/en_US.lang",
            "persona.offer.title=Over Here!\n",
        ),
    ]);
    assert_eq!(pack.id().uuid, BUILTIN_PACK);
    let pieces = crate::common::load(&pack);
    assert_eq!(found(&pieces), Vec::<String>::new());
    assert_eq!(
        only(&pieces).animations[0].stored_name,
        format!("animation.overhead_wave.{id}")
    );
}

#[cfg(feature = "animation")]
#[test]
fn pe01_the_emote_s_file_is_a_client_animation_file() {
    use bedrock_addon::document::Content;
    use bedrock_addon::families::animation::AnimationFile;
    use bedrock_addon::load::LoadContext;

    let pieces = emote(
        WAVE,
        &[(
            "wave.animation.json",
            r#"{ "format_version": "1.8.0", "animations": { "animation.wave": { "loop": true, "bones": { "hip": { "rotation": [0, 10, 0] } } } } }"#,
        )],
    );
    let emote = only(&pieces).emote.as_ref().unwrap();
    let text = bedrock_addon::json::to_json(&emote.json).unwrap();
    let cx = emote.context(&LoadContext::detached());
    let document = AnimationFile::parse(&emote.path, text.as_bytes(), &cx)
        .unwrap_or_else(|rejected| panic!("{rejected:?}"));
    assert!(!document.diagnostics().has_errors());
    let animation = document
        .value()
        .animations
        .get(&format!("animation.wave.{PIECE_PACK}"))
        .expect("the renamed animation");
    let bones: Vec<&str> = animation
        .bones
        .iter()
        .flatten()
        .map(|(name, _)| name.as_str())
        .collect();
    assert_eq!(bones, ["waist"]);
}

/// A1 and A4 are the emote parser's level-3 lines,
/// after which the piece is kept: kept errors.
#[test]
fn a1_and_a4_are_kept_errors() {
    let missing = emote(WAVE, &[]);
    let unnamed = emote(
        WAVE,
        &[(
            "wave.animation.json",
            r#"{ "animations": { "animation.other": {} } }"#,
        )],
    );
    for pieces in [&missing, &unnamed] {
        assert_eq!(pieces.pieces.len(), 1);
        let errors: Vec<_> = pieces
            .report()
            .filter(|found| found.severity == Severity::Error)
            .collect();
        // The entry's line alone: A5 is for an emote with no entry.
        assert_eq!(errors.len(), 1, "{:#?}", lines(pieces));
        assert!(errors.iter().all(|found| found.kept), "{errors:#?}");
    }
}

#[test]
fn pe02_a1_an_animation_file_that_is_not_there() {
    let pieces = emote(WAVE, &[]);
    assert_eq!(
        found(&pieces),
        [format!(
            "error ReadError {META} animation_sources/0/animationFile"
        )]
    );
    assert_eq!(
        text_of(&pieces, "ReadError"),
        format!(
            "Trying to parse emote {PIECE_PACK} with missing anim file: pieces/hat/wave.animation.json"
        )
    );
    assert!(only(&pieces).animations.is_empty());
    assert!(only(&pieces).emote.is_none());
}

#[test]
fn a1_an_animation_file_that_cannot_be_read() {
    let pack = piece_pack(&[
        (
            META,
            &format!(
                r#"{{ "piece_id": "aa8d3c95-f189-cf08-0ae3-5cc1e090343b", "piece_name": "hat", "piece_type": "persona_emote", "animation_sources": {WAVE} }}"#
            ),
        ),
        (
            "pieces/hat/wave.animation.json",
            r#"{ "animations": { "animation.wave": {} } }"#,
        ),
        ("texts/en_US.lang", "persona.hat.title=Hat\n"),
    ]);
    let watched = Watched::new(&pack, &["pieces/hat/wave.animation.json"]);
    let pieces = PersonaPiece::load_all(&watched);
    assert_eq!(
        found(&pieces),
        [format!(
            "error ReadError {META} animation_sources/0/animationFile"
        )]
    );
    assert!(pieces.report().any(|found| matches!(
        found.code,
        DiagnosticCode::ReadError {
            kind: std::io::ErrorKind::PermissionDenied,
            ..
        }
    )));
}

#[test]
fn a1_an_animation_file_the_pack_does_not_list_is_never_asked_for() {
    let pack = piece_pack(&[
        (
            META,
            &format!(
                r#"{{ "piece_id": "aa8d3c95-f189-cf08-0ae3-5cc1e090343b", "piece_name": "hat", "piece_type": "persona_emote", "animation_sources": {WAVE} }}"#
            ),
        ),
        ("texts/en_US.lang", "persona.hat.title=Hat\n"),
    ]);
    // A source that would answer otherwise if it were asked.
    let watched = Watched::new(&pack, &["pieces/hat/wave.animation.json"]);
    let pieces = PersonaPiece::load_all(&watched);
    assert_eq!(
        found(&pieces),
        [format!(
            "error ReadError {META} animation_sources/0/animationFile"
        )]
    );
    assert!(pieces.report().any(|found| matches!(
        found.code,
        DiagnosticCode::ReadError {
            kind: std::io::ErrorKind::NotFound,
            ..
        }
    )));
    assert_eq!(*watched.read.borrow(), [META, "texts/en_US.lang"]);
}

/// 1.26.60 has no line for the file itself: the emote parser loads it and, holding no
/// animation of the name, logs A4.
#[test]
fn a2_an_animation_file_that_is_no_json_object_is_told_as_a4() {
    for text in ["[]", "{", "3", ""] {
        let pieces = emote(WAVE, &[("wave.animation.json", text)]);
        assert_eq!(
            found(&pieces),
            [format!(
                "error UnresolvedReference {META} animation_sources/0/name"
            )],
            "{text}"
        );
        assert_eq!(
            text_of(&pieces, "UnresolvedReference"),
            format!(
                "Animation animation.wave declared by emote piece {PIECE_PACK} was not found in file pieces/hat/wave.animation.json"
            )
        );
    }
}

/// As a file that is no JSON object: told as A4.
#[test]
fn pe03_a3_an_animation_file_without_animations_is_told_as_a4() {
    for text in [
        r#"{ "format_version": "1.8.0" }"#,
        r#"{ "format_version": "1.8.0", "animations": null }"#,
    ] {
        let pieces = emote(WAVE, &[("wave.animation.json", text)]);
        assert_eq!(
            found(&pieces),
            [format!(
                "error UnresolvedReference {META} animation_sources/0/name"
            )],
            "{text}"
        );
    }
}

#[test]
fn pe04_a4_a_name_that_is_no_animation_of_the_file() {
    let pieces = emote(
        r#"[{ "name": "animation.nope", "animationFile": "wave.animation.json" }]"#,
        &[(
            "wave.animation.json",
            r#"{ "animations": { "animation.wave": {} } }"#,
        )],
    );
    assert_eq!(
        found(&pieces),
        [format!(
            "error UnresolvedReference {META} animation_sources/0/name"
        )]
    );
    assert_eq!(
        text_of(&pieces, "UnresolvedReference"),
        format!(
            "Animation animation.nope declared by emote piece {PIECE_PACK} was not found in file pieces/hat/wave.animation.json"
        )
    );
    // `animations` that is no object holds no animation.
    let pieces = emote(WAVE, &[("wave.animation.json", r#"{ "animations": [] }"#)]);
    assert_eq!(
        found(&pieces),
        [format!(
            "error UnresolvedReference {META} animation_sources/0/name"
        )]
    );
}

#[test]
fn pe05_a5_an_emote_without_animation_sources() {
    let pieces = hat(
        r#"{ "piece_id": "aa8d3c95-f189-cf08-0ae3-5cc1e090343b", "piece_name": "hat", "piece_type": "persona_emote" }"#,
        &[],
    );
    assert_eq!(found(&pieces), [A5]);
    let pieces = emote("[]", &[]);
    assert_eq!(found(&pieces), [A5]);
    // The piece is kept: it is the emote that cannot be played. The emote parser logs
    // "Trying to parse invalid emote animation definition with pieceId %s" at level 3 and
    // returns: a kept error.
    assert_eq!(pieces.pieces.len(), 1);
    let a5 = pieces
        .report()
        .find(|found| crate::common::line(found) == A5)
        .expect("the A5 finding");
    assert!(a5.kept && !a5.rejects(), "{a5:?}");
    assert_eq!(
        message(a5),
        format!("Trying to parse invalid emote animation definition with pieceId {PIECE_PACK}")
    );
}

#[test]
fn pe06_the_key_is_animation_file_in_camel_case() {
    // `animation_file` is no member of an animation source, and `animationFile` is required.
    let pieces = emote(
        r#"[{ "name": "animation.wave", "animation_file": "wave.animation.json" }]"#,
        &[(
            "wave.animation.json",
            r#"{ "animations": { "animation.wave": {} } }"#,
        )],
    );
    assert_eq!(
        crate::common::refused_at(&pieces, META),
        [
            "error UnrecognizedField animation_sources/0/animation_file",
            "error MissingRequiredField animation_sources/0/animationFile",
        ]
    );
    // An empty name is joined to the folder, as the game joins it.
    let pieces = emote(
        r#"[{ "name": "animation.wave", "animationFile": "" }]"#,
        &[(
            "wave.animation.json",
            r#"{ "animations": { "animation.wave": {} } }"#,
        )],
    );
    assert_eq!(
        text_of(&pieces, "ReadError"),
        format!("Trying to parse emote {PIECE_PACK} with missing anim file: pieces/hat/")
    );
}

/// The emote parser checks and plays the first entry alone: a
/// first entry that fails is told and leaves no emote, a later one that fails is not told.
#[test]
fn of_several_animations_the_first_alone_is_checked_and_played() {
    let pieces = emote(
        r#"[{ "name": "animation.nope", "animationFile": "a.animation.json" }, { "name": "animation.b", "animationFile": "b.animation.json" }, { "name": "animation.a", "animationFile": "a.animation.json" }, { "name": "animation.c", "animationFile": "c.animation.json" }]"#,
        &[
            (
                "a.animation.json",
                r#"{ "animations": { "animation.a": {} } }"#,
            ),
            (
                "b.animation.json",
                r#"{ "animations": { "animation.b": {} } }"#,
            ),
        ],
    );
    assert_eq!(
        found(&pieces),
        [format!(
            "error UnresolvedReference {META} animation_sources/0/name"
        )]
    );
    let piece = only(&pieces);
    let names: Vec<&str> = piece
        .animations
        .iter()
        .map(|animation| animation.name.as_str())
        .collect();
    assert_eq!(names, ["animation.b", "animation.a"]);
    // The first entry failed: no emote, the later ones are not tried.
    assert!(piece.emote.is_none());
    // A good first entry is the emote; a later entry that fails says nothing.
    let pieces = emote(
        r#"[{ "name": "animation.a", "animationFile": "a.animation.json" }, { "name": "animation.nope", "animationFile": "a.animation.json" }, { "name": "animation.c", "animationFile": "c.animation.json" }]"#,
        &[(
            "a.animation.json",
            r#"{ "animations": { "animation.a": {} } }"#,
        )],
    );
    assert!(found(&pieces).is_empty(), "{:#?}", found(&pieces));
    assert_eq!(
        only(&pieces)
            .emote
            .as_ref()
            .map(|emote| emote.path.as_str()),
        Some("pieces/hat/a.animation.json")
    );
}

/// The emote parser runs for an emote alone: the animations of
/// a piece of another type are not checked and not told.
#[test]
fn the_animations_of_a_piece_that_is_no_emote_are_not_told() {
    let pieces = hat(
        &format!(
            r#"{{ "piece_id": "aa8d3c95-f189-cf08-0ae3-5cc1e090343b", "piece_name": "hat", "piece_type": "persona_capes", "animation_sources": {WAVE} }}"#
        ),
        &[("wave.animation.json", "[]")],
    );
    // The member's own note (the meta reader ignores it for this type) and nothing else.
    assert_eq!(
        found(&pieces),
        ["info IgnoredMember pieces/hat/hat.meta.json animation_sources"]
    );
}

#[cfg(all(feature = "animation", feature = "molang"))]
#[test]
fn pe07_an_emote_s_molang_is_of_version_0() {
    use bedrock_addon::document::Content;
    use bedrock_addon::families::animation::AnimationFile;
    use bedrock_addon::families::persona::EMOTE_MOLANG_VERSION;
    use bedrock_addon::load::LoadContext;
    use bedrock_addon::molang::{MolangVersion, RawVersion};
    use bedrock_addon::version::MinEngineVersion;

    assert_eq!(EMOTE_MOLANG_VERSION, MolangVersion::V0);
    let animation = r#"{ "format_version": "1.8.0", "animations": { "animation.wave": { "blend_weight": "query.is_on_screen ? 1 : 0", "anim_time_update": "query.distance_from_camera", "start_delay": "'a' + 1", "bones": { "head": { "rotation": ["math.sin(query.anim_time * 180) * 10", 0, 0] } } } } }"#;
    // The piece pack's manifest says 1.21.0.
    let pack = crate::common::open(
        &crate::common::manifest(PIECE_PACK, "persona_piece", Some("[1, 21, 0]")),
        &[
            (
                META,
                &format!(
                    r#"{{ "piece_id": "aa8d3c95-f189-cf08-0ae3-5cc1e090343b", "piece_name": "hat", "piece_type": "persona_emote", "animation_sources": {WAVE} }}"#
                ),
            ),
            ("pieces/hat/wave.animation.json", animation),
            ("texts/en_US.lang", "persona.hat.title=Hat\n"),
        ],
    );
    let pieces = PersonaPiece::load_all(&OfPack(&pack));
    assert_eq!(found(&pieces), Vec::<String>::new());
    let emote = only(&pieces).emote.as_ref().unwrap();
    let text = bedrock_addon::json::to_json(&emote.json).unwrap();

    // The pack's own context: Molang version 13 of 1.21.0.
    let mut of_pack = LoadContext::detached();
    of_pack.pack_type = bedrock_addon::pack::PackType::Resource;
    let v1_21 = MinEngineVersion::new(1, 21, 0);
    of_pack.molang_version = RawVersion::from(v1_21.molang_version());
    of_pack.min_engine_version = Some(v1_21);
    of_pack.molang = Some(std::sync::Arc::new(
        bedrock_addon::molang::MolangCheck::default(),
    ));

    // As an emote: version 0, where adding to a string is not yet refused.
    let cx = emote.context(&of_pack);
    assert_eq!(cx.molang_version, RawVersion::from(MolangVersion::V0));
    assert_eq!(cx.min_engine_version, Some(MinEngineVersion::new(1, 14, 0)));
    // A client animation, whatever the type of the pack the piece is in.
    let mut of_piece_pack = of_pack.clone();
    of_piece_pack.pack_type = bedrock_addon::pack::PackType::PersonaPiece;
    assert_eq!(
        emote.context(&of_piece_pack).pack_type,
        bedrock_addon::pack::PackType::Resource
    );
    let as_emote = AnimationFile::parse(&emote.path, text.as_bytes(), &cx)
        .unwrap_or_else(|rejected| panic!("{rejected:?}"));
    assert_eq!(
        as_emote
            .diagnostics()
            .iter()
            .map(crate::common::line)
            .collect::<Vec<_>>(),
        Vec::<String>::new()
    );

    // The same file as an ordinary client animation of that pack is refused.
    let ordinary = AnimationFile::parse(&emote.path, text.as_bytes(), &of_pack);
    let findings: Vec<String> = match &ordinary {
        Ok(document) => document
            .diagnostics()
            .iter()
            .map(crate::common::line)
            .collect(),
        Err(rejected) => rejected
            .diagnostics
            .iter()
            .map(crate::common::line)
            .collect(),
    };
    assert!(
        findings
            .iter()
            .any(|line| line.starts_with("error InvalidMolang")),
        "{findings:?}"
    );
}

// ---- store info and texts -----------------------------------------------------------------

/// A hair piece named `a` with the store-info `store`, in a pack that names `a` and `b`.
fn with_store(store: &str) -> PersonaPieces {
    let pack = piece_pack(&[
        (
            META,
            r#"{ "piece_id": "aa8d3c95-f189-cf08-0ae3-5cc1e090343b", "piece_name": "a", "piece_type": "persona_hair" }"#,
        ),
        ("pieces/hat/hat.store_info.json", store),
        ("textures/x.png", "png"),
        ("texts/en_US.lang", "persona.a.title=A\npersona.b.title=B\n"),
    ]);
    crate::common::load(&pack)
}

const STORE: &str = "pieces/hat/hat.store_info.json";

#[test]
fn ps01_the_store_name_replaces_the_piece_name() {
    let pieces = with_store(r#"{ "piece_store_name": "b", "piece_thumbnail": "textures/x" }"#);
    assert_eq!(found(&pieces), Vec::<String>::new());
    let piece = only(&pieces);
    assert_eq!(piece.name_key(), "persona.b.title");
    assert_eq!(piece.thumbnail(), Some("textures/x"));
    assert_eq!(
        piece.store_info.as_ref().unwrap().origin().path.as_str(),
        STORE
    );
}

/// The client ignores it in silence: an information of ours.
#[test]
fn ps02_s1_a_store_info_that_is_no_json_object_is_ignored() {
    for store in ["{", "[]", "3", "", "null"] {
        let pieces = with_store(store);
        assert_eq!(
            found(&pieces),
            [format!("info MalformedInput {STORE}")],
            "{store}"
        );
        let piece = only(&pieces);
        assert!(piece.store_info.is_none());
        assert_eq!(piece.name_key(), "persona.a.title");
        assert_eq!(piece.thumbnail(), None);
    }
}

#[test]
fn a_store_info_that_cannot_be_read_is_ignored() {
    let pack = piece_pack(&[
        (META, &meta("")),
        (STORE, r#"{ "piece_store_name": "b" }"#),
        ("texts/en_US.lang", "persona.hat.title=Hat\n"),
    ]);
    let watched = Watched::new(&pack, &[STORE]);
    let pieces = PersonaPiece::load_all(&watched);
    // The game ignores it in silence: an information of ours.
    assert_eq!(found(&pieces), [format!("info ReadError {STORE}")]);
    assert_eq!(only(&pieces).name_key(), "persona.hat.title");
}

#[test]
fn ps03_s2_the_thumbnail_with_and_without_its_extension() {
    for thumbnail in ["textures/x", "textures/x.png"] {
        let pieces = with_store(&format!(
            r#"{{ "piece_store_name": "b", "piece_thumbnail": "{thumbnail}" }}"#
        ));
        assert_eq!(found(&pieces), Vec::<String>::new(), "{thumbnail}");
    }
    for thumbnail in ["textures/y", "x", "textures/x.tga", "pieces/hat/x", ""] {
        let pieces = with_store(&format!(
            r#"{{ "piece_store_name": "b", "piece_thumbnail": "{thumbnail}" }}"#
        ));
        assert_eq!(
            found(&pieces),
            [format!("info UnresolvedReference {STORE} piece_thumbnail")],
            "{thumbnail}"
        );
    }
}

#[test]
fn ps04_s4_a_piece_s_texts_without_en_us() {
    let other = hat(
        &meta(""),
        &[("texts/de_DE.lang", "persona.offer.title=Hut\n")],
    );
    assert_eq!(found(&other), ["info IgnoredFile pieces/hat/texts"]);
    assert!(other.report().any(
        |found| matches!(found.code, DiagnosticCode::IgnoredFile { .. })
            && found.severity == Severity::Info
    ));
    assert_eq!(only(&other).texts, None);
    assert_eq!(only(&other).offer_title, None);

    let own = hat(
        &meta(""),
        &[
            ("texts/de_DE.lang", "persona.offer.title=Hut\n"),
            ("texts/en_US.lang", "persona.offer.title=Hat of hats\n"),
        ],
    );
    assert_eq!(found(&own), Vec::<String>::new());
    assert_eq!(
        only(&own).texts.as_ref().map(PackPath::as_str),
        Some("pieces/hat/texts")
    );
    assert_eq!(only(&own).offer_title.as_deref(), Some("Hat of hats"));
}

#[test]
fn ps05_s5_a_store_info_blanks_what_it_does_not_set() {
    let pieces = with_store(r#"{ "piece_thumbnail": "textures/x" }"#);
    assert_eq!(
        found(&pieces),
        [
            format!("info MissingRequiredField {STORE} piece_store_name"),
            // The blank name has no translation.
            format!("info UnresolvedLangKey {META}"),
        ]
    );
    let piece = only(&pieces);
    assert_eq!(piece.name_key(), "persona..title");

    let pieces = with_store(r#"{ "piece_store_name": "b" }"#);
    assert_eq!(
        found(&pieces),
        [format!("info MissingRequiredField {STORE} piece_thumbnail")]
    );
    assert_eq!(only(&pieces).thumbnail(), Some(""));

    let pieces = with_store("{}");
    assert_eq!(
        found(&pieces),
        [
            format!("info MissingRequiredField {STORE} piece_store_name"),
            format!("info MissingRequiredField {STORE} piece_thumbnail"),
            format!("info UnresolvedLangKey {META}"),
        ]
    );
}

#[test]
fn ps06_the_name_key_is_built_once() {
    for name in ["hat", "persona.hat", "hat.title", "persona.hat.title"] {
        assert_eq!(name_key(name), "persona.hat.title", "{name}");
        let pieces = hat(
            &format!(
                r#"{{ "piece_id": "aa8d3c95-f189-cf08-0ae3-5cc1e090343b", "piece_name": "{name}", "piece_type": "persona_hair" }}"#
            ),
            &[],
        );
        assert_eq!(found(&pieces), Vec::<String>::new());
        assert_eq!(only(&pieces).name_key(), "persona.hat.title");
    }
    assert_eq!(name_key(""), "persona..title");
    assert_eq!(
        name_key("dr.default.emote.overheadWave"),
        "persona.dr.default.emote.overheadWave.title"
    );
    // "contains", wherever it is.
    assert_eq!(name_key("x.persona.y"), "x.persona.y.title");
    assert_eq!(name_key("a.title.b"), "persona.a.title.b");
}

#[test]
fn s3_a_name_no_text_translates() {
    let piece = (
        META,
        r#"{ "piece_id": "aa8d3c95-f189-cf08-0ae3-5cc1e090343b", "piece_name": "wig", "piece_type": "persona_hair" }"#,
    );
    let unnamed = piece_pack(&[piece, ("texts/en_US.lang", "persona.hat.title=Hat\n")]);
    let pieces = crate::common::load(&unnamed);
    assert_eq!(found(&pieces), [format!("info UnresolvedLangKey {META}")]);
    let finding = pieces.report().next().unwrap();
    assert!(matches!(finding.code, DiagnosticCode::UnresolvedLangKey));
    assert!(matches!(
        finding.detail.as_deref(),
        Some(DiagnosticDetail::Reference { id, .. }) if id == "persona.wig.title"
    ));

    // Any `.lang` of the pack names it.
    let named = piece_pack(&[
        piece,
        ("texts/en_US.lang", "persona.hat.title=Hat\n"),
        (
            "texts/de_DE.lang",
            "## a comment\n\npersona.wig.title=Per\u{fc}cke\t# note\n",
        ),
    ]);
    assert_eq!(found(&crate::common::load(&named)), Vec::<String>::new());

    // A text the caller supplies (the vanilla resource pack's) names it.
    let supplied = PersonaPiece::load_all_with_texts(
        &OfPack(&unnamed),
        &[b"a=b\n", b"persona.wig.title=Wig\n"],
    );
    assert_eq!(found(&supplied), Vec::<String>::new());

    // The piece's own offer title stands in for it.
    let own = piece_pack(&[
        piece,
        ("pieces/hat/texts/en_US.lang", "persona.offer.title=Wig\n"),
    ]);
    let pieces = crate::common::load(&own);
    assert_eq!(found(&pieces), Vec::<String>::new());
    assert_eq!(only(&pieces).offer_title.as_deref(), Some("Wig"));

    // Its own texts without that key do not.
    let other = piece_pack(&[piece, ("pieces/hat/texts/en_US.lang", "persona.other=x\n")]);
    assert_eq!(
        found(&crate::common::load(&other)),
        [format!("info UnresolvedLangKey {META}")]
    );
}

#[test]
fn s5_a_member_of_another_json_type_is_not_a_missing_one() {
    // It has its own finding; the name is blank all the same.
    let pieces = with_store(r#"{ "piece_store_name": 1, "piece_thumbnail": "textures/x" }"#);
    assert_eq!(
        found(&pieces),
        [
            format!("info InvalidType {STORE} piece_store_name"),
            format!("info UnresolvedLangKey {META}"),
        ]
    );
    assert_eq!(only(&pieces).name_key(), "persona..title");
    // `null` is absent to the game.
    let pieces = with_store(r#"{ "piece_store_name": "b", "piece_thumbnail": null }"#);
    assert_eq!(
        found(&pieces),
        [format!("info MissingRequiredField {STORE} piece_thumbnail")]
    );
    // An unknown member is kept and named.
    let pieces =
        with_store(r#"{ "piece_store_name": "b", "piece_thumbnail": "textures/x", "price": 3 }"#);
    assert_eq!(
        found(&pieces),
        [format!("info UnrecognizedField {STORE} price")]
    );
    let store = only(&pieces).store_info.as_ref().unwrap().value();
    assert!(store.extra.get("price").is_some());
}

#[test]
fn a_piece_at_the_pack_root_looks_its_files_up_there() {
    let pack = builtin_pack(&[
        (
            "root.meta.json",
            r#"{ "piece_id": "4c8ae710-df2e-47cd-814d-cc7bf21a3d67", "piece_name": "hat", "piece_type": "persona_emote", "texture_sources": [{ "texture": "hat" }], "animation_sources": [{ "name": "animation.wave", "animationFile": "wave.animation.json" }] }"#,
        ),
        ("hat.png", "png"),
        (
            "wave.animation.json",
            r#"{ "animations": { "animation.wave": {} } }"#,
        ),
        ("texts/en_US.lang", "persona.hat.title=Hat\n"),
    ]);
    let pieces = crate::common::load(&pack);
    assert_eq!(found(&pieces), Vec::<String>::new());
    let piece = only(&pieces);
    assert_eq!(piece.directory.as_str(), "");
    assert_eq!(piece.animations[0].file.as_str(), "wave.animation.json");
    // The pack's `texts/en_US.lang` is the root piece's own (step 2a keys it to the root).
    assert_eq!(piece.texts.as_ref().map(PackPath::as_str), Some("texts"));
    assert_eq!(piece.offer_title, None);
}

#[test]
fn a_piece_at_the_pack_root_has_the_root_texts_as_its_own() {
    // `texts/en_US.lang` fills the `loc` slot of its grandparent, which for
    // the pack's own `texts/` is the pack root.
    let pack = builtin_pack(&[
        (
            "root.meta.json",
            r#"{ "piece_id": "4c8ae710-df2e-47cd-814d-cc7bf21a3d67", "piece_name": "hat", "piece_type": "persona_hair" }"#,
        ),
        (
            "pieces/cap/cap.meta.json",
            r#"{ "piece_id": "5c8ae710-df2e-47cd-814d-cc7bf21a3d67", "piece_name": "cap", "piece_type": "persona_hair" }"#,
        ),
        ("texts/en_US.lang", "persona.offer.title=The Hat\n"),
    ]);
    let pieces = crate::common::load(&pack);
    // The root piece is shown by its offer title (no S3); a piece of another folder is not.
    assert_eq!(
        found(&pieces),
        ["info UnresolvedLangKey pieces/cap/cap.meta.json"]
    );
    let root = &pieces.pieces[0];
    assert_eq!(root.directory.as_str(), "");
    assert_eq!(root.texts.as_ref().map(PackPath::as_str), Some("texts"));
    assert_eq!(root.offer_title.as_deref(), Some("The Hat"));
    let cap = &pieces.pieces[1];
    assert_eq!(cap.texts, None);
    assert_eq!(cap.offer_title, None);

    // Without an `en_US.lang` the root `texts/` is no piece's own: the pack's names only.
    let pack = builtin_pack(&[
        (
            "root.meta.json",
            r#"{ "piece_id": "4c8ae710-df2e-47cd-814d-cc7bf21a3d67", "piece_name": "hat", "piece_type": "persona_hair" }"#,
        ),
        ("texts/de_DE.lang", "persona.offer.title=Der Hut\n"),
    ]);
    let pieces = crate::common::load(&pack);
    assert_eq!(found(&pieces), ["info UnresolvedLangKey root.meta.json"]);
    assert_eq!(only(&pieces).texts, None);
    assert_eq!(only(&pieces).offer_title, None);
}

// ---- nesting ------------------------------------------------------------------------------

/// `{ "deep": [[…]] }` with `arrays` arrays inside one another.
fn nested(arrays: usize) -> String {
    format!(
        r#"{{ "deep": {}1{} }}"#,
        "[".repeat(arrays),
        "]".repeat(arrays)
    )
}

#[test]
fn a_file_nested_past_the_reader_s_bound_is_no_json() {
    use bedrock_addon::json::MAX_JSON_DEPTH;

    // At the bound (the root object and 127 arrays) each file is read, held and dropped.
    let at = nested(MAX_JSON_DEPTH - 1);
    let deep_meta = format!(
        r#"{{ "piece_id": "aa8d3c95-f189-cf08-0ae3-5cc1e090343b", "piece_name": "hat", "piece_type": "persona_emote", "animation_sources": {WAVE}, "deep": {}1{} }}"#,
        "[".repeat(MAX_JSON_DEPTH - 1),
        "]".repeat(MAX_JSON_DEPTH - 1)
    );
    let animation = format!(
        r#"{{ "animations": {{ "animation.wave": {{}} }}, "deep": {}1{} }}"#,
        "[".repeat(MAX_JSON_DEPTH - 1),
        "]".repeat(MAX_JSON_DEPTH - 1)
    );
    let emote_meta = format!(
        r#"{{ "piece_id": "aa8d3c95-f189-cf08-0ae3-5cc1e090343b", "piece_name": "hat", "piece_type": "persona_emote", "animation_sources": {WAVE} }}"#
    );
    let pieces = hat(
        &emote_meta,
        &[
            ("hat.geometry.json", &at),
            ("hat.store_info.json", &at),
            ("wave.animation.json", &animation),
        ],
    );
    assert_eq!(
        found(&pieces),
        [
            "info UnrecognizedField pieces/hat/hat.store_info.json deep".to_owned(),
            "info MissingRequiredField pieces/hat/hat.store_info.json piece_store_name".to_owned(),
            "info MissingRequiredField pieces/hat/hat.store_info.json piece_thumbnail".to_owned(),
            "info UnresolvedLangKey pieces/hat/hat.meta.json".to_owned(),
        ]
    );
    let piece = only(&pieces);
    assert!(piece.geometry.as_ref().unwrap().model("deep").is_some());
    assert!(piece.emote.is_some());
    // A meta file at the bound is read: its deep member is an unknown one, which the 1.26.40+
    // reader refuses (not a reader that gave up on the depth).
    let pieces = hat(&deep_meta, &[("wave.animation.json", &animation)]);
    assert_eq!(
        crate::common::refused_at(&pieces, META),
        ["error UnrecognizedField deep"]
    );
    drop(pieces);

    // One level more: no JSON to the reader, in each of the four files.
    let past = nested(MAX_JSON_DEPTH);
    let pieces = hat(&past, &[]);
    assert_eq!(found(&pieces), [format!("error MalformedInput {META}")]);
    assert!(pieces.pieces.is_empty());
    // The reader's message names the byte of the bracket that is one too deep: after
    // `{ "deep": ` (10 bytes) and the 127 arrays the bound allows.
    let reader_s = |pieces: &PersonaPieces| -> Vec<String> {
        pieces
            .report()
            .filter_map(|found| match &found.code {
                DiagnosticCode::MalformedInput { message, .. } => Some(message.clone()),
                _ => None,
            })
            .collect()
    };
    let too_deep = format!("nesting too deep at byte {}", 10 + MAX_JSON_DEPTH - 1);
    assert_eq!(reader_s(&pieces), std::slice::from_ref(&too_deep));
    assert_eq!(
        message(pieces.report().next().unwrap()),
        format!("Could not parse metafile json file {META}")
    );
    let emote_meta = format!(
        r#"{{ "piece_id": "aa8d3c95-f189-cf08-0ae3-5cc1e090343b", "piece_name": "hat", "piece_type": "persona_emote", "animation_sources": {WAVE} }}"#
    );
    let pieces = hat(
        &emote_meta,
        &[
            ("hat.geometry.json", &past),
            ("hat.store_info.json", &past),
            ("wave.animation.json", &past),
        ],
    );
    assert_eq!(
        found(&pieces),
        [
            "error MalformedInput pieces/hat/hat.geometry.json",
            "error UnresolvedReference pieces/hat/hat.meta.json animation_sources/0/name",
            "info MalformedInput pieces/hat/hat.store_info.json",
        ]
    );
    // The animation file is told as A4, without the reader's message.
    assert_eq!(reader_s(&pieces), [too_deep.clone(), too_deep]);
    assert_eq!(pieces.pieces.len(), 1);
}
