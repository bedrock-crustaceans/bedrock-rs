//! JSON parse and typed-deserialisation errors: the facet-json error kind → code table,
//! one test per row (and per kind where a row lists several). Kinds facet-json raises from text
//! are produced by parsing; the rest are built by hand.

#![allow(clippy::disallowed_methods, clippy::disallowed_types, reason = "tests read fixtures and call facet-json directly")]
use std::borrow::Cow;

use bedrock_addon_core::diagnostic::{DiagnosticCode, code_for_deserialize_error_kind};
use facet::Facet;
use facet_format::DeserializeErrorKind;

#[derive(Facet, Debug)]
struct Friction {
    value: f32,
}

#[derive(Facet, Debug)]
struct Count {
    count: u8,
}

#[derive(Facet, Debug)]
#[repr(u8)]
enum Side {
    Server,
    Client,
}

#[derive(Facet, Debug)]
struct Pair {
    pair: (u8, u8),
}

#[derive(Facet, Debug)]
struct Expression {
    expression: molangx::json::MolangValueRepr,
}

#[derive(Facet, Debug)]
struct Sided {
    side: Side,
}

/// The kind facet-json reports for `input` read as `T`, whether it has a path, and the code.
fn parse<T: Facet<'static> + std::fmt::Debug>(
    input: &str,
) -> (DeserializeErrorKind, bool, DiagnosticCode) {
    let error = facet_json::from_str::<T>(input).expect_err("input must fail");
    let has_path = error.path.is_some();
    let code = code_for_deserialize_error_kind(&error.kind, has_path, input.trim().is_empty());
    (error.kind, has_path, code)
}

/// The code for a hand-built kind, with a path and a non-blank input.
fn code_of(kind: &DeserializeErrorKind) -> DiagnosticCode {
    code_for_deserialize_error_kind(kind, true, false)
}

fn is_malformed_input(code: &DiagnosticCode) -> bool {
    matches!(code, DiagnosticCode::MalformedInput { text: None, message } if !message.is_empty())
}

// ---- row 1: parser errors → MalformedInput (facet's message) ----

#[test]
fn parser_unexpected_char_is_malformed_input() {
    let (kind, has_path, code) = parse::<Friction>("{\"value\": @}");
    assert!(
        matches!(kind, DeserializeErrorKind::UnexpectedChar { .. }),
        "{kind:?}"
    );
    assert!(!has_path);
    assert!(is_malformed_input(&code), "{code:?}");
}

#[test]
fn parser_unexpected_eof_after_content_is_malformed_input() {
    let (kind, _, code) = parse::<Friction>("{\"value\": 1");
    assert!(
        matches!(kind, DeserializeErrorKind::UnexpectedEof { .. }),
        "{kind:?}"
    );
    assert!(is_malformed_input(&code), "{code:?}");
}

#[test]
fn parser_invalid_utf8_is_malformed_input() {
    let input = b"{\"value\": \"\xff\"}";
    let error = facet_json::from_slice::<Friction>(input).expect_err("invalid UTF-8");
    assert!(
        matches!(error.kind, DeserializeErrorKind::InvalidUtf8 { .. }),
        "{:?}",
        error.kind
    );
    let code = code_for_deserialize_error_kind(&error.kind, error.path.is_some(), false);
    assert!(is_malformed_input(&code), "{code:?}");
}

#[test]
fn parser_unexpected_token_without_path_is_malformed_input() {
    let (kind, has_path, code) = parse::<Friction>("{\"value\" 1}");
    assert!(
        matches!(kind, DeserializeErrorKind::UnexpectedToken { .. }),
        "{kind:?}"
    );
    assert!(!has_path);
    assert!(is_malformed_input(&code), "{code:?}");
}

// ---- row 2: UnexpectedEof with nothing before it → EmptyDocument ----

#[test]
fn empty_file_is_empty_document() {
    let (kind, _, code) = parse::<Friction>("");
    assert!(
        matches!(kind, DeserializeErrorKind::UnexpectedEof { .. }),
        "{kind:?}"
    );
    assert!(
        matches!(code, DiagnosticCode::EmptyDocument { text: None }),
        "{code:?}"
    );
}

#[test]
fn whitespace_only_file_is_empty_document() {
    let (kind, _, code) = parse::<Friction>(" \r\n\t ");
    assert!(
        matches!(kind, DeserializeErrorKind::UnexpectedEof { .. }),
        "{kind:?}"
    );
    assert!(
        matches!(code, DiagnosticCode::EmptyDocument { text: None }),
        "{code:?}"
    );
}

// ---- row 3: DuplicateField → MalformedInput ----

