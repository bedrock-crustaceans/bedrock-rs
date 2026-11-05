//! Reading and writing [`Biome`] and [`Beta`] as version structs: the payload object (core's
//! `write_envelope` puts it in its document).

use super::component::write_components;
use super::read::{V1_26_30, beta_payload, payload};
use super::{Beta, Biome, Description};
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::document::ComponentMap;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    BETA_VERSION, ParseCx, VersionStruct, Versioned, write_payload,
};

use super::component::BiomeComponent;

/// The payload object: `description` and `components`; the writer adds the document's unknown
/// members after them.
pub(crate) fn object(
    description: &Description,
    components: &ComponentMap<BiomeComponent>,
) -> Result<Value, EncodeError> {
    let mut object = facet_value::VObject::new();
    object.insert("description", write_payload(description)?);
    object.insert("components", write_components(components)?);
    Ok(object.into())
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
        })
    }

    fn write(&self) -> Result<Value, EncodeError> {
        object(&self.description, &self.components)
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
        })
    }

    fn write(&self) -> Result<Value, EncodeError> {
        let mut object = facet_value::VObject::new();
        object.insert("description", write_payload(&self.description)?);
        object.insert("components", write_components(&self.components)?);
        if let Some(attributes) = &self.environment_attributes {
            object.insert("environment_attributes", write_payload(attributes)?);
        }
        Ok(object.into())
    }
}

impl MolangFields for Beta {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visitor.nested("components", &mut self.components);
    }
}
