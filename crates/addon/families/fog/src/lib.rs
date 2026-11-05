//! Fog settings: `fogs/*.json` of a resource pack (`minecraft:fog_settings`). The game has two
//! parsers behind one entry point: a strict schema with three versioned roots below
//! 1.26.20 (`history`), and a cereal loader for exactly 1.26.20, whose shape is the
//! newest struct here.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod history;

mod command;
mod gates;
mod jsonutil;
mod load;
mod new;
mod read;
mod rules;
mod texts;
mod write;

#[cfg(test)]
mod tests;

use bedrock_addon_core::document::Extra;
use bedrock_addon_core::id::FogId;
use bedrock_addon_core::version::known;

pub(crate) use jsonutil::{Era, read_document};
#[doc(hidden)]
pub use load::{load_registry, upgrade_document};
pub(crate) use write::{color_ju, float, norm_ju, object_with};

pub use command::{FOG_STACK_LIMIT, FogCommandError, FogStack, is_valid_fog_identifier};
pub use gates::{FOG_FIELD_GATES, FogFieldGate};

/// The `format_version` of the newest fog struct, and the only one it is read at.
pub(crate) const LATEST: bedrock_addon_core::version::FormatVersion = known::V1_26_20;
/// The oldest `format_version` the game reads fog settings at.
pub(crate) const FLOOR: bedrock_addon_core::version::FormatVersion = known::V1_16_100;

/// Fog settings (`minecraft:fog_settings`): how far and in which colour fog is drawn in air,
/// weather and the fluids, and the volumetric fog of the deferred renderer.
///
/// JSON: `{"format_version": "1.26.20", "minecraft:fog_settings": {"description":
/// {"identifier"}, "distance"?: {…}, "volumetric"?: {…}}}`. Only `description.identifier` is
/// required; an absent part is not set by this definition and the game takes it from the next
/// fog on the player's stack. A member the schema does not know is kept in `extra` and
/// reported; the game ignores it. A file from 1.16.100 up to 1.26.20 is upgraded when it loads
/// ([`history`] has the older shapes).
///
/// Written, the settings are a file of 1.26.20, which the game holds to rules the older
/// formats do not have (`fog_start` at most `fog_end`, a density with `uniform` or with
/// both heights, …). Settings that break one of them, as those of an older file may, are
/// written in the older schema instead, at the oldest format version that has their
/// members (1.16.100, 1.21.0 or 1.21.90), where they read back as themselves with the
/// rule's warning. Where the older schema cannot hold them either (a distance entry
/// without one of its four members, an unknown member, a keyframed value, an identifier
/// without a namespace), `to_json` is `NotRepresentable` at the entry that breaks the rule.
#[derive(Clone, Debug, PartialEq)]
#[doc(alias = "minecraft:fog_settings")]
#[doc(alias = "fogs")]
pub struct FogSettings {
    /// `description`: the definition's name.
    pub description: FogDescription,
    /// `distance`: the fog of the classic renderer, per medium.
    pub distance: Option<FogDistanceCollection>,
    /// `volumetric`: the volumetric fog of the deferred renderer.
    pub volumetric: Option<FogVolumetricCollection>,
    /// The members the schema does not know, as written.
    pub extra: Extra,
}

/// `description` of [`FogSettings`].
#[derive(Clone, Debug, PartialEq)]
pub struct FogDescription {
    /// `identifier`: any non-empty text; compared as written, case included. The namespace
    /// `minecraft` (the text before the first `:`, or the whole text, lower-cased) is the base
    /// game's: no other pack may use it.
    pub identifier: FogId,
    /// The members the schema does not know.
    pub extra: Extra,
}

