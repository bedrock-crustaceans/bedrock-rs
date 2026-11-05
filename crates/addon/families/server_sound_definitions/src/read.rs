//! Reading [`SoundMetadataFile`]: the one struct's `VersionStruct`, its payload read strictly.

use super::{SoundMetadataFile, V1_26_60};
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, JsonPath};
use bedrock_addon_core::json::{
    EncodeError, JsonSlice, NullRule, Value, keep_unknown_members, parse_at, reject_nulls,
};
use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{ParseCx, VersionStruct, Versioned, write_payload};

impl MolangFields for SoundMetadataFile {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

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
        let value: Self = match parse_at::<Self>(payload) {
            Ok(value) => value,
            Err(error) => {
                d.push(error.into_diagnostic(&cx.at, |code, _| code));
                return None;
            }
        };
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
