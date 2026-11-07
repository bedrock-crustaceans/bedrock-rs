//! P7), M4, M5 and M7.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::families::persona::{PackKind, PieceType, TESTING_PACK};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{PackOrigin, PackRepository, PackType, WorldPacks};
use uuid::Uuid;

use bedrock_addon::families::persona::PersonaPiece;

use crate::common::{
    ORDINARY_PACK, PIECE_PACK, Reversed, builtin_pack, lines, load, manifest, only, open,
    ordinary_pack, piece_pack,
};

/// A meta file of a hair piece named `hat` with `members` after its name and type.
pub fn meta(members: &str) -> String {
    let comma = if members.is_empty() { "" } else { ", " };
    // `piece_id` is required from 1.26.40: a made-up one unless `members` has its own.
    let id = if members.contains(r#""piece_id""#) {
        ""
    } else {
        r#""piece_id": "aa8d3c95-f189-cf08-0ae3-5cc1e090343b", "#
    };
    format!(r#"{{ {id}"piece_name": "hat", "piece_type": "persona_hair"{comma}{members} }}"#)
}

/// The pack-wide names: `hat`, and the store name of pp01.
const NAMES: &str = "persona.hat.title=Hat\npersona.cool_hat.title=Cool Hat\n";

#[test]
fn pp01_a_single_piece_is_identified_by_its_pack() {
    let pack = piece_pack(&[
        (
            "pieces/hat/hat.meta.json",
            &meta(r#""texture_sources": [{ "texture": "hat.png" }]"#),
        ),
        (
            "pieces/hat/hat.store_info.json",
            r#"{ "piece_store_name": "cool_hat", "piece_thumbnail": "textures/persona_thumbnails/hat" }"#,
        ),
        ("pieces/hat/hat.png", "png"),
        ("textures/persona_thumbnails/hat.png", "png"),
        ("texts/en_US.lang", NAMES),
    ]);
    assert_eq!(pack.pack_type(), PackType::PersonaPiece);
    let pieces = load(&pack);
    assert_eq!(pieces.kind, PackKind::PersonaPiece);
    assert_eq!(lines(&pieces), Vec::<String>::new());
    let piece = only(&pieces);
    assert_eq!(piece.id, PIECE_PACK);
    assert_eq!(piece.directory.as_str(), "pieces/hat");
    assert_eq!(piece.piece_type(), PieceType::Hair);
    assert_eq!(piece.piece_type().value(), 14);
    assert_eq!(piece.name_key(), "persona.cool_hat.title");
    assert_eq!(
        piece.meta.origin().path.as_str(),
        "pieces/hat/hat.meta.json"
    );
    assert!(pieces.rejected.is_empty());
}

#[test]
fn pp02_m5_a_piece_id_in_a_piece_pack_is_ignored_deviation() {
    let other = "11111111-2222-4333-8444-555555555555";
    let pack = piece_pack(&[
        (
            "pieces/hat/hat.meta.json",
            &meta(&format!(r#""piece_id": "{other}""#)),
        ),
        ("texts/en_US.lang", NAMES),
    ]);
    let pieces = load(&pack);
    // The member is required from 1.26.40, so its being ignored is no finding.
    assert_eq!(lines(&pieces), Vec::<String>::new());
    assert_eq!(only(&pieces).id, PIECE_PACK);
}

#[test]
fn m5_a_piece_id_of_the_wrong_type_invalidates_the_piece() {
    // The 1.26.40+ reader types the member as a string before the pack's kind is looked at.
    let pack = piece_pack(&[
        ("pieces/hat/hat.meta.json", &meta(r#""piece_id": 7"#)),
        ("texts/en_US.lang", NAMES),
    ]);
    assert_eq!(
        lines(&load(&pack)),
        [
            "error core/invalid_type pieces/hat/hat.meta.json piece_id",
            "error persona/meta_not_parsed pieces/hat/hat.meta.json"
        ]
    );
}

#[test]
fn pp03_p4_a_piece_pack_with_two_piece_folders_keeps_one() {
    let pack = piece_pack(&[
        ("pieces/b/b.meta.json", &meta("")),
        ("pieces/a/a.meta.json", &meta("")),
        ("texts/en_US.lang", NAMES),
    ]);
    let pieces = load(&pack);
    assert_eq!(lines(&pieces), ["info core/identifier_collision pieces/b"]);
    assert!(pieces.report().any(
        |found| matches!(found.code.id(), "core/identifier_collision")
            && found.severity == Severity::Info
    ));
    let directories: Vec<&str> = pieces
        .pieces
        .iter()
        .map(|piece| piece.directory.as_str())
        .collect();
    assert_eq!(directories, ["pieces/a", "pieces/b"]);
    let kept: Vec<&str> = pieces
        .kept()
        .map(|piece| piece.directory.as_str())
        .collect();
    assert_eq!(kept, ["pieces/a"]);
    assert_eq!(
        pieces
            .get(&PIECE_PACK)
            .map(|piece| piece.directory.as_str()),
        Some("pieces/a")
    );
}

#[test]
fn p4_three_piece_folders_are_one_finding_and_one_is_none() {
    let three = piece_pack(&[
        ("pieces/c/c.meta.json", &meta("")),
        ("pieces/b/b.meta.json", &meta("")),
        ("pieces/a/a.meta.json", &meta("")),
        ("texts/en_US.lang", NAMES),
    ]);
    let pieces = load(&three);
    assert_eq!(lines(&pieces), ["info core/identifier_collision pieces/b"]);
    assert_eq!(pieces.pieces.len(), 3);
    assert_eq!(pieces.kept().count(), 1);

    let one = piece_pack(&[
        ("pieces/a/a.meta.json", &meta("")),
        ("texts/en_US.lang", NAMES),
    ]);
    assert_eq!(lines(&load(&one)), Vec::<String>::new());
}

#[test]
fn p4_counts_only_the_folders_that_made_a_piece() {
    // P4 is about pieces that share the pack's UUID. A folder whose meta file is refused
    // made no piece: with one piece left nothing is shared, whichever folder comes first.
    for (first, second) in [(meta(""), "[]".to_owned()), ("[]".to_owned(), meta(""))] {
        let pack = piece_pack(&[
            ("pieces/a/a.meta.json", &first),
            ("pieces/b/b.meta.json", &second),
            ("texts/en_US.lang", NAMES),
        ]);
        let pieces = load(&pack);
        let refused = if first == "[]" { "a" } else { "b" };
        assert_eq!(
            lines(&pieces),
            [format!(
                "error persona/meta_not_parsed pieces/{refused}/{refused}.meta.json"
            )]
        );
        assert_eq!(pieces.pieces.len(), 1);
    }
    // With two pieces behind a refused folder the finding is at the second folder that made
    // a piece, never at the folder of the piece the client keeps.
    let pack = piece_pack(&[
        ("pieces/a/a.meta.json", "[]"),
        ("pieces/b/b.meta.json", &meta("")),
        ("pieces/c/c.meta.json", &meta("")),
        ("texts/en_US.lang", NAMES),
    ]);
    let pieces = load(&pack);
    assert_eq!(
        lines(&pieces),
        [
            "info core/identifier_collision pieces/c",
            "error persona/meta_not_parsed pieces/a/a.meta.json"
        ]
    );
    let kept: Vec<&str> = pieces
        .kept()
        .map(|piece| piece.directory.as_str())
        .collect();
    assert_eq!(kept, ["pieces/b"]);
    // A refused folder between two pieces changes nothing.
    let pack = piece_pack(&[
        ("pieces/a/a.meta.json", &meta("")),
        ("pieces/b/b.meta.json", "[]"),
        ("pieces/c/c.meta.json", &meta("")),
        ("texts/en_US.lang", NAMES),
    ]);
    assert_eq!(
        lines(&load(&pack)),
        [
            "info core/identifier_collision pieces/c",
            "error persona/meta_not_parsed pieces/b/b.meta.json"
        ]
    );
}

const FIRST: &str = "4c8ae710-df2e-47cd-814d-cc7bf21a3d67";
const SECOND: &str = "5c8ae710-df2e-47cd-814d-cc7bf21a3d67";

#[test]
fn pp04_the_built_in_pack_identifies_pieces_by_their_piece_id() {
    let pack = builtin_pack(&[
        (
            "pieces/a/a.meta.json",
            &meta(&format!(r#""piece_id": "{FIRST}""#)),
        ),
        (
            "pieces/b/b.meta.json",
            &meta(&format!(r#""piece_id": "{SECOND}""#)),
        ),
        ("texts/en_US.lang", NAMES),
    ]);
    assert_eq!(pack.pack_type(), PackType::Resource);
    let pieces = load(&pack);
    assert_eq!(pieces.kind, PackKind::BuiltIn);
    assert_eq!(lines(&pieces), Vec::<String>::new());
    let ids: Vec<Uuid> = pieces.pieces.iter().map(|piece| piece.id).collect();
    assert_eq!(
        ids,
        [
            Uuid::parse_str(FIRST).unwrap(),
            Uuid::parse_str(SECOND).unwrap()
        ]
    );
    assert_eq!(pieces.kept().count(), 2);
    assert_eq!(
        pieces
            .get(&Uuid::parse_str(SECOND).unwrap())
            .map(|piece| piece.directory.as_str()),
        Some("pieces/b")
    );
}

#[test]
fn the_testing_pack_identifies_pieces_by_their_piece_id_too() {
    let pack = open(
        &manifest(TESTING_PACK, "resources", Some("[1, 14, 0]")),
        &[
            (
                "pieces/a/a.meta.json",
                &meta(&format!(r#""piece_id": "{FIRST}""#)),
            ),
            ("texts/en_US.lang", NAMES),
        ],
    );
    let pieces = load(&pack);
    assert_eq!(pieces.kind, PackKind::BuiltIn);
    assert_eq!(only(&pieces).id, Uuid::parse_str(FIRST).unwrap());
}

#[test]
fn pp05_m4_a_built_in_piece_without_a_uuid_is_skipped() {
    let pack = builtin_pack(&[
        (
            "pieces/a/a.meta.json",
            &meta(&format!(r#""piece_id": "{FIRST}""#)),
        ),
        ("pieces/b/b.meta.json", &meta(r#""piece_id": "not-a-uuid""#)),
        // No `piece_id`: refused by the 1.26.40+ reader before M4 looks (D4).
        (
            "pieces/c/c.meta.json",
            r#"{ "piece_name": "hat", "piece_type": "persona_hair" }"#,
        ),
        (
            "pieces/d/d.meta.json",
            &meta(r#""piece_id": "00000000-0000-0000-0000-000000000000""#),
        ),
        ("texts/en_US.lang", NAMES),
    ]);
    let pieces = load(&pack);
    assert_eq!(
        lines(&pieces),
        [
            "error core/invalid_value pieces/b/b.meta.json piece_id",
            "error core/missing_required_field pieces/c/c.meta.json piece_id",
            "error persona/meta_not_parsed pieces/c/c.meta.json",
            "error core/invalid_value pieces/d/d.meta.json piece_id",
        ]
    );
    assert_eq!(only(&pieces).directory.as_str(), "pieces/a");
    let rejected: Vec<&str> = pieces
        .rejected
        .iter()
        .map(|rejected| rejected.origin.path.as_str())
        .collect();
    assert_eq!(
        rejected,
        [
            "pieces/b/b.meta.json",
            "pieces/c/c.meta.json",
            "pieces/d/d.meta.json"
        ]
    );
    assert!(pieces.rejected.iter().all(|rejected| {
        rejected.diagnostics.iter().any(|found| {
            matches!(
                found.code.id(),
                "core/invalid_value" | "core/missing_required_field"
            ) && found.severity == Severity::Error
        })
    }));
}

#[test]
fn m4_a_skipped_piece_keeps_the_findings_of_its_file() {
    let pack = builtin_pack(&[
        (
            "pieces/a/a.meta.json",
            &meta(r#""piece_id": "not-a-uuid", "tint_color": "red""#),
        ),
        ("texts/en_US.lang", NAMES),
    ]);
    let pieces = load(&pack);
    assert_eq!(
        lines(&pieces),
        [
            "info core/invalid_value pieces/a/a.meta.json tint_color",
            "error core/invalid_value pieces/a/a.meta.json piece_id",
        ]
    );
    assert_eq!(pieces.rejected.len(), 1);
    assert_eq!(pieces.rejected[0].diagnostics.len(), 2);
}

#[test]
fn every_finding_of_a_piece_names_the_pack() {
    use bedrock_addon::document::Content;
    // A piece's findings, a skipped file's, and those of the files beside a meta file.
    let pack = piece_pack(&[
        (
            "pieces/a/a.meta.json",
            &meta(
                r#""tint_color": "red", "texture_sources": [{ "texture": "nope" }], "geometry_sources": [{ "geometry": "g" }]"#,
            ),
        ),
        ("pieces/a/a.geometry.json", "{"),
        (
            "pieces/a/a.store_info.json",
            r#"{ "piece_store_name": "hat", "piece_thumbnail": "t", "bar": 1 }"#,
        ),
        ("pieces/b/b.meta.json", "[]"),
    ]);
    let pieces = load(&pack);
    let of_pieces: Vec<_> = pieces
        .pieces
        .iter()
        .flat_map(PersonaPiece::report)
        .chain(pieces.rejected.iter().flat_map(|r| r.diagnostics.iter()))
        .collect();
    assert_eq!(of_pieces.len(), 7, "{:#?}", lines(&pieces));
    for found in of_pieces {
        let named = found.location.pack.as_ref().unwrap_or_else(|| {
            panic!("no pack on {}", crate::common::line(found));
        });
        assert_eq!(named.id().uuid, PIECE_PACK);
        assert_eq!(named.name(), "the pack");
    }
    // A file read on its own is of no pack.
    let alone = bedrock_addon::families::persona::PieceMetadata::parse(
        &bedrock_addon::storage::PackPath::from_static("pieces/a/a.meta.json"),
        meta(r#""tint_color": "red""#).as_bytes(),
        &bedrock_addon::load::LoadContext::detached(),
    )
    .unwrap();
    assert_eq!(alone.diagnostics().len(), 1);
    assert!(
        alone
            .diagnostics()
            .iter()
            .all(|f| f.location.pack.is_none())
    );
}

#[test]
fn every_finding_of_the_pack_names_the_pack() {
    use bedrock_addon::families::persona::{BUILTIN_PACK, PersonaPieces};

    // Every finding of `pieces.diagnostics` names the pack `uuid`; their lines.
    let of_pack = |pieces: &PersonaPieces, uuid: Uuid| -> Vec<String> {
        for found in &pieces.diagnostics {
            let named = found.location.pack.as_ref().unwrap_or_else(|| {
                panic!("no pack on {}", crate::common::line(found));
            });
            assert_eq!(named.id().uuid, uuid);
            assert_eq!(named.name(), "the pack");
        }
        pieces.diagnostics.iter().map(crate::common::line).collect()
    };
    // P2
    let ordinary = load(&ordinary_pack(&[("pieces/x/x.meta.json", "{}")]));
    assert_eq!(
        of_pack(&ordinary, ORDINARY_PACK),
        ["info core/ignored_file pieces/x/x.meta.json"]
    );
    // P4 (at the first folder the client does not keep), P6, P7, S4
    let piece = load(&piece_pack(&[
        ("pieces/a/a.meta.json", &meta("")),
        ("pieces/b/a.meta.json", &meta("")),
        ("pieces/b/b.meta.json", &meta("")),
        ("pieces/b/texts/de_DE.lang", "a=b\n"),
        ("pieces/c/c.meta.json", &meta("")),
        ("pieces/d/d.geometry.json", "{}"),
        ("texts/en_US.lang", NAMES),
    ]));
    assert_eq!(
        of_pack(&piece, PIECE_PACK),
        [
            "info core/shadowed_definition pieces/b",
            "info core/missing_required_field pieces/d",
            "info core/identifier_collision pieces/b",
            "info core/ignored_file pieces/b/texts",
        ]
    );
    // M7
    let same = meta(&format!(r#""piece_id": "{FIRST}""#));
    let builtin = load(&builtin_pack(&[
        ("pieces/a/a.meta.json", &same),
        ("pieces/b/b.meta.json", &same),
        ("texts/en_US.lang", NAMES),
    ]));
    assert_eq!(
        of_pack(&builtin, BUILTIN_PACK),
        ["info core/identifier_collision pieces/b/b.meta.json piece_id"]
    );
}

#[test]
fn m7_two_built_in_pieces_with_one_id_keep_the_first() {
    let same = meta(&format!(r#""piece_id": "{FIRST}""#));
    let pack = builtin_pack(&[
        ("pieces/a/a.meta.json", &same),
        ("pieces/b/b.meta.json", &same),
        ("pieces/c/c.meta.json", &same),
        (
            "pieces/d/d.meta.json",
            &meta(&format!(r#""piece_id": "{SECOND}""#)),
        ),
        ("texts/en_US.lang", NAMES),
    ]);
    let pieces = load(&pack);
    assert_eq!(
        lines(&pieces),
        [
            "info core/identifier_collision pieces/b/b.meta.json piece_id",
            "info core/identifier_collision pieces/c/c.meta.json piece_id",
        ]
    );
    assert_eq!(pieces.pieces.len(), 4);
    let kept: Vec<&str> = pieces
        .kept()
        .map(|piece| piece.directory.as_str())
        .collect();
    assert_eq!(kept, ["pieces/a", "pieces/d"]);
    assert_eq!(
        pieces
            .get(&Uuid::parse_str(FIRST).unwrap())
            .map(|piece| piece.directory.as_str()),
        Some("pieces/a")
    );
}

#[test]
fn a_piece_the_client_skips_holds_no_identity() {
    // The client stores a piece under its id only once it is made: a folder it skips (M2) does
    // not keep the next folder with that id out, in the two- and the three-folder form.
    for others in [&["pieces/b"][..], &["pieces/b", "pieces/c"][..]] {
        let good = meta("");
        let mut files = vec![
            ("pieces/a/a.meta.json", "[]"),
            ("pieces/b/b.meta.json", good.as_str()),
            ("texts/en_US.lang", NAMES),
        ];
        if others.len() == 2 {
            files.push(("pieces/c/c.meta.json", good.as_str()));
        }
        let pieces = load(&piece_pack(&files));
        // P4 is said only of folders that made a piece, at the second of them: never at
        // the folder of the piece that is kept.
        let mut expected = vec!["error persona/meta_not_parsed pieces/a/a.meta.json"];
        if others.len() == 2 {
            expected.insert(0, "info core/identifier_collision pieces/c");
        }
        assert_eq!(lines(&pieces), expected);
        let kept: Vec<&str> = pieces
            .kept()
            .map(|piece| piece.directory.as_str())
            .collect();
        assert_eq!(kept, ["pieces/b"]);
        assert_eq!(
            pieces
                .get(&PIECE_PACK)
                .map(|piece| piece.directory.as_str()),
            Some("pieces/b")
        );
        assert_eq!(pieces.pieces.len(), others.len());
        assert_eq!(pieces.rejected.len(), 1);
    }
    // The same in a built-in pack, where the skipped piece (M4) is between two of one id.
    let same = meta(&format!(r#""piece_id": "{FIRST}""#));
    let pack = builtin_pack(&[
        ("pieces/a/a.meta.json", &meta(r#""piece_id": "not-a-uuid""#)),
        ("pieces/b/b.meta.json", &same),
        ("pieces/c/c.meta.json", &same),
        ("texts/en_US.lang", NAMES),
    ]);
    let pieces = load(&pack);
    assert_eq!(
        lines(&pieces),
        [
            "info core/identifier_collision pieces/c/c.meta.json piece_id",
            "error core/invalid_value pieces/a/a.meta.json piece_id",
        ]
    );
    let kept: Vec<&str> = pieces
        .kept()
        .map(|piece| piece.directory.as_str())
        .collect();
    assert_eq!(kept, ["pieces/b"]);
}

#[test]
fn m7_the_id_is_compared_as_a_uuid() {
    // The same UUID in another letter case is the same piece.
    let pack = builtin_pack(&[
        (
            "pieces/a/a.meta.json",
            &meta(&format!(r#""piece_id": "{FIRST}""#)),
        ),
        (
            "pieces/b/b.meta.json",
            &meta(&format!(r#""piece_id": "{}""#, FIRST.to_uppercase())),
        ),
        ("texts/en_US.lang", NAMES),
    ]);
    assert_eq!(
        lines(&load(&pack)),
        ["info core/identifier_collision pieces/b/b.meta.json piece_id"]
    );
}

#[test]
fn pp06_p2_piece_files_in_an_ordinary_pack_are_not_read() {
    let pack = ordinary_pack(&[
        ("pieces/x/x.meta.json", "this is never parsed"),
        ("pieces/x/x.store_info.json", "{}"),
        ("pieces/x/x.geometry.json", "{}"),
        ("pieces/x/x.png", "png"),
        ("pieces/x/texts/en_US.lang", "a=b\n"),
        ("models/entity/x.geometry.json", "{}"),
        ("models/x.meta.json", "{}"),
    ]);
    let pieces = load(&pack);
    assert_eq!(pieces.kind, PackKind::Ordinary);
    assert!(pieces.pieces.is_empty());
    assert!(pieces.rejected.is_empty());
    assert_eq!(
        lines(&pieces),
        [
            "info core/ignored_file pieces/x/x.geometry.json",
            "info core/ignored_file pieces/x/x.meta.json",
            "info core/ignored_file pieces/x/x.store_info.json",
        ]
    );
    // The pack itself is an ordinary resource pack and is served.
    assert_eq!(pack.pack_type(), PackType::Resource);
    assert!(!pack.report().has_errors());
}

#[test]
fn p2_a_behavior_pack_s_piece_files_are_not_read_either() {
    let pack = open(
        &manifest(ORDINARY_PACK, "data", Some("[1, 14, 0]")),
        &[("pieces/x/x.meta.json", "{}")],
    );
    let pieces = load(&pack);
    assert_eq!(pieces.kind, PackKind::Ordinary);
    assert_eq!(
        lines(&pieces),
        ["info core/ignored_file pieces/x/x.meta.json"]
    );
}

#[test]
fn pp07_p1_a_piece_pack_is_never_on_a_resource_stack() {
    let pack = piece_pack(&[("pieces/hat/hat.meta.json", &meta(""))]);
    let world = WorldPacks::parse(
        None,
        Some(format!(r#"[{{"pack_id": "{PIECE_PACK}", "version": [1, 0, 0]}}]"#).as_bytes()),
    )
    .expect("a world list");
    let mut repository = PackRepository::new();
    repository.add_pack(pack, PackOrigin::User);
    let stacks = repository.stacks(&world, LoadOptions::default());
    assert!(
        stacks
            .resource
            .packs()
            .all(|on_stack| on_stack.id().uuid != PIECE_PACK),
        "the piece pack is served as a resource pack"
    );
    // P1: the listed pack is skipped with its own warning, at the world's entry.
    let found: Vec<_> = stacks
        .selection()
        .iter()
        .filter(|d| matches!(d.code.id(), "core/persona_piece_pack"))
        .collect();
    assert_eq!(found.len(), 1, "{found:?}");
    // Adding to the stack has no kind check and no line: an information.
    assert_eq!(found[0].severity, Severity::Info);
    assert_eq!(
        found[0]
            .location
            .path
            .as_ref()
            .map(bedrock_addon::storage::PackPath::as_str),
        Some("world_resource_packs.json")
    );
    assert_eq!(found[0].location.json_path.to_string(), "0/pack_id");
}

#[test]
fn pp08_only_the_suffix_makes_a_meta_file() {
    let pack = piece_pack(&[
        ("pieces/steve_mouth/steve_mouth.meta.meta.json", &meta("")),
        ("pieces/steve_mouth/other.META.JSON", "[]"),
        ("pieces/steve_mouth/meta.json", "[]"),
        ("texts/en_US.lang", NAMES),
    ]);
    let pieces = load(&pack);
    assert_eq!(lines(&pieces), Vec::<String>::new());
    assert_eq!(
        only(&pieces).meta.origin().path.as_str(),
        "pieces/steve_mouth/steve_mouth.meta.meta.json"
    );
}

#[test]
fn the_other_suffixes_and_the_texts_folder_are_matched_letter_for_letter() {
    // discovery is by suffix only, case-sensitive, for each of
    // `.geometry.json`, `.store_info.json` and `texts/en_US.lang`.
    let pack = piece_pack(&[
        (
            "pieces/hat/hat.meta.json",
            &meta(r#""geometry_sources": [{ "geometry": "g" }]"#),
        ),
        ("pieces/hat/hat.GEOMETRY.json", r#"{ "g": {} }"#),
        ("pieces/hat/hat.geometry.JSON", r#"{ "g": {} }"#),
        ("pieces/hat/hat.Geometry.json", r#"{ "g": {} }"#),
        ("pieces/hat/hat.STORE_INFO.json", "[]"),
        ("pieces/hat/hat.store_info.JSON", "[]"),
        ("pieces/hat/hat.Store_Info.json", "[]"),
        ("pieces/hat/Texts/en_US.lang", "persona.offer.title=A\n"),
        ("pieces/hat/TEXTS/en_US.lang", "persona.offer.title=B\n"),
        ("pieces/hat/texts/EN_US.LANG", "persona.offer.title=C\n"),
        // None of these makes a folder a piece's (no P6).
        ("pieces/x/x.GEOMETRY.json", "{}"),
        ("pieces/y/y.STORE_INFO.json", "{}"),
        ("pieces/z/Texts/en_US.lang", "a=b\n"),
        ("texts/en_US.lang", NAMES),
    ]);
    let pieces = load(&pack);
    assert_eq!(
        lines(&pieces),
        // G3: no geometry file to look the model up in.
        [
            "error persona/geometry_name_not_found pieces/hat/hat.meta.json geometry_sources/0/geometry"
        ]
    );
    let piece = only(&pieces);
    assert!(piece.geometry.is_none());
    assert!(piece.store_info.is_none());
    assert_eq!(piece.texts, None);
    assert_eq!(piece.offer_title, None);
    assert_eq!(piece.name_key(), "persona.hat.title");

    // `en_us.lang` in a folder `texts` is a text file of the folder, and not the one the
    // client keys on (S4); the same file as `en_US.lang` is.
    let lower = load(&piece_pack(&[
        ("pieces/hat/hat.meta.json", &meta("")),
        ("pieces/hat/texts/en_us.lang", "persona.offer.title=A\n"),
        ("texts/en_US.lang", NAMES),
    ]));
    assert_eq!(lines(&lower), ["info core/ignored_file pieces/hat/texts"]);
    assert_eq!(only(&lower).offer_title, None);
    let exact = load(&piece_pack(&[
        ("pieces/hat/hat.meta.json", &meta("")),
        ("pieces/hat/hat.geometry.json", "{}"),
        ("pieces/hat/hat.store_info.json", "[]"),
        ("pieces/hat/texts/en_US.lang", "persona.offer.title=A\n"),
    ]));
    assert_eq!(
        lines(&exact),
        ["info core/syntax pieces/hat/hat.store_info.json"]
    );
    assert!(only(&exact).geometry.is_some());
    assert_eq!(only(&exact).offer_title.as_deref(), Some("A"));
    // The pack's own names: a root folder `Texts` holds none of them.
    let names = load(&piece_pack(&[
        ("pieces/hat/hat.meta.json", &meta("")),
        ("Texts/en_US.lang", NAMES),
        ("texts/en_US.LANG", NAMES),
    ]));
    assert_eq!(
        lines(&names),
        ["info core/unresolved_lang_key pieces/hat/hat.meta.json"]
    );
}

#[test]
fn a_piece_folder_may_be_anywhere_and_at_any_depth() {
    let pack = builtin_pack(&[
        (
            "pieces/body/medium/medium.meta.json",
            &meta(&format!(r#""piece_id": "{FIRST}""#)),
        ),
        (
            "elsewhere/x.meta.json",
            &meta(&format!(r#""piece_id": "{SECOND}""#)),
        ),
        ("texts/en_US.lang", NAMES),
    ]);
    let pieces = load(&pack);
    let directories: Vec<&str> = pieces
        .pieces
        .iter()
        .map(|piece| piece.directory.as_str())
        .collect();
    assert_eq!(directories, ["elsewhere", "pieces/body/medium"]);
}

#[test]
fn pp09_p6_a_folder_without_a_meta_file_is_no_piece_deviation() {
    let pack = piece_pack(&[
        ("pieces/x/x.geometry.json", "{}"),
        ("pieces/x/x.store_info.json", "{}"),
        ("pieces/y/y.store_info.json", "{}"),
        ("pieces/z/texts/en_US.lang", "a=b\n"),
        ("pieces/w/w.geometry.json", "{}"),
        // Neither a picture nor a translation of another language makes a folder a piece's.
        ("pieces/v/v.png", "png"),
        ("pieces/u/texts/de_DE.lang", "a=b\n"),
        // The pack's own names are no piece's.
        ("texts/en_US.lang", NAMES),
    ]);
    let pieces = load(&pack);
    assert!(pieces.pieces.is_empty());
    assert_eq!(
        lines(&pieces),
        [
            "info core/missing_required_field pieces/w",
            "info core/missing_required_field pieces/x",
            "info core/missing_required_field pieces/y",
            "info core/missing_required_field pieces/z",
        ]
    );
}

#[test]
fn p6_a_folder_without_a_meta_file_has_no_texts_to_report() {
    // S4 is a piece's: a folder the client makes no piece of is reported once, by P6.
    let pack = piece_pack(&[
        ("pieces/x/x.geometry.json", "{}"),
        ("pieces/x/texts/de_DE.lang", "a=b\n"),
        ("texts/en_US.lang", NAMES),
    ]);
    let pieces = load(&pack);
    assert_eq!(
        lines(&pieces),
        ["info core/missing_required_field pieces/x"]
    );
}

#[test]
fn pp10_p7_of_two_meta_files_in_a_folder_one_is_read_deviation() {
    let pack = piece_pack(&[
        (
            "pieces/x/a.meta.json",
            r#"{ "piece_id": "aa8d3c95-f189-cf08-0ae3-5cc1e090343b", "piece_name": "hat", "piece_type": "persona_top" }"#,
        ),
        ("pieces/x/b.meta.json", &meta("")),
        ("texts/en_US.lang", NAMES),
    ]);
    let pieces = load(&pack);
    assert_eq!(lines(&pieces), ["info core/shadowed_definition pieces/x"]);
    let piece = only(&pieces);
    // The later one by name stands, as a later file replaces the slot in the client's walk.
    assert_eq!(piece.meta.origin().path.as_str(), "pieces/x/b.meta.json");
    assert_eq!(piece.piece_type(), PieceType::Hair);
}

#[test]
fn p7_three_meta_files_are_one_finding() {
    let pack = piece_pack(&[
        ("pieces/x/a.meta.json", "[]"),
        ("pieces/x/b.meta.json", "[]"),
        ("pieces/x/c.meta.json", &meta("")),
        ("texts/en_US.lang", NAMES),
    ]);
    let pieces = load(&pack);
    // The shadowed files are never read: their broken JSON is not reported.
    assert_eq!(lines(&pieces), ["info core/shadowed_definition pieces/x"]);
    assert_eq!(
        only(&pieces).meta.origin().path.as_str(),
        "pieces/x/c.meta.json"
    );
}

#[test]
fn pp11_p3_a_piece_pack_s_min_engine_version_is_ignored() {
    let with = open(
        &manifest(PIECE_PACK, "persona_piece", Some("[1, 21, 0]")),
        &[("pieces/hat/hat.meta.json", &meta(""))],
    );
    let ignored = |pack: &bedrock_addon::pack::Pack| {
        pack.report()
            .diagnostics()
            .iter()
            .filter(|found| {
                matches!(found.code.id(), "core/ignored_property")
                    && found.severity == Severity::Warning
            })
            .count()
    };
    assert_eq!(ignored(&with), 1, "{:?}", with.report().diagnostics());
    let without = piece_pack(&[("pieces/hat/hat.meta.json", &meta(""))]);
    assert_eq!(ignored(&without), 0, "{:?}", without.report().diagnostics());
}

#[test]
fn the_load_is_the_same_whatever_order_the_files_come_in() {
    let files = [
        (
            "pieces/a/a.meta.json",
            meta(&format!(r#""piece_id": "{FIRST}""#)),
        ),
        (
            "pieces/b/b.meta.json",
            meta(&format!(r#""piece_id": "{FIRST}""#)),
        ),
        ("pieces/b/c.meta.json", meta("")),
        ("pieces/c/c.geometry.json", "{}".to_owned()),
    ];
    let forward: Vec<(&str, &str)> = files
        .iter()
        .map(|(path, text)| (*path, text.as_str()))
        .collect();
    let pack = builtin_pack(&forward);
    let one = load(&pack);
    let other = PersonaPiece::load_all(&Reversed(&pack));
    assert_eq!(format!("{one:?}"), format!("{other:?}"));
    assert!(!lines(&one).is_empty());
}

#[test]
fn m6_p5_the_allow_list_and_the_signature_are_not_checked_deviation() {
    // the allow list is a list of vanilla's own UUIDs, signatures are the
    // Marketplace's. A built-in piece with an id of its own loads without a finding.
    let pack = builtin_pack(&[
        (
            "pieces/a/a.meta.json",
            &meta(r#""piece_id": "11111111-2222-4333-8444-555555555555""#),
        ),
        ("texts/en_US.lang", NAMES),
    ]);
    let pieces = load(&pack);
    assert_eq!(lines(&pieces), Vec::<String>::new());
    assert_eq!(pieces.pieces.len(), 1);
}

// A `Pack` is a `PieceSource` itself (`PersonaPiece::load_all(&pack)`): the same pieces as through a wrapper, and a file that is indexed and not
// read is told apart from one the pack does not have.
#[test]
fn a_pack_is_a_piece_source() {
    use bedrock_addon::families::persona::PieceSource;
    use bedrock_addon::storage::PackPath;

    let pack = piece_pack(&[
        ("pieces/hat/hat.meta.json", &meta("")),
        ("texts/en_US.lang", NAMES),
    ]);
    let direct = PersonaPiece::load_all(&pack);
    let wrapped = load(&pack);
    assert_eq!(direct.kind, PackKind::PersonaPiece);
    assert_eq!(direct.pieces.len(), 1);
    assert_eq!(
        direct.pieces[0].meta.origin().path,
        wrapped.pieces[0].meta.origin().path
    );
    assert_eq!(direct.rejected.len(), wrapped.rejected.len());
    assert_eq!(PieceSource::id(&pack), pack.id());
    assert_eq!(PieceSource::paths(&pack).count(), pack.paths().count());
    assert_eq!(
        PieceSource::read(&pack, &PackPath::from_static("pieces/none.json")),
        Err(std::io::ErrorKind::NotFound)
    );
    assert!(PieceSource::read(&pack, &PackPath::from_static("texts/en_US.lang")).is_ok());
    assert!(!PieceSource::name(&pack).is_empty());
}
