//! Loading the texture sets of a resource stack: one walk over the `texture_set.json` files of
//! every pack, one parse job per file, and one combine that indexes the sets by the texture
//! each sits beside and holds each to the stack rules. An image is never read: the stack lists
//! the packs' files, and every rule of the load needs only which pack has which path. A pack's
//! capabilities are the stack entry's (`StackPack::capabilities`).

use bedrock_addon_core::load::{CollectionLoader, LoadContext, StackCx, WalkFile};
use bedrock_addon_core::pack_identity::PackPath;

use super::TextureSets;
use super::set_load::{
    LintPack, PackSets, SetCopy, capability_lints, check_layer_images, parse_file, texture_path,
};
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_support::texture::document::origin_of;
use bedrock_addon_support::texture::files::{
    IndexedFile, StackFiles, is_texture_file, listed_files,
};
use bedrock_addon_support::texture::walk::walk;

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
pub const fn load_sets() -> CollectionLoader<TextureSets, SetFileParsed> {
    CollectionLoader {
        walk: walk(is_read, None),
        parse: parse_walk_file,
        combine,
        needs: &[],
        then: None,
    }
}

impl bedrock_addon_core::load::Reported for TextureSets {
    fn reported(
        &self,
    ) -> Box<dyn Iterator<Item = &bedrock_addon_core::diagnostic::Diagnostic> + '_> {
        Box::new(self.diagnostics())
    }
}
