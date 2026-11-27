//! Structures (`.mcstructure`): `Structure`.
//!
//! Structures: `structures/**/*.mcstructure` of a behaviour pack, little-endian NBT read by
//! the game's structure loader. One format (1), so there is no version chain and no `history`.
//!
//! The reader stays binary: the file is NBT, with no JSON for a typed read to take.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod codes;
mod read;
mod text;
mod validate;
mod write;

use std::collections::BTreeMap;

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::load::{Accept, CollectionLoader, LoadContext, StackCx, Walk, WalkFile};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use indexmap::IndexMap;

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, JsonPath, Phase, builders};
use bedrock_addon_core::document::{
    Content, Document, Files, FilesBuilder, FilesRule, Origin, Rejected, VersionInfo, unversioned,
};
use bedrock_addon_core::id::StructureId;
use bedrock_addon_core::json::{EncodeError, Unrepresentable, WriteStyle};
use bedrock_addon_core::version::FormatVersion;

/// A structure template (`.mcstructure`): a box of block indices into named palettes, plus the
/// entities saved with it.
///
/// NBT (little-endian, root name empty): `{format_version: 1, size: [x, y, z],
/// structure_world_origin: [x, y, z], structure: {block_indices, entities, palette},
/// neighbor_aware_block_upgrade_version?}`. Every rule of the format fails the whole file;
/// vanilla then tries the same structure in the next pack of the stack.
#[derive(Clone, Debug, PartialEq)]
#[doc(alias = "mcstructure")]
pub struct Structure {
    /// The format (NBT `format_version`, Int): always [`Structure::FORMAT_VERSION`].
    pub format_version: i32,
    /// Which neighbour-aware block upgrades the blocks already had (NBT
    /// `neighbor_aware_block_upgrade_version`, Int, 0 or 1). Absent reads as 0: walls are
    /// re-connected to their neighbours when the structure is placed; 1 places them as stored.
    pub neighbor_aware_block_upgrade_version: Option<i32>,
    /// The size of the box in blocks, x, y, z (NBT `size`, three Ints, none negative).
    pub size: [i32; 3],
    /// Where the box's minimum corner was in the world it was saved from (NBT
    /// `structure_world_origin`, three Ints).
    pub structure_world_origin: [i32; 3],
    /// The blocks, palettes and entities (NBT `structure`).
    pub structure: Contents,
    /// Root tags the format does not define, kept for write-back.
    pub extra: nbtx::Compound,
    /// How the file was laid out (tag order, integer widths, empty-list tags), so an unchanged
    /// structure is written back byte for byte; not part of equality. Public only so that
    /// struct-update syntax (`..Structure::new(size)`) works outside the crate; `Default` is
    /// vanilla's layout.
    #[doc(hidden)]
    pub layout: Layout,
}

/// The blocks, palettes and entities of a [`Structure`] (NBT `structure`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Contents {
    /// Two layers of palette indices, one per block position of the box (NBT
    /// `block_indices`, a list of exactly two lists of Ints). Layer 0 is the block, layer 1
    /// the extra block in the same position (water in a waterlogged block). Positions run z
    /// fastest, then y, then x ([`Structure::index`]); a negative index leaves the world's
    /// block in place, an index past the palette places air. Both layers have the same
    /// length, the box's volume, or are both empty.
    pub block_indices: [Vec<i32>; 2],
    /// The entities, as raw actor NBT (NBT `entities`, a list of compounds; not validated).
    pub entities: Vec<nbtx::Compound>,
    /// The palettes by name (NBT `palette`); [`Structure::DEFAULT_PALETTE`] is the one placed.
    pub palette: IndexMap<String, Palette>,
    /// Tags the format does not define, kept for write-back.
    pub extra: nbtx::Compound,
}

