//! Points of interest: `Poi`, `PoiType`, `PoiTag`.
//!
//! Points of interest: the two documents of a behaviour pack's `poi/blocks` and `poi/tags`,
//! `minecraft:poi_block` (a POI type: the blocks that are one) and `minecraft:poi_tag` (a tag
//! and its place in the tag hierarchy), each one struct at format 1.26.40, loaded together as
//! the `Poi` group behind the `poi` experiment.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod history;
#[doc(hidden)]
pub use link::{Targets, link};

mod link;
mod read;
mod resolve;
mod types;
mod validate;

#[cfg(test)]
mod tests;

pub use types::{PoiDescription, PoiTag, PoiTagDescription, PoiType};

use std::collections::HashSet;

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Phase, VanillaText,
};
use bedrock_addon_core::document::unversioned::origin_of;
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Document, JsonDefinition, JsonDocument, JsonPlaced, NoBeta,
    Notice, Origin, Registry, RegistryBuilder, Rejected, default_path_flat,
};
use bedrock_addon_core::id::BlockId;
use bedrock_addon_core::id::{PoiTagId, PoiTypeId, kind};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Prepared};
use bedrock_addon_core::load::{self, CollectionLoader, LoadContext, StackCx, Walk, WalkFile};
use bedrock_addon_core::load::{Experiment, Reported};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput};
use bedrock_addon_core::versioning::{
    BelowFloor, DocumentRead, Envelope, Family, InputKind, Loaded, MissingVersion, NoComponents,
    Parsed, Single, ValidateCx, finish_file, parse_as_written, parse_document,
};
use history::{PoiTagFamily, PoiTypeFamily};
use indexmap::IndexMap;
use read::V1_26_40;
use resolve::{TagRecord, TypeRecord, resolve};

/// The points of interest of the behaviour packs (`bp.poi()`): the POI types and the tag
/// documents, loaded and resolved together.
///
/// A type is keyed by its `description.identifier`; the definition loaded last wins (the
/// stack is walked from the lowest pack up, so the highest pack's), and a block listed by two
/// types belongs to the one loaded last. The tag documents of one identifier are unioned.
/// Nothing is loaded without the `poi` experiment.
#[derive(Debug)]
#[non_exhaustive]
pub struct Poi {
    /// The POI types (`minecraft:poi_block`), by identifier.
    pub types: Registry<PoiType>,
    /// The tag documents (`minecraft:poi_tag`), by identifier.
    pub tags: Registry<PoiTag>,
    /// What the documents resolve to together: each block's type and the tag hierarchy.
    pub(crate) resolved: resolve::Resolved,
    /// The findings of the group itself: the experiment gate and the checks over every
    /// document.
    pub(crate) findings: Diagnostics,
}

impl Poi {
    /// The POI type `block` belongs to: the type loaded last among those listing it.
    pub fn type_of_block(&self, block: &BlockId) -> Option<&PoiTypeId> {
        self.resolved.blocks.get(block)
    }

    /// Every tag, in the order they came to exist: a tag has a document, or is named by a
    /// type's `tags`.
    pub fn tag_names(&self) -> impl Iterator<Item = &PoiTagId> + '_ {
        self.resolved.tags.keys()
    }

    /// Whether `tag` exists.
    pub fn contains_tag(&self, tag: &PoiTagId) -> bool {
        self.resolved.tags.contains_key(tag)
    }

    /// The types `tag` holds: the ones that name it, then the ones its narrower tags hold.
    /// Empty for a tag that does not exist.
    pub fn members(&self, tag: &PoiTagId) -> &[PoiTypeId] {
        self.resolved
            .tags
            .get(tag)
            .map_or(&[], |node| node.members.as_slice())
    }

    /// Whether `poi_type` is a member of `tag`, directly or through a narrower tag.
    pub fn has_tag(&self, poi_type: &PoiTypeId, tag: &PoiTagId) -> bool {
        self.members(tag).contains(poi_type)
    }

    /// The broader tags of `tag`: the ones it names in `parents` and the ones that name it in
    /// `children`, without the edges of a cycle and the ones to a tag that does not exist.
    pub fn parents(&self, tag: &PoiTagId) -> &[PoiTagId] {
        self.resolved
            .tags
            .get(tag)
            .map_or(&[], |node| node.parents.as_slice())
    }

    /// The narrower tags of `tag`: the mirror of [`parents`](Self::parents).
    pub fn children(&self, tag: &PoiTagId) -> &[PoiTagId] {
        self.resolved
            .tags
            .get(tag)
            .map_or(&[], |node| node.children.as_slice())
    }

    /// Every file that failed to load: the types' (`poi/blocks`), then the tags' (`poi/tags`).
    pub fn rejected(&self) -> impl Iterator<Item = &Rejected> + '_ {
        self.types.rejected().iter().chain(self.tags.rejected())
    }

    /// Every finding of the group: the types', the tags', then the group's own.
    pub fn diagnostics(&self) -> impl Iterator<Item = &Diagnostic> + '_ {
        self.types
            .diagnostics()
            .chain(self.tags.diagnostics())
            .chain(self.findings.iter())
    }
}

