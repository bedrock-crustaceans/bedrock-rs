//! `Content` for the two language files, and the merge keys.

use bedrock_addon_core::pack_identity::{PackPath, PackSide};

use super::read::{codes_json, names_json, read_languages, read_names};
use super::{Language, LanguageNames, Languages};
use crate::SIDES;
use bedrock_addon_core::document::{
    Content, Document, JsonContent, Merge, Placed, Rejected, unversioned,
};
use bedrock_addon_core::json::{EncodeError, WriteStyle, write_json};
use bedrock_addon_core::load::LoadContext;

impl bedrock_addon_core::__private::Sealed for Languages {}
impl bedrock_addon_core::__private::Sealed for LanguageNames {}

/// Keyed by the code: `Merged::origin` names the pack that listed it first.
impl Merge for Languages {
    type Key = Language;
}

impl Content for Languages {
    const NAME: &'static str = "language list";
    const SIDES: &'static [PackSide] = SIDES;

    /// Reads one `texts/languages.json`.
    fn parse(path: &PackPath, bytes: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
        let origin = unversioned::lone(path);
        match read_languages(bytes) {
            Ok((languages, diagnostics)) => {
                Ok(unversioned::document(languages, origin, diagnostics))
            }
            Err(finding) => Err(unversioned::reject(origin, finding)),
        }
    }

    /// Writes the codes as a JSON array; the names belong to `language_names.json`.
    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        write_json(&codes_json(self), out, style)
    }
}

impl JsonContent for Languages {}

impl Placed for Languages {
    /// `texts/languages.json`.
    fn default_path(&self) -> PackPath {
        PackPath::from_static("texts/languages.json")
    }
}

impl Content for LanguageNames {
    const NAME: &'static str = "language names";
    const SIDES: &'static [PackSide] = SIDES;

    /// Reads one `texts/language_names.json`.
    fn parse(path: &PackPath, bytes: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
        let origin = unversioned::lone(path);
        match read_names(bytes) {
            Ok((names, diagnostics)) => Ok(unversioned::document(names, origin, diagnostics)),
            Err(finding) => Err(unversioned::reject(origin, finding)),
        }
    }

    /// Writes the `[code, name]` pairs as a JSON array.
    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        write_json(&names_json(self), out, style)
    }
}

impl JsonContent for LanguageNames {}

impl Placed for LanguageNames {
    /// `texts/language_names.json`.
    fn default_path(&self) -> PackPath {
        PackPath::from_static("texts/language_names.json")
    }
}
