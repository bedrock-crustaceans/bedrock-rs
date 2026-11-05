//! Loading trade tables: the family's version-chain parameters, `Content`, the walk of
//! `trading/` and the combine rule (the top pack's file at a path wins).

use super::history::{Any, TradeFamily, v0};
use super::parse::CEREAL;
use super::{TradeTable, validate};
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, JsonPath};
use bedrock_addon_core::document::{
    Content, Document, Files, FilesBuilder, FilesRule, JsonContent, Origin, Rejected, VersionInfo,
};
use bedrock_addon_core::json::{
    EncodeError, EnvelopeVersion, JsonSlice, MAX_JSON_DEPTH, NullRule, Value, WriteStyle,
    check_depth, probe_envelope, strip_bom,
};
use bedrock_addon_core::load::{
    Accept, AppDataPlace, CollectionLoader, ListDepth, LoadContext, StackCx, Walk, WalkFile,
    WalkOrder,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput, PickedVersion};
use bedrock_addon_core::versioning::{
    EnvelopeCheck, ErrorMask, Family, Identity, InputKind, Loaded, NoComponents, ParseCx, Parsed,
    StructIndex, ValidateCx, VersionChain, finish, parse_as_written,
};

/// The pack directory of trade tables.
const DIRECTORY: &str = "trading";

impl Family for TradeFamily {
    const NAME: &'static str = "trade table";
    // The document is `{"format_version"?, "tiers": [...]}`; the game binds the table to the
    // `tiers` member alone, so the payload is the array.
    const PAYLOAD_KEY: &'static str = "tiers";
    const DIRECTORIES: &'static [&'static str] = &[DIRECTORY];
    // The legacy slice starts at 0.0.0.
    const FLOOR: FormatVersion = v0::VERSION;
    type Latest = TradeTable;
    type Beta = std::convert::Infallible;
    type Component = NoComponents;
    type Any = Any;
    // The legacy parser reads the whole document and ignores other root members; the strict
    // cereal slice (1.21.30 on) checks the envelope.
    const ENVELOPE: EnvelopeCheck = EnvelopeCheck::From(CEREAL);
    // A missing `format_version` is 0.0.0 with no log entry.
    const ENVELOPE_WARN_MISSING_VERSION: bool = false;
    // The game reads a table as a whole document: the version is the one its JSON reader picks
    // from the text, the size limits apply, and rule N2 holds at the cereal slice.
    const INPUT: InputKind = InputKind::Document;

    /// The legacy slice `[0.0.0, 1.21.30)` is a hand-written parser: its `null` handling is the
    /// family's own, so no rule.
    /// The cereal slice takes rule N2.
    fn null_rule(index: StructIndex) -> Option<NullRule> {
        match index {
            StructIndex::Stable(0) => None,
            _ => Some(NullRule::Rejected),
        }
    }

    /// The game reads the cereal slice strictly, so an unknown member is an error there; the
    /// legacy slice is a hand parser with no cereal mask.
    fn error_mask(index: StructIndex) -> ErrorMask {
        match index {
            StructIndex::Stable(0) => ErrorMask::Unstrict,
            _ => ErrorMask::Strict,
        }
    }

    fn identity(_: JsonSlice<'_>) -> Identity {
        Identity::default()
    }

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(table) = v {
            validate::validate(table, cx);
        }
    }
}

impl bedrock_addon_core::__private::Sealed for TradeTable {}

impl Content for TradeTable {
    const NAME: &'static str = "trade table";
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        parse_file(PackRef::app_data(), PackSide::Behavior, path, bytes, cx)
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        self.write_into(out, style)
    }
}

impl JsonContent for TradeTable {}

