//! The appearance components: the billboard a particle is drawn as, its tint and its
//! lighting.

use super::types::{ColorExpr, TintColor};
use crate::codes;
use crate::derived::{KEPT, L0, L1, LEGACY, STRICT, legacy, line, refuse, unread, wrong};
use crate::read::defaults;
use bedrock_addon_core::diagnostic::Code;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::molang::Molang;
use bedrock_addon_core::schema::{self, FromJson, Schema};
use facet::Facet;

/// A Molang field that is 0 when the document does not write it.
const ZERO: Molang = Molang::Const(0.0);
/// A Molang field that is 1 when the document does not write it.
const ONE: Molang = Molang::Const(1.0);
/// Two Molang fields that are 0.
const ZEROS: [Molang; 2] = [ZERO, ZERO];
/// Two Molang fields that are 1.
const ONES: [Molang; 2] = [ONE, ONE];

/// How a billboard's facing direction is chosen (`direction.mode`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet)]
#[non_exhaustive]
#[repr(u8)]
pub enum DirectionMode {
    /// The direction the particle moves in (JSON `"derive_from_velocity"`).
    #[default]
    #[facet(rename = "derive_from_velocity")]
    DeriveFromVelocity,
    /// `custom_direction` (JSON `"custom"`).
    #[facet(rename = "custom")]
    Custom,
}

bedrock_addon_core::impl_no_molang!(DirectionMode);
bedrock_addon_core::impl_schema_leaf!(DirectionMode);

impl FromJson for DirectionMode {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        match node.value().as_string().map(facet_value::VString::as_str) {
            Some("derive_from_velocity") => Some(Self::DeriveFromVelocity),
            Some("custom") => Some(Self::Custom),
            Some(_) if !legacy(read) => {
                refuse(read, node.path(), Code::from(&core_codes::INVALID_VALUE));
                None
            }
            _ => {
                wrong(read, node, Some(&codes::DIRECTION_MODE));
                None
            }
        }
    }
}

/// `minecraft:particle_appearance_billboard/direction`: the direction a billboard that
/// faces a direction uses.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(
    schema::unknown = "report",
    schema::molang = KEPT,
    schema::after_read = direction
)]
pub struct DirectionSettings {
    /// Where the direction comes from (JSON `mode`, required from format 1.20.80).
    #[facet(
        default,
        schema::read_with = DirectionMode::from_json,
        schema::era(STRICT, missing = core_codes::MISSING_REQUIRED_FIELD)
    )]
    pub mode: DirectionMode,
    /// The direction of mode `custom` (JSON `custom_direction`; default `[0, 0, 0]`).
    #[facet(schema::era(LEGACY, molang = L0))]
    pub custom_direction: Option<[Molang; 3]>,
    /// Below this speed the particle keeps its last direction (JSON
    /// `min_speed_threshold`; default 0, and 0.01 in a document below format 1.20.80,
    /// which the upgrade writes out).
    pub min_speed_threshold: Option<f32>,
    /// Unknown members.
    #[facet(skip_deserializing)]
    pub extra: UnknownMembers,
}

defaults!(DirectionSettings {
    custom_direction: [Molang; 3] = "custom_direction", [ZERO, ZERO, ZERO];
    min_speed_threshold: f32 = "min_speed_threshold", 0.0;
});

/// mode `custom` needs `custom_direction`.
fn direction(settings: &mut DirectionSettings, node: &Node<'_>, read: &mut Read<'_>) {
    if settings.mode == DirectionMode::Custom && settings.custom_direction.is_none() {
        line(read, node.path(), &codes::DIRECTION_CUSTOM);
    }
}

