//! An integer token too large for 64 bits reads as the double it spells, through every entry
//! point of the text reader and into every typed member: never as a string, never as the end of
//! the file. Vanilla's readers (rapidjson, jsoncpp, the Minecraft Document reader) read such a
//! token as a double; facet-json 0.50.0-rc.7 gave an untyped `Value` the digits as a string
//! up to 128 bits and refused the token (and so the file) beyond.

use bedrock_addon_core::diagnostic::codes;
use std::borrow::Cow;

use bedrock_addon_core::diagnostic::{Code, Diagnostics, JsonPath, Severity, Span};
use bedrock_addon_core::json::kit::{Node, Profile, Read};
use bedrock_addon_core::json::parse::{Options, members_of, parse, parse_bytes, parse_slice};
use bedrock_addon_core::json::{
    EnvelopeVersion, JsonSlice, Value, from_str, from_str_jsonc, last_wins, probe_envelope, to_json,
};
use bedrock_addon_core::schema::{FromJson, Schema, read_document};
use bedrock_addon_core::versioning::Reading;
use facet::Facet;

/// What a plain typed member reads by: facet's own typed read (scalars converted and told, a
/// number beyond its float an infinity told, a `null` the member's own), stopping at the first
/// failure.
const PLAIN: Reading = Reading::OWN;

/// The strict reading the strict wrappers stood for.
const STRICT: Reading = Reading::OWN.strict();

/// [`Typed`]'s reading: its `flag` was a strict wrapper and its floats plain members, which
/// read a number beyond their width as an infinity: strict scalars, a float beyond its width
/// clamped and told.
const STRICT_CLAMPED: Reading =
    Reading::OWN.with_scalars(bedrock_addon_core::versioning::Scalars::Strict);

/// `text` read as `T` under `reading`: the value, or the first error's code and path.
fn read_as<T: FromJson>(text: &str, reading: Reading) -> Result<T, (Code, JsonPath)> {
    let parsed = parse(text, Options::default()).expect("JSON");
    let mut d = Diagnostics::default();
    let value = {
        let mut read = Read::new(reading, &mut d).profile(Profile::FIRST_FAILURE);
        read_document::<T>(&Node::parsed(&parsed, JsonPath::new()), &mut read)
    };
    let error = d.iter().find(|f| f.severity >= Severity::Error);
    match (value, error) {
        // A value the read built is the read's: a converted scalar is told and kept.
        (Some(value), _) => Ok(value),
        (None, Some(error)) => Err((error.code.clone(), error.location.json_path.clone())),
        (None, None) => panic!("`{text}` failed and told nothing: {d:?}"),
    }
}

/// The first error the plain read of `[token]` as `T` tells, value or not: what the door's
/// passes told of a member that read the token as another value.
fn refused<T: FromJson>(token: &str) -> Option<(Code, JsonPath)> {
    let text = format!("[{token}]");
    let parsed = parse(&text, Options::default()).expect("JSON");
    let mut d = Diagnostics::default();
    {
        let mut read = Read::new(PLAIN, &mut d).profile(Profile::FIRST_FAILURE);
        let _ = read_document::<Vec<T>>(&Node::parsed(&parsed, JsonPath::new()), &mut read);
    }
    d.iter()
        .find(|f| f.severity >= Severity::Error)
        .map(|error| (error.code.clone(), error.location.json_path.clone()))
}

/// A string or a number, the string arm first: what a member that takes either is read as.
#[derive(Debug, PartialEq, Facet, Schema)]
#[facet(untagged)]
#[repr(u8)]
enum StringOrNumber {
    Text(String),
    Number(f64),
}

/// Whether the token is an integer that fits neither `i64` nor `u64`: this test's own rule.
fn is_wide(token: &str) -> bool {
    let digits = token.strip_prefix('-').unwrap_or(token);
    !digits.is_empty()
        && digits.bytes().all(|b| b.is_ascii_digit())
        && token.parse::<i64>().is_err()
        && token.parse::<u64>().is_err()
}

/// The double a wide integer spells, and the real token that spells it: the shortest decimal
/// that reads back to it, `1e999` beyond the range of a double.
fn real_of(token: &str) -> (f64, String) {
    let value: f64 = token.parse().unwrap();
    let text = if value.is_infinite() {
        (if value < 0.0 { "-1e999" } else { "1e999" }).to_owned()
    } else {
        format!("{value:?}")
    };
    assert!(text.contains(['.', 'e']), "`{text}` is a real token");
    (value, text)
}

