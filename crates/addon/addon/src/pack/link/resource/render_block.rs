//! The blocks the local lights of the deferred renderer are keyed by.
//!
//! Each key of `minecraft:local_light_settings` (`local_lighting/local_lighting.json`,
//! merged over the stack) is a block: one of the behaviour stack's or one of the game's. The
//! client checks the keys once every pack and every block is registered, and drops the entry
//! of a key that is no block.
// The check and its text are the deferred-settings family's (`check_links`): this file
// answers what a key names.

use super::super::{Findings, LinkCx};
use crate::families::deferred_settings::{Targets, Texture, check_links};
use crate::id::BlockId;

/// What the local-light keys of the deferred settings name on the stacks of a pass.
#[derive(Debug)]
struct Stack<'c, 'a> {
    cx: &'c LinkCx<'a>,
}

impl Targets for Stack<'_, '_> {
    /// A block a behaviour pack defines or one of the game's.
    fn block(&self, id: &BlockId) -> bool {
        !self.cx.block(id).is_missing()
    }

    /// The caustics textures are another rule's: every path is one frame here.
    fn texture(&self, _: &str) -> Texture {
        Texture::Image {
            width: 1,
            height: 1,
        }
    }
}

/// Each key of the merged local lights must be a block of the
/// behaviour stack or of the game; else `UnresolvedReference`, a kept error with the client's
/// text ("%s is not a block") at the key in the file of the pack whose entry won: the
/// client drops the entry.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    let settings = cx.resource().deferred_settings();
    if settings.local_lighting.lights.is_empty() {
        return;
    }
    out.extend(check_links(settings, &Stack { cx }));
}