/// `uv/flipbook`: an animation that steps through frames of the texture.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "report", schema::molang = KEPT)]
pub struct Flipbook {
    /// The first frame's corner (JSON `base_UV`, required).
    #[facet(
        rename = "base_UV",
        default = ZEROS,
        schema::era(LEGACY, missing = codes::REQUIRED_MISSING, molang = L0),
        schema::era(STRICT, missing = core_codes::MISSING_REQUIRED_FIELD)
    )]
    pub base_uv: [Molang; 2],
    /// A frame's size (JSON `size_UV`; default `[1, 1]`).
    #[facet(rename = "size_UV")]
    pub size_uv: Option<[f32; 2]>,
    /// The step from one frame to the next (JSON `step_UV`; default `[0, 0]`).
    #[facet(rename = "step_UV")]
    pub step_uv: Option<[f32; 2]>,
    /// Frames per second (JSON `frames_per_second`; default 0).
    pub frames_per_second: Option<f32>,
    /// The number of frames (JSON `max_frame`, required).
    #[facet(
        default = ONE,
        schema::era(LEGACY, missing = codes::REQUIRED_MISSING, molang = L1),
        schema::era(STRICT, missing = core_codes::MISSING_REQUIRED_FIELD)
    )]
    pub max_frame: Molang,
    /// The animation takes the particle's whole life (JSON `stretch_to_lifetime`;
    /// default `false`).
    pub stretch_to_lifetime: Option<bool>,
    /// The animation starts again after the last frame (JSON `loop`; default `false`).
    #[facet(rename = "loop")]
    pub looping: Option<bool>,
    /// Unknown members.
    #[facet(skip_deserializing)]
    pub extra: UnknownMembers,
}

defaults!(Flipbook {
    size_uv: [f32; 2] = "size_UV", [1.0, 1.0];
    step_uv: [f32; 2] = "step_UV", [0.0, 0.0];
    frames_per_second: f32 = "frames_per_second", 0.0;
    stretch_to_lifetime: bool = "stretch_to_lifetime", false;
    looping: bool = "loop", false;
});

/// `minecraft:particle_appearance_billboard/uv`: the part of the texture a particle
/// shows.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(
    schema::unknown = "report",
    schema::molang = KEPT,
    schema::after_read = uv
)]
pub struct Uv {
    /// The width the UVs are given in (JSON `texture_width`; default 1).
    pub texture_width: Option<i32>,
    /// The height the UVs are given in (JSON `texture_height`; default 1).
    pub texture_height: Option<i32>,
    /// The corner of the area (JSON `uv`; default `[0, 0]`). Not used with a flipbook.
    #[facet(schema::era(LEGACY, molang = L0))]
    pub uv: Option<[Molang; 2]>,
    /// The size of the area (JSON `uv_size`; default `[1, 1]`). Not used with a
    /// flipbook.
    #[facet(schema::era(LEGACY, molang = L1))]
    pub uv_size: Option<[Molang; 2]>,
    /// A frame animation (JSON `flipbook`).
    #[facet(schema::era(LEGACY, wrong_type = codes::BAD_FLIPBOOK))]
    pub flipbook: Option<Flipbook>,
    /// Unknown members.
    #[facet(skip_deserializing)]
    pub extra: UnknownMembers,
}

defaults!(Uv {
    texture_width: i32 = "texture_width", 1;
    texture_height: i32 = "texture_height", 1;
    uv: [Molang; 2] = "uv", ZEROS;
    uv_size: [Molang; 2] = "uv_size", ONES;
});

/// Whether the object at `node` has the member `key`.
fn has(node: &Node<'_>, key: &str) -> bool {
    node.value()
        .as_object()
        .is_some_and(|object| object.contains_key(key))
}

/// The lines of the UV readers: the 1.20.80 reader's for an area without `uv` or
/// `uv_size`, the legacy parser's for the old spellings of the texture size.
fn uv(uv: &mut Uv, node: &Node<'_>, read: &mut Read<'_>) {
    if legacy(read) {
        for (key, text) in [
            ("texturewidth", &codes::TEXTURE_WIDTH_NAME),
            ("textureheight", &codes::TEXTURE_HEIGHT_NAME),
        ] {
            if has(node, key) {
                line(read, &node.path().child(key), text);
            }
        }
        return;
    }
    if uv.flipbook.is_some() {
        return;
    }
    if uv.uv.is_none() {
        line(read, node.path(), &codes::MISSING_UV);
    }
    if uv.uv_size.is_none() {
        line(read, node.path(), &codes::MISSING_UV_SIZE);
    }
}

