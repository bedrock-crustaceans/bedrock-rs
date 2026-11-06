//! `files.rs`), the
//! member rules of geometry (pg05 to pg07) and rules M1 to M3, M8 to M12, M14 to
//! M19, G4 to G6 and A6.

use bedrock_addon::diagnostic::{Diagnostic, DiagnosticCode, Severity};
use bedrock_addon::document::Content;
use bedrock_addon::families::persona::{
    ArmSize, BodySize, FaceFlag, GeometrySource, HexColor, Named, PersonaPiece, PersonaPieces,
    PieceMetadata, PieceSide, PieceSource, PieceType, TintColor, TintColorOrString, Written, Zones,
};
use bedrock_addon::load::LoadContext;
use bedrock_addon::pack::PackIdVersion;
use bedrock_addon::storage::PackPath;
use bytes::Bytes;

use crate::common::{OfPack, hat, line, lines, message, only, piece_pack, refused_at};
use crate::packs::meta;

const FILE: &str = "pieces/hat/hat.meta.json";

/// The pack-wide name of the piece every test here builds.
const NAMES: &str = "persona.hat.title=Hat\n";

/// A hair piece named `hat` with `members`, loaded from a piece pack.
fn hat_with(members: &str) -> PersonaPieces {
    load_hat(&meta(members))
}

/// The meta file `text` as the piece `pieces/hat`, loaded from a piece pack.
fn load_hat(text: &str) -> PersonaPieces {
    let pack = piece_pack(&[(FILE, text), ("texts/en_US.lang", NAMES)]);
    PersonaPiece::load_all(&OfPack(&pack))
}

/// The findings of `pieces`, each line with the meta file's name cut off.
fn found(pieces: &PersonaPieces) -> Vec<String> {
    lines(pieces)
        .into_iter()
        .map(|line| line.replace(&format!(" {FILE}"), ""))
        .collect()
}

/// The first finding with `code` among those of `pieces`.
fn first(pieces: &PersonaPieces, wanted: &str) -> Diagnostic {
    pieces
        .report()
        .find(|found| line(found).contains(wanted))
        .unwrap_or_else(|| panic!("no {wanted} in {:?}", lines(pieces)))
        .clone()
}

/// The error findings of the rejected meta file of `pieces` ([`refused_at`]).
fn refused(pieces: &PersonaPieces) -> Vec<String> {
    refused_at(pieces, FILE)
}

fn value(pieces: &PersonaPieces) -> &PieceMetadata {
    only(pieces).meta.value()
}

#[test]
fn pm01_m3_an_unknown_piece_type_invalidates_the_piece() {
    // 1.26.20 read it as `Unknown` (0) with "Found an unknown asset type"; the 1.26.40+ enum
    // refuses it (E1), and so a missing, empty or `null` type: `strict.rs` has each case.
    let pieces = load_hat(
        r#"{ "piece_id": "aa8d3c95-f189-cf08-0ae3-5cc1e090343b", "piece_name": "hat", "piece_type": "persona_hat" }"#,
    );
    assert!(pieces.pieces.is_empty());
    let finding = first(&pieces, "InvalidEnumValue");
    assert_eq!(finding.severity, Severity::Error);
    assert_eq!(message(&finding), "invalid enum value");
    // A model built by hand with such a text still answers `Unknown`.
    let mut meta = PieceMetadata::default();
    meta.piece_type = Some(Written::Other("persona_hat".to_owned()));
    assert_eq!(meta.piece_type(), PieceType::Unknown);
    assert_eq!(meta.piece_type().value(), 0);
    assert_eq!(PieceMetadata::default().piece_type(), PieceType::Unknown);
}

/// The 25 names the 1.26.60 enum binds:
/// `persona_coco` (28) and
/// `unsupported` (29).
const PIECE_TYPES: [(&str, u8); 25] = [
    ("persona_skeleton", 1),
    ("persona_body", 2),
    ("persona_skin", 3),
    ("persona_bottom", 4),
    ("persona_feet", 5),
    ("persona_dress", 6),
    ("persona_top", 7),
    ("persona_high_pants", 8),
    ("persona_hand", 9),
    ("persona_outerwear", 10),
    ("persona_facial_hair", 11),
    ("persona_mouth", 12),
    ("persona_eyes", 13),
    ("persona_hair", 14),
    ("persona_hood", 15),
    ("persona_back", 16),
    ("persona_face_accessory", 17),
    ("persona_head", 18),
    ("persona_legs", 19),
    ("persona_arms", 22),
    ("persona_capes", 25),
    ("persona_classic_skin", 26),
    ("persona_emote", 27),
    ("persona_coco", 28),
    ("unsupported", 29),
];

/// The four values 1.26.20 read by name and the 1.26.40+ enum keeps internal-only.
const ONE_SIDED: [(&str, u8); 4] = [
    ("persona_left_leg", 20),
    ("persona_right_leg", 21),
    ("persona_left_arm", 23),
    ("persona_right_arm", 24),
];

#[test]
fn pm02_m16_every_piece_type_of_the_enum() {
    for (name, number) in PIECE_TYPES {
        // An emote without animations has a finding of its own (A5), told apart below.
        let pieces = load_hat(&format!(
            r#"{{ "piece_id": "aa8d3c95-f189-cf08-0ae3-5cc1e090343b", "piece_name": "hat", "piece_type": "{name}" }}"#
        ));
        let meta = value(&pieces);
        assert_eq!(meta.piece_type().value(), number, "{name}");
        assert_eq!(meta.piece_type().name(), name);
        assert_eq!(PieceType::from_name(name), Some(meta.piece_type()));
        assert!(!meta.piece_type().is_one_sided(), "{name}");
        let expected: Vec<&str> = if name == "persona_emote" {
            vec!["error MissingRequiredField animation_sources"]
        } else {
            vec![]
        };
        assert_eq!(found(&pieces), expected, "{name}");
    }
    // M4: the one-sided types have no name from 1.26.40, nor has the name the game prints for
    // an unknown type; a file with one is refused (`strict.rs`).
    for (name, _) in ONE_SIDED {
        assert_eq!(PieceType::from_name(name), None, "{name}");
    }
    assert_eq!(PieceType::from_name("persona_unknown"), None);
}

#[test]
fn the_piece_types_are_the_game_s_30_values() {
    let all: Vec<(&str, u8)> = PieceType::ALL
        .iter()
        .map(|piece_type| (piece_type.name(), piece_type.value()))
        .collect();
    let mut expected = vec![("persona_unknown", 0)];
    expected.extend(PIECE_TYPES);
    expected.extend(ONE_SIDED);
    expected.sort_by_key(|(_, number)| *number);
    assert_eq!(all, expected);
    assert_eq!(PieceType::from_name("persona_unknown"), None);
    assert_eq!(PieceType::from_name("Persona_Hair"), None);
    assert_eq!(PieceType::from_name("persona_unkown"), None);
}