/// One palette of a [`Structure`]: the blocks its indices name and per-position data.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Palette {
    /// The blocks, as raw block NBT `{name, states, version}` (NBT `block_palette`, a list of
    /// compounds). Kept as written: vanilla resolves an entry, upgrading it by its `version`,
    /// only when the structure is placed.
    pub block_palette: Vec<nbtx::Compound>,
    /// Block entities and scheduled ticks by position index (NBT `block_position_data`, a
    /// compound keyed by the index in decimal).
    pub block_position_data: BTreeMap<i32, BlockPositionData>,
    /// Tags the format does not define, kept for write-back.
    pub extra: nbtx::Compound,
}

/// What one position of a [`Palette`] carries besides its block.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlockPositionData {
    /// The block entity, as raw NBT (NBT `block_entity_data`, a compound).
    pub block_entity_data: Option<nbtx::Compound>,
    /// Scheduled block ticks (NBT `tick_queue_data`, a list of compounds).
    pub tick_queue_data: Option<Vec<TickQueueEntry>>,
    /// Tags the format does not define, kept for write-back.
    pub extra: nbtx::Compound,
}

/// One scheduled tick of a block position.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TickQueueEntry {
    /// Ticks until the block ticks (NBT `tick_delay`, Int); absent, or not an Int, reads as 0
    /// ([`TickQueueEntry::tick_delay`]).
    pub tick_delay: Option<i32>,
    /// Tags the format does not define, kept for write-back.
    pub extra: nbtx::Compound,
}

/// The on-disk layout of a loaded [`Structure`], kept so that write-back reproduces the file.
///
/// Empty for a file already in vanilla's layout and for a structure built in code.
#[doc(hidden)]
#[derive(Clone, Default)]
pub struct Layout {
    /// The file's tree, root name included, when it differs from what the writer produces
    /// from the typed value.
    pub(crate) source: Option<std::sync::Arc<nbtx::Named<nbtx::Value>>>,
    /// Bytes after the root tag, which vanilla does not read.
    pub(crate) trailing: Box<[u8]>,
}

/// Layouts never decide equality: two structures with the same content are equal however their
/// files were laid out.
impl PartialEq for Layout {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

/// Only whether a layout is kept.
impl std::fmt::Debug for Layout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Layout")
            .field("kept", &self.source.is_some())
            .finish_non_exhaustive()
    }
}

impl Structure {
    /// The only format vanilla reads.
    pub const FORMAT_VERSION: i32 = 1;
    /// The palette vanilla places.
    pub const DEFAULT_PALETTE: &'static str = "default";
    /// The namespace of a structure file directly under `structures/`.
    pub const DEFAULT_NAMESPACE: &'static str = "mystructure";

    /// [`neighbor_aware_block_upgrade_version`](Self::neighbor_aware_block_upgrade_version),
    /// 0 when absent.
    pub fn neighbor_aware_block_upgrade_version(&self) -> i32 {
        self.neighbor_aware_block_upgrade_version.unwrap_or(0)
    }
}

impl TickQueueEntry {
    /// [`tick_delay`](Self::tick_delay), 0 when absent.
    pub fn tick_delay(&self) -> i32 {
        self.tick_delay.unwrap_or(0)
    }
}

#[cfg(test)]
mod tests;

impl bedrock_addon_core::__private::Sealed for Structure {}

impl Content for Structure {
    const NAME: &'static str = "structure";
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        parse_file(PackRef::app_data(), PackSide::Behavior, path, bytes, cx)
    }

    /// Writes the structure's file: little-endian NBT, not JSON (the file kind has no JSON
    /// form). An unchanged loaded structure is written back byte for byte.
    fn write_json(&self, out: &mut Vec<u8>, _: WriteStyle) -> Result<(), EncodeError> {
        out.extend(self.to_bytes()?);
        Ok(())
    }

    /// Always `NotRepresentable` (`Unsupported`): a structure is binary NBT; use
    /// [`write_json`](Content::write_json) for its bytes.
    fn to_json(&self) -> Result<String, EncodeError> {
        Err(EncodeError::NotRepresentable {
            at: JsonPath::new(),
            what: Unrepresentable::Unsupported,
        })
    }
}