/// `minecraft:particle_appearance_billboard`: the particle is a flat quad.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(
    schema::unknown = "report",
    schema::molang = KEPT,
    schema::after_read = billboard
)]
pub struct ParticleAppearanceBillboard {
    /// The quad's half width and half height (JSON `size`, required).
    #[facet(
        default = ONES,
        schema::era(LEGACY, missing = codes::REQUIRED_MISSING, molang = L1),
        schema::era(STRICT, missing = core_codes::MISSING_REQUIRED_FIELD)
    )]
    pub size: [Molang; 2],
    /// How the quad faces the camera (JSON `facing_camera_mode`): `rotate_xyz`,
    /// `rotate_y`, `lookat_xyz`, `lookat_y`, `lookat_direction`, `direction_x`,
    /// `direction_y`, `direction_z`, `emitter_transform_xy`, `emitter_transform_xz` or
    /// `emitter_transform_yz`. The loaders do not check the value. A document below
    /// format 1.20.80 that does not write it means `lookat_xyz`, which the upgrade
    /// writes out.
    pub facing_camera_mode: Option<String>,
    /// The direction for the modes that face one (JSON `direction`).
    #[facet(schema::era(LEGACY, wrong_type = codes::DIRECTION_NOT_OBJECT))]
    pub direction: Option<DirectionSettings>,
    /// The part of the texture shown (JSON `uv`).
    #[facet(schema::era(LEGACY, wrong_type = codes::BAD_UV))]
    pub uv: Option<Uv>,
    /// Unknown members.
    #[facet(skip_deserializing)]
    pub extra: UnknownMembers,
}

/// When the legacy reader of `direction` fails (`direction` is no object, or its mode is
/// `custom` without `custom_direction`) the billboard parser logs "The direction sub section
/// is not properly defined!" and returns before it reaches `uv`, so the game never reads the
/// `uv` and the crate leaves it out and reports it as unread.
fn billboard(billboard: &mut ParticleAppearanceBillboard, node: &Node<'_>, read: &mut Read<'_>) {
    if !legacy(read) {
        return;
    }
    let Some(direction) = node.value().as_object().and_then(|m| m.get("direction")) else {
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
    line(
        read,
        &node.path().child("direction"),
        &codes::DIRECTION_NOT_PROPERLY_DEFINED,
    );
    if has(node, "uv") {
        billboard.uv = None;
        unread(read, &node.path().child("uv"));
    }
}

/// White: what the legacy parser gives a tint without a usable colour.
const WHITE: TintColor = TintColor::Color(ColorExpr::Rgba([ONE, ONE, ONE, ONE]));

/// `minecraft:particle_appearance_tinting`: the particle's colour.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "report")]
pub struct ParticleAppearanceTinting {
    /// The colour, or a gradient (JSON `color`, required from format 1.20.80; white in
    /// an older document that does not write it).
    #[facet(
        default = WHITE,
        schema::read_with = TintColor::from_json,
        schema::era(LEGACY, missing = codes::NO_COLOR),
        schema::era(STRICT, missing = core_codes::MISSING_REQUIRED_FIELD)
    )]
    pub color: TintColor,
    /// Unknown members.
    #[facet(skip_deserializing)]
    pub extra: UnknownMembers,
}

/// `minecraft:particle_appearance_lighting`: the particle is lit by the world. It has no
/// members.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "report")]
pub struct ParticleAppearanceLighting {
    /// Unknown members.
    #[facet(skip_deserializing)]
    pub extra: UnknownMembers,
}
