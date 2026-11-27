//! Fog settings at `[1.16.100, 1.21.0)`: the game's strict schema
//! `fog_description_schema` at its first root, and the step to 1.21.0.
//!
//! Every object rejects a member the schema does not declare, every member of a distance
//! setting is required, and `transition_fog` may be written on every distance entry.

use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx};

pub use super::payload::{
    CoeffColorJu, Coefficients, Density, DensitySetting, Description, Distance, DistanceType,
    FogColorJu, FogPayload, FogSetting, HenyeyGreensteinG, HgSetting, InitFog, MediaCoefficients,
    TransitionFog, Volumetric,
};
use super::v1_21_0;

/// The whole document at 1.16.100: [`FogPayload`] read with the first root, which declares
/// neither `volumetric.density.weather` nor `volumetric.henyey_greenstein_g`.
pub type FogDocument = Legacy<FogPayload, 1, 16, 100>;

// the 1.21.0 root differs from the 1.16.100 root only by a version gate, which
// adds `volumetric.density.weather`: the payload carries on unchanged.
impl LegacyStep<1, 16, 100> for FogPayload {
    type Next = v1_21_0::FogDocument;

    fn step(this: FogDocument, _: &mut UpgradeCx<'_>) -> Self::Next {
        this.retag()
    }
}
