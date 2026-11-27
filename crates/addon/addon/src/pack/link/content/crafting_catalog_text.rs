//! The localisation keys the crafting catalogue names.

use crate::pack::link::prelude::*;

/// The name of a group a pack adds to the crafting catalogue, outside the `minecraft`
/// namespace, is its localisation key. Vanilla does not look it up: a lint, told by the
/// catalogue family's checker for each name no `.lang` file of either stack defines.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    if !cx.lints() {
        return;
    }
    let catalog = cx.behavior().crafting_catalog();
    let defined = |key: &str| !cx.lang_key(key).is_missing();
    out.extend(crate::families::crafting_catalog::link_group_names(catalog, &defined));
}
