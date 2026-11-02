//! Reading a Molang field from an untyped `Value` (hand-written parsers) and pass 1 on a typed
//! one: the reader check and the context version.

use std::sync::Arc;

use super::{
    JsonScalar, JsonType, Molang, MolangList, MolangSource, OtherJson, RawVersion, ReadField,
    ReaderKind, ReprError, SourceForm,
};
use crate::diagnostic::{Diagnostic, DiagnosticCode, JsonPath, Location, Phase, Severity, Span};
use crate::json::Value;

/// The JSON value of a Molang field as the crate's readers take it: a number of either tag is a
/// `Number`, a bool a `Bool`, a string a `String`; an object with a string `expression` and an
/// integer-tagged `version` that fits an `i16` is the object form (other members ignored, as
/// molangx's facet impl ignores them); `null`, arrays and any other object are `Other`.
///
/// An integer literal outside `i64` / `u64` reaches a `Value` as a string of its digits, so it is
/// read as a Molang string, not a constant (an accepted loss).
#[doc(hidden)]
#[must_use]
pub fn json_scalar(value: &Value) -> JsonScalar {
    if let Some(number) = value.as_number() {
        return JsonScalar::Number(number.to_f64_lossy());
    }
    if let Some(b) = value.as_bool() {
        return JsonScalar::Bool(b);
    }
    if let Some(text) = value.as_string() {
        return JsonScalar::String(Arc::from(text.as_str()));
    }
    if value.is_null() {
        return JsonScalar::Other(OtherJson::Null);
    }
    if value.is_array() {
        return JsonScalar::Other(OtherJson::Array);
    }
    // An object, or a value kind facet-json never produces from text.
    let object_form = value.as_object().and_then(|object| {
        let expression = object.get("expression")?.as_string()?;
        let version = object.get("version")?.as_number()?;
        // `12.0` is a real token: not the object form (molangx's `VersionNotInteger`).
        if !version.is_integer() {
            return None;
        }
        let version = i16::try_from(version.to_i64()?).ok()?;
        Some(JsonScalar::Object {
            expression: Arc::from(expression.as_str()),
            version: RawVersion(version),
        })
    });
    object_form.unwrap_or(JsonScalar::Other(OtherJson::Object))
}

/// Reads a Molang field of kind `reader` from an untyped value: [`json_scalar`], then molangx's
/// reader, which applies the reader's rules and gives a plain string `context_version`. A number
/// beyond `f32` is read as [`read_finite`] reads it; a caller that reports findings calls that.
///
/// `ReadField::NoExpression` and `ReadField::ExpressionArray` arise only under
/// `ReaderKind::ScalarOrArray`; [`read_list`] reads such a field whole.
#[doc(hidden)]
pub fn read(
    value: &Value,
    reader: ReaderKind,
    context_version: RawVersion,
) -> Result<ReadField, ReprError> {
    read_finite(json_scalar(value), reader, context_version).map(|(field, _)| field)
}

/// Reads `json` as a field of kind `reader`, as molangx's reader does, but for a number whose
/// nearest `f32` is an infinity (`1e39`, `1e999`, an integer token past 2^128): that is read as
/// the largest finite float of its sign, and the second part of the result is `true` so the
/// caller can report it once ([`beyond_f32_warning`]). This is the crate's own
/// rule for every family: vanilla's `asFloat` gives the infinity, which no writer can spell.
///
/// `ReaderKind::StrictVersioned` refuses such a number with `NumberOutOfRange`, as molangx does;
/// nothing is clamped there. A typed `Molang` member never sees such a number as a constant:
/// facet refuses it (`json::clamp_text` rewrites the text first where a family wants the clamp).
#[doc(hidden)]
pub fn read_finite(
    json: JsonScalar,
    reader: ReaderKind,
    context_version: RawVersion,
) -> Result<(ReadField, bool), ReprError> {
    let (json, clamped) = match json {
        JsonScalar::Number(number)
            if reader != ReaderKind::StrictVersioned && beyond_f32(number).is_some() =>
        {
            let limit = f64::from(f32::MAX).copysign(number);
            (JsonScalar::Number(limit), true)
        }
        json => (json, false),
    };
    Molang::read(json, reader, context_version).map(|field| (field, clamped))
}