#[test]
fn the_traits_of_a_piece_type_are_the_game_s_masks() {
    // each trait is a bit mask over the type's number.
    const CAN_BE_NONE: u32 = 0x0a4f_fe20;
    const RECOLOURABLE: u32 = 0x7808;
    const SIDED: u32 = 0x0090_0000 | 0x0120_0000 | 0x0048_0000;
    const ONE_SIDED: u32 = 0x0090_0000 | 0x0120_0000;
    assert_eq!(PieceType::ALL.len(), 30);
    for piece_type in PieceType::ALL {
        let bit = 1u32 << piece_type.value();
        let name = piece_type.name();
        assert_eq!(piece_type.can_be_none(), CAN_BE_NONE & bit != 0, "{name}");
        assert_eq!(
            piece_type.is_recolorable(),
            piece_type.value() <= 14 && RECOLOURABLE & bit != 0,
            "{name}"
        );
        assert_eq!(piece_type.is_arm_or_leg(), SIDED & bit != 0, "{name}");
        assert_eq!(piece_type.is_one_sided(), ONE_SIDED & bit != 0, "{name}");
    }
    // The fifteen the game names.
    let none: Vec<&str> = PieceType::ALL
        .iter()
        .filter(|piece_type| piece_type.can_be_none())
        .map(|piece_type| piece_type.name())
        .collect();
    let mut named = [
        "persona_feet",
        "persona_hand",
        "persona_outerwear",
        "persona_facial_hair",
        "persona_mouth",
        "persona_eyes",
        "persona_hair",
        "persona_hood",
        "persona_back",
        "persona_face_accessory",
        "persona_head",
        "persona_legs",
        "persona_arms",
        "persona_capes",
        "persona_emote",
    ];
    let mut none_sorted = none.clone();
    none_sorted.sort_unstable();
    named.sort_unstable();
    assert_eq!(none_sorted, named);
}

#[test]
fn the_other_name_sets_are_the_game_s() {
    let body: Vec<(&str, u8)> = BodySize::ALL
        .iter()
        .map(|v| (v.name(), v.value()))
        .collect();
    // `""` is the bound constant 5 of `BodySizeType`.
    let bound = [
        ("smaller", 0),
        ("small", 1),
        ("medium", 2),
        ("tall", 3),
        ("", 5),
    ];
    assert_eq!(body, bound);
    for (name, number) in bound {
        assert_eq!(BodySize::from_name(name).map(BodySize::value), Some(number));
    }
    assert_eq!(BodySize::from_name("Medium"), None);

    let arm: Vec<(&str, u8)> = ArmSize::ALL.iter().map(|v| (v.name(), v.value())).collect();
    // `""` is the bound constant 3 of `ArmSizeType`.
    let bound = [("slim", 0), ("wide", 1), ("", 3)];
    assert_eq!(arm, bound);
    for (name, number) in bound {
        assert_eq!(ArmSize::from_name(name).map(ArmSize::value), Some(number));
    }
    assert_eq!(ArmSize::from_name("Slim"), None);

    let side: Vec<(&str, u8)> = PieceSide::ALL
        .iter()
        .map(|v| (v.name(), v.value()))
        .collect();
    assert_eq!(side, [("left", 0), ("both", 1), ("right", 2)]);
    for (name, number) in [("left", 0), ("both", 1), ("right", 2)] {
        assert_eq!(
            PieceSide::from_name(name).map(PieceSide::value),
            Some(number)
        );
    }
}

