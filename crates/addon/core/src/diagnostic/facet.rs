//! The facet-json error kind → code table of the crate.

use facet_format::DeserializeErrorKind;

use super::DiagnosticCode;

/// The code for one facet-json `DeserializeError`, by its kind (the crate's table).
///
/// `has_path` is whether the error carries a path (facet's deserializer sets one, its parser
/// does not); `blank_input` is whether the input was empty or whitespace only. Parser errors are
/// `MalformedInput` with facet's message, `UnexpectedEof` on a blank input is `EmptyDocument`,
/// a missing member `MissingRequiredField`, a wrong type or shape `InvalidType`, an unknown enum
/// name `InvalidEnumValue`, a value error `InvalidValue`. `Unsupported` is a wrong type when it
/// has a path and `MalformedInput` otherwise. The kinds that only a bug of this crate can cause are
/// `MalformedInput` and trip a debug assertion.
#[allow(clippy::match_same_arms)] // one arm per kind of the kind-to-code table, plus the fallback
pub fn code_for_deserialize_error_kind(
    kind: &DeserializeErrorKind,
    has_path: bool,
    blank_input: bool,
) -> DiagnosticCode {
    use DeserializeErrorKind as K;
    match kind {
        K::UnexpectedEof { .. } if blank_input => DiagnosticCode::EmptyDocument { text: None },
        // facet-json (rc.7) reports an unknown name of a plain enum this way, not as
        // `UnknownVariant`.
        K::UnexpectedToken {
            expected: UNKNOWN_ENUM_NAME,
            ..
        } if has_path => DiagnosticCode::InvalidEnumValue { text: None },
        K::UnexpectedToken { .. } if has_path => DiagnosticCode::InvalidType { text: None },
        K::UnexpectedChar { .. }
        | K::UnexpectedEof { .. }
        | K::InvalidUtf8 { .. }
        | K::UnexpectedToken { .. }
        | K::DuplicateField { .. } => malformed(kind),
        K::MissingField { .. } => DiagnosticCode::MissingRequiredField { text: None },
        K::TypeMismatch { .. }
        | K::ShapeMismatch { .. }
        | K::NoMatchingVariant { .. }
        | K::Reflect { .. }
        | K::Solver { .. } => DiagnosticCode::InvalidType { text: None },
        K::UnknownVariant { .. } => DiagnosticCode::InvalidEnumValue { text: None },
        K::NumberOutOfRange { .. } | K::InvalidValue { .. } | K::Validation { .. } => {
            DiagnosticCode::InvalidValue { text: None }
        }
        // Version structs keep unknown keys (`extra`), so facet does not raise this; were it
        // raised, it would be this code.
        K::UnknownField { .. } => DiagnosticCode::UnrecognizedField { text: None },
        // facet-json (rc.7) reports an object given to an untagged enum without a struct arm
        // (an identifier proxy, say) this way: the input causes it, so it is a wrong type at
        // its path and no assertion.
        K::Unsupported { .. } if has_path => DiagnosticCode::InvalidType { text: None },
        K::Unsupported { .. } => malformed(kind),
        K::CannotBorrow { .. }
        | K::RawCaptureNotSupported { .. }
        | K::Materialize { .. }
        | K::Alloc { .. }
        | K::Io { .. }
        | K::Bug { .. } => {
            debug_assert!(false, "facet-json reported an internal error: {kind}");
            malformed(kind)
        }
        // `DeserializeErrorKind` is `#[non_exhaustive]`: a kind added by a facet bump reads as
        // a malformed input until the table names it.
        _ => malformed(kind),
    }
}

/// What facet-json's enum deserializer expects when a plain enum's name is unknown.
const UNKNOWN_ENUM_NAME: &str = "known enum variant";

fn malformed(kind: &DeserializeErrorKind) -> DiagnosticCode {
    DiagnosticCode::MalformedInput {
        message: kind.to_string(),
        text: None,
    }
}
