//! Jigsaw structures (`Jigsaw`, `JigsawStructure`, `TemplatePool`, `StructureSet`,
//! `ProcessorList`, `JigsawMetadata`): the four JSON documents below `worldgen/` of a behaviour
//! pack (`minecraft:jigsaw`, `minecraft:template_pool`, `minecraft:structure_set`,
//! `minecraft:processor_list`, each one struct at format 1.21.20) and the binary metadata cache
//! `worldgen/metadata/*.dat`, loaded together as the `Jigsaw` group.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod history;
#[doc(hidden)]
pub use link::{Targets, link};
pub use metadata::{
    JigsawBlockMetadata, JigsawMetadata, JointType, MetadataRegistry, StructureMetadata,
};
pub use pool::{PoolDescription, PoolElement, Projection, TemplatePool, TemplatePoolElement};
pub use processor::{
    Axis, BlockEntityModifier, BlockPredicate, CappedDelegate, IntProvider, IntProviderType,
    PositionPredicate, Processor, ProcessorList, ProcessorListDescription, ProcessorRule,
};
pub use set::{
    PlacementType, RandomSpreadPlacement, SpreadType, StructureSet, StructureSetDescription,
    StructureSetEntry,
};
pub use structure::{
    DimensionPadding, GenerationStep, GroupedPoolAlias, HeightProvider, HeightmapProjection,
    JigsawStructure, LiquidSettings, MaxDistanceFromCenter, PoolAlias, StructureDescription,
    TerrainAdaptation, VerticalAnchor, Weighted,
};

mod link;
mod metadata;
mod pool;
mod processor;
mod read;
mod set;
mod structure;
mod validate;

#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};
use std::convert::Infallible;

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Phase, Severity, VanillaText,
};
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, FilesBuilder, FilesRule,
    JsonDefinition, JsonDocument, JsonPlaced, NoBeta, Notice, RegistryBuilder, Rejected,
    default_path,
};
use bedrock_addon_core::document::{Files, Registry};
use bedrock_addon_core::id::{
    Id, JigsawStructureId, Kind, ProcessorListId, StructureSetId, TemplatePoolId, kind,
};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Unrepresentable, WriteStyle};
use bedrock_addon_core::load::Reported;
use bedrock_addon_core::load::{self, DocumentSource};
use bedrock_addon_core::load::{Accept, CollectionLoader, LoadContext, StackCx, Walk, WalkFile};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    BelowFloor, Envelope, Family, InputKind, Loaded, NoComponents, Reading, Single, StructIndex,
    UnknownMask, ValidateCx,
};
use history::{
    MetadataAny, MetadataFamily, PoolFamily, ProcessorListFamily, StructureFamily,
    StructureSetFamily,
};
use read::{V1_21_20, parse_metadata};

/// The jigsaw documents of the behaviour packs (`bp.jigsaw()`): five collections loaded and
/// used together.
///
/// Each JSON family is keyed by its `description.identifier`; the highest-priority pack's
/// definition wins, and inside one pack the first file (vanilla drops a later duplicate
/// silently; the crate reports it as `ShadowedDefinition`). The families are independent: one
/// identifier may name a structure, a pool, a set and a processor list. The metadata files are
/// keyed by path, the top pack's copy winning.
#[derive(Debug)]
#[non_exhaustive]
pub struct Jigsaw {
    /// The jigsaw structures (`worldgen/structures/**/*.json`).
    pub structures: Registry<JigsawStructure>,
    /// The template pools (`worldgen/template_pools/**/*.json`).
    pub pools: Registry<TemplatePool>,
    /// The structure sets (`worldgen/structure_sets/**/*.json`).
    pub structure_sets: Registry<StructureSet>,
    /// The processor lists (`worldgen/processors/**/*.json`).
    pub processors: Registry<ProcessorList>,
    /// The metadata caches (`worldgen/metadata/*.dat`).
    pub metadata: Files<JigsawMetadata>,
}

