//! Flipbooks: `textures/flipbook_textures.json`, the animated tiles of the terrain atlas. The
//! file of every pack is merged entry by entry from the lowest pack up
//! and the merged array is read once.
//! What a flipbook says of the terrain atlas and of its image is a link rule: the family
//! knows neither.
//!
//! The reader stays a hand walk over the merged array, as the client reads it after its own merge:
//! no schema stands behind it.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod codes;
mod flipbook;

use crate::flipbook::{FlipbookCopy, PATH, merge_flipbooks, parse_copy};
use bedrock_addon_core::document::{Merged, RawLayer};
use bedrock_addon_core::load::{CollectionLoader, LoadContext, StackCx, Walk, WalkFile};
pub use flipbook::{Flipbook, FlipbookKey, Flipbooks};

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
///
/// A hand loader rather than `load::merged`: a copy is the file's JSON as written, merged into
/// the lower copies before the merged array is read once, not a document read on its own.
#[doc(hidden)]
pub const LOADER: CollectionLoader<Merged<Flipbooks, RawLayer>, FlipbookCopy> = CollectionLoader {
    walk: Walk::exact_files(&[PATH]),
    parse,
    combine,
    needs: &[],
    then: None,
};
