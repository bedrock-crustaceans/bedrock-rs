//! Texture sets: `<texture>.texture_set.json` beside the texture it describes, the layers
//! (`color`, `metalness_emissive_roughness`, `normal`, `heightmap`, …) the deferred renderer
//! reads. The client has no upgrade path: each of its schemas reads only the documents whose
//! `format_version` is exactly its version ([`history`]).
//!
//! The group a stack loads is the sets of every pack by the texture each sits beside, held to
//! the rules of the stack: each layer's image is in the set's own pack, and the capability lints
//! of the packs that ship sets. Which file the client loads for a texture path, and the images
//! of the layers, are the stack's composition of this family with the texture files.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod history;
mod load;
mod set;
mod set_load;
mod set_read;
mod set_write;
mod texts;

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics};
use bedrock_addon_core::document::{Document, Rejected};

pub use set::{LayerKind, LayerValue, Rgb, Rgba, TextureName, TextureSet};

#[doc(hidden)]
pub use load::load_sets;
#[doc(hidden)]
pub use set_load::{PackSets, SetCopy, layer_path, payload_at};

/// The texture sets of a resource stack (`rp.texture_sets()`).
#[derive(Debug)]
#[non_exhaustive]
pub struct TextureSets {
    /// The texture sets of each pack, in stack order, by the texture path each sits beside.
    sets: Vec<PackSets>,
    /// The findings that belong to no set.
    diagnostics: Diagnostics,
}

impl TextureSets {
    /// The sets of a stack with the findings of its rules.
    pub(crate) fn new(sets: Vec<PackSets>, diagnostics: Diagnostics) -> Self {
        Self { sets, diagnostics }
    }

    /// The texture sets of the stack that loaded, top pack first, each with the texture path
    /// it sits beside. A set a higher pack overrides is listed too: the stack's resolver of a
    /// texture path says which one the client uses.
    pub fn sets(&self) -> impl Iterator<Item = (&str, &Document<TextureSet>)> + '_ {
        self.set_copies()
            .filter_map(|(texture, copy)| Some((texture, copy.as_ref().ok()?)))
    }

    /// The texture sets of the stack that did not load, top pack first: for each the client
    /// shows the missing texture.
    pub fn rejected_sets(&self) -> impl Iterator<Item = (&str, &Rejected)> + '_ {
        self.set_copies()
            .filter_map(|(texture, copy)| Some((texture, copy.as_ref().err()?)))
    }

    /// Every texture set file of the stack, top pack first.
    fn set_copies(&self) -> impl Iterator<Item = (&str, &SetCopy)> + '_ {
        self.sets
            .iter()
            .flat_map(|pack| pack.iter().map(|(texture, copy)| (texture.as_str(), copy)))
    }

    /// The set of the pack at `stack_index` beside the texture `texture`, as it loaded.
    #[doc(hidden)]
    pub fn set_of(&self, stack_index: usize, texture: &str) -> Option<&SetCopy> {
        self.sets.get(stack_index)?.get(texture)
    }

    /// The sets of each pack, in stack order.
    #[doc(hidden)]
    pub fn packs(&self) -> &[PackSets] {
        &self.sets
    }

    /// Every finding of the group: the sets, then the rest.
    pub fn diagnostics(&self) -> impl Iterator<Item = &Diagnostic> + '_ {
        let sets = self.set_copies().flat_map(|(_, copy)| match copy {
            Ok(set) => set.diagnostics().iter(),
            Err(rejected) => rejected.diagnostics.iter(),
        });
        sets.chain(self.diagnostics.iter())
    }
}
