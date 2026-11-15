//! `minecraft:local_light_settings`: the light each block gives off in the deferred renderer.

use indexmap::IndexMap;

use super::read;
use super::value::Rgb8;
use bedrock_addon_core::diagnostic::Code;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::id::BlockId;
use bedrock_addon_core::json::kit::{Expected, Node, Read};
use bedrock_addon_core::schema::{self, FromJson, Schema, read_entries};
use facet::Facet;
use std::collections::HashSet;

/// The local lights of the deferred renderer (`minecraft:local_light_settings`, the one file
/// `local_lighting/local_lighting.json` of each pack), at its only format version (1.21.120).
///
/// JSON: `{"format_version": "1.21.120", "minecraft:local_light_settings": {"<block>":
/// {"light_type"?: "static_light" | "point_light", "light_color"?: colour}, …}}`. Every pack's
/// file counts: per block the highest-priority pack's entry wins. A block without an entry
/// keeps the game's own light.
#[derive(Clone, Debug, Default, PartialEq)]
#[doc(alias = "minecraft:local_light_settings")]
#[doc(alias = "local_lighting")]
pub struct LocalLightSettings {
    /// The light of each block, by block identifier, in document order.
    pub lights: IndexMap<BlockId, LocalLight>,
}

/// The light one block gives off.
#[derive(Clone, Debug, Default, PartialEq, Facet, Schema)]
pub struct LocalLight {
    /// Whether the light is a point light (JSON `light_type`; see
    /// [`LocalLight::light_type`]).
    pub light_type: Option<LightType>,
    /// The light's colour (JSON `light_color`); left out, the block keeps the game's standard
    /// light colour.
    pub light_color: Option<Rgb8>,
    /// Members the game does not know, kept for write-back.
    pub extra: UnknownMembers,
}

/// The kind of a block's local light.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet, Schema)]
#[facet(schema::string_only, rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum LightType {
    /// `"static_light"`: lights its surroundings without a moving highlight; the default.
    #[default]
    StaticLight,
    /// `"point_light"`: a point light with specular highlights and shadows.
    PointLight,
}

/// The entries of the file, each by the block it names. Every entry is read, so every failing
/// one is reported; one failure rejects the file. Of two keys that name one block (`torch`,
/// `minecraft:torch`) the later entry stands, with a `DuplicateIdentifier` information. The
/// game keeps both strings and resolves each to the same block without a word, so the finding
/// is the crate's own.
impl FromJson for LocalLightSettings {
    const EXPECTED: Option<Expected> = Some(Expected::Object);

    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let mut seen = HashSet::new();
        let lights = read_entries::<BlockId, LocalLight>(node, read, |entry, read| {
            let light = LocalLight::from_json(entry, read)?;
            let block = BlockId::parse(entry.path().last_key().unwrap_or_default());
            if !seen.insert(block) {
                read.push(read::info(
                    entry,
                    Code::from(&core_codes::DUPLICATE_IDENTIFIER),
                ));
            }
            Some(light)
        })?;
        Some(Self { lights })
    }
}

bedrock_addon_core::impl_no_molang!(LocalLightSettings);

super::defaults! {
    LocalLight {
        light_type: LightType = LightType::StaticLight;
    }
}

mod codec {
    //! Reading and writing `minecraft:local_light_settings`.

    use super::{LocalLight, LocalLightSettings};
    use crate::read;
    use crate::write::{self, Object, Written};
    use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
    use bedrock_addon_core::versioning::{ParseCx, VersionStruct};

    /// The members of one block's entry.
    const MEMBERS: &[&str] = &["light_type", "light_color"];

    fn write_light(light: &LocalLight, at: &JsonPath) -> Written {
        let mut object = Object::new(at);
        object.opt("light_type", light.light_type.as_ref(), |kind, _| {
            Ok(Value::from(kind.name()))
        })?;
        object.opt("light_color", light.light_color.as_ref(), write::rgb)?;
        object.finish(&light.extra, MEMBERS)
    }

    impl VersionStruct for LocalLightSettings {
        fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
            read::payload(payload, cx, d)
        }

        fn write(&self) -> Result<Value, EncodeError> {
            let mut object = Object::new(&write::payload("minecraft:local_light_settings"));
            for (block, light) in &self.lights {
                object.req(block.as_str(), light, write_light)?;
            }
            object.finish(
                &bedrock_addon_core::document::UnknownMembers::default(),
                &[],
            )
        }
    }
}
