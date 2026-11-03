//! Reading Molang fields: the reader matrix and facet
//! cases of molang `tests/serialised_forms.rs`, copied as a table into
//! `tests/fixtures/molang/reader_matrix.json`, run three ways that must agree:
//!
//! 1. molang's `MolangValueRepr::read` on the `JsonScalar` the row gives;
//! 2. the crate's `molang::read` on the `facet_value::Value` facet-json parses from the row's text;
//! 3. facet-json deserialising the typed field from the text (`Molang`, or `Option<MolangList>`
//!    for the scalar-or-array reader), then pass 1 (reader check and context version).
//!
//! Way 2 differs from way 1 in one thing: a number beyond `f32` is the largest finite float, not
//! the infinity. The typed path is narrower where the crate says so: a number not finite as an
//! `f32` is rejected by facet under every reader, marked `"facet": "narrower"`.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::sync::Arc;

use bedrock_addon_core::diagnostic::{DiagnosticCode, code_for_deserialize_error_kind};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::molang::{
    self, JsonScalar, JsonType, Molang, MolangList, MolangSource, OtherJson, RawVersion, ReadField,
    ReaderKind, ReprError,
};

const READERS: [(&str, ReaderKind); 5] = [
    ("strict", ReaderKind::StrictVersioned),
    ("lenient", ReaderKind::LenientVersioned),
    ("biome", ReaderKind::BiomeHeightRange),
    ("list", ReaderKind::ScalarOrArray),
    ("schema", ReaderKind::SchemaValidated),
];

fn reader_matrix() -> Value {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/molang/reader_matrix.json"
    );
    let text = std::fs::read_to_string(path).expect("the reader matrix fixture");
    facet_json::from_str::<Value>(&text).expect("the fixture is JSON")
}

fn text(value: &Value) -> &str {
    value.as_string().expect("a string").as_str()
}

fn items(value: &Value) -> Vec<&Value> {
    value.as_array().expect("an array").iter().collect()
}

/// A number column: a JSON number, or one of the names JSON cannot hold.
fn number(value: &Value) -> f64 {
    if let Some(number) = value.as_number() {
        return number.to_f64_lossy();
    }
    match text(value) {
        "-0.0" => -0.0,
        "NaN" => f64::NAN,
        "inf" => f64::INFINITY,
        "-inf" => f64::NEG_INFINITY,
        "f32::MAX" => f64::from(f32::MAX),
        "f32::MIN" => f64::from(f32::MIN),
        other => panic!("not a number name: {other}"),
    }
}

fn raw(value: &Value) -> RawVersion {
    let raw = value
        .as_number()
        .and_then(facet_value::VNumber::to_i64)
        .expect("a version");
    RawVersion(i16::try_from(raw).expect("an i16 version"))
}

fn json_type(name: &str) -> JsonType {
    match name {
        "null" => JsonType::Null,
        "bool" => JsonType::Bool,
        "number" => JsonType::Number,
        "string" => JsonType::String,
        "array" => JsonType::Array,
        "object" => JsonType::Object,
        other => panic!("not a JSON type: {other}"),
    }
}

/// The row's `scalar` column: what a caller hands the crate's reader.
fn scalar(column: &Value) -> JsonScalar {
    let parts = items(column);
    match text(parts[0]) {
        "number" => JsonScalar::Number(number(parts[1])),
        "bool" => JsonScalar::Bool(parts[1].as_bool().expect("a bool")),
        "string" => JsonScalar::String(Arc::from(text(parts[1]))),
        "object" => JsonScalar::Object {
            expression: Arc::from(text(parts[1])),
            version: raw(parts[2]),
        },
        "null" => JsonScalar::Other(OtherJson::Null),
        "array" => JsonScalar::Other(OtherJson::Array),
        "other_object" => JsonScalar::Other(OtherJson::Object),
        other => panic!("not a scalar kind: {other}"),
    }
}