impl Jigsaw {
    /// Every finding of the five collections, structures first.
    pub fn diagnostics(
        &self,
    ) -> impl Iterator<Item = &bedrock_addon_core::diagnostic::Diagnostic> + '_ {
        self.structures
            .diagnostics()
            .chain(self.pools.diagnostics())
            .chain(self.structure_sets.diagnostics())
            .chain(self.processors.diagnostics())
            .chain(self.metadata.diagnostics())
    }
}

/// `worldgen/structures`: `minecraft:jigsaw` documents.
const STRUCTURES: &str = "worldgen/structures";
/// `worldgen/template_pools`: `minecraft:template_pool` documents.
const POOLS: &str = "worldgen/template_pools";
/// `worldgen/structure_sets`: `minecraft:structure_set` documents.
const SETS: &str = "worldgen/structure_sets";
/// `worldgen/processors`: `minecraft:processor_list` documents.
const PROCESSORS: &str = "worldgen/processors";
/// `worldgen/metadata`: the binary metadata files.
const METADATA: &str = "worldgen/metadata";

/// The oldest metadata registry.
pub(crate) const V1_21_50: FormatVersion = FormatVersion::new(1, 21, 50);

/// The line vanilla logs after a failed parse, with the file's path.
const STRUCTURE_FAILED: &str = "Failed to parse jigsaw structure definition '{}'.";
/// The line vanilla logs after a failed parse, with the file's path.
const POOL_FAILED: &str = "Failed to parse jigsaw structure template pool '{}'.";
/// The line vanilla logs after a failed parse, with the file's path.
const SET_FAILED: &str = "Failed to parse structure set definition '{}'.";
/// The line vanilla logs after a failed parse, with the file's path.
const PROCESSORS_FAILED: &str = "Failed to parse jigsaw structure processor list '{}'.";

/// The envelope of the four JSON documents.
const JSON_ENVELOPE: Envelope = Envelope {
    // The game reads all four documents as Minecraft documents: the version as the JSON reader
    // takes it, the size limits, and `null` as a member of the wrong type.
    input: InputKind::Document,
    // What the document does not read is kept for write-back: a number no double holds is the
    // largest finite one, with a finding at it.
    clamps_beyond_double: true,
    // Below the first slice the versioned loader has no parser at all.
    below_floor: BelowFloor::NoParser,
    // The default checks the envelope always. The game checks the member count of a document
    // from 1.21.20 only; below it no struct exists, so checking always changes only which text
    // rejects a three-member file below 1.21.20.
    ..Envelope::DEFAULT
};

/// The game's four slices (processor list, template pool, structure, structure set) are not
/// strict: unknown members are warnings.
const JSON_READING: Reading = Reading::DOCUMENT.with_unknown(UnknownMask::Warn);

impl Family for StructureFamily {
    const NAME: &'static str = "jigsaw structure";
    const PAYLOAD_KEY: &'static str = "minecraft:jigsaw";
    const FLOOR: FormatVersion = V1_21_20;
    const ENVELOPE: Envelope = JSON_ENVELOPE;
    type Latest = JigsawStructure;
    type Beta = NoBeta<kind::JigsawStructure>;
    type Component = NoComponents;
    type Any = Single<Self>;

    fn reading(_: StructIndex, _: DocumentSource) -> Reading {
        JSON_READING
    }

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(value) = v {
            validate::structure(value, &cx.at, cx.diagnostics);
        }
    }

    fn close(
        diagnostics: &mut Diagnostics,
        _: Option<JsonSlice<'_>>,
        path: &PackPath,
        failed: bool,
    ) {
        if failed {
            failure_line(diagnostics, path, STRUCTURE_FAILED);
        }
    }
}

impl JsonDocument for JigsawStructure {
    type Family = StructureFamily;
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];
}

impl JsonPlaced for JigsawStructure {
    /// `worldgen/structures/<namespace>/<name>.json`.
    fn place(&self) -> PackPath {
        default_path(STRUCTURES, &self.description.identifier)
    }
}

impl JsonDefinition for JigsawStructure {
    type Kind = kind::JigsawStructure;
    fn identifier(&self) -> &JigsawStructureId {
        &self.description.identifier
    }
}

