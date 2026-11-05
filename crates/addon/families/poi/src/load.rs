//! Loading the POI documents: the two families' chain parameters, `Content` / `Definition`,
//! the walk of the POI directories and the group's combine step.

use std::collections::HashSet;
use std::convert::Infallible;

use super::history::{PoiTagAny, PoiTagFamily, PoiTypeAny, PoiTypeFamily};
use super::parse::V1_26_40;
use super::resolve::{TagRecord, TypeRecord, resolve};
use super::{Poi, PoiTag, PoiType, validate};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Phase, VanillaText,
};
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, NoBeta, Notice, Origin, Placed,
    Registry, RegistryBuilder, Rejected, default_path_flat,
};
use bedrock_addon_core::id::{Id, PoiTypeId, kind};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Prepared, Value, WriteStyle, write_json};
use bedrock_addon_core::load::{
    Accept, AppDataPlace, CollectionLoader, ListDepth, LoadContext, StackCx, Walk, WalkFile,
    WalkOrder,
};
use bedrock_addon_core::load::{Experiment, Reported};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput, PickedVersion};
use bedrock_addon_core::versioning::{
    ErrorMask, Family, Identity, InputKind, Loaded, MissingVersion, NoComponents, Parsed,
    StructIndex, ValidateCx, VersionStruct, Versioned, finish, parse_as_written,
};
use indexmap::IndexMap;

/// The directory of the type documents, which the game walks without descending into
/// sub-folders.
const TYPES: &str = "poi/blocks";

/// The directory of the tag documents, which the game also walks without descending into
/// sub-folders.
const TAGS: &str = "poi/tags";

/// A POI family: one struct, no beta struct.
pub(crate) trait PoiFamily:
    Family<Beta = Infallible, Latest: Definition + VersionStruct + Clone>
{
    /// The document's identifier.
    fn identifier(value: &Self::Latest) -> &Id<<Self::Latest as Definition>::Kind>;
}