fn decimal(digits: usize) -> String {
    "1234567890".chars().cycle().take(digits).collect()
}

/// The boundary table: each token and whether it is wide.
fn boundaries() -> Vec<(String, bool)> {
    let mut table: Vec<(String, bool)> = vec![
        (i64::MIN.to_string(), false),
        ("-9223372036854775809".into(), true), // i64::MIN - 1
        (i64::MAX.to_string(), false),
        (u64::MAX.to_string(), false),
        ("18446744073709551616".into(), true), // u64::MAX + 1
        (format!("-{}", u64::MAX), true),
        (i128::MAX.to_string(), true),
        ("170141183460469231731687303715884105728".into(), true), // i128::MAX + 1
        (i128::MIN.to_string(), true),
        ("-170141183460469231731687303715884105729".into(), true), // i128::MIN - 1
        (u128::MAX.to_string(), true),
        ("340282366920938463463374607431768211456".into(), true), // u128::MAX + 1
        ("-340282366920938463463374607431768211456".into(), true),
        // The doubles that round up to a power of ten, and the ties of the rounding.
        ("99999999999999999999".into(), true),
        ("18446744073709552000".into(), true),
        ("18446744073709553665".into(), true),
        ("-0".into(), false),
        ("0".into(), false),
        ("007".into(), false),
        (format!("{}7", "0".repeat(30)), false),
        (format!("-{}7", "0".repeat(30)), false),
        (format!("{}18446744073709551615", "0".repeat(5)), false),
        (format!("{}18446744073709551616", "0".repeat(5)), true),
        (format!("-{}9223372036854775809", "0".repeat(400)), true),
    ];
    for digits in [40, 100, 308, 309, 310, 400, 5000] {
        table.push((decimal(digits), true));
        table.push((format!("-{}", decimal(digits)), true));
        table.push(("9".repeat(digits), true));
    }
    for (token, wide) in &table {
        assert_eq!(is_wide(token), *wide, "`{token}`");
    }
    table
}

/// The one element of `[token]` read as `T`, by the plain reading.
fn element<T: FromJson>(token: &str) -> Result<T, (Code, JsonPath)> {
    strict_or_not::<T>(token, PLAIN)
}

/// The one element of `[token]` read as `T` by the strict reading.
fn strict<T: FromJson>(token: &str) -> Result<T, (Code, JsonPath)> {
    strict_or_not::<T>(token, STRICT)
}

fn strict_or_not<T: FromJson>(token: &str, reading: Reading) -> Result<T, (Code, JsonPath)> {
    read_as::<Vec<T>>(&format!("[{token}]"), reading).map(|mut list| {
        assert_eq!(list.len(), 1);
        list.remove(0)
    })
}

/// What `token` reads as, as each type, in one comparable text.
fn readings(token: &str) -> Vec<String> {
    fn show<T: FromJson + std::fmt::Debug>(token: &str, reading: Reading) -> String {
        match strict_or_not::<T>(token, reading) {
            Ok(value) => format!("{value:?}"),
            Err((code, at)) => format!("{code:?} at {at}"),
        }
    }
    vec![
        show::<f64>(token, PLAIN),
        show::<f32>(token, PLAIN),
        show::<i64>(token, PLAIN),
        show::<u64>(token, PLAIN),
        show::<i32>(token, PLAIN),
        show::<u8>(token, PLAIN),
        show::<bool>(token, PLAIN),
        show::<String>(token, PLAIN),
        show::<Option<f64>>(token, PLAIN),
        show::<StringOrNumber>(token, PLAIN),
        show::<f64>(token, STRICT),
        show::<f32>(token, STRICT),
        show::<i64>(token, STRICT),
        show::<u64>(token, STRICT),
        show::<String>(token, STRICT),
    ]
}

