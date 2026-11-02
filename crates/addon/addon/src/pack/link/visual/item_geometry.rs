//! `geometry` values name geometries, for the description of an attachable.

use super::super::{Findings, LinkCx};
use super::user::User;

pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings, attachables: &[User<'_>]) {
    for user in attachables {
        super::user_geometry::check(cx, out, user);
    }
}
