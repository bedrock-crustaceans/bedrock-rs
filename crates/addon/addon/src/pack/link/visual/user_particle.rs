//! `description.particle_effects` of a client entity, an attachable or an atomic client entity:
//! each value names a particle effect.

use bedrock_addon_core::diagnostic::{Severity, VanillaText};
use bedrock_addon_core::id::ParticleId;

use super::super::{At, Findings, LinkCx, Target, kinds, unresolved};
use super::user::{Family, User};

/// Vanilla's text for a particle the particle group lacks: the file, then the value as
/// written.
const PARTICLE_NOT_FOUND: &str = "Entity .json '{}' referenced particle '{}' that doesn't exist";

/// `particle_effects`: each value names a particle effect.
pub(super) fn check(cx: &LinkCx<'_>, out: &mut Findings, user: &User<'_>) {
    let lint = user.family == Family::Atomic;
    if lint && !cx.lints() {
        return;
    }
    for (key, value) in user.particle_effects.into_iter().flatten() {
        let name = user.folded(value.as_str());
        let Target::Missing(hint) = cx.particle(&ParticleId::parse(&*name)) else {
            continue;
        };
        let path = user.entry("particle_effects", key);
        let at = if lint {
            At::new(user.origin, path, Severity::Info)
        } else {
            // The game logs the miss at the error level and goes on with the next entry: a
            // kept error.
            At::new(user.origin, path, Severity::Error).with_text(VanillaText::Formatted {
                template: PARTICLE_NOT_FOUND,
                args: vec![user.origin.path.to_string(), value.as_str().to_owned()],
            })
        };
        let finding = unresolved(at, kinds::PARTICLE, &name, hint);
        out.push(if lint { finding } else { finding.into_kept() });
    }
}