/// The directory of the type documents, which the game walks without descending into
/// sub-folders.
const TYPES: &str = "poi/blocks";

/// The directory of the tag documents, which the game also walks without descending into
/// sub-folders.
const TAGS: &str = "poi/tags";

/// The envelope of both documents.
const POI_ENVELOPE: Envelope = Envelope {
    // Both loaders read a document input: the version is read as that input reads it, size
    // limits apply, and a `null` where a value is read rejects the file.
    input: InputKind::Document,
    // The game's versioned loader does not check the version itself, so a missing version is
    // rejected with "no valid version passed by the caller or found in the input".
    missing: MissingVersion::REJECT,
    // Below the first slice the versioned loader has no parser at all.
    below_floor: BelowFloor::NoParser,
    ..Envelope::DEFAULT
};

impl Family for PoiTypeFamily {
    const NAME: &'static str = "poi type";
    const PAYLOAD_KEY: &'static str = "minecraft:poi_block";
    const FLOOR: FormatVersion = V1_26_40;
    const ENVELOPE: Envelope = POI_ENVELOPE;
    type Latest = PoiType;
    type Beta = NoBeta<kind::PoiType>;
    type Component = NoComponents;
    type Any = Single<Self>;

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(value) = v {
            validate::poi_type(value, &cx.at, cx.diagnostics);
        }
    }

    fn close(diagnostics: &mut Diagnostics, _: Option<JsonSlice<'_>>, _: &PackPath, failed: bool) {
        if failed {
            diagnostics.push(failed_to_load());
        }
    }
}

impl JsonDocument for PoiType {
    type Family = PoiTypeFamily;
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    /// Not read at all without the `poi` experiment.
    fn read(origin: Origin, bytes: &[u8], cx: &LoadContext) -> DocumentRead<PoiTypeFamily> {
        read::<PoiTypeFamily>(origin, bytes, cx)
    }

    /// Gated as the whole load is; the file is not closed (no "Failed to load POI Type").
    fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> Result<Document<Single<PoiTypeFamily>>, Rejected> {
        as_written::<PoiTypeFamily>(path, bytes, cx)
    }
}

impl JsonPlaced for PoiType {
    /// `poi/blocks/<name>.json` for a `minecraft:` identifier and `poi/blocks/<namespace>_<name>.json`
    /// for any other: the walk reads no sub-folder, so a deeper path would never be loaded.
    fn place(&self) -> PackPath {
        default_path_flat(TYPES, &self.description.identifier)
    }
}

impl JsonDefinition for PoiType {
    type Kind = kind::PoiType;
    fn identifier(&self) -> &PoiTypeId {
        &self.description.identifier
    }
}

impl PoiType {
    /// Runs the family's rules on the value as if it were loaded from a behaviour pack with
    /// `cx`: the document is written and read back, so the parse constraints and the
    /// document's own rules both run. The value is not changed.
    pub fn validate(&self, cx: &LoadContext) -> Diagnostics {
        validate_detached(self, cx)
    }
}

