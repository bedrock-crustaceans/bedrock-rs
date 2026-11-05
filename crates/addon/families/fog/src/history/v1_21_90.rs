//! Fog settings at `[1.21.90, 1.26.20)`: the strict schema's third root, which adds
//! `volumetric.henyey_greenstein_g`, and the step to the cereal shape of 1.26.20.

use facet_value::VObject;

use super::{v1_16_100, v1_21_0};
use crate::{
    ColorNorm, FogColor, FogDescription, FogDistanceCollection, FogDistanceSetting, FogSettings,
    FogTransitionSetting, FogVolumetricCollection, FogVolumetricDensityCollection,
    FogVolumetricDensitySetting, FogVolumetricHenyeyGreensteinGCollection,
    FogVolumetricHenyeyGreensteinGSetting, FogVolumetricMediaCoeffsCollection,
    FogVolumetricMediaCoeffsSetting, FogWaterDistanceSetting, Keyframed,
};
use crate::{Era, float, object_with, read_document};
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Phase};
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::id::FogId;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::version::{FormatVersion, known};
use bedrock_addon_core::versioning::{ParseCx, Upgrade, UpgradeCx, VersionStruct, Versioned};

pub use super::v1_21_0::{
    CoeffColorJu, Coefficients, Density, DensitySetting, Description, Distance, DistanceType,
    FogColorJu, FogSetting, InitFog, MediaCoefficients, TransitionFog,
};

