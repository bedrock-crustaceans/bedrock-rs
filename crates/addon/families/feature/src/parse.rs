//! Parsing a feature payload the way vanilla's strict schema validates it: typed from its slice
//! of the text, and every unknown member, wrong type or missing required member rejects the
//! file: the client validates with unknown children not allowed.

use facet::Facet;

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, VanillaText,
};
use bedrock_addon_core::json::{
    JsonSlice, NullRule, SliceError, Value, parse_at, parse_at_null_absent, unknown_members,
};
use bedrock_addon_core::versioning::ParseCx;

/// The client's text for a member the schema does not have.
pub(crate) const NOT_VALID_HERE: &str = "child '{}' not valid here.";
/// The client's text for a required member that is absent.
pub(crate) const REQUIRED_CHILD: &str = bedrock_addon_core::diagnostic::cereal::REQUIRED_CHILD_TEXT;
/// The client's text for an integer outside its node's range.
pub(crate) const INT_OUT_OF_RANGE: &str = "Value '{}' outside valid range [{}, {}]";
/// The client's text for a float outside its node's range.
pub(crate) const FLOAT_OUT_OF_RANGE: &str = "Value '{}' outside valid range [{}, {}]";
/// The client's text for an `IntRange` whose lower end is outside the node's range.
pub(crate) const RANGE_MIN_OUT_OF_RANGE: &str =
    "'range_min' with value '{}' outside valid range [{}, {}]";
/// The client's text for an `IntRange` whose upper end is outside the node's range.
pub(crate) const RANGE_MAX_OUT_OF_RANGE: &str =
    "'range_max' with value '{}' outside valid range [{}, {}]";
/// The client's text for an array below its minimum length.
pub(crate) const ARRAY_TOO_SMALL: &str = "Array too small ({} < {})";
/// The client's text for an array above its maximum length.
pub(crate) const ARRAY_TOO_LARGE: &str = "Array too large ({} > {})";

/// A `VanillaText::Formatted` of `template` with `args`.
pub(crate) fn formatted(template: &'static str, args: &[&dyn std::fmt::Display]) -> VanillaText {
    VanillaText::Formatted {
        template,
        args: args.iter().map(ToString::to_string).collect(),
    }
}

/// The last member name of `path` as the file has it (`"y"` for
/// `minecraft:scatter_feature/distribution/y`); an element's number for a path that ends at
/// an array element.
pub(crate) fn last_key(path: &JsonPath) -> String {
    match path.segments().last() {
        Some(bedrock_addon_core::diagnostic::JsonPathSegment::Key(key)) => key.to_owned(),
        Some(bedrock_addon_core::diagnostic::JsonPathSegment::Index(index)) => index.to_string(),
        None => String::new(),
    }
}

/// `UnrecognizedField` (error) with vanilla's text, for the member at `at`.
pub(crate) fn not_valid_here(at: JsonPath) -> Diagnostic {
    let key = last_key(&at);
    Diagnostic::error(DiagnosticCode::UnrecognizedField {
        text: Some(formatted(NOT_VALID_HERE, &[&key])),
    })
    .at(at)
}

/// `MissingRequiredField` (error) with vanilla's text, for the member at `at`.
pub(crate) fn required_child(at: JsonPath) -> Diagnostic {
    let key = last_key(&at);
    Diagnostic::error(DiagnosticCode::MissingRequiredField {
        text: Some(formatted(REQUIRED_CHILD, &[&key])),
    })
    .at(at)
}

/// Parses `T` from `payload` strictly: a failed typed parse is one error at the failing member
/// (an unknown enum name is `InvalidValue`, as vanilla's enum nodes reject it), and every member
/// the struct does not take is an `UnrecognizedField` error. `None` when anything failed.
pub(crate) fn parse_strict<'a, T: Facet<'a>>(
    payload: JsonSlice<'a>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Option<T> {
    parse_strict_with(payload, cx, d, &[])
}

/// [`parse_strict`] with the setters' texts for members whose value is a name: an unknown name
/// of the member `key` is `InvalidValue` with `text` (vanilla's "Bad value for …" messages).
pub(crate) fn parse_strict_with<'a, T: Facet<'a>>(
    payload: JsonSlice<'a>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
    texts: &[(&str, &'static str)],
) -> Option<T> {
    // The null rule of the selected version, which the family declares (`Family::INPUT` is
    // `JsonCpp`): under rule N1 a member written as `null` is absent.
    let absent = cx.load.null_rule == Some(NullRule::Absent);
    let parsed = if absent {
        parse_at_null_absent::<T>(payload)
    } else {
        parse_at::<T>(payload)
    };
    let parsed = match parsed {
        Ok(value) => Some(value),
        Err(error) => {
            let mut finding = slice_error(error, &cx.at);
            if let DiagnosticCode::InvalidValue { text } = &mut finding.code {
                let key = last_key(&finding.location.json_path);
                if let Some((_, message)) = texts.iter().find(|(k, _)| *k == key) {
                    *text = Some(VanillaText::Fixed(message));
                }
            }
            d.push(finding);
            None
        }
    };
    // Vanilla's validation reports every unknown member, also beside another failure.
    if let Ok(json) = bedrock_addon_core::json::from_str_jsonc::<Value>(payload.text) {
        if absent {
            // The members read as absent, which the write-back writes as their defaults
            // told here once, the only N1 read of a feature payload.
            let found = bedrock_addon_core::json::null_members(T::SHAPE, &json);
            bedrock_addon_core::json::tell_null_members(&found, &cx.at, d);
        }
        let unknown = unknown_members(T::SHAPE, &json);
        for (path, _) in &unknown {
            d.push(not_valid_here(cx.at.join(path)));
        }
        if !unknown.is_empty() {
            return None;
        }
    }
    parsed
}

/// The finding of a failed typed parse of `slice`, at `base` followed by the failing member.
fn slice_error(error: SliceError, base: &JsonPath) -> Diagnostic {
    error.into_diagnostic(base, |code, error| {
        let at = || base.join(error.relative_path());
        match code {
            // The client's enum members reject an unknown name as a value.
            DiagnosticCode::InvalidEnumValue { .. } => DiagnosticCode::InvalidValue { text: None },
            DiagnosticCode::MissingRequiredField { .. } => DiagnosticCode::MissingRequiredField {
                text: Some(formatted(REQUIRED_CHILD, &[&last_key(&at())])),
            },
            DiagnosticCode::UnrecognizedField { .. } => DiagnosticCode::UnrecognizedField {
                text: Some(formatted(NOT_VALID_HERE, &[&last_key(&at())])),
            },
            other => other,
        }
    })
}
