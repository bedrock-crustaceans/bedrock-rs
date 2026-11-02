//! Write-back: a meta or store-info file is written as it was read,
//! member for member, and nothing the author did not write is invented.

use bedrock_addon::document::Content;
use bedrock_addon::families::persona::{
    PieceMetadata, PieceStoreInfo, PieceType, TintColorOrString, Written,
};
use bedrock_addon::json::Value;
use bedrock_addon::load::LoadContext;
use bedrock_addon::storage::PackPath;

/// A JSON text as a value.
fn json(text: &str) -> Value {
    bedrock_addon::json::from_str_jsonc::<Value>(text).unwrap_or_else(|e| panic!("{text}: {e}"))
}

/// A JSON text as this file's own reader takes it, token by token. The written text and the
/// input are compared in this form and not as the library's values, so that a reading the
/// library gets wrong on both sides cannot make them equal: a number token is a number and a
/// quoted text is a string, whatever the library reads them as.
#[derive(Debug, Clone, PartialEq)]
enum Token {
    Null,
    Bool(bool),
    /// An integer token that 64 bits hold.
    Whole(i128),
    /// A real token, or an integer token past 64 bits: the double it spells.
    Real(f64),
    String(String),
    Array(Vec<Token>),
    Object(Vec<(String, Token)>),
}

/// The reader of [`Token`]s: plain JSON, no comments.
struct Text<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Text<'_> {
    fn peek(&mut self) -> u8 {
        while self.bytes[self.at].is_ascii_whitespace() {
            self.at += 1;
        }
        self.bytes[self.at]
    }

    fn value(&mut self) -> Token {
        match self.peek() {
            b'{' => {
                self.at += 1;
                let mut members = Vec::new();
                loop {
                    match self.peek() {
                        b'}' => break,
                        b',' => self.at += 1,
                        _ => {
                            let key = self.string();
                            assert_eq!(self.peek(), b':');
                            self.at += 1;
                            members.push((key, self.value()));
                        }
                    }
                }
                self.at += 1;
                Token::Object(members)
            }
            b'[' => {
                self.at += 1;
                let mut elements = Vec::new();
                loop {
                    match self.peek() {
                        b']' => break,
                        b',' => self.at += 1,
                        _ => elements.push(self.value()),
                    }
                }
                self.at += 1;
                Token::Array(elements)
            }
            b'"' => Token::String(self.string()),
            b't' => self.word("true", Token::Bool(true)),
            b'f' => self.word("false", Token::Bool(false)),
            b'n' => self.word("null", Token::Null),
            _ => self.number(),
        }
    }

    fn word(&mut self, word: &str, token: Token) -> Token {
        assert!(self.bytes[self.at..].starts_with(word.as_bytes()));
        self.at += word.len();
        token
    }

    fn number(&mut self) -> Token {
        let start = self.at;
        while self
            .bytes
            .get(self.at)
            .is_some_and(|byte| b"+-0123456789.eE".contains(byte))
        {
            self.at += 1;
        }
        let token = std::str::from_utf8(&self.bytes[start..self.at]).unwrap();
        assert!(!token.is_empty(), "a number at byte {start}");
        let whole = token
            .parse::<i64>()
            .map(i128::from)
            .or_else(|_| token.parse::<u64>().map(i128::from));
        match whole {
            Ok(whole) => Token::Whole(whole),
            Err(_) => Token::Real(token.parse().unwrap_or_else(|e| panic!("{token}: {e}"))),
        }
    }

    fn string(&mut self) -> String {
        assert_eq!(self.bytes[self.at], b'"');
        self.at += 1;
        let mut out = Vec::new();
        loop {
            let byte = self.bytes[self.at];
            self.at += 1;
            match byte {
                b'"' => break,
                b'\\' => {
                    let escape = self.bytes[self.at];
                    self.at += 1;
                    let plain = match escape {
                        b'n' => '\n',
                        b't' => '\t',
                        b'r' => '\r',
                        b'b' => '\u{8}',
                        b'f' => '\u{c}',
                        b'u' => {
                            let hex = std::str::from_utf8(&self.bytes[self.at..self.at + 4]);
                            self.at += 4;
                            let unit = u32::from_str_radix(hex.unwrap(), 16).unwrap();
                            char::from_u32(unit).expect("no surrogate in these tests")
                        }
                        other => char::from(other),
                    };
                    out.extend_from_slice(plain.encode_utf8(&mut [0; 4]).as_bytes());
                }
                other => out.push(other),
            }
        }
        String::from_utf8(out).unwrap()
    }
}

