//! The items an aim-assist preset names: `liquid_targeting_list` and the keys of
//! `item_settings`.

use super::super::LinkCx;
use crate::id::ItemId;

/// Whether `key` (the name lower-cased and without its auxiliary value, as the game registers
/// it) is an item of the stack, the item of a block of the stack or one of the game's.
pub(super) fn resolves(cx: &LinkCx<'_>, key: &str) -> bool {
    !cx.item(&ItemId::parse(key)).is_missing()
}
