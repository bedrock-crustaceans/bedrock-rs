//! Writing a `blocks.json` file from its members as held.

use facet_value::VObject;

use super::BlocksJson;
use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::json::{EncodeError, Unrepresentable, Value, WriteStyle, write_json};

impl BlocksJson {
    /// The file: `format_version` first, when the file has one, as vanilla's files have it,
    /// then every entry in order. An entry named `format_version` has no place in the file and
    /// is not written.
    fn document(&self) -> Result<Value, EncodeError> {
        // Not core's `write_envelope`: the version is one member among the block keys, not an
        // envelope around one payload.
        let mut object = VObject::new();
        if let Some(version) = &self.format_version {
            object.insert("format_version", version.clone());
        }
        for (name, value) in &self.entries {
            if name == "format_version" {
                let at = JsonPath::new().child(name);
                return Err(EncodeError::NotRepresentable {
                    at,
                    what: Unrepresentable::Unsupported,
                });
            }
            object.insert(name.as_str(), value.clone());
        }
        Ok(object.into())
    }

    /// Writes the file into `out`.
    pub(super) fn write_into(
        &self,
        out: &mut Vec<u8>,
        style: WriteStyle,
    ) -> Result<(), EncodeError> {
        write_json(&self.document()?, out, style)
    }
}