#[test]
fn duplicate_field_is_malformed_input() {
    let kind = DeserializeErrorKind::DuplicateField {
        field: Cow::Borrowed("value"),
        first_span: None,
    };
    assert!(is_malformed_input(&code_of(&kind)));
}

// ---- row 4: MissingField → MissingRequiredField ----

#[test]
fn missing_field_is_missing_required_field() {
    let (kind, _, code) = parse::<Friction>("{}");
    assert!(
        matches!(kind, DeserializeErrorKind::MissingField { .. }),
        "{kind:?}"
    );
    assert!(
        matches!(code, DiagnosticCode::MissingRequiredField { text: None }),
        "{code:?}"
    );
}

// ---- row 5: type and shape failures → InvalidType ----

#[test]
fn type_mismatch_is_invalid_type() {
    let kind = DeserializeErrorKind::TypeMismatch {
        expected: <f32 as Facet>::SHAPE,
        got: Cow::Borrowed("string"),
    };
    assert!(matches!(
        code_of(&kind),
        DiagnosticCode::InvalidType { text: None }
    ));
}

#[test]
fn shape_mismatch_is_invalid_type() {
    let kind = DeserializeErrorKind::ShapeMismatch {
        expected: <f32 as Facet>::SHAPE,
        got: <String as Facet>::SHAPE,
    };
    assert!(matches!(
        code_of(&kind),
        DiagnosticCode::InvalidType { text: None }
    ));
}

#[test]
fn unexpected_token_with_path_is_invalid_type() {
    let (kind, has_path, code) = parse::<Pair>("{\"pair\": 5}");
    assert!(
        matches!(kind, DeserializeErrorKind::UnexpectedToken { .. }),
        "{kind:?}"
    );
    assert!(has_path);
    assert!(
        matches!(code, DiagnosticCode::InvalidType { text: None }),
        "{code:?}"
    );
}

#[test]
fn no_matching_variant_is_invalid_type() {
    let (kind, _, code) = parse::<Expression>("{\"expression\": [1]}");
    assert!(
        matches!(kind, DeserializeErrorKind::NoMatchingVariant { .. }),
        "{kind:?}"
    );
    assert!(
        matches!(code, DiagnosticCode::InvalidType { text: None }),
        "{code:?}"
    );
}

/// A rejection by the `molangx` facet proxy, which `Reflect` includes.
#[test]
fn reflect_is_invalid_type() {
    let (kind, _, code) =
        parse::<Expression>("{\"expression\": {\"expression\": \"1\", \"version\": 1.5}}");
    assert!(
        matches!(kind, DeserializeErrorKind::Reflect { .. }),
        "{kind:?}"
    );
    assert!(
        matches!(code, DiagnosticCode::InvalidType { text: None }),
        "{code:?}"
    );
}

#[test]
fn solver_is_invalid_type() {
    let kind = DeserializeErrorKind::Solver {
        message: Cow::Borrowed("No matching configuration"),
    };
    assert!(matches!(
        code_of(&kind),
        DiagnosticCode::InvalidType { text: None }
    ));
}

// ---- row 6: UnknownVariant → InvalidEnumValue ----

#[test]
fn unknown_variant_is_invalid_enum_value() {
    let kind = DeserializeErrorKind::UnknownVariant {
        variant: Cow::Borrowed("Both"),
        enum_shape: <Side as Facet>::SHAPE,
    };
    assert!(matches!(
        code_of(&kind),
        DiagnosticCode::InvalidEnumValue { text: None }
    ));
}

/// facet-json reports an unknown name of a plain (externally tagged) enum as `UnexpectedToken`
/// expecting a "known enum variant", not as `UnknownVariant`; it is still `InvalidEnumValue`.
#[test]
fn unknown_enum_name_reported_as_unexpected_token_is_invalid_enum_value() {
    let (kind, has_path, code) = parse::<Sided>("{\"side\": \"Both\"}");
    assert!(
        matches!(
            kind,
            DeserializeErrorKind::UnexpectedToken {
                expected: "known enum variant",
                ..
            }
        ),
        "{kind:?}"
    );
    assert!(has_path);
    assert!(
        matches!(code, DiagnosticCode::InvalidEnumValue { text: None }),
        "{code:?}"
    );
}

// ---- row 7: value errors → InvalidValue ----

#[test]
fn number_out_of_range_is_invalid_value() {
    let (kind, _, code) = parse::<Count>("{\"count\": 300}");
    assert!(
        matches!(kind, DeserializeErrorKind::NumberOutOfRange { .. }),
        "{kind:?}"
    );
    assert!(
        matches!(code, DiagnosticCode::InvalidValue { text: None }),
        "{code:?}"
    );
}