/// The `Family`, `PoiFamily`, `Content`, `Definition` and `Placed` impls of one document.
macro_rules! document {
    (
        $family:ty, $root:ty, $any:ty, $kind:ty,
        name = $name:literal, key = $key:literal, directory = $dir:expr,
        validate = $validate:path $(,)?
    ) => {
        impl Family for $family {
            const NAME: &'static str = $name;
            const PAYLOAD_KEY: &'static str = $key;
            const DIRECTORIES: &'static [&'static str] = &[$dir];
            const FLOOR: FormatVersion = V1_26_40;
            type Latest = $root;
            type Beta = Infallible;
            type Component = NoComponents;
            type Any = $any;
            // The game's versioned loader does not check the version itself, so a missing
            // version is rejected with "no valid version passed by the caller or found in the
            // input".
            const MISSING_VERSION: MissingVersion = MissingVersion::Reject;
            // Both loaders read a document input: the version is read as that input reads it,
            // size limits apply, and a `null` where a value is read rejects the file.
            const INPUT: InputKind = InputKind::Document;

            /// The one slice of each document is read strictly: an unknown member rejects the
            /// file.
            fn error_mask(_: StructIndex) -> ErrorMask {
                ErrorMask::Strict
            }

            fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
                if let Loaded::Stable(value) = v {
                    $validate(value, &cx.at, cx.diagnostics);
                }
            }
        }

        impl PoiFamily for $family {
            fn identifier(value: &$root) -> &Id<$kind> {
                &value.description.identifier
            }
        }

        impl bedrock_addon_core::__private::Sealed for $root {}

        impl Content for $root {
            const NAME: &'static str = $name;
            const SIDES: &'static [PackSide] = &[PackSide::Behavior];

            fn parse(
                path: &PackPath,
                bytes: &[u8],
                cx: &LoadContext,
            ) -> Result<Document<Self>, Rejected> {
                parse_file::<$family>(PackRef::app_data(), PackSide::Behavior, path, bytes, cx).1
            }

            fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
                write_document::<$family>(self, out, style)
            }
        }

        impl bedrock_addon_core::document::JsonContent for $root {}

        impl Definition for $root {
            type Kind = $kind;
            type File = $root;
            type Beta = NoBeta<$kind>;
        }

        impl Placed for $root {
            /// `<directory>/<name>.json` for a `minecraft:` identifier and
            /// `<directory>/<namespace>_<name>.json` for any other, in the document type's own
            /// directory (`poi/blocks`, `poi/tags`): the walk reads no sub-folder, so a deeper
            /// path would never be loaded.
            fn default_path(&self) -> PackPath {
                default_path_flat($dir, &self.description.identifier)
            }
        }

        impl $root {
            /// Parses one file without upgrading it: the struct its `format_version` selects.
            /// The family has one struct, so this is [`Content::parse`]'s value before the
            /// document's own checks.
            ///
            /// # Errors
            ///
            /// Returns [`Rejected`] when the file cannot be read into a value at all, such as bytes
            /// that are not JSON or a document the family refuses; it carries every finding about
            /// the file.
            pub fn parse_as_written(
                path: &PackPath,
                bytes: &[u8],
                cx: &LoadContext,
            ) -> Result<Document<$any>, Rejected> {
                as_written::<$family>(path, bytes, cx)
            }

            /// Runs the family's rules on the value as if it were loaded from a behaviour
            /// pack with `cx`: the document is written and read back, so the parse constraints
            /// and the document's own rules both run. The value is not changed.
            pub fn validate(&self, cx: &LoadContext) -> Diagnostics {
                validate_detached::<$family>(self, cx)
            }
        }

        impl $any {
            /// Runs the rest of the load on a document [`parse_as_written`] gave: the same
            /// result `Content::parse` gives for its file.
            ///
            /// [`parse_as_written`]: $root::parse_as_written
            ///
            /// # Errors
            ///
            /// Returns [`Rejected`] when an upgrade step or a check refuses the document; it
            /// carries every finding about the file.
            pub fn upgrade(
                document: Document<Self>,
                cx: &LoadContext,
            ) -> Result<Document<$root>, Rejected> {
                upgrade::<$family>(document, cx)
            }
        }
    };
}

document!(
    PoiTypeFamily,
    PoiType,
    PoiTypeAny,
    kind::PoiType,
    name = "poi type",
    key = "minecraft:poi_block",
    directory = TYPES,
    validate = validate::poi_type,
);
document!(
    PoiTagFamily,
    PoiTag,
    PoiTagAny,
    kind::PoiTag,
    name = "poi tag",
    key = "minecraft:poi_tag",
    directory = TAGS,
    validate = validate::poi_tag,
);

