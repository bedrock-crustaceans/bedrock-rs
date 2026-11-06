//! The fog document below 1.26.20, one payload for the three roots of the game's strict
//! schema `fog_description_schema`: 1.16.100, 1.21.0 (which adds `volumetric.density.weather`)
//! and 1.21.90 (which adds `volumetric.henyey_greenstein_g`). Each era is
//! `Legacy<FogPayload, ..>`: it reads the document with its root, which refuses a member a later
//! root adds (an unknown member fails the file), and writes it without those members.

use facet_value::VObject;

pub use crate::DistanceType;
use crate::{Era, color_ju, float, norm_ju, object_with, read_document};
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::version::{FormatVersion, known};
use bedrock_addon_core::versioning::{LegacyPayload, ParseCx};

/// The whole document: below 1.26.20 the game reads `format_version` and the payload with one
/// schema, so a third root member fails the file like any unknown member. The members a later
/// era adds are `None` in a document an earlier era read.
#[derive(Clone, Debug, PartialEq)]
pub struct FogPayload {
    /// `format_version`, as written.
    pub format_version: String,
    /// `minecraft:fog_settings.description`.
    pub description: Description,
    /// `minecraft:fog_settings.distance`.
    pub distance: Option<Distance>,
    /// `minecraft:fog_settings.volumetric`.
    pub volumetric: Option<Volumetric>,
}

// A fog document has no Molang field.
bedrock_addon_core::impl_no_molang!(FogPayload);

/// The format that adds `volumetric.density.weather`.
const V1_21_0: FormatVersion = FormatVersion::new(1, 21, 0);

/// The root of the strict schema that reads the struct of `version`.
fn era(version: FormatVersion) -> Era {
    if version >= known::V1_21_90 {
        Era::V1_21_90
    } else if version >= V1_21_0 {
        Era::V1_21_0
    } else {
        Era::V1_16_100
    }
}

impl LegacyPayload for FogPayload {
    fn parse(
        document: JsonSlice<'_>,
        version: FormatVersion,
        cx: &ParseCx<'_>,
        d: &mut Diagnostics,
    ) -> Option<Self> {
        read_document(document, era(version), &cx.at, d)
    }

    fn write(&self, version: FormatVersion) -> Result<Value, EncodeError> {
        let mut settings = VObject::new();
        settings.insert("description", description(&self.description));
        if let Some(distance) = &self.distance {
            settings.insert("distance", self::distance(distance)?);
        }
        if let Some(volumetric) = &self.volumetric {
            let mut object = VObject::new();
            if let Some(density) = &volumetric.density {
                object.insert("density", self::density(density, version)?);
            }
            if let Some(media) = &volumetric.media_coefficients {
                object.insert("media_coefficients", media_coefficients(media)?);
            }
            if version >= known::V1_21_90
                && let Some(hg) = &volumetric.henyey_greenstein_g
            {
                let entry = |setting: Option<HgSetting>| {
                    setting
                        .map(|s| {
                            object_with([(
                                "henyey_greenstein_g",
                                Some(float(s.henyey_greenstein_g)?),
                            )])
                        })
                        .transpose()
                };
                object.insert(
                    "henyey_greenstein_g",
                    object_with([("air", entry(hg.air)?), ("water", entry(hg.water)?)])?,
                );
            }
            settings.insert("volumetric", Value::from(object));
        }
        Ok(document(&self.format_version, settings))
    }
}

/// `description`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Description {
    /// `identifier`: lower-cased, it must be `namespace:name` over `a`–`z`, digits, `.` and
    /// `_`; kept as written.
    pub identifier: String,
}

/// `distance`: one setting per medium.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Distance {
    /// `air`.
    pub air: Option<FogSetting>,
    /// `weather`.
    pub weather: Option<FogSetting>,
    /// `water`.
    pub water: Option<FogSetting>,
    /// `lava`.
    pub lava: Option<FogSetting>,
    /// `lava_resistance`.
    pub lava_resistance: Option<FogSetting>,
    /// `powder_snow`.
    pub powder_snow: Option<FogSetting>,
}

