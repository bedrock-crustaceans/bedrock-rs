//! The fog family's field gates: which `format_version` first
//! declares a member, which one stops declaring it, and which engine first reads a value form
//! that came without a new `format_version`. The family has no components, so this table is
//! what a component table is elsewhere.

use bedrock_addon_core::version::FormatVersion;

/// One row of [`FOG_FIELD_GATES`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FogFieldGate {
    /// The member's dotted path below `minecraft:fog_settings`; `*` stands for every entry of
    /// a collection.
    pub path: &'static str,
    /// The first `format_version` whose struct declares it.
    pub released: FormatVersion,
    /// The first `format_version` whose struct no longer declares it.
    pub removed: Option<FormatVersion>,
    /// The first engine that reads the value form the row is about, when the form arrived
    /// without a new `format_version`.
    pub since_engine: Option<FormatVersion>,
    /// What the row is about.
    pub note: &'static str,
}

const V1_16_100: FormatVersion = FormatVersion::new(1, 16, 100);
const V1_21_0: FormatVersion = FormatVersion::new(1, 21, 0);
const V1_21_90: FormatVersion = FormatVersion::new(1, 21, 90);
const V1_26_20: FormatVersion = FormatVersion::new(1, 26, 20);
const E1_26_30: FormatVersion = FormatVersion::new(1, 26, 30);

const fn gate(path: &'static str, released: FormatVersion, note: &'static str) -> FogFieldGate {
    FogFieldGate {
        path,
        released,
        removed: None,
        since_engine: None,
        note,
    }
}

const fn keyframes(path: &'static str) -> FogFieldGate {
    FogFieldGate {
        path,
        released: V1_26_20,
        removed: None,
        since_engine: Some(E1_26_30),
        note: "the keyframe object form",
    }
}

/// The fog members whose presence depends on a version, in the order of the folder's table.
pub const FOG_FIELD_GATES: &[FogFieldGate] = &[
    gate("description.identifier", V1_16_100, "the definition's name"),
    gate(
        "distance.*",
        V1_16_100,
        "air, weather, water, lava, lava_resistance, powder_snow",
    ),
    FogFieldGate {
        path: "distance.*.transition_fog",
        released: V1_16_100,
        removed: Some(V1_26_20),
        since_engine: None,
        note: "on an entry other than water: an unknown member from 1.26.20",
    },
    gate(
        "distance.water.transition_fog",
        V1_16_100,
        "the water transition",
    ),
    gate(
        "volumetric.density.*",
        V1_16_100,
        "air, water, lava, lava_resistance",
    ),
    gate(
        "volumetric.density.weather",
        V1_21_0,
        "an unknown member below 1.21.0",
    ),
    gate(
        "volumetric.media_coefficients.*",
        V1_16_100,
        "air, water, cloud",
    ),
    gate(
        "volumetric.henyey_greenstein_g.*",
        V1_21_90,
        "an unknown member below 1.21.90",
    ),
    keyframes("volumetric.density.*.max_density"),
    keyframes("volumetric.density.*.zero_density_height"),
    keyframes("volumetric.density.*.max_density_height"),
    keyframes("volumetric.media_coefficients.*.scattering"),
    keyframes("volumetric.media_coefficients.*.absorption"),
    keyframes("volumetric.henyey_greenstein_g.*.henyey_greenstein_g"),
];
