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
mod read;
mod set_load;
mod texts;
mod types;
mod write;

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics};
use bedrock_addon_core::document::{Document, Rejected};

pub use types::{LayerKind, LayerValue, Rgb, Rgba, TextureName, TextureSet};

use crate::set_load::{LintPack, capability_lints, check_layer_images, parse_file, texture_path};
use bedrock_addon_core::document::unversioned::origin_of;
use bedrock_addon_core::load::{Accept, CollectionLoader, LoadContext, StackCx, Walk, WalkFile};
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_support::texture::files::{
    IndexedFile, StackFiles, is_texture_file, listed_files,
};
#[doc(hidden)]
pub use set_load::{PackSets, SetCopy, layer_path, payload_at};

/// The texture sets of a resource stack (`rp.texture_sets()`).
///
/// Hand-written: the sets are indexed by the texture each sits beside, per pack, and held to
/// rules that read the packs' listings (each layer's image in the set's own pack) and their
/// capabilities. The file itself is a [`JsonDocument`](bedrock_addon_core::document::JsonDocument).
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

/// One walked texture set file: where it is and what its parse job made of it.
#[derive(Debug)]
pub struct SetFileParsed {
    /// The pack.
    pack: bedrock_addon_core::pack_identity::PackRef,
    /// The path in the pack.
    path: PackPath,
    /// The key the stack reads the file at.
    key: PackPath,
    /// The texture it sits beside and the document, when the file is some texture's set.
    set: Option<(String, Box<SetCopy>)>,
}

/// The files whose content the group reads: every texture set.
fn is_read(path: &PackPath) -> bool {
    path.as_str().ends_with("texture_set.json")
}

/// One file of the walk.
fn parse_walk_file(file: &WalkFile, cx: &LoadContext) -> SetFileParsed {
    let set = texture_path(file.key.as_str()).map(|texture| {
        let copy = parse_file(origin_of(file), &file.bytes, cx);
        (texture.to_owned(), Box::new(copy))
    });
    SetFileParsed {
        pack: file.pack.clone(),
        path: file.path.clone(),
        key: file.key.clone(),
        set,
    }
}

/// The stack rules of the texture sets: each layer's image in the set's own pack, and
/// the capability lints.
fn check_sets(
    sets: &mut [PackSets],
    files: &StackFiles,
    stack: &StackCx<'_>,
    diagnostics: &mut Diagnostics,
) {
    for (index, sets) in sets.iter_mut().enumerate() {
        check_layer_images(sets, index, files);
    }
    let lint_packs = stack.packs.iter().zip(&files.packs);
    let lint_packs: Vec<LintPack<'_>> = lint_packs
        .map(|(entry, files)| LintPack { entry, files })
        .collect();
    capability_lints(&lint_packs, diagnostics);
}

/// The texture sets, from every parsed file in walk order.
fn combine(parsed: Vec<SetFileParsed>, stack: &StackCx<'_>) -> TextureSets {
    let mut files = listed_files(stack);
    let mut sets: Vec<PackSets> = stack.packs.iter().map(|_| PackSets::new()).collect();
    for file in parsed {
        let index = file.pack.index();
        // A file the walk read is a file of its pack even where the listing lacks it.
        let pack = index.and_then(|index| files.packs.get_mut(index));
        if let Some(pack) = pack.filter(|_| is_texture_file(&file.key)) {
            let path = file.path.clone();
            pack.files
                .insert(file.key.as_str().to_owned(), IndexedFile { path });
        }
        let slot = index.and_then(|index| sets.get_mut(index));
        if let (Some((texture, copy)), Some(slot)) = (file.set, slot) {
            slot.insert(texture, *copy);
        }
    }
    let mut diagnostics = Diagnostics::default();
    check_sets(&mut sets, &files, stack, &mut diagnostics);
    TextureSets::new(sets, diagnostics)
}

/// The loader of `rp.texture_sets()`: every texture set of every resource pack, held to the
/// rules of the stack.
#[doc(hidden)]
pub const LOADER: CollectionLoader<TextureSets, SetFileParsed> = CollectionLoader {
    walk: Walk::json_tree_in(&[""]).accept(Accept::Fn(is_read)),
    parse: parse_walk_file,
    combine,
    needs: &[],
    then: None,
};

impl bedrock_addon_core::load::Reported for TextureSets {
    fn reported(
        &self,
    ) -> Box<dyn Iterator<Item = &bedrock_addon_core::diagnostic::Diagnostic> + '_> {
        Box::new(self.diagnostics())
    }
}
