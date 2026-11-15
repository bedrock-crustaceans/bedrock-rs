//! The reading kit of the camera and aim-assist documents, which are read whole on the raw-text
//! path: the strict cereal read and its bounds, the identity of a payload that names itself, the
//! upgrade of an as-written document, the closing line, the source version and the root a
//! raw-text struct writes. The files themselves are read by core's `parse_document`.

use crate::diagnostic::codes as core_codes;
use crate::diagnostic::{Code, Diagnostic, Diagnostics, JsonPath};
use crate::document::{Document, Rejected};
use crate::json::kit::{Expected, Profile, Texts, WrongKind};
use crate::json::{JsonSlice, Value};
use crate::load::LoadContext;
use crate::schema::{FromJson, Schema, SchemaCx};
use crate::version::FormatVersionInput;
use crate::versioning::{
    BeyondWidth, Family, Identity, Loaded, ParseCx, Parsed, Reading, Scalars, beta_refused, finish,
    read_payload_with,
};

/// `reading` with the scalars of vanilla's strict cereal readers, which read every document of
/// these families: a member takes only a token of its own type (an integer member no real), and
/// a number beyond its float's width is a type error. [`read`] reads by it.
#[must_use]
pub const fn strict(reading: Reading) -> Reading {
    reading
        .with_scalars(Scalars::Strict)
        .with_beyond_width(BeyondWidth::Error)
}

/// A value of another JSON type than a member takes, as the strict cereal loaders word it: an
/// object where a struct or a map is bound, an array where a list is, the plain type error for
/// the rest.
fn wrong_kind(expected: Expected, _: &Value) -> Code {
    match expected {
        Expected::Object => crate::diagnostic::cereal::expected_object(),
        Expected::Array => core_codes::INVALID_TYPE_WORDED.with(["expected an array"]),
        _ => Code::from(&core_codes::INVALID_TYPE),
    }
}

/// `text` read as `T` as vanilla's cereal loader reads a document: member by member by
/// [`read_payload_with`] under the slice's reading, every failure told in file order, a missing
/// member and a wrong type as the loader words them, an unknown member worded `unknown` (the
/// family's line). `rewritten` is the note a scalar is told with that a slice read by the
/// non-strict reader ([`Scalars::NonStrict`] in its reading) takes as the other JSON type
/// ([`Read::non_strict`](crate::json::kit::Read::non_strict)); a strict slice tells none.
pub fn read<T: FromJson>(
    text: JsonSlice<'_>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
    unknown: &'static str,
    rewritten: Option<&'static str>,
) -> Option<T> {
    let non_strict = cx.load.reading.scalars == Scalars::NonStrict;
    read_payload_with(text, cx, d, |read| {
        let read = read.profile(Profile::EVERY_FAILURE).texts(Texts {
            wrong_kind: Some(WrongKind(wrong_kind)),
            missing: Some(crate::diagnostic::cereal::MISSING_REQUIRED_FIELD_TEXT),
            unknown: Some(unknown),
            ..Texts::CODES
        });
        match rewritten {
            Some(note) if non_strict => read.non_strict(note),
            _ => read,
        }
    })
}

/// The bounds `value` states (`Schema::check`), at the document's path under the slice's rule,
/// worded with the game's range text where a range has both ends and plain where it has one.
pub fn check<T: Schema>(value: &mut T, cx: &ParseCx<'_>, d: &mut Diagnostics) {
    let mut at = SchemaCx::new(cx.load.reading, cx.at.clone(), d)
        .with_range_text(true)
        .with_one_sided_range_text(false);
    value.check(&mut at);
}

/// The closing line `text` of a failed file.
pub fn closing_line(diagnostics: &mut Diagnostics, text: &'static str) {
    diagnostics.push(Diagnostic::of(
        core_codes::DOCUMENT_REJECTED_WORDED.with([text]),
    ));
}

