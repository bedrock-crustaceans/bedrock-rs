//! Format 1.26.0: `minecraft:village_type` is new and `minecraft:replace_biomes` accepts the
//! Nether.

use crate::component::BiomeComponent;
use crate::read::payload;
use crate::write::object;
use crate::{Biome as Latest, Description};
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::document::{ComponentMap, Extra};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::known;
use bedrock_addon_core::versioning::{ParseCx, Upgrade, UpgradeCx, VersionStruct, Versioned};

/// A biome of format `[1.26.0, 1.26.30)`: the payload of `minecraft:biome`.
#[derive(Clone, Debug, PartialEq)]
pub struct Biome {
    /// The biome's name (JSON `description`).
    pub description: Description,
    /// The components (JSON `components`).
    pub components: ComponentMap<BiomeComponent>,
    /// Unknown members of `minecraft:biome`.
    pub extra: Extra,
}

impl Versioned for Biome {
    const VERSION: FormatVersion = known::V1_26_0;
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

/// The step to 1.26.30: every surface-builder alternative is copied into the same alternative
/// of the wider 1.26.30 variant, so every component moves unchanged.
impl Upgrade for Biome {
    type Next = Latest;

    fn upgrade(self, _: &mut UpgradeCx<'_>) -> Latest {
        Latest {
            description: self.description,
            components: self.components,
            extra: self.extra,
        }
    }
}