/// A JSON text as tokens.
fn tokens(text: &str) -> Token {
    let mut text = Text {
        bytes: text.as_bytes(),
        at: 0,
    };
    let root = text.value();
    assert!(
        text.bytes[text.at..].iter().all(u8::is_ascii_whitespace),
        "content after the root"
    );
    root
}

/// Whether two texts are the same JSON: members compared by key, whatever their order; a
/// number equals a number of its kind (an integer token is not a real one) and nothing else.
/// Two reals are the same when they are the same double, bit for bit: `-0.0` is not `0.0`.
fn same(a: &Token, b: &Token) -> bool {
    match (a, b) {
        (Token::Real(a), Token::Real(b)) => a.to_bits() == b.to_bits(),
        (Token::Object(a), Token::Object(b)) => {
            a.len() == b.len()
                && a.iter().all(|(key, value)| {
                    b.iter()
                        .find(|(other, _)| other == key)
                        .is_some_and(|(_, other)| same(value, other))
                })
        }
        (Token::Array(a), Token::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(a, b)| same(a, b))
        }
        _ => a == b,
    }
}

fn meta(text: &str) -> PieceMetadata {
    PieceMetadata::parse(
        &PackPath::from_static("pieces/hat/hat.meta.json"),
        text.as_bytes(),
        &LoadContext::detached(),
    )
    .unwrap_or_else(|rejected| panic!("{text}: {rejected:?}"))
    .into_value()
}

fn store(text: &str) -> PieceStoreInfo {
    PieceStoreInfo::parse(
        &PackPath::from_static("pieces/hat/hat.store_info.json"),
        text.as_bytes(),
        &LoadContext::detached(),
    )
    .unwrap_or_else(|rejected| panic!("{text}: {rejected:?}"))
    .into_value()
}

/// Whether the 1.26.40+ reader refuses the meta file `text`.
fn refuses(text: &str) -> bool {
    PieceMetadata::parse(
        &PackPath::from_static("pieces/hat/hat.meta.json"),
        text.as_bytes(),
        &LoadContext::detached(),
    )
    .is_err()
}

/// The members every valid meta file of these tests begins with.
const REQUIRED: &str = r#""piece_id": "aa8d3c95-f189-cf08-0ae3-5cc1e090343b", "piece_name": "hat", "piece_type": "persona_hair""#;

/// Writes `text` back and requires the same JSON, and the same value when that is read again.
fn round_trip_meta(text: &str) -> String {
    let read = meta(text);
    let written = read.to_json().unwrap_or_else(|e| panic!("{text}: {e}"));
    assert!(
        same(&tokens(&written), &tokens(text)),
        "written:\n{written}\nread:\n{text}"
    );
    assert_eq!(meta(&written), read, "{text}");
    written
}

fn round_trip_store(text: &str) -> String {
    let read = store(text);
    let written = read.to_json().unwrap_or_else(|e| panic!("{text}: {e}"));
    assert!(
        same(&tokens(&written), &tokens(text)),
        "written:\n{written}\nread:\n{text}"
    );
    assert_eq!(store(&written), read, "{text}");
    written
}

