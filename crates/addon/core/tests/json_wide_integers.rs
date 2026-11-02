//! An integer token too large for 64 bits reads as the double it spells, through
//! every entry point of the door (`json/read.rs`): never as a string, never as the end of the
//! file. Vanilla's readers (rapidjson, jsoncpp, the Minecraft Document reader) read such a
//! token as a double; facet-json 0.50.0-rc.7 gives an untyped `Value` the digits as a string
//! up to 128 bits and refuses the token (and so the file) beyond.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::borrow::Cow;

use bedrock_addon_core::diagnostic::{DiagnosticCode, Span, code_for_deserialize_error_kind};
use bedrock_addon_core::json::{
    EnvelopeVersion, JsonSlice, StrictFloat, StrictInt, StrictString, Value, from_slice_jsonc,
    from_str, from_str_borrowed_jsonc, from_str_jsonc, last_wins, parse_at, probe_envelope,
    to_json,
};
use facet::Facet;
use facet_json::{DeserializeError, RawJson};
use indexmap::IndexMap;

/// A string or a number, the string arm first: what a member that takes either is read as.
#[derive(Debug, PartialEq, Facet)]
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

fn code(error: &DeserializeError) -> DiagnosticCode {
    code_for_deserialize_error_kind(&error.kind, error.path.is_some(), false)
}

/// The one element of `[token]` read as `T`.
fn element<T: Facet<'static>>(token: &str) -> Result<T, DeserializeError> {
    from_str_jsonc::<Vec<T>>(&format!("[{token}]")).map(|mut list| {
        assert_eq!(list.len(), 1);
        list.remove(0)
    })
}

/// What `token` reads as, as each type, in one comparable text.
fn readings(token: &str) -> Vec<String> {
    fn show<T: Facet<'static> + std::fmt::Debug>(token: &str) -> String {
        match element::<T>(token) {
            Ok(value) => format!("{value:?}"),
            Err(error) => format!("{:?} `{}`", code(&error), error.kind),
        }
    }
    vec![
        show::<f64>(token),
        show::<f32>(token),
        show::<i64>(token),
        show::<u64>(token),
        show::<i32>(token),
        show::<u8>(token),
        show::<bool>(token),
        show::<String>(token),
        show::<Option<f64>>(token),
        show::<StringOrNumber>(token),
        show::<StrictFloat<f64>>(token),
        show::<StrictFloat<f32>>(token),
        show::<StrictInt<i64>>(token),
        show::<StrictInt<u64>>(token),
        show::<StrictString>(token),
    ]
}

