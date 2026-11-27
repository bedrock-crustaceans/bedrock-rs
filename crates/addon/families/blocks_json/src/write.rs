//! Writing a `blocks.json` file from its members as held.

use facet_value::VObject;

use super::BlocksJson;
use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::json::{EncodeError, Unrepresentable, Value, WriteStyle, write_json};

impl BlocksJson {
    /// The file: every entry in order, with `format_version`, when the file has one, after
    /// `format_version_index` of them (after the last one when there are fewer). An entry
    /// named `format_version` has no place in the file and is not written.
    fn document(&self) -> Result<Value, EncodeError> {
        // Not core's `write_envelope`: the version is one member among the block keys, at
        // the place it was read, not an envelope around one payload.
        let mut object = VObject::new();
        let version_at = self.format_version_index.min(self.entries.len());
        for (index, (name, value)) in self.entries.iter().enumerate() {
            if index == version_at
                && let Some(version) = &self.format_version
            {
                object.insert("format_version", version.clone());
            }
            if name == "format_version" {
                let at = JsonPath::new().child(name);
                return Err(EncodeError::NotRepresentable {
                    at,
                    what: Unrepresentable::Unsupported,
                });
            }
            object.insert(name.as_str(), value.clone());
        }
        if version_at == self.entries.len()
            && let Some(version) = &self.format_version
        {
            object.insert("format_version", version.clone());
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
