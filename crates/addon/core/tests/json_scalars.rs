//! The scalar judge: how a scalar token is read against its member's declared type under each
//! profile a slice reads by. Its oracles are the readers it stands for: the strict wrappers
//! (`Strict`, and `NonStrict` under `with_reader_mode`), the post-hoc misread walk of a plain
//! member (`ConvertAndTell`), and the legacy block reader's fixtures (`addon/tests/block/
//! legacy_reader.rs`). The `JsonCpp` accept set is held against entity's generated game table
//! in the entity crate.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use bedrock_addon_core::document::Unknown;
use bedrock_addon_core::json::{
    BeyondWidth, Declared, Misreading, ReaderMode, Refusal, Respelt, ScalarRules, Scalars,
    StrictBool, StrictFloat, StrictInt, StrictString, Token, Value, Verdict, from_str, judge,
    keep_members, with_reader_mode,
};
use facet::Facet;

#[derive(Debug, Facet)]
struct Holder<T> {
    v: T,
}

/// Every token the tables run over: each JSON scalar type, integers at the edges of every
/// width, integral and fractional reals, numbers beyond a float and the non-strict reader's
/// fixtures.
const TOKENS: &[&str] = &[
    "0",
    "1",
    "2",
    "-1",
    "7",
    "16",
    "127",
    "128",
    "-128",
    "-129",
    "255",
    "256",
    "257",
    "32767",
    "65535",
    "65536",
    "2147483647",
    "2147483648",
    "-2147483648",
    "-2147483649",
    "3000000000",
    "4294967296",
    "9223372036854775807",
    "9223372036854775808",
    "18446744073709551615",
    "-9223372036854775808",
    "18446744073709551616",
    "0.0",
    "-0.0",
    "0.5",
    "1.0",
    "1.5",
    "-1.5",
    "3.0",
    "7.9",
    "1e1",
    "-7.9",
    "1e10",
    "1e20",
    "3.4e38",
    "1e39",
    "-1e39",
    "1e300",
    "\"1\"",
    "\"x\"",
    "\"true\"",
    "\"\"",
    "true",
    "false",
    "null",
];

fn value(text: &str) -> Value {
    from_str::<Value>(text).unwrap()
}

/// The verdict on `text` for a member of `declared` under `rules`.
fn verdict(declared: Declared, text: &str, rules: ScalarRules) -> Verdict {
    let value = value(text);
    let token = Token::of(&value).unwrap();
    judge(declared, token, rules)
}

const fn rules(scalars: Scalars, beyond_width: BeyondWidth) -> ScalarRules {
    ScalarRules::PLAIN
        .with_scalars(scalars)
        .with_beyond_width(beyond_width)
}

const STRICT: ScalarRules = rules(Scalars::Strict, BeyondWidth::Error);
const NON_STRICT: ScalarRules = rules(Scalars::NonStrict, BeyondWidth::Error);
const JSON_CPP: ScalarRules = rules(Scalars::JsonCpp, BeyondWidth::Error);

/// A value as the test compares it: a respelling, or a string (never respelt).
#[derive(Clone, Copy, Debug, PartialEq)]
enum Got {
    Bool(bool),
    UInt(u64),
    Int(i64),
    Real(f64),
    Text,
}

impl From<Respelt> for Got {
    fn from(respelt: Respelt) -> Self {
        match respelt {
            Respelt::Bool(b) => Self::Bool(b),
            Respelt::UInt(u) => Self::UInt(u),
            Respelt::Int(i) => Self::Int(i),
            Respelt::Real(x) => Self::Real(x),
            other => panic!("no respelling {other:?} is compared"),
        }
    }
}

/// A wrapper's read value, as the token a respelling would hand facet.
trait Read: Sized + for<'a> Facet<'a> {
    const DECLARED: Declared;
    fn token(&self) -> Got;
}

macro_rules! int {
    ($($t:ty => $bits:literal $signed:literal),*) => {$(
        impl Read for StrictInt<$t> {
            const DECLARED: Declared = Declared::Int { bits: $bits, signed: $signed };
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap, clippy::cast_lossless)]
            fn token(&self) -> Got {
                if (self.0 as i128) < 0 {
                    Got::Int(self.0 as i64)
                } else {
                    Got::UInt(self.0 as u64)
                }
            }
        }
    )*};
}
int!(i8 => 8 true, i16 => 16 true, i32 => 32 true, i64 => 64 true,
    u8 => 8 false, u16 => 16 false, u32 => 32 false, u64 => 64 false);

impl Read for StrictFloat<f32> {
    const DECLARED: Declared = Declared::Float { bits: 32 };
    fn token(&self) -> Got {
        Got::Real(f64::from(self.0))
    }
}

impl Read for StrictFloat<f64> {
    const DECLARED: Declared = Declared::Float { bits: 64 };
    fn token(&self) -> Got {
        Got::Real(self.0)
    }
}

impl Read for StrictBool {
    const DECLARED: Declared = Declared::Bool;
    fn token(&self) -> Got {
        Got::Bool(self.0)
    }
}