impl Family for PoiTagFamily {
    const NAME: &'static str = "poi tag";
    const PAYLOAD_KEY: &'static str = "minecraft:poi_tag";
    const FLOOR: FormatVersion = V1_26_40;
    const ENVELOPE: Envelope = POI_ENVELOPE;
    type Latest = PoiTag;
    type Beta = NoBeta<kind::PoiTag>;
    type Component = NoComponents;
    type Any = Single<Self>;

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(value) = v {
            validate::poi_tag(value, &cx.at, cx.diagnostics);
        }
    }

    fn close(diagnostics: &mut Diagnostics, _: Option<JsonSlice<'_>>, _: &PackPath, failed: bool) {
        if failed {
            diagnostics.push(failed_to_load());
        }
    }
}

impl JsonDocument for PoiTag {
    type Family = PoiTagFamily;
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    /// Not read at all without the `poi` experiment.
    fn read(origin: Origin, bytes: &[u8], cx: &LoadContext) -> DocumentRead<PoiTagFamily> {
        read::<PoiTagFamily>(origin, bytes, cx)
    }

    /// Gated as the whole load is; the file is not closed (no "Failed to load POI Type").
    fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> Result<Document<Single<PoiTagFamily>>, Rejected> {
        as_written::<PoiTagFamily>(path, bytes, cx)
    }
}

impl JsonPlaced for PoiTag {
    /// `poi/tags/<name>.json` for a `minecraft:` identifier and `poi/tags/<namespace>_<name>.json`
    /// for any other: the walk reads no sub-folder, so a deeper path would never be loaded.
    fn place(&self) -> PackPath {
        default_path_flat(TAGS, &self.description.identifier)
    }
}

impl JsonDefinition for PoiTag {
    type Kind = kind::PoiTag;
    fn identifier(&self) -> &PoiTagId {
        &self.description.identifier
    }
}

impl PoiTag {
    /// Runs the family's rules on the value as if it were loaded from a behaviour pack with
    /// `cx`: the document is written and read back, so the parse constraints and the
    /// document's own rules both run. The value is not changed.
    pub fn validate(&self, cx: &LoadContext) -> Diagnostics {
        validate_detached(self, cx)
    }
}

// ---- one file --------------------------------------------------------------------------------

/// The finding of an unreadable file.
fn malformed() -> Diagnostic {
    Diagnostic::error(DiagnosticCode::MalformedInput {
        message: "not UTF-8 JSON within the nesting limit".to_owned(),
        text: None,
    })
}

/// Whether the POI documents of a pack loaded with `cx` are read at all: vanilla builds the
/// whole POI system, its loaders included, only under the `poi` experiment: when the level is
/// set up, the experiment check skips the registry, both loaders and the serialisation.
fn enabled(cx: &LoadContext) -> bool {
    cx.experiments.contains(Experiment::Poi)
}

/// The finding of a file that is not read because the `poi` experiment is off
/// (the crate's own note: vanilla says nothing).
fn gate_closed() -> Diagnostic {
    Diagnostic::info(DiagnosticCode::MissingToggles { text: None })
}

/// A file that did not load, with its findings placed.
fn reject(origin: Origin, diagnostics: Diagnostics) -> Rejected {
    Rejected::located(origin, diagnostics)
}

/// `JsonDocument::read` of family `F`: core's read, behind the experiment gate.
fn read<F: Family>(origin: Origin, bytes: &[u8], cx: &LoadContext) -> DocumentRead<F> {
    if !enabled(cx) {
        return DocumentRead::Rejected(None, unread(origin, gate_closed()));
    }
    parse_document::<F>(origin, bytes, cx)
}

/// The rest of the load of a parsed file, closed once by core's `finish_file`.
fn finished<F: Family>(
    parsed: Parsed<F>,
    origin: Origin,
    cx: &LoadContext,
) -> Result<Document<F::Latest>, Rejected> {
    let outcome = finish_file(parsed, None, &origin.path, cx);
    let mut diagnostics = outcome.diagnostics;
    if let Some(Loaded::Stable(value)) = outcome.value {
        diagnostics.locate(&origin);
        Ok(Document::new(value, origin, outcome.version, diagnostics))
    } else {
        Err(reject(origin, diagnostics))
    }
}

