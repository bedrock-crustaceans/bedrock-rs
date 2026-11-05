//! `texture_backwards_compatibility_mapping.json`: the game's own table of textures that moved.
//! The client reads it from its app package only, never from a pack, and its layout is not
//! modelled, so the file is held as the JSON it is.
//!
//! Nothing is read into a type: the file is held as the JSON it is.

#![cfg_attr(docsrs, feature(doc_cfg))]

use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics};
use bedrock_addon_core::document::{Content, Document, JsonContent, Placed, Rejected, unversioned};
use bedrock_addon_core::json::{EncodeError, Value, WriteStyle, write_json};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::{PackPath, PackSide};
use bedrock_addon_support::texture::jsoncpp::read_lenient;

/// The file's path at the root of the app package.
const PATH: &str = "texture_backwards_compatibility_mapping.json";

/// The backwards-compatibility texture mapping of the game's app package.
///
/// A resource pack that ships the file is not read for it. The typed layout is unknown (the
/// reader's keys are `back_compat_texture`, `uv`, `uv_size` and `base_size`), so the document
/// is its JSON.
#[doc(alias = "texture_backwards_compatibility_mapping.json")]
#[derive(Clone, Debug, PartialEq)]
pub struct LegacyPathMapping {
    /// The file's JSON, as read.
    pub json: Value,
}

impl bedrock_addon_core::__private::Sealed for LegacyPathMapping {}

impl Content for LegacyPathMapping {
    const NAME: &'static str = "legacy texture path mapping";
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// Reads the file as JSON (comments allowed). Rejected when it is no JSON.
    fn parse(path: &PackPath, bytes: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
        // A number no double holds is read as the largest finite one, with a warning.
        bedrock_addon_core::document::FiniteText::load(bytes, |bytes| {
            let origin = unversioned::lone(path);
            let mut diagnostics = Diagnostics::default();
            match read_lenient(bytes) {
                Ok(read) => {
                    let mapping = LegacyPathMapping { json: read.value };
                    Ok(unversioned::document(mapping, origin, diagnostics))
                }
                Err(message) => {
                    let code = DiagnosticCode::MalformedInput {
                        message: message.to_string(),
                        text: None,
                    };
                    diagnostics.push(Diagnostic::error(code).in_file(&origin));
                    Err(unversioned::rejected(origin, diagnostics))
                }
            }
        })
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        write_json(&self.json, out, style)
    }
}

impl JsonContent for LegacyPathMapping {}

impl Placed for LegacyPathMapping {
    /// `texture_backwards_compatibility_mapping.json`, at the root.
    fn default_path(&self) -> PackPath {
        PackPath::from_static(PATH)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_addon_core::diagnostic::Severity;

    fn parse(bytes: &[u8]) -> Result<Document<LegacyPathMapping>, Rejected> {
        let path = PackPath::from_static(PATH);
        LegacyPathMapping::parse(&path, bytes, &LoadContext::detached())
    }

    #[test]
    fn the_mapping_is_held_as_its_json_and_written_back() {
        let text = br#"// the game's table
{ "textures/old": { "back_compat_texture": "textures/new", "uv": [0, 0], "uv_size": [16, 16], "base_size": [64, 64] } }"#;
        let first = parse(text).unwrap().into_value();
        assert!(first.json.as_object().unwrap().contains_key("textures/old"));
        let written = first.to_json().unwrap();
        let second = parse(written.as_bytes()).unwrap().into_value();
        assert_eq!(first, second);
        assert_eq!(second.to_json().unwrap(), written);
    }

    #[test]
    fn the_mapping_is_placed_at_the_root() {
        let mapping = parse(b"{}").unwrap().into_value();
        assert_eq!(
            mapping.default_path().as_str(),
            "texture_backwards_compatibility_mapping.json"
        );
    }

    #[test]
    fn a_file_that_is_no_json_is_rejected() {
        let rejected = parse(b"{").unwrap_err();
        let finding = rejected.diagnostics.iter().next().unwrap();
        assert!(matches!(
            finding.code,
            DiagnosticCode::MalformedInput { .. }
        ));
        assert_eq!(finding.severity, Severity::Error);
    }
}