/// The whole document, as [`v1_21_0::FogDocument`] with this version's `volumetric`.
#[derive(Clone, Debug, PartialEq)]
pub struct FogDocument {
    /// `format_version`, as written.
    pub format_version: String,
    /// `minecraft:fog_settings.description`.
    pub description: Description,
    /// `minecraft:fog_settings.distance`.
    pub distance: Option<Distance>,
    /// `minecraft:fog_settings.volumetric`.
    pub volumetric: Option<Volumetric>,
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

impl Versioned for FogDocument {
    const VERSION: FormatVersion = known::V1_21_90;
}

impl VersionStruct for FogDocument {
    fn parse(document: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        read_document(document, Era::V1_21_90, &cx.at, d)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        let mut settings = VObject::new();
        settings.insert("description", v1_16_100::description(&self.description));
        if let Some(distance) = &self.distance {
            settings.insert("distance", v1_16_100::distance(distance)?);
        }
        if let Some(volumetric) = &self.volumetric {
            let mut object = v1_21_0::volumetric(&v1_21_0::Volumetric {
                density: volumetric.density.clone(),
                media_coefficients: volumetric.media_coefficients.clone(),
            })?;
            if let Some(hg) = &volumetric.henyey_greenstein_g {
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
        Ok(v1_16_100::document(&self.format_version, settings))
    }
}

// ---- the step to the cereal shape --------------------------------------------------------------

/// Op 7: a `fog_color` means the same in both eras, so it is copied (the array clamped to
/// 0–255, which the old struct already did).
// The strict schema's colour parser and the cereal colour setter both give `c / 255`.
fn color(color: FogColorJu) -> FogColor {
    match color {
        FogColorJu::Hex(text) => FogColor::Hex(text),
        FogColorJu::Array(rgb) => {
            FogColor::Rgb255(rgb.map(|c| u8::try_from(c.clamp(0, 255)).unwrap_or(u8::MAX)))
        }
    }
}

/// Op 9: a coefficient colour means the same in both eras.
fn coefficient(color: CoeffColorJu) -> Keyframed<ColorNorm> {
    Keyframed::Single(match color {
        CoeffColorJu::Hex(text) => ColorNorm::Hex(text),
        CoeffColorJu::Array(rgb) => ColorNorm::Array(rgb),
    })
}

/// Op 3: the four required members become `Some`.
fn setting(setting: &FogSetting) -> FogDistanceSetting {
    FogDistanceSetting {
        fog_start: Some(setting.fog_start),
        fog_end: Some(setting.fog_end),
        render_distance_type: Some(setting.render_distance_type),
        fog_color: Some(color(setting.fog_color.clone())),
        extra: UnknownMembers::default(),
    }
}

fn init_fog(init: InitFog) -> FogDistanceSetting {
    FogDistanceSetting {
        fog_start: Some(init.fog_start),
        fog_end: Some(init.fog_end),
        render_distance_type: Some(init.render_distance_type),
        fog_color: Some(color(init.fog_color)),
        extra: UnknownMembers::default(),
    }
}

/// Ops 3 and 5: an entry other than `water`; its `transition_fog` is dropped with an info.
// Ours. Vanilla's strict schema keeps it in `FogSetting::mTransitionSetting` for every
// distance type; the cereal shape has it on water only.
fn entry(
    name: &str,
    old: Option<FogSetting>,
    cx: &mut UpgradeCx<'_>,
) -> Option<FogDistanceSetting> {
    let old = old?;
    if old.transition_fog.is_some() {
        let mut at = JsonPath::new();
        for key in ["minecraft:fog_settings", "distance", name, "transition_fog"] {
            at.push(key);
        }
        cx.diagnostics.push(
            Diagnostic::info(DiagnosticCode::DroppedField { text: None })
                .at(at)
                .phase(Phase::Upgrade),
        );
    }
    Some(setting(&old))
}

/// Op 4: `water` keeps its `transition_fog`, every member `Some`.
fn water(old: FogSetting) -> FogWaterDistanceSetting {
    let base = setting(&old);
    FogWaterDistanceSetting {
        fog_start: base.fog_start,
        fog_end: base.fog_end,
        render_distance_type: base.render_distance_type,
        fog_color: base.fog_color,
        transition_fog: old.transition_fog.map(|t| FogTransitionSetting {
            init_fog: Some(init_fog(t.init_fog)),
            min_percent: Some(t.min_percent),
            mid_seconds: Some(t.mid_seconds),
            mid_percent: Some(t.mid_percent),
            max_seconds: Some(t.max_seconds),
            extra: UnknownMembers::default(),
        }),
        extra: UnknownMembers::default(),
    }
}

/// Op 8: every present member wrapped, each scalar a single keyframe.
fn density(setting: DensitySetting) -> FogVolumetricDensitySetting {
    FogVolumetricDensitySetting {
        max_density: Some(Keyframed::Single(setting.max_density)),
        uniform: setting.uniform,
        zero_density_height: setting.zero_density_height.map(Keyframed::Single),
        max_density_height: setting.max_density_height.map(Keyframed::Single),
        extra: UnknownMembers::default(),
    }
}

fn coefficients(c: Coefficients) -> FogVolumetricMediaCoeffsSetting {
    FogVolumetricMediaCoeffsSetting {
        absorption: Some(coefficient(c.absorption)),
        scattering: Some(coefficient(c.scattering)),
        extra: UnknownMembers::default(),
    }
}

fn hg(setting: HgSetting) -> FogVolumetricHenyeyGreensteinGSetting {
    FogVolumetricHenyeyGreensteinGSetting {
        henyey_greenstein_g: Some(Keyframed::Single(setting.henyey_greenstein_g)),
        extra: UnknownMembers::default(),
    }
}

// Vanilla has no upgrader for fog files: each era's parser writes the runtime definition
// itself. The step gives the cereal value whose runtime definition equals the one the strict
// schema parser builds.
impl Upgrade for FogDocument {
    type Next = FogSettings;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        // Op 1: `format_version` is not copied; `UpgradeCx::original_version` keeps it.
        FogSettings {
            // Op 2.
            description: FogDescription {
                identifier: FogId::parse(self.description.identifier),
                extra: UnknownMembers::default(),
            },
            distance: self.distance.map(|d| FogDistanceCollection {
                air: entry("air", d.air, cx),
                weather: entry("weather", d.weather, cx),
                water: d.water.map(water),
                lava: entry("lava", d.lava, cx),
                lava_resistance: entry("lava_resistance", d.lava_resistance, cx),
                powder_snow: entry("powder_snow", d.powder_snow, cx),
                extra: UnknownMembers::default(),
            }),
            volumetric: self.volumetric.map(|v| FogVolumetricCollection {
                density: v.density.map(|d| FogVolumetricDensityCollection {
                    air: d.air.map(density),
                    weather: d.weather.map(density),
                    water: d.water.map(density),
                    lava: d.lava.map(density),
                    lava_resistance: d.lava_resistance.map(density),
                    extra: UnknownMembers::default(),
                }),
                media_coefficients: v.media_coefficients.map(|m| {
                    FogVolumetricMediaCoeffsCollection {
                        air: m.air.map(coefficients),
                        water: m.water.map(coefficients),
                        cloud: m.cloud.map(coefficients),
                        extra: UnknownMembers::default(),
                    }
                }),
                // Op 10.
                henyey_greenstein_g: v.henyey_greenstein_g.map(|g| {
                    FogVolumetricHenyeyGreensteinGCollection {
                        air: g.air.map(hg),
                        water: g.water.map(hg),
                        extra: UnknownMembers::default(),
                    }
                }),
                extra: UnknownMembers::default(),
            }),
            extra: UnknownMembers::default(),
        }
    }
}