/// The expected result of one reader.
#[allow(clippy::cast_possible_truncation)]
fn expected(column: &Value, reader: ReaderKind) -> Result<ReadField, ReprError> {
    let parts = items(column);
    let value = |value: Molang| Ok(ReadField::Value(value));
    match text(parts[0]) {
        "const" => value(Molang::Const(number(parts[1]) as f32)),
        "bool" => value(Molang::Bool(parts[1].as_bool().expect("a bool"))),
        "string" => value(Molang::Expr(MolangSource::string(
            text(parts[1]),
            raw(parts[2]),
        ))),
        "object" => value(Molang::Expr(MolangSource::object(
            text(parts[1]),
            raw(parts[2]),
        ))),
        "no_expression" => Ok(ReadField::NoExpression),
        "expression_array" => Ok(ReadField::ExpressionArray),
        "unsupported" => Err(ReprError::UnsupportedType {
            reader,
            found: json_type(text(parts[1])),
        }),
        "out_of_range" => Err(ReprError::NumberOutOfRange { value: 0.0 }),
        other => panic!("not a result kind: {other}"),
    }
}

/// Equality with `f32` bits for constants (so `-0.0` and NaN compare) and any number for an
/// out-of-range error (the row does not repeat the input).
fn same(left: &Result<ReadField, ReprError>, right: &Result<ReadField, ReprError>) -> bool {
    match (left, right) {
        (Ok(ReadField::Value(Molang::Const(a))), Ok(ReadField::Value(Molang::Const(b)))) => {
            a.to_bits() == b.to_bits() || (a.is_nan() && b.is_nan())
        }
        (Err(ReprError::NumberOutOfRange { .. }), Err(ReprError::NumberOutOfRange { .. })) => true,
        _ => left == right,
    }
}

#[derive(facet::Facet)]
struct ValueField {
    field: Molang,
}

#[derive(facet::Facet)]
struct ListField {
    field: Option<MolangList>,
}

/// The code a facet rejection of a field becomes (the crate's facet-error table).
fn rejection(error: &facet_json::DeserializeError) -> DiagnosticCode {
    code_for_deserialize_error_kind(&error.kind, error.path.is_some(), false)
}

/// An object that lacks `expression` or `version`.
fn lacks_an_object_form_member(value: &Value) -> bool {
    value
        .as_object()
        .is_some_and(|object| object.get("expression").is_none() || object.get("version").is_none())
}

/// The code a reader error becomes (the crate's Molang detail mapping).
fn reader_code(error: &ReprError) -> DiagnosticCode {
    match error {
        ReprError::NumberOutOfRange { .. } => DiagnosticCode::InvalidValue { text: None },
        _ => DiagnosticCode::InvalidType { text: None },
    }
}