// In the shape of the retail 1.26.45 `resource_packs/persona/pieces/` files (a body, an eyes
// piece, an emote, a shirt), with made-up ids and shortened lists; the third is
// `emote_wave/overhead_wave.meta.json` as shipped. The smoke test writes all 66 real files back.
const VANILLA: [&str; 4] = [
    r##"{
  "piece_id": "da1c5bbd-8f07-4e34-89b2-5c2d0e1a3b2f",
  "piece_name": "medium",
  "piece_type": "persona_body",
  "body_variant": "medium",
  "clothing_map": "medium_clothing_map",
  "face_clothing_map": "medium_face_clothing_map",
  "geometry_sources": [
    { "geometry": "geometry.medium_body", "arm_size": "", "zone": [ "body" ] },
    { "geometry": "geometry.medium_head", "arm_size": "", "zone": [ "head" ], "is_face": "true" },
    { "geometry": "geometry.medium_left_arm_slim", "arm_size": "slim", "zone": [ "left_arm" ] },
    { "geometry": "geometry.medium_left_arm_wide", "arm_size": "wide", "zone": [ "left_arm" ] }
  ]
}"##,
    r##"{
  "piece_id": "b3c0d6c1-56f9-4b9e-8f5d-6d0b8a8f1c11",
  "piece_name": "standard_eyes",
  "piece_type": "persona_eyes",
  "texture_sources": [
    { "texture": "standard_eyes", "tint_map": "standard_eyes_tintmap", "animated": true, "frames": 2, "use_face_uv": true }
  ],
  "tint_base_color": { "r_color": "#FFFFFF" },
  "tint_color": { "r_color": "#2b2b2B", "g_color": "#ffffff", "b_color": "#000000", "a_color": "#00000000" }
}"##,
    r##"{
  "piece_id": "4c8ae710-df2e-47cd-814d-cc7bf21a3d67",
  "piece_name": "overhead_wave",
  "piece_type": "persona_emote",
  "animation_sources": [
    { "name": "animation.overhead_wave", "animationFile": "overhead_wave.animation.json" }
  ]
}"##,
    r##"{
  "piece_id": "8f0b3b1e-5c3a-4d0e-9d55-1c2f7c1a9e10",
  "piece_name": "ari_shirt",
  "piece_type": "persona_top",
  "zone": [ "body_front_upper", "body_back_upper", "left_arm_upper", "right_arm_upper" ],
  "texture_sources": [ { "texture": "ari_shirt" } ]
}"##,
];

#[test]
fn a_vanilla_meta_file_is_written_as_it_was_read() {
    for text in VANILLA {
        round_trip_meta(text);
    }
}

#[test]
fn every_member_in_every_form_is_written_as_it_was_read() {
    round_trip_meta(
        r##"{
  "piece_id": "not-a-uuid", "piece_name": "", "piece_type": "persona_coco", "piece_sub_type": "persona_top",
  "body_variant": "", "clothing_map": "c", "face_clothing_map": "f", "zone": "one",
  "texture_sources": [ { "texture": "" }, { "texture": "", "tint_map": "t", "animated": false, "frames": -1, "use_face_uv": false } ],
  "geometry_sources": [ { "geometry": "g", "arm_size": "", "body_size": "", "side": "both", "zone": [], "is_face": "yes", "texture": "t", "tint_map": "m", "animated": true, "frames": 0 } ],
  "animation_sources": [ { "name": "n", "animationFile": "f" } ],
  "tint_color": "red", "tint_base_color": { "r_color": "", "a_color": "#12345678" },
  "allow_tint_override": true
}"##,
    );
    round_trip_meta(&format!(
        r#"{{ {REQUIRED}, "allow_tint_override": false, "zone": ["a", "b"] }}"#
    ));
}

#[test]
fn pm09_pm18_unknown_members_are_written_back() {
    // A file with one is refused when read (`strict.rs`); a model built by hand may hold them,
    // and they are written after the known members.
    let mut model = meta(&format!(
        r#"{{ {REQUIRED}, "geometry_sources": [{{ "geometry": "geometry.hat" }}] }}"#
    ));
    model.extra.insert("foo".to_owned(), json("1"));
    model.geometry_sources.as_mut().unwrap()[0]
        .extra
        .insert("use_face_uv".to_owned(), json("true"));
    let written = model.to_json().unwrap();
    let root = json(&written);
    let root = root.as_object().unwrap();
    assert_eq!(
        root.get("foo").and_then(|v| v.as_number()?.to_i64()),
        Some(1)
    );
    let source = root
        .get("geometry_sources")
        .and_then(|v| v.as_array())
        .and_then(|v| v.get(0))
        .and_then(|v| v.as_object())
        .unwrap();
    assert_eq!(
        source.get("use_face_uv").and_then(|v| v.as_bool()),
        Some(true)
    );
    // Known members first, in the game's order; the unknown ones after them.
    let keys: Vec<&str> = root.keys().map(|key| key.as_str()).collect();
    assert_eq!(
        keys,
        [
            "piece_id",
            "piece_name",
            "piece_type",
            "geometry_sources",
            "foo"
        ]
    );
    assert!(refuses(&written));
}

