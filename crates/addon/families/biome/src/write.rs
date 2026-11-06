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
use bedrock_addon_core::versioning::{BETA_VERSION, ParseCx, VersionStruct, Versioned};
use bedrock_addon_shared_biome_env::{beta_object, payload_object};

use super::component::BiomeComponent;

/// The payload object: `description` and `components`; the writer adds the document's unknown
/// members after them.
pub(crate) fn object(
    description: &Description,
    components: &ComponentMap<BiomeComponent>,
) -> Result<Value, EncodeError> {
    Ok(payload_object(description, write_components(components)?)?.into())
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
        beta_object(
            &self.description,
            write_components(&self.components)?,
            self.environment_attributes.as_ref(),
        )
    }
}

impl MolangFields for Beta {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visitor.nested("components", &mut self.components);
    }
}