#[test]
fn a_wide_integer_is_a_real_number_in_an_untyped_value() {
    for (token, wide) in boundaries() {
        let value: Value = element(&token).unwrap_or_else(|e| panic!("`{token}`: {e:?}"));
        assert!(value.as_string().is_none(), "`{token}` read as a string");
        let number = value
            .as_number()
            .unwrap_or_else(|| panic!("`{token}` is a number"));
        if wide {
            let (spelt, _) = real_of(&token);
            assert!(number.is_float(), "`{token}` is a real");
            assert_eq!(
                number.to_f64_lossy().to_bits(),
                spelt.to_bits(),
                "`{token}`"
            );
        } else {
            assert!(number.is_integer(), "`{token}` stays an integer");
            let exact = token.parse::<i128>().unwrap();
            assert_eq!(number.to_i128(), Some(exact), "`{token}`");
        }
    }
}

#[test]
fn a_wide_integer_reads_as_its_double_into_float_members() {
    for (token, wide) in boundaries() {
        if !wide {
            continue;
        }
        let (spelt, _) = real_of(&token);
        let double: f64 = element(&token).unwrap_or_else(|e| panic!("`{token}`: {e:?}"));
        assert_eq!(double.to_bits(), spelt.to_bits(), "`{token}`");
        // An `f32` member takes the double, narrowed: the way vanilla narrows it.
        #[allow(clippy::cast_possible_truncation)]
        let narrowed = spelt as f32;
        let float: f32 = element(&token).unwrap();
        assert_eq!(float.to_bits(), narrowed.to_bits(), "`{token}`");
        let option: Option<f64> = element(&token).unwrap();
        assert_eq!(option.map(f64::to_bits), Some(spelt.to_bits()), "`{token}`");
    }
}

/// An integer member refuses a wide integer where it stands, as it refuses the real number of
/// that size: a type error with a path, not a syntax error of the file. But for the one double
/// a 64-bit member holds exactly: `-9223372036854775809` is the double -2^63, `i64::MIN`, which
/// jsoncpp's `asInt64` takes (`InRange` is inclusive) and the reader reads; facet-json's typed
/// read refused it.
#[test]
fn a_wide_integer_in_an_integer_member_is_a_type_error_at_the_member() {
    for (token, wide) in boundaries() {
        if !wide {
            continue;
        }
        #[allow(
            clippy::cast_precision_loss,
            clippy::float_cmp,
            reason = "the double -2^63 is exact, and the test asks for it exactly"
        )]
        let exact_min = real_of(&token).0 == i64::MIN as f64;
        if exact_min {
            assert_eq!(refused::<i64>(&token), None, "`{token}`");
            assert_eq!(element::<i64>(&token), Ok(i64::MIN), "`{token}`");
        }
        let i64_refusal = if exact_min {
            strict::<i64>(&token).unwrap_err()
        } else {
            refused::<i64>(&token).unwrap_or_else(|| panic!("`{token}` into i64"))
        };
        for (code, at) in [
            i64_refusal,
            refused::<u64>(&token).unwrap_or_else(|| panic!("`{token}` into u64")),
            refused::<i32>(&token).unwrap_or_else(|| panic!("`{token}` into i32")),
            refused::<u8>(&token).unwrap_or_else(|| panic!("`{token}` into u8")),
            strict::<i64>(&token).unwrap_err(),
            strict::<u64>(&token).unwrap_err(),
            refused::<bool>(&token).unwrap_or_else(|| panic!("`{token}` into bool")),
        ] {
            assert!(!at.is_empty(), "`{token}`: {code:?} at the list");
            assert_eq!(
                code,
                Code::from(&codes::INVALID_TYPE),
                "`{token}`: {code:?} at {at}"
            );
        }
    }
}

/// A string is a string only when its token is one.
#[test]
fn a_wide_integer_is_never_a_string() {
    for (token, wide) in boundaries() {
        if !wide {
            continue;
        }
        let (spelt, _) = real_of(&token);
        let (code, _) = strict::<String>(&token).unwrap_err();
        assert_eq!(code, Code::from(&codes::INVALID_TYPE));
        let either: StringOrNumber = element(&token).unwrap();
        assert_eq!(either, StringOrNumber::Number(spelt), "`{token}`");
        if spelt.is_finite() {
            let strict: f64 = strict(&token).unwrap();
            assert_eq!(strict.to_bits(), spelt.to_bits());
        }
        // The same digits in quotes are the string.
        let quoted = format!("\"{token}\"");
        assert_eq!(strict::<String>(&quoted).unwrap(), token);
        assert_eq!(
            element::<StringOrNumber>(&quoted).unwrap(),
            StringOrNumber::Text(token.clone())
        );
        let value: Value = element(&quoted).unwrap();
        assert_eq!(
            value.as_string().map(facet_value::VString::as_str),
            Some(token.as_str())
        );
    }
}

