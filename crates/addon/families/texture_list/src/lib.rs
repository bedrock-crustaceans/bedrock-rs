//! Texture lists: `textures/textures_list.json`, the textures a pack asks the client to
//! preload. Vanilla validates nothing here; the client keeps whatever its JSON library made of
//! the file, and falls back to scanning `textures/` when that is empty.
//!
//! The group a stack loads is every pack's list, what the client does with each
//! ([`TextureListMode`]) and the paths the packs claim for preloading ([`TextureClaim`]),
//! which needs the packs' file listings and the layer paths of their texture sets; nothing of
//! the texture set family itself.
//!
//! The reader stays a hand walk: the client keeps whatever its JSON library made of the file and
//! validates nothing, so there is no schema to follow.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod codes;
mod list;

use indexmap::IndexMap;

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics};
use bedrock_addon_core::document::Document;

use crate::list::PackLint;
use bedrock_addon_core::document::Origin;
use bedrock_addon_core::document::unversioned;
use bedrock_addon_core::json::jsoncpp::lenient::read_lenient;
use bedrock_addon_core::load::{Accept, CollectionLoader, LoadContext, StackCx, Walk, WalkFile};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_shared_shapes::texture::files::{
    IndexedFile, StackFiles, is_texture_file, listed_files,
};
pub use list::{TextureClaim, TextureList, TextureListMode, TextureListRoot, scan_claims};

/// The texture lists of a resource stack (`rp.texture_lists()`).
///
/// Hand-written: one walk reads two file kinds (the lists, and the texture sets for the layer
/// paths the client's scan leaves out), and the claims come from the packs' listings.
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

/// What one walked file is.
#[derive(Debug)]
pub(crate) enum Parsed {
    /// A texture list, with what reading it found.
    List(Box<(TextureList, Diagnostics)>),
    /// A texture set: the image paths its layers name, folder included.
    Set(Vec<String>),
}

/// One walked file: where it is, and what its parse job made of it.
#[derive(Debug)]
pub struct ListFileParsed {
    /// The pack.
    pack: PackRef,
    /// The path in the pack.
    path: PackPath,
    /// The key the stack reads the file at.
    key: PackPath,
    /// The file's content.
    parsed: Parsed,
}

/// The files whose content the group reads: the list and every texture set.
fn is_read(path: &PackPath) -> bool {
    let path = path.as_str();
    path == list::PATH || path.ends_with("texture_set.json")
}

/// The pack's own texture list, which the client reads beside the active subpack's: the
/// subpack's elements are appended to it.
fn is_list(path: &PackPath) -> bool {
    path.as_str() == list::PATH
}

/// One file of the walk.
fn parse_walk_file(file: &WalkFile, _: &LoadContext) -> ListFileParsed {
    let key = file.key.as_str();
    let parsed = if key == list::PATH {
        let mut findings = Diagnostics::default();
        let read = TextureList::read(Some(&file.bytes), &mut findings);
        Parsed::List(Box::new((read, findings)))
    } else {
        Parsed::Set(layer_paths(key, &file.bytes))
    };
    ListFileParsed {
        pack: file.pack.clone(),
        path: file.path.clone(),
        key: file.key.clone(),
        parsed,
    }
}

/// The image paths the layers of the texture set at `key` name: each string layer that is no
/// hex colour, behind the set's folder. What the client's scan leaves out of its claims.
fn layer_paths(key: &str, bytes: &[u8]) -> Vec<String> {
    let folder = key.rsplit_once('/').map_or("", |(folder, _)| folder);
    let Ok(read) = read_lenient(bytes) else {
        return Vec::new();
    };
    let layers = read
        .value
        .as_object()
        .and_then(|root| root.get("minecraft:texture_set"))
        .and_then(bedrock_addon_core::json::Value::as_object);
    layers
        .into_iter()
        .flatten()
        .filter_map(|(_, layer)| layer.as_string())
        .map(facet_value::VString::as_str)
        .filter(|layer| !layer.starts_with('#'))
        .map(|layer| format!("{folder}/{layer}"))
        .collect()
}

/// The lists read of one pack: its own, and its active subpack's, each with what reading it
/// found and its path.
#[derive(Default)]
struct PackLists {
    /// The list at the pack's own `textures/textures_list.json`.
    own: Option<(TextureList, Diagnostics, PackPath)>,
    /// The list of the active subpack, read at the same key.
    subpack: Option<(TextureList, Diagnostics, PackPath)>,
}