/// The error each loader logs after a file that failed to load: both the type loader and the
/// tag loader log the one text "Failed to load POI Type", after the versioned loader's own log
/// lines.
fn failed_to_load() -> Diagnostic {
    Diagnostic::error(DiagnosticCode::DocumentRejected {
        text: Some(VanillaText::Fixed("Failed to load POI Type")),
    })
}

/// The finding of a value that has no JSON form: an `InvalidValue` error at the member the
/// writer refused, below the payload `key`. (No POI member can be refused at present: names are
/// strings and the numbers are `u8`. A value that cannot be written must not pass for a
/// valid one.)
pub(crate) fn unwritable(error: &EncodeError, key: &str) -> Diagnostic {
    let mut at = JsonPath::new();
    at.push(key);
    // `EncodeError` is `#[non_exhaustive]`: an error without a place is at the payload.
    if let EncodeError::NotRepresentable { at: member, .. } = error {
        at.append(member);
    }
    Diagnostic::error(DiagnosticCode::InvalidValue { text: None })
        .at(at)
        .phase(Phase::Validate)
}

/// `Root::validate`: the findings of loading what the value writes.
fn validate_detached<T: JsonPlaced>(value: &T, cx: &LoadContext) -> Diagnostics {
    let path = value.place();
    let text = match <T as Content>::to_json(value) {
        Ok(text) => text,
        Err(error) => {
            let mut diagnostics = Diagnostics::default();
            diagnostics.push(unwritable(&error, <T::Family as Family>::PAYLOAD_KEY));
            return diagnostics;
        }
    };
    match <T as Content>::parse(&path, text.as_bytes(), cx) {
        Ok(document) => document.diagnostics().clone(),
        Err(rejected) => rejected.diagnostics,
    }
}

/// A file that is not read: the gate's finding, or the unreadable text's.
fn unread(origin: Origin, finding: Diagnostic) -> Rejected {
    let mut diagnostics = Diagnostics::default();
    diagnostics.push(finding);
    reject(origin, diagnostics)
}