/// The rule in one line: a wide integer reads, as every type, exactly as the real token that
/// spells its double reads; beyond the range of a double that is `1e999`, read as before.
#[test]
fn a_wide_integer_reads_as_every_type_as_its_real_spelling_does() {
    for (token, wide) in boundaries() {
        if !wide {
            continue;
        }
        let (_, real) = real_of(&token);
        assert_eq!(readings(&token), readings(&real), "`{token}` and `{real}`");
        let wide_value: Value = element(&token).unwrap();
        let real_value: Value = element(&real).unwrap();
        assert_eq!(wide_value, real_value, "`{token}` and `{real}`");
    }
    // What `1e999` is: infinity in a float, refused by the strict float.
    let huge = "9".repeat(400);
    assert_eq!(element::<f64>(&huge).unwrap(), f64::INFINITY);
    assert_eq!(element::<f64>("1e999").unwrap(), f64::INFINITY);
    assert_eq!(
        element::<f32>(&format!("-{huge}")).unwrap(),
        f32::NEG_INFINITY
    );
    assert!(strict::<f64>(&huge).is_err());
    assert!(strict::<f64>("1e999").is_err());
}

/// An integer that fits 64 bits reads as it did, leading zeros and `-0` included.
#[test]
fn an_integer_within_64_bits_reads_as_before() {
    for (token, wide) in boundaries() {
        if wide {
            continue;
        }
        let exact = token.parse::<i128>().unwrap();
        assert_eq!(
            element::<i64>(&token).ok().map(i128::from),
            i64::try_from(exact).ok().map(i128::from),
            "`{token}`"
        );
        assert_eq!(
            element::<u64>(&token).ok().map(i128::from),
            u64::try_from(exact).ok().map(i128::from),
            "`{token}`"
        );
        // Nothing in the text is rewritten: the value is the integer.
        let text = format!("[{token}]");
        let exact_value = i64::try_from(exact)
            .map_or_else(|_| Value::from(u64::try_from(exact).unwrap()), Value::from);
        assert_eq!(
            from_str_jsonc::<Value>(&text).unwrap(),
            Value::from(facet_value::VArray::from_iter([exact_value])),
            "`{token}`"
        );
        assert_eq!(
            element::<String>(&token).unwrap(),
            exact.to_string(),
            "`{token}`"
        );
    }
}

/// Digits in keys, strings and comments are text, and a number with a fraction or an
/// exponent is a real already: none of them is touched.
#[test]
fn digit_runs_outside_integer_tokens_are_untouched() {
    let run = decimal(60);
    let text = format!(
        "// {run}\n{{ /* {run} */ \"{run}\": \"{run}\", \"-{run}\": [\"a {run} b\", \
         \"\\\"{run}\"], // {run}\n \"real\": [{run}.5, {run}e0, {run}E+2, -{run}.0e-3, \
         1e999, 0.{run}, 1e{run}] }} // {run}"
    );
    let door: Value = from_str_jsonc(&text).unwrap();
    let object = door.as_object().unwrap();
    assert_eq!(
        object
            .get(run.as_str())
            .unwrap()
            .as_string()
            .unwrap()
            .as_str(),
        run
    );
    let list = object.get(format!("-{run}").as_str()).unwrap();
    let list = list.as_array().unwrap();
    assert_eq!(list[0].as_string().unwrap().as_str(), format!("a {run} b"));
    assert_eq!(list[1].as_string().unwrap().as_str(), format!("\"{run}"));
    // The same text as the slices of its members: the key and the strings are the file's.
    let members = members_of(JsonSlice::new(&text)).unwrap();
    let (key, raw) = members.get_index(0).unwrap();
    assert_eq!(key, &run);
    assert_eq!(raw.text, format!("\"{run}\""));
    let real = object.get("real").unwrap().as_array().unwrap();
    assert!(real.iter().all(|number| {
        number
            .as_number()
            .is_some_and(facet_value::VNumber::is_float)
    }));
}

// --- every entry point ----------------------------------------------------------------------