impl Family for PoolFamily {
    const NAME: &'static str = "template pool";
    const PAYLOAD_KEY: &'static str = "minecraft:template_pool";
    const FLOOR: FormatVersion = V1_21_20;
    const ENVELOPE: Envelope = JSON_ENVELOPE;
    type Latest = TemplatePool;
    type Beta = NoBeta<kind::TemplatePool>;
    type Component = NoComponents;
    type Any = Single<Self>;

    fn reading(_: StructIndex, _: DocumentSource) -> Reading {
        JSON_READING
    }

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(value) = v {
            validate::pool(value, &cx.at, cx.diagnostics);
        }
    }

    fn close(
        diagnostics: &mut Diagnostics,
        _: Option<JsonSlice<'_>>,
        path: &PackPath,
        failed: bool,
    ) {
        if failed {
            failure_line(diagnostics, path, POOL_FAILED);
        }
    }
}

impl JsonDocument for TemplatePool {
    type Family = PoolFamily;
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];
}

impl JsonPlaced for TemplatePool {
    /// `worldgen/template_pools/<namespace>/<name>.json`.
    fn place(&self) -> PackPath {
        default_path(POOLS, &self.description.identifier)
    }
}

impl JsonDefinition for TemplatePool {
    type Kind = kind::TemplatePool;
    fn identifier(&self) -> &TemplatePoolId {
        &self.description.identifier
    }
}

impl Family for StructureSetFamily {
    const NAME: &'static str = "structure set";
    const PAYLOAD_KEY: &'static str = "minecraft:structure_set";
    const FLOOR: FormatVersion = V1_21_20;
    const ENVELOPE: Envelope = JSON_ENVELOPE;
    type Latest = StructureSet;
    type Beta = NoBeta<kind::StructureSet>;
    type Component = NoComponents;
    type Any = Single<Self>;

    fn reading(_: StructIndex, _: DocumentSource) -> Reading {
        JSON_READING
    }

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(value) = v {
            validate::set(value, &cx.at, cx.diagnostics);
        }
    }

    fn close(
        diagnostics: &mut Diagnostics,
        _: Option<JsonSlice<'_>>,
        path: &PackPath,
        failed: bool,
    ) {
        if failed {
            failure_line(diagnostics, path, SET_FAILED);
        }
    }
}

impl JsonDocument for StructureSet {
    type Family = StructureSetFamily;
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];
}

impl JsonPlaced for StructureSet {
    /// `worldgen/structure_sets/<namespace>/<name>.json`.
    fn place(&self) -> PackPath {
        default_path(SETS, &self.description.identifier)
    }
}

impl JsonDefinition for StructureSet {
    type Kind = kind::StructureSet;
    fn identifier(&self) -> &StructureSetId {
        &self.description.identifier
    }
}

impl Family for ProcessorListFamily {
    const NAME: &'static str = "processor list";
    const PAYLOAD_KEY: &'static str = "minecraft:processor_list";
    const FLOOR: FormatVersion = V1_21_20;
    const ENVELOPE: Envelope = JSON_ENVELOPE;
    type Latest = ProcessorList;
    type Beta = NoBeta<kind::ProcessorList>;
    type Component = NoComponents;
    type Any = Single<Self>;

    fn reading(_: StructIndex, _: DocumentSource) -> Reading {
        JSON_READING
    }

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(value) = v {
            validate::processors(value, &cx.at, cx.diagnostics);
        }
    }

    fn close(
        diagnostics: &mut Diagnostics,
        _: Option<JsonSlice<'_>>,
        path: &PackPath,
        failed: bool,
    ) {
        if failed {
            failure_line(diagnostics, path, PROCESSORS_FAILED);
        }
    }
}

impl JsonDocument for ProcessorList {
    type Family = ProcessorListFamily;
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];
}

impl JsonPlaced for ProcessorList {
    /// `worldgen/processors/<namespace>/<name>.json`.
    fn place(&self) -> PackPath {
        default_path(PROCESSORS, &self.description.identifier)
    }
}

impl JsonDefinition for ProcessorList {
    type Kind = kind::ProcessorList;
    fn identifier(&self) -> &ProcessorListId {
        &self.description.identifier
    }
}

