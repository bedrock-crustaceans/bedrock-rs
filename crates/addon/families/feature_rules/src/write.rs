//! Writing [`FeatureRule`]: the payload (the envelope is core's `write_envelope`).

use super::read::parse_rule;
use super::{FLOOR, FeatureRule};
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
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