/// One distance setting: the four members are required.
#[derive(Clone, Debug, PartialEq)]
pub struct FogSetting {
    /// `fog_color`.
    pub fog_color: FogColorJu,
    /// `fog_start`: any number.
    pub fog_start: f32,
    /// `fog_end`: at least 0.
    pub fog_end: f32,
    /// `render_distance_type`: `"fixed"` or `"render"`, in any case.
    pub render_distance_type: DistanceType,
    /// `transition_fog`: allowed on every entry, used for water.
    pub transition_fog: Option<TransitionFog>,
}

/// `transition_fog`: every member is required.
#[derive(Clone, Debug, PartialEq)]
pub struct TransitionFog {
    /// `init_fog`.
    pub init_fog: InitFog,
    /// `min_percent`: 0 to 1.
    pub min_percent: f32,
    /// `mid_seconds`: at least 0.
    pub mid_seconds: f32,
    /// `mid_percent`: 0 to 1.
    pub mid_percent: f32,
    /// `max_seconds`: at least 0.
    pub max_seconds: f32,
}

/// `transition_fog.init_fog`: a distance setting without a transition of its own.
#[derive(Clone, Debug, PartialEq)]
pub struct InitFog {
    /// `fog_color`.
    pub fog_color: FogColorJu,
    /// `fog_start`.
    pub fog_start: f32,
    /// `fog_end`: at least 0.
    pub fog_end: f32,
    /// `render_distance_type`.
    pub render_distance_type: DistanceType,
}

/// One density setting. No rule ties `uniform` to the heights at these versions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DensitySetting {
    /// `max_density`: required, 0 to 1.
    pub max_density: f32,
    /// `max_density_height`: −64 to 320.
    pub max_density_height: Option<f32>,
    /// `zero_density_height`: −64 to 320.
    pub zero_density_height: Option<f32>,
    /// `uniform`.
    pub uniform: Option<bool>,
}

/// `volumetric.media_coefficients`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MediaCoefficients {
    /// `air`.
    pub air: Option<Coefficients>,
    /// `water`.
    pub water: Option<Coefficients>,
    /// `cloud`.
    pub cloud: Option<Coefficients>,
}

/// The coefficients of one medium: both are required.
#[derive(Clone, Debug, PartialEq)]
pub struct Coefficients {
    /// `scattering`.
    pub scattering: CoeffColorJu,
    /// `absorption`.
    pub absorption: CoeffColorJu,
}

/// A `fog_color` of the old schema: `"#rrggbb"` (hex digits of either case) or an array of at
/// least three integers.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum FogColorJu {
    /// `"#rrggbb"`, as written.
    Hex(String),
    /// The first three elements, each clamped to 0–255 as the game clamps it; an element that
    /// is no integer (`0.5`, `1.0`) is 255.
    Array([i32; 3]),
}

/// A coefficient colour of the old schema: `"#rrggbb"` or an array of at least three numbers.
#[derive(Clone, Debug, PartialEq)]
pub enum CoeffColorJu {
    /// `"#rrggbb"`, as written.
    Hex(String),
    /// The first three elements as written; the game clamps each to 0–1.
    Array([f32; 3]),
}

/// `volumetric.density`, with `weather`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Density {
    /// `air`.
    pub air: Option<DensitySetting>,
    /// `weather`: new at 1.21.0.
    pub weather: Option<DensitySetting>,
    /// `water`.
    pub water: Option<DensitySetting>,
    /// `lava`.
    pub lava: Option<DensitySetting>,
    /// `lava_resistance`.
    pub lava_resistance: Option<DensitySetting>,
}

/// `volumetric`, with `henyey_greenstein_g`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Volumetric {
    /// `density`.
    pub density: Option<Density>,
    /// `media_coefficients`.
    pub media_coefficients: Option<MediaCoefficients>,
    /// `henyey_greenstein_g`: new at 1.21.90.
    pub henyey_greenstein_g: Option<HenyeyGreensteinG>,
}

/// `volumetric.henyey_greenstein_g`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HenyeyGreensteinG {
    /// `air`.
    pub air: Option<HgSetting>,
    /// `water`.
    pub water: Option<HgSetting>,
}