impl Family for MetadataFamily {
    const NAME: &'static str = "jigsaw structure metadata";
    const PAYLOAD_KEY: &'static str = "minecraft:jigsaw_structure_metadata";
    const FLOOR: FormatVersion = V1_21_50;
    type Latest = MetadataRegistry;
    type Beta = Infallible;
    type Component = NoComponents;
    type Any = MetadataAny;
}

// ---- one JSON file -------------------------------------------------------------------------

/// After a failed parse (a parse error, not a registration check), the family's line `failed`
/// with the file's path.
fn failure_line(diagnostics: &mut Diagnostics, path: &PackPath, failed: &'static str) {
    let parse_error = diagnostics
        .iter()
        .any(|d| d.severity >= Severity::Error && d.phase == Phase::Parse);
    if parse_error {
        diagnostics.push(Diagnostic::error(DiagnosticCode::DocumentRejected {
            text: Some(VanillaText::Formatted {
                template: failed,
                args: vec![path.as_str().to_owned()],
            }),
        }));
    }
}

// ---- one metadata file ---------------------------------------------------------------------

impl bedrock_addon_core::__private::Sealed for JigsawMetadata {}

impl Content for JigsawMetadata {
    const NAME: &'static str = "jigsaw structure metadata";
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    fn parse(path: &PackPath, bytes: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
        parse_metadata(PackRef::app_data(), PackSide::Behavior, path, bytes)
    }

    /// The file is binary: it has no JSON form ([`JigsawMetadata::to_bytes`] writes it).
    fn write_json(&self, _: &mut Vec<u8>, _: WriteStyle) -> Result<(), EncodeError> {
        Err(no_json())
    }

    /// The file is binary: it has no JSON form ([`JigsawMetadata::to_bytes`] writes it).
    fn to_json(&self) -> Result<String, EncodeError> {
        Err(no_json())
    }
}

/// Why a metadata file has no JSON form.
fn no_json() -> EncodeError {
    EncodeError::NotRepresentable {
        at: JsonPath::new(),
        what: Unrepresentable::Unsupported,
    }
}

// ---- the group -----------------------------------------------------------------------------

/// What one walked file parses to.
#[doc(hidden)]
#[derive(Debug)]
pub enum JigsawFile {
    /// A `minecraft:jigsaw` file.
    Structure(Box<Contribution<JigsawStructure>>),
    /// A `minecraft:template_pool` file.
    Pool(Box<Contribution<TemplatePool>>),
    /// A `minecraft:structure_set` file.
    Set(Box<Contribution<StructureSet>>),
    /// A `minecraft:processor_list` file.
    Processors(Box<Contribution<ProcessorList>>),
    /// A metadata file, by its key.
    Metadata(PackPath, Box<Result<Document<JigsawMetadata>, Rejected>>),
}

/// The four JSON directories take files whose extension is exactly `json` (recursive); the
/// metadata directory takes files whose extension is exactly `dat`, directly in it.
fn accept(path: &PackPath) -> bool {
    let path = path.as_str();
    let extension = |wanted: &str| {
        path.rsplit_once('.')
            .is_some_and(|(_, extension)| extension == wanted)
    };
    let metadata = format!("{METADATA}/");
    match path.find(&metadata) {
        Some(at) => {
            let rest = &path[at + metadata.len()..];
            !rest.contains('/') && extension("dat")
        }
        None => extension("json"),
    }
}

/// `worldgen/{structures, template_pools, structure_sets, processors}/**/*.json` and
/// `worldgen/metadata/*.dat`, packs top first (vanilla's walk; the first definition of an
/// identifier wins).
const WALK: Walk = Walk::json_tree_in(&[STRUCTURES, POOLS, SETS, PROCESSORS, METADATA])
    .accept(Accept::Fn(accept))
    .top_down();

/// One JSON family's contribution from its file: the parse of the registry constructor.
fn contribution<D: JsonDefinition>(file: &WalkFile, cx: &LoadContext) -> Contribution<D> {
    (load::registry::<D>(WALK).parse)(file, cx)
}

