//! Loading structures: `Content`, the walk of `structures/`, the combine rule and the lookups
//! vanilla does by identifier.

use bedrock_addon_core::load::{
    Accept, AppDataPlace, CollectionLoader, ListDepth, LoadContext, StackCx, Walk, WalkFile,
    WalkOrder,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use indexmap::IndexMap;

use super::{Structure, parse, text};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity,
};
use bedrock_addon_core::document::{
    Content, Document, Files, FilesBuilder, FilesRule, Origin, Rejected, VersionInfo,
};
use bedrock_addon_core::id::StructureId;
use bedrock_addon_core::json::{EncodeError, Unrepresentable, WriteStyle};
use bedrock_addon_core::version::FormatVersion;

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
    let one = FormatVersion::new(1, 0, 0);
    VersionInfo::new(None, false, one, one, false)
}

/// Gives every finding the document's pack and file.
fn locate(diagnostics: &mut Diagnostics, origin: &Origin) {
    let found = std::mem::take(diagnostics);
    for mut finding in found {
        finding.location.pack = Some(origin.pack.clone());
        finding.location.path = Some(origin.path.clone());
        diagnostics.push(finding);
    }
}

/// Loads one structure file of `pack`.
pub(crate) fn parse_file(
    pack: PackRef,
    side: PackSide,
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> Result<Document<Structure>, Rejected> {
    let origin = Origin::new(pack, side, path.clone(), None, Vec::new());
    let (value, mut diagnostics) = parse::read(bytes, cx);
    locate(&mut diagnostics, &origin);
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
            return Err(Diagnostic::new(
                DiagnosticCode::NamespaceRule {
                    text: Some(text::not_namespaced(id.as_str())),
                },
                Severity::Error,
                Phase::Link,
                Location::default(),
            ));
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
pub(super) fn keyed(origin: &PackPath, key: &PackPath) -> bool {
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
const WALK: Walk = Walk {
    directories: &[FOLDER],
    depth: ListDepth::Recursive,
    accept: Accept::Fn(is_structure_or_template),
    order: WalkOrder::TopDown,
    directory_major: false,
    app_data: &[],
    app_data_place: AppDataPlace::After,
    in_pack: Some(read_in_pack),
    shadowed: None,
    read_error: None,
};

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
        return Parsed::Ignored(Diagnostic::new(
            DiagnosticCode::IgnoredFile { text: None },
            Severity::Info,
            Phase::Parse,
            Location::new(
                Some(file.pack.clone()),
                Some(file.path.clone()),
                JsonPath::new(),
                None,
            ),
        ));
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

/// The loader of `bp.structures()`.
pub const fn load_files() -> CollectionLoader<Files<Structure>, Parsed> {
    CollectionLoader {
        walk: WALK,
        parse: parse_walk_file,
        combine,
        needs: &[],
        then: None,
    }
}