/// The texture paths the packs claim for preloading, each with the highest pack that claims
/// it: a pack in list mode claims its entries, a pack in scan mode what the scan of its
/// `textures/` finds.
fn compute_claims(
    files: &StackFiles,
    modes: &[TextureListMode],
    layers: &[Vec<String>],
) -> IndexMap<String, TextureClaim> {
    let mut claims = IndexMap::new();
    for (stack_index, pack) in files.packs.iter().enumerate() {
        let scanned;
        let locations: &[String] =
            if let Some(TextureListMode::List { entries }) = modes.get(stack_index) {
                entries
            } else {
                let layers = layers.get(stack_index).map_or(&[][..], Vec::as_slice);
                // The client scans the active subpack's directory, then the pack's own.
                // A subpack's file stands at a key that is not its path.
                let of = |subpack: bool| {
                    pack.files
                        .iter()
                        .filter(move |(key, file)| (file.path.as_str() != *key) == subpack)
                        .map(|(key, _)| key.as_str())
                };
                scanned = scan_claims(of(true).chain(of(false)), layers);
                &scanned
            };
        for location in locations {
            claims
                .entry(location.clone())
                .or_insert_with(|| TextureClaim {
                    location: location.clone(),
                    pack: pack.pack.clone(),
                    stack_index,
                });
        }
    }
    claims
}

/// The lists, from every parsed file in walk order.
fn combine(parsed: Vec<ListFileParsed>, stack: &StackCx<'_>) -> TextureLists {
    let mut files = listed_files(stack);
    let mut read_lists: Vec<PackLists> = stack.packs.iter().map(|_| PackLists::default()).collect();
    let mut layers: Vec<Vec<String>> = stack.packs.iter().map(|_| Vec::new()).collect();
    for file in parsed {
        let index = file.pack.index();
        // A file the walk read is a file of its pack even where the listing lacks it.
        let pack = index.and_then(|index| files.packs.get_mut(index));
        if let Some(pack) = pack.filter(|_| is_texture_file(&file.key)) {
            let path = file.path.clone();
            pack.files
                .insert(file.key.as_str().to_owned(), IndexedFile { path });
        }
        match file.parsed {
            Parsed::List(read) => {
                if let Some(slot) = index.and_then(|index| read_lists.get_mut(index)) {
                    let (list, findings) = *read;
                    // The pack's own list is the file at its own path; the other is the
                    // active subpack's, read at the same key.
                    let own = file.path == file.key;
                    let slot = if own {
                        &mut slot.own
                    } else {
                        &mut slot.subpack
                    };
                    *slot = Some((list, findings, file.path));
                }
            }
            Parsed::Set(paths) => {
                if let Some(slot) = index.and_then(|index| layers.get_mut(index)) {
                    slot.extend(paths);
                }
            }
        }
    }
    let mut diagnostics = Diagnostics::default();
    let mut lists = Vec::new();
    let mut modes = Vec::new();
    for (index, read) in read_lists.into_iter().enumerate() {
        let (Some(pack), Some(entry)) = (files.packs.get(index), stack.packs.get(index)) else {
            continue;
        };
        let lint = PackLint {
            files: pack,
            layers: layers.get(index).map_or(&[], Vec::as_slice),
            base_game: entry.context.is_base_game_pack,
        };
        let missing = TextureList::default();
        let own = read.own.as_ref().map_or(&missing, |(own, _, _)| own);
        let subpack = read.subpack.as_ref().map(|(subpack, _, _)| subpack);
        // The subpack's elements after the pack's; a subpack list that contributes nothing
        // leaves the pack's list as it is.
        let mode = own.mode(subpack);
        lint.pack(mode == TextureListMode::Scan, &mut diagnostics);
        // The omitted-images lint is reported for the list the client uses, on the last list
        // that put entries in it.
        let appended = subpack.is_some_and(|subpack| !subpack.is_empty());
        let mut copies = [(read.own, !appended), (read.subpack, appended)];
        for (copy, last) in &mut copies {
            let Some((document, mut findings, path)) = copy.take() else {
                continue;
            };
            lint.stale(&document, &mut findings);
            if *last && let TextureListMode::List { entries } = &mode {
                lint.omitted(entries, &mut findings);
            }
            let origin = Origin::file(pack.pack.clone(), PackSide::Resource, path);
            lists.push(unversioned::document(document, origin, findings));
        }
        modes.push(mode);
    }
    let claims = compute_claims(&files, &modes, &layers);
    TextureLists::new(lists, modes, claims, diagnostics)
}

/// The loader of `rp.texture_lists()`: every pack's `textures/textures_list.json`, with what
/// the client's scan of each pack's `textures/` claims.
#[doc(hidden)]
pub const LOADER: CollectionLoader<TextureLists, ListFileParsed> = CollectionLoader {
    walk: Walk::json_tree_in(&[""])
        .accept(Accept::Fn(is_read))
        .shadowed(is_list),
    parse: parse_walk_file,
    combine,
    needs: &[],
    then: None,
};

impl bedrock_addon_core::load::Reported for TextureLists {
    fn reported(
        &self,
    ) -> Box<dyn Iterator<Item = &bedrock_addon_core::diagnostic::Diagnostic> + '_> {
        Box::new(self.diagnostics())
    }
}
