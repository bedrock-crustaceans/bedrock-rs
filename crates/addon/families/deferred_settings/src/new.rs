//! Constructors of the settings documents.

use super::{
    AtmosphereSettings, ColorGradingDescription, ColorGradingSettings, CubemapSettings,
    Description, LightingSettings, WaterSettings,
};
use bedrock_addon_core::id::{
    AtmosphereSettingsId, ColorGradingSettingsId, CubemapSettingsId, LightingSettingsId,
    WaterSettingsId,
};

impl LightingSettings {
    /// Lighting settings named `identifier` that set nothing.
    #[must_use]
    pub fn new(identifier: LightingSettingsId) -> Self {
        Self {
            description: Some(Description::new(identifier)),
            ..Self::default()
        }
    }
}

impl WaterSettings {
    /// Water settings named `identifier` that set nothing.
    #[must_use]
    pub fn new(identifier: WaterSettingsId) -> Self {
        Self {
            description: Some(Description::new(identifier)),
            ..Self::default()
        }
    }
}

impl AtmosphereSettings {
    /// Atmosphere settings named `identifier` that set nothing.
    #[must_use]
    pub fn new(identifier: AtmosphereSettingsId) -> Self {
        Self {
            description: Some(Description::new(identifier)),
            ..Self::default()
        }
    }
}

impl ColorGradingSettings {
    /// Colour-grading settings named `identifier` that set nothing.
    #[must_use]
    pub fn new(identifier: ColorGradingSettingsId) -> Self {
        Self {
            description: Some(ColorGradingDescription {
                identifier: Some(identifier),
                ..ColorGradingDescription::default()
            }),
            ..Self::default()
        }
    }
}

impl CubemapSettings {
    /// Cubemap settings named `identifier` that set nothing.
    #[must_use]
    pub fn new(identifier: CubemapSettingsId) -> Self {
        Self {
            description: Some(Description::new(identifier)),
            ..Self::default()
        }
    }
}
