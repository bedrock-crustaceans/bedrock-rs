//! `description.animations` of a client entity, an attachable or an atomic client entity: each
//! value names an animation or an animation controller of the resource stack. The game
//! resolves it when it plays and logs nothing: a note.

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::id::AnimationRef;

use super::user::User;
use crate::pack::link::prelude::*;

/// `animations`: each value names an animation or an animation controller of the resource
/// stack, its prefix saying which; a value with neither prefix is ignored by the game and
/// by this crate.
pub(super) fn check(cx: &LinkCx<'_>, out: &mut Findings, user: &User<'_>) {
    if !cx.lints() {
        return;
    }
    for (key, value) in user.animations.into_iter().flatten() {
        let name = user.folded(value.as_str());
        let target = cx.animation(
            crate::pack::PackSide::Resource,
            &AnimationRef::parse(&*name),
        );
        if let Some(Target::Missing(hint)) = target {
            let at = At::new(user.origin, user.animation_path(key), Severity::Info);
            out.push(unresolved(at, kinds::ANIMATION, &name, hint));
        }
    }
}
