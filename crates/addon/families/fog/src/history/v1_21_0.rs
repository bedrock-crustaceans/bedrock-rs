//! Fog settings at `[1.21.0, 1.21.90)`: the strict schema's second root, which adds
//! `volumetric.density.weather`, and the step to 1.21.90.

use facet_value::VObject;

use super::{v1_16_100, v1_21_90};
use crate::{Era, read_document};
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{ParseCx, Upgrade, UpgradeCx, VersionStruct, Versioned};

pub use super::v1_16_100::{
    CoeffColorJu, Coefficients, DensitySetting, Description, Distance, DistanceType, FogColorJu,
    FogSetting, InitFog, MediaCoefficients, TransitionFog,
};

/// The whole document, as [`v1_16_100::FogDocument`] with this version's `volumetric`.
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

/// `volumetric`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Volumetric {
    /// `density`.
    pub density: Option<Density>,
    /// `media_coefficients`.
    pub media_coefficients: Option<MediaCoefficients>,
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

impl Versioned for FogDocument {
    const VERSION: FormatVersion = FormatVersion::new(1, 21, 0);
}

/// A 1.21.90 document without the member that version added.
pub(super) fn narrow(document: v1_21_90::FogDocument) -> FogDocument {
    FogDocument {
        format_version: document.format_version,
        description: document.description,
        distance: document.distance,
        volumetric: document.volumetric.map(|v| Volumetric {
            density: v.density,
            media_coefficients: v.media_coefficients,
        }),
    }
}

/// `volumetric` without `henyey_greenstein_g`.
pub(super) fn volumetric(volumetric: &Volumetric) -> Result<VObject, EncodeError> {
    let mut object = VObject::new();
    if let Some(density) = &volumetric.density {
        let plain = v1_16_100::Density {
            air: density.air,
            water: density.water,
            lava: density.lava,
            lava_resistance: density.lava_resistance,
        };
        object.insert(
            "density",
            v1_16_100::density(&plain, density.weather.as_ref())?,
        );
    }
    if let Some(media) = &volumetric.media_coefficients {
        object.insert("media_coefficients", v1_16_100::media_coefficients(media)?);
    }
    Ok(object)
}

impl VersionStruct for FogDocument {
    fn parse(document: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        read_document(document, Era::V1_21_0, &cx.at, d).map(narrow)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        let mut settings = VObject::new();
        settings.insert("description", v1_16_100::description(&self.description));
        if let Some(distance) = &self.distance {
            settings.insert("distance", v1_16_100::distance(distance)?);
        }
        if let Some(volumetric) = &self.volumetric {
            settings.insert("volumetric", Value::from(self::volumetric(volumetric)?));
        }
        Ok(v1_16_100::document(&self.format_version, settings))
    }
}

// the 1.21.90 root adds `volumetric.henyey_greenstein_g` behind a version gate
// and changes nothing else.
impl Upgrade for FogDocument {
    type Next = v1_21_90::FogDocument;

    fn upgrade(self, _: &mut UpgradeCx<'_>) -> Self::Next {
        v1_21_90::FogDocument {
            format_version: self.format_version,
            description: self.description,
            distance: self.distance,
            volumetric: self.volumetric.map(|v| v1_21_90::Volumetric {
                density: v.density,
                media_coefficients: v.media_coefficients,
                henyey_greenstein_g: None,
            }),
        }
    }
}