/// Path 3 for one reader: the typed field (a member of a struct, as in a version struct) from
/// the text, then pass 1; `Err` with the code of facet's rejection.
fn typed(
    json: &str,
    reader: ReaderKind,
    context: RawVersion,
) -> Result<Result<ReadField, ReprError>, Box<DiagnosticCode>> {
    let text = format!(r#"{{"field": {json}}}"#);
    if reader == ReaderKind::ScalarOrArray {
        let mut list = facet_json::from_str::<ListField>(&text)
            .map_err(|e| Box::new(rejection(&e)))?
            .field;
        return Ok(match &mut list {
            None => Ok(ReadField::NoExpression),
            Some(list @ MolangList::Many(_)) => {
                molang::settle_list(list, context).map(|()| ReadField::ExpressionArray)
            }
            Some(list) => molang::settle_list(list, context).map(|()| match list {
                MolangList::One(value) => ReadField::Value(value.clone()),
                MolangList::Many(_) => ReadField::ExpressionArray,
            }),
        });
    }
    let mut value = facet_json::from_str::<ValueField>(&text)
        .map_err(|e| Box::new(rejection(&e)))?
        .field;
    Ok(molang::settle(&mut value, reader, context).map(|()| ReadField::Value(value)))
}

#[test]
fn the_reader_matrix_agrees_three_ways() {
    let matrix = reader_matrix();
    let rows = items(matrix.as_object().unwrap().get("rows").unwrap());
    assert_eq!(rows.len(), 58, "the fixture's row count");
    let mut failures = Vec::new();
    for row in rows {
        let row = row.as_object().unwrap();
        let case = text(row.get("case").unwrap());
        let context = raw(row.get("context").unwrap());
        let input = scalar(row.get("scalar").unwrap());
        let json = row
            .get("json")
            .and_then(Value::as_string)
            .map(facet_value::VString::as_str);
        let narrower = row.get("facet").is_some_and(|f| text(f) == "narrower");
        let expect = row.get("expect").unwrap().as_object().unwrap();
        for (name, reader) in READERS {
            let want = expected(expect.get(name).unwrap(), reader);
            let repr_result =
                molang::molang::json::MolangValueRepr::read(input.clone(), reader, context);
            if !same(&repr_result, &want) {
                failures.push(format!(
                    "{case} [{name}] crate: {repr_result:?}, want {want:?}"
                ));
            }
            let Some(json) = json else { continue };
            let value = facet_json::from_str::<Value>(json).unwrap();
            let read_result = molang::read(&value, reader, context);
            // The crate's reader gives a number beyond `f32` as the largest finite float, where
            // molang's `MolangValueRepr::read` gives the infinity.
            let want_read = match &want {
                Ok(ReadField::Value(Molang::Const(x))) if x.is_infinite() => {
                    Ok(ReadField::Value(Molang::Const(f32::MAX.copysign(*x))))
                }
                want => want.clone(),
            };
            if !same(&read_result, &want_read) {
                failures.push(format!(
                    "{case} [{name}] molang::read: {read_result:?}, want {want:?}"
                ));
            }
            let facet = typed(json, reader, context);
            let invalid_type = DiagnosticCode::InvalidType { text: None };
            match (&facet, &want) {
                // facet refuses a number not finite as an `f32` through molang's proxy, which the
                // crate maps to `InvalidType`; the strict reader on a `Value` says `InvalidValue`.
                (Err(code), Err(ReprError::NumberOutOfRange { .. })) if **code == invalid_type => {}
                // An object without `expression` or `version` misses a member of molang's
                // object-form struct (`MissingRequiredField`); on a `Value` it is just not the
                // object form (`InvalidType`).
                (Err(code), Err(_))
                    if **code == (DiagnosticCode::MissingRequiredField { text: None })
                        && lacks_an_object_form_member(&value) => {}
                (Err(code), Err(error)) if **code == reader_code(error) => {}
                (Err(code), Ok(_)) if narrower && **code == invalid_type => {}
                (Err(code), _) => failures.push(format!(
                    "{case} [{name}] facet rejects as {code:?}, want {want:?}"
                )),
                (Ok(_), Ok(_)) if narrower => {
                    failures.push(format!(
                        "{case} [{name}] facet accepts a row marked narrower"
                    ));
                }
                (Ok(got), _) if !same(got, &want) => {
                    failures.push(format!(
                        "{case} [{name}] facet + pass 1: {got:?}, want {want:?}"
                    ));
                }
                (Ok(_), _) => {}
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} disagreements:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// A scalar-or-array field read from a `Value` gives the same list as facet + pass 1.
#[test]
fn a_list_read_from_a_value_equals_the_typed_list() {
    for json in [
        "null",
        "1.5",
        "true",
        r#""v.x""#,
        r#"[1, "q.x", [2, false], []]"#,
    ] {
        let value = facet_json::from_str::<Value>(json).unwrap();
        let read_result = molang::read_list(&value, RawVersion(9)).unwrap();
        let mut typed = facet_json::from_str::<Option<MolangList>>(json).unwrap();
        if let Some(list) = &mut typed {
            molang::settle_list(list, RawVersion(9)).unwrap();
        }
        assert_eq!(read_result, typed, "{json}");
    }
    let nested = facet_json::from_str::<Value>(r#"["v.x", ["q.y"]]"#).unwrap();
    assert_eq!(
        molang::read_list(&nested, RawVersion(9)).unwrap(),
        Some(MolangList::Many(vec![
            MolangList::One(Molang::Expr(MolangSource::string("v.x", 9))),
            MolangList::Many(vec![MolangList::One(Molang::Expr(MolangSource::string(
                "q.y", 9
            )))]),
        ]))
    );
}

#[test]
fn a_list_rejects_objects_and_null_elements() {
    for (json, found) in [
        (r#"{"expression": "1", "version": 1}"#, JsonType::Object),
        (r#"[1, {"a": 1}]"#, JsonType::Object),
        ("[1, null]", JsonType::Null),
    ] {
        let value = facet_json::from_str::<Value>(json).unwrap();
        assert_eq!(
            molang::read_list(&value, RawVersion(9)),
            Err(ReprError::UnsupportedType {
                reader: ReaderKind::ScalarOrArray,
                found
            }),
            "{json}"
        );
    }
}

/// An integer literal outside `i64` / `u64` reaches a `Value` as a string of its digits, so a
/// hand-written parser reads it as a Molang string where vanilla reads a real (an accepted
/// loss). The typed path agrees: facet-json hands molang's proxy the same string.
#[test]
fn an_integer_literal_past_64_bits_reads_as_a_string_from_a_value_deviation() {
    let json = "99999999999999999999";
    let value = facet_json::from_str::<Value>(json).unwrap();
    assert_eq!(
        molang::read(&value, ReaderKind::LenientVersioned, RawVersion(13)),
        Ok(ReadField::Value(Molang::Expr(MolangSource::string(
            json, 13
        ))))
    );
    assert_eq!(
        facet_json::from_str::<Molang>(json).ok(),
        Some(Molang::Expr(MolangSource::string_without_context(json)))
    );
}

/// Writing back gives the form that was read: an object stays an object, a
/// string a string, a bool a bool, a number a number of the same value.
#[test]
fn write_gives_back_the_form_that_was_read() {
    for json in [
        "0.5",
        "-3",
        "true",
        "false",
        r#""q.is_baby ? 1 : 2""#,
        r#""""#,
        r#"{"expression": "1 + 2", "version": 42}"#,
        r#"{"expression": "v.x", "version": -1}"#,
    ] {
        let value = facet_json::from_str::<Value>(json).unwrap();
        let Ok(ReadField::Value(read)) =
            molang::read(&value, ReaderKind::LenientVersioned, RawVersion(6))
        else {
            panic!("{json} reads as one value");
        };
        let written = molang::write(&read).unwrap();
        assert_eq!(written.value_type(), value.value_type(), "{json}");
        assert_eq!(
            molang::json_scalar(&written),
            molang::json_scalar(&value),
            "{json}"
        );
        assert_eq!(
            molang::read(&written, ReaderKind::LenientVersioned, RawVersion(6)),
            Ok(ReadField::Value(read)),
            "{json} reads back"
        );
    }
}

/// The text of `json` read as a Molang constant and written back, through `molang::write`
/// (a `Value`) and through the typed writer; the two must agree.
fn rewritten(json: &str) -> String {
    let value = facet_json::from_str::<Value>(json).unwrap();
    let Ok(ReadField::Value(read)) =
        molang::read(&value, ReaderKind::LenientVersioned, RawVersion(6))
    else {
        panic!("{json} reads as one value");
    };
    assert!(matches!(read, Molang::Const(_)), "{json} is a constant");
    let by_value = bedrock_addon_core::json::to_json(&molang::write(&read).unwrap()).unwrap();
    let typed = bedrock_addon_core::json::to_json(&read).unwrap();
    assert_eq!(by_value, typed, "{json}: both writers give one text");
    // Whatever the text, it reads back as the same constant.
    let again = facet_json::from_str::<Value>(typed.trim_end()).unwrap();
    assert_eq!(
        molang::read(&again, ReaderKind::LenientVersioned, RawVersion(6)),
        Ok(ReadField::Value(read)),
        "{json} reads back"
    );
    typed.trim_end().to_owned()
}

// A constant is written as the token an author writes for it, not widened (`1`, not `1.0`;
// `0.1`, not `0.10000000149011612`).
#[test]
fn a_constant_keeps_the_token_an_author_writes() {
    for token in [
        "1",
        "0",
        "-3",
        "64",
        "16777215",
        "-16777215",
        "0.5",
        "0.1",
        "2.5",
        "-0.25",
        "3.14159",
        "100.5",
    ] {
        assert_eq!(rewritten(token), token);
    }
}

/// What the constant does not hold is not kept: an integral real becomes the integer, more
/// digits than an `f32` has become its shortest spelling. The value is the same.
#[test]
fn a_constant_spelled_another_way_is_written_in_its_shortest_form() {
    for (token, written) in [
        ("1.0", "1"),
        ("-2.00", "-2"),
        ("1e3", "1000"),
        ("0.10", "0.1"),
        ("0.100000001490116", "0.1"),
        ("0.30000001192092896", "0.3"),
        // An `f32` holds every integer below 2^24; from there on the real spelling is kept, so
        // a value the integer token would not name exactly is never written as one.
        ("16777216", "16777216.0"),
        ("16777217", "16777216.0"),
        ("1e10", "10000000000.0"),
        // The sign of a negative zero is a real's.
        ("-0.0", "-0.0"),
        // An integer `-0` is zero: there is no negative integer zero to read.
        ("-0", "0"),
    ] {
        assert_eq!(rewritten(token), written, "{token}");
    }
}

/// A spread of `f32` values, the neighbours of each included: the written text always reads
/// back as the same `f32`, also where the shortest decimal lies between two `f64`s.
#[test]
fn every_written_constant_reads_back_as_the_same_f32() {
    let mut bits = 0x0000_0001_u32;
    let mut checked = 0_u32;
    while bits < 0x7f80_0000 {
        for candidate in [bits.saturating_sub(1), bits, bits + 1] {
            for sign in [0, 0x8000_0000] {
                let x = f32::from_bits(candidate | sign);
                if !x.is_finite() {
                    continue;
                }
                let written = molang::write(&Molang::Const(x)).unwrap();
                let JsonScalar::Number(back) = molang::json_scalar(&written) else {
                    panic!("{x:?} is written as a number");
                };
                #[allow(clippy::cast_possible_truncation)]
                let back = back as f32;
                assert_eq!(back.to_bits(), x.to_bits(), "{x:?}");
                checked += 1;
            }
        }
        // A stride that is no power of two visits every exponent and many mantissas.
        bits += 0x0003_9f1b;
    }
    assert!(checked > 50_000, "{checked}");
}

#[test]
fn a_non_finite_constant_is_not_written() {
    use bedrock_addon_core::json::Unrepresentable;
    assert_eq!(
        molang::write(&Molang::Const(f32::NAN)),
        Err(Unrepresentable::NaN)
    );
    assert_eq!(
        molang::write(&Molang::Const(f32::INFINITY)),
        Err(Unrepresentable::Infinity)
    );
    assert_eq!(
        molang::write(&Molang::Const(f32::NEG_INFINITY)),
        Err(Unrepresentable::NegativeInfinity)
    );
}

/// The typed write-back through the crate's JSON writer keeps each form too.
#[test]
fn the_json_writer_keeps_each_form() {
    #[derive(facet::Facet)]
    struct Fields {
        number: Molang,
        flag: Molang,
        text: Molang,
        object: Molang,
        list: Option<MolangList>,
    }
    let fields = Fields {
        number: Molang::Const(2.5),
        flag: Molang::Bool(true),
        text: Molang::Expr(MolangSource::string("q.x", 13)),
        object: Molang::Expr(MolangSource::object("q.x", 4)),
        list: Some(MolangList::Many(vec![
            MolangList::One(Molang::Const(1.5)),
            MolangList::One(Molang::Expr(MolangSource::string("v.y", 13))),
        ])),
    };
    let text = bedrock_addon_core::json::to_json(&fields).unwrap();
    let value = facet_json::from_str::<Value>(&text).unwrap();
    let object = value.as_object().unwrap();
    assert_eq!(
        molang::json_scalar(object.get("number").unwrap()),
        JsonScalar::Number(2.5)
    );
    assert_eq!(object.get("flag").unwrap().as_bool(), Some(true));
    assert_eq!(
        molang::json_scalar(object.get("text").unwrap()),
        JsonScalar::String("q.x".into())
    );
    assert_eq!(
        molang::json_scalar(object.get("object").unwrap()),
        JsonScalar::Object {
            expression: "q.x".into(),
            version: RawVersion(4)
        }
    );
    assert_eq!(
        molang::read_list(object.get("list").unwrap(), RawVersion(13)).unwrap(),
        fields.list
    );
}

/// A number whose nearest `f32` is an infinity is read as the
/// largest finite float of its sign under every reader but the strict one, which refuses it
/// as molang does; the caller is told, so that it can say so once.
#[test]
fn a_number_beyond_f32_is_read_as_the_largest_finite_float() {
    let context = RawVersion(-1);
    for (json, limit) in [
        ("1e39", f32::MAX),
        ("-1e39", f32::MIN),
        ("1e999", f32::MAX),
        ("-1e999", f32::MIN),
        // The first double whose nearest float is the infinity, and 2^128 as an integer token.
        ("3.4028235677973366e38", f32::MAX),
        ("340282366920938463463374607431768211456", f32::MAX),
    ] {
        let value = bedrock_addon_core::json::from_str::<Value>(json).unwrap();
        for (name, reader) in READERS {
            let scalar = molang::json_scalar(&value);
            let read = molang::read(&value, reader, context);
            let finite = molang::read_finite(scalar, reader, context);
            if reader == ReaderKind::StrictVersioned {
                assert!(
                    matches!(read, Err(ReprError::NumberOutOfRange { .. })),
                    "{json} [{name}]: {read:?}"
                );
                assert!(
                    matches!(finite, Err(ReprError::NumberOutOfRange { .. })),
                    "{json} [{name}]: {finite:?}"
                );
                continue;
            }
            let want = ReadField::Value(Molang::Const(limit));
            assert!(
                same(&read, &Ok(want.clone())),
                "{json} [{name}]: read gives {read:?}"
            );
            let (field, clamped) = finite.unwrap();
            assert!(same(&Ok(field), &Ok(want)), "{json} [{name}]");
            assert!(clamped, "{json} [{name}]: the caller is told");
        }
    }
    // The largest float itself, and a number within it, are not clamped.
    for json in [
        "3.4028235e38",
        "-3.4028235e38",
        "3.4028234663852886e38",
        "1e38",
        "0",
        "-2.5",
    ] {
        let value = bedrock_addon_core::json::from_str::<Value>(json).unwrap();
        for (name, reader) in READERS {
            // The strict reader is molang's, unchanged: it refuses the shortest text of the
            // largest float, which as a double is a little above it.
            if reader == ReaderKind::StrictVersioned {
                continue;
            }
            let (field, clamped) =
                molang::read_finite(molang::json_scalar(&value), reader, context).unwrap();
            assert!(!clamped, "{json} [{name}]");
            #[allow(clippy::cast_possible_truncation)]
            let narrow = json.parse::<f64>().unwrap() as f32;
            assert!(
                same(&Ok(field), &Ok(ReadField::Value(Molang::Const(narrow)))),
                "{json} [{name}]"
            );
        }
    }
    // A list read from a value clamps each element alike, and what was read is written.
    let value = bedrock_addon_core::json::from_str::<Value>("[1e39, [-1e999, 2]]").unwrap();
    let list = molang::read_list(&value, context).unwrap().unwrap();
    assert_eq!(
        list,
        MolangList::Many(vec![
            MolangList::One(Molang::Const(f32::MAX)),
            MolangList::Many(vec![
                MolangList::One(Molang::Const(f32::MIN)),
                MolangList::One(Molang::Const(2.0)),
            ]),
        ])
    );
    assert!(molang::write_list(&list).is_ok());
}
