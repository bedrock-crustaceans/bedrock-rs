//! The scalar judge: how a scalar token is read against its member's declared type under each
//! profile a slice reads by. Its oracles were the readers it stands for, now deleted with the
//! old reader: the strict wrappers (`Strict`, and `NonStrict` under the reader mode) and the
//! post-hoc misread walk of a plain member (`ConvertAndTell`); and the legacy block reader's
//! fixtures (`families/block/tests/block/legacy_reader.rs`).
//!
//! The old reader's side of the tables is frozen in `json_scalars.pins`, one line per cell:
//! `key<TAB>verdict<TAB>door`, the judge's verdict and what the old reader read (checked to agree
//! while it ran: a wrapper refused what the judge refuses and read the token as written
//! or as the judge respells it; the misread walk told what the judge says). The tests hold the
//! judge to the pinned verdict; `BEDROCK_DOOR_PINS=<dir>` writes each test's cells (the pinned
//! old read beside) to `<dir>/<test>` instead of checking them.

use bedrock_addon_core::json::{
    BeyondWidth, Declared, Refusal, Respelt, ScalarRules, Scalars, Token, Value, Verdict,
    from_str_jsonc, judge,
};
use std::collections::HashMap;
use std::sync::OnceLock;

/// The frozen cells: key to `(verdict, door)`.
fn frozen() -> &'static HashMap<&'static str, (&'static str, &'static str)> {
    static FROZEN: OnceLock<HashMap<&'static str, (&'static str, &'static str)>> = OnceLock::new();
    FROZEN.get_or_init(|| {
        include_str!("json_scalars.pins")
            .lines()
            .filter_map(|line| {
                let mut parts = line.split('\t');
                Some((parts.next()?, (parts.next()?, parts.next()?)))
            })
            .collect()
    })
}

/// The cell `key` of `test`: the judge's `verdict` must be the pinned one. Under
/// `BEDROCK_DOOR_PINS` the cell (with the pinned old read) is written to that directory
/// instead.
fn cell(test: &str, key: &str, verdict: &str) {
    let pinned = frozen().get(key).copied();
    if let Ok(dir) = std::env::var("BEDROCK_DOOR_PINS") {
        use std::io::Write as _;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(std::path::Path::new(&dir).join(test))
            .expect("the pin directory");
        let door = pinned.map_or("", |(_, door)| door);
        file.write_all(format!("{key}\t{verdict}\t{door}\n").as_bytes())
            .expect("a pin line");
        return;
    }
    let Some((pinned_verdict, _)) = pinned else {
        panic!("no pin for {key}");
    };
    assert_eq!(verdict, pinned_verdict, "the judge at {key}");
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
    from_str_jsonc(text).unwrap()
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

/// The declared types of the strict wrappers' table, by the wrapper each stood for.
const WRAPPED: &[Declared] = &[
    Declared::Int {
        bits: 8,
        signed: true,
    },
    Declared::Int {
        bits: 16,
        signed: true,
    },
    Declared::Int {
        bits: 32,
        signed: true,
    },
    Declared::Int {
        bits: 64,
        signed: true,
    },
    Declared::Int {
        bits: 8,
        signed: false,
    },
    Declared::Int {
        bits: 16,
        signed: false,
    },
    Declared::Int {
        bits: 32,
        signed: false,
    },
    Declared::Int {
        bits: 64,
        signed: false,
    },
    Declared::Float { bits: 32 },
    Declared::Float { bits: 64 },
    Declared::Bool,
    Declared::Text,
];

/// The judge under `rules` reads every token for every wrapped type as the wrapper under the
/// reader `mode` (`Strict`, `NonStrict`) read it: the pinned verdict.
fn agree(mode: &str, rules: ScalarRules) {
    for declared in WRAPPED {
        for text in TOKENS {
            let verdict = verdict(*declared, text, rules);
            let key = format!("wrapper {mode} {declared:?} {text}");
            cell("wrapper", &key, &format!("{verdict:?}"));
        }
    }
}

#[test]
fn strict_judges_every_token_as_the_strict_wrappers_read_it() {
    agree("Strict", STRICT);
}

#[test]
fn non_strict_judges_every_token_as_the_non_strict_reader_reads_it() {
    agree("NonStrict", NON_STRICT);
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
}

/// `families/block/tests/block/legacy_reader.rs`, the legacy block reader's fixtures, as respellings.
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

/// A plain member of `declared`, read by facet from every token it took, and the misread walk's
/// word on it (both pinned): the judge under `ConvertAndTell` says the same.
fn tells_as_the_misread_walk(declared: Declared) {
    for beyond_width in [BeyondWidth::ClampAndTell, BeyondWidth::Error] {
        let rules = rules(Scalars::ConvertAndTell, beyond_width);
        for text in TOKENS {
            let key = format!("misread {beyond_width:?} {declared:?} {text}");
            cell(
                "misread",
                &key,
                &format!("{:?}", verdict(declared, text, rules)),
            );
        }
    }
}

#[test]
fn convert_and_tell_tells_what_the_misread_walk_of_a_plain_member_tells() {
    for declared in [
        Declared::Int {
            bits: 8,
            signed: true,
        },
        Declared::Int {
            bits: 32,
            signed: true,
        },
        Declared::Int {
            bits: 8,
            signed: false,
        },
        Declared::Int {
            bits: 64,
            signed: false,
        },
        Declared::Float { bits: 32 },
        Declared::Float { bits: 64 },
        Declared::Bool,
        Declared::Text,
    ] {
        tells_as_the_misread_walk(declared);
    }
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
    // An `f64` holds every number the parser reads.
    assert_eq!(
        verdict(Declared::Float { bits: 64 }, "1e300", STRICT),
        Verdict::Keep
    );
}
