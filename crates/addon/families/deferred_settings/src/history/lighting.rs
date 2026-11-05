//! Lighting settings as written before format 1.26.0.
//!
//! - 1.21.40: the sun's and moon's colours have alpha, which the next step drops.
//! - 1.21.60: the colours are RGB.
//! - 1.21.70: `sky.intensity` exists (a plain number).
//! - 1.21.80: the sun and moon move under `directional_lights.orbital`, beside `flash`.
//! - 1.26.0 (the newest): `ambient` and `sky.intensity` are keyframed.

use crate::{LightingSettings, Rgb8};
use bedrock_addon_core::document::UnknownMembers;

/// The ambient light before format 1.26.0: plain values, both required.
#[derive(Clone, Debug, PartialEq)]
pub struct AmbientLight {
    /// The ambient colour (JSON `color`).
    pub color: Rgb8,
    /// The ambient strength, 0–5 (JSON `illuminance`).
    pub illuminance: f32,
    /// Members the game does not know, kept for write-back.
    pub extra: UnknownMembers,
}

/// The sky light of formats 1.21.70 and 1.21.80.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Sky {
    /// The strength of the sky light, 0.1–1 (JSON `intensity`).
    pub intensity: Option<f32>,
    /// Members the game does not know, kept for write-back.
    pub extra: UnknownMembers,
}

/// Format 1.21.40.
pub mod v1_21_40 {
    use super::AmbientLight;
    use crate::{Description, Emissive, OrbitalLights, Rgba8};
    use bedrock_addon_core::document::UnknownMembers;
    use bedrock_addon_core::id::kind;

    /// Lighting settings at format 1.21.40: the sun, moon and orbital offset directly in
    /// `directional_lights`, their colours with alpha.
    #[derive(Clone, Debug, Default, PartialEq)]
    pub struct LightingSettings {
        /// The settings' name (JSON `description`).
        pub description: Option<Description<kind::LightingSettings>>,
        /// The sun and the moon (JSON `directional_lights`).
        pub directional_lights: Option<OrbitalLights<Rgba8>>,
        /// How light-emitting surfaces are drawn (JSON `emissive`).
        pub emissive: Option<Emissive>,
        /// The light that reaches every surface (JSON `ambient`).
        pub ambient: Option<AmbientLight>,
        /// Members the game does not know, kept for write-back.
        pub extra: UnknownMembers,
    }
}

/// Format 1.21.60.
pub mod v1_21_60 {
    use super::AmbientLight;
    use crate::{Description, Emissive, OrbitalLights};
    use bedrock_addon_core::document::UnknownMembers;
    use bedrock_addon_core::id::kind;

    /// Lighting settings at format 1.21.60: as 1.21.40, the colours without alpha.
    #[derive(Clone, Debug, Default, PartialEq)]
    pub struct LightingSettings {
        /// The settings' name (JSON `description`).
        pub description: Option<Description<kind::LightingSettings>>,
        /// The sun and the moon (JSON `directional_lights`).
        pub directional_lights: Option<OrbitalLights>,
        /// How light-emitting surfaces are drawn (JSON `emissive`).
        pub emissive: Option<Emissive>,
        /// The light that reaches every surface (JSON `ambient`).
        pub ambient: Option<AmbientLight>,
        /// Members the game does not know, kept for write-back.
        pub extra: UnknownMembers,
    }
}

/// Format 1.21.70.
pub mod v1_21_70 {
    use super::{AmbientLight, Sky};
    use crate::{Description, Emissive, OrbitalLights};
    use bedrock_addon_core::document::UnknownMembers;
    use bedrock_addon_core::id::kind;

    /// Lighting settings at format 1.21.70: as 1.21.60, with `sky`.
    #[derive(Clone, Debug, Default, PartialEq)]
    pub struct LightingSettings {
        /// The settings' name (JSON `description`).
        pub description: Option<Description<kind::LightingSettings>>,
        /// The sun and the moon (JSON `directional_lights`).
        pub directional_lights: Option<OrbitalLights>,
        /// How light-emitting surfaces are drawn (JSON `emissive`).
        pub emissive: Option<Emissive>,
        /// The light that reaches every surface (JSON `ambient`).
        pub ambient: Option<AmbientLight>,
        /// The light of the sky (JSON `sky`).
        pub sky: Option<Sky>,
        /// Members the game does not know, kept for write-back.
        pub extra: UnknownMembers,
    }
}

/// Format 1.21.80.
pub mod v1_21_80 {
    use super::{AmbientLight, Sky};
    use crate::{Description, DirectionalLights, Emissive};
    use bedrock_addon_core::document::UnknownMembers;
    use bedrock_addon_core::id::kind;

    /// Lighting settings at format 1.21.80: the sun and moon under
    /// `directional_lights.orbital`, beside `flash`; `ambient` and `sky` still plain values.
    #[derive(Clone, Debug, Default, PartialEq)]
    pub struct LightingSettings {
        /// The settings' name (JSON `description`).
        pub description: Option<Description<kind::LightingSettings>>,
        /// The sun, the moon and the End's flash (JSON `directional_lights`).
        pub directional_lights: Option<DirectionalLights>,
        /// How light-emitting surfaces are drawn (JSON `emissive`).
        pub emissive: Option<Emissive>,
        /// The light that reaches every surface (JSON `ambient`).
        pub ambient: Option<AmbientLight>,
        /// The light of the sky (JSON `sky`).
        pub sky: Option<Sky>,
        /// Members the game does not know, kept for write-back.
        pub extra: UnknownMembers,
    }
}

chain! {
    latest = LightingSettings;
    /// A lighting-settings file parsed by the struct its `format_version` selects, not
    /// upgraded.
    enum Any {
        /// Format 1.21.40.
        V1_21_40(v1_21_40::LightingSettings),
        /// Format 1.21.60.
        V1_21_60(v1_21_60::LightingSettings),
        /// Format 1.21.70.
        V1_21_70(v1_21_70::LightingSettings),
        /// Format 1.21.80.
        V1_21_80(v1_21_80::LightingSettings),
        /// Format 1.26.0: the newest lighting settings.
        V1_26_0(LightingSettings),
    }
}
