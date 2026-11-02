//! Loading the jigsaw documents: the five families' chain parameters, `Content` /
//! `Definition`, the walk of `worldgen/`, and the group's combine rule with the checks that
//! need every pool.

use std::collections::{HashMap, HashSet};
use std::convert::Infallible;

use super::history::{
    MetadataAny, MetadataFamily, PoolAny, PoolFamily, ProcessorListAny, ProcessorListFamily,
    StructureAny, StructureFamily, StructureSetAny, StructureSetFamily,
};
use super::parse::V1_21_20;
use super::{
    Jigsaw, JigsawMetadata, JigsawStructure, MetadataRegistry, PoolElement, ProcessorList,
    StructureSet, TemplatePool, validate,
};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity, VanillaText,
};
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, FilesBuilder, FilesRule, NoBeta,
    Notice, Origin, Placed, RegistryBuilder, Rejected, VersionInfo, default_path,
};
use bedrock_addon_core::id::{Id, Kind, kind};
use bedrock_addon_core::json::{
    EncodeError, EnvelopeVersion, JsonSlice, MAX_JSON_DEPTH, Unrepresentable, Value, WriteStyle,
    check_depth, probe_envelope, strip_bom, write_json,
};
use bedrock_addon_core::load::Reported;
use bedrock_addon_core::load::{
    AppDataPlace, CollectionLoader, ListDepth, LoadContext, StackCx, Walk, WalkFile, WalkOrder,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput};
use bedrock_addon_core::versioning::{
    ComponentTable, EnvelopeCheck, ErrorMask, Family, Identity, InputKind, Loaded, NoComponents,
    Parsed, Selection, StructIndex, ValidateCx, VersionStruct, Versioned, finish, parse_as_written,
    select,
};

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
const V1_21_50: FormatVersion = FormatVersion::new(1, 21, 50);

/// A JSON jigsaw family: one struct, no beta struct, and the line vanilla logs after a failed
/// parse.
pub(crate) trait JsonFamily:
    Family<Beta = Infallible, Latest: Definition + VersionStruct + Clone>
{
    /// "Failed to parse … '{}'.", with the file's path.
    const FAILED: &'static str;
    /// The document's identifier.
    fn identifier(value: &Self::Latest) -> &Id<<Self::Latest as Definition>::Kind>;
}

