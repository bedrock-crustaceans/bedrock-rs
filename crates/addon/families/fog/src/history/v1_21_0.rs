//! Fog settings at `[1.21.0, 1.21.90)`: the strict schema's second root, which adds
//! `volumetric.density.weather`, and the step to 1.21.90.

use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx};

pub use super::payload::{
    CoeffColorJu, Coefficients, Density, DensitySetting, Description, Distance, DistanceType,
    FogColorJu, FogPayload, FogSetting, HenyeyGreensteinG, HgSetting, InitFog, MediaCoefficients,
    TransitionFog, Volumetric,
};
use super::v1_21_90;

/// The whole document at 1.21.0: [`FogPayload`] read with the second root, which adds
/// `volumetric.density.weather`.
pub type FogDocument = Legacy<FogPayload, 1, 21, 0>;

// the 1.21.90 root adds `volumetric.henyey_greenstein_g` behind a version gate
// and changes nothing else: the payload carries on unchanged.
impl LegacyStep<1, 21, 0> for FogPayload {
    type Next = v1_21_90::FogDocument;

    fn step(this: FogDocument, _: &mut UpgradeCx<'_>) -> Self::Next {
        this.retag()
    }
}