/// `Root::parse_as_written` of family `F`: gated, and not closed.
fn as_written<F: Family>(
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> Result<Document<F::Any>, Rejected> {
    let origin = Origin::file(PackRef::app_data(), PackSide::Behavior, path.clone());
    if !enabled(cx) {
        return Err(unread(origin, gate_closed()));
    }
    let Ok(prepared) = Prepared::new(bytes) else {
        return Err(unread(origin, malformed()));
    };
    let parsed = parse_as_written::<F>(prepared.slice(), cx);
    let mut diagnostics = parsed.diagnostics;
    match parsed.value {
        Some(value) => {
            diagnostics.locate(&origin);
            Ok(Document::new(value, origin, parsed.version, diagnostics))
        }
        None => Err(reject(origin, diagnostics)),
    }
}

/// `history::upgrade_type`/`upgrade_tag` of family `F`: rebuilds the parse stage from the
/// document and finishes it, gated as the whole load is.
pub(crate) fn upgrade<F: Family>(
    document: Document<F::Any>,
    cx: &LoadContext,
) -> Result<Document<F::Latest>, Rejected> {
    let origin = document.origin().clone();
    if !enabled(cx) {
        return Err(unread(origin, gate_closed()));
    }
    let version = document.version().clone();
    let diagnostics = document.diagnostics().clone();
    let value = document.into_value();
    let at = JsonPath::new().child(F::PAYLOAD_KEY);
    let written = FormatVersionInput {
        version: Some(version.effective),
        beta_literal: false,
        prerelease: false,
    };
    let parsed = Parsed::<F>::from_value(Some(value), version, written, at, diagnostics);
    finished::<F>(parsed, origin, cx)
}

// ---- the group -------------------------------------------------------------------------------

/// What one walked file parses to.
#[doc(hidden)]
#[derive(Debug)]
pub enum PoiFile {
    /// A file of `poi/blocks`, loaded as a `minecraft:poi_block` document.
    Type(Box<Contribution<PoiType>>),
    /// A file of `poi/tags`, loaded as a `minecraft:poi_tag` document.
    Tag(Box<Contribution<PoiTag>>),
    /// A file of a pack loaded without the `poi` experiment: not read, only where it is.
    Gated(Box<Origin>),
}

/// Every `*.json` directly in `poi/blocks` and `poi/tags`, packs from the lowest up: the
/// definition loaded last wins. Neither loader reads a sub-folder.
const WALK: Walk = Walk::json_children_in(&[TYPES, TAGS]);

/// One family's contribution from its file: the parse of the registry constructor (through
/// the gate of `JsonDocument::read`).
fn contribution<D: JsonDefinition>(file: &WalkFile, cx: &LoadContext) -> Contribution<D> {
    (load::registry::<D>(WALK).parse)(file, cx)
}

/// Parses one walked file as the document of its directory: `poi/blocks` holds types and
/// `poi/tags` tags, whatever the payload key says (a file of the other kind has no payload
/// under the key its loader asks for: `MissingPayload`). Without the `poi` experiment the
/// file is not read at all.
fn parse_walk_file(file: &WalkFile, cx: &LoadContext) -> PoiFile {
    if !enabled(cx) {
        return PoiFile::Gated(Box::new(origin_of(file)));
    }
    if file.key.as_str().starts_with(TAGS) {
        PoiFile::Tag(Box::new(contribution(file, cx)))
    } else {
        PoiFile::Type(Box::new(contribution(file, cx)))
    }
}

/// The group, from every parsed file in walk order.
fn combine(parsed: Vec<PoiFile>, stack: &StackCx<'_>) -> Poi {
    // Types: the definition loaded last wins per identifier, the replaced one a
    // `ShadowedDefinition` info (the game's registry overwrites a name in place).
    let mut types = RegistryBuilder::<PoiType>::new(
        CombineRule::LastLoadedWins {
            replaced: Some(Notice::shadowed()),
        },
        CombineRule::top_wins(),
        stack.options,
    );
    // Tags: the documents of one identifier are unioned (the game serves every call for a
    // tag name from one builder).
    let mut tags = TagDocuments {
        builder: RegistryBuilder::<PoiTag>::new(
            CombineRule::Accumulate { accumulate: union },
            CombineRule::top_wins(),
            stack.options,
        ),
        pack: None,
        batch: Vec::new(),
    };
    let mut findings = Diagnostics::default();
    // The packs already told that their POI files are not read.
    let mut gated: HashSet<Option<usize>> = HashSet::new();
    // The effective type documents, in the order they were loaded: a later definition of an
    // identifier replaces the earlier one and takes its place at the end.
    let mut effective: IndexMap<PoiTypeId, TypeRecord> = IndexMap::new();
    // Every loaded tag document, in walk order.
    let mut documents: Vec<TagRecord> = Vec::new();
    for file in parsed {
        match file {
            PoiFile::Type(contribution) => {
                if let Contribution::Stable(id, document) = &*contribution {
                    effective.shift_remove(id);
                    effective.insert(
                        id.clone(),
                        TypeRecord {
                            id: id.clone(),
                            origin: document.origin().clone(),
                            blocks: document.blocks.clone(),
                            tags: document.tags().to_vec(),
                        },
                    );
                }
                types.push(*contribution);
            }
            PoiFile::Tag(contribution) => {
                if let Contribution::Stable(id, document) = &*contribution {
                    documents.push(TagRecord {
                        id: id.clone(),
                        origin: document.origin().clone(),
                        children: document.children().to_vec(),
                        parents: document.parents().to_vec(),
                    });
                }
                tags.push(*contribution);
            }
            // One info per pack, at its first POI file in walk order.
            PoiFile::Gated(origin) => {
                if gated.insert(origin.pack.index()) {
                    let mut finding = gate_closed();
                    finding.location.pack = Some(origin.pack.clone());
                    finding.location.path = Some(origin.path.clone());
                    findings.push(finding);
                }
            }
        }
    }
    let effective: Vec<TypeRecord> = effective.into_values().collect();
    let resolved = resolve(&effective, &documents, &mut findings);
    Poi {
        types: types.finish(),
        tags: tags.finish(),
        resolved,
        findings,
    }
}

/// Adds to `winner`'s `parents` and `children` the entries of `other` it does not have.
fn union(winner: &mut PoiTag, other: &PoiTag) {
    for (list, more) in [
        (&mut winner.children, &other.children),
        (&mut winner.parents, &other.parents),
    ] {
        let Some(more) = more else {
            continue;
        };
        let list = list.get_or_insert_with(Vec::new);
        for tag in more {
            if !list.contains(tag) {
                list.push(tag.clone());
            }
        }
    }
}

/// The tag registry while the group combines. The registry's rule unions the documents of
/// different packs; the documents one pack has of an identifier are unioned here first, into
/// the first of them, so that they reach the rule as one (it would report a second definition
/// in one pack as a duplicate, which is no finding).
struct TagDocuments {
    builder: RegistryBuilder<PoiTag>,
    /// The pack the batch belongs to (its stack index), while the batch holds something.
    pack: Option<usize>,
    /// The contributions of that pack, in walk order (a pack's files are walked together).
    batch: Vec<Contribution<PoiTag>>,
}

impl TagDocuments {
    /// Adds the next tag file of the walk.
    fn push(&mut self, contribution: Contribution<PoiTag>) {
        let pack = match &contribution {
            Contribution::Stable(_, document) => document.origin().pack.index(),
            Contribution::Beta(_, document) => document.origin().pack.index(),
            Contribution::Rejected(_, rejected) => rejected.origin.pack.index(),
        };
        if self.pack != pack {
            self.flush();
        }
        self.pack = pack;
        if let Contribution::Stable(id, later) = &contribution {
            let earlier = self.batch.iter_mut().find_map(|held| match held {
                Contribution::Stable(held_id, document) if held_id == id => Some(document),
                _ => None,
            });
            if let Some(earlier) = earlier {
                let mut value = earlier.value().clone();
                union(&mut value, later.value());
                let mut diagnostics = earlier.diagnostics().clone();
                diagnostics.extend(later.diagnostics().iter().cloned());
                *earlier = Document::new(
                    value,
                    earlier.origin().clone(),
                    earlier.version().clone(),
                    diagnostics,
                );
                return;
            }
        }
        self.batch.push(contribution);
    }

    /// Hands the batch to the registry.
    fn flush(&mut self) {
        for contribution in self.batch.drain(..) {
            self.builder.push(contribution);
        }
    }

    /// The registry.
    fn finish(mut self) -> Registry<PoiTag> {
        self.flush();
        self.builder.finish()
    }
}

#[doc(hidden)]
/// The loader of `bp.poi()`: the POI directories of every behaviour pack, combined into the
/// group.
pub const LOADER: CollectionLoader<Poi, PoiFile> = CollectionLoader {
    walk: WALK,
    parse: parse_walk_file,
    combine,
    needs: &[],
    then: None,
};

impl Reported for Poi {
    fn reported(&self) -> Box<dyn Iterator<Item = &Diagnostic> + '_> {
        Box::new(self.diagnostics())
    }
}