/// The `Family`, `JsonFamily`, `Content`, `Definition` and `Placed` impls of one JSON family.
macro_rules! json_family {
    (
        $family:ty, $root:ty, $any:ty, $kind:ty,
        name = $name:literal, key = $key:literal, directory = $dir:expr,
        failed = $failed:literal, validate = $validate:path $(,)?
    ) => {
        impl Family for $family {
            // What the document does not read is kept for write-back: a number no double holds is
            // the largest finite one, with a finding at it.
            const CLAMPS_BEYOND_DOUBLE: bool = true;
            const NAME: &'static str = $name;
            const PAYLOAD_KEY: &'static str = $key;
            const DIRECTORIES: &'static [&'static str] = &[$dir];
            const FLOOR: FormatVersion = V1_21_20;
            type Latest = $root;
            type Beta = Infallible;
            type Component = NoComponents;
            type Any = $any;
            // The game checks the member count of a document from 1.21.20 only; below it no
            // struct exists, so checking always changes only which text rejects a
            // three-member file below 1.21.20.
            const ENVELOPE: EnvelopeCheck = EnvelopeCheck::Always;
            // The game reads all four documents as Minecraft documents: the version as the
            // JSON reader takes it, the size limits, and `null` as a member of the wrong
            // type.
            const INPUT: InputKind = InputKind::Document;

            /// The game's four slices (processor list, template pool, structure, structure
            /// set) are not strict: unknown members are warnings.
            fn error_mask(_: StructIndex) -> ErrorMask {
                ErrorMask::Unstrict
            }

            fn components() -> &'static ComponentTable<Self> {
                &ComponentTable::EMPTY
            }

            fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
                if let Loaded::Stable(value) = v {
                    $validate(value, &cx.at, cx.diagnostics);
                }
            }

            fn actor_type(_: &Identity) -> ActorType {
                ActorType::Mob
            }
        }

        impl JsonFamily for $family {
            const FAILED: &'static str = $failed;

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

            fn to_json(&self) -> Result<String, EncodeError> {
                let mut out = Vec::new();
                write_document::<$family>(self, &mut out, WriteStyle::default())?;
                Ok(String::from_utf8(out).unwrap_or_default())
            }
        }

        impl Definition for $root {
            type Kind = $kind;
            type File = $root;
            type Beta = NoBeta<$kind>;
        }

        impl Placed for $root {
            /// `<directory>/<namespace>/<name>.json`.
            fn default_path(&self) -> PackPath {
                default_path($dir, &self.description.identifier)
            }
        }

        impl $root {
            /// Parses one file without upgrading it: the struct its `format_version` selects.
            /// The family has one struct, so this is [`Content::parse`]'s value before the
            /// registration checks.
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

json_family!(
    StructureFamily,
    JigsawStructure,
    StructureAny,
    kind::JigsawStructure,
    name = "jigsaw structure",
    key = "minecraft:jigsaw",
    directory = STRUCTURES,
    failed = "Failed to parse jigsaw structure definition '{}'.",
    validate = validate::structure,
);
json_family!(
    PoolFamily,
    TemplatePool,
    PoolAny,
    kind::TemplatePool,
    name = "template pool",
    key = "minecraft:template_pool",
    directory = POOLS,
    failed = "Failed to parse jigsaw structure template pool '{}'.",
    validate = validate::pool,
);
json_family!(
    StructureSetFamily,
    StructureSet,
    StructureSetAny,
    kind::StructureSet,
    name = "structure set",
    key = "minecraft:structure_set",
    directory = SETS,
    failed = "Failed to parse structure set definition '{}'.",
    validate = validate::set,
);
json_family!(
    ProcessorListFamily,
    ProcessorList,
    ProcessorListAny,
    kind::ProcessorList,
    name = "processor list",
    key = "minecraft:processor_list",
    directory = PROCESSORS,
    failed = "Failed to parse jigsaw structure processor list '{}'.",
    validate = validate::processors,
);

impl Family for MetadataFamily {
    const NAME: &'static str = "jigsaw structure metadata";
    const PAYLOAD_KEY: &'static str = "minecraft:jigsaw_structure_metadata";
    const DIRECTORIES: &'static [&'static str] = &[METADATA];
    const FLOOR: FormatVersion = V1_21_50;
    type Latest = MetadataRegistry;
    type Beta = Infallible;
    type Component = NoComponents;
    type Any = MetadataAny;

    fn components() -> &'static ComponentTable<Self> {
        &ComponentTable::EMPTY
    }
}

// ---- one JSON file -------------------------------------------------------------------------

/// The text of a file, after the BOM and the depth check; `None` when it is not UTF-8 or too
/// deep.
fn text(bytes: &[u8]) -> Option<&str> {
    let bytes = strip_bom(bytes);
    check_depth(bytes, MAX_JSON_DEPTH).ok()?;
    std::str::from_utf8(bytes).ok()
}

/// The finding of an unreadable file.
fn malformed() -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::MalformedInput {
            message: "not UTF-8 JSON within the nesting limit".to_owned(),
            text: None,
        },
        Severity::Error,
        Phase::Parse,
        Location::default(),
    )
}

