//! The appearance components: the billboard a particle is drawn as, its tint and its
//! lighting.

use super::types::TintColor;
use crate::read::{Members, Node, Opt, Reader, Writer, object};
use crate::texts;
use bedrock_addon_core::diagnostic::DiagnosticCode;
use bedrock_addon_core::json::{EncodeError, Value};
use bedrock_addon_core::molang::Molang;

/// How a billboard's facing direction is chosen (`direction.mode`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DirectionMode {
    /// The direction the particle moves in (JSON `"derive_from_velocity"`).
    #[default]
    DeriveFromVelocity,
    /// `custom_direction` (JSON `"custom"`).
    Custom,
}

impl Node for DirectionMode {
    fn read(value: &Value, r: &mut Reader<'_>, _: Opt) -> Option<Self> {
        match value.as_string().map(facet_value::VString::as_str) {
            Some("derive_from_velocity") => Some(Self::DeriveFromVelocity),
            Some("custom") => Some(Self::Custom),
            Some(_) if !r.legacy() => {
                r.error(DiagnosticCode::InvalidValue { text: None });
                None
            }
            _ => {
                r.wrong_type(value, Some(texts::DIRECTION_MODE));
                None
            }
        }
    }

    fn fallback(_: Opt) -> Self {
        Self::DeriveFromVelocity
    }

    fn write(&self, _: &mut Writer) -> Result<Value, EncodeError> {
        Ok(Value::from(match self {
            Self::DeriveFromVelocity => "derive_from_velocity",
            Self::Custom => "custom",
        }))
    }
}

object! {
    /// `minecraft:particle_appearance_billboard/direction`: the direction a billboard that
    /// faces a direction uses.
    pub struct DirectionSettings finish = direction {
        /// Where the direction comes from (JSON `mode`, required from format 1.20.80).
        req mode: DirectionMode = "mode", Opt::NONE;
        /// The direction of mode `custom` (JSON `custom_direction`; default `[0, 0, 0]`).
        opt custom_direction: [Molang; 3] = "custom_direction", Opt::NONE,
            default [Molang::Const(0.0), Molang::Const(0.0), Molang::Const(0.0)];
        /// Below this speed the particle keeps its last direction (JSON
        /// `min_speed_threshold`; default 0, and 0.01 in a document below format 1.20.80,
        /// which the upgrade writes out).
        opt min_speed_threshold: f32 = "min_speed_threshold", Opt::with_default(0.01), default 0.0;
    }
}

/// mode `custom` needs `custom_direction`.
fn direction(settings: &mut DirectionSettings, _: Members<'_>, r: &mut Reader<'_>) {
    if settings.mode == DirectionMode::Custom && settings.custom_direction.is_none() {
        r.warn(texts::DIRECTION_CUSTOM);
    }
}

object! {
    /// `uv/flipbook`: an animation that steps through frames of the texture.
    pub struct Flipbook {
        /// The first frame's corner (JSON `base_UV`, required).
        req base_uv: [Molang; 2] = "base_UV", Opt::required(0.0);
        /// A frame's size (JSON `size_UV`; default `[1, 1]`).
        opt size_uv: [f32; 2] = "size_UV", Opt::with_default(1.0), default [1.0, 1.0];
        /// The step from one frame to the next (JSON `step_UV`; default `[0, 0]`).
        opt step_uv: [f32; 2] = "step_UV", Opt::NONE, default [0.0, 0.0];
        /// Frames per second (JSON `frames_per_second`; default 0).
        opt frames_per_second: f32 = "frames_per_second", Opt::NONE, default 0.0;
        /// The number of frames (JSON `max_frame`, required).
        req max_frame: Molang = "max_frame", Opt::required(1.0);
        /// The animation takes the particle's whole life (JSON `stretch_to_lifetime`;
        /// default `false`).
        opt stretch_to_lifetime: bool = "stretch_to_lifetime", Opt::NONE, default false;
        /// The animation starts again after the last frame (JSON `loop`; default `false`).
        opt looping: bool = "loop", Opt::NONE, default false;
    }
}

