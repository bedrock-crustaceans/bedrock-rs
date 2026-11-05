//! Loading voxel shapes: the family's chain parameters, `Content` / `Definition`, the walk of
//! `shapes/` and the combine rule.

use super::{FLOOR, VoxelShape, texts, validate};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, Location, Phase, Severity, VanillaText,
};
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, NoBeta, Notice, Origin, Placed,
    Registry, RegistryBuilder, Rejected, default_path,
};
use bedrock_addon_core::id::kind;
use bedrock_addon_core::json::{
    EncodeError, JsonSlice, MAX_JSON_DEPTH, WriteStyle, check_depth, strip_bom,
};
use bedrock_addon_core::load::{
    Accept, AppDataPlace, CollectionLoader, ListDepth, LoadContext, StackCx, Walk, WalkFile,
    WalkOrder,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput, PickedVersion};
use bedrock_addon_core::versioning::{
    ComponentTable, EnvelopeCheck, ErrorMask, Family, InputKind, Loaded, MissingVersion,
    NoComponents, StructIndex, ValidateCx, finish, parse_as_written,
};

/// The voxel-shape family of the version-chain machinery.
#[derive(Clone, Copy, Debug)]
pub(crate) struct VoxelShapeFamily;

bedrock_addon_core::version_chain! {
    family = VoxelShapeFamily;
    /// A voxel-shape document parsed by the struct its `format_version` selects: there is one.
    #[derive(Clone, Debug, PartialEq)]
    pub(crate) enum Any {
        /// Format 1.21.110 up to the engine.
        V1_21_110(VoxelShape),
    }
    molang = none;
}

impl Family for VoxelShapeFamily {
    const NAME: &'static str = "voxel shape";
    const PAYLOAD_KEY: &'static str = "minecraft:voxel_shape";
    const DIRECTORIES: &'static [&'static str] = &["shapes"];
    const FLOOR: FormatVersion = FLOOR;
    type Latest = VoxelShape;
    type Beta = std::convert::Infallible;
    type Component = NoComponents;
    type Any = Any;
    // The document input takes the raw text below 1.21.110 or without a version, and exactly
    // two members from 1.21.110 on.
    const ENVELOPE: EnvelopeCheck = EnvelopeCheck::From(FLOOR);
    // Without a version the loader falls back to the caller's, which is none: "no valid version
    // passed by the caller or found in the input" and the file is skipped.
    const MISSING_VERSION: MissingVersion = MissingVersion::Reject;
    // The game reads a shape through the document-input reader of the cereal-based families:
    // the version is read from the document, the size limits apply, and a member written
    // `null` fails its type.
    const INPUT: InputKind = InputKind::Document;

    /// The one slice is registered with the strict error mask: an unknown member rejects the
    /// file.
    fn error_mask(_: StructIndex) -> ErrorMask {
        ErrorMask::Strict
    }

    fn components() -> &'static ComponentTable<Self> {
        &ComponentTable::EMPTY
    }

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(shape) = v {
            validate::rules(shape, cx.load.is_base_game_pack, &cx.at, cx.diagnostics);
        }
    }
}

impl bedrock_addon_core::__private::Sealed for VoxelShape {}

impl Content for VoxelShape {
    const NAME: &'static str = "voxel shape";
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        parse_file(PackRef::app_data(), PackSide::Behavior, path, bytes, cx)
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        self.write_into(out, style)
    }

    fn to_json(&self) -> Result<String, EncodeError> {
        let mut out = Vec::new();
        self.write_json(&mut out, WriteStyle::default())?;
        Ok(String::from_utf8(out).unwrap_or_default())
    }
}

impl Definition for VoxelShape {
    type Kind = kind::VoxelShape;
    type File = VoxelShape;
    type Beta = NoBeta<kind::VoxelShape>;
}

impl Placed for VoxelShape {
    /// `shapes/<identifier>.json`, the identifier's `:` written `_`.
    fn default_path(&self) -> PackPath {
        default_path("shapes", &self.description.identifier)
    }
}

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

/// Gives every finding the document's pack and file.
fn locate(diagnostics: &mut Diagnostics, origin: &Origin) {
    let found = std::mem::take(diagnostics);
    for mut finding in found {
        finding.location.pack = Some(origin.pack.clone());
        finding.location.path = Some(origin.path.clone());
        diagnostics.push(finding);
    }
}

