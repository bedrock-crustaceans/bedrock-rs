//! The checks the client runs when it applies a loaded client biome that need no other file:
//! a failing component means nothing of the document is applied.
//!
//! The client runs them on the document that claimed its identifier and on no other, so they
//! are no part of a file's parse: the registry's combine step runs them on the claiming
//! document of each identifier, and `Content::parse` on the one file it is given.
//!
//! The checks that look a name up in another registry (fog, render configurations, cubemaps,
//! the server's biome list) belong to the linking pass.

use super::component::{
    AmbientSounds, BiomeMusic, ClientBiomeComponent, Precipitation, WaterAppearance, child,
};
use super::parse::PAYLOAD_KEY;
use super::texts::{NO_SOUND, OPACITY, PRECIPITATION, VOLUME, fixed};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity,
};
use bedrock_addon_core::document::{Component, ComponentMap};

/// An error of a component check: vanilla's text at the component (or its member).
fn fail<T: Component>(text: &'static str, member: Option<&str>, d: &mut Diagnostics) {
    let mut path = child(&JsonPath::new(), &[PAYLOAD_KEY, "components", T::NAME]);
    if let Some(member) = member {
        path.push(member);
    }
    d.push(Diagnostic::new(
        DiagnosticCode::InvalidValue { text: fixed(text) },
        Severity::Error,
        Phase::Validate,
        Location::new(None, None, path, None),
    ));
}

/// `value` lies in `[0, 1]`.
fn unit(value: f32) -> bool {
    (0.0..=1.0).contains(&value)
}

/// The component checks on the components of the document that claims its identifier. The
/// errors, at paths in the file; empty when the document is applied.
pub(crate) fn check_components(components: &ComponentMap<ClientBiomeComponent>) -> Diagnostics {
    let mut found = Diagnostics::default();
    let cx = &mut found;
    // The music volume is between 0 and 1.
    if let Some(music) = components.get::<BiomeMusic>()
        && music
            .volume_multiplier
            .is_some_and(|volume| !unit(volume.0))
    {
        fail::<BiomeMusic>(VOLUME, Some("volume_multiplier"), cx);
    }
    // The water opacity is between 0 and 1.
    if let Some(water) = components.get::<WaterAppearance>()
        && water
            .surface_opacity
            .is_some_and(|opacity| !unit(opacity.0))
    {
        fail::<WaterAppearance>(OPACITY, Some("surface_opacity"), cx);
    }
    // At least one of the six ambient sounds is named.
    if let Some(sounds) = components.get::<AmbientSounds>()
        && !sounds.has_sound()
    {
        fail::<AmbientSounds>(NO_SOUND, None, cx);
    }
    // At most one precipitation type is present; presence counts, not the value.
    if let Some(particles) = components.get::<Precipitation>() {
        let present = [
            particles.ash,
            particles.blue_spores,
            particles.red_spores,
            particles.white_ash,
        ]
        .iter()
        .filter(|member| member.is_some())
        .count();
        if present > 1 {
            fail::<Precipitation>(PRECIPITATION, None, cx);
        }
    }
    found
}