object! {
    /// `minecraft:particle_appearance_billboard/uv`: the part of the texture a particle
    /// shows.
    pub struct Uv finish = uv {
        /// The width the UVs are given in (JSON `texture_width`; default 1).
        opt texture_width: i32 = "texture_width", Opt::with_default(1.0), default 1;
        /// The height the UVs are given in (JSON `texture_height`; default 1).
        opt texture_height: i32 = "texture_height", Opt::with_default(1.0), default 1;
        /// The corner of the area (JSON `uv`; default `[0, 0]`). Not used with a flipbook.
        opt uv: [Molang; 2] = "uv", Opt::NONE,
            default [Molang::Const(0.0), Molang::Const(0.0)];
        /// The size of the area (JSON `uv_size`; default `[1, 1]`). Not used with a
        /// flipbook.
        opt uv_size: [Molang; 2] = "uv_size", Opt::with_default(1.0),
            default [Molang::Const(1.0), Molang::Const(1.0)];
        /// A frame animation (JSON `flipbook`).
        opt flipbook: Flipbook = "flipbook", Opt::NONE.wrong(texts::BAD_FLIPBOOK);
    }
}

/// The lines of the UV readers: the 1.20.80 reader's for an area without `uv` or
/// `uv_size`, the legacy parser's for the old spellings of the texture size.
fn uv(uv: &mut Uv, m: Members<'_>, r: &mut Reader<'_>) {
    if r.legacy() {
        for (key, text) in [
            ("texturewidth", texts::TEXTURE_WIDTH_NAME),
            ("textureheight", texts::TEXTURE_HEIGHT_NAME),
        ] {
            if m.get(key).is_some() {
                r.key(key, |r| r.warn(text));
            }
        }
        return;
    }
    if uv.flipbook.is_some() {
        return;
    }
    if uv.uv.is_none() {
        r.warn(texts::MISSING_UV);
    }
    if uv.uv_size.is_none() {
        r.warn(texts::MISSING_UV_SIZE);
    }
}

object! {
    /// `minecraft:particle_appearance_billboard`: the particle is a flat quad.
    pub struct ParticleAppearanceBillboard finish = billboard {
        /// The quad's half width and half height (JSON `size`, required).
        req size: [Molang; 2] = "size", Opt::required(1.0);
        /// How the quad faces the camera (JSON `facing_camera_mode`): `rotate_xyz`,
        /// `rotate_y`, `lookat_xyz`, `lookat_y`, `lookat_direction`, `direction_x`,
        /// `direction_y`, `direction_z`, `emitter_transform_xy`, `emitter_transform_xz` or
        /// `emitter_transform_yz`. The loaders do not check the value. A document below
        /// format 1.20.80 that does not write it means `lookat_xyz`, which the upgrade
        /// writes out.
        opt facing_camera_mode: String = "facing_camera_mode", Opt::NONE;
        /// The direction for the modes that face one (JSON `direction`).
        opt direction: DirectionSettings = "direction",
            Opt::NONE.wrong(texts::DIRECTION_NOT_OBJECT);
        /// The part of the texture shown (JSON `uv`).
        opt uv: Uv = "uv", Opt::NONE.wrong(texts::BAD_UV);
    }
}

/// When the legacy reader of `direction` fails (`direction` is no object, or its mode is
/// `custom` without `custom_direction`) the billboard parser logs "The direction sub section
/// is not properly defined!" and returns before it reaches `uv`, so the game never reads the
/// `uv` and the crate leaves it out and reports it as unread.
fn billboard(billboard: &mut ParticleAppearanceBillboard, m: Members<'_>, r: &mut Reader<'_>) {
    if !r.legacy() {
        return;
    }
    let Some(direction) = m.get("direction") else {
        return;
    };
    let failed = direction.as_object().is_none_or(|settings| {
        settings
            .get("mode")
            .and_then(Value::as_string)
            .is_some_and(|mode| mode.as_str() == "custom")
            && settings.get("custom_direction").is_none()
    });
    if !failed {
        return;
    }
    // The game writes this line to the content log at the error level.
    r.key("direction", |r| {
        r.content_error(texts::DIRECTION_NOT_PROPERLY_DEFINED);
    });
    if m.get("uv").is_some() {
        billboard.uv = None;
        r.unread("uv");
    }
}

object! {
    /// `minecraft:particle_appearance_tinting`: the particle's colour.
    pub struct ParticleAppearanceTinting {
        /// The colour, or a gradient (JSON `color`, required from format 1.20.80; white in
        /// an older document that does not write it).
        req color: TintColor = "color", Opt::NONE.missing(texts::NO_COLOR);
    }
}

object! {
    /// `minecraft:particle_appearance_lighting`: the particle is lit by the world. It has no
    /// members.
    pub struct ParticleAppearanceLighting {}
}