/// Below 1.21.110 vanilla has no slice at all: the game says "no parser available" with the
/// version as written, where core's floor check says "To use …, use json format version …
/// or higher". The version is the one the document input picked from the text (a number's
/// token, an array in dotted form).
fn no_parser_below_floor(diagnostics: &mut Diagnostics, document: JsonSlice<'_>) {
    let written = || match FormatVersionInput::pick(document.text) {
        PickedVersion::Version { written, .. } => written,
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

/// After a file the schema loader rejected (a parse error; not a namespace or geometry
/// failure, which log one line of their own), vanilla's "Failed to load voxel shape file" with
/// the file's path in its pack.
fn failure_line(diagnostics: &mut Diagnostics, path: &PackPath) {
    let rejected_by_schema = diagnostics
        .iter()
        .any(|finding| finding.severity >= Severity::Error && finding.phase == Phase::Parse);
    if rejected_by_schema {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::DocumentRejected {
                text: Some(texts::failed_to_load(path.as_str())),
            },
            Severity::Error,
            Phase::Parse,
            Location::default(),
        ));
    }
}

/// Loads one shape file of `pack`.
pub(crate) fn parse_file(
    pack: PackRef,
    side: PackSide,
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> Result<Document<VoxelShape>, Rejected> {
    let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
    let origin = Origin::new(pack, side, path.clone(), None, Vec::new());
    let Some(text) = text(bytes) else {
        let mut diagnostics = Diagnostics::default();
        diagnostics.push(malformed());
        failure_line(&mut diagnostics, path);
        locate(&mut diagnostics, &origin);
        return Err(Rejected::new(origin, diagnostics));
    };
    let document = JsonSlice::new(text);
    let outcome = finish(parse_as_written::<VoxelShapeFamily>(document, cx), cx);
    let mut diagnostics = outcome.diagnostics;
    no_parser_below_floor(&mut diagnostics, document);
    failure_line(&mut diagnostics, path);
    locate(&mut diagnostics, &origin);
    match outcome.value {
        Some(Loaded::Stable(shape)) => {
            Ok(Document::new(shape, origin, outcome.version, diagnostics))
        }
        _ => Err(Rejected::new(origin, diagnostics)),
    }
}

// ---- the collection ----------------------------------------------------------------------

/// A shape file: a file below `shapes/` whose extension is `json`, compared as written.
// The extension is compared case-sensitively.
fn is_json(path: &PackPath) -> bool {
    path.as_str()
        .rsplit_once('.')
        .is_some_and(|(_, extension)| extension == "json")
}

/// `shapes/**/*.json`, packs bottom-up: vanilla walks the stack from its last pack to its first
/// and binds an identifier to the shape it loads last, so the highest pack's shape is the
/// effective one. Sub-folders are read in every pack (the game reads them only in packs
/// outside the app package).
const WALK: Walk = Walk {
    directories: &["shapes"],
    depth: ListDepth::Recursive,
    accept: Accept::Fn(is_json),
    order: WalkOrder::BottomUp,
    directory_major: false,
    app_data: &[],
    app_data_place: AppDataPlace::After,
    in_pack: None,
    shadowed: None,
    read_error: None,
};

/// One file's contribution.
fn parse_walk_file(file: &WalkFile, cx: &LoadContext) -> Contribution<VoxelShape> {
    match parse_file(file.pack.clone(), file.side, &file.path, &file.bytes, cx) {
        Ok(document) => Contribution::Stable(document.description.identifier.clone(), document),
        Err(rejected) => Contribution::Rejected(None, rejected),
    }
}

/// The registry: per identifier the shape loaded last, each replaced one noted with
/// `ShadowedDefinition` at info (the crate's own note; vanilla replaces silently). A file that
/// failed binds nothing.
fn combine(parsed: Vec<Contribution<VoxelShape>>, stack: &StackCx<'_>) -> Registry<VoxelShape> {
    let mut builder = RegistryBuilder::new(
        CombineRule::LastLoadedWins {
            replaced: Some(Notice::shadowed()),
        },
        CombineRule::top_wins(),
        stack.options,
    );
    for contribution in parsed {
        builder.push(contribution);
    }
    builder.finish()
}

/// The loader of `bp.voxel_shapes()`: `shapes/**/*.json` of every behaviour pack, the shape
/// loaded last winning per identifier.
pub const fn load_registry() -> CollectionLoader<Registry<VoxelShape>, Contribution<VoxelShape>> {
    CollectionLoader {
        walk: WALK,
        parse: parse_walk_file,
        combine,
        needs: &[],
        then: None,
    }
}