/// The version information of a loaded structure: format 1, written as an Int, never upgraded.
fn version() -> VersionInfo {
    unversioned::version_at(FormatVersion::new(1, 0, 0))
}

/// Loads one structure file of `pack`.
pub(crate) fn parse_file(
    pack: PackRef,
    side: PackSide,
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> Result<Document<Structure>, Rejected> {
    let origin = Origin::file(pack, side, path.clone());
    let (value, mut diagnostics) = read::read(bytes, cx);
    diagnostics.locate(&origin);
    match value {
        Some(structure) => Ok(Document::new(structure, origin, version(), diagnostics)),
        None => Err(Rejected::new(origin, diagnostics)),
    }
}

// ---- lookups by identifier ---------------------------------------------------------------

/// The folder below a pack root that holds structures (`BEHAVIOR_PACK_STRUCTURES_FOLDER`).
const FOLDER: &str = "structures";
/// The extension of a structure file, compared exactly.
const EXTENSION: &str = ".mcstructure";

impl Structure {
    /// The identifier vanilla lists the file at `path` under, or
    /// `None` for a path that is not `structures/…/<name>.mcstructure` (the extension compared
    /// exactly). A file directly under `structures/` is `mystructure:<name>`; deeper, the first
    /// folder is the namespace and the rest of the path the name:
    /// `structures/stuff/towers/diamond.mcstructure` is `stuff:towers/diamond`.
    pub fn id_of(path: &PackPath) -> Option<StructureId> {
        // Two parts give `mystructure:<stem>`; more give the first folder, then the remaining
        // folders and the stem joined with `/`.
        let rest = path
            .as_str()
            .strip_prefix(FOLDER)?
            .strip_prefix('/')?
            .strip_suffix(EXTENSION)?;
        let id = match rest.split_once('/') {
            None => format!("{}:{rest}", Structure::DEFAULT_NAMESPACE),
            Some((namespace, name)) => format!("{namespace}:{name}"),
        };
        Some(StructureId::parse(id))
    }

    /// The file `/structure load`, structure blocks and scripts read first for `id`:
    /// `structures/<namespace>/<name>.mcstructure`; `None` without a namespace (or for a name
    /// no pack path can hold).
    pub fn path_of(id: &StructureId) -> Option<PackPath> {
        let namespace = namespace(id)?;
        PackPath::new(&format!("{FOLDER}/{namespace}/{}{EXTENSION}", id.path())).ok()
    }

    /// The structure `/structure load`, structure blocks and scripts get for `id`: the stack is
    /// walked from the top; each pack offers [`path_of`](Self::path_of)'s file, or, for a
    /// `mystructure` identifier and only when the pack lacks that file,
    /// `structures/<name>.mcstructure`; the first copy that loads wins, a copy that fails passes
    /// the turn to the next pack. (Where a pack fails its primary copy and has a root copy, the
    /// collection also lists the root copies of the other packs without a primary copy under the
    /// primary path, so its entry there may be a root file.)
    ///
    /// An identifier without a namespace is vanilla's error `Structure '<id>' must be prefixed
    /// by a namespace`. `Ok(None)` when no pack has a copy that loads (vanilla then reads the
    /// world's own saved structures, which are not pack content).
    ///
    /// # Errors
    ///
    /// Returns a [`Diagnostic`] when the identifier has no namespace.
    #[allow(
        clippy::result_large_err,
        reason = "the crate's finding type; one per failed lookup"
    )]
    pub fn find<'a>(
        structures: &'a Files<Structure>,
        id: &StructureId,
    ) -> Result<Option<&'a Document<Structure>>, Diagnostic> {
        // An identifier without a namespace is the error, before any lookup.
        let Some(namespace) = namespace(id) else {
            return Err(Diagnostic::of(text::not_namespaced(id.as_str())).phase(Phase::Link));
        };
        let Some(primary) = Structure::path_of(id) else {
            return Ok(None);
        };
        let first = structures.get(&primary);
        // The root path is tried only for the default namespace.
        if namespace != Structure::DEFAULT_NAMESPACE {
            return Ok(first);
        }
        let Ok(secondary) = PackPath::new(&format!("{FOLDER}/{}{EXTENSION}", id.path())) else {
            return Ok(first);
        };
        // A pack's root copy is read only when the pack has no primary copy. A pack whose
        // primary copy loaded is at or above `first`, which wins the tie below; a pack whose
        // primary copy failed has passed its turn, and the root copies of the packs below it
        // then stand in at the primary path (`stand_ins`), so `first` has them.
        let failed_primary = |pack: &PackRef| {
            structures
                .rejected()
                .iter()
                .any(|r| r.origin.pack == *pack && keyed(&r.origin.path, &primary))
        };
        let second = structures
            .layers(&secondary)
            .find(|d| !failed_primary(&d.origin().pack));
        Ok(match (first, second) {
            (Some(a), Some(b)) => Some(if rank(b) < rank(a) { b } else { a }),
            (a, b) => a.or(b),
        })
    }

    /// The template a jigsaw pool element's `location` names (`readStructure`): the file
    /// `structures/<location without its namespace>.mcstructure`, the top copy that loads.
    /// No `mystructure` rule applies. (When no pack has it, vanilla tries the Java `.nbt`
    /// templates of the base-game packs, which are not pack content here.)
    pub fn find_template<'a>(
        structures: &'a Files<Structure>,
        location: &StructureId,
    ) -> Option<&'a Document<Structure>> {
        // The root path is the name without its namespace.
        let path = PackPath::new(&format!("{FOLDER}/{}{EXTENSION}", location.path())).ok()?;
        structures.get(&path)
    }
}