#[cfg(test)]
mod reading_table {
    //! The family's reading table: what the pipeline reads each of its files and structs by.

    use bedrock_addon_core::versioning::testing::{assert_reading_table, reading_table};

    #[test]
    fn poi_type_reads_as_pinned() {
        assert_reading_table(
            "PoiTypeFamily",
            &reading_table::<super::PoiTypeFamily>(),
            r"
                text: Document, clamps beyond a double: false, duplicates kept in: []
                envelope: Always
                missing version: rejected: NoValidVersion
                below floor: NoParser
                empty document: Error
                1.26.40..=1.26.40 (0..=0): json Rejected/ConvertAndTell/ClampAndTell/Error/fatal/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Error/fatal/Refuse
            ",
        );
    }

    #[test]
    fn poi_tag_reads_as_pinned() {
        assert_reading_table(
            "PoiTagFamily",
            &reading_table::<super::PoiTagFamily>(),
            r"
                text: Document, clamps beyond a double: false, duplicates kept in: []
                envelope: Always
                missing version: rejected: NoValidVersion
                below floor: NoParser
                empty document: Error
                1.26.40..=1.26.40 (0..=0): json Rejected/ConvertAndTell/ClampAndTell/Error/fatal/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Error/fatal/Refuse
            ",
        );
    }
}