impl Read for StrictString {
    const DECLARED: Declared = Declared::Text;
    fn token(&self) -> Got {
        Got::Text
    }
}

/// The token as written, as the member reads it.
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn as_written(declared: Declared, text: &str) -> Got {
    let float = matches!(declared, Declared::Float { .. });
    match Token::of(&value(text)).unwrap() {
        Token::Bool(b) => Got::Bool(b),
        Token::UInt(u) if float => as_width(declared, u as f64),
        Token::Int(i) if float => as_width(declared, i as f64),
        Token::UInt(u) => Got::UInt(u),
        Token::Int(i) => Got::Int(i),
        Token::Real(x) => as_width(declared, x),
        Token::Str(_) => Got::Text,
        Token::Null => panic!("null reads as nothing"),
        other => panic!("no token {other:?} is compared"),
    }
}

/// `x` as a float member of `declared`'s width holds it.
#[allow(clippy::cast_possible_truncation)]
fn as_width(declared: Declared, x: f64) -> Got {
    match declared {
        Declared::Float { bits: 32 } => Got::Real(f64::from(x as f32)),
        _ => Got::Real(x),
    }
}

/// `W` under `mode` and the judge under `rules` agree on every token: what the wrapper refuses
/// is refused, and what it reads is the token as written or the respelling the judge hands
/// facet.
fn agree<W: Read>(mode: ReaderMode, rules: ScalarRules) {
    for text in TOKENS {
        let wrapper = with_reader_mode(mode, || {
            from_str::<Holder<W>>(&format!(r#"{{"v": {text}}}"#)).map(|h| h.v)
        });
        let verdict = verdict(W::DECLARED, text, rules);
        match (wrapper, &verdict) {
            (Err(_), Verdict::Refused(_)) => {}
            (Ok(read), Verdict::Keep) => {
                let written = as_written(W::DECLARED, text);
                // A float reads the token at its width; an integer as written.
                assert_eq!(read.token(), written, "{text} into {:?}", W::DECLARED);
            }
            (Ok(read), Verdict::Respell(respelt)) => {
                assert_eq!(
                    read.token(),
                    Got::from(*respelt),
                    "{text} into {:?}",
                    W::DECLARED
                );
            }
            (wrapper, verdict) => panic!(
                "{text} into {:?}: the wrapper {}, the judge says {verdict:?}",
                W::DECLARED,
                if wrapper.is_ok() {
                    "reads it"
                } else {
                    "refuses it"
                }
            ),
        }
    }
}

#[test]
fn strict_judges_every_token_as_the_strict_wrappers_read_it() {
    agree::<StrictInt<i8>>(ReaderMode::Strict, STRICT);
    agree::<StrictInt<i16>>(ReaderMode::Strict, STRICT);
    agree::<StrictInt<i32>>(ReaderMode::Strict, STRICT);
    agree::<StrictInt<i64>>(ReaderMode::Strict, STRICT);
    agree::<StrictInt<u8>>(ReaderMode::Strict, STRICT);
    agree::<StrictInt<u16>>(ReaderMode::Strict, STRICT);
    agree::<StrictInt<u32>>(ReaderMode::Strict, STRICT);
    agree::<StrictInt<u64>>(ReaderMode::Strict, STRICT);
    agree::<StrictFloat<f32>>(ReaderMode::Strict, STRICT);
    agree::<StrictFloat<f64>>(ReaderMode::Strict, STRICT);
    agree::<StrictBool>(ReaderMode::Strict, STRICT);
    agree::<StrictString>(ReaderMode::Strict, STRICT);
}

#[test]
fn non_strict_judges_every_token_as_the_non_strict_reader_reads_it() {
    agree::<StrictInt<i8>>(ReaderMode::NonStrict, NON_STRICT);
    agree::<StrictInt<i16>>(ReaderMode::NonStrict, NON_STRICT);
    agree::<StrictInt<i32>>(ReaderMode::NonStrict, NON_STRICT);
    agree::<StrictInt<i64>>(ReaderMode::NonStrict, NON_STRICT);
    agree::<StrictInt<u8>>(ReaderMode::NonStrict, NON_STRICT);
    agree::<StrictInt<u16>>(ReaderMode::NonStrict, NON_STRICT);
    agree::<StrictInt<u32>>(ReaderMode::NonStrict, NON_STRICT);
    agree::<StrictInt<u64>>(ReaderMode::NonStrict, NON_STRICT);
    agree::<StrictFloat<f32>>(ReaderMode::NonStrict, NON_STRICT);
    agree::<StrictFloat<f64>>(ReaderMode::NonStrict, NON_STRICT);
    agree::<StrictBool>(ReaderMode::NonStrict, NON_STRICT);
    agree::<StrictString>(ReaderMode::NonStrict, NON_STRICT);
}

#[test]
fn a_strict_refusal_names_the_type_or_the_range() {
    let u8 = Declared::Int {
        bits: 8,
        signed: false,
    };
    assert_eq!(
        verdict(u8, "1.0", STRICT),
        Verdict::Refused(Refusal::WrongType)
    );
    assert_eq!(
        verdict(u8, "\"1\"", STRICT),
        Verdict::Refused(Refusal::WrongType)
    );
    assert_eq!(
        verdict(u8, "null", STRICT),
        Verdict::Refused(Refusal::WrongType)
    );
    assert_eq!(
        verdict(u8, "256", STRICT),
        Verdict::Refused(Refusal::OutOfRange)
    );
    assert_eq!(
        verdict(u8, "-1", STRICT),
        Verdict::Refused(Refusal::OutOfRange)
    );
    let f32 = Declared::Float { bits: 32 };
    assert_eq!(
        verdict(f32, "1e39", STRICT),
        Verdict::Refused(Refusal::BeyondWidth)
    );
    assert_eq!(
        verdict(f32, "true", STRICT),
        Verdict::Refused(Refusal::WrongType)
    );
}

/// A 128 bit member takes every integer token of its sign, at the edges of 64 bits too, and
/// the judge is total over the widths (`bits: 128` signed once overflowed its range).
#[test]
fn a_128_bit_member_takes_every_integer_token_of_its_sign() {
    let i128 = Declared::Int {
        bits: 128,
        signed: true,
    };
    let u128 = Declared::Int {
        bits: 128,
        signed: false,
    };
    for text in [
        "0",
        "-1",
        "9223372036854775807",
        "18446744073709551615",
        "-9223372036854775808",
    ] {
        assert_eq!(verdict(i128, text, STRICT), Verdict::Keep, "i128 {text}");
    }
    for text in ["0", "9223372036854775807", "18446744073709551615"] {
        assert_eq!(verdict(u128, text, STRICT), Verdict::Keep, "u128 {text}");
    }
    for text in ["-1", "-9223372036854775808"] {
        assert_eq!(
            verdict(u128, text, STRICT),
            Verdict::Refused(Refusal::OutOfRange),
            "u128 {text}"
        );
    }
    // The `JsonCpp` reader truncates a real into them.
    assert_eq!(
        verdict(i128, "-7.9", JSON_CPP),
        Verdict::Respell(Respelt::Int(-7))
    );
    assert_eq!(
        verdict(u128, "-7.9", JSON_CPP),
        Verdict::Refused(Refusal::OutOfRange)
    );
}

/// `addon/tests/block/legacy_reader.rs`, the legacy block reader's fixtures, as respellings.
#[test]
fn the_legacy_reader_fixtures_respell_as_pinned() {
    let u8 = Declared::Int {
        bits: 8,
        signed: false,
    };
    for (text, read) in [
        ("true", Respelt::UInt(1)),
        ("false", Respelt::UInt(0)),
        ("257", Respelt::UInt(1)),
        ("7.9", Respelt::UInt(7)),
        ("3.0", Respelt::UInt(3)),
        ("-1", Respelt::UInt(255)),
        ("1e39", Respelt::UInt(255)),
        ("-2147483649", Respelt::UInt(255)),
    ] {
        assert_eq!(
            verdict(u8, text, NON_STRICT),
            Verdict::Respell(read),
            "light_emission {text}"
        );
    }
    for text in ["7", "16"] {
        assert_eq!(verdict(u8, text, NON_STRICT), Verdict::Keep, "{text}");
    }
    for (text, read) in [
        ("0", false),
        ("1", true),
        ("2", true),
        ("-1", true),
        ("0.0", false),
        ("0.5", true),
    ] {
        assert_eq!(
            verdict(Declared::Bool, text, NON_STRICT),
            Verdict::Respell(Respelt::Bool(read)),
            "face_dimming {text}"
        );
    }
    let f32 = Declared::Float { bits: 32 };
    assert_eq!(
        verdict(f32, "true", NON_STRICT),
        Verdict::Respell(Respelt::Real(1.0))
    );
    assert_eq!(
        verdict(f32, "false", NON_STRICT),
        Verdict::Respell(Respelt::Real(0.0))
    );
    for declared in [f32, u8] {
        assert_eq!(
            verdict(declared, "\"1\"", NON_STRICT),
            Verdict::Refused(Refusal::WrongType)
        );
        assert_eq!(
            verdict(declared, "\"3\"", NON_STRICT),
            Verdict::Refused(Refusal::WrongType)
        );
    }
}

/// A plain member of `T`, read by facet from every token it takes, and the misread walk's word
/// on it: the judge under `ConvertAndTell` says the same.
fn tells_as_the_misread_walk<T: for<'a> Facet<'a>>(declared: Declared) {
    for beyond_width in [BeyondWidth::ClampAndTell, BeyondWidth::Error] {
        let rules = rules(Scalars::ConvertAndTell, beyond_width);
        for text in TOKENS {
            let source = format!(r#"{{"v": {text}}}"#);
            let Ok(read) = from_str::<Holder<T>>(&source) else {
                // facet's own refusal: what the judge says is never told.
                continue;
            };
            let misread: Vec<Misreading> =
                keep_members(&read, &value(&source), &mut Unknown::new())
                    .1
                    .into_iter()
                    .map(|misread| misread.kind)
                    .collect();
            let expected = match (misread.as_slice(), beyond_width) {
                ([], _) => Verdict::Keep,
                ([Misreading::WrongType], _) => Verdict::Converted,
                ([Misreading::BeyondFloat], BeyondWidth::ClampAndTell) => Verdict::BeyondFloat,
                ([Misreading::BeyondFloat], _) => Verdict::Refused(Refusal::BeyondWidth),
                (other, _) => panic!("{text}: {other:?}"),
            };
            assert_eq!(
                verdict(declared, text, rules),
                expected,
                "{text} into {declared:?}"
            );
        }
    }
}

#[test]
fn convert_and_tell_tells_what_the_misread_walk_of_a_plain_member_tells() {
    tells_as_the_misread_walk::<i8>(Declared::Int {
        bits: 8,
        signed: true,
    });
    tells_as_the_misread_walk::<i32>(Declared::Int {
        bits: 32,
        signed: true,
    });
    tells_as_the_misread_walk::<u8>(Declared::Int {
        bits: 8,
        signed: false,
    });
    tells_as_the_misread_walk::<u64>(Declared::Int {
        bits: 64,
        signed: false,
    });
    tells_as_the_misread_walk::<f32>(Declared::Float { bits: 32 });
    tells_as_the_misread_walk::<f64>(Declared::Float { bits: 64 });
    tells_as_the_misread_walk::<bool>(Declared::Bool);
    tells_as_the_misread_walk::<String>(Declared::Text);
}

#[test]
fn json_cpp_takes_the_token_types_of_the_game_table() {
    // entity's generated table: `int` INT|UINT|REAL, `float` INT|UINT|REAL|BOOL, `string`
    // STRING, `bool` BOOL (the entity crate holds the table to `JsonCpp::accepts`).
    let int = Declared::Int {
        bits: 32,
        signed: true,
    };
    let float = Declared::Float { bits: 32 };
    let takes = |declared: Declared, text: &str| {
        !matches!(verdict(declared, text, JSON_CPP), Verdict::Refused(_))
    };
    // (token, int, float, string, bool)
    for (text, row) in [
        ("null", [false, false, false, false]),
        ("-7", [true, true, false, false]),
        ("7", [true, true, false, false]),
        ("7.5", [true, true, false, false]),
        ("\"7\"", [false, false, true, false]),
        ("true", [false, true, false, true]),
    ] {
        assert_eq!(
            [
                takes(int, text),
                takes(float, text),
                takes(Declared::Text, text),
                takes(Declared::Bool, text)
            ],
            row,
            "{text}"
        );
    }
    // A real into an integer is its truncation toward zero; a bool into a float `0.0` or
    // `1.0`.
    assert_eq!(
        verdict(int, "7.5", JSON_CPP),
        Verdict::Respell(Respelt::UInt(7))
    );
    assert_eq!(
        verdict(int, "-7.5", JSON_CPP),
        Verdict::Respell(Respelt::Int(-7))
    );
    assert_eq!(
        verdict(float, "true", JSON_CPP),
        Verdict::Respell(Respelt::Real(1.0))
    );
}

#[test]
fn a_number_beyond_its_float_is_refused_or_clamped_and_told_by_the_slice() {
    let f32 = Declared::Float { bits: 32 };
    for scalars in [Scalars::Strict, Scalars::ConvertAndTell, Scalars::NonStrict] {
        assert_eq!(
            verdict(f32, "1e39", rules(scalars, BeyondWidth::ClampAndTell)),
            Verdict::BeyondFloat,
            "{scalars:?}"
        );
        assert_eq!(
            verdict(f32, "1e39", rules(scalars, BeyondWidth::Error)),
            Verdict::Refused(Refusal::BeyondWidth),
            "{scalars:?}"
        );
        assert_eq!(
            verdict(f32, "3.4e38", rules(scalars, BeyondWidth::Error)),
            Verdict::Keep
        );
    }
    // An `f64` holds every number the door reads.
    assert_eq!(
        verdict(Declared::Float { bits: 64 }, "1e300", STRICT),
        Verdict::Keep
    );
}

// --- the pre-pass over a whole text ----------------------------------------------------------

mod prepass {
    use super::*;
    use bedrock_addon_core::json::{
        JsonSlice, NullRule, ScalarFailure, SliceError, parse_at, parse_with_rules,
    };

    /// The members the block family reads with strict wrappers, wrapped.
    #[derive(Debug, PartialEq, Facet)]
    struct Wrapped {
        #[facet(default = StrictFloat(0.0_f32))]
        x: StrictFloat<f32>,
        #[facet(default = StrictInt(0_u8))]
        n: StrictInt<u8>,
        #[facet(default = StrictBool(false))]
        b: StrictBool,
        #[facet(default = StrictString(String::new()))]
        s: StrictString,
        #[facet(default)]
        list: Vec<StrictInt<i16>>,
        #[facet(default)]
        maybe: Option<StrictInt<u32>>,
    }

    /// The same members, plain.
    #[derive(Debug, PartialEq, Facet)]
    struct Plain {
        #[facet(default)]
        x: f32,
        #[facet(default)]
        n: u8,
        #[facet(default)]
        b: bool,
        #[facet(default)]
        s: String,
        #[facet(default)]
        list: Vec<i16>,
        #[facet(default)]
        maybe: Option<u32>,
    }

    impl From<Wrapped> for Plain {
        fn from(w: Wrapped) -> Self {
            Self {
                x: w.x.0,
                n: w.n.0,
                b: w.b.0,
                s: w.s.0,
                list: w.list.into_iter().map(|i| i.0).collect(),
                maybe: w.maybe.map(|m| m.0),
            }
        }
    }

    /// A failed read as the finding sees it: its code, path and span.
    fn failure(error: &SliceError) -> (String, String, (u32, u32)) {
        (
            error.code_with_path(error.has_path()).name().to_owned(),
            error.relative_path().to_string(),
            (error.span.offset, error.span.len),
        )
    }

    const TEXTS: &[&str] = &[
        r#"{"x": 1.5, "n": 7, "b": true, "s": "a", "list": [1, -2], "maybe": 3}"#,
        r#"{"x": "1.5"}"#,
        r#"{"x": true}"#,
        r#"{"x": 1e39}"#,
        r#"{"x": null}"#,
        r#"{"n": 1.0}"#,
        r#"{"n": 256}"#,
        r#"{"n": -1}"#,
        r#"{"n": "7"}"#,
        r#"{"n": 257}"#,
        r#"{"n": 7.9}"#,
        r#"{"n": true}"#,
        r#"{"n": 1e39}"#,
        r#"{"n": -2147483649}"#,
        r#"{"b": 1}"#,
        r#"{"b": 0.0}"#,
        r#"{"b": "true"}"#,
        r#"{"s": 5}"#,
        r#"{"s": true}"#,
        r#"{"list": [1, "2", 3]}"#,
        r#"{"list": [1, 2.5, 40000]}"#,
        r#"{"list": [null]}"#,
        r#"{"x": [1]}"#,
        r#"{"x": []}"#,
        r#"{"n": ["a", 2]}"#,
        r#"{"b": {"a": 1}}"#,
        r#"{"s": {}}"#,
        r#"{"list": [[1]]}"#,
        r#"{"maybe": null}"#,
        r#"{"maybe": -3}"#,
        // Two faults: the first in document order is the one told.
        r#"{"s": 5, "n": "x"}"#,
        // Comments between the tokens and an integer past 64 bits: one copy, and the
        // respellings on it.
        "{\"n\": /* a */ true, // b\n \"x\": 18446744073709551616, \"b\": 2}",
        // A member written twice: the last one is read.
        r#"{"n": "x", "n": 7}"#,
        r#"{"n": 7, "n": true}"#,
    ];

    /// The plain members under `rules` read every text as the wrappers do under `mode`: the
    /// same value, or the same failure at the same place.
    fn agree(mode: ReaderMode, rules: ScalarRules, null: Option<NullRule>) {
        for text in TEXTS {
            let slice = JsonSlice::new(text);
            let wrapped = with_reader_mode(mode, || parse_at::<Wrapped>(slice));
            let plain = parse_with_rules::<Plain>(slice, null, rules);
            match (wrapped, plain) {
                (Ok(wrapped), Ok(plain)) => {
                    assert_eq!(Plain::from(wrapped), plain.value, "{text}");
                    assert!(plain.told.is_empty(), "{text}: {:?}", plain.told);
                }
                (Err(wrapped), Err(ScalarFailure::Read(plain))) => {
                    assert_eq!(failure(&wrapped), failure(&plain), "{text}");
                }
                (wrapped, plain) => panic!("{text}: wrapped {wrapped:?}, plain {plain:?}"),
            }
        }
    }

    #[test]
    fn plain_members_under_the_strict_rule_read_as_the_strict_wrappers() {
        agree(ReaderMode::Strict, STRICT, Some(NullRule::Rejected));
        agree(ReaderMode::Strict, STRICT, None);
    }

    #[test]
    fn plain_members_under_the_non_strict_rule_read_as_the_wrappers_of_the_non_strict_reader() {
        agree(ReaderMode::NonStrict, NON_STRICT, None);
    }

    #[test]
    fn a_respelt_token_borrows_the_rest_from_the_text() {
        #[derive(Debug, Facet)]
        struct Borrowing<'a> {
            name: std::borrow::Cow<'a, str>,
            on: bool,
        }
        let text = r#"{"name": "stone", "on": 1}"#;
        let read = parse_with_rules::<Borrowing<'_>>(JsonSlice::new(text), None, NON_STRICT)
            .unwrap()
            .value;
        assert!(read.on);
        assert!(
            matches!(read.name, std::borrow::Cow::Borrowed("stone")),
            "{read:?}"
        );
    }

    #[test]
    fn a_refused_member_is_left_out_and_told_under_default_and_tell() {
        let rules = STRICT.with_default_and_tell(true);
        let text = r#"{"n": "x", "b": true, "s": 5}"#;
        let read = parse_with_rules::<Plain>(JsonSlice::new(text), None, rules).unwrap();
        assert_eq!(read.value.n, 0);
        assert!(read.value.b);
        assert_eq!(read.value.s, "");
        let told: Vec<(String, Misreading)> = read
            .told
            .iter()
            .map(|t| (t.path.to_string(), t.kind))
            .collect();
        assert_eq!(
            told,
            [
                ("n".to_owned(), Misreading::Defaulted),
                ("s".to_owned(), Misreading::Defaulted)
            ]
        );
    }

    /// Block's `minecraft:destructible_by_explosion`, wrapped.
    #[derive(Debug, PartialEq, Facet)]
    #[facet(untagged)]
    #[repr(u8)]
    enum ShorthandWrapped {
        Enabled(bool),
        Params(ParamsWrapped),
    }

    #[derive(Debug, PartialEq, Facet)]
    struct ParamsWrapped {
        resistance: StrictFloat<f32>,
    }

    /// The same, plain.
    #[derive(Debug, PartialEq, Facet)]
    #[facet(untagged)]
    #[repr(u8)]
    enum Shorthand {
        Enabled(bool),
        Params(Params),
    }

    #[derive(Debug, PartialEq, Facet)]
    struct Params {
        resistance: f32,
    }

    #[test]
    fn a_scalar_no_arm_takes_is_refused_before_the_read() {
        let refused =
            |text: &str| match parse_with_rules::<Shorthand>(JsonSlice::new(text), None, STRICT) {
                Err(ScalarFailure::Refused(misread)) => Some(misread.path.to_string()),
                _ => None,
            };
        assert_eq!(refused("5"), Some(String::new()));
        assert_eq!(refused("\"true\""), Some(String::new()));
        assert_eq!(refused("null"), Some(String::new()));
        assert_eq!(refused("true"), None);
        assert_eq!(refused(r#"{"resistance": 3}"#), None);
    }

    /// A scalar refused before an untagged enum's scalar no arm takes, in document order.
    #[derive(Debug, Facet)]
    struct BeforeTheArm {
        #[facet(default)]
        s: String,
        #[facet(default)]
        n: u8,
        shorthand: Shorthand,
    }

    #[test]
    fn a_fault_before_a_refused_arm_is_the_one_told() {
        let read = |text: &str| {
            parse_with_rules::<BeforeTheArm>(JsonSlice::new(text), None, STRICT).map(|_| ())
        };
        // The refused string comes first: the typed read fails there, as it does with no arm
        // after it (where the strict wrapper's refusal is, `agree`).
        let Err(ScalarFailure::Read(error)) = read(r#"{"s": 5, "shorthand": 5}"#) else {
            panic!("the refused string is not the one told");
        };
        let Err(ScalarFailure::Read(alone)) =
            parse_with_rules::<Plain>(JsonSlice::new(r#"{"s": 5, "n": 1}"#), None, STRICT)
        else {
            panic!("the string is not refused");
        };
        assert_eq!(failure(&error), failure(&alone));
        assert_eq!(error.relative_path().to_string(), "s");
        // So does an array where a scalar belongs, refused at its last element.
        let Err(ScalarFailure::Read(error)) = read(r#"{"n": [1], "shorthand": 5}"#) else {
            panic!("the array is not the one told");
        };
        assert_eq!(error.relative_path().to_string(), "n/0");
        // A respelling is no fault: the arm is refused as before.
        let respelling = STRICT.with_scalars(Scalars::NonStrict);
        let Err(ScalarFailure::Refused(misread)) = parse_with_rules::<BeforeTheArm>(
            JsonSlice::new(r#"{"n": true, "shorthand": 5}"#),
            None,
            respelling,
        ) else {
            panic!("the arm is not refused");
        };
        assert_eq!(misread.path.to_string(), "shorthand");
        // With nothing before it, the arm is refused before the read.
        assert!(matches!(
            read(r#"{"shorthand": 5, "s": 5}"#),
            Err(ScalarFailure::Refused(misread)) if misread.path.to_string() == "shorthand"
        ));
    }

    #[test]
    fn a_refused_arm_carries_the_span_of_its_scalar_in_the_file() {
        // The payload is a slice of a larger file: the span is the file's.
        let file = r#"{"x": {"s": "a", "shorthand": "no"}}"#;
        let start = file.find(r#"{"s""#).expect("the payload");
        let slice = JsonSlice {
            text: &file[start..file.len() - 1],
            offset: start,
        };
        let Err(ScalarFailure::Refused(refused)) =
            parse_with_rules::<BeforeTheArm>(slice, None, STRICT)
        else {
            panic!("the arm is not refused");
        };
        let value = file.find(r#""no""#).expect("the value");
        let span = bedrock_addon_core::diagnostic::Span {
            offset: u32::try_from(value).expect("a short file"),
            len: 4,
        };
        assert_eq!(refused.span, Some(span));
        assert_eq!(refused.path.to_string(), "shorthand");
        // The finding as the misread tells it, now at the scalar's place.
        let at = bedrock_addon_core::diagnostic::JsonPath::new().child("x");
        let finding = refused.spanned_finding(&at);
        assert_eq!(finding.location.span, Some(span));
        let plain = refused.finding(&at);
        assert_eq!(
            plain.location.span, None,
            "the misread's own finding is unchanged"
        );
        assert_eq!(
            (finding.code, finding.severity),
            (plain.code, plain.severity)
        );
        // A refused arm read after an earlier fault was read after all keeps its span too.
        let respelling = STRICT.with_scalars(Scalars::NonStrict);
        let text = r#"{"n": true, "shorthand": 5}"#;
        let Err(ScalarFailure::Refused(refused)) =
            parse_with_rules::<BeforeTheArm>(JsonSlice::new(text), None, respelling)
        else {
            panic!("the arm is not refused");
        };
        let scalar = u32::try_from(text.find('5').expect("the scalar")).expect("short");
        assert_eq!(
            refused.span,
            Some(bedrock_addon_core::diagnostic::Span {
                offset: scalar,
                len: 1
            })
        );
    }

    #[test]
    fn a_member_inside_an_untagged_arm_is_refused_as_the_wrapper_refuses_it() {
        for text in [
            r#"{"resistance": "3"}"#,
            r#"{"resistance": true}"#,
            r#"{"resistance": 1e39}"#,
            r#"{"resistance": null}"#,
            r#"{"resistance": 3, "zz": 1}"#,
            r#"{"resistance": 3}"#,
            "false",
        ] {
            let slice = JsonSlice::new(text);
            let wrapped = parse_at::<ShorthandWrapped>(slice);
            let plain = parse_with_rules::<Shorthand>(slice, Some(NullRule::Rejected), STRICT);
            match (wrapped, plain) {
                (Ok(ShorthandWrapped::Enabled(a)), Ok(read)) => {
                    assert_eq!(read.value, Shorthand::Enabled(a), "{text}");
                }
                (Ok(ShorthandWrapped::Params(a)), Ok(read)) => assert_eq!(
                    read.value,
                    Shorthand::Params(Params {
                        resistance: a.resistance.0
                    }),
                    "{text}"
                ),
                (Err(wrapped), Err(ScalarFailure::Read(plain))) => {
                    assert_eq!(failure(&wrapped), failure(&plain), "{text}");
                }
                (wrapped, plain) => panic!("{text}: wrapped {wrapped:?}, plain {plain:?}"),
            }
        }
    }

    /// Under rule N1 the value handed back is the one the pre-pass walked: the members read as
    /// absent are out of it, and named, so a caller does not prune it again.
    /// A struct member, and a struct in a list, the text may write as `null`.
    #[derive(Debug, PartialEq, Facet)]
    struct Nested {
        #[facet(default)]
        inner: Params,
        #[facet(default)]
        list: Vec<Params>,
        #[facet(default)]
        n: u8,
    }

    impl Default for Params {
        fn default() -> Self {
            Self { resistance: 0.0 }
        }
    }

    #[test]
    fn a_null_read_as_a_struct_is_told_unless_the_rules_leave_it_to_the_null_rule() {
        let text = r#"{"inner": null, "list": [null, {"resistance": 1}], "n": "7"}"#;
        let told = |rules: ScalarRules| {
            let read = parse_with_rules::<Nested>(JsonSlice::new(text), None, rules).unwrap();
            let told: Vec<(String, Misreading)> = read
                .told
                .iter()
                .map(|misread| (misread.path.to_string(), misread.kind))
                .collect();
            (read.value, told)
        };
        let plain = ScalarRules::PLAIN;
        assert!(plain.tells_null_struct, "today's rule: told");
        let (value, all) = told(plain);
        assert_eq!(
            all,
            [
                ("inner".to_owned(), Misreading::WrongType),
                ("list/0".to_owned(), Misreading::WrongType),
                ("n".to_owned(), Misreading::WrongType),
            ]
        );
        // Left to the family's null rule: the struct is read as before, and only the
        // conversion is told.
        let (quiet_value, quiet) = told(plain.with_tells_null_struct(false));
        assert_eq!(quiet_value, value);
        assert_eq!(quiet, [("n".to_owned(), Misreading::WrongType)]);
        // The strict reading of the six families that leave the told list unread: with the
        // null structs left out it holds nothing.
        let strict_text = r#"{"inner": null, "list": [null]}"#;
        let strict = |rules: ScalarRules| {
            parse_with_rules::<Nested>(JsonSlice::new(strict_text), None, rules)
                .unwrap()
                .told
                .len()
        };
        assert_eq!(strict(STRICT), 2);
        assert_eq!(strict(STRICT.with_tells_null_struct(false)), 0);
    }

    #[test]
    fn under_null_absent_the_value_comes_back_without_its_absent_members() {
        let text = r#"{"n": null, "s": "a", "maybe": null, "list": [1]}"#;
        let read = parse_with_rules::<Plain>(JsonSlice::new(text), Some(NullRule::Absent), STRICT)
            .unwrap();
        assert_eq!(read.value.n, 0);
        assert_eq!(read.json, Some(value(r#"{"s": "a", "list": [1]}"#)));
        let absent: Vec<String> = read.absent.iter().map(ToString::to_string).collect();
        assert_eq!(absent, ["n", "maybe"]);
        // Under another rule the value is the text's, `null` members and all.
        let read =
            parse_with_rules::<Plain>(JsonSlice::new(r#"{"maybe": null}"#), None, STRICT).unwrap();
        assert_eq!(read.json, Some(value(r#"{"maybe": null}"#)));
        assert!(read.absent.is_empty());
    }

    /// A struct with a flattened one, as the envelope shell has.
    #[derive(Debug, Facet)]
    struct WithFlattened {
        #[facet(default)]
        own: u8,
        #[facet(flatten)]
        inner: Flat,
    }

    #[derive(Debug, Facet)]
    struct Flat {
        #[facet(default)]
        n: u8,
        #[facet(default)]
        on: bool,
    }

    /// The pre-pass does not walk a flattened struct: under the strict rule its plain scalars
    /// are facet's, read and converted as written, and nothing is told of them. Pinned as it
    /// is today (a slice whose struct flattens one, entity's conversion among them, sees it).
    #[test]
    fn a_flattened_structs_scalars_are_not_judged() {
        let text = r#"{"own": 1, "n": "7", "on": true}"#;
        let read = parse_with_rules::<WithFlattened>(JsonSlice::new(text), None, STRICT)
            .expect("the flattened string is read");
        assert_eq!(read.value.inner.n, 7);
        assert!(read.told.is_empty(), "{:?}", read.told);
        // The struct's own member is judged.
        let own = r#"{"own": "1", "n": 7}"#;
        assert!(matches!(
            parse_with_rules::<WithFlattened>(JsonSlice::new(own), None, STRICT),
            Err(ScalarFailure::Read(error)) if error.relative_path().to_string() == "own"
        ));
    }

    /// An untagged enum whose object arms share a key of different scalar types.
    #[derive(Debug, PartialEq, Facet)]
    #[facet(untagged)]
    #[repr(u8)]
    enum SharedKey {
        Count(Counted),
        Flag(Flagged),
    }

    #[derive(Debug, PartialEq, Facet)]
    struct Counted {
        k: u8,
    }

    #[derive(Debug, PartialEq, Facet)]
    struct Flagged {
        k: bool,
    }

    /// The token a refused scalar is handed facet as (`false` for a member that is no bool, `2`
    /// for a bool) takes part in facet's narrowing of the arms by the key's scalar type: where
    /// another arm's field of the key takes the poison, facet reads that arm. Pinned as it is
    /// today.
    #[test]
    fn a_poisoned_scalar_steers_the_arms_that_share_its_key() {
        let read = |text: &str| {
            parse_with_rules::<SharedKey>(JsonSlice::new(text), None, STRICT).map(|read| read.value)
        };
        // As written, each arm takes its own type.
        assert_eq!(
            read(r#"{"k": 7}"#).unwrap(),
            SharedKey::Count(Counted { k: 7 })
        );
        assert_eq!(
            read(r#"{"k": true}"#).unwrap(),
            SharedKey::Flag(Flagged { k: true })
        );
        // `"7"` is walked through the first arm, refused there, and handed facet as `false`.
        let poisoned = read(r#"{"k": "7"}"#);
        assert!(
            matches!(poisoned, Ok(SharedKey::Flag(Flagged { k: false }))),
            "{poisoned:?}"
        );
    }

    #[test]
    fn the_plain_reading_reads_as_facet_and_tells_as_the_misread_walk() {
        let text = r#"{"n": "7", "x": 1e39, "s": 5, "list": [1e1]}"#;
        let read =
            parse_with_rules::<Plain>(JsonSlice::new(text), None, ScalarRules::PLAIN).unwrap();
        assert_eq!(read.value.n, 7);
        assert_eq!(read.value.s, "5");
        assert_eq!(read.value.list, [10]);
        assert!(read.value.x.is_infinite());
        let told: Vec<(String, Misreading)> = read
            .told
            .iter()
            .map(|t| (t.path.to_string(), t.kind))
            .collect();
        assert_eq!(
            told,
            [
                ("n".to_owned(), Misreading::WrongType),
                ("x".to_owned(), Misreading::BeyondFloat),
                ("s".to_owned(), Misreading::WrongType),
            ]
        );
    }
}
