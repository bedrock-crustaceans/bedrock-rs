//! The entities an aim-assist document names: `exclusion_settings.entities`, the
//! entries of an old `exclusion_list`, and the keys of a category's `priorities.entities`.

use crate::id::EntityId;
use crate::pack::link::prelude::*;

/// Whether `key` (the name lower-cased and without its `<event>`, as the game registers it)
/// is an entity of the stack or one of the game's.
pub(super) fn resolves(cx: &LinkCx<'_>, key: &str) -> bool {
    !cx.entity(&EntityId::parse(key)).is_missing()
}
