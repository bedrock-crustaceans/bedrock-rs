//! The blocks a jigsaw document names: a jigsaw processor's blocks.

use crate::id::BlockId;
use crate::pack::link::prelude::*;

/// Whether a block of a processor (`block_ignore.blocks[]`, `block_match.block`,
/// `random_block_match.block`, `output_state`, `blockstate_match.block_state`) is a block of
/// the stack or one of the game's.
pub(super) fn jigsaw_block(cx: &LinkCx<'_>, id: &BlockId) -> bool {
    !cx.block(id).is_missing()
}