/// A document with wide integers of every size class, as values at several depths.
fn wide_document() -> (String, [f64; 4]) {
    let tokens = [
        "18446744073709551616".to_owned(),
        "-9223372036854775809".to_owned(),
        decimal(40),
        format!("-{}", decimal(100)),
    ];
    let text = format!(
        "{{\"a\": {}, \"b\": [{}, {{\"c\": {}}}], \"d\": {}, \"e\": \"kept\"}}",
        tokens[0], tokens[1], tokens[2], tokens[3]
    );
    (text, tokens.map(|token| real_of(&token).0))
}

/// The four wide integers of [`wide_document`] as `value` holds them.
fn four_of(value: &Value) -> [f64; 4] {
    let real = |value: &Value| {
        let number = value.as_number().expect("a number");
        assert!(number.is_float());
        number.to_f64_lossy()
    };
    let object = value.as_object().unwrap();
    let b = object.get("b").unwrap().as_array().unwrap();
    assert_eq!(
        object.get("e").unwrap().as_string().unwrap().as_str(),
        "kept"
    );
    [
        real(object.get("a").unwrap()),
        real(&b[0]),
        real(b[1].as_object().unwrap().get("c").unwrap()),
        real(object.get("d").unwrap()),
    ]
}

#[test]
fn every_entry_point_reads_a_wide_integer_as_its_double() {
    let (text, expected) = wide_document();
    assert_eq!(four_of(&from_str::<Value>(&text).unwrap()), expected);
    assert_eq!(four_of(&from_str_jsonc::<Value>(&text).unwrap()), expected);
    assert_eq!(
        four_of(&parse(&text, Options::default()).unwrap().value),
        expected
    );
    assert_eq!(
        four_of(&parse_bytes(text.as_bytes(), Options::default()).unwrap()),
        expected
    );
    assert_eq!(
        four_of(
            &parse_slice(JsonSlice::new(&text), Options::default())
                .unwrap()
                .value
        ),
        expected
    );
    // Bytes whose comment is not UTF-8.
    let mut bytes = b"/* \xff */ ".to_vec();
    bytes.extend_from_slice(text.as_bytes());
    assert_eq!(
        four_of(&parse_bytes(&bytes, Options::default()).unwrap()),
        expected
    );
    // With comments, and content after the root that is never read.
    let commented = format!(
        "// {}\n{text} /* {} */ {}",
        decimal(50),
        decimal(50),
        decimal(50)
    );
    assert_eq!(
        four_of(&from_str_jsonc::<Value>(&commented).unwrap()),
        expected
    );
    // Plain JSON has no comments, with a wide integer as without one.
    assert!(from_str::<Value>(&commented).is_err());
    assert!(from_str::<Value>(&format!("/* c */ {text}")).is_err());
    assert!(from_str::<Value>("/* c */ [1]").is_err());
    let plain = from_str::<Value>(&format!("{text} tail")).unwrap();
    assert_eq!(four_of(&plain), expected);
    // The duplicate cleaner leaves the tokens alone, and its text reads the same.
    let doubled = text.replacen("{\"a\"", "{\"a\": 1, \"a\"", 1);
    let cleaned = last_wins(&doubled);
    assert!(matches!(cleaned, Cow::Owned(_)));
    assert!(cleaned.contains("18446744073709551616"));
    assert_eq!(
        four_of(&from_str_jsonc::<Value>(&cleaned).unwrap()),
        expected
    );
}

/// An integer wider than 128 bits anywhere in a file does not end the envelope probe.
#[test]
fn the_envelope_probe_reads_a_file_with_an_integer_of_any_size() {
    for digits in [20, 39, 40, 100, 400, 5000] {
        let big = "9".repeat(digits);
        let text = format!(
            "{{\"format_version\": \"1.21.40\", \"minecraft:x\": {{\"a\": {big}, \"b\": \
             [-{big}]}}, \"other\": {big}}}"
        );
        let envelope = probe_envelope(JsonSlice::new(&text))
            .unwrap_or_else(|error| panic!("{digits} digits: {error}"));
        assert_eq!(
            envelope.format_version,
            EnvelopeVersion::String("1.21.40".into())
        );
        assert_eq!(envelope.member_count(), 3);
        let payload = envelope.payload("minecraft:x").unwrap();
        assert_eq!(payload.text, format!("{{\"a\": {big}, \"b\": [-{big}]}}"));
        assert_eq!(
            &text[payload.offset..payload.offset + payload.text.len()],
            payload.text
        );
        assert_eq!(envelope.payload("other").unwrap().text, big);
        // A version that is such a number is malformed, as any number is; the file is read.
        let text = format!("{{\"format_version\": {big}, \"minecraft:x\": {{}}}}");
        let envelope = probe_envelope(JsonSlice::new(&text)).unwrap();
        assert_eq!(envelope.format_version, EnvelopeVersion::Malformed);
        let text = format!("{{\"format_version\": [1, {big}, 0], \"minecraft:x\": {{}}}}");
        let envelope = probe_envelope(JsonSlice::new(&text)).unwrap();
        assert_eq!(envelope.format_version, EnvelopeVersion::Malformed);
    }
}