/// `history::upgrade` of family `F`: rebuilds the parse stage from the document and finishes
/// it. `at` is the payload's path (the root for a family read whole).
///
/// # Errors
///
/// `Rejected` when the upgrade or the family's checks leave no value; it carries the located
/// findings.
pub fn upgrade<F: Family>(
    document: Document<F::History>,
    at: JsonPath,
    cx: &LoadContext,
) -> Result<Document<F::Latest>, Rejected> {
    let (value, origin, version, diagnostics) = document.into_parsed_parts();
    let written = FormatVersionInput {
        version: Some(version.effective),
        beta_literal: false,
        prerelease: false,
    };
    let parsed = Parsed::<F>::from_value(Some(value), version, written, at, diagnostics);
    let mut outcome = finish(parsed, cx);
    // A beta document is refused as the blanket `Content::parse` refuses it (core's
    // `beta_refused`), and the file closed as failed.
    if matches!(outcome.value, Some(Loaded::Beta(_))) {
        outcome.diagnostics.push(beta_refused());
    }
    let failed = !matches!(outcome.value, Some(Loaded::Stable(_)));
    <F as Family>::close(&mut outcome.diagnostics, None, &origin.path, failed);
    let mut diagnostics = outcome.diagnostics;
    diagnostics.locate(&origin);
    match outcome.value {
        Some(Loaded::Stable(value)) => {
            Ok(Document::new(value, origin, outcome.version, diagnostics))
        }
        _ => Err(Rejected::new(origin, diagnostics)),
    }
}

/// The identity of a payload whose identifier is its own member `key` (not in a
/// `description`).
pub fn identity(payload: JsonSlice<'_>, key: &str) -> Identity {
    let identifier = crate::json::from_str_jsonc::<Value>(payload.text)
        .ok()
        .and_then(|value| {
            Some(
                value
                    .as_object()?
                    .get(key)?
                    .as_string()?
                    .as_str()
                    .to_owned(),
            )
        });
    Identity {
        identifier,
        runtime_identifier: None,
    }
}

/// `{"format_version": version, key: payload}`: the whole document of a family read whole.
pub fn document(version: &str, key: &str, payload: Value) -> Value {
    let mut root = facet_value::VObject::new();
    root.insert("format_version", version);
    root.insert(key, payload);
    root.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::{Location, Phase, Severity};
    use crate::load::LoadContext;
    use crate::schema;
    use crate::version::{FormatVersion, FormatVersionInput};

    #[test]
    fn the_closing_line_is_a_rejection_with_its_text() {
        let mut lines = Diagnostics::default();
        closing_line(&mut lines, "closing");
        let lines: Vec<_> = lines.iter().cloned().collect();
        assert_eq!(
            lines,
            [Diagnostic::new(
                core_codes::DOCUMENT_REJECTED_WORDED.with(["closing"]),
                Severity::Error,
                Phase::Parse,
                Location::default()
            )]
        );
    }

    #[derive(Debug, PartialEq, facet::Facet, Schema)]
    struct Probe {
        #[facet(schema::min = 0.0)]
        least: Option<f32>,
        #[facet(schema::min = 0.0, schema::max = 1.0)]
        unit: Option<f32>,
        list: Option<Vec<u8>>,
        #[facet(schema::required)]
        name: Option<String>,
    }

    /// `text` read and checked as a `Probe` under the strict slice's reading, each finding as
    /// `severity path text`.
    fn probe(text: &str) -> (Option<Probe>, Vec<String>) {
        let mut load = LoadContext::detached();
        load.reading = strict(Reading::DOCUMENT);
        let cx = ParseCx {
            load: &load,
            written: FormatVersionInput {
                version: Some(FormatVersion::new(1, 0, 0)),
                beta_literal: false,
                prerelease: false,
            },
            original_version: FormatVersion::new(1, 0, 0),
            at: JsonPath::of_keys(["payload"]),
        };
        let mut d = Diagnostics::default();
        let mut value = read::<Probe>(JsonSlice::new(text), &cx, &mut d, "not in the schema", None);
        if let Some(value) = &mut value {
            check(value, &cx, &mut d);
        }
        let told = d
            .iter()
            .map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code))
            .collect();
        (value, told)
    }

    #[test]
    fn a_document_is_read_as_the_strict_loader_words_it_and_every_failure_is_told() {
        let (value, told) = probe(r#"{"least": "x", "list": 1, "extra": 0}"#);
        assert_eq!(value, None);
        assert_eq!(
            told,
            [
                "Error payload/least wrong JSON type",
                "Error payload/list expected an array",
                "Error payload/extra not in the schema",
                "Error payload/name missing required field",
            ]
        );
        let (value, told) = probe("[]");
        assert_eq!((value, told.len()), (None, 1));
    }

    #[test]
    fn the_bounds_are_checked_after_the_read_and_a_one_sided_one_has_no_text() {
        let (value, told) = probe(r#"{"least": -1, "unit": 2, "name": "n"}"#);
        assert!(value.is_some());
        assert_eq!(
            told,
            [
                "Error payload/least invalid value",
                "Error payload/unit the input is outside of the accepted range [0, 1]",
            ]
        );
    }
}
