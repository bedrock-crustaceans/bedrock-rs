//! The materials and the render controllers an attachable names.

use super::user::User;
use super::user_render;
use crate::pack::link::prelude::*;

pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings, attachables: &[User<'_>]) {
    for user in attachables {
        user_render::materials(cx, out, user);
    }
    for user in attachables {
        user_render::material_shaders(cx, out, user);
    }
    for user in attachables {
        user_render::render_controllers(cx, out, user);
    }
}
