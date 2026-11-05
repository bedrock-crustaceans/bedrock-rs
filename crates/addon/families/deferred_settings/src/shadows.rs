//! `minecraft:shadow_settings`: the pack-wide shadow style.

use bedrock_addon_core::document::UnknownMembers;

/// The shadow style of the deferred renderer (`minecraft:shadow_settings`, the one file
/// `shadows/global.json`), at its only format version (1.21.80).
///
/// JSON: `{"format_version": "1.21.80", "minecraft:shadow_settings": {"shadow_style"?:
/// "soft_shadows" | "blocky_shadows", "texel_size"?: 1–1024}}`. A member the document leaves
/// out is not written back. The highest-priority pack whose file loads decides, whole.
#[derive(Clone, Debug, Default, PartialEq)]
#[doc(alias = "minecraft:shadow_settings")]
#[doc(alias = "shadows")]
pub struct ShadowSettings {
    /// How shadows are drawn (JSON `shadow_style`; see [`ShadowSettings::shadow_style`]).
    pub shadow_style: Option<ShadowStyle>,
    /// The size of the texel that blocky shadows align to (JSON `texel_size`, 1–1024; see
    /// [`ShadowSettings::texel_size`]).
    pub texel_size: Option<u16>,
    /// Members the game does not know, kept for write-back.
    pub extra: UnknownMembers,
}

/// How the deferred renderer draws shadows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ShadowStyle {
    /// `"soft_shadows"`: smooth shadow edges, the game's default.
    #[default]
    SoftShadows,
    /// `"blocky_shadows"`: shadows aligned to texels.
    BlockyShadows,
}

impl ShadowStyle {
    /// The JSON names, in the order the game lists them.
    pub(crate) const NAMES: &'static [(&'static str, Self)] = &[
        ("soft_shadows", Self::SoftShadows),
        ("blocky_shadows", Self::BlockyShadows),
    ];
}

super::defaults! {
    ShadowSettings {
        shadow_style: ShadowStyle = ShadowStyle::SoftShadows;
        texel_size: u16 = 16;
    }
}

mod codec {
    //! Reading and writing `minecraft:shadow_settings`.

    use super::{ShadowSettings, ShadowStyle};
    use crate::read::{self, members};
    use crate::write::{self, Object};
    use bedrock_addon_core::diagnostic::Diagnostics;
    use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
    use bedrock_addon_core::versioning::{ParseCx, VersionStruct};

    /// The bounds of `texel_size`.
    const TEXEL_SIZE: (u16, u16) = (1, 1024);
    /// The members the struct declares.
    const MEMBERS: &[&str] = &["shadow_style", "texel_size"];

    impl VersionStruct for ShadowSettings {
        fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
            let (min, max) = TEXEL_SIZE;
            let mut m = members(&read::payload(payload, cx), d).ok()?;
            let shadow_style = m.opt("shadow_style", d, |n, d| {
                read::choice(ShadowStyle::NAMES, n, d)
            });
            let texel_size = m.opt("texel_size", d, |n, d| {
                read::integer_in(i32::from(min), i32::from(max), n, d)
            });
            let extra = m.finish(&[], d);
            Some(Self {
                shadow_style: shadow_style.ok()?,
                texel_size: texel_size.ok()?.and_then(|size| u16::try_from(size).ok()),
                extra,
            })
        }

        fn write(&self) -> Result<Value, EncodeError> {
            let (min, max) = TEXEL_SIZE;
            let mut object = Object::new(&write::payload("minecraft:shadow_settings"));
            object.opt("shadow_style", self.shadow_style.as_ref(), |style, at| {
                write::choice(ShadowStyle::NAMES, *style, at)
            })?;
            object.opt("texel_size", self.texel_size.as_ref(), |size, at| {
                write::integer_in(i64::from(min), i64::from(max), i64::from(*size), at)
            })?;
            object.finish(&self.extra, MEMBERS)
        }
    }
}