/// The origin of a file of `pack`.
fn origin(pack: PackRef, side: PackSide, path: &PackPath) -> Origin {
    Origin::new(pack, side, path.clone(), None, Vec::new())
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

/// Below 1.21.20 vanilla has no slice at all: the game says "no parser available",
/// where the generic floor check says "To use …, use json format version … or higher".
fn no_parser_below_floor(diagnostics: &mut Diagnostics, document: JsonSlice<'_>) {
    let written = || match probe_envelope(document).map(|e| e.format_version) {
        Ok(EnvelopeVersion::String(text)) => text,
        Ok(EnvelopeVersion::Array([a, b, c])) => format!("{a}.{b}.{c}"),
        _ => String::new(),
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

/// After a failed parse (a parse error, not a registration check), the family's line
/// with the file's path.
fn failure_line<F: JsonFamily>(diagnostics: &mut Diagnostics, path: &PackPath) {
    let parse_error = diagnostics
        .iter()
        .any(|d| d.severity >= Severity::Error && d.phase == Phase::Parse);
    if parse_error {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::DocumentRejected {
                text: Some(VanillaText::Formatted {
                    template: F::FAILED,
                    args: vec![path.as_str().to_owned()],
                }),
            },
            Severity::Error,
            Phase::Parse,
            Location::default(),
        ));
    }
}

/// The parse stage of one file of family `F`, the floor text corrected.
fn parse_stage<F: JsonFamily>(text: &str, cx: &LoadContext) -> Parsed<F> {
    let document = JsonSlice::new(text);
    let mut parsed = parse_as_written::<F>(document, cx);
    no_parser_below_floor(&mut parsed.diagnostics, document);
    parsed
}

/// What loading one file gives: the identifier it names, when the payload has one, and the
/// document or its rejection.
pub(crate) type FileOutcome<T> = (
    Option<Id<<T as Definition>::Kind>>,
    Result<Document<T>, Rejected>,
);

/// Loads one file of family `F`: the identifier it names (when the payload has one) and the
/// document or its rejection.
pub(crate) fn parse_file<F: JsonFamily>(
    pack: PackRef,
    side: PackSide,
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> FileOutcome<F::Latest> {
    let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
    let origin = origin(pack, side, path);
    let Some(text) = text(bytes) else {
        let mut diagnostics = Diagnostics::default();
        diagnostics.push(malformed());
        failure_line::<F>(&mut diagnostics, path);
        locate(&mut diagnostics, &origin);
        return (None, Err(Rejected::new(origin, diagnostics)));
    };
    let parsed = parse_stage::<F>(text, cx);
    let identifier = parsed.identity.identifier.clone().map(Id::parse);
    (identifier, finish_file::<F>(parsed, origin, cx))
}

/// The rest of the load of a parsed file.
fn finish_file<F: JsonFamily>(
    parsed: Parsed<F>,
    origin: Origin,
    cx: &LoadContext,
) -> Result<Document<F::Latest>, Rejected> {
    let outcome = finish(parsed, cx);
    let mut diagnostics = outcome.diagnostics;
    if let Some(Loaded::Stable(value)) = outcome.value {
        locate(&mut diagnostics, &origin);
        Ok(Document::new(value, origin, outcome.version, diagnostics))
    } else {
        failure_line::<F>(&mut diagnostics, &origin.path);
        locate(&mut diagnostics, &origin);
        Err(Rejected::new(origin, diagnostics))
    }
}

/// [`parse_as_written`] of family `F`.
fn as_written<F: JsonFamily>(
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> Result<Document<F::Any>, Rejected> {
    let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
    let origin = origin(PackRef::app_data(), PackSide::Behavior, path);
    let Some(text) = text(bytes) else {
        let mut diagnostics = Diagnostics::default();
        diagnostics.push(malformed());
        failure_line::<F>(&mut diagnostics, path);
        locate(&mut diagnostics, &origin);
        return Err(Rejected::new(origin, diagnostics));
    };
    let parsed = parse_stage::<F>(text, cx);
    let mut diagnostics = parsed.diagnostics;
    if let Some(value) = parsed.value {
        locate(&mut diagnostics, &origin);
        Ok(Document::new(value, origin, parsed.version, diagnostics))
    } else {
        failure_line::<F>(&mut diagnostics, path);
        locate(&mut diagnostics, &origin);
        Err(Rejected::new(origin, diagnostics))
    }
}

/// The upgrade of a document of family `F`: rebuilds the parse stage from the document and
/// finishes it.
fn upgrade<F: JsonFamily>(
    document: Document<F::Any>,
    cx: &LoadContext,
) -> Result<Document<F::Latest>, Rejected> {
    let origin = document.origin().clone();
    let version = document.version().clone();
    let diagnostics = document.diagnostics().clone();
    let value = document.into_value();
    let mut at = JsonPath::new();
    at.push(F::PAYLOAD_KEY);
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

/// The whole document of family `F`: `format_version` 1.21.20 and the payload.
fn write_document<F: JsonFamily>(
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

// ---- one metadata file ---------------------------------------------------------------------

/// The leading string of a metadata file and the bytes after it: an unsigned LEB128 length,
/// then that many bytes of UTF-8. `None` when the stream
/// ends early or the bytes are not UTF-8.
fn leading_string(bytes: &[u8]) -> Option<(&str, &[u8])> {
    let mut length: u64 = 0;
    let mut shift = 0;
    let mut at = 0;
    loop {
        let byte = *bytes.get(at)?;
        at += 1;
        length |= u64::from(byte & 0x7f).checked_shl(shift)?;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift >= 35 {
            return None;
        }
    }
    let end = at.checked_add(usize::try_from(length).ok()?)?;
    let text = std::str::from_utf8(bytes.get(at..end)?).ok()?;
    Some((text, bytes.get(end..)?))
}

/// A metadata file's finding at the file.
fn metadata_finding(code: DiagnosticCode, severity: Severity) -> Diagnostic {
    Diagnostic::new(code, severity, Phase::Parse, Location::default())
}

/// Loads one `worldgen/metadata/*.dat` file: the leading version string checked as vanilla
/// checks it, the body kept undecoded with a `SkippedCacheDocument` information (the crate's
/// own).
pub(crate) fn parse_metadata(
    pack: PackRef,
    side: PackSide,
    path: &PackPath,
    bytes: &[u8],
) -> Result<Document<JigsawMetadata>, Rejected> {
    let origin = origin(pack, side, path);
    let mut diagnostics = Diagnostics::default();
    let reject = |mut diagnostics: Diagnostics, finding: Diagnostic| {
        diagnostics.push(finding);
        locate(&mut diagnostics, &origin);
        Err(Rejected::new(origin.clone(), diagnostics))
    };
    let Some((written, body)) = leading_string(bytes) else {
        return reject(diagnostics, bad_leading_string(path));
    };
    // An empty string is not parsed as a version, so the game ends up with no version at all.
    if written.is_empty() {
        return reject(
            diagnostics,
            metadata_finding(
                DiagnosticCode::MissingFormatVersion {
                    text: Some(VanillaText::NoValidVersion),
                },
                Severity::Error,
            ),
        );
    }
    let Some(input) = FormatVersionInput::parse(written) else {
        return reject(diagnostics, bad_leading_string(path));
    };
    let Selection::Struct(index) = select::<MetadataFamily>(&input) else {
        return reject(
            diagnostics,
            metadata_finding(
                DiagnosticCode::UnsupportedVersion {
                    text: Some(VanillaText::NoParserForVersion {
                        version: written.to_owned(),
                    }),
                },
                Severity::Error,
            ),
        );
    };
    let effective = input.version.unwrap_or(V1_21_50);
    let parsed_as =
        <MetadataAny as bedrock_addon_core::versioning::VersionChain<MetadataFamily>>::VERSIONS
            .get(index)
            .copied()
            .unwrap_or(effective);
    diagnostics.push(metadata_finding(
        DiagnosticCode::SkippedCacheDocument,
        Severity::Info,
    ));
    locate(&mut diagnostics, &origin);
    let value = JigsawMetadata {
        format_version: written.to_owned(),
        body: body.to_vec(),
    };
    let version = VersionInfo::new(input.version, false, effective, parsed_as, false);
    Ok(Document::new(value, origin, version, diagnostics))
}

/// "Failed to parse format version string of file %s; it will be skipped."
fn bad_leading_string(path: &PackPath) -> Diagnostic {
    metadata_finding(
        DiagnosticCode::MalformedFormatVersion {
            text: Some(VanillaText::Formatted {
                template: "Failed to parse format version string of file {}; it will be skipped.",
                args: vec![path.as_str().to_owned()],
            }),
        },
        Severity::Error,
    )
}

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

impl JigsawMetadata {
    /// The file's bytes: the version string, length-prefixed, then the body as read.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(5 + self.format_version.len() + self.body.len());
        let mut length = self.format_version.len();
        loop {
            // The low seven bits of `length`, with the continuation bit when more follow.
            let low = u8::try_from(length & 0x7f).unwrap_or(0);
            length >>= 7;
            if length == 0 {
                out.push(low);
                break;
            }
            out.push(low | 0x80);
        }
        out.extend_from_slice(self.format_version.as_bytes());
        out.extend_from_slice(&self.body);
        out
    }
}

// ---- the group -----------------------------------------------------------------------------

/// What one walked file parses to.
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
const WALK: Walk = Walk {
    directories: &[STRUCTURES, POOLS, SETS, PROCESSORS, METADATA],
    depth: ListDepth::Recursive,
    accept,
    order: WalkOrder::TopDown,
    directory_major: false,
    app_data: &[],
    app_data_place: AppDataPlace::After,
    in_pack: None,
    shadowed: None,
    read_error: None,
};

/// One family's contribution from its file.
fn contribution<F: JsonFamily>(file: &WalkFile, cx: &LoadContext) -> Contribution<F::Latest> {
    match parse_file::<F>(file.pack.clone(), file.side, &file.path, &file.bytes, cx) {
        (_, Ok(document)) => Contribution::Stable(F::identifier(&document).clone(), document),
        (id, Err(rejected)) => Contribution::Rejected(id, rejected),
    }
}

/// Parses one walked file by its directory.
fn parse_walk_file(file: &WalkFile, cx: &LoadContext) -> JigsawFile {
    match WALK.matches(&file.key) {
        Some(0) => JigsawFile::Structure(Box::new(contribution::<StructureFamily>(file, cx))),
        Some(1) => JigsawFile::Pool(Box::new(contribution::<PoolFamily>(file, cx))),
        Some(2) => JigsawFile::Set(Box::new(contribution::<StructureSetFamily>(file, cx))),
        Some(3) => JigsawFile::Processors(Box::new(contribution::<ProcessorListFamily>(file, cx))),
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
/// one pack the first file in walk order; a dropped definition is a `ShadowedDefinition` information.
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
                    self.builder.push_diagnostic(Diagnostic::new(
                        DiagnosticCode::ShadowedDefinition { text: None },
                        Severity::Info,
                        Phase::Validate,
                        at_document(document, JsonPath::new()),
                    ));
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

/// A location in `document`'s file.
fn at_document<T>(document: &Document<T>, json_path: JsonPath) -> Location {
    let origin = document.origin();
    Location::new(
        Some(origin.pack.clone()),
        Some(origin.path.clone()),
        json_path,
        None,
    )
}

/// The checks over every effective template pool.
#[derive(Default)]
struct PoolChecks {
    /// Each single element's location, with its processors and projection.
    elements: HashMap<String, (String, super::Projection)>,
    /// The pool names, lowercased.
    names: HashSet<String>,
}

impl PoolChecks {
    /// The findings of one pool, registered after the ones seen so far.
    fn check(&mut self, pool: &Document<TemplatePool>, stack: &StackCx<'_>) -> Vec<Diagnostic> {
        let mut found = Vec::new();
        let mut at = JsonPath::new();
        at.push("minecraft:template_pool");
        // The game lowercases a pool name when it registers it; release builds keep the first.
        let name = pool.description.identifier.as_str().to_lowercase();
        let built_in =
            stack.options.built_ins.as_ref().is_some_and(|built_ins| {
                built_ins.contains(<kind::TemplatePool as Kind>::NAME, &name)
            });
        if !self.names.insert(name.clone()) || built_in {
            let mut path = at.clone();
            path.push("description");
            path.push("identifier");
            found.push(Diagnostic::new(
                DiagnosticCode::DuplicateIdentifier {
                    text: Some(VanillaText::Formatted {
                        template: "Jigsaw pool name `{}` already registered",
                        args: vec![name],
                    }),
                },
                // The game only asserts here, with no content-log line, so the crate reports
                // an information of its own.
                Severity::Info,
                Phase::Validate,
                at_document(pool, path),
            ));
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
                    let mut path = at.clone();
                    path.push("elements");
                    path.push_index(i);
                    path.push("element");
                    path.push("location");
                    found.push(Diagnostic::new(
                        DiagnosticCode::InvalidValue {
                            text: Some(VanillaText::Formatted {
                                template: "Redefinition of existing element with location '{}'.",
                                args: vec![location.as_str().to_owned()],
                            }),
                        },
                        // A crate-own lint: Info.
                        Severity::Info,
                        Phase::Validate,
                        at_document(pool, path),
                    ));
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
/// dropped silently, which is the first-wins rule of the registry above), so the one collision is a pack's `minecraft:trial_chambers` against
/// the game's own: the pack's set stays effective and the game's record is refused with the
/// text. The finding is placed at the pack's file.
fn game_record_refused(document: &Document<StructureSet>) -> Option<Diagnostic> {
    (document.description.identifier.as_str() == GAME_RECORDED_SET).then(|| {
        Diagnostic::new(
            DiagnosticCode::DuplicateIdentifier {
                text: Some(VanillaText::Formatted {
                    template: "Structure set '{}' has already been registered",
                    args: vec![GAME_RECORDED_SET.to_owned()],
                }),
            },
            Severity::Error,
            Phase::Validate,
            at_document(document, JsonPath::new()),
        )
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

/// The loader of `bp.jigsaw()`: the five directories of every behaviour pack, combined into
/// the group.
#[must_use]
pub const fn load_group() -> CollectionLoader<Jigsaw, JigsawFile> {
    CollectionLoader {
        walk: WALK,
        parse: parse_walk_file,
        combine,
        needs: &[],
        then: None,
    }
}

impl Reported for Jigsaw {
    fn reported(&self) -> Box<dyn Iterator<Item = &Diagnostic> + '_> {
        Box::new(self.diagnostics())
    }
}