/// The whole document of family `F`: `format_version` 1.26.40 and the payload, its members in
/// the struct's order (vanilla's bind order); a member left out (`None`) is not written.
fn write_document<F: PoiFamily>(
    value: &F::Latest,
    out: &mut Vec<u8>,
    style: WriteStyle,
) -> Result<(), EncodeError> {
    let mut object = facet_value::VObject::new();
    object.insert(
        "format_version",
        <F::Latest as Versioned>::VERSION.to_string(),
    );
    object.insert(F::PAYLOAD_KEY, value.write()?);
    write_json(&Value::from(object), out, style)
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

/// The origin of a file of `pack`.
fn origin(pack: PackRef, side: PackSide, path: &PackPath) -> Origin {
    Origin::file(pack, side, path.clone())
}

/// A file that did not load, with its findings placed.
fn reject(origin: Origin, diagnostics: Diagnostics) -> Rejected {
    Rejected::located(origin, diagnostics)
}

/// What loading one file gives: the identifier it names, when the payload has one, and the
/// document or its rejection.
pub(crate) type FileOutcome<T> = (
    Option<Id<<T as Definition>::Kind>>,
    Result<Document<T>, Rejected>,
);

/// Loads one file of family `F`.
pub(crate) fn parse_file<F: PoiFamily>(
    pack: PackRef,
    side: PackSide,
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> FileOutcome<F::Latest> {
    let origin = origin(pack, side, path);
    if !enabled(cx) {
        return (None, Err(unread(origin, gate_closed())));
    }
    let Ok(prepared) = Prepared::new(bytes) else {
        let mut diagnostics = Diagnostics::default();
        diagnostics.push(malformed());
        diagnostics.push(failed_to_load());
        return (None, Err(reject(origin, diagnostics)));
    };
    let text = prepared.as_str();
    let parsed = parse_stage::<F>(text, cx);
    let identifier = parsed.identity.identifier.clone().map(Id::parse);
    (identifier, finish_file::<F>(parsed, origin, cx))
}

/// The parse stage of one file of family `F`, the floor text corrected.
fn parse_stage<F: PoiFamily>(text: &str, cx: &LoadContext) -> Parsed<F> {
    let document = JsonSlice::new(text);
    let mut parsed = parse_as_written::<F>(document, cx);
    no_parser_below_floor(&mut parsed.diagnostics, document);
    parsed
}

/// Below 1.26.40 vanilla has no slice at all: its text is "no parser available", where the
/// generic floor check says "To use …, use json format version … or higher". The version is the
/// one the document reader picked from the text (a number's token too), printed as the game
/// prints a parsed version: `1.26` is `1.26.0`.
fn no_parser_below_floor(diagnostics: &mut Diagnostics, document: JsonSlice<'_>) {
    let written = || match FormatVersionInput::pick(document.text) {
        PickedVersion::Version { input, written } => {
            bedrock_addon_core::versioning::no_parser_version(&input, &written)
        }
        PickedVersion::Absent | PickedVersion::Malformed => String::new(),
    };
    let found = std::mem::take(diagnostics);
    for mut finding in found {
        if let DiagnosticCode::UnsupportedVersion {
            text: Some(text @ VanillaText::UseFormatVersion { .. }),
        } = &mut finding.code
        {
            *text = VanillaText::NoParserForVersion { version: written() };
        }
        diagnostics.push(finding);
    }
}

/// The rest of the load of a parsed file.
fn finish_file<F: PoiFamily>(
    parsed: Parsed<F>,
    origin: Origin,
    cx: &LoadContext,
) -> Result<Document<F::Latest>, Rejected> {
    let outcome = finish(parsed, cx);
    let mut diagnostics = outcome.diagnostics;
    if let Some(Loaded::Stable(value)) = outcome.value {
        diagnostics.locate(&origin);
        Ok(Document::new(value, origin, outcome.version, diagnostics))
    } else {
        diagnostics.push(failed_to_load());
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

/// `Root::validate` of family `F`: the findings of loading what the value writes.
fn validate_detached<F: PoiFamily>(value: &F::Latest, cx: &LoadContext) -> Diagnostics
where
    F::Latest: Placed,
{
    let path = value.default_path();
    let text = match value.to_json() {
        Ok(text) => text,
        Err(error) => {
            let mut diagnostics = Diagnostics::default();
            diagnostics.push(unwritable(&error, F::PAYLOAD_KEY));
            return diagnostics;
        }
    };
    let loaded = parse_file::<F>(
        PackRef::app_data(),
        PackSide::Behavior,
        &path,
        text.as_bytes(),
        cx,
    );
    match loaded.1 {
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

/// `Root::parse_as_written` of family `F`.
fn as_written<F: PoiFamily>(
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> Result<Document<F::Any>, Rejected> {
    let origin = origin(PackRef::app_data(), PackSide::Behavior, path);
    if !enabled(cx) {
        return Err(unread(origin, gate_closed()));
    }
    let Ok(prepared) = Prepared::new(bytes) else {
        return Err(unread(origin, malformed()));
    };
    let text = prepared.as_str();
    let parsed = parse_stage::<F>(text, cx);
    let mut diagnostics = parsed.diagnostics;
    match parsed.value {
        Some(value) => {
            diagnostics.locate(&origin);
            Ok(Document::new(value, origin, parsed.version, diagnostics))
        }
        None => Err(reject(origin, diagnostics)),
    }
}

/// `history::*Any::upgrade` of family `F`: rebuilds the parse stage from the document and
/// finishes it.
fn upgrade<F: PoiFamily>(
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
    let parsed = Parsed::<F>::new(
        Some(value),
        version,
        written,
        Identity::default(),
        ActorType::Mob,
        false,
        at,
        diagnostics,
    );
    finish_file::<F>(parsed, origin, cx)
}

// ---- the group -------------------------------------------------------------------------------

/// What one walked file parses to.
#[derive(Debug)]
pub enum PoiFile {
    /// A file of `poi/blocks`, loaded as a `minecraft:poi_block` document.
    Type(Box<Contribution<PoiType>>),
    /// A file of `poi/tags`, loaded as a `minecraft:poi_tag` document.
    Tag(Box<Contribution<PoiTag>>),
    /// A file of a pack loaded without the `poi` experiment: not read, only where it is.
    Gated(Box<Origin>),
}

/// Files whose extension is exactly `json`, as for the game's other category loaders.
fn accept(path: &PackPath) -> bool {
    let name = path.as_str().rsplit('/').next().unwrap_or_default();
    name.rsplit_once('.')
        .is_some_and(|(_, extension)| extension == "json")
}

/// Every `*.json` directly in `poi/blocks` and `poi/tags`, packs from the lowest up: the
/// definition loaded last wins. Neither loader reads a sub-folder.
const WALK: Walk = Walk {
    directories: &[TYPES, TAGS],
    depth: ListDepth::Children,
    accept: Accept::Fn(accept),
    order: WalkOrder::BottomUp,
    directory_major: false,
    app_data: &[],
    app_data_place: AppDataPlace::After,
    in_pack: None,
    shadowed: None,
    read_error: None,
};

/// One family's contribution from its file.
fn contribution<F: PoiFamily>(file: &WalkFile, cx: &LoadContext) -> Contribution<F::Latest> {
    match parse_file::<F>(file.pack.clone(), file.side, &file.path, &file.bytes, cx) {
        (_, Ok(document)) => Contribution::Stable(F::identifier(&document).clone(), document),
        (id, Err(rejected)) => Contribution::Rejected(id, rejected),
    }
}

/// Parses one walked file as the document of its directory: `poi/blocks` holds types and
/// `poi/tags` tags, whatever the payload key says (a file of the other kind has no payload
/// under the key its loader asks for: `MissingPayload`). Without the `poi` experiment the
/// file is not read at all.
fn parse_walk_file(file: &WalkFile, cx: &LoadContext) -> PoiFile {
    if !enabled(cx) {
        return PoiFile::Gated(Box::new(origin(file.pack.clone(), file.side, &file.path)));
    }
    if file.key.as_str().starts_with(TAGS) {
        PoiFile::Tag(Box::new(contribution::<PoiTagFamily>(file, cx)))
    } else {
        PoiFile::Type(Box::new(contribution::<PoiTypeFamily>(file, cx)))
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

/// The loader of `bp.poi()`: the POI directories of every behaviour pack, combined into the
/// group.
pub const fn load_group() -> CollectionLoader<Poi, PoiFile> {
    CollectionLoader {
        walk: WALK,
        parse: parse_walk_file,
        combine,
        needs: &[],
        then: None,
    }
}

impl Reported for Poi {
    fn reported(&self) -> Box<dyn Iterator<Item = &Diagnostic> + '_> {
        Box::new(self.diagnostics())
    }
}