#[test]
fn invalid_value_is_invalid_value() {
    let kind = DeserializeErrorKind::InvalidValue {
        message: Cow::Borrowed("not a UUID"),
    };
    assert!(matches!(
        code_of(&kind),
        DiagnosticCode::InvalidValue { text: None }
    ));
}

#[test]
fn validation_is_invalid_value() {
    let kind = DeserializeErrorKind::Validation {
        field: "count",
        message: Cow::Borrowed("must be positive"),
    };
    assert!(matches!(
        code_of(&kind),
        DiagnosticCode::InvalidValue { text: None }
    ));
}

// ---- row 8: UnknownField is not raised; if it is, it is UnrecognizedField ----

#[test]
fn unknown_field_is_not_raised_for_a_plain_struct() {
    let parsed = facet_json::from_str::<Friction>("{\"value\": 1, \"extra\": 2}");
    assert!(
        parsed.is_ok(),
        "unknown keys are ignored by facet: {parsed:?}"
    );
    let kind = DeserializeErrorKind::UnknownField {
        field: Cow::Borrowed("extra"),
        suggestion: None,
    };
    assert!(matches!(
        code_of(&kind),
        DiagnosticCode::UnrecognizedField { text: None }
    ));
}

// ---- row 9: the crate's own bugs → MalformedInput with facet's text, plus a debug assertion ----

/// The internal kinds panic in debug builds (the `debug_assert!`) and map to `MalformedInput` in
/// release builds.
fn assert_internal(kind: &DeserializeErrorKind) {
    let result = std::panic::catch_unwind(|| code_of(kind));
    if cfg!(debug_assertions) {
        assert!(result.is_err(), "{kind:?} must trip the debug assertion");
    } else {
        let code = result.expect("no assertion in release");
        assert!(is_malformed_input(&code), "{code:?}");
    }
}

/// A scalar-or-scalar untagged enum, the shape of an identifier proxy.
#[derive(Facet, Debug)]
#[facet(untagged)]
#[repr(u8)]
#[allow(dead_code)]
enum NameOrNumber {
    Name(String),
    Number(i64),
}

#[derive(Facet, Debug)]
struct Named {
    identifier: NameOrNumber,
}

// facet-json reports an object given to an untagged enum without a struct arm as `Unsupported`.
// The input causes it, not a bug of this crate: a wrong type, no assertion.
#[test]
fn an_object_given_to_a_scalar_untagged_enum_is_invalid_type_and_no_panic() {
    let (kind, has_path, code) = parse::<Named>(r#"{"identifier": {}}"#);
    assert!(
        matches!(kind, DeserializeErrorKind::Unsupported { .. }),
        "{kind:?}"
    );
    assert!(has_path);
    assert!(
        matches!(code, DiagnosticCode::InvalidType { text: None }),
        "{code:?}"
    );
}

#[test]
fn unsupported_without_a_path_is_malformed_input_and_no_panic() {
    let kind = DeserializeErrorKind::Unsupported {
        message: Cow::Borrowed("x"),
    };
    let code = code_for_deserialize_error_kind(&kind, false, false);
    assert!(is_malformed_input(&code), "{code:?}");
}

#[test]
fn cannot_borrow_is_malformed_input_and_a_bug() {
    assert_internal(&DeserializeErrorKind::CannotBorrow {
        reason: Cow::Borrowed("escaped string"),
    });
}

#[test]
fn raw_capture_not_supported_is_malformed_input_and_a_bug() {
    assert_internal(&DeserializeErrorKind::RawCaptureNotSupported {
        shape: <f32 as Facet>::SHAPE,
    });
}

#[test]
fn materialize_is_malformed_input_and_a_bug() {
    assert_internal(&DeserializeErrorKind::Materialize {
        expected: <f32 as Facet>::SHAPE,
        actual: <u8 as Facet>::SHAPE,
    });
}

#[test]
fn alloc_is_malformed_input_and_a_bug() {
    assert_internal(&DeserializeErrorKind::Alloc {
        shape: <f32 as Facet>::SHAPE,
        operation: "alloc",
    });
}

#[test]
fn io_is_malformed_input_and_a_bug() {
    assert_internal(&DeserializeErrorKind::Io {
        message: Cow::Borrowed("read"),
    });
}

#[test]
fn bug_is_malformed_input_and_a_bug() {
    assert_internal(&DeserializeErrorKind::Bug {
        error: Cow::Borrowed("x"),
        context: "y",
    });
}

/// The message `MalformedInput` carries is facet's own text for the kind.
#[test]
fn malformed_input_carries_facets_message() {
    let (kind, _, code) = parse::<Friction>("{\"value\": @}");
    let DiagnosticCode::MalformedInput { message, .. } = code else {
        panic!("{code:?}");
    };
    assert_eq!(message, kind.to_string());
}