/// The namespace of `id`, when it has a non-empty one.
fn namespace(id: &StructureId) -> Option<&str> {
    id.namespace().filter(|namespace| !namespace.is_empty())
}

/// A document's place in the stack: its pack's index (0 is the top).
fn rank(document: &Document<Structure>) -> usize {
    document.origin().pack.index().unwrap_or(usize::MAX)
}

/// Whether a file's path in its pack (`origin`, a subpack folder included) is `key`.
pub(crate) fn keyed(origin: &PackPath, key: &PackPath) -> bool {
    let (origin, key) = (origin.as_str(), key.as_str());
    origin == key
        || (origin.starts_with("subpacks/")
            && origin
                .strip_suffix(key)
                .is_some_and(|folder| folder.ends_with('/')))
}

// ---- the collection ----------------------------------------------------------------------

/// What one walked file gives the collection.
#[allow(clippy::large_enum_variant, reason = "one per walked file, moved once")]
#[derive(Clone, Debug)]
#[doc(hidden)]
pub enum Parsed {
    /// A structure file under the path the collection keys it by.
    File(PackPath, Result<Document<Structure>, Rejected>),
    /// A finding about a file the collection does not read.
    Ignored(Diagnostic),
}

/// A structure file, or a Java template (`.nbt`); extensions compared exactly.
#[allow(
    clippy::case_sensitive_file_extension_comparisons,
    reason = "vanilla compares the extension byte for byte"
)]
fn is_structure_or_template(path: &PackPath) -> bool {
    let path = path.as_str();
    path.ends_with(EXTENSION) || path.ends_with(".nbt")
}

/// `structures/**/*.mcstructure` (and `.nbt`, to report it), top pack first.
const WALK: Walk = Walk::json_tree_in(&[FOLDER])
    .accept(Accept::Fn(is_structure_or_template))
    .top_down()
    .in_pack(read_in_pack);