#[test]
fn a_wide_integer_is_a_real_number_in_an_untyped_value() {
    for (token, wide) in boundaries() {
        let value: Value = element(&token).unwrap_or_else(|e| panic!("`{token}`: {e}"));
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
        let double: f64 = element(&token).unwrap_or_else(|e| panic!("`{token}`: {e}"));
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
/// that size: a type error with a path, not a syntax error of the file.
#[test]
fn a_wide_integer_in_an_integer_member_is_a_type_error_at_the_member() {
    for (token, wide) in boundaries() {
        if !wide {
            continue;
        }
        for error in [
            element::<i64>(&token).unwrap_err(),
            element::<u64>(&token).unwrap_err(),
            element::<i32>(&token).unwrap_err(),
            element::<u8>(&token).unwrap_err(),
            element::<StrictInt<i64>>(&token).unwrap_err(),
            element::<StrictInt<u64>>(&token).unwrap_err(),
            element::<bool>(&token).unwrap_err(),
        ] {
            assert!(error.path.is_some(), "`{token}`: {error:?}");
            assert_eq!(
                code(&error),
                DiagnosticCode::InvalidType { text: None },
                "`{token}`: {error:?}"
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
        let error = element::<StrictString>(&token).unwrap_err();
        assert_eq!(code(&error), DiagnosticCode::InvalidType { text: None });
        let either: StringOrNumber = element(&token).unwrap();
        assert_eq!(either, StringOrNumber::Number(spelt), "`{token}`");
        if spelt.is_finite() {
            let strict: StrictFloat<f64> = element(&token).unwrap();
            assert_eq!(strict.0.to_bits(), spelt.to_bits());
        }
        // The same digits in quotes are the string.
        let quoted = format!("\"{token}\"");
        assert_eq!(element::<StrictString>(&quoted).unwrap().0, token);
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
#[expect(clippy::float_cmp, reason = "the reader gives exact values")]
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
    assert!(element::<StrictFloat<f64>>(&huge).is_err());
    assert!(element::<StrictFloat<f64>>("1e999").is_err());
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
        // facet's own reading of the text is the reference: nothing in it is rewritten.
        let text = format!("[{token}]");
        let facet: Value = facet_json::from_str_jsonc(&text).unwrap();
        assert_eq!(from_str_jsonc::<Value>(&text).unwrap(), facet, "`{token}`");
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
    let facet: Value = facet_json::from_str_jsonc(&text).unwrap();
    let door: Value = from_str_jsonc(&text).unwrap();
    assert_eq!(door, facet);
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
    // The same text read with borrowed strings: the key and the strings are the file's.
    let borrowed: IndexMap<Cow<'_, str>, RawJson<'_>> = from_str_borrowed_jsonc(&text).unwrap();
    let (key, raw) = borrowed.get_index(0).unwrap();
    assert_eq!(key, &run);
    assert_eq!(raw.as_str(), format!("\"{run}\""));
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
#[expect(clippy::float_cmp, reason = "the reader gives exact values")]
fn every_entry_point_reads_a_wide_integer_as_its_double() {
    let (text, expected) = wide_document();
    assert_eq!(four_of(&from_str::<Value>(&text).unwrap()), expected);
    assert_eq!(four_of(&from_str_jsonc::<Value>(&text).unwrap()), expected);
    assert_eq!(
        four_of(&from_str_borrowed_jsonc::<Value>(&text).unwrap()),
        expected
    );
    assert_eq!(
        four_of(&from_slice_jsonc::<Value>(text.as_bytes()).unwrap()),
        expected
    );
    assert_eq!(
        four_of(&parse_at::<Value>(JsonSlice::new(&text)).unwrap()),
        expected
    );
    // Bytes whose comment is not UTF-8.
    let mut bytes = b"/* \xff */ ".to_vec();
    bytes.extend_from_slice(text.as_bytes());
    assert_eq!(
        four_of(&from_slice_jsonc::<Value>(&bytes).unwrap()),
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

/// What a typed read borrows from a text with a wide integer is the caller's text: strings
/// and keys without escapes, and raw slices, which hold the author's digits and lie where
/// `JsonSlice::nested` looks for them.
#[test]
#[expect(clippy::float_cmp, reason = "the reader gives exact values")]
fn a_borrowed_read_of_a_text_with_a_wide_integer_borrows_from_that_text() {
    #[derive(Debug, Facet)]
    struct Shell<'a> {
        name: Cow<'a, str>,
        escaped: Cow<'a, str>,
        plain: &'a str,
        wide: f64,
        members: IndexMap<Cow<'a, str>, RawJson<'a>>,
    }
    let big = decimal(45);
    let text = format!(
        "{{\"wide\": {big}, \"name\": \"borrowed\", \"escaped\": \"a\\nb\", \"plain\": \"p\", \
         \"members\": {{\"first\": {{\"x\": [{big}, \"s\"]}}, \"second\": -{big}}}}}"
    );
    let shell: Shell<'_> = from_str_borrowed_jsonc(&text).unwrap();
    assert_eq!(shell.wide, real_of(&big).0);
    let within = |part: &str| {
        let start = part.as_ptr() as usize;
        let base = text.as_ptr() as usize;
        start >= base && start + part.len() <= base + text.len()
    };
    assert!(matches!(shell.name, Cow::Borrowed("borrowed")));
    assert!(within(&shell.name), "the string is a slice of the text");
    assert_eq!(shell.escaped, "a\nb");
    assert!(within(shell.plain));
    let document = JsonSlice::new(&text);
    for (key, raw) in &shell.members {
        assert!(matches!(key, Cow::Borrowed(_)) && within(key));
        let slice = document.nested(raw).expect("the raw slice is the text's");
        assert_eq!(
            &text[slice.offset..slice.offset + slice.text.len()],
            raw.as_str()
        );
    }
    assert_eq!(
        shell.members["first"].as_str(),
        format!("{{\"x\": [{big}, \"s\"]}}")
    );
    assert_eq!(shell.members["second"].as_str(), format!("-{big}"));
    // A slice is read through the door again.
    let first: Value = parse_at(document.nested(&shell.members["first"]).unwrap()).unwrap();
    let x = first
        .as_object()
        .unwrap()
        .get("x")
        .unwrap()
        .as_array()
        .unwrap();
    assert_eq!(x[0].as_number().unwrap().to_f64_lossy(), real_of(&big).0);
    assert_eq!(x[1].as_string().unwrap().as_str(), "s");
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
#[derive(Debug, Facet)]
struct Typed {
    #[facet(default)]
    first: Option<f64>,
    #[facet(default)]
    second: Option<f32>,
    #[facet(default)]
    list: Vec<f64>,
    #[facet(default)]
    flag: Option<StrictInt<i32>>,
    #[facet(default)]
    ints: Vec<i32>,
    #[facet(default)]
    inner: Option<Box<Typed>>,
}

/// A finding after wide integers carries the span it has in the author's text, on the same
/// line and on later lines: the place the same document with short real numbers is faulted
/// at (facet puts a strict member's fault on its key), found again in the author's text.
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
        let crate_error = parse_at::<Typed>(JsonSlice::new(&text)).unwrap_err();
        let reference_error = parse_at::<Typed>(JsonSlice::new(&reference)).unwrap_err();
        assert_eq!(crate_error.path, reference_error.path, "`{text}`");
        assert_eq!(
            code(&crate_error.error),
            code(&reference_error.error),
            "`{text}`"
        );
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
        let error = parse_at::<Typed>(moved).unwrap_err();
        assert_eq!(error.span.offset, 1000 + crate_error.span.offset);
    }
    assert!(on_the_fault >= 1, "a span on the digits of a wide integer");
    // A syntax error after wide integers is at its byte.
    let text = format!(
        "{{\"first\": {}, \"second\": {},\n \"flag\" 1}}",
        wide[0], wide[1]
    );
    let error = from_str_jsonc::<Typed>(&text).unwrap_err();
    assert_eq!(
        error.span.map(|span| span.offset as usize),
        Some(text.find(" 1}").unwrap() + 1)
    );
}

// --- write-back -----------------------------------------------------------------------------

/// A wide integer is written as the shortest decimal that reads back to its double (the one
/// write-back difference of the class: the author's digits are respelt); the written text
/// reads to an equal value and writes to the same text.
#[test]
fn a_wide_integer_is_written_as_the_shortest_decimal_of_its_double() {
    #[derive(Debug, PartialEq, Facet)]
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
    let typed: Volume = from_str_jsonc(&text).unwrap();
    let written = to_json(&typed).unwrap();
    assert!(written.contains("\"volume\": 1.8446744e19"), "{written}");
    assert!(
        written.contains("\"gain\": 1.2345678901234568e39"),
        "{written}"
    );
    let again: Volume = from_str_jsonc(&written).unwrap();
    assert_eq!(again, typed);
    assert_eq!(to_json(&again).unwrap(), written);
    // Beyond the range of a double: infinity, which has no JSON form, as `1e999` has none.
    let value: Value = from_str_jsonc(&format!("[{}]", decimal(400))).unwrap();
    assert!(to_json(&value).is_err());
    assert!(to_json(&from_str_jsonc::<Value>("[1e999]").unwrap()).is_err());
}