#[test]
fn pm03_m3_the_sub_type() {
    let top = hat_with(r#""piece_sub_type": "persona_top""#);
    assert_eq!(found(&top), Vec::<String>::new());
    assert_eq!(value(&top).piece_sub_type(), Some(PieceType::Top));

    let absent = hat_with("");
    assert_eq!(value(&absent).piece_sub_type(), None);
    assert_eq!(value(&absent).piece_sub_type, None);

    // 1.26.40+ binds the sub-type to the piece-type enum: the empty text,
    // an unknown name and a one-sided type are each refused (`strict.rs`).
    for refused in [r#""""#, r#""persona_hat""#, r#""persona_left_arm""#] {
        let pieces = hat_with(&format!(r#""piece_sub_type": {refused}"#));
        assert!(pieces.pieces.is_empty(), "{refused}");
    }
    // A model built by hand: the empty text is no sub-type, another text `Unknown`.
    let mut meta = PieceMetadata::default();
    meta.piece_sub_type = Some(Written::Other(String::new()));
    assert_eq!(meta.piece_sub_type(), None);
    meta.piece_sub_type = Some(Written::Other("persona_hat".to_owned()));
    assert_eq!(meta.piece_sub_type(), Some(PieceType::Unknown));
}

#[test]
fn pm04_m8_the_body_variant() {
    // 1.26.20's "huge" was medium with a debug assert; the 1.26.40+ enum refuses it.
    let huge = hat_with(r#""body_variant": "huge""#);
    assert!(huge.pieces.is_empty());
    assert_eq!(first(&huge, "InvalidEnumValue").severity, Severity::Error);

    let absent = hat_with("");
    assert_eq!(found(&absent), Vec::<String>::new());
    assert_eq!(value(&absent).body_variant, None);
    assert_eq!(value(&absent).body_variant(), BodySize::Medium);

    // The empty text is the bound constant 5, which the game takes for medium.
    let empty = hat_with(r#""body_variant": """#);
    assert_eq!(found(&empty), Vec::<String>::new());
    assert_eq!(
        value(&empty).body_variant,
        Some(Written::Known(BodySize::Unspecified))
    );
    assert_eq!(value(&empty).body_variant(), BodySize::Medium);

    for (name, size) in [
        ("smaller", BodySize::Smaller),
        ("small", BodySize::Small),
        ("medium", BodySize::Medium),
        ("tall", BodySize::Tall),
    ] {
        let pieces = hat_with(&format!(r#""body_variant": "{name}""#));
        assert_eq!(found(&pieces), Vec::<String>::new());
        assert_eq!(value(&pieces).body_variant(), size);
    }
}

#[test]
fn pm06_m10_the_forms_of_a_tint() {
    let pieces =
        hat_with(r##""tint_base_color": { "r_color": "#FF8800" }, "tint_color": "#ff0000""##);
    assert_eq!(found(&pieces), Vec::<String>::new());
    let meta = value(&pieces);
    let Some(TintColorOrString::Channels(base)) = &meta.tint_base_color else {
        panic!("{:?}", meta.tint_base_color);
    };
    assert_eq!(base.r_color, Some(HexColor("#FF8800".to_owned())));
    assert_eq!(
        (&base.g_color, &base.b_color, &base.a_color),
        (&None, &None, &None)
    );
    assert_eq!(
        meta.tint_base_color.as_ref().unwrap().slots(),
        [Some(0x00FF_8800), None, None, None]
    );
    assert_eq!(
        meta.tint_color,
        Some(TintColorOrString::String(HexColor("#ff0000".to_owned())))
    );
    assert_eq!(
        meta.tint_color.as_ref().unwrap().slots(),
        [Some(0x00FF_0000); 4]
    );

    // `variant<TintColor, string>`: any other JSON type is `0x8`, the piece is refused.
    for wrong in ["5", "true", "[]", r##"["#ff0000"]"##, "null"] {
        for member in ["tint_color", "tint_base_color"] {
            let pieces = hat_with(&format!(r#""{member}": {wrong}"#));
            assert_eq!(
                refused(&pieces),
                [format!("error InvalidType {member}")],
                "{wrong}"
            );
        }
    }
}

#[test]
fn a_tint_colour_left_out_is_the_empty_text() {
    // the default-constructed `TintColor` holds `""` in each of
    // its four colours; the game's hex-string parse of `""`
    // is no colour, so a colour written `""` and one left out are the same slot.
    let pieces = hat_with(r##""tint_base_color": { "r_color": "#FF8800" }"##);
    let Some(TintColorOrString::Channels(base)) = &value(&pieces).tint_base_color else {
        panic!("{:?}", value(&pieces).tint_base_color);
    };
    assert_eq!(base.colors(), ["#FF8800", "", "", ""]);
    assert_eq!(TintColor::default().colors(), ["", "", "", ""]);
    let empty = hat_with(
        r##""tint_base_color": { "r_color": "#FF8800", "g_color": "", "b_color": "", "a_color": "" }"##,
    );
    assert_eq!(found(&empty), Vec::<String>::new());
    let Some(TintColorOrString::Channels(written)) = &value(&empty).tint_base_color else {
        panic!();
    };
    assert_eq!(written.colors(), base.colors());
    assert_eq!(
        value(&empty).tint_base_color.as_ref().unwrap().slots(),
        value(&pieces).tint_base_color.as_ref().unwrap().slots()
    );
}

#[test]
fn m10_a_channel_that_is_no_string() {
    let pieces = hat_with(r##""tint_color": { "r_color": 5, "g_color": "#00ff00", "x": 1 }"##);
    assert_eq!(
        refused(&pieces),
        [
            "error InvalidType tint_color/r_color",
            "error UnrecognizedField tint_color/x"
        ]
    );
}

#[test]
fn pm07_m11_a_colour_that_is_no_hex_colour() {
    let pieces = hat_with(r#""tint_color": { "r_color": "red" }"#);
    assert_eq!(found(&pieces), ["info InvalidValue tint_color/r_color"]);
    let Some(TintColorOrString::Channels(tint)) = &value(&pieces).tint_color else {
        panic!();
    };
    // Kept as written.
    assert_eq!(tint.r_color, Some(HexColor("red".to_owned())));

    for bad in [
        "#",
        "#12345",
        "#1234567",
        "#123456789",
        "#12345g",
        "123456",
        "1234567",
        "#12 456",
        " #123456",
    ] {
        for (members, at) in [
            (format!(r#""tint_color": "{bad}""#), "tint_color"),
            (
                format!(r#""tint_base_color": {{ "a_color": "{bad}" }}"#),
                "tint_base_color/a_color",
            ),
            (
                format!(r#""tint_color": {{ "g_color": "{bad}" }}"#),
                "tint_color/g_color",
            ),
            (
                format!(r#""tint_color": {{ "b_color": "{bad}" }}"#),
                "tint_color/b_color",
            ),
        ] {
            assert_eq!(
                found(&hat_with(&members)),
                [format!("info InvalidValue {at}")],
                "{bad:?}"
            );
        }
    }
    for good in ["#123456", "#12345678", "#aBcDeF", "#ABCDEF01"] {
        let pieces = hat_with(&format!(
            r#""tint_color": "{good}", "tint_base_color": {{ "r_color": "{good}", "g_color": "{good}", "b_color": "{good}", "a_color": "{good}" }}"#
        ));
        assert_eq!(found(&pieces), Vec::<String>::new(), "{good}");
    }
}

#[test]
fn a_colour_is_read_as_the_game_reads_it() {
    // The first character is dropped unchecked, the rest is `0xAARRGGBB`; no text, no colour.
    assert_eq!(HexColor("#FF8800".to_owned()).argb(), Some(0x00FF_8800));
    assert_eq!(HexColor("#80FF8800".to_owned()).argb(), Some(0x80FF_8800));
    assert_eq!(HexColor("xff".to_owned()).argb(), Some(0xFF));
    assert_eq!(HexColor("#".to_owned()).argb(), Some(0));
    assert_eq!(HexColor("red".to_owned()).argb(), Some(0xED));
    assert_eq!(HexColor(String::new()).argb(), None);
    assert_eq!(
        HexColor("#FFFFFFFFFF".to_owned()).argb(),
        Some(u32::MAX),
        "past 32 bits the number stays at its top"
    );
}

#[test]
fn pm08_m12_the_forms_of_a_zone() {
    let one = hat_with(r#""zone": "body_front_upper""#);
    assert_eq!(found(&one), Vec::<String>::new());
    assert_eq!(
        value(&one).zone,
        Some(Zones::One("body_front_upper".to_owned()))
    );
    let many = hat_with(r#""zone": ["a", "b"]"#);
    assert_eq!(found(&many), Vec::<String>::new());
    assert_eq!(
        value(&many).zone,
        Some(Zones::Many(vec!["a".to_owned(), "b".to_owned()]))
    );
    assert_eq!(value(&many).zone.as_ref().unwrap().as_slice().len(), 2);

    // `variant<vector<string>, string>`: any other JSON type is `0x8`, the piece is refused.
    for wrong in ["3", "true", "{}", "null"] {
        let pieces = hat_with(&format!(r#""zone": {wrong}"#));
        assert_eq!(refused(&pieces), ["error InvalidType zone"], "{wrong}");
    }
    let mixed = hat_with(r#""zone": ["a", 3, "c"]"#);
    assert_eq!(refused(&mixed), ["error InvalidType zone/1"]);

    // The same member of a geometry source.
    let source = hat(
        &meta(r#""geometry_sources": [{ "geometry": "g", "zone": ["head", "body"] }]"#),
        &[("hat.geometry.json", r#"{ "g": {} }"#)],
    );
    let sources = value(&source).geometry_sources.as_ref().unwrap();
    assert_eq!(
        sources[0].zone,
        Some(Zones::Many(vec!["head".to_owned(), "body".to_owned()]))
    );
    let source = hat_with(r#""geometry_sources": [{ "geometry": "g", "zone": 3 }]"#);
    assert_eq!(
        refused(&source),
        ["error InvalidType geometry_sources/0/zone"]
    );
}

#[test]
fn pm09_m14_an_unknown_member_invalidates_the_piece() {
    // 1.26.20 ignored it; under 0xfffc it is `0x4`, an error (M2).
    let pieces = hat_with(r#""foo": 1, "texture_sources": [{ "texture": "", "bar": [2] }]"#);
    assert_eq!(
        refused(&pieces),
        [
            "error UnrecognizedField foo",
            "error UnrecognizedField texture_sources/0/bar",
        ]
    );
    let finding = first(&pieces, "UnrecognizedField");
    // By legacy name: the variant and the definitions that were it.
    assert!(matches!(finding.code.name(), "UnrecognizedField"));
}

#[test]
fn a_member_named_extra_is_an_unknown_member() {
    let pieces = hat_with(r#""extra": { "piece_name": "x" }"#);
    assert_eq!(refused(&pieces), ["error UnrecognizedField extra"]);
}

#[test]
fn pm10_m15_a_format_version_beside_flat_members_invalidates_the_piece() {
    // M15: from 1.26.40 the version is read; a flat file with one fails the two-member
    // envelope check (A9) at 1.26.40 and above, and has no slice (A2) below it.
    let pieces = hat_with(r#""format_version": "1.26.40""#);
    assert_eq!(refused(&pieces), ["error MalformedInput"]);
    assert_eq!(
        message(&first(&pieces, "MalformedInput")),
        "A Minecraft Document must have exactly two members: \"format_version\" and a payload member"
    );
    let pieces = hat_with(r#""format_version": "1.20.0""#);
    assert_eq!(
        refused(&pieces),
        ["error UnsupportedVersion format_version"]
    );
}

#[test]
fn pm11_a_document_with_the_envelope_is_read_from_its_payload() {
    // At exactly 1.26.40 the envelope is the loader's document form.
    let pieces = load_hat(
        r#"{ "format_version": "1.26.40", "minecraft:persona_piece_meta": { "piece_id": "aa8d3c95-f189-cf08-0ae3-5cc1e090343b", "piece_name": "hat", "piece_type": "persona_top" } }"#,
    );
    assert_eq!(found(&pieces), Vec::<String>::new());
    let meta = value(&pieces);
    assert_eq!(meta.piece_type(), PieceType::Top);
    assert_eq!(
        meta.envelope
            .as_ref()
            .and_then(|v| v.as_string())
            .map(facet_value::VString::as_str),
        Some("1.26.40")
    );
    assert_eq!(only(&pieces).name_key(), "persona.hat.title");
    // The payload is read by the same strict slice.
    let pieces = load_hat(
        r#"{ "format_version": "1.26.40", "minecraft:persona_piece_meta": { "piece_id": "aa8d3c95-f189-cf08-0ae3-5cc1e090343b", "piece_name": "hat", "piece_type": "persona_top", "foo": 1, "body_variant": "huge" } }"#,
    );
    assert_eq!(
        refused(&pieces),
        [
            "error UnrecognizedField minecraft:persona_piece_meta/foo",
            "error InvalidEnumValue minecraft:persona_piece_meta/body_variant",
        ]
    );
}

#[test]
fn a_version_other_than_1_26_40_invalidates_the_piece() {
    // The loader's range is [1.26.40, 1.26.40]: A2 at every other version, the newer ones and
    // a pre-release of 1.26.40 included; `beta` has no slice either.
    for version in [
        r#""1.26.50""#,
        r#""1.26.60""#,
        r#""1.26.30""#,
        r#""1.26.40-beta.1""#,
        r#""beta""#,
        "[1, 26, 50]",
    ] {
        let pieces = load_hat(&format!(
            r#"{{ "format_version": {version}, "minecraft:persona_piece_meta": {{ "piece_id": "x", "piece_name": "hat", "piece_type": "persona_top" }} }}"#
        ));
        assert_eq!(
            refused(&pieces),
            ["error UnsupportedVersion format_version"],
            "{version}"
        );
        assert!(
            message(&first(&pieces, "UnsupportedVersion"))
                .starts_with("unsupported version: no parser available for version "),
            "{version}"
        );
    }
    // The array form of 1.26.40 is 1.26.40.
    let pieces = load_hat(
        r#"{ "format_version": [1, 26, 40], "minecraft:persona_piece_meta": { "piece_id": "x", "piece_name": "hat", "piece_type": "persona_top" } }"#,
    );
    assert_eq!(found(&pieces), Vec::<String>::new());
    // A value that is no version (A8).
    for version in ["true", r#""one""#, "1.5", "null"] {
        let pieces = load_hat(&format!(
            r#"{{ "format_version": {version}, "minecraft:persona_piece_meta": {{}} }}"#
        ));
        assert_eq!(
            refused(&pieces),
            ["error MalformedFormatVersion format_version"],
            "{version}"
        );
    }
}

#[test]
fn only_the_two_member_document_is_an_envelope() {
    // A third member beside the two (A9).
    let pieces = load_hat(
        r#"{ "format_version": "1.26.40", "minecraft:persona_piece_meta": { "piece_type": "persona_top" }, "piece_name": "hat" }"#,
    );
    assert_eq!(refused(&pieces), ["error MalformedInput"]);
    // Two members, but no payload under the key (A10).
    let pieces = load_hat(r#"{ "format_version": "1.26.40", "minecraft:persona_piece": {} }"#);
    assert_eq!(refused(&pieces), ["error MissingPayload"]);
    // A payload that is no object (`0x8`).
    let pieces = load_hat(r#"{ "format_version": "1.26.40", "minecraft:persona_piece_meta": 3 }"#);
    assert_eq!(
        refused(&pieces),
        ["error InvalidType minecraft:persona_piece_meta"]
    );
    // Without a version the payload key is an unknown member of the flat file.
    let pieces = load_hat(r#"{ "minecraft:persona_piece_meta": {} }"#);
    assert_eq!(
        refused(&pieces),
        [
            "error UnrecognizedField minecraft:persona_piece_meta",
            "error MissingRequiredField piece_id",
            "error MissingRequiredField piece_name",
            "error MissingRequiredField piece_type",
        ]
    );
}

const NOT_PARSED: &str = "Could not parse metafile json file pieces/hat/hat.meta.json";

#[test]
fn pm12_pm13_m2_a_meta_file_that_is_no_json_object_is_skipped() {
    for text in [
        "[]",
        r#"{ "piece_id": "aa8d3c95-f189-cf08-0ae3-5cc1e090343b", "piece_name": "hat""#,
        "3",
        r#""hat""#,
        "null",
        "",
        "{ a }",
    ] {
        let pieces = load_hat(text);
        assert!(pieces.pieces.is_empty(), "{text}");
        assert_eq!(found(&pieces), ["error MalformedInput"], "{text}");
        let finding = first(&pieces, "MalformedInput");
        // By legacy name: the variant and the definitions that were it.
        assert!(matches!(finding.code.name(), "MalformedInput"));
        assert_eq!(message(&finding), NOT_PARSED);
        assert_eq!(pieces.rejected.len(), 1);
        assert_eq!(pieces.rejected[0].origin.path.as_str(), FILE);
        assert!(pieces.rejected[0].diagnostics.has_errors());
    }
    // On its own the file is refused the same way.
    let rejected = PieceMetadata::parse(
        &PackPath::from_static(FILE),
        b"[]",
        &LoadContext::detached(),
    )
    .unwrap_err();
    assert_eq!(rejected.diagnostics.len(), 1);
    assert_eq!(
        message(rejected.diagnostics.first_error().unwrap()),
        NOT_PARSED
    );
}

#[test]
fn a_meta_file_is_read_as_the_game_s_reader_reads_it() {
    // A byte-order mark, comments and a member written twice (the last stands).
    let text = "\u{feff}// a comment\n{ \"piece_id\": \"x\", \"piece_name\": \"first\", /* another */ \"piece_type\": \"persona_hair\", \"piece_name\": \"hat\" }";
    let pieces = load_hat(text);
    assert_eq!(found(&pieces), Vec::<String>::new());
    assert_eq!(value(&pieces).piece_name.as_deref(), Some("hat"));
}

#[test]
fn content_after_the_root_is_reported_and_ignored() {
    let text = format!("\u{feff}{} trailing", meta(""));
    let pieces = load_hat(&text);
    assert_eq!(found(&pieces), ["info MalformedInput"]);
    assert_eq!(value(&pieces).piece_name.as_deref(), Some("hat"));
    let finding = first(&pieces, "MalformedInput");
    let span = finding.location.span.expect("the content's place");
    let start = span.offset as usize;
    assert_eq!(
        &text.as_bytes()[start..start + span.len as usize],
        b"trailing"
    );
}

/// A pack whose meta file cannot be read.
struct Unreadable<'a>(OfPack<'a>);

impl PieceSource for Unreadable<'_> {
    fn id(&self) -> &PackIdVersion {
        self.0.id()
    }

    fn paths(&self) -> impl Iterator<Item = &PackPath> + '_ {
        self.0.paths()
    }

    fn read(&self, path: &PackPath) -> Result<Bytes, std::io::ErrorKind> {
        if path.as_str() == FILE {
            Err(std::io::ErrorKind::PermissionDenied)
        } else {
            self.0.read(path)
        }
    }
}

#[test]
fn m1_a_meta_file_that_cannot_be_read_is_skipped() {
    let pack = piece_pack(&[(FILE, &meta("")), ("texts/en_US.lang", NAMES)]);
    let pieces = PersonaPiece::load_all(&Unreadable(OfPack(&pack)));
    assert!(pieces.pieces.is_empty());
    assert_eq!(found(&pieces), ["error ReadError"]);
    let finding = first(&pieces, "ReadError");
    assert!(matches!(
        finding.code,
        DiagnosticCode::ReadError {
            kind: std::io::ErrorKind::PermissionDenied,
            ..
        }
    ));
    assert_eq!(
        message(&finding),
        "Could not find expected metafile json file pieces/hat/hat.meta.json"
    );
    assert_eq!(pieces.rejected.len(), 1);
}

#[test]
fn pm16_m17_an_entry_with_an_empty_texture_is_skipped_deviation() {
    // `texture` is required from 1.26.40 (`strict.rs`); an empty one is a texture the builder
    // skips.
    let pieces = hat(
        &meta(
            r#""texture_sources": [{ "texture": "", "tint_map": "x_tintmap" }, { "texture": "hat" }]"#,
        ),
        &[("hat.png", "png")],
    );
    assert_eq!(
        found(&pieces),
        ["info MissingRequiredField texture_sources/0"]
    );
    let meta = value(&pieces);
    assert_eq!(meta.texture_sources.as_ref().map(Vec::len), Some(2));
    let textures = meta.textures();
    assert_eq!(textures.len(), 1);
    assert_eq!(textures[0].texture, "hat");
}

#[test]
fn m17_a_geometry_source_is_an_entry_of_the_texture_builder_only_when_it_says_so() {
    // Vanilla's 116 geometry sources name no texture: the builder skips them and nothing is
    // lost. One that sets a texture member but no texture loses that member.
    let pieces = hat(
        &meta(
            r#""geometry_sources": [{ "geometry": "geometry.hat" }, { "geometry": "geometry.hat", "tint_map": "t" }, { "geometry": "geometry.hat", "texture": "" }, { "geometry": "geometry.hat", "frames": 2 }, { "geometry": "geometry.hat", "animated": true }, { "geometry": "geometry.hat", "texture": "hat", "tint_map": "hat" }]"#,
        ),
        &[
            ("hat.png", "png"),
            ("hat.geometry.json", r#"{ "geometry.hat": {} }"#),
        ],
    );
    assert_eq!(
        found(&pieces),
        [
            "info MissingRequiredField geometry_sources/1",
            "info MissingRequiredField geometry_sources/2",
            "info MissingRequiredField geometry_sources/3",
            "info MissingRequiredField geometry_sources/4",
        ]
    );
    let textures = value(&pieces).textures();
    assert_eq!(textures.len(), 1);
    assert_eq!(textures[0].geometry_source, Some(5));
    assert_eq!(textures[0].tint_map.as_deref(), Some("hat"));
}

#[test]
fn pm17_the_game_s_defaults_are_on_the_resolved_view() {
    let pieces = hat(
        &meta(r#""texture_sources": [{ "texture": "hat" }]"#),
        &[("hat.png", "png")],
    );
    assert_eq!(found(&pieces), Vec::<String>::new());
    let read = value(&pieces);
    let source = &read.texture_sources.as_ref().unwrap()[0];
    assert_eq!(
        (
            source.animated,
            source.frames,
            source.use_face_uv,
            &source.tint_map
        ),
        (None, None, None, &None)
    );
    assert_eq!(read.allow_tint_override, None);
    assert!(read.allows_tint_override());
    let texture = &read.textures()[0];
    assert!(!texture.animated);
    assert_eq!(texture.frames, 1);
    assert!(!texture.use_face_uv);
    assert_eq!(texture.geometry_source, None);

    let set = hat(
        &meta(
            r#""allow_tint_override": false, "texture_sources": [{ "texture": "hat", "animated": true, "frames": 2, "use_face_uv": true, "tint_map": "hat" }], "geometry_sources": [{ "geometry": "geometry.hat", "texture": "hat", "animated": true, "frames": 3 }]"#,
        ),
        &[
            ("hat.png", "png"),
            ("hat.geometry.json", r#"{ "geometry.hat": {} }"#),
        ],
    );
    assert_eq!(found(&set), Vec::<String>::new());
    let meta = value(&set);
    assert!(!meta.allows_tint_override());
    let textures = meta.textures();
    assert!(textures[0].animated && textures[0].use_face_uv);
    assert_eq!(textures[0].frames, 2);
    assert_eq!(textures[0].tint_map.as_deref(), Some("hat"));
    assert!(textures[1].animated);
    assert_eq!(textures[1].frames, 3);
    // Never read for a geometry source.
    assert!(!textures[1].use_face_uv);
}

#[test]
fn pm18_m18_use_face_uv_is_no_member_of_a_geometry_source() {
    let pieces = hat(
        &meta(r#""geometry_sources": [{ "geometry": "geometry.hat", "use_face_uv": true }]"#),
        &[("hat.geometry.json", r#"{ "geometry.hat": {} }"#)],
    );
    assert_eq!(
        refused(&pieces),
        ["error UnrecognizedField geometry_sources/0/use_face_uv"]
    );
}

#[test]
fn a_member_of_the_wrong_json_type_invalidates_the_piece() {
    // 1.26.20 converted (the JSON library's string, bool and int conversions) in silence; the 1.26.40+
    // reader refuses the member (`0x8`, Revision 1).
    for (member, wrong) in [
        ("piece_id", "5"),
        ("piece_name", "5"),
        ("piece_name", "[]"),
        ("piece_type", "14"),
        ("piece_sub_type", "true"),
        ("body_variant", "2"),
        ("clothing_map", "{}"),
        ("face_clothing_map", "1.5"),
        ("allow_tint_override", r#""true""#),
        ("allow_tint_override", "1"),
        ("texture_sources", "{}"),
        ("texture_sources", r#""hat""#),
        ("geometry_sources", "7"),
        ("animation_sources", r#"{ "name": "a" }"#),
    ] {
        let pieces = load_hat(&format!(
            r#"{{ "piece_id": "aa8d3c95-f189-cf08-0ae3-5cc1e090343b", "piece_name": "hat", "piece_type": "persona_hair", "{member}": {wrong} }}"#
        ));
        assert_eq!(
            refused(&pieces),
            [format!("error InvalidType {member}")],
            "{member}: {wrong}"
        );
    }
}

#[test]
fn a_member_of_the_wrong_json_type_has_that_one_finding_deviation() {
    // A member the reader did not take is not also a missing one: no D4 for a `piece_type`
    // or a `texture` that is a number, no G3 for a `geometry` that is one.
    let pieces = load_hat(
        r#"{ "piece_id": "aa8d3c95-f189-cf08-0ae3-5cc1e090343b", "piece_name": "hat", "piece_type": 14 }"#,
    );
    assert_eq!(refused(&pieces), ["error InvalidType piece_type"]);

    let pieces = hat_with(r#""texture_sources": [{ "texture": 5 }]"#);
    assert_eq!(
        refused(&pieces),
        ["error InvalidType texture_sources/0/texture"]
    );
    let pieces =
        hat_with(r#""geometry_sources": [{ "geometry": "g", "texture": 5, "frames": 2 }]"#);
    assert_eq!(
        refused(&pieces),
        ["error InvalidType geometry_sources/0/texture"]
    );
    for wrong in ["5", "null"] {
        let pieces = hat_with(&format!(
            r#""geometry_sources": [{{ "geometry": {wrong} }}]"#
        ));
        assert_eq!(
            refused(&pieces),
            ["error InvalidType geometry_sources/0/geometry"],
            "{wrong}"
        );
    }
}

#[test]
fn a_null_member_is_a_member_of_the_wrong_type() {
    // Rule N2: 1.26.20 read `null` as absent; the 1.26.40+ reader finds the
    // member and its type refuses `null`.
    let pieces = hat_with(
        r#""piece_sub_type": null, "body_variant": null, "zone": null, "texture_sources": null, "tint_color": null, "color_swatches": null, "allow_tint_override": null"#,
    );
    assert_eq!(
        refused(&pieces),
        [
            "error InvalidType piece_sub_type",
            "error InvalidType body_variant",
            "error InvalidType zone",
            "error InvalidType texture_sources",
            "error InvalidType tint_color",
            "error UnrecognizedField color_swatches",
            "error InvalidType allow_tint_override",
        ]
    );
}

#[test]
fn the_members_of_an_entry_are_typed_too() {
    let pieces = hat(
        &meta(
            r#""texture_sources": [{ "texture": "hat", "tint_map": 1, "animated": "yes", "frames": "2", "use_face_uv": 1 }], "geometry_sources": [{ "geometry": 1, "arm_size": 1, "body_size": 2, "side": 3, "is_face": 1, "texture": 4, "tint_map": 5, "animated": 6, "frames": true }], "animation_sources": [{ "name": 1, "animationFile": 2 }]"#,
        ),
        &[("hat.png", "png")],
    );
    let expected: Vec<String> = [
        "texture_sources/0/tint_map",
        "texture_sources/0/animated",
        "texture_sources/0/frames",
        "texture_sources/0/use_face_uv",
        "geometry_sources/0/geometry",
        "geometry_sources/0/arm_size",
        "geometry_sources/0/body_size",
        "geometry_sources/0/side",
        "geometry_sources/0/is_face",
        "geometry_sources/0/texture",
        "geometry_sources/0/tint_map",
        "geometry_sources/0/animated",
        "geometry_sources/0/frames",
        "animation_sources/0/name",
        "animation_sources/0/animationFile",
    ]
    .iter()
    .map(|member| format!("error InvalidType {member}"))
    .collect();
    assert_eq!(refused(&pieces), expected);
}

#[test]
fn an_entry_that_is_no_object_invalidates_the_piece() {
    for member in ["texture_sources", "geometry_sources", "animation_sources"] {
        let pieces = hat(&meta(&format!(r#""{member}": [{{}}, 3]"#)), &[]);
        // The list's one finding is its bad entry, the earlier entry's dropped.
        assert_eq!(
            refused(&pieces),
            [format!("error InvalidType {member}/1")],
            "{member}"
        );
    }
}

#[test]
fn a_list_left_as_written_has_the_one_finding_of_its_bad_entry_deviation() {
    // Nothing is said about the entries before the bad one or after it.
    for (list, bad) in [
        (
            r#""texture_sources": [{ "foo": 1, "texture": "" }, 3, { "bar": 1 }]"#,
            "texture_sources/1",
        ),
        (
            r#""texture_sources": [3, { "foo": 1, "texture": "" }]"#,
            "texture_sources/0",
        ),
        (
            r#""geometry_sources": [{ "geometry": "g", "side": "up", "arm_size": "thin", "texture": "" }, []]"#,
            "geometry_sources/1",
        ),
        (
            r#""animation_sources": [{ "name": 1, "more": 2 }, "wave"]"#,
            "animation_sources/1",
        ),
        (r#""zone": ["head", 3]"#, "zone/1"),
    ] {
        let pieces = hat_with(list);
        assert_eq!(
            refused(&pieces),
            [format!("error InvalidType {bad}")],
            "{list}"
        );
        assert_eq!(pieces.rejected[0].diagnostics.len(), 2, "{list}");
    }
    // A number past a double in such a list keeps its finding, before the bad entry and
    // after it: the file holds another number than the author wrote there, which
    // is said of the file, not of an entry the model lacks.
    let pieces =
        hat_with(r#""texture_sources": [{ "texture": "", "foo": 1e999 }, 3, { "bar": -1e999 }]"#);
    assert_eq!(refused(&pieces), ["error InvalidType texture_sources/1"]);
    let told: Vec<String> = lines(&pieces)
        .into_iter()
        .filter(|line| line.starts_with("info InvalidValue"))
        .map(|line| line.replace(&format!(" {FILE}"), ""))
        .collect();
    assert_eq!(
        told,
        [
            "info InvalidValue texture_sources/0/foo",
            "info InvalidValue texture_sources/2/bar",
        ]
    );
    // A list every entry of which is read keeps the findings of each.
    let pieces = hat_with(r#""texture_sources": [{ "foo": 1, "texture": "" }, { "bar": 1 }]"#);
    assert_eq!(
        refused(&pieces),
        [
            "error UnrecognizedField texture_sources/0/foo",
            "error UnrecognizedField texture_sources/1/bar",
            "error MissingRequiredField texture_sources/1/texture",
        ]
    );
}

#[test]
fn frames_is_a_whole_number_of_32_bits() {
    // `optional<int>`: a whole number of 32 bits loads; any other spelling is `0x8` and the
    // piece is refused (1.26.20 read it with the JSON library's int conversion).
    for (frames, read) in [
        ("2", Some(2)),
        ("0", Some(0)),
        ("-1", Some(-1)),
        ("2147483647", Some(i32::MAX)),
        ("-2147483648", Some(i32::MIN)),
        ("2147483648", None),
        ("-2147483649", None),
        ("2.5", None),
        ("2.0", None),
        ("1e39", None),
        ("1e999", None),
        ("-1e999", None),
        ("18446744073709551616", None),
        ("null", None),
        ("true", None),
        (r#""2""#, None),
    ] {
        let pieces = hat(
            &meta(&format!(
                r#""texture_sources": [{{ "texture": "hat", "frames": {frames} }}]"#
            )),
            &[("hat.png", "png")],
        );
        if let Some(read) = read {
            assert_eq!(found(&pieces), Vec::<String>::new(), "{frames}");
            let source = &value(&pieces).texture_sources.as_ref().unwrap()[0];
            assert_eq!(source.frames, Some(read), "{frames}");
            assert_eq!(value(&pieces).textures()[0].frames, read, "{frames}");
        } else {
            assert_eq!(
                refused(&pieces),
                ["error InvalidType texture_sources/0/frames"],
                "{frames}"
            );
        }
    }
}

#[test]
fn the_texture_s_frame_count_is_the_game_s_reading_of_what_was_written() {
    // Without the member: the game's default.
    let pieces = hat(
        &meta(
            r#""texture_sources": [{ "texture": "hat" }], "geometry_sources": [{ "geometry": "g", "texture": "hat", "frames": 3 }]"#,
        ),
        &[("hat.png", "png"), ("hat.geometry.json", r#"{ "g": {} }"#)],
    );
    let textures = value(&pieces).textures();
    assert_eq!(textures[0].frames, 1);
    assert_eq!(textures[1].frames, 3);
    assert_eq!(textures[1].geometry_source, Some(0));

    // A model built by hand may hold `frames` as written in `extra` (no loaded file does):
    // the frame count is what the JSON library's int conversion makes of it.
    for (frames, read) in [
        ("2.5", 2),
        ("-1e39", i32::MIN),
        ("4294967298", 2),
        ("null", 1),
        ("true", 1),
        (r#""2""#, 0),
    ] {
        let mut meta = PieceMetadata::default();
        let mut source = bedrock_addon::families::persona::TextureSource::default();
        source.texture = Some("hat".to_owned());
        source.extra.insert(
            "frames".to_owned(),
            bedrock_addon::json::from_str::<bedrock_addon::json::Value>(frames).unwrap(),
        );
        meta.texture_sources = Some(vec![source]);
        assert_eq!(meta.textures()[0].frames, read, "{frames}");
    }
}

#[test]
fn pg05_g4_the_arm_size_of_a_geometry_source() {
    let pieces = hat(
        &meta(
            r#""geometry_sources": [{ "geometry": "g", "arm_size": "" }, { "geometry": "g", "arm_size": "slim" }, { "geometry": "g", "arm_size": "wide" }, { "geometry": "g" }]"#,
        ),
        &[("hat.geometry.json", r#"{ "g": {} }"#)],
    );
    assert_eq!(found(&pieces), Vec::<String>::new());
    let sources = value(&pieces).geometry_sources.as_ref().unwrap();
    let read: Vec<Option<ArmSize>> = sources.iter().map(GeometrySource::arm_size).collect();
    assert_eq!(read, [None, Some(ArmSize::Slim), Some(ArmSize::Wide), None]);
    assert_eq!(
        sources[0].arm_size,
        Some(Written::Known(ArmSize::Unspecified))
    );
    assert_eq!(sources[3].arm_size, None);
    // 1.26.20 took "thin" for wide; the 1.26.40+ enum refuses it.
    let thin = hat_with(r#""geometry_sources": [{ "geometry": "g", "arm_size": "thin" }]"#);
    assert_eq!(
        refused(&thin),
        ["error InvalidEnumValue geometry_sources/0/arm_size"]
    );
}

#[test]
fn the_body_size_of_a_geometry_source() {
    let pieces = hat(
        &meta(
            r#""geometry_sources": [{ "geometry": "g" }, { "geometry": "g", "body_size": "" }, { "geometry": "g", "body_size": "tall" }]"#,
        ),
        &[("hat.geometry.json", r#"{ "g": {} }"#)],
    );
    // The same enum as `body_variant`: the empty text is the constant the game takes for
    // medium.
    assert_eq!(found(&pieces), Vec::<String>::new());
    let sources = value(&pieces).geometry_sources.as_ref().unwrap();
    let read: Vec<Option<BodySize>> = sources.iter().map(GeometrySource::body_size).collect();
    assert_eq!(read, [None, Some(BodySize::Medium), Some(BodySize::Tall)]);
    let huge = hat_with(r#""geometry_sources": [{ "geometry": "g", "body_size": "huge" }]"#);
    assert_eq!(
        refused(&huge),
        ["error InvalidEnumValue geometry_sources/0/body_size"]
    );
}

/// The findings of a piece of `piece_type` whose one geometry source has `side`.
fn sided(piece_type: &str, side: &str) -> PersonaPieces {
    hat(
        &format!(
            r#"{{ "piece_id": "aa8d3c95-f189-cf08-0ae3-5cc1e090343b", "piece_name": "hat", "piece_type": "{piece_type}", "geometry_sources": [{{ "geometry": "g", "side": "{side}" }}] }}"#
        ),
        &[("hat.geometry.json", r#"{ "g": {} }"#)],
    )
}

/// G5's text goes to the game's error display, no content
/// log; G5a has none: informations of ours.
#[test]
fn pg06_g5_g5a_the_side_of_a_geometry_source_deviation() {
    const G5: &str = "info InvalidValue geometry_sources/0/side";
    let side_findings = |piece_type: &str, side: &str| -> Vec<String> {
        found(&sided(piece_type, side))
            .into_iter()
            .filter(|line| line.ends_with("/side"))
            .collect()
    };
    assert_eq!(side_findings("persona_arms", "left"), Vec::<String>::new());
    assert_eq!(side_findings("persona_top", "left"), [G5]);
    // G5 carries the game's text, G5a (ours) none.
    let top = sided("persona_top", "left");
    assert_eq!(
        message(&first(&top, "/side")),
        "Trying to assign the side attribute to a piece that is not an arm or leg (pieceType: persona_top, side: 'left')"
    );

    // The two arm and leg types a file can name, against every side the enum binds: `both` is
    // bound from 1.26.40 and still skipped by the geometry reader (G5a).
    for piece_type in ["persona_arms", "persona_legs"] {
        for side in ["left", "right", "both"] {
            let findings = side_findings(piece_type, side);
            if side == "both" {
                assert_eq!(findings, [G5], "{piece_type} {side}");
                let pieces = sided(piece_type, side);
                assert!(!message(&first(&pieces, "/side")).contains("Trying to assign"));
            } else {
                assert_eq!(findings, Vec::<String>::new(), "{piece_type} {side}");
            }
        }
    }
    // Any other type takes no side, whatever it says.
    for piece_type in ["persona_hair", "persona_body"] {
        for side in ["left", "right", "both"] {
            let pieces = sided(piece_type, side);
            assert!(
                message(&first(&pieces, "/side")).starts_with("Trying to assign"),
                "{piece_type} {side}"
            );
        }
    }
    // A side outside the enum, and a one-sided piece type, are refused (`strict.rs`).
    for side in ["", "Left", "up"] {
        assert!(sided("persona_arms", side).pieces.is_empty(), "{side}");
    }
    assert!(sided("persona_left_arm", "left").pieces.is_empty());
    // The one-sided types of a model built by hand keep 1.26.20's rule.
    assert!(PieceType::LeftArm.accepts_side("left"));
    assert!(!PieceType::RightArm.accepts_side("left"));
    // Without a side there is nothing to check.
    let none = hat(
        &meta(r#""geometry_sources": [{ "geometry": "g" }]"#),
        &[("hat.geometry.json", r#"{ "g": {} }"#)],
    );
    assert_eq!(found(&none), Vec::<String>::new());
}

#[test]
fn a_side_and_a_zone_make_a_side_piece() {
    let both = hat(
        r#"{ "piece_id": "aa8d3c95-f189-cf08-0ae3-5cc1e090343b", "piece_name": "hat", "piece_type": "persona_arms", "geometry_sources": [{ "geometry": "g" }, { "geometry": "g", "side": "left", "zone": "left_arm" }] }"#,
        &[("hat.geometry.json", r#"{ "g": {} }"#)],
    );
    assert!(value(&both).can_be_side_piece());
    // "Empty" is the reader's: a missing member and an empty list. A string is never empty to
    // it, whatever it holds.
    for members in [
        r#""side": "left", "zone": """#,
        r#""side": "right", "zone": "left_arm""#,
        r#""side": "both", "zone": ["a"]"#,
    ] {
        let pieces = hat(
            &format!(
                r#"{{ "piece_id": "aa8d3c95-f189-cf08-0ae3-5cc1e090343b", "piece_name": "hat", "piece_type": "persona_arms", "geometry_sources": [{{ "geometry": "g", {members} }}] }}"#
            ),
            &[("hat.geometry.json", r#"{ "g": {} }"#)],
        );
        assert!(value(&pieces).can_be_side_piece(), "{members}");
    }
    for members in [
        r#""side": "left""#,
        r#""zone": "left_arm""#,
        r#""side": "left", "zone": []"#,
    ] {
        let pieces = hat(
            &format!(
                r#"{{ "piece_id": "aa8d3c95-f189-cf08-0ae3-5cc1e090343b", "piece_name": "hat", "piece_type": "persona_arms", "geometry_sources": [{{ "geometry": "g", {members} }}] }}"#
            ),
            &[("hat.geometry.json", r#"{ "g": {} }"#)],
        );
        assert!(!value(&pieces).can_be_side_piece(), "{members}");
    }
}

#[test]
fn pg07_g6_is_face_is_a_string() {
    // 1.26.40 types it an optional string:
    // vanilla's `"true"` is the type, a boolean is not.
    let pieces = hat(
        &meta(
            r#""geometry_sources": [{ "geometry": "g", "is_face": "true" }, { "geometry": "g", "is_face": "false" }, { "geometry": "g" }, { "geometry": "g", "is_face": "yes" }]"#,
        ),
        &[("hat.geometry.json", r#"{ "g": {} }"#)],
    );
    assert_eq!(found(&pieces), Vec::<String>::new());
    let sources = value(&pieces).geometry_sources.as_ref().unwrap();
    let read: Vec<Option<bool>> = sources.iter().map(GeometrySource::is_face).collect();
    assert_eq!(read, [Some(true), Some(false), None, Some(false)]);
    assert_eq!(sources[0].is_face, Some(FaceFlag::Text("true".to_owned())));
    for flag in ["true", "false"] {
        let pieces = hat_with(&format!(
            r#""geometry_sources": [{{ "geometry": "g", "is_face": {flag} }}]"#
        ));
        assert_eq!(
            refused(&pieces),
            ["error InvalidType geometry_sources/0/is_face"]
        );
    }
    // A model built by hand may hold a boolean.
    assert!(FaceFlag::Bool(true).value());
}

#[test]
fn a6_animation_sources_of_a_piece_that_is_no_emote_are_never_played() {
    let pieces = hat(
        &meta(
            r#""animation_sources": [{ "name": "animation.wave", "animationFile": "wave.animation.json" }]"#,
        ),
        &[(
            "wave.animation.json",
            r#"{ "animations": { "animation.wave": {} } }"#,
        )],
    );
    assert_eq!(found(&pieces), ["info IgnoredMember animation_sources"]);
    assert!(matches!(
        first(&pieces, "IgnoredMember").code,
        DiagnosticCode::IgnoredMember { .. }
    ));
    // The entries are still checked and kept, as the game keeps them.
    assert_eq!(only(&pieces).animations.len(), 1);
    assert!(only(&pieces).emote.is_none());
}

#[test]
fn the_colour_picker_s_rule() {
    // Tinting allowed, and one of the five recolourable types (a swatch invalidates the piece
    // from 1.26.40: `strict::color_swatches_is_no_member_from_1_26_40`).
    for (text, allowed) in [
        (
            r#"{ "piece_id": "x", "piece_name": "hat", "piece_type": "persona_hair" }"#,
            true,
        ),
        (
            r#"{ "piece_id": "x", "piece_name": "hat", "piece_type": "persona_skin" }"#,
            true,
        ),
        (
            r#"{ "piece_id": "x", "piece_name": "hat", "piece_type": "persona_facial_hair" }"#,
            true,
        ),
        (
            r#"{ "piece_id": "x", "piece_name": "hat", "piece_type": "persona_mouth" }"#,
            true,
        ),
        (
            r#"{ "piece_id": "x", "piece_name": "hat", "piece_type": "persona_eyes" }"#,
            true,
        ),
        (
            r#"{ "piece_id": "x", "piece_name": "hat", "piece_type": "persona_top" }"#,
            false,
        ),
        (
            r#"{ "piece_id": "x", "piece_name": "hat", "piece_type": "persona_hood" }"#,
            false,
        ),
        (
            r#"{ "piece_id": "x", "piece_name": "hat", "piece_type": "persona_hair", "allow_tint_override": false }"#,
            false,
        ),
    ] {
        let document = PieceMetadata::parse(
            &PackPath::from_static(FILE),
            text.as_bytes(),
            &LoadContext::detached(),
        )
        .expect("an object");
        assert_eq!(
            document.value().is_color_change_allowed(),
            allowed,
            "{text}"
        );
    }
}

#[test]
fn m16_a_one_sided_piece_type_invalidates_the_piece() {
    // M4: internal-only from 1.26.40 (the binder's help text).
    for name in [
        "persona_left_leg",
        "persona_right_leg",
        "persona_left_arm",
        "persona_right_arm",
    ] {
        let pieces = load_hat(&format!(
            r#"{{ "piece_id": "aa8d3c95-f189-cf08-0ae3-5cc1e090343b", "piece_name": "hat", "piece_type": "{name}" }}"#
        ));
        assert_eq!(refused(&pieces), ["error InvalidEnumValue piece_type"]);
    }
    // The two-sided types have a name in both builds.
    for name in ["persona_arms", "persona_legs"] {
        let pieces = load_hat(&format!(
            r#"{{ "piece_id": "aa8d3c95-f189-cf08-0ae3-5cc1e090343b", "piece_name": "hat", "piece_type": "{name}" }}"#
        ));
        assert_eq!(found(&pieces), Vec::<String>::new());
    }
}
