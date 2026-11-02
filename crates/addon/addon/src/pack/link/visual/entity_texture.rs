//! `textures` values are texture paths, for the description of a client entity (and of an atomic client entity).

use super::super::{Findings, LinkCx};
use super::user::User;

pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings, entities: &[User<'_>]) {
    for user in entities {
        super::user_texture::check(cx, out, user);
    }
}