/// Parses one walked file by its directory.
fn parse_walk_file(file: &WalkFile, cx: &LoadContext) -> JigsawFile {
    match WALK.matches(&file.key) {
        Some(0) => JigsawFile::Structure(Box::new(contribution(file, cx))),
        Some(1) => JigsawFile::Pool(Box::new(contribution(file, cx))),
        Some(2) => JigsawFile::Set(Box::new(contribution(file, cx))),
        Some(3) => JigsawFile::Processors(Box::new(contribution(file, cx))),
        _ => JigsawFile::Metadata(
            file.key.clone(),
            Box::new(parse_metadata(
                file.pack.clone(),
                file.side,
                &file.path,
                &file.bytes,
            )),
        ),
    }
}

/// One family's registry while the group combines: the highest-priority pack wins, and inside
/// one pack the first file in walk order; a dropped definition is a `ShadowedDefinition`
/// information.
///
/// A document that parsed holds its identifier whether or not it is registered afterwards:
/// vanilla drops every later document of that identifier while it loads the packs, and runs
/// the registration checks only then. A file that failed to parse holds nothing.
struct FamilyRegistry<T: Definition> {
    builder: RegistryBuilder<T>,
    /// The identifiers defined so far, with the pack that defined each.
    seen: HashSet<(Id<T::Kind>, Option<usize>)>,
    /// The identifiers held so far: by a registered document, or by one that parsed and
    /// failed a registration check.
    held: HashSet<Id<T::Kind>>,
    /// The identifiers held by a document that failed a registration check.
    refused: HashSet<Id<T::Kind>>,
}

impl<T: Definition> FamilyRegistry<T> {
    fn new(stack: &StackCx<'_>) -> Self {
        Self {
            builder: RegistryBuilder::new(
                CombineRule::TopWins {
                    shadowed: Some(Notice::shadowed()),
                },
                CombineRule::top_wins(),
                stack.options,
            ),
            seen: HashSet::new(),
            held: HashSet::new(),
            refused: HashSet::new(),
        }
    }

    /// Adds a contribution; `true` when it is the document that is registered under its
    /// identifier, `false` when a document read before it holds the identifier or the file
    /// gave no document.
    fn push(&mut self, contribution: Contribution<T>) -> bool {
        match &contribution {
            Contribution::Stable(id, document) => {
                let key = (id.clone(), document.origin().pack.index());
                // A second definition in the same pack, or one below a document that was
                // refused at registration: the registry's own rule reports neither.
                if !self.seen.insert(key) || self.refused.contains(id) {
                    self.builder.push_diagnostic(
                        Diagnostic::info(DiagnosticCode::ShadowedDefinition { text: None })
                            .in_file(document.origin())
                            .phase(Phase::Validate),
                    );
                    return false;
                }
                let first = self.held.insert(id.clone());
                self.builder.push(contribution);
                first
            }
            Contribution::Rejected(Some(id), rejected)
                if validate::failed_registration(&rejected.diagnostics) =>
            {
                if self.held.insert(id.clone()) {
                    self.refused.insert(id.clone());
                }
                self.builder.push(contribution);
                false
            }
            _ => {
                self.builder.push(contribution);
                false
            }
        }
    }
}

/// The checks over every effective template pool.
#[derive(Default)]
struct PoolChecks {
    /// Each single element's location, with its processors and projection.
    elements: HashMap<String, (String, Projection)>,
    /// The pool names, lowercased.
    names: HashSet<String>,
}

