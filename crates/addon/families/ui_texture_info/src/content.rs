//! `Content` of a side file.

use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::{PackPath, PackSide};

use super::{UiTextureInfo, write};
use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath, Severity};
use bedrock_addon_core::document::{Content, Document, JsonContent, Origin, Rejected};
use bedrock_addon_core::json::{EncodeError, Value, WriteStyle};
use bedrock_addon_support::fixed_path::{document, finding, json_text, lone, malformed, reject};
use bedrock_addon_support::jsoncpp::read;

impl bedrock_addon_core::__private::Sealed for UiTextureInfo {}

/// The root object of a file, or the rejection.
fn root_object(origin: &Origin, bytes: &[u8]) -> Result<Value, Rejected> {
    let rejection = |message: String| {
        let code = malformed(message, None);
        reject(
            origin.clone(),
            finding(code, Severity::Error, JsonPath::new()),
        )
    };
    let root = read(bytes).map_err(|error| rejection(error.to_string()))?;
    if root.is_object() {
        Ok(root)
    } else {
        Err(rejection("the root is not an object".to_owned()))
    }
}

impl Content for UiTextureInfo {
    const NAME: &'static str = "ui texture info";
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// Reads one `textures/ui/<image>.json`. A file that is no JSON object is rejected (the
    /// game ignores the result of its parse and reads nothing from it).
    fn parse(path: &PackPath, bytes: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
        let origin = lone(path);
        // A file that is no JSON object has nothing a writer could give back: rejected.
        let mut root = root_object(&origin, bytes)?;
        let mut diagnostics = Diagnostics::default();
        // A number no double holds is read as the largest finite one, with a warning.
        bedrock_addon_core::json::clamp_non_finite_warned(
            &mut root,
            None,
            &JsonPath::new(),
            &mut diagnostics,
        );
        let info = Self::from_value(&root, &mut diagnostics);
        Ok(document(info, origin, diagnostics))
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        json_text(&write::texture_info_value(self), out, style)
    }
}

impl JsonContent for UiTextureInfo {}
