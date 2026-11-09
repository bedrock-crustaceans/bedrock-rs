//! `particle_effects` values name particle effects, for the description of a client entity (and of an atomic client entity).

use super::user::User;
use crate::pack::link::prelude::*;

pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings, entities: &[User<'_>]) {
    for user in entities {
        super::user_particle::check(cx, out, user);
    }
}
