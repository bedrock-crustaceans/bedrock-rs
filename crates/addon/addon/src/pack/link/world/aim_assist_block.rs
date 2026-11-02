//! The blocks an aim-assist document names: `exclusion_settings.blocks`, the entries of
//! an old `exclusion_list`, and the keys of a category's `priorities.blocks`.

use super::super::LinkCx;
use crate::id::BlockId;

/// Whether `key` (the name lower-cased, as the game registers it) is a block of the stack or
/// one of the game's.
pub(super) fn resolves(cx: &LinkCx<'_>, key: &str) -> bool {
    !cx.block(&BlockId::parse(key)).is_missing()
}