/// `distance`: one setting per medium the camera can be in.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FogDistanceCollection {
    /// `air`.
    pub air: Option<FogDistanceSetting>,
    /// `weather`: in rain or snow.
    pub weather: Option<FogDistanceSetting>,
    /// `water`; the only medium with a `transition_fog`.
    pub water: Option<FogWaterDistanceSetting>,
    /// `lava`.
    pub lava: Option<FogDistanceSetting>,
    /// `lava_resistance`: in lava with fire resistance.
    pub lava_resistance: Option<FogDistanceSetting>,
    /// `powder_snow`.
    pub powder_snow: Option<FogDistanceSetting>,
    /// The members the schema does not know.
    pub extra: Extra,
}

/// One distance setting. Every member is optional; a member a present setting leaves out is 0
/// (`fog_start`, `fog_end`) or `fixed` (`render_distance_type`), which the accessors give.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FogDistanceSetting {
    /// `fog_start`: where the fog begins; at most `fog_end`.
    pub fog_start: Option<f32>,
    /// `fog_end`: where the fog is opaque; at least 0.
    pub fog_end: Option<f32>,
    /// `render_distance_type`: what the two distances are measured in.
    pub render_distance_type: Option<DistanceType>,
    /// `fog_color`.
    pub fog_color: Option<FogColor>,
    /// The members the schema does not know (a `transition_fog` outside `water` is one).
    pub extra: Extra,
}

/// The `water` distance setting: the members of [`FogDistanceSetting`] and `transition_fog`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FogWaterDistanceSetting {
    /// `fog_start`: where the fog begins; at most `fog_end`.
    pub fog_start: Option<f32>,
    /// `fog_end`: where the fog is opaque; at least 0.
    pub fog_end: Option<f32>,
    /// `render_distance_type`: what the two distances are measured in.
    pub render_distance_type: Option<DistanceType>,
    /// `fog_color`.
    pub fog_color: Option<FogColor>,
    /// `transition_fog`: how the fog changes while the camera stays under water.
    pub transition_fog: Option<FogTransitionSetting>,
    /// The members the schema does not know.
    pub extra: Extra,
}

/// `transition_fog` of the water setting: from `init_fog` towards the setting itself over time.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FogTransitionSetting {
    /// `init_fog`: the fog on entering the water.
    pub init_fog: Option<FogDistanceSetting>,
    /// `min_percent`: 0 to 1.
    pub min_percent: Option<f32>,
    /// `mid_seconds`: at least 0, and below `max_seconds` unless both are 0.
    pub mid_seconds: Option<f32>,
    /// `mid_percent`: 0 to 1.
    pub mid_percent: Option<f32>,
    /// `max_seconds`: at least 0.
    pub max_seconds: Option<f32>,
    /// The members the schema does not know.
    pub extra: Extra,
}

/// `volumetric`: the fog of the deferred renderer.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FogVolumetricCollection {
    /// `density`: how dense the fog is, per medium.
    pub density: Option<FogVolumetricDensityCollection>,
    /// `media_coefficients`: how each medium scatters and absorbs light.
    pub media_coefficients: Option<FogVolumetricMediaCoeffsCollection>,
    /// `henyey_greenstein_g`: the scattering direction, per medium.
    pub henyey_greenstein_g: Option<FogVolumetricHenyeyGreensteinGCollection>,
    /// The members the schema does not know.
    pub extra: Extra,
}

/// `volumetric.density`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FogVolumetricDensityCollection {
    /// `air`.
    pub air: Option<FogVolumetricDensitySetting>,
    /// `weather`.
    pub weather: Option<FogVolumetricDensitySetting>,
    /// `water`.
    pub water: Option<FogVolumetricDensitySetting>,
    /// `lava`.
    pub lava: Option<FogVolumetricDensitySetting>,
    /// `lava_resistance`.
    pub lava_resistance: Option<FogVolumetricDensitySetting>,
    /// The members the schema does not know.
    pub extra: Extra,
}

