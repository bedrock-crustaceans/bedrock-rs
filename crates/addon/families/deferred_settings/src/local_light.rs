//! `minecraft:local_light_settings`: the light each block gives off in the deferred renderer.

use indexmap::IndexMap;

use super::value::Rgb8;
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::id::BlockId;

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
#[derive(Clone, Debug, Default, PartialEq)]
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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum LightType {
    /// `"static_light"`: lights its surroundings without a moving highlight; the default.
    #[default]
    StaticLight,
    /// `"point_light"`: a point light with specular highlights and shadows.
    PointLight,
}

impl LightType {
    /// The JSON names, in the order the game lists them.
    pub(crate) const NAMES: &'static [(&'static str, Self)] = &[
        ("static_light", Self::StaticLight),
        ("point_light", Self::PointLight),
    ];
}

super::defaults! {
    LocalLight {
        light_type: LightType = LightType::StaticLight;
    }
}

mod codec {
    //! Reading and writing `minecraft:local_light_settings`.

    use indexmap::map::Entry;

    use super::{LightType, LocalLight, LocalLightSettings};
    use crate::read::{self, Node, Read, members};
    use crate::write::{self, Object, Written};
    use bedrock_addon_core::diagnostic::{DiagnosticCode, Diagnostics, JsonPath, Severity};
    use bedrock_addon_core::id::BlockId;
    use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
    use bedrock_addon_core::versioning::{ParseCx, VersionStruct};

    /// The members of one block's entry.
    const MEMBERS: &[&str] = &["light_type", "light_color"];

    fn light(node: &Node<'_>, d: &mut Diagnostics) -> Read<LocalLight> {
        let mut m = members(node, d)?;
        let light_type = m.opt("light_type", d, |n, d| read::choice(LightType::NAMES, n, d));
        let light_color = m.opt("light_color", d, read::rgb);
        let extra = m.finish(&[], d);
        Ok(LocalLight {
            light_type: light_type?,
            light_color: light_color?,
            extra,
        })
    }

    fn write_light(light: &LocalLight, at: &JsonPath) -> Written {
        let mut object = Object::new(at);
        object.opt("light_type", light.light_type.as_ref(), |kind, at| {
            write::choice(LightType::NAMES, *kind, at)
        })?;
        object.opt("light_color", light.light_color.as_ref(), write::rgb)?;
        object.finish(&light.extra, MEMBERS)
    }

    impl VersionStruct for LocalLightSettings {
        /// Every entry is read, so every failing one is reported; one failure rejects the
        /// file. Of two keys that name one block (`torch`, `minecraft:torch`) the later
        /// entry stands, with a `DuplicateIdentifier` information. The game keeps both strings
        /// and resolves each to the same block without a word, so the finding is the crate's
        /// own.
        fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
            let m = members(&read::payload(payload, cx), d).ok()?;
            let mut lights = indexmap::IndexMap::new();
            let mut failed = false;
            for (key, node) in m.rest() {
                let Ok(light) = light(&node, d) else {
                    failed = true;
                    continue;
                };
                match lights.entry(BlockId::parse(key)) {
                    Entry::Vacant(entry) => {
                        entry.insert(light);
                    }
                    Entry::Occupied(mut entry) => {
                        d.push(node.finding(
                            DiagnosticCode::DuplicateIdentifier { text: None },
                            Severity::Info,
                        ));
                        entry.insert(light);
                    }
                }
            }
            (!failed).then_some(Self { lights })
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
