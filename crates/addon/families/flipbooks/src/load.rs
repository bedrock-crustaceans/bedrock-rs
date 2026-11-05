//! Loading the flipbooks of a resource stack: `textures/flipbook_textures.json` of every pack,
//! merged entry by entry from the lowest pack up.

use bedrock_addon_core::load::{CollectionLoader, LoadContext, StackCx, WalkFile};
use bedrock_addon_core::pack_identity::PackPath;

use super::flipbook::{FlipbookCopy, Flipbooks, PATH, merge_flipbooks, parse_copy};
use bedrock_addon_core::document::{Merged, RawLayer};
use bedrock_addon_support::texture::walk::walk;

/// The file the walk reads.
fn accepts(path: &PackPath) -> bool {
    path.as_str() == PATH
}

/// One pack's copy of the file.
fn parse(file: &WalkFile, _: &LoadContext) -> FlipbookCopy {
    parse_copy(file)
}

/// The merged flipbooks, from every copy in walk order.
fn combine(copies: Vec<FlipbookCopy>, _: &StackCx<'_>) -> Merged<Flipbooks, RawLayer> {
    merge_flipbooks(copies)
}

/// The loader of `rp.flipbooks()`: every pack's `textures/flipbook_textures.json`, merged
/// entry by entry from the lowest pack up.
pub const LOADER: CollectionLoader<Merged<Flipbooks, RawLayer>, FlipbookCopy> = CollectionLoader {
    walk: walk(accepts, None),
    parse,
    combine,
    needs: &[],
    then: None,
};