/// One density setting. Either `uniform` is `true` and neither height is written, or both
/// heights are written with `zero_density_height` at or above `max_density_height`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FogVolumetricDensitySetting {
    /// `max_density`: 0 to 1.
    pub max_density: Option<Keyframed<f32>>,
    /// `uniform`: the same density at every height.
    pub uniform: Option<bool>,
    /// `zero_density_height`: the height at which the fog ends, −64 to 320.
    pub zero_density_height: Option<Keyframed<f32>>,
    /// `max_density_height`: the height below which the fog is at `max_density`, −64 to 320.
    pub max_density_height: Option<Keyframed<f32>>,
    /// The members the schema does not know.
    pub extra: Extra,
}

/// `volumetric.media_coefficients`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FogVolumetricMediaCoeffsCollection {
    /// `air`.
    pub air: Option<FogVolumetricMediaCoeffsSetting>,
    /// `water`.
    pub water: Option<FogVolumetricMediaCoeffsSetting>,
    /// `cloud`.
    pub cloud: Option<FogVolumetricMediaCoeffsSetting>,
    /// The members the schema does not know.
    pub extra: Extra,
}

/// The coefficients of one medium.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FogVolumetricMediaCoeffsSetting {
    /// `absorption`: how much of each colour the medium absorbs.
    pub absorption: Option<Keyframed<ColorNorm>>,
    /// `scattering`: how much of each colour the medium scatters.
    pub scattering: Option<Keyframed<ColorNorm>>,
    /// The members the schema does not know.
    pub extra: Extra,
}

/// `volumetric.henyey_greenstein_g`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FogVolumetricHenyeyGreensteinGCollection {
    /// `air`.
    pub air: Option<FogVolumetricHenyeyGreensteinGSetting>,
    /// `water`.
    pub water: Option<FogVolumetricHenyeyGreensteinGSetting>,
    /// The members the schema does not know.
    pub extra: Extra,
}

/// The scattering direction of one medium.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FogVolumetricHenyeyGreensteinGSetting {
    /// `henyey_greenstein_g`: −1 (backwards) to 1 (forwards).
    pub henyey_greenstein_g: Option<Keyframed<f32>>,
    /// The members the schema does not know.
    pub extra: Extra,
}

/// What `fog_start` and `fog_end` are measured in (`render_distance_type`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DistanceType {
    /// `"fixed"`: blocks. The value of a setting that leaves the member out.
    #[default]
    Fixed,
    /// `"render"`: a fraction of the render distance (−1 to 1 for the start, up to 1 for the
    /// end; the game warns outside that).
    Render,
}

/// A fog colour (`fog_color`): `"#RRGGBB"` or `[r, g, b]` with integers 0 to 255.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum FogColor {
    /// `"#RRGGBB"`, as written (hex digits of either case).
    Hex(String),
    /// `[r, g, b]`; an integer outside 0 to 255 is clamped when read, as the game clamps it.
    Rgb255([u8; 3]),
}

/// A coefficient colour (`absorption`, `scattering`): `"#RRGGBB"` or `[r, g, b]` with numbers
/// the game clamps to 0 to 1.
#[derive(Clone, Debug, PartialEq)]
pub enum ColorNorm {
    /// `"#RRGGBB"`, as written.
    Hex(String),
    /// `[r, g, b]` as written; [`ColorNorm::rgb`] gives the clamped components.
    Array([f32; 3]),
}

/// A value that may change over the day: one value, or values by time key
/// (`{"0.0": 0.02, "0.5": 0.1}`). The keyed form is read by engines from 1.26.30.
#[derive(Clone, Debug, PartialEq)]
pub enum Keyframed<T> {
    /// One value.
    Single(T),
    /// Values by time, sorted by time. Two keys of a file that are one time (`"0"` and
    /// `"0.0"`, or `"-0"` and `"0"`) are one keyframe, the later one, with its time as written.
    /// A list built in memory that holds one time twice is not written: `to_json` refuses it
    /// with `NotRepresentable` at the second keyframe of that time.
    Keys(Vec<(f32, T)>),
}

/// A keyframe time with `-0` as `0`, so that two finite times are one time exactly when
/// their bits are equal.
pub(crate) fn same_zero(time: f32) -> f32 {
    time + 0.0
}