impl TradeTable {
    /// Parses one trade-table file without upgrading it: the struct its `format_version`
    /// selects.
    ///
    /// # Errors
    ///
    /// Returns [`Rejected`] when the file cannot be read into a value at all, such as bytes that
    /// are not JSON or a document the family refuses; it carries every finding about the file.
    pub fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> Result<Document<Any>, Rejected> {
        let origin = origin(PackRef::app_data(), PackSide::Behavior, path);
        let parsed = match parse_stage(bytes, cx) {
            Ok(parsed) => parsed,
            Err(finding) => return Err(reject(origin, *finding)),
        };
        let mut diagnostics = parsed.diagnostics;
        locate(&mut diagnostics, &origin);
        match parsed.value {
            Some(value) => Ok(Document::new(value, origin, parsed.version, diagnostics)),
            None => Err(Rejected::new(origin, diagnostics)),
        }
    }
}

/// `history::upgrade`: rebuilds the parse stage from the document and finishes it.
pub(crate) fn upgrade_document(
    document: Document<Any>,
    cx: &LoadContext,
) -> Result<Document<TradeTable>, Rejected> {
    let origin = document.origin().clone();
    let version = document.version().clone();
    let diagnostics = document.diagnostics().clone();
    let value = document.into_value();
    let at = match value {
        Any::V0(_) => JsonPath::new(),
        Any::V1_21_30(_) => payload_at(),
    };
    let written = FormatVersionInput {
        version: Some(version.effective),
        beta_literal: false,
        prerelease: false,
    };
    let parsed = Parsed::<TradeFamily>::new(
        Some(value),
        version,
        written,
        Identity::default(),
        ActorType::Mob,
        false,
        at,
        diagnostics,
    );
    finished(parsed, origin, cx)
}

/// The path of the payload in a checked envelope.
fn payload_at() -> JsonPath {
    let mut at = JsonPath::new();
    at.push(<TradeFamily as Family>::PAYLOAD_KEY);
    at
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
    Diagnostic::error(DiagnosticCode::MalformedInput {
        message: "not UTF-8 JSON within the nesting limit".to_owned(),
        text: None,
    })
}

/// A rejection with one finding.
fn reject(origin: Origin, finding: Diagnostic) -> Rejected {
    let mut diagnostics = Diagnostics::default();
    diagnostics.push(finding);
    Rejected::located(origin, diagnostics)
}

/// The origin of a file of `pack`.
fn origin(pack: PackRef, side: PackSide, path: &PackPath) -> Origin {
    Origin::file(pack, side, path.clone())
}

/// Gives every finding the document's pack and file.
fn locate(diagnostics: &mut Diagnostics, origin: &Origin) {
    // In place: the list keeps where the members of the upgraded table are in the file.
    for finding in diagnostics.iter_mut() {
        finding.location.pack = Some(origin.pack.clone());
        finding.location.path = Some(origin.path.clone());
    }
}

/// The parse stage of one file: the struct its version selects, not upgraded. `Err` is a
/// finding that rejects the file before any struct is chosen.
///
/// Two cases the generic stage does not express go to the legacy parser here: a document
/// that is JSON `null` (the parser's own "must containt" text), and a `format_version` that is
/// no string (a number or an array) whose version, as the game's JSON reader picks it from the
/// text, selects the legacy slice: the legacy parser then reads its own version only from a
/// string and takes 0.0.0 without a message. A non-string that gives no version (`null`, a
/// bool, an object) is "Malformed format version" and one that selects the cereal slice is
/// read there, both by the generic stage (the document input). A pre-release of
/// 1.21.30 sorts below 1.21.30 and reaches the legacy slice through the generic stage.
fn parse_stage(bytes: &[u8], cx: &LoadContext) -> Result<Parsed<TradeFamily>, Box<Diagnostic>> {
    let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
    let text = text(bytes).ok_or_else(|| Box::new(malformed()))?;
    let document = JsonSlice::new(text);
    match probe_envelope(document) {
        Err(_) if is_null(text) => Ok(legacy(document, cx)),
        Ok(envelope)
            if matches!(
                envelope.format_version,
                EnvelopeVersion::Malformed | EnvelopeVersion::Array(_)
            ) && picks_legacy(text) =>
        {
            Ok(legacy(document, cx))
        }
        _ => Ok(parse_as_written::<TradeFamily>(document, cx)),
    }
}

