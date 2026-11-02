//! The attachables' half of the render-controller rule: an attachable is a user of each render
//! controller it names (`render_user.rs` holds the rule).

use super::render_user::Uses;
use super::user::User;

pub(super) fn uses(attachables: &[User<'_>], uses: &mut Uses) {
    for user in attachables {
        uses.add(user);
    }
}
