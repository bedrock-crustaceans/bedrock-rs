//! The client entities' half of the render-controller rule: a client entity is a user of each
//! render controller it names (`render_user.rs` holds the rule). An atomic client entity is
//! not: its controllers are judged by what the game says when it draws the entity
//! (`entity_render.rs`).

use super::render_user::Uses;
use super::user::{Family, User};

pub(super) fn uses(entities: &[User<'_>], uses: &mut Uses) {
    for user in entities {
        if user.family == Family::ClientEntity {
            uses.add(user);
        }
    }
}
