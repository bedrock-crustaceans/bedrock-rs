//! `textures` values are texture paths, for the description of an attachable.

use super::user::User;
use crate::pack::link::prelude::*;

pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings, attachables: &[User<'_>]) {
    for user in attachables {
        super::user_texture::check(cx, out, user);
    }
}
