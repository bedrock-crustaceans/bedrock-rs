//! `minecraft:shadow_settings`: the pack-wide shadow style.

use super::read;
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

/// The bounds of `texel_size`.
const TEXEL_SIZE: (u16, u16) = (1, 1024);

/// The shadow style of the deferred renderer (`minecraft:shadow_settings`, the one file
/// `shadows/global.json`), at its only format version (1.21.80).
///
/// JSON: `{"format_version": "1.21.80", "minecraft:shadow_settings": {"shadow_style"?:
/// "soft_shadows" | "blocky_shadows", "texel_size"?: 1–1024}}`. A member the document leaves
/// out is not written back. The highest-priority pack whose file loads decides, whole.
#[derive(Clone, Debug, Default, PartialEq, Facet, Schema)]
#[doc(alias = "minecraft:shadow_settings")]
#[doc(alias = "shadows")]
pub struct ShadowSettings {
    /// How shadows are drawn (JSON `shadow_style`; see [`ShadowSettings::shadow_style`]).
    pub shadow_style: Option<ShadowStyle>,
    /// The size of the texel that blocky shadows align to (JSON `texel_size`, 1–1024; see
    /// [`ShadowSettings::texel_size`]).
    #[facet(schema::read_with = read::whole_in(TEXEL_SIZE))]
    pub texel_size: Option<u16>,
    /// Members the game does not know, kept for write-back.
    pub extra: UnknownMembers,
}

/// How the deferred renderer draws shadows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet, Schema)]
#[facet(schema::string_only, rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum ShadowStyle {
    /// `"soft_shadows"`: smooth shadow edges, the game's default.
    #[default]
    SoftShadows,
    /// `"blocky_shadows"`: shadows aligned to texels.
    BlockyShadows,
}

super::defaults! {
    ShadowSettings {
        shadow_style: ShadowStyle = ShadowStyle::SoftShadows;
        texel_size: u16 = 16;
    }
}

mod codec {
    //! Reading and writing `minecraft:shadow_settings`.

    use super::{ShadowSettings, TEXEL_SIZE};
    use crate::read;
    use crate::write::{self, Object};
    use bedrock_addon_core::diagnostic::Diagnostics;
    use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
    use bedrock_addon_core::versioning::{ParseCx, VersionStruct};

    /// The members the struct declares.
    const MEMBERS: &[&str] = &["shadow_style", "texel_size"];

    impl VersionStruct for ShadowSettings {
        fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
            read::payload(payload, cx, d)
        }

        fn write(&self) -> Result<Value, EncodeError> {
            let (min, max) = TEXEL_SIZE;
            let mut object = Object::new(&write::payload("minecraft:shadow_settings"));
            object.opt("shadow_style", self.shadow_style.as_ref(), |style, _| {
                Ok(Value::from(style.name()))
            })?;
            object.opt("texel_size", self.texel_size.as_ref(), |size, at| {
                write::integer_in(i64::from(min), i64::from(max), i64::from(*size), at)
            })?;
            object.finish(&self.extra, MEMBERS)
        }
    }
}