#[test]
fn a_real_keeps_the_sign_of_its_zero() {
    // The comparison of this file tells `-0.0` from `0.0`, and the writer keeps the sign in
    // a member kept as written (a store-info file keeps unknown members; a meta file with one
    // is refused).
    assert!(!same(&tokens("[-0.0]"), &tokens("[0.0]")));
    assert!(same(&tokens("[-0.0]"), &tokens("[-0.0]")));
    round_trip_store(r#"{ "foo": [-0.0, 0.0, -1.5] }"#);
    round_trip_store(r#"{ "piece_store_name": -0.0 }"#);
    let mut model = meta(&format!("{{ {REQUIRED} }}"));
    model
        .extra
        .insert("foo".to_owned(), json("[-0.0, 0.0, -1.5]"));
    let written = model.to_json().unwrap();
    assert!(same(
        &tokens(&written),
        &tokens(&format!(r#"{{ {REQUIRED}, "foo": [-0.0, 0.0, -1.5] }}"#))
    ));
}

#[test]
fn nothing_the_author_did_not_write_is_written() {
    let required = format!("{{ {REQUIRED} }}");
    assert!(same(
        &tokens(&round_trip_meta(&required)),
        &tokens(&required)
    ));
    let written = round_trip_meta(&format!(
        r#"{{ {REQUIRED}, "texture_sources": [{{ "texture": "hat" }}] }}"#
    ));
    assert!(!written.contains("frames"));
    assert!(!written.contains("animated"));
    assert!(!written.contains("allow_tint_override"));
    assert!(!written.contains("piece_sub_type"));
    // A model built by hand with nothing in it is written as nothing.
    assert!(same(
        &tokens(&PieceMetadata::default().to_json().unwrap()),
        &tokens("{}")
    ));
    assert!(same(&tokens(&round_trip_store("{}")), &tokens("{}")));
}

#[test]
fn members_of_another_json_type_are_refused_in_a_meta_file_and_kept_in_a_store_info() {
    for members in [
        r#""piece_sub_type": true, "body_variant": 2"#,
        r#""clothing_map": {}, "face_clothing_map": 1.5, "allow_tint_override": "true", "zone": 3"#,
        r#""zone": ["a", 3], "tint_color": 5, "tint_base_color": []"#,
        r#""texture_sources": {}, "geometry_sources": 7, "animation_sources": "x""#,
        r#""texture_sources": [{ "texture": "hat" }, 3], "geometry_sources": [null], "animation_sources": [[]]"#,
        r#""texture_sources": [{ "texture": 1, "tint_map": 2, "animated": "yes", "frames": 2.5, "use_face_uv": 1 }]"#,
        r#""texture_sources": [{ "texture": "t", "frames": 2147483648 }]"#,
        r#""geometry_sources": [{ "geometry": 1, "arm_size": 1, "body_size": 2, "side": 3, "zone": {}, "is_face": 1 }]"#,
        r#""animation_sources": [{ "name": 1, "animationFile": 2 }]"#,
        r##""tint_color": { "r_color": 5, "g_color": null, "b_color": [], "a_color": "#000000" }"##,
        r#""zone": null, "texture_sources": null, "tint_color": null, "allow_tint_override": null"#,
        r#""geometry_sources": [{ "geometry": "g", "is_face": true }]"#,
    ] {
        assert!(
            refuses(&format!("{{ {REQUIRED}, {members} }}")),
            "{members}"
        );
    }
    for text in [
        r#"{ "piece_store_name": 1, "piece_thumbnail": [] }"#,
        r#"{ "piece_store_name": null, "piece_thumbnail": null, "more": 1 }"#,
        r#"{ "piece_store_name": "dr.default.hat", "piece_thumbnail": "textures/persona_thumbnails/hat.png" }"#,
    ] {
        round_trip_store(text);
    }
}

#[test]
fn the_envelope_is_written_back() {
    let text = r#"{ "format_version": "1.26.40", "minecraft:persona_piece_meta": { "piece_id": "aa8d3c95-f189-cf08-0ae3-5cc1e090343b", "piece_name": "hat", "piece_type": "persona_top" } }"#;
    let written = round_trip_meta(text);
    let root = json(&written);
    let keys: Vec<&str> = root
        .as_object()
        .unwrap()
        .keys()
        .map(|key| key.as_str())
        .collect();
    assert_eq!(keys, ["format_version", "minecraft:persona_piece_meta"]);
    // Whatever 1.26.40 was written as.
    round_trip_meta(&format!(
        r#"{{ "format_version": [1, 26, 40], "minecraft:persona_piece_meta": {{ {REQUIRED} }} }}"#
    ));
    // The forms the 1.26.40+ loader refuses (`meta.rs`).
    for text in [
        r#"{ "format_version": 3, "minecraft:persona_piece_meta": {} }"#,
        r#"{ "format_version": "1.26.40", "piece_name": "hat" }"#,
        r#"{ "format_version": "1.26.40", "minecraft:persona_piece_meta": 3 }"#,
        r#"{ "format_version": "1.26.40", "minecraft:persona_piece_meta": {}, "piece_name": "hat" }"#,
    ] {
        assert!(refuses(text), "{text}");
    }
}

#[test]
fn a_member_set_in_the_model_stands_over_the_one_kept_as_written() {
    // A model built by hand holds `piece_type: 14` as written; once the model names a type,
    // that is the member.
    let mut read = PieceMetadata::default();
    read.extra.insert("piece_type".to_owned(), json("14"));
    read.extra.insert("tint_color".to_owned(), json("5"));
    read.piece_type = Some(Written::Known(PieceType::Hair));
    read.tint_color = Some(TintColorOrString::String(
        bedrock_addon::families::persona::HexColor("#ffffff".to_owned()),
    ));
    let written = read.to_json().unwrap();
    assert!(same(
        &tokens(&written),
        &tokens(r##"{ "piece_type": "persona_hair", "tint_color": "#ffffff" }"##)
    ));

    let mut read = store(r#"{ "piece_store_name": 1 }"#);
    read.piece_store_name = Some("hat".to_owned());
    assert!(same(
        &tokens(&read.to_json().unwrap()),
        &tokens(r#"{ "piece_store_name": "hat" }"#)
    ));
}

#[test]
fn a_number_past_a_double_is_held_at_the_largest_and_written() {
    // `1e999` has no JSON spelling as a value: read as the largest double, with a finding. A
    // meta file can hold one only in a member the reader refuses; a store-info file keeps it.
    let rejected = PieceMetadata::parse(
        &PackPath::from_static("pieces/hat/hat.meta.json"),
        br#"{ "piece_id": "x", "piece_name": "hat", "piece_type": "persona_hair", "texture_sources": [{ "texture": "t", "frames": 1e999 }] }"#,
        &LoadContext::detached(),
    )
    .unwrap_err();
    let findings: Vec<String> = rejected
        .diagnostics
        .iter()
        .map(crate::common::line)
        .collect();
    assert_eq!(
        findings,
        [
            "info InvalidValue pieces/hat/hat.meta.json texture_sources/0/frames",
            "error InvalidType pieces/hat/hat.meta.json texture_sources/0/frames",
            "error MalformedInput pieces/hat/hat.meta.json",
        ]
    );
    let document = PieceStoreInfo::parse(
        &PackPath::from_static("pieces/hat/hat.store_info.json"),
        br#"{ "piece_store_name": "n", "piece_thumbnail": "t", "foo": [1e999, -1e999] }"#,
        &LoadContext::detached(),
    )
    .unwrap();
    let findings: Vec<String> = document
        .diagnostics()
        .iter()
        .map(crate::common::line)
        .collect();
    assert_eq!(
        findings,
        [
            "info InvalidValue pieces/hat/hat.store_info.json foo/0",
            "info InvalidValue pieces/hat/hat.store_info.json foo/1",
            "info UnrecognizedField pieces/hat/hat.store_info.json foo",
        ]
    );
    let written = document.value().to_json().expect("finite numbers");
    assert_eq!(&store(&written), document.value());
    let root = json(&written);
    let foo = root
        .as_object()
        .unwrap()
        .get("foo")
        .unwrap()
        .as_array()
        .unwrap();
    let numbers: Vec<f64> = foo
        .as_slice()
        .iter()
        .map(|v| v.as_number().unwrap().to_f64_lossy())
        .collect();
    assert_eq!(numbers, [f64::MAX, f64::MIN]);
}

// ---- integers wider than 64 bits ----------------------------------------------------------
//
// The game's reader takes an integer token no 64-bit integer holds as the double it spells,
// and so does the library's once its reader does (core package CORE-BIGINT): until then such
// a token reaches this family as a string and these tests fail.

/// 2^64 as an integer token.
const WIDE: &str = "18446744073709551616";
/// The shortest decimal of the double that token spells: how the number is written.
const WIDE_WRITTEN: &str = "1.8446744073709552e19";
/// That double.
const WIDE_VALUE: f64 = 18_446_744_073_709_551_616.0;

/// Whether `value` is the number `wanted`.
fn is_number(value: Option<&Value>, wanted: f64) -> bool {
    value
        .and_then(Value::as_number)
        .is_some_and(|number| number.to_f64_lossy().to_bits() == wanted.to_bits())
}

#[test]
fn a_wide_integer_in_a_meta_file_is_a_number_of_the_wrong_type() {
    // Where a string or an `int` is read the member is refused (`0x8`), and the reader's one
    // finding says so: the token reaches it as a number, never as a quoted text.
    for (members, path) in [
        (format!(r#""clothing_map": {WIDE}"#), "clothing_map"),
        (
            format!(r#""texture_sources": [{{ "texture": {WIDE} }}]"#),
            "texture_sources/0/texture",
        ),
        (
            format!(r#""texture_sources": [{{ "texture": "hat", "frames": -{WIDE} }}]"#),
            "texture_sources/0/frames",
        ),
    ] {
        let rejected = PieceMetadata::parse(
            &PackPath::from_static("pieces/hat/hat.meta.json"),
            format!("{{ {REQUIRED}, {members} }}").as_bytes(),
            &LoadContext::detached(),
        )
        .unwrap_err();
        let errors: Vec<String> = rejected
            .diagnostics
            .iter()
            .map(|found| crate::common::line(found).replace(" pieces/hat/hat.meta.json", ""))
            .collect();
        assert_eq!(
            errors,
            [
                format!("error InvalidType {path}"),
                "error MalformedInput".to_owned()
            ],
            "{members}"
        );
    }
    // A model built by hand holding one writes it as the number it is.
    let mut model = meta(&format!("{{ {REQUIRED} }}"));
    model.extra.insert("foo".to_owned(), json(WIDE));
    let written = model.to_json().unwrap();
    assert!(is_number(
        json(&written).as_object().unwrap().get("foo"),
        WIDE_VALUE
    ));
    assert_eq!(written.matches(WIDE_WRITTEN).count(), 1, "{written}");
}

#[test]
fn a_wide_integer_in_a_store_info_is_a_number_of_the_wrong_type() {
    let text =
        format!(r#"{{ "piece_store_name": {WIDE}, "piece_thumbnail": -{WIDE}, "price": {WIDE} }}"#);
    let document = PieceStoreInfo::parse(
        &PackPath::from_static("pieces/hat/hat.store_info.json"),
        text.as_bytes(),
        &LoadContext::detached(),
    )
    .unwrap();
    let findings: Vec<String> = document
        .diagnostics()
        .iter()
        .map(|found| crate::common::line(found).replace(" pieces/hat/hat.store_info.json", ""))
        .collect();
    // Each member's own finding; a member of another type is not a missing one (S5).
    assert_eq!(
        findings,
        [
            "info InvalidType piece_store_name",
            "info InvalidType piece_thumbnail",
            "info UnrecognizedField price",
        ]
    );
    let read = document.value();
    assert_eq!(
        (&read.piece_store_name, &read.piece_thumbnail),
        (&None, &None)
    );
    assert!(is_number(read.extra.get("piece_store_name"), WIDE_VALUE));
    assert!(is_number(read.extra.get("piece_thumbnail"), -WIDE_VALUE));
    assert!(is_number(read.extra.get("price"), WIDE_VALUE));
    let written = round_trip_store(&text);
    assert_eq!(written.matches(WIDE_WRITTEN).count(), 3, "{written}");
    assert!(!written.contains(&format!("{WIDE}\"")), "{written}");
}

/// A small deterministic generator.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        // SplitMix64.
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn pick<'a>(&mut self, forms: &[&'a str]) -> &'a str {
        forms[(self.next() % forms.len() as u64) as usize]
    }

    /// `key: form` with one of `forms`, or nothing one time in three.
    fn member(&mut self, key: &str, forms: &[&str]) -> Option<String> {
        if self.next().is_multiple_of(3) {
            return None;
        }
        Some(format!(r#""{key}": {}"#, self.pick(forms)))
    }
}

const STRINGS: &[&str] = &[
    r#""hat""#,
    r#""""#,
    r#""persona_hair""#,
    r#""persona_left_arm""#,
    r#""persona_emote""#,
    r#""medium""#,
    r#""slim""#,
    r#""left""#,
    r#""both""#,
    r#""primary""#,
    r##""#ff0000""##,
    r##""#12345678""##,
    r#""red""#,
    r#""true""#,
    r#""é \"q\" \\ \n""#,
    r#""a/b.png""#,
];
const WRONG: &[&str] = &[
    "null",
    "true",
    "false",
    "0",
    "5",
    "-3",
    "2.5",
    "1e39",
    "-1e39",
    "2147483648",
    "[]",
    "{}",
    r#"["a"]"#,
    r#"["a", 3]"#,
    r##"{ "r_color": "#ffffff" }"##,
    r#"{ "r_color": 5, "x": 1 }"#,
    r#"[{}]"#,
    r#"[3]"#,
    "18446744073709551616",
];
const FLAGS: &[&str] = &["true", "false"];
const INTS: &[&str] = &["0", "1", "2", "-1", "2147483647", "-2147483648"];

/// One form of a member: mostly a string or what the member expects, sometimes something else.
fn form(rng: &mut Rng, expected: &[&'static str]) -> &'static str {
    match rng.next() % 4 {
        0 => rng.pick(WRONG),
        1 => rng.pick(STRINGS),
        _ => rng.pick(expected),
    }
}

fn object(members: Vec<Option<String>>) -> String {
    let members: Vec<String> = members.into_iter().flatten().collect();
    format!("{{ {} }}", members.join(", "))
}

/// `key: form` with one of `forms`, always.
fn always(rng: &mut Rng, key: &str, forms: &[&str]) -> Option<String> {
    Some(format!(r#""{key}": {}"#, rng.pick(forms)))
}

/// An entry with its `required` members and some of its `optional` ones, each in a form the
/// 1.26.40+ reader takes.
fn generated_entry(
    rng: &mut Rng,
    required: &[(&str, &[&'static str])],
    optional: &[(&str, &[&'static str])],
) -> String {
    let mut members = Vec::new();
    for (key, forms) in required {
        members.push(always(rng, key, forms));
    }
    for (key, forms) in optional {
        members.push(rng.member(key, forms));
    }
    object(members)
}

fn generated_list(
    rng: &mut Rng,
    required: &[(&str, &[&'static str])],
    optional: &[(&str, &[&'static str])],
) -> String {
    let count = rng.next() % 4;
    let entries: Vec<String> = (0..count)
        .map(|_| generated_entry(rng, required, optional))
        .collect();
    format!("[{}]", entries.join(", "))
}

/// Text forms the reader takes as a string.
const TEXTS: &[&str] = &[
    r#""hat""#,
    r#""""#,
    r##""#ff0000""##,
    r#""true""#,
    r#""é \"q\" \\ \n""#,
    r#""a/b.png""#,
];
const PIECE_TYPES: &[&str] = &[
    r#""persona_hair""#,
    r#""persona_emote""#,
    r#""persona_coco""#,
    r#""unsupported""#,
    r#""persona_arms""#,
];
const ARM_SIZES: &[&str] = &[r#""slim""#, r#""wide""#, r#""""#];
const BODY_SIZES: &[&str] = &[r#""smaller""#, r#""medium""#, r#""tall""#, r#""""#];
const SIDES: &[&str] = &[r#""left""#, r#""both""#, r#""right""#];
const ZONES: &[&str] = &[r#""head""#, r#"["head", "body"]"#, "[]"];

/// One meta file the 1.26.40+ reader takes, over the forms of every member.
fn generated_meta(rng: &mut Rng) -> String {
    let texture_sources = generated_list(
        rng,
        &[("texture", TEXTS)],
        &[
            ("tint_map", TEXTS),
            ("animated", FLAGS),
            ("frames", INTS),
            ("use_face_uv", FLAGS),
        ],
    );
    let geometry_sources = generated_list(
        rng,
        &[("geometry", TEXTS)],
        &[
            ("arm_size", ARM_SIZES),
            ("body_size", BODY_SIZES),
            ("side", SIDES),
            ("zone", ZONES),
            ("is_face", &[r#""true""#, r#""false""#, r#""yes""#]),
            ("texture", TEXTS),
            ("tint_map", TEXTS),
            ("animated", FLAGS),
            ("frames", INTS),
        ],
    );
    let animation_sources = generated_list(rng, &[("name", TEXTS), ("animationFile", TEXTS)], &[]);
    let tints: &[&str] = &[
        r##""#ff0000""##,
        r##"{ "r_color": "#ffffff" }"##,
        r##"{ "r_color": "#ffffff", "g_color": "#000000", "b_color": "red", "a_color": "" }"##,
        "{}",
    ];
    let members = vec![
        always(rng, "piece_id", TEXTS),
        always(rng, "piece_name", TEXTS),
        always(rng, "piece_type", PIECE_TYPES),
        rng.member("piece_sub_type", PIECE_TYPES),
        rng.member("body_variant", BODY_SIZES),
        rng.member("clothing_map", TEXTS),
        rng.member("face_clothing_map", TEXTS),
        rng.member("zone", ZONES),
        rng.member("texture_sources", &[&texture_sources]),
        rng.member("geometry_sources", &[&geometry_sources]),
        rng.member("animation_sources", &[&animation_sources]),
        rng.member("tint_color", tints),
        rng.member("tint_base_color", tints),
        rng.member("allow_tint_override", FLAGS),
    ];
    let flat = object(members);
    match rng.next() % 8 {
        0 => {
            format!(r#"{{ "format_version": "1.26.40", "minecraft:persona_piece_meta": {flat} }}"#)
        }
        _ => flat,
    }
}

#[test]
fn generated_meta_files_are_written_as_they_were_read() {
    let mut rng = Rng(0x5EED);
    let mut with_sources = 0;
    let mut in_envelope = 0;
    for _ in 0..4000 {
        let text = generated_meta(&mut rng);
        let read = meta(&text);
        if read
            .texture_sources
            .as_ref()
            .is_some_and(|list| !list.is_empty())
        {
            with_sources += 1;
        }
        if read.envelope.is_some() {
            in_envelope += 1;
        }
        round_trip_meta(&text);
    }
    assert!(
        with_sources > 1000 && in_envelope > 300,
        "{with_sources} {in_envelope}"
    );
}

#[test]
fn a_generated_meta_file_with_a_member_the_reader_refuses_is_refused() {
    let mut rng = Rng(0xBAD);
    for _ in 0..1000 {
        let text = generated_meta(&mut rng);
        let bad = format!(
            r#""{}": {}"#,
            rng.pick(&["foo", "color_swatches", "extra"]),
            rng.pick(WRONG)
        );
        let worse = match text.strip_suffix(" }") {
            Some(open) if !text.starts_with(r#"{ "format_version""#) => format!("{open}, {bad} }}"),
            _ => continue,
        };
        assert!(refuses(&worse), "{worse}");
    }
}

#[test]
fn generated_store_infos_are_written_as_they_were_read() {
    let mut rng = Rng(0xFACE);
    for _ in 0..2000 {
        let members = vec![
            {
                let value = form(&mut rng, STRINGS);
                rng.member("piece_store_name", &[value])
            },
            {
                let value = form(&mut rng, STRINGS);
                rng.member("piece_thumbnail", &[value])
            },
            rng.member("extra", WRONG),
        ];
        round_trip_store(&object(members));
    }
}