/// Whether the walk reads `path` in the pack loaded with `cx`: every structure file, and a
/// `.nbt` template outside the base-game packs, where it is reported. A base-game pack's
/// templates are vanilla's own jigsaw data, which nothing here reads: they are left out of the
/// walk, so their bytes are never asked for.
//
fn read_in_pack(path: &PackPath, cx: &LoadContext) -> bool {
    path.as_str().ends_with(EXTENSION) || !cx.is_base_game_pack
}

/// One file's contribution. A `.nbt` file is a Java template, which vanilla reads from
/// base-game packs only: the walk leaves it out there ([`read_in_pack`]), and in any other
/// pack it is an `IgnoredFile` info (the crate's own report; the game ignores the file).
fn parse_walk_file(file: &WalkFile, cx: &LoadContext) -> Parsed {
    if !file.path.as_str().ends_with(EXTENSION) {
        return Parsed::Ignored(
            Diagnostic::of(&core_codes::IGNORED_FILE).in_file(&unversioned::origin_of(file)),
        );
    }
    Parsed::File(
        file.key.clone(),
        parse_file(file.pack.clone(), file.side, &file.path, &file.bytes, cx),
    )
}

/// The files by path: the top copy that loads wins; a failed copy's own findings name its pack
///, no fall-back notice is added.
fn combine(parsed: Vec<Parsed>, stack: &StackCx<'_>) -> Files<Structure> {
    let mut builder =
        FilesBuilder::new(FilesRule::TopWinsFallback { fallback: None }, stack.options);
    let stand_ins = stand_ins(&parsed);
    for file in parsed {
        match file {
            Parsed::File(key, copy) => builder.push(key, copy),
            Parsed::Ignored(finding) => builder.push_diagnostic(finding),
        }
    }
    for (key, document) in stand_ins {
        builder.push(key, Ok(document));
    }
    builder.finish()
}

/// The root copies that stand in at a `mystructure` primary path, so that the path's top copy
/// that loads is what the game's structure lookup reads.
///
/// Only where a pack's primary copy failed while the pack has a root copy that loaded: that
/// pack has passed its turn, but its root copy is the effective file of the root path, hiding
/// the lower packs' root copies from [`Structure::find`]. For such a name every pack without a
/// primary copy offers its root copy (when it loaded) under the primary path as well: the copy
/// vanilla tries for that pack. A stand-in carries no findings (its file reports them once).
fn stand_ins(parsed: &[Parsed]) -> Vec<(PackPath, Document<Structure>)> {
    type Copy = Result<Document<Structure>, Rejected>;
    let pack_of = |copy: &Copy| match copy {
        Ok(document) => document.origin().pack.clone(),
        Err(rejected) => rejected.origin.pack.clone(),
    };
    // Every copy of each path, indexed once.
    let mut by_key: IndexMap<&PackPath, Vec<&Copy>> = IndexMap::new();
    for file in parsed {
        if let Parsed::File(key, copy) = file {
            by_key.entry(key).or_default().push(copy);
        }
    }
    let mut out = Vec::new();
    for (primary, copies) in &by_key {
        let Some(roots) = root_of(primary).and_then(|root| by_key.get(&root)) else {
            continue;
        };
        let loaded_root = |pack: &PackRef| {
            roots
                .iter()
                .any(|root| matches!(root, Ok(document) if document.origin().pack == *pack))
        };
        // A pack whose primary copy failed and whose root copy loaded hides the roots below.
        let hidden = copies
            .iter()
            .any(|copy| matches!(copy, Err(rejected) if loaded_root(&rejected.origin.pack)));
        if !hidden {
            continue;
        }
        for document in roots.iter().filter_map(|root| root.as_ref().ok()) {
            let pack = &document.origin().pack;
            let has_primary = copies.iter().any(|copy| pack_of(copy) == *pack);
            if !has_primary {
                out.push((
                    (*primary).clone(),
                    Document::new(
                        document.value().clone(),
                        document.origin().clone(),
                        document.version().clone(),
                        Diagnostics::default(),
                    ),
                ));
            }
        }
    }
    out
}

