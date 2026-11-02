//! The structures a jigsaw document names (`single_pool_element.location`).

use super::super::LinkCx;
use crate::id::StructureId;

/// Whether a pool element's template is a file of the stack
/// (`structures/<location without its namespace>.mcstructure`), one a host says the game
/// has, or a legacy template of a pack the game ships.
pub(super) fn jigsaw_template(cx: &LinkCx<'_>, location: &StructureId) -> bool {
    !cx.structure_template(location).is_missing() || legacy_template(cx, location)
}

/// Whether the game's linked-asset check finds the template as a file: a template of
/// [`jigsaw_template`], or any behaviour pack listing `structures/<location without its
/// namespace>.nbt` or `.mcstructure` (the check registers the packs' files, whether or not
/// generation reads them).
pub(super) fn structure_file(cx: &LinkCx<'_>, location: &StructureId) -> bool {
    if jigsaw_template(cx, location) {
        return true;
    }
    let paths = ["nbt", "mcstructure"]
        .map(|extension| format!("structures/{}.{extension}", location.path()));
    cx.behavior().stack_packs().iter().any(|pack| {
        paths.iter().any(|path| {
            pack.files()
                .binary_search_by(|file| file.key.as_str().cmp(path.as_str()))
                .is_ok()
        })
    })
}

/// Whether a pack the game ships lists `structures/<location without its namespace>.nbt`:
/// the template the game reads when no pack has the `.mcstructure`, from its own packs only.
/// The same file in any other pack is never read and is no template. The listing answers;
/// the bytes are not asked for.
// Only base-game packs' `.nbt` structures are read. A pack's listing is in key order: one
// bisection per pack of the game.
fn legacy_template(cx: &LinkCx<'_>, location: &StructureId) -> bool {
    let path = format!("structures/{}.nbt", location.path());
    cx.behavior()
        .stack_packs()
        .iter()
        .filter(|pack| pack.context.is_base_game_pack)
        .any(|pack| {
            pack.files()
                .binary_search_by(|file| file.key.as_str().cmp(path.as_str()))
                .is_ok()
        })
}
