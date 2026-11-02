//! The blocks of a point-of-interest type (`poi_block.blocks[]`).

use super::super::{Findings, LinkCx};
use crate::families::poi;
use crate::id::BlockId;

/// The blocks of the pass, for the family's checker.
#[derive(Debug)]
struct Blocks<'c, 'a>(&'c LinkCx<'a>);

impl poi::Targets for Blocks<'_, '_> {
    fn block(&self, id: &BlockId) -> bool {
        // The source member's own rule (a namespace is required) was applied at load.
        !self.0.block(id).is_missing()
    }
}

/// Each `blocks` entry of an effective type that names no block of the stack and none of the
/// game's: an `UnresolvedReference` finding at the entry. The type keeps its other blocks.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    out.extend(poi::link(cx.behavior().poi(), &Blocks(cx)));
}
