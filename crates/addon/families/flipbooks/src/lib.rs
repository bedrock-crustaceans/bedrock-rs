//! Flipbooks: `textures/flipbook_textures.json`, the animated tiles of the terrain atlas. The
//! file of every pack is merged entry by entry from the lowest pack up
//! and the merged array is read once.
//! What a flipbook says of the terrain atlas and of its image is a link rule: the family
//! knows neither.
//!
//! An entry is read by the derived reader of [`Flipbook`] under `Scalars::Lenient` (the client's
//! jsoncpp coerces every member and refuses none) and a silent `null`: once per pack's copy, for
//! the findings that name the pack, and once on the merged array, for the value. What stays hand
//! code is what is not an entry: the file's root, the elements that are no object, the clamp of
//! `ticks_per_frame` and the entries of one key.

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
