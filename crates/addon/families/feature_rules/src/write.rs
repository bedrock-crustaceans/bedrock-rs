//! Writing [`FeatureRule`]: the payload, and the document with its envelope.

use super::parse::parse_rule;
use super::{FLOOR, FeatureRule};
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value, WriteStyle, write_json};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{ParseCx, VersionStruct, Versioned, write_payload};
use facet_value::VObject;

impl Versioned for FeatureRule {
    const VERSION: FormatVersion = FLOOR;
}

impl VersionStruct for FeatureRule {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        parse_rule(payload, cx, d)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        let mut object = VObject::new();
        object.insert("description", write_payload(&self.description)?);
        object.insert("conditions", write_payload(&self.conditions)?);
        if let Some(params) = &self.distribution {
            object.insert("distribution", params.write()?);
        }
        for (key, value) in self.extra.iter() {
            object.insert(key, value.clone());
        }
        Ok(object.into())
    }
}

impl FeatureRule {
    /// The whole document: `format_version` (the rule's own) and the payload.
    pub(crate) fn document(&self) -> Result<Value, EncodeError> {
        let mut object = VObject::new();
        object.insert(
            "format_version",
            self.source_format_version.max(FLOOR).to_string(),
        );
        object.insert("minecraft:feature_rules", self.write()?);
        Ok(object.into())
    }

    /// Writes the document into `out`.
    pub(crate) fn write_into(
        &self,
        out: &mut Vec<u8>,
        style: WriteStyle,
    ) -> Result<(), EncodeError> {
        write_json(&self.document()?, out, style)
    }
}
