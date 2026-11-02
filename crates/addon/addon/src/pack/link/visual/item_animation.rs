//! `animations` values name animations and animation controllers of the resource stack, for the description of an attachable.

use super::super::{Findings, LinkCx};
use super::user::User;

pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings, attachables: &[User<'_>]) {
    for user in attachables {
        super::user_animation::check(cx, out, user);
    }
}