/// Whether the nearest `f32` of `number` is an infinity, and then whether it is below zero.
/// A NaN, which no JSON text spells, is not.
#[doc(hidden)]
#[must_use]
pub fn beyond_f32(number: f64) -> Option<bool> {
    #[allow(
        clippy::cast_possible_truncation,
        reason = "the nearest f32 is the question"
    )]
    let nearest = number as f32;
    nearest.is_infinite().then_some(number < 0.0)
}

/// The finding of a number read as the largest finite float ([`read_finite`]): an
/// `InvalidValue` information of the parse phase at `at`, on the number's token when `span`
/// is known. The clamp is the crate's own, for the write-back; the
/// game's readers hold the infinity in silence.
#[doc(hidden)]
#[must_use]
pub fn beyond_f32_warning(at: JsonPath, span: Option<Span>) -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::InvalidValue { text: None },
        Severity::Info,
        Phase::Parse,
        Location::new(None, None, at, span),
    )
}

/// Reads a value-or-array field (`ReaderKind::ScalarOrArray`) whole: `null` is `None`, an array
/// is read element by element, recursively. A `null` element or any object is rejected: the
/// list has no slot for them, as its facet impl does not.
#[doc(hidden)]
pub fn read_list(
    value: &Value,
    context_version: RawVersion,
) -> Result<Option<MolangList>, ReprError> {
    if value.is_null() {
        return Ok(None);
    }
    read_element(value, context_version).map(Some)
}

fn read_element(value: &Value, context_version: RawVersion) -> Result<MolangList, ReprError> {
    let reader = ReaderKind::ScalarOrArray;
    if let Some(elements) = value.as_array() {
        return elements
            .iter()
            .map(|element| read_element(element, context_version))
            .collect::<Result<_, _>>()
            .map(MolangList::Many);
    }
    match read(value, reader, context_version)? {
        ReadField::Value(value) => Ok(MolangList::One(value)),
        // `null` here is an element: the top-level `null` was handled by the caller.
        _ => Err(ReprError::UnsupportedType {
            reader,
            found: JsonType::Null,
        }),
    }
}

/// Pass 1 for a typed `Molang` field: whether `reader` accepts the value facet produced
/// (`MolangValueRepr::check`), then the context version for a string form; an object form keeps
/// its own.
#[doc(hidden)]
pub fn settle(
    value: &mut Molang,
    reader: ReaderKind,
    context_version: RawVersion,
) -> Result<(), ReprError> {
    value.check(reader)?;
    value.set_context_version(context_version);
    Ok(())
}

/// Pass 1 for a typed value-or-array field: every element is checked under
/// `ReaderKind::ScalarOrArray` and takes `context_version`.
#[doc(hidden)]
pub fn settle_list(list: &mut MolangList, context_version: RawVersion) -> Result<(), ReprError> {
    match list {
        MolangList::One(value) => settle(value, ReaderKind::ScalarOrArray, context_version),
        MolangList::Many(elements) => elements
            .iter_mut()
            .try_for_each(|element| settle_list(element, context_version)),
    }
}

/// Pass 1 for a string-only field (`MolangSource`): molangx has no string-only reader, so the
/// object form is refused here (as `reader`'s `UnsupportedType` of an object), then the string
/// takes `context_version`.
#[doc(hidden)]
pub fn settle_source(
    source: &mut MolangSource,
    reader: ReaderKind,
    context_version: RawVersion,
) -> Result<(), ReprError> {
    if !matches!(source.form(), SourceForm::String { .. }) {
        return Err(ReprError::UnsupportedType {
            reader,
            found: JsonType::Object,
        });
    }
    source.set_context_version(context_version);
    Ok(())
}
