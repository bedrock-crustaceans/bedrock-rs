//! Fog settings at `[1.21.90, 1.26.20)`: the strict schema's third root, which adds
//! `volumetric.henyey_greenstein_g`, and the step to the cereal shape of 1.26.20.

use crate::{
    ColorNorm, FogColor, FogDescription, FogDistanceCollection, FogDistanceSetting, FogSettings,
    FogTransitionSetting, FogVolumetricCollection, FogVolumetricDensityCollection,
    FogVolumetricDensitySetting, FogVolumetricHenyeyGreensteinGCollection,
    FogVolumetricHenyeyGreensteinGSetting, FogVolumetricMediaCoeffsCollection,
    FogVolumetricMediaCoeffsSetting, FogWaterDistanceSetting, Keyframed,
};
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, JsonPath, Phase};
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::id::FogId;
use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx};

pub use super::payload::{
    CoeffColorJu, Coefficients, Density, DensitySetting, Description, Distance, DistanceType,
    FogColorJu, FogPayload, FogSetting, HenyeyGreensteinG, HgSetting, InitFog, MediaCoefficients,
    TransitionFog, Volumetric,
};

/// The whole document at 1.21.90: [`FogPayload`] read with the third root, which adds
/// `volumetric.henyey_greenstein_g`.
pub type FogDocument = Legacy<FogPayload, 1, 21, 90>;

// ---- the step to the cereal shape --------------------------------------------------------------

/// Op 7: a `fog_color` means the same in both eras, so it is copied (the array clamped to
/// 0–255, which the old struct already did).
// The strict schema's colour parser and the cereal colour setter both give `c / 255`.
fn color(color: FogColorJu) -> FogColor {
    match color {
        FogColorJu::Hex(text) => FogColor::Hex(text),
        FogColorJu::Array(rgb) => {
            FogColor::Array(rgb.map(|c| u8::try_from(c.clamp(0, 255)).unwrap_or(u8::MAX)))
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
impl LegacyStep<1, 21, 90> for FogPayload {
    type Next = FogSettings;

    fn step(this: FogDocument, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let fog = this.0;
        // Op 1: `format_version` is not copied; `UpgradeCx::original_version` keeps it.
        FogSettings {
            // Op 2.
            description: FogDescription {
                identifier: FogId::parse(fog.description.identifier),
                extra: UnknownMembers::default(),
            },
            distance: fog.distance.map(|d| FogDistanceCollection {
                air: entry("air", d.air, cx),
                weather: entry("weather", d.weather, cx),
                water: d.water.map(water),
                lava: entry("lava", d.lava, cx),
                lava_resistance: entry("lava_resistance", d.lava_resistance, cx),
                powder_snow: entry("powder_snow", d.powder_snow, cx),
                extra: UnknownMembers::default(),
            }),
            volumetric: fog.volumetric.map(|v| FogVolumetricCollection {
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
