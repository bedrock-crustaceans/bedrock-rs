//! Reading [`SoundMetadataFile`]: the one struct's `VersionStruct`, its payload read strictly.

use super::{SoundMetadataFile, V1_26_60};
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, JsonPath};
use bedrock_addon_core::json::{
    EncodeError, JsonSlice, NullRule, ScalarFailure, ScalarRead, Value, keep_unknown_members,
    parse_with_rules, reject_nulls,
};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{ParseCx, VersionStruct, Versioned, write_payload};

impl Versioned for SoundMetadataFile {
    const VERSION: FormatVersion = V1_26_60;
}

/// The tag values that are neither a string nor an array of strings.
/// facet reads a number into the untagged string arm, so the JSON is checked first.
fn mistyped_tags(payload: &Value) -> Vec<JsonPath> {
    let mut out = Vec::new();
    let definitions = payload.as_array().map(facet_value::VArray::as_slice);
    for (index, definition) in definitions.into_iter().flatten().enumerate() {
        let tags = definition
            .as_object()
            .and_then(|definition| definition.get("tags"))
            .and_then(Value::as_object);
        for (key, value) in tags.into_iter().flatten() {
            let strings = |value: &Value| value.is_string();
            let fits = strings(value)
                || value
                    .as_array()
                    .is_some_and(|values| values.as_slice().iter().all(strings));
            if !fits {
                out.push(
                    JsonPath::new()
                        .index(index)
                        .child("tags")
                        .child(key.as_str()),
                );
            }
        }
    }
    out
}

impl VersionStruct for SoundMetadataFile {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let json = bedrock_addon_core::json::from_str_jsonc::<Value>(payload.text).ok();
        if json.as_ref().is_some_and(|json| !json.is_array()) {
            // The payload is the array itself.
            d.push(Diagnostic::error(DiagnosticCode::InvalidType { text: None }).at(cx.at.clone()));
            return None;
        }
        let mistyped = json.as_ref().map(mistyped_tags).unwrap_or_default();
        if !mistyped.is_empty() {
            for at in &mistyped {
                d.push(
                    Diagnostic::error(DiagnosticCode::InvalidType { text: None })
                        .at(cx.at.join(at)),
                );
            }
            return None;
        }
        // The scalars are read by the slice's rules (`Family::reading`: strict, a number beyond
        // its float a type error).
        // A `null` read as a struct is the null rule's: `reject_nulls` below tells it where the
        // input rejects nulls, and elsewhere it reads as its defaults untold.
        let rules = cx.load.scalar_rules.with_tells_null_struct(false);
        let ScalarRead { value, told, .. } =
            match parse_with_rules::<Self>(payload, cx.load.null_rule, rules) {
                Ok(read) => read,
                Err(ScalarFailure::Read(error)) => {
                    d.push(error.into_diagnostic(&cx.at, |code, _| code));
                    return None;
                }
                Err(ScalarFailure::Refused(misread)) => {
                    d.push(misread.finding(&cx.at));
                    return None;
                }
            };
        // Under the strict rules the pre-pass refuses every conversion, so it tells nothing.
        for misread in told {
            d.push(misread.finding(&cx.at));
        }
        // A member written as `null` is present and fails its type: the game reads the file
        // with null members rejected.
        if cx.load.null_rule == Some(NullRule::Rejected) && reject_nulls(&value, payload, &cx.at, d)
        {
            return None;
        }
        // The strict mask: an unknown member at any level rejects the file.
        let mut kept = bedrock_addon_core::document::Unknown::new();
        let unknown = json
            .map(|json| keep_unknown_members(&value, &json, &mut kept))
            .unwrap_or_default();
        d.unknown_mut().absorb(&cx.at, kept);
        for (path, _) in &unknown {
            d.push(
                Diagnostic::error(DiagnosticCode::UnrecognizedField { text: None })
                    .at(cx.at.join(path)),
            );
        }
        // An error-level finding rejects the document, so the value goes no further.
        Some(value)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}
