//! Reading and writing [`Biome`] and [`Beta`] as version structs: the payload object, and the
//! document with its envelope.

use super::component::write_components;
use super::parse::{V1_26_30, beta_payload, payload};
use super::{Beta, Biome, Description};
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::document::{ComponentMap, Extra};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value, WriteStyle, write_json};
use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    BETA_VERSION, ParseCx, VersionStruct, Versioned, write_payload,
};

use super::component::BiomeComponent;

/// The payload object: `description`, `components`, then the unknown members.
pub(crate) fn object(
    description: &Description,
    components: &ComponentMap<BiomeComponent>,
    extra: &Extra,
) -> Result<Value, EncodeError> {
    let mut object = facet_value::VObject::new();
    object.insert("description", write_payload(description)?);
    object.insert("components", write_components(components)?);
    for (key, value) in extra.iter() {
        object.insert(key, value.clone());
    }
    Ok(object.into())
}

/// The whole document: `format_version` and the payload.
fn document(version: &str, payload: Value) -> Value {
    let mut root = facet_value::VObject::new();
    root.insert("format_version", Value::from(version));
    root.insert("minecraft:biome", payload);
    root.into()
}

impl Versioned for Biome {
    const VERSION: FormatVersion = V1_26_30;
}

impl VersionStruct for Biome {
    fn parse(slice: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let payload = payload(slice, Self::VERSION, cx, d)?;
        Some(Self {
            description: payload.description,
            components: payload.components,
            extra: payload.extra,
        })
    }

    fn write(&self) -> Result<Value, EncodeError> {
        object(&self.description, &self.components, &self.extra)
    }
}

impl MolangFields for Biome {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visitor.nested("components", &mut self.components);
    }
}

impl Versioned for Beta {
    const VERSION: FormatVersion = BETA_VERSION;
}

impl VersionStruct for Beta {
    fn parse(slice: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let (payload, environment_attributes) = beta_payload(slice, cx, d)?;
        Some(Self {
            description: payload.description,
            components: payload.components,
            environment_attributes,
            extra: payload.extra,
        })
    }

    fn write(&self) -> Result<Value, EncodeError> {
        let mut object = facet_value::VObject::new();
        object.insert("description", write_payload(&self.description)?);
        object.insert("components", write_components(&self.components)?);
        if let Some(attributes) = &self.environment_attributes {
            object.insert("environment_attributes", write_payload(attributes)?);
        }
        for (key, value) in self.extra.iter() {
            object.insert(key, value.clone());
        }
        Ok(object.into())
    }
}

impl MolangFields for Beta {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visitor.nested("components", &mut self.components);
    }
}

impl Biome {
    /// Writes the document, at format 1.26.30, into `out`.
    pub(crate) fn write_into(
        &self,
        out: &mut Vec<u8>,
        style: WriteStyle,
    ) -> Result<(), EncodeError> {
        write_json(
            &document(&Self::VERSION.to_string(), self.write()?),
            out,
            style,
        )
    }
}

impl Beta {
    /// Writes the document, at `"format_version": "beta"`, into `out`.
    pub(crate) fn write_into(
        &self,
        out: &mut Vec<u8>,
        style: WriteStyle,
    ) -> Result<(), EncodeError> {
        write_json(&document("beta", self.write()?), out, style)
    }
}