impl PoolChecks {
    /// The findings of one pool, registered after the ones seen so far.
    fn check(&mut self, pool: &Document<TemplatePool>, stack: &StackCx<'_>) -> Vec<Diagnostic> {
        let mut found = Vec::new();
        let at = JsonPath::new().child("minecraft:template_pool");
        // The game lowercases a pool name when it registers it; release builds keep the first.
        let name = pool.description.identifier.as_str().to_lowercase();
        let built_in =
            stack.options.built_ins.as_ref().is_some_and(|built_ins| {
                built_ins.contains(<kind::TemplatePool as Kind>::NAME, &name)
            });
        if !self.names.insert(name.clone()) || built_in {
            let path = at.child("description").child("identifier");
            // The game only asserts here, with no content-log line, so the crate reports
            // an information of its own.
            found.push(
                Diagnostic::info(DiagnosticCode::DuplicateIdentifier {
                    text: Some(VanillaText::Formatted {
                        template: "Jigsaw pool name `{}` already registered",
                        args: vec![name],
                    }),
                })
                .at(path)
                .phase(Phase::Validate)
                .in_file(pool.origin()),
            );
        }
        // One element per location; a different one is a redefinition. The game registers
        // only one element, silently, so the text "Redefinition of existing element with
        // location '{}'." is the crate's own.

        for (i, entry) in pool.elements.iter().enumerate() {
            let PoolElement::Single {
                location,
                processors,
                projection,
                ..
            } = &entry.element
            else {
                continue;
            };
            let contents = (
                processors
                    .as_ref()
                    .map(|p| p.as_str().to_owned())
                    .unwrap_or_default(),
                projection.unwrap_or_default(),
            );
            match self.elements.get(location.as_str()) {
                Some(existing) if *existing != contents => {
                    let path = at
                        .child("elements")
                        .index(i)
                        .child("element")
                        .child("location");
                    found.push(
                        // A crate-own lint: Info.
                        Diagnostic::info(DiagnosticCode::InvalidValue {
                            text: Some(VanillaText::Formatted {
                                template: "Redefinition of existing element with location '{}'.",
                                args: vec![location.as_str().to_owned()],
                            }),
                        })
                        .at(path)
                        .in_file(pool.origin())
                        .phase(Phase::Validate),
                    );
                }
                Some(_) => {}
                None => {
                    self.elements.insert(location.as_str().to_owned(), contents);
                }
            }
        }
        found
    }
}

/// The one structure set the game records itself, after the packs' sets
/// (`minecraft:trial_chambers`, when the base game version allows it).
const GAME_RECORDED_SET: &str = "minecraft:trial_chambers";

/// The game's registry keeps the set already recorded under a name and logs "Structure set
/// '%s' has already been registered" at the content log's error level for the later one. The
/// packs' sets reach it first, already one per identifier (a later file of an identifier is
/// dropped silently, which is the first-wins rule of the registry above), so the one collision is a
/// pack's `minecraft:trial_chambers` against the game's own: the pack's set stays effective and the
/// game's record is refused with the text. The finding is placed at the pack's file.
fn game_record_refused(document: &Document<StructureSet>) -> Option<Diagnostic> {
    (document.description.identifier.as_str() == GAME_RECORDED_SET).then(|| {
        Diagnostic::error(DiagnosticCode::DuplicateIdentifier {
            text: Some(VanillaText::Formatted {
                template: "Structure set '{}' has already been registered",
                args: vec![GAME_RECORDED_SET.to_owned()],
            }),
        })
        .in_file(document.origin())
        .phase(Phase::Validate)
    })
}

/// The group, from every parsed file in walk order.
fn combine(parsed: Vec<JigsawFile>, stack: &StackCx<'_>) -> Jigsaw {
    let mut structures = FamilyRegistry::<JigsawStructure>::new(stack);
    let mut pools = FamilyRegistry::<TemplatePool>::new(stack);
    let mut sets = FamilyRegistry::<StructureSet>::new(stack);
    let mut processors = FamilyRegistry::<ProcessorList>::new(stack);
    let mut metadata = FilesBuilder::<JigsawMetadata>::new(FilesRule::TopWins, stack.options);
    let mut checks = PoolChecks::default();
    for file in parsed {
        match file {
            JigsawFile::Structure(c) => {
                structures.push(*c);
            }
            JigsawFile::Pool(c) => {
                // The walk is top first, so the first definition of an identifier is the
                // effective one: only it is registered.
                let c = *c;
                let pool = match &c {
                    Contribution::Stable(_, document) => Some(document.clone()),
                    _ => None,
                };
                if pools.push(c)
                    && let Some(document) = pool
                {
                    for finding in checks.check(&document, stack) {
                        pools.builder.push_diagnostic(finding);
                    }
                }
            }
            JigsawFile::Set(c) => {
                let c = *c;
                let refused = match &c {
                    Contribution::Stable(_, document) => game_record_refused(document),
                    _ => None,
                };
                if sets.push(c)
                    && let Some(finding) = refused
                {
                    sets.builder.push_diagnostic(finding);
                }
            }
            JigsawFile::Processors(c) => {
                processors.push(*c);
            }
            JigsawFile::Metadata(key, copy) => metadata.push(key, *copy),
        }
    }
    Jigsaw {
        structures: structures.builder.finish(),
        pools: pools.builder.finish(),
        structure_sets: sets.builder.finish(),
        processors: processors.builder.finish(),
        metadata: metadata.finish(),
    }
}