/// Whether the version the game's JSON reader picks from `text` selects the legacy slice (below
/// 1.21.30).
fn picks_legacy(text: &str) -> bool {
    match FormatVersionInput::pick(text) {
        PickedVersion::Version { input, .. } => input.version.is_some_and(|v| v < CEREAL),
        PickedVersion::Absent | PickedVersion::Malformed => false,
    }
}

/// Whether the text is the JSON value `null`.
fn is_null(text: &str) -> bool {
    bedrock_addon_core::json::from_str_jsonc::<Value>(text).is_ok_and(|v| v.is_null())
}

/// The legacy parse of the whole document at 0.0.0, for a `format_version` that is no string
/// (as if absent).
fn legacy(document: JsonSlice<'_>, cx: &LoadContext) -> Parsed<TradeFamily> {
    let effective = v0::VERSION;
    let written = FormatVersionInput {
        version: Some(effective),
        beta_literal: false,
        prerelease: false,
    };
    let parse = ParseCx {
        load: cx,
        written,
        original_version: effective,
        at: JsonPath::new(),
    };
    let mut diagnostics = Diagnostics::default();
    let value = <Any as VersionChain<TradeFamily>>::parse_struct(
        document,
        StructIndex::Stable(0),
        &parse,
        &mut diagnostics,
    )
    .filter(|_| !diagnostics.has_errors());
    Parsed::new(
        value,
        VersionInfo::new(None, false, effective, v0::VERSION, false),
        written,
        Identity::default(),
        ActorType::Mob,
        false,
        JsonPath::new(),
        diagnostics,
    )
}

/// The rest of the load of a parsed file: the upgrade, the checks, the document.
fn finished(
    parsed: Parsed<TradeFamily>,
    origin: Origin,
    cx: &LoadContext,
) -> Result<Document<TradeTable>, Rejected> {
    let outcome = finish(parsed, cx);
    let mut diagnostics = outcome.diagnostics;
    locate(&mut diagnostics, &origin);
    match outcome.value {
        Some(Loaded::Stable(table)) => {
            Ok(Document::new(table, origin, outcome.version, diagnostics))
        }
        _ => Err(Rejected::new(origin, diagnostics)),
    }
}

/// Loads one trade-table file of `pack`.
pub(crate) fn parse_file(
    pack: PackRef,
    side: PackSide,
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> Result<Document<TradeTable>, Rejected> {
    let origin = origin(pack, side, path);
    match parse_stage(bytes, cx) {
        Ok(parsed) => finished(parsed, origin, cx),
        Err(finding) => Err(reject(origin, *finding)),
    }
}

// ---- the collection ----------------------------------------------------------------------

/// A trade-table file: a `.json` file below `trading/`.
fn is_json(path: &PackPath) -> bool {
    path.as_str()
        .rsplit_once('.')
        .is_some_and(|(_, extension)| extension == "json")
}

/// `trading/**/*.json` of every pack, bottom-up.
const WALK: Walk = Walk {
    directories: &[DIRECTORY],
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

/// One copy of a path: the key it is filed under and the document or its rejection.
pub type FileCopy = (PackPath, Result<Document<TradeTable>, Rejected>);

/// One file's copy.
fn parse_walk_file(file: &WalkFile, cx: &LoadContext) -> FileCopy {
    let copy = parse_file(file.pack.clone(), file.side, &file.path, &file.bytes, cx);
    (file.key.clone(), copy)
}

/// The files, the top pack's copy of a path winning; a failing top copy leaves the path
/// without a file (vanilla looks a table up once, by path).
fn combine(parsed: Vec<FileCopy>, stack: &StackCx<'_>) -> Files<TradeTable> {
    let mut builder = FilesBuilder::new(FilesRule::TopWins, stack.options);
    for (key, copy) in parsed {
        builder.push(key, copy);
    }
    builder.finish()
}

/// The loader of `bp.trade_tables()`: `trading/**/*.json` of every behaviour pack, keyed by
/// path, the top pack's copy winning.
pub const fn load_files() -> CollectionLoader<Files<TradeTable>, FileCopy> {
    CollectionLoader {
        walk: WALK,
        parse: parse_walk_file,
        combine,
        needs: &[],
        then: None,
    }
}