// --- spans ----------------------------------------------------------------------------------

/// A typed target with a member that fails.
#[derive(Debug, PartialEq, Facet, Schema)]
struct Typed {
    first: Option<f64>,
    second: Option<f32>,
    #[facet(default)]
    list: Vec<f64>,
    flag: Option<i32>,
    #[facet(default)]
    ints: Vec<i32>,
    inner: Option<Box<Typed>>,
}

/// A finding after wide integers carries the span it has in the author's text, on the same
/// line and on later lines: the place the same document with short real numbers is faulted
/// at, found again in the author's text.
#[test]
fn a_finding_after_wide_integers_keeps_its_span() {
    let wide = [
        decimal(40),
        format!("-{}", decimal(77)),
        "18446744073709551616".to_owned(),
    ];
    let short = ["1.5", "-2.5", "3.5"];
    // The template with its numbers filled in, and where each of its characters starts.
    let fill = |template: &str, numbers: [&str; 3]| {
        let (mut text, mut starts) = (String::new(), Vec::new());
        for character in template.chars() {
            starts.push(text.len());
            match character {
                'A' => text.push_str(numbers[0]),
                'B' => text.push_str(numbers[1]),
                'C' => text.push_str(numbers[2]),
                other => text.push(other),
            }
        }
        (text, starts)
    };
    let mut on_the_fault = 0;
    for template in [
        r#"{"first": A, "second": B, "flag": "not a number"}"#,
        "{\"first\": A,\n \"second\": B,\n\n \"flag\": \"not a number\"\n}",
        r#"{"list": [A, B, C], "flag": "not a number"}"#,
        "{\"first\": A, /* A */ \"inner\": {\"second\": B,\n\"flag\": \"not a number\"}}",
        r#"{"first": C, "second": B, "flag": true}"#,
        r#"{"first": A, "second": B, "list": [C, A, "x", B]}"#,
        "{\"first\": A, \"second\": B,\n\"list\": [C, A], \"inner\": {\"first\": [B]}}",
        r#"{"first": A, "list": {"k": B}, "second": C}"#,
        // The wide integer is itself the fault.
        r#"{"first": A, "flag": B, "second": 1}"#,
        r#"{"first": A, "inner": {"flag": C}, "second": B}"#,
        r#"{"first": A, "ints": [1, 2, B, 3], "second": C}"#,
        "{\"first\": C,\n\"inner\": {\"ints\": [1,\n A]}}",
    ] {
        let (text, starts) = fill(template, [&wide[0], &wide[1], &wide[2]]);
        let (reference, reference_starts) = fill(template, short);
        let crate_error = failure(JsonSlice::new(&text));
        let reference_error = failure(JsonSlice::new(&reference));
        assert_eq!(crate_error.path, reference_error.path, "`{text}`");
        assert_eq!(crate_error.code, reference_error.code, "`{text}`");
        let part = |text: &str, span: Span| {
            text[span.offset as usize..(span.offset + span.len) as usize].to_owned()
        };
        // The faulted text of the reference, with the author's numbers in it.
        let mut faulted = part(&reference, reference_error.span);
        for (short, wide) in short.iter().zip(&wide) {
            faulted = faulted.replace(short, wide);
        }
        assert_eq!(
            part(&text, crate_error.span),
            faulted,
            "`{text}`: {crate_error:?}"
        );
        // The same character of the template starts both.
        let character = reference_starts
            .iter()
            .position(|start| *start == reference_error.span.offset as usize)
            .expect("the reference's span starts a character of the template");
        assert_eq!(
            crate_error.span.offset as usize, starts[character],
            "`{text}`"
        );
        let first_wide = wide.iter().filter_map(|token| text.find(token)).min();
        assert!(
            crate_error.span.offset as usize > first_wide.unwrap(),
            "`{text}`"
        );
        on_the_fault += usize::from(wide.contains(&faulted));
        // At a file offset, the slice's offset is added.
        let moved = JsonSlice {
            text: &text,
            offset: 1000,
        };
        let error = failure(moved);
        assert_eq!(error.span.offset, 1000 + crate_error.span.offset);
    }
    assert!(on_the_fault >= 1, "a span on the digits of a wide integer");
    // A syntax error after wide integers is at its byte.
    let text = format!(
        "{{\"first\": {}, \"second\": {},\n \"flag\" 1}}",
        wide[0], wide[1]
    );
    let error = parse(&text, Options::default()).unwrap_err();
    assert_eq!(error.span.offset as usize, text.find(" 1}").unwrap() + 1);
}