/// The root path `structures/<name>.mcstructure` of a `mystructure` primary path
/// `structures/mystructure/<name>.mcstructure`.
fn root_of(primary: &PackPath) -> Option<PackPath> {
    let name = primary
        .as_str()
        .strip_prefix(FOLDER)?
        .strip_prefix('/')?
        .strip_prefix(Structure::DEFAULT_NAMESPACE)?
        .strip_prefix('/')?;
    PackPath::new(&format!("{FOLDER}/{name}")).ok()
}

/// The loader of `bp.structures()`, the family's own: the walk reports the `.nbt` templates it
/// does not read (`Parsed::Ignored`) and the combine adds the `mystructure` stand-ins, which
/// `load::files` has no shape for.
#[doc(hidden)]
pub const LOADER: CollectionLoader<Files<Structure>, Parsed> = CollectionLoader {
    walk: WALK,
    parse: parse_walk_file,
    combine,
    needs: &[],
    then: None,
};

impl Structure {
    /// The most positions [`Structure::new`] fills: 64 × 384 × 64, the largest box
    /// `/structure save` takes in the overworld.
    pub const NEW_FILL_LIMIT: usize = 64 * 384 * 64;

    /// A structure of `size` blocks that keeps every world block where it is placed: both
    /// index layers all -1, one per position; no palettes, no entities, origin `[0, 0, 0]`.
    ///
    /// The layers are left empty, which vanilla accepts for any size, when the box has no
    /// position (a component below 1) or more than [`Structure::NEW_FILL_LIMIT`] of them
    /// (a file itself has no size limit: fill `block_indices` yourself for a larger box).
    /// Never panics, whatever the size.
    pub fn new(size: [i32; 3]) -> Structure {
        let volume = size
            .iter()
            .try_fold(1_usize, |volume, side| {
                volume.checked_mul(usize::try_from(*side).unwrap_or(0))
            })
            .filter(|volume| *volume <= Structure::NEW_FILL_LIMIT)
            .unwrap_or(0);
        Structure {
            format_version: Structure::FORMAT_VERSION,
            neighbor_aware_block_upgrade_version: None,
            size,
            structure_world_origin: [0, 0, 0],
            structure: Contents {
                block_indices: [vec![-1; volume], vec![-1; volume]],
                ..Contents::default()
            },
            extra: nbtx::Compound::new(),
            layout: Layout::default(),
        }
    }

    /// Runs the format's load rules as if the structure were read from a pack with `cx`: the
    /// findings loading its file would give (none for a valid structure, apart from the crate's own
    /// infos).
    pub fn validate(&self, cx: &LoadContext) -> Diagnostics {
        match self.to_bytes() {
            Ok(bytes) => read::read(&bytes, cx).1,
            Err(error) => {
                let mut findings = Diagnostics::default();
                findings.push(
                    Diagnostic::of(builders::syntax(error.to_string())).phase(Phase::Validate),
                );
                findings
            }
        }
    }

    /// The index in each [`block_indices`](crate::Contents::block_indices) layer of the
    /// position `[x, y, z]` relative to the box's minimum corner: z runs fastest, then y, then
    /// x. `None` outside the box, and for a position
    /// whose index does not fit a `usize`.
    pub fn index(&self, position: [i32; 3]) -> Option<usize> {
        let [x, y, z] = position.map(usize::try_from);
        let [sx, sy, sz] = self.size.map(usize::try_from);
        let ([x, y, z], [sx, sy, sz]) =
            ([x.ok()?, y.ok()?, z.ok()?], [sx.ok()?, sy.ok()?, sz.ok()?]);
        if x >= sx || y >= sy || z >= sz {
            return None;
        }
        x.checked_mul(sy)?
            .checked_add(y)?
            .checked_mul(sz)?
            .checked_add(z)
    }
}
