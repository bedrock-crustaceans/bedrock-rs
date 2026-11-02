//! The texture set of format 1.16.100: the first schema, without
//! `metalness_emissive_roughness_subsurface`.

use super::super::TextureSet as Latest;
use super::super::set::{LayerValue, Rgb, Rgba, TextureName};
use super::super::set_read::{Schema, V1_16_100};
use super::super::set_write::{parse_layers, write_layers};
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{ParseCx, Upgrade, UpgradeCx, VersionStruct, Versioned};

/// A texture set as format 1.16.100 writes it: `color`, `normal`, `heightmap` and
/// `metalness_emissive_roughness`. Any other member fails the client's schema.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextureSet {
    /// JSON `color`.
    pub color: Option<LayerValue<Rgba>>,
    /// JSON `normal`.
    pub normal: Option<TextureName>,
    /// JSON `heightmap`.
    pub heightmap: Option<TextureName>,
    /// JSON `metalness_emissive_roughness`.
    pub metalness_emissive_roughness: Option<LayerValue<Rgb>>,
}

impl Versioned for TextureSet {
    const VERSION: FormatVersion = V1_16_100;
}

impl VersionStruct for TextureSet {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let set = parse_layers(payload, Schema::V1_16_100, &cx.at, d)?;
        Some(Self {
            color: set.color,
            normal: set.normal,
            heightmap: set.heightmap,
            metalness_emissive_roughness: set.metalness_emissive_roughness,
        })
    }

    fn write(&self) -> Result<Value, EncodeError> {
        Ok(write_layers(&self.clone().widen()))
    }
}

impl TextureSet {
    /// The same layers as the newest document holds them.
    fn widen(self) -> Latest {
        Latest {
            color: self.color,
            normal: self.normal,
            heightmap: self.heightmap,
            metalness_emissive_roughness: self.metalness_emissive_roughness,
            metalness_emissive_roughness_subsurface: None,
        }
    }
}

/// The step `v1_16_100 → v1_21_30`: the four layers move unchanged and
/// `metalness_emissive_roughness_subsurface` is absent.
impl Upgrade for TextureSet {
    type Next = Latest;

    fn upgrade(self, _: &mut UpgradeCx<'_>) -> Latest {
        self.widen()
    }
}
