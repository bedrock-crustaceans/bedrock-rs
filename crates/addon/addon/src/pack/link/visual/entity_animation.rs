//! `animations` values name animations and animation controllers of the resource stack, for the description of a client entity (and of an atomic client entity).

use super::user::User;
use crate::pack::link::prelude::*;

pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings, entities: &[User<'_>]) {
    for user in entities {
        super::user_animation::check(cx, out, user);
    }
}
