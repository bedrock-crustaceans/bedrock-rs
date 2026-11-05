//! Texture lists: `textures/textures_list.json`, the textures a pack asks the client to
//! preload. Vanilla validates nothing here; the client keeps whatever its JSON library made of
//! the file, and falls back to scanning `textures/` when that is empty.
//!
//! The group a stack loads is every pack's list, what the client does with each
//! ([`TextureListMode`]) and the paths the packs claim for preloading ([`TextureClaim`]),
//! which needs the packs' file listings and the layer paths of their texture sets; nothing of
//! the texture set family itself.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod list;
mod load;
mod texts;

use indexmap::IndexMap;

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics};
use bedrock_addon_core::document::Document;

pub use list::{TextureClaim, TextureList, TextureListMode, TextureListRoot, scan_claims};

#[doc(hidden)]
pub use load::LOADER;

/// The texture lists of a resource stack (`rp.texture_lists()`).
#[derive(Debug)]
#[non_exhaustive]
pub struct TextureLists {
    /// The preload lists: each pack's `textures/textures_list.json`, top pack first. A pack
    /// without the file has no entry; with an active subpack that has its own list, the
    /// subpack's file stands for the pack's.
    pub lists: Vec<Document<TextureList>>,
    /// What the client does with each pack's list, in stack order.
    modes: Vec<TextureListMode>,
    /// What the packs claim for preloading ([`claims`](Self::claims)).
    claims: IndexMap<String, TextureClaim>,
    /// The findings that belong to no list.
    diagnostics: Diagnostics,
}

impl TextureLists {
    /// The lists of a stack with what the client makes of them.
    pub(crate) fn new(
        lists: Vec<Document<TextureList>>,
        modes: Vec<TextureListMode>,
        claims: IndexMap<String, TextureClaim>,
        diagnostics: Diagnostics,
    ) -> Self {
        Self {
            lists,
            modes,
            claims,
            diagnostics,
        }
    }

    /// What the client does with the list of the pack at `stack_index` (0 is the top):
    /// preload its entries, or scan the pack's `textures/`.
    #[must_use]
    pub fn list_mode(&self, stack_index: usize) -> Option<&TextureListMode> {
        self.modes.get(stack_index)
    }

    /// The texture paths the stack's packs claim for preloading, each with the highest pack
    /// that claims it: a pack in list mode claims its entries, a pack in scan mode what the
    /// scan of its `textures/` finds. A claim only says which pack asked; the texture itself
    /// is loaded through the whole stack (`TextureFiles::source`). The first claim of a path wins.
    #[must_use]
    pub fn claims(&self) -> &IndexMap<String, TextureClaim> {
        &self.claims
    }

    /// Every finding of the group: the lists, then the rest.
    pub fn diagnostics(&self) -> impl Iterator<Item = &Diagnostic> + '_ {
        self.lists
            .iter()
            .flat_map(|list| list.diagnostics().iter())
            .chain(self.diagnostics.iter())
    }
}