#[doc(hidden)]
/// The loader of `bp.jigsaw()`: the five directories of every behaviour pack, combined into
/// the group.
pub const LOADER: CollectionLoader<Jigsaw, JigsawFile> = CollectionLoader {
    walk: WALK,
    parse: parse_walk_file,
    combine,
    needs: &[],
    then: None,
};

impl Reported for Jigsaw {
    fn reported(&self) -> Box<dyn Iterator<Item = &Diagnostic> + '_> {
        Box::new(self.diagnostics())
    }
}

#[cfg(test)]
mod reading_table {
    //! The family's reading table: what the pipeline reads each of its files and structs by.

    use bedrock_addon_core::versioning::testing::{assert_reading_table, reading_table};

    #[test]
    fn structure_reads_as_pinned() {
        assert_reading_table(
            "StructureFamily",
            &reading_table::<super::StructureFamily>(),
            r"
                text: Document, clamps beyond a double: true, duplicates kept in: []
                envelope: Always
                missing version: at the floor, told at Info: None
                below floor: NoParser
                empty document: Error
                1.21.20..=1.21.20 (0..=0): json Rejected/Strict/Warn/fatal/Refuse, mcb Own/Strict/Warn/fatal/Refuse
            ",
        );
    }

    #[test]
    fn pool_reads_as_pinned() {
        assert_reading_table(
            "PoolFamily",
            &reading_table::<super::PoolFamily>(),
            r"
                text: Document, clamps beyond a double: true, duplicates kept in: []
                envelope: Always
                missing version: at the floor, told at Info: None
                below floor: NoParser
                empty document: Error
                1.21.20..=1.21.20 (0..=0): json Rejected/Strict/Warn/fatal/Refuse, mcb Own/Strict/Warn/fatal/Refuse
            ",
        );
    }

    #[test]
    fn structure_set_reads_as_pinned() {
        assert_reading_table(
            "StructureSetFamily",
            &reading_table::<super::StructureSetFamily>(),
            r"
                text: Document, clamps beyond a double: true, duplicates kept in: []
                envelope: Always
                missing version: at the floor, told at Info: None
                below floor: NoParser
                empty document: Error
                1.21.20..=1.21.20 (0..=0): json Rejected/Strict/Warn/fatal/Refuse, mcb Own/Strict/Warn/fatal/Refuse
            ",
        );
    }

    #[test]
    fn processor_list_reads_as_pinned() {
        assert_reading_table(
            "ProcessorListFamily",
            &reading_table::<super::ProcessorListFamily>(),
            r"
                text: Document, clamps beyond a double: true, duplicates kept in: []
                envelope: Always
                missing version: at the floor, told at Info: None
                below floor: NoParser
                empty document: Error
                1.21.20..=1.21.20 (0..=0): json Rejected/Strict/Warn/fatal/Refuse, mcb Own/Strict/Warn/fatal/Refuse
            ",
        );
    }

    #[test]
    fn metadata_reads_as_pinned() {
        assert_reading_table(
            "jigsaw::MetadataFamily",
            &reading_table::<super::MetadataFamily>(),
            r"
                text: Root, clamps beyond a double: false, duplicates kept in: []
                envelope: Always
                missing version: at the floor, told at Info: None
                below floor: UseFormatVersion
                empty document: Error
                1.21.50..=1.21.80 (0..=1): json Own/Strict/Warn/lenient/Refuse, mcb Own/Strict/Warn/lenient/Refuse
            ",
        );
    }
}