/// The scattering direction of one medium.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HgSetting {
    /// `henyey_greenstein_g`: required, −1 to 1.
    pub henyey_greenstein_g: f32,
}

// ---- writing --------------------------------------------------------------------------------

/// The root object: `format_version` and the payload.
fn document(format_version: &str, settings: VObject) -> Value {
    let mut root = VObject::new();
    root.insert("format_version", format_version);
    root.insert("minecraft:fog_settings", Value::from(settings));
    root.into()
}

fn description(description: &Description) -> Value {
    let mut object = VObject::new();
    object.insert("identifier", description.identifier.as_str());
    object.into()
}

fn init_fog(init: &InitFog) -> Result<Value, EncodeError> {
    object_with([
        ("fog_color", Some(color_ju(&init.fog_color))),
        ("fog_start", Some(float(init.fog_start)?)),
        ("fog_end", Some(float(init.fog_end)?)),
        (
            "render_distance_type",
            Some(init.render_distance_type.as_str().into()),
        ),
    ])
}

fn fog_setting(setting: &FogSetting) -> Result<Value, EncodeError> {
    let transition = match &setting.transition_fog {
        Some(t) => Some(object_with([
            ("init_fog", Some(init_fog(&t.init_fog)?)),
            ("min_percent", Some(float(t.min_percent)?)),
            ("mid_seconds", Some(float(t.mid_seconds)?)),
            ("mid_percent", Some(float(t.mid_percent)?)),
            ("max_seconds", Some(float(t.max_seconds)?)),
        ])?),
        None => None,
    };
    object_with([
        ("fog_color", Some(color_ju(&setting.fog_color))),
        ("fog_start", Some(float(setting.fog_start)?)),
        ("fog_end", Some(float(setting.fog_end)?)),
        (
            "render_distance_type",
            Some(setting.render_distance_type.as_str().into()),
        ),
        ("transition_fog", transition),
    ])
}

fn distance(distance: &Distance) -> Result<Value, EncodeError> {
    let entry = |setting: &Option<FogSetting>| setting.as_ref().map(fog_setting).transpose();
    object_with([
        ("air", entry(&distance.air)?),
        ("weather", entry(&distance.weather)?),
        ("water", entry(&distance.water)?),
        ("lava", entry(&distance.lava)?),
        ("lava_resistance", entry(&distance.lava_resistance)?),
        ("powder_snow", entry(&distance.powder_snow)?),
    ])
}

fn density_setting(setting: &DensitySetting) -> Result<Value, EncodeError> {
    let optional = |x: Option<f32>| x.map(float).transpose();
    object_with([
        ("max_density", Some(float(setting.max_density)?)),
        ("max_density_height", optional(setting.max_density_height)?),
        (
            "zero_density_height",
            optional(setting.zero_density_height)?,
        ),
        ("uniform", setting.uniform.map(Value::from)),
    ])
}

/// `density`; `weather` is the member 1.21.0 adds (left out below it).
fn density(density: &Density, version: FormatVersion) -> Result<Value, EncodeError> {
    let entry = |setting: Option<&DensitySetting>| setting.map(density_setting).transpose();
    let weather = density.weather.as_ref().filter(|_| version >= V1_21_0);
    object_with([
        ("air", entry(density.air.as_ref())?),
        ("weather", entry(weather)?),
        ("water", entry(density.water.as_ref())?),
        ("lava", entry(density.lava.as_ref())?),
        ("lava_resistance", entry(density.lava_resistance.as_ref())?),
    ])
}

fn media_coefficients(media: &MediaCoefficients) -> Result<Value, EncodeError> {
    let entry = |c: &Option<Coefficients>| {
        c.as_ref()
            .map(|c| {
                object_with([
                    ("scattering", Some(norm_ju(&c.scattering)?)),
                    ("absorption", Some(norm_ju(&c.absorption)?)),
                ])
            })
            .transpose()
    };
    object_with([
        ("air", entry(&media.air)?),
        ("water", entry(&media.water)?),
        ("cloud", entry(&media.cloud)?),
    ])
}
