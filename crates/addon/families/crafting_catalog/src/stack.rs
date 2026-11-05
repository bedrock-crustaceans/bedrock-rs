//! `bp.crafting_catalog()`: which files of a stack are read, the item registry the merge
//! resolves names against, and the merge of the files.
//!
//! Vanilla merges the catalogue after item registration, against its item registry. Here the
//! registry is what a behaviour stack says about it: the items its packs define, the blocks its
//! packs define (a block has an item of its own name), and the built-in items.
//!
//! The catalogue reads its one file and asks the stack for the rest (`StackCx::registered`): an
//! item is registered exactly when `bp.items()` has it and a block exactly when
//! `bp.blocks()` has it, stable or beta, so a definition its own family rejects registers
//! nothing. No other content module is named: the loader declares the two identifier kinds it
//! needs, and the stack combines the collections that register them first. Without the `block`
//! feature no collection registers blocks: a name that is no item and could be a pack's block
//! is kept without a warning, since the catalogue cannot know.

use bedrock_addon_core::load::{
    Accept, AppDataPlace, BuiltIns, CollectionLoader, ListDepth, LoadContext, Registration,
    StackCx, StackPack, Walk, WalkFile, WalkOrder,
};
use bedrock_addon_core::pack_identity::PackPath;
use uuid::Uuid;

use super::load::parse_file;
use super::merge::{self, Items, Merger};
use super::{CatalogKey, CraftingCatalog, PATH};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticDetail, Location, Phase, Severity,
};
use bedrock_addon_core::document::{Document, Merge, Merged, Rejected};
use bedrock_addon_core::id::{BlockId, ItemId, Kind, kind};

impl Merge for CraftingCatalog {
    type Key = CatalogKey;
}

/// One catalogue file of the walk, loaded or rejected.
pub type CatalogFile = Result<Document<CraftingCatalog>, Rejected>;

/// The catalogue file: the one path, as written.
fn is_catalog(path: &PackPath) -> bool {
    path.as_str() == PATH
}

/// `item_catalog/crafting_item_catalog.json` of every pack; packs bottom-up.
const WALK: Walk = Walk {
    directories: &["item_catalog"],
    depth: ListDepth::Recursive,
    accept: Accept::Fn(is_catalog),
    order: WalkOrder::BottomUp,
    directory_major: false,
    app_data: &[],
    app_data_place: AppDataPlace::After,
    in_pack: None,
    shadowed: None,
    read_error: None,
};

/// The identifier kinds the merge asks the stack about: the items and the blocks its packs
/// registered.
const NEEDS: &[&str] = &[<kind::Item as Kind>::NAME, <kind::Block as Kind>::NAME];

/// The parse step: one pack's catalogue file.
fn parse_walk_file(file: &WalkFile, cx: &LoadContext) -> CatalogFile {
    parse_file(file.pack.clone(), file.side, &file.path, &file.bytes, cx)
}

/// The items and blocks of a stack.
struct StackItems<'a> {
    /// The stack, with its item and block registrations in view.
    stack: &'a StackCx<'a>,
    /// The host's built-in content, when it gave one.
    built_ins: Option<&'a dyn BuiltIns>,
}

impl Items for StackItems<'_> {
    fn is_registered(&self, id: &ItemId) -> bool {
        if self.stack.registered(id).is_registered() {
            return true;
        }
        // A block has an item of its own name.
        let name = format!("{}:{}", id.namespace().unwrap_or_default(), id.path());
        let block = BlockId::parse(name.as_str());
        match self.stack.registered(&block) {
            Registration::By { .. } => return true,
            // No block collection in view (the `block` feature is off): a pack's block cannot
            // be known, so a name that may be one is kept without a warning. A
            // `minecraft:` name cannot be a pack's block (the block family refuses it), so it
            // is judged by the built-in items below.
            Registration::NotInView if !block.is_vanilla() => return true,
            _ => {}
        }
        // The built-in items include the blocks that have an item; a block without one
        // (`minecraft:fire`) is no item.
        match self.built_ins {
            Some(built_ins) => built_ins.contains(<kind::Item as Kind>::NAME, &name),
            // Without a host: the vanilla item identifiers the crate was generated with.
            None => bedrock_addon_core::vanilla::ids()
                .items
                .contains(&name.as_str()),
        }
    }
}

/// The header UUID of the vanilla behaviour pack and of every versioned slice of it
/// (`VANILLA_BEHAVIOR_PACK_UUID`).
const VANILLA_BEHAVIOR_PACK: Uuid = Uuid::from_u128(0xfe9f8597_5454_481a_8730_8d070a8e2e58);

/// The stack index of the pack whose file is the base catalogue: from the
/// topmost base-game pack with the vanilla UUID, the first pack downwards that has the file.
/// `None` when the stack has no such vanilla pack, or none at or below it has the file.
// A non-vanilla pack on the way down is a debug-only assert in vanilla; it is still tested
// for the file.
fn base(packs: &[StackPack], has_file: impl Fn(usize) -> bool) -> Option<usize> {
    let topmost_vanilla = packs.iter().position(|pack| {
        pack.context.is_base_game_pack && pack.pack.id().uuid == VANILLA_BEHAVIOR_PACK
    })?;
    (topmost_vanilla..packs.len()).find(|&index| has_file(index))
}

/// The stack index of the pack a parsed file came from.
fn pack_index(file: &CatalogFile) -> Option<usize> {
    match file {
        Ok(document) => document.origin().pack.index(),
        Err(rejected) => rejected.origin.pack.index(),
    }
}

/// The note of a stack whose files merge without a base catalogue. Vanilla
/// has a debug assert here and no content-log line, so the crate reports an information.
// vanilla = "debug assert; no catalog loaded"
fn no_base() -> Diagnostic {
    let mut warning = Diagnostic::new(
        DiagnosticCode::UnresolvedReference { text: None },
        Severity::Info,
        Phase::Validate,
        Location::default(),
    );
    warning.detail = Some(Box::new(DiagnosticDetail::Reference {
        kind: "base crafting item catalog",
        id: PATH.to_owned(),
        hint: None,
    }));
    warning
}

/// The stack's catalogue: the base file first, then the file of every pack above it, the top
/// pack last. Packs below the base are not read. Without a base the files of the stack merge
/// onto empty categories, with one warning.
fn combine(files: Vec<CatalogFile>, stack: &StackCx<'_>) -> Merged<CraftingCatalog> {
    let registry = StackItems {
        stack,
        built_ins: stack.options.built_ins.as_deref(),
    };
    let with_file: Vec<usize> = files.iter().filter_map(pack_index).collect();
    let base = base(stack.packs, |index| with_file.contains(&index));
    // The files are in walk order: bottom-up.
    let walk = files
        .into_iter()
        .filter(|file| match (base, pack_index(file)) {
            (Some(base), Some(index)) => index <= base,
            _ => true,
        })
        .collect();
    let mut merger = Merger::new(&registry);
    let mut warn = base.is_none();
    Merged::merge(walk, merge::empty(), |merged, layer, cx| {
        if std::mem::take(&mut warn) {
            cx.push_diagnostic(no_base());
        }
        merger.layer(merged, layer, cx);
    })
}

/// The loader of `bp.crafting_catalog()`.
pub const fn load() -> CollectionLoader<Merged<CraftingCatalog>, CatalogFile> {
    CollectionLoader {
        walk: WALK,
        parse: parse_walk_file,
        combine,
        needs: NEEDS,
        then: None,
    }
}
