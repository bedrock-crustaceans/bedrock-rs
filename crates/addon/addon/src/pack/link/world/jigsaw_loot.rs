//! The loot tables world generation names (`append_loot.loot_table` of a jigsaw processor
//! rule).

use super::super::LinkCx;
use crate::id::LootTablePath;

/// Whether the loot table a processor rule appends is a table of the stack (compared
/// without case, with or without its `.json`) or one of the game's.
pub(super) fn jigsaw_loot_table(cx: &LinkCx<'_>, path: &LootTablePath) -> bool {
    !cx.loot_table(path.as_str()).is_missing()
}
