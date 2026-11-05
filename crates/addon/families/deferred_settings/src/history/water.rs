//! Water settings as written before format 1.26.20.
//!
//! - 1.21.40: `description`, `particle_concentrations` and `waves` with `sampleWidth`.
//! - 1.21.80: adds `caustics` and `waves.direction_increment`.
//! - 1.21.120: `waves.sampleWidth` is gone (the step drops it).
//! - 1.26.0: adds `biome_water_color_contribution`.
//! - 1.26.20 (the newest): the same members; `description` is required.

use crate::WaterSettings;
use bedrock_addon_core::document::UnknownMembers;

/// The waves of formats 1.21.40 and 1.21.80: with `sampleWidth`, which 1.21.120 dropped.
/// `direction_increment` is read from 1.21.80.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Waves {
    /// Whether the surface has waves (JSON `enabled`).
    pub enabled: Option<bool>,
    /// The frequency of the first wave, 0.01–3 (JSON `frequency`).
    pub frequency: Option<f32>,
    /// How many waves are summed, 1–30 (JSON `octaves`).
    pub octaves: Option<u8>,
    /// The height of the waves, 0–3 (JSON `depth`).
    pub depth: Option<f32>,
    /// The width the surface is sampled over, 0.01–1 (JSON `sampleWidth`).
    pub sample_width: Option<f32>,
    /// How fast the waves move, 0.01–10 (JSON `speed`).
    pub speed: Option<f32>,
    /// How strongly each wave drags the next, −1–1 (JSON `pull`).
    pub pull: Option<f32>,
    /// How much each wave is blended with the next, 0–1 (JSON `mix`).
    pub mix: Option<f32>,
    /// The frequency factor from one wave to the next, 0–2 (JSON `frequency_scaling`).
    pub frequency_scaling: Option<f32>,
    /// The speed factor from one wave to the next, 0–2 (JSON `speed_scaling`).
    pub speed_scaling: Option<f32>,
    /// The turn of direction from one wave to the next in degrees, 0–360 (JSON
    /// `direction_increment`; always `None` in a 1.21.40 document).
    pub direction_increment: Option<f32>,
    /// How sharp the wave crests are, 1–10 (JSON `shape`).
    pub shape: Option<f32>,
    /// Members the game does not know, kept for write-back.
    pub extra: UnknownMembers,
}

/// Format 1.21.40.
pub mod v1_21_40 {
    use super::Waves;
    use crate::{Description, ParticleConcentrations};
    use bedrock_addon_core::document::UnknownMembers;
    use bedrock_addon_core::id::kind;

    /// Water settings at format 1.21.40.
    #[derive(Clone, Debug, Default, PartialEq)]
    pub struct WaterSettings {
        /// The settings' name (JSON `description`).
        pub description: Option<Description<kind::WaterSettings>>,
        /// What is dissolved in the water (JSON `particle_concentrations`).
        pub particle_concentrations: Option<ParticleConcentrations>,
        /// The waves of the surface (JSON `waves`).
        pub waves: Option<Waves>,
        /// Members the game does not know, kept for write-back.
        pub extra: UnknownMembers,
    }
}

/// Format 1.21.80.
pub mod v1_21_80 {
    use super::Waves;
    use crate::{Caustics, Description, ParticleConcentrations};
    use bedrock_addon_core::document::UnknownMembers;
    use bedrock_addon_core::id::kind;

    /// Water settings at format 1.21.80: with `caustics` and `waves.direction_increment`.
    #[derive(Clone, Debug, Default, PartialEq)]
    pub struct WaterSettings {
        /// The settings' name (JSON `description`).
        pub description: Option<Description<kind::WaterSettings>>,
        /// What is dissolved in the water (JSON `particle_concentrations`).
        pub particle_concentrations: Option<ParticleConcentrations>,
        /// The light patterns on surfaces under water (JSON `caustics`).
        pub caustics: Option<Caustics>,
        /// The waves of the surface (JSON `waves`).
        pub waves: Option<Waves>,
        /// Members the game does not know, kept for write-back.
        pub extra: UnknownMembers,
    }
}

/// Format 1.21.120.
pub mod v1_21_120 {
    use crate::{Caustics, Description, ParticleConcentrations, Waves};
    use bedrock_addon_core::document::UnknownMembers;
    use bedrock_addon_core::id::kind;

    /// Water settings at format 1.21.120: `waves` without `sampleWidth`.
    #[derive(Clone, Debug, Default, PartialEq)]
    pub struct WaterSettings {
        /// The settings' name (JSON `description`).
        pub description: Option<Description<kind::WaterSettings>>,
        /// What is dissolved in the water (JSON `particle_concentrations`).
        pub particle_concentrations: Option<ParticleConcentrations>,
        /// The light patterns on surfaces under water (JSON `caustics`).
        pub caustics: Option<Caustics>,
        /// The waves of the surface (JSON `waves`).
        pub waves: Option<Waves>,
        /// Members the game does not know, kept for write-back.
        pub extra: UnknownMembers,
    }
}

/// Format 1.26.0.
pub mod v1_26_0 {
    /// Water settings at format 1.26.0: the members of the newest struct, `description` still
    /// optional.
    #[derive(Clone, Debug, Default, PartialEq)]
    pub struct WaterSettings(pub crate::WaterSettings);
}

chain! {
    latest = WaterSettings;
    /// A water-settings file parsed by the struct its `format_version` selects, not upgraded.
    enum Any {
        /// Format 1.21.40.
        V1_21_40(v1_21_40::WaterSettings),
        /// Format 1.21.80.
        V1_21_80(v1_21_80::WaterSettings),
        /// Format 1.21.120.
        V1_21_120(v1_21_120::WaterSettings),
        /// Format 1.26.0.
        V1_26_0(v1_26_0::WaterSettings),
        /// Format 1.26.20: the newest water settings.
        V1_26_20(WaterSettings),
    }
}
