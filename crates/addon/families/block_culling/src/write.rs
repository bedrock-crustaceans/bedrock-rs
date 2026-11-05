//! Writing a culling-rules document with its envelope.

use facet_value::{VArray, VObject};

use super::{CullingRules, Description, GeometryPart, Rule};
use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
use bedrock_addon_core::json::{
    EncodeError, JsonSlice, Unrepresentable, Value, WriteStyle, write_json,
};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{ParseCx, VersionStruct, Versioned};

use super::culling::{Slice, read};

/// The payload of a culling-rules document: only the members the value holds are written.
pub(super) fn payload(description: &Description, rules: &[Rule]) -> Value {
    let mut object = VObject::new();
    let mut described = VObject::new();
    described.insert("identifier", description.identifier.as_str());
    object.insert("description", described);
    let mut array = VArray::new();
    for rule in rules {
        array.push(Value::from(self::rule(rule)));
    }
    object.insert("rules", array);
    object.into()
}

/// One rule, in vanilla's member order.
fn rule(rule: &Rule) -> VObject {
    let mut object = VObject::new();
    if let Some(condition) = rule.condition {
        object.insert("condition", condition.name());
    }
    if let Some(flag) = rule.cull_against_full_and_opaque {
        object.insert("cull_against_full_and_opaque", flag);
    }
    object.insert("geometry_part", geometry_part(&rule.geometry_part));
    object.insert("direction", rule.direction.name());
    object
}

/// `geometry_part`.
fn geometry_part(part: &GeometryPart) -> VObject {
    let mut object = VObject::new();
    object.insert("bone", part.bone.as_str());
    if let Some(cube) = part.cube {
        object.insert("cube", cube);
    }
    if let Some(face) = part.face {
        object.insert("face", face.name());
    }
    object
}

impl Versioned for CullingRules {
    const VERSION: FormatVersion = CullingRules::FORMAT_VERSION;
}

impl VersionStruct for CullingRules {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let payload = read(payload, Slice::V1_21_80, &cx.at, d)?;
        Some(Self {
            description: payload.description,
            rules: payload.rules,
            source_format_version: cx.original_version,
        })
    }

    fn write(&self) -> Result<Value, EncodeError> {
        Ok(payload(&self.description, &self.rules))
    }
}

impl CullingRules {
    /// The whole document at format 1.21.80, the only version that reads every member; a
    /// value the loader would refuse to read back (a cube index below 0) is not written.
    fn document(&self) -> Result<Value, EncodeError> {
        let payload_at = || JsonPath::new().child("minecraft:block_culling_rules");
        // An identifier or a rule the loader refuses (in every pack: the `minecraft` namespace
        // is a base-game pack's to use, so it is written).
        if super::validate::identifier(self.description.identifier.as_str(), true).is_err() {
            let at = payload_at().child("description").child("identifier");
            return Err(EncodeError::NotRepresentable {
                at,
                what: Unrepresentable::Unsupported,
            });
        }
        if let Some(problem) = super::validate::rules(&self.rules).first() {
            let at = payload_at()
                .child("rules")
                .index(problem.index)
                .child("geometry_part");
            return Err(EncodeError::NotRepresentable {
                at,
                what: Unrepresentable::Unsupported,
            });
        }
        for (index, rule) in self.rules.iter().enumerate() {
            if rule.geometry_part.cube.is_some_and(|cube| cube < 0) {
                let at = JsonPath::new()
                    .child("minecraft:block_culling_rules")
                    .child("rules")
                    .index(index)
                    .child("geometry_part")
                    .child("cube");
                return Err(EncodeError::NotRepresentable {
                    at,
                    what: Unrepresentable::Unsupported,
                });
            }
        }
        let mut object = VObject::new();
        object.insert("format_version", Self::FORMAT_VERSION.to_string().as_str());
        object.insert("minecraft:block_culling_rules", self.write()?);
        Ok(object.into())
    }

    /// Writes the document into `out`.
    pub(super) fn write_into(
        &self,
        out: &mut Vec<u8>,
        style: WriteStyle,
    ) -> Result<(), EncodeError> {
        write_json(&self.document()?, out, style)
    }
}