/// The first error the strict reading of `slice` as [`Typed`] tells: its code, path and span.
#[derive(Debug)]
struct Failure {
    code: Code,
    path: JsonPath,
    span: Span,
}

fn failure(slice: JsonSlice<'_>) -> Failure {
    let parsed = parse_slice(slice, Options::default()).expect("JSON");
    let mut d = Diagnostics::default();
    let value = {
        let mut read = Read::new(STRICT_CLAMPED, &mut d).profile(Profile::FIRST_FAILURE);
        read_document::<Typed>(&Node::parsed(&parsed, JsonPath::new()), &mut read)
    };
    assert!(value.is_none(), "`{}` read", slice.text);
    let error = d
        .iter()
        .find(|f| f.severity >= Severity::Error)
        .expect("an error");
    Failure {
        code: error.code.clone(),
        path: error.location.json_path.clone(),
        span: error.location.span.expect("a span"),
    }
}

// --- write-back -----------------------------------------------------------------------------

/// A wide integer is written as the shortest decimal that reads back to its double (the one
/// write-back difference of the class: the author's digits are respelt); the written text
/// reads to an equal value and writes to the same text.
#[test]
fn a_wide_integer_is_written_as_the_shortest_decimal_of_its_double() {
    #[derive(Debug, PartialEq, Facet, Schema)]
    struct Volume {
        volume: f32,
        gain: f64,
    }
    let text = "{\"sound\": 18446744073709551616, \"list\": [123456789012345678901234567890, \
                -9223372036854775809, 18446744073709551615, 7], \"text\": \
                \"18446744073709551616\"}";
    let value: Value = from_str_jsonc(text).unwrap();
    let written = to_json(&value).unwrap();
    for spelling in [
        "\"sound\": 1.8446744073709552e19",
        "1.2345678901234568e29",
        "-9.223372036854776e18",
        "18446744073709551615,",
        "\"text\": \"18446744073709551616\"",
    ] {
        assert!(written.contains(spelling), "`{spelling}` in {written}");
    }
    assert!(!written.contains("\"1234"), "{written}");
    let again: Value = from_str_jsonc(&written).unwrap();
    assert_eq!(again, value);
    assert_eq!(to_json(&again).unwrap(), written);

    // A typed `f32` member writes the shortest decimal of the `f32`.
    let text = format!(
        "{{\"volume\": 18446744073709551616, \"gain\": {}}}",
        decimal(40)
    );
    let typed: Volume = read_as(&text, PLAIN).unwrap();
    let written = to_json(&typed).unwrap();
    assert!(written.contains("\"volume\": 1.8446744e19"), "{written}");
    assert!(
        written.contains("\"gain\": 1.2345678901234568e39"),
        "{written}"
    );
    let again: Volume = read_as(&written, PLAIN).unwrap();
    assert_eq!(again, typed);
    assert_eq!(to_json(&again).unwrap(), written);
    // Beyond the range of a double: infinity, which has no JSON form, as `1e999` has none.
    let value: Value = from_str_jsonc(&format!("[{}]", decimal(400))).unwrap();
    assert!(to_json(&value).is_err());
    assert!(to_json(&from_str_jsonc::<Value>("[1e999]").unwrap()).is_err());
}
