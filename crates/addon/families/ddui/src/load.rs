//! Loading data-driven UI: the three families' chain parameters, the envelope the client's
//! loader insists on, `Content` / `Definition` / `Placed`, the walk of `ddui/` and the group's
//! combine rule.

use std::collections::HashSet;
use std::convert::Infallible;

use bedrock_addon_core::load::{
    AppDataPlace, CollectionLoader, ListDepth, LoadContext, StackCx, Walk, WalkFile, WalkOrder,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};

use super::history::{
    CompositionAny, CompositionFamily, RootAny, RootFamily, StyleAny, StyleFamily,
};
use super::{
    DataDrivenUi, FORMAT_VERSION, UiComposition, UiRoot, UiStyle, read, style, validate, write,
};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticDetail, Diagnostics, JsonPath, Location, Phase, Severity,
    Span, VanillaText,
};
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, NoBeta, Notice, Origin, Placed,
    RegistryBuilder, Rejected, VersionInfo, default_path_flat_for_vanilla,
};
use bedrock_addon_core::id::{Id, kind};
use bedrock_addon_core::json::{
    EncodeError, JsonSlice, MAX_JSON_DEPTH, Value, WriteStyle, check_depth, last_wins,
    probe_envelope, strip_bom, write_json,
};
use bedrock_addon_core::load::Reported;
use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput, PickedVersion};
use bedrock_addon_core::versioning::{
    BETA_VERSION, BetaPermission, ComponentTable, ErrorMask, Family, Identity, InputKind, Loaded,
    MissingVersion, NoComponents, NoStable, ParseCx, Parsed, RangeModel, StructIndex, ValidateCx,
    VersionStruct, Versioned, finish, no_parser_version, parse_as_written,
};

/// `ddui/root`: `minecraft:ui-root` documents.
pub(crate) const ROOTS: &str = "ddui/root";
/// `ddui/composition`: `minecraft:ui-composition` documents.
pub(crate) const COMPOSITIONS: &str = "ddui/composition";
/// `ddui/styling`: `minecraft:ui-style` documents.
pub(crate) const STYLES: &str = "ddui/styling";

/// The client's text for a file whose bytes could not be read; `{}` is the file.
const UNREAD: &str = "Failed to load the contents of DataDrivenUI file {}; it will be skipped.";

/// The loader's text for a file whose payload member is not the directory's.
const MISSING_OBJECT: &str = "Failed to parse DataDrivenUI file. Missing object \"{}\"";
/// The loader's closing line of a file the typed load or a validator failed.
const FAILED_TO_LOAD: &str = "Failed to load DataDrivenUI file";
/// The text for a `format_version` string that is no version; the document input logs it
/// itself, not the loader.
const MALFORMED_VERSION: &str = "Malformed format version";
/// The client's text for an extension point no loaded root declares.
const NOT_DEFINED: &str = "extension_point '{}' is not defined.";

/// A family of this module: one struct, stable (roots, compositions) or beta (styles).
pub(crate) trait UiFamily: Family {
    /// The struct a loaded document holds.
    type Doc;

    /// The struct of a loaded value: the one this family has.
    fn take(loaded: Loaded<Self>) -> Option<Self::Doc>;

    /// The extension point a parsed document asks for (a composition's
    /// `attribs.extension_point`), whether or not the document then passes validation.
    fn extension_point(_: &Self::Any) -> Option<String> {
        None
    }

    /// The whole document of `value`, `format_version` included.
    fn document(value: &Self::Doc) -> Result<Value, EncodeError>;
}

impl Family for RootFamily {
    const NAME: &'static str = "data-driven UI root";
    const PAYLOAD_KEY: &'static str = "minecraft:ui-root";
    const DIRECTORIES: &'static [&'static str] = &[ROOTS];
    // The loader reads exactly one version, so the range is closed at both ends:
    // [1.21.130, 1.21.130].
    const FLOOR: FormatVersion = FORMAT_VERSION;
    const RANGE_MODEL: RangeModel = RangeModel::ClosedToLatest;
    type Latest = UiRoot;
    type Beta = Infallible;
    type Component = NoComponents;
    type Any = RootAny;
    // The loader never reaches a document without a version; `stage` reports it with the
    // loader's own text.
    const MISSING_VERSION: MissingVersion = MissingVersion::Reject;
    // The loader reads a roots file through the document input: it takes the version as that
    // input reads it, and rule N2 holds.
    const INPUT: InputKind = InputKind::Document;

    fn components() -> &'static ComponentTable<Self> {
        &ComponentTable::EMPTY
    }

    // The one version is read under the strict error mask: unknown members and type errors
    // refuse the file.
    fn error_mask(_: StructIndex) -> ErrorMask {
        ErrorMask::Strict
    }

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(root) = v {
            validate::namespace(root.description.identifier.as_str(), cx);
        }
    }
}

impl UiFamily for RootFamily {
    type Doc = UiRoot;

    fn take(loaded: Loaded<Self>) -> Option<UiRoot> {
        match loaded {
            Loaded::Stable(root) => Some(root),
            Loaded::Beta(never) => match never {},
        }
    }

    fn document(value: &UiRoot) -> Result<Value, EncodeError> {
        write::root_document(value)
    }
}

impl Family for CompositionFamily {
    const NAME: &'static str = "data-driven UI composition";
    const PAYLOAD_KEY: &'static str = "minecraft:ui-composition";
    const DIRECTORIES: &'static [&'static str] = &[COMPOSITIONS];
    // Exactly one version is read, as for roots.
    const FLOOR: FormatVersion = FORMAT_VERSION;
    const RANGE_MODEL: RangeModel = RangeModel::ClosedToLatest;
    type Latest = UiComposition;
    type Beta = Infallible;
    type Component = NoComponents;
    type Any = CompositionAny;
    const MISSING_VERSION: MissingVersion = MissingVersion::Reject;
    // The same document input as for roots: the version as it reads it, and rule N2.
    const INPUT: InputKind = InputKind::Document;

    fn components() -> &'static ComponentTable<Self> {
        &ComponentTable::EMPTY
    }

    // The strict error mask, as for roots.
    fn error_mask(_: StructIndex) -> ErrorMask {
        ErrorMask::Strict
    }

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(composition) = v {
            // The namespace rule does not stop the validation: the extension-point rules run
            // after it either way.
            validate::namespace(composition.description.identifier.as_str(), cx);
            validate::external_extension_point(composition, cx);
        }
    }
}

impl UiFamily for CompositionFamily {
    type Doc = UiComposition;

    fn take(loaded: Loaded<Self>) -> Option<UiComposition> {
        match loaded {
            Loaded::Stable(composition) => Some(composition),
            Loaded::Beta(never) => match never {},
        }
    }

    fn extension_point(any: &CompositionAny) -> Option<String> {
        let CompositionAny::V1_21_130(composition) = any;
        composition.attribs.extension_point.clone()
    }

    fn document(value: &UiComposition) -> Result<Value, EncodeError> {
        write::composition_document(value)
    }
}

impl Family for StyleFamily {
    const NAME: &'static str = "data-driven UI style";
    const PAYLOAD_KEY: &'static str = "minecraft:ui-style";
    const DIRECTORIES: &'static [&'static str] = &[STYLES];
    // Unused: there is no stable struct, so every version but `"beta"` has no parser.
    const FLOOR: FormatVersion = FormatVersion::new(0, 0, 0);
    type Latest = NoStable;
    type Beta = UiStyle;
    // Every pack may load the beta version, with or without the Beta APIs toggle.
    const BETA_PERMISSION: BetaPermission = BetaPermission::Always;
    type Component = NoComponents;
    type Any = StyleAny;
    const MISSING_VERSION: MissingVersion = MissingVersion::Reject;
    // The same document input as for roots: the version as it reads it, and rule N2.
    const INPUT: InputKind = InputKind::Document;

    fn components() -> &'static ComponentTable<Self> {
        &ComponentTable::EMPTY
    }

    // The strict error mask, as for roots.
    fn error_mask(_: StructIndex) -> ErrorMask {
        ErrorMask::Strict
    }

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        // Styles follow the namespace rule of roots and compositions, with the same two texts.
        if let Loaded::Beta(style) = v {
            validate::namespace(style.description.identifier.as_str(), cx);
        }
    }
}

impl UiFamily for StyleFamily {
    type Doc = UiStyle;

    fn take(loaded: Loaded<Self>) -> Option<UiStyle> {
        match loaded {
            Loaded::Beta(style) => Some(style),
            Loaded::Stable(_) => None,
        }
    }

    fn document(value: &UiStyle) -> Result<Value, EncodeError> {
        let mut root = facet_value::VObject::new();
        root.insert("format_version", "beta");
        root.insert(StyleFamily::PAYLOAD_KEY, style::payload(value)?);
        Ok(Value::from(root))
    }
}

impl Versioned for UiStyle {
    const VERSION: FormatVersion = BETA_VERSION;
}

/// A style has no Molang.
impl MolangFields for UiStyle {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

impl VersionStruct for UiStyle {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        read::style(payload, &cx.at, d)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        style::payload(self)
    }
}

impl Versioned for UiRoot {
    const VERSION: FormatVersion = FORMAT_VERSION;
}

impl Versioned for UiComposition {
    const VERSION: FormatVersion = FORMAT_VERSION;
}

impl VersionStruct for UiRoot {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        read::root(payload, cx.load.engine_version, &cx.at, d)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        write::root_payload(self)
    }
}

impl VersionStruct for UiComposition {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        read::composition(payload, cx.load.engine_version, &cx.at, d)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        write::composition_payload(self)
    }
}

// ---- one file ------------------------------------------------------------------------------

/// The text of a file, after the BOM and the depth check; `None` when it is not UTF-8 or too
/// deep.
fn text(bytes: &[u8]) -> Option<&str> {
    let bytes = strip_bom(bytes);
    check_depth(bytes, MAX_JSON_DEPTH).ok()?;
    std::str::from_utf8(bytes).ok()
}

/// A finding of the envelope stage.
fn finding(
    code: DiagnosticCode,
    severity: Severity,
    at: JsonPath,
    span: Option<Span>,
) -> Diagnostic {
    Diagnostic::new(
        code,
        severity,
        Phase::Parse,
        Location::new(None, None, at, span),
    )
}

/// The path of the envelope's `format_version`.
fn version_at() -> JsonPath {
    let mut at = JsonPath::new();
    at.push("format_version");
    at
}

/// The loader's "Missing object" error for family `F`.
fn missing_object<F: Family>() -> Diagnostic {
    finding(
        DiagnosticCode::MissingPayload {
            text: Some(VanillaText::Formatted {
                template: MISSING_OBJECT,
                args: vec![F::PAYLOAD_KEY.to_owned()],
            }),
        },
        Severity::Error,
        JsonPath::new(),
        None,
    )
}

/// The loader's closing line.
fn failed_to_load() -> Diagnostic {
    finding(
        DiagnosticCode::DocumentRejected {
            text: Some(VanillaText::Fixed(FAILED_TO_LOAD)),
        },
        Severity::Error,
        JsonPath::new(),
        None,
    )
}

/// A document the envelope stage refused, with its findings.
fn refused<F: Family>(diagnostics: Diagnostics) -> Parsed<F> {
    Parsed::new(
        None,
        VersionInfo::new(None, false, F::FLOOR, F::FLOOR, false),
        FormatVersionInput {
            version: Some(F::FLOOR),
            beta_literal: false,
            prerelease: false,
        },
        Identity::default(),
        ActorType::Mob,
        false,
        JsonPath::new(),
        diagnostics,
    )
}

/// One file read up to its struct: the parse stage, whether the typed load was reached (its
/// failure gets the loader's closing line) and the extension point the struct asks for.
struct Stage<F: UiFamily> {
    parsed: Parsed<F>,
    typed: bool,
    extension_point: Option<String>,
}

/// The envelope as the client's loader reads it, then the struct through the chain.
///
/// What the envelope reader refuses leaves the payload name empty, so the loader's one text is
/// "Missing object": JSON that does not parse, a root without exactly two members (a
/// missing `format_version` among them) and a `format_version` that gives no version. The
/// reader's own text is kept beside it as an info finding (the client writes it to a logger
/// it never prints). The version is the first textual `"format_version"` of the file, a
/// number read as its text. Two members without a `"format_version"` reach the typed load,
/// which has no version to select a parser by.
fn stage<F: UiFamily>(document: JsonSlice<'_>, cx: &LoadContext) -> Stage<F> {
    let mut diagnostics = Diagnostics::default();
    let Ok(envelope) = probe_envelope(document) else {
        // Core's reading of a text that is no object: an empty document or a syntax error,
        // each demoted to the note it is in this family.
        let parsed = parse_as_written::<F>(document, cx);
        for mut note in parsed.diagnostics {
            if note.severity == Severity::Error {
                note.severity = Severity::Info;
            }
            diagnostics.push(note);
        }
        diagnostics.push(missing_object::<F>());
        return Stage {
            parsed: refused(diagnostics),
            typed: false,
            extension_point: None,
        };
    };
    // The version is the first textual `"format_version"` of the file, a number read as its
    // text.
    let picked = FormatVersionInput::pick(document.text);
    let note = if envelope.member_count() != 2 {
        Some(finding(
            DiagnosticCode::MalformedInput {
                message: format!(
                    "{} root members, not \"format_version\" and a payload member",
                    envelope.member_count()
                ),
                text: Some(VanillaText::DocumentMemberCount),
            },
            Severity::Info,
            JsonPath::new(),
            None,
        ))
    } else if picked == PickedVersion::Malformed {
        Some(finding(
            DiagnosticCode::MalformedFormatVersion {
                text: Some(VanillaText::Fixed(MALFORMED_VERSION)),
            },
            Severity::Info,
            version_at(),
            None,
        ))
    } else {
        None
    };
    let missing = note.is_some() || envelope.payload(F::PAYLOAD_KEY).is_none();
    let versioned = matches!(picked, PickedVersion::Version { .. });
    if missing || !versioned {
        // This stage is the reader of the root of a file it refuses itself; the chain's
        // parse stage reports the same for every other file.
        diagnostics.extend(bedrock_addon_core::json::trailing_content_finding(document));
    }
    if missing {
        diagnostics.extend(note);
        diagnostics.push(missing_object::<F>());
        return Stage {
            parsed: refused(diagnostics),
            typed: false,
            extension_point: None,
        };
    }
    let PickedVersion::Version { input, written } = picked else {
        // No version in the input and none from the caller.
        diagnostics.push(finding(
            DiagnosticCode::UnsupportedVersion {
                text: Some(VanillaText::NoValidVersion),
            },
            Severity::Error,
            version_at(),
            None,
        ));
        return Stage {
            parsed: refused(diagnostics),
            typed: true,
            extension_point: None,
        };
    };
    let mut parsed = parse_as_written::<F>(document, cx);
    if parsed.value.is_none() {
        no_parser(
            &mut parsed.diagnostics,
            &no_parser_version(&input, &written),
        );
    }
    let extension_point = parsed.value.as_ref().and_then(F::extension_point);
    Stage {
        parsed,
        typed: true,
        extension_point,
    }
}

/// The loader has one closed range and no beta version: every version outside it, the beta
/// literal included, is "no parser available for version …", where the generic checks say "To
/// use …, use json format version … or higher" or ask for the Beta APIs toggle.
fn no_parser(diagnostics: &mut Diagnostics, version: &str) {
    let found = std::mem::take(diagnostics);
    for mut finding in found {
        if matches!(
            finding.code,
            DiagnosticCode::UnsupportedVersion {
                text: Some(VanillaText::UseFormatVersion { .. })
            } | DiagnosticCode::BetaFeaturesNotEnabled { .. }
        ) {
            finding.code = DiagnosticCode::UnsupportedVersion {
                text: Some(VanillaText::NoParserForVersion {
                    version: version.to_owned(),
                }),
            };
        }
        diagnostics.push(finding);
    }
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

/// Gives every finding that names a member and no place in the file the place of that member
/// (the validators see the value, not the text). A file has a handful of them at most.
fn place(diagnostics: &mut Diagnostics, document: JsonSlice<'_>) {
    let found = std::mem::take(diagnostics);
    for mut finding in found {
        if finding.location.span.is_none() && !finding.location.json_path.is_empty() {
            finding.location.span = read::span_at(document, &finding.location.json_path);
        }
        diagnostics.push(finding);
    }
}

/// The rejection of a file that is no text the reader takes: not UTF-8, or nested too deep.
fn unreadable<F: UiFamily>() -> Diagnostics {
    let message = "not UTF-8 JSON within the nesting limit".to_owned();
    let mut diagnostics = Diagnostics::default();
    diagnostics.push(finding(
        DiagnosticCode::MalformedInput {
            message: message.clone(),
            text: Some(VanillaText::DocumentJson { message }),
        },
        Severity::Info,
        JsonPath::new(),
        None,
    ));
    diagnostics.push(missing_object::<F>());
    diagnostics
}

/// What loading one file gives.
#[doc(hidden)]
#[derive(Debug)]
pub struct FileOutcome<T> {
    /// The identifier the payload names, when it is a string.
    pub(super) identifier: Option<String>,
    /// The document or its rejection.
    pub(super) result: Result<Document<T>, Rejected>,
    /// The extension point the struct asks for and where the file writes it, whether the
    /// document loaded or a validator refused it; `None` when the struct was not read.
    pub(super) extension_point: Option<(String, Option<Span>)>,
}

/// Loads one file of family `F`.
fn parse_file<F: UiFamily>(
    pack: PackRef,
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> FileOutcome<F::Doc> {
    let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
    let origin = Origin::new(pack, PackSide::Resource, path.clone(), None, Vec::new());
    let Some(text) = text(bytes) else {
        let mut diagnostics = unreadable::<F>();
        locate(&mut diagnostics, &origin);
        return FileOutcome {
            identifier: None,
            result: Err(Rejected::new(origin, diagnostics)),
            extension_point: None,
        };
    };
    // Of a member written twice the last value stands, before any look at the text.
    let cleaned = last_wins(text);
    let document = JsonSlice::new(&cleaned);
    let Stage {
        parsed,
        typed,
        extension_point,
    } = stage::<F>(document, cx);
    let identifier = parsed.identity.identifier.clone();
    let extension_point = extension_point.map(|name| {
        let mut at = JsonPath::new();
        for member in [F::PAYLOAD_KEY, "attribs", "extension_point"] {
            at.push(member);
        }
        (name, read::span_at(document, &at))
    });
    let outcome = finish(parsed, cx);
    let mut diagnostics = outcome.diagnostics;
    place(&mut diagnostics, document);
    let mut version = outcome.version;
    if matches!(outcome.value, Some(Loaded::Beta(_))) {
        version.parsed_as = BETA_VERSION;
    }
    let result = if let Some(value) = outcome.value.and_then(F::take) {
        locate(&mut diagnostics, &origin);
        Ok(Document::new(value, origin, version, diagnostics))
    } else {
        if typed {
            diagnostics.push(failed_to_load());
        }
        locate(&mut diagnostics, &origin);
        Err(Rejected::new(origin, diagnostics))
    };
    FileOutcome {
        identifier,
        result,
        extension_point,
    }
}

/// Parses a file of family `F` without upgrading it: the struct the file's `format_version`
/// selects.
fn as_written<F: UiFamily>(
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> Result<Document<F::Any>, Rejected> {
    let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
    let origin = Origin::new(
        PackRef::app_data(),
        PackSide::Resource,
        path.clone(),
        None,
        Vec::new(),
    );
    let Some(text) = text(bytes) else {
        let mut diagnostics = unreadable::<F>();
        locate(&mut diagnostics, &origin);
        return Err(Rejected::new(origin, diagnostics));
    };
    let cleaned = last_wins(text);
    let document = JsonSlice::new(&cleaned);
    let Stage { parsed, typed, .. } = stage::<F>(document, cx);
    let mut diagnostics = parsed.diagnostics;
    place(&mut diagnostics, document);
    if let Some(value) = parsed.value {
        locate(&mut diagnostics, &origin);
        Ok(Document::new(value, origin, parsed.version, diagnostics))
    } else {
        if typed {
            diagnostics.push(failed_to_load());
        }
        locate(&mut diagnostics, &origin);
        Err(Rejected::new(origin, diagnostics))
    }
}

/// The `Content` impl of a root type and its `parse_as_written`.
macro_rules! content {
    ($root:ty, $family:ty, $any:ty, name = $name:literal) => {
        impl bedrock_addon_core::__private::Sealed for $root {}

        impl Content for $root {
            const NAME: &'static str = $name;
            const SIDES: &'static [PackSide] = &[PackSide::Resource];

            /// One file outside a stack: everything the client checks in the file itself.
            /// Whether a composition's `extension_point` is declared by a root is known only
            /// to a stack (`PackStack::data_driven_ui`).
            fn parse(
                path: &PackPath,
                bytes: &[u8],
                cx: &LoadContext,
            ) -> Result<Document<Self>, Rejected> {
                parse_file::<$family>(PackRef::app_data(), path, bytes, cx).result
            }

            fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
                write_json(&<$family as UiFamily>::document(self)?, out, style)
            }

            fn to_json(&self) -> Result<String, EncodeError> {
                let mut out = Vec::new();
                self.write_json(&mut out, WriteStyle::default())?;
                Ok(String::from_utf8(out).unwrap_or_default())
            }
        }

        impl $root {
            /// Parses one file without upgrading it: the struct its `format_version` selects.
            /// There is one struct, so the value is the newest already.
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
    };
}

content!(UiRoot, RootFamily, RootAny, name = "data-driven UI root");
content!(
    UiComposition,
    CompositionFamily,
    CompositionAny,
    name = "data-driven UI composition"
);
content!(
    UiStyle,
    StyleFamily,
    StyleAny,
    name = "data-driven UI style"
);

impl Definition for UiRoot {
    type Kind = kind::UiRoot;
    type File = UiRoot;
    type Beta = NoBeta<kind::UiRoot>;
}

impl Definition for UiComposition {
    type Kind = kind::UiComposition;
    type File = UiComposition;
    type Beta = NoBeta<kind::UiComposition>;
}

impl Definition for UiStyle {
    type Kind = bedrock_addon_core::id::kind::UiStyle;
    type File = UiStyle;
    type Beta = NoBeta<bedrock_addon_core::id::kind::UiStyle>;
}

impl Placed for UiStyle {
    /// `ddui/styling/<name>.json` for a `minecraft:` identifier (the `/` of the name as `_`),
    /// `ddui/styling/<namespace>/<name>.json` for any other, as roots.
    fn default_path(&self) -> PackPath {
        default_path_flat_for_vanilla(STYLES, &self.description.identifier)
    }
}

impl Placed for UiRoot {
    /// `ddui/root/<name>.json` for a `minecraft:` identifier (the `/` of the name as `_`),
    /// which a shipped pack's flat walk reads back; `ddui/root/<namespace>/<name>.json` for
    /// any other.
    fn default_path(&self) -> PackPath {
        default_path_flat_for_vanilla(ROOTS, &self.description.identifier)
    }
}

impl Placed for UiComposition {
    /// `ddui/composition/<name>.json` for a `minecraft:` identifier (the `/` of the name as
    /// `_`), which a shipped pack's flat walk reads back;
    /// `ddui/composition/<namespace>/<name>.json` for any other.
    fn default_path(&self) -> PackPath {
        default_path_flat_for_vanilla(COMPOSITIONS, &self.description.identifier)
    }
}

// ---- the group -----------------------------------------------------------------------------

/// What one walked file parses to.
#[derive(Debug)]
pub enum UiFile {
    /// A file of `ddui/root`.
    Root(Box<FileOutcome<UiRoot>>),
    /// A file of `ddui/composition`.
    Composition(Box<FileOutcome<UiComposition>>),
    /// A file of `ddui/styling`.
    Style(Box<FileOutcome<UiStyle>>),
}

/// A data-driven UI file: an extension exactly `json` (a case-sensitive compare).
fn is_json(path: &PackPath) -> bool {
    path.as_str()
        .rsplit_once('.')
        .is_some_and(|(_, extension)| extension == "json")
}

/// A file below a sub-folder of its directory (`ddui/root/sub/x.json`).
fn nested(key: &PackPath) -> bool {
    key.segments().count() > 3
}

/// Whether the walk reads `key`: a directory pack outside the app package is walked with its
/// sub-folders, a pack the game ships flat (as dialogue's walk).
fn reached(key: &PackPath, cx: &LoadContext) -> bool {
    !(cx.is_base_game_pack && nested(key))
}

/// `ddui/root`, then `ddui/composition`, then `ddui/styling`, each over the whole stack from
/// its first entry (the highest priority) to its last: a later load replaces an earlier one,
/// so the lowest pack that defines an identifier wins.
const WALK: Walk = Walk {
    directories: &[ROOTS, COMPOSITIONS, STYLES],
    depth: ListDepth::Recursive,
    accept: is_json,
    order: WalkOrder::TopDown,
    directory_major: true,
    app_data: &[],
    app_data_place: AppDataPlace::After,
    in_pack: Some(reached),
    shadowed: None,
    read_error: Some(UNREAD),
};

/// Parses one walked file by its directory.
fn parse_walk_file(walked: &WalkFile, cx: &LoadContext) -> UiFile {
    let (pack, path, bytes) = (walked.pack.clone(), &walked.path, &walked.bytes);
    match WALK.matches(&walked.key) {
        Some(0) => UiFile::Root(Box::new(parse_file::<RootFamily>(pack, path, bytes, cx))),
        Some(1) => UiFile::Composition(Box::new(parse_file::<CompositionFamily>(
            pack, path, bytes, cx,
        ))),
        _ => UiFile::Style(Box::new(parse_file::<StyleFamily>(pack, path, bytes, cx))),
    }
}

/// A registry in which the later load of an identifier replaces the earlier one whole and in
/// its place; the replaced one is a `ShadowedDefinition` info (the crate's own: the game
/// replaces silently).
fn last_loaded_wins<T: Definition>(stack: &StackCx<'_>) -> RegistryBuilder<T> {
    RegistryBuilder::new(
        CombineRule::LastLoadedWins {
            replaced: Some(Notice::shadowed()),
        },
        CombineRule::last_loaded_wins(),
        stack.options,
    )
}

/// The `IgnoredFile` infos of the files below `directory` the walk does not read for their
/// extension (the crate's own: the game writes a developer-log line), in stack order.
fn ignored_files<T: Definition>(
    directory: usize,
    stack: &StackCx<'_>,
    registry: &mut RegistryBuilder<T>,
) {
    let prefix = format!("{}/", WALK.directories[directory]);
    for pack in stack.packs {
        for file in pack.files() {
            let key = file.key.as_str();
            if !key.starts_with(&prefix) || is_json(&file.key) || !reached(&file.key, &pack.context)
            {
                continue;
            }
            registry.push_diagnostic(Diagnostic::new(
                DiagnosticCode::IgnoredFile { text: None },
                Severity::Info,
                Phase::Validate,
                Location::new(
                    Some(pack.pack.clone()),
                    Some(file.path.clone()),
                    JsonPath::new(),
                    None,
                ),
            ));
        }
    }
}

/// The error of a composition whose extension point no stored root declares.
fn not_defined(origin: &Origin, name: &str, span: Option<Span>) -> Diagnostic {
    let mut at = JsonPath::new();
    at.push(CompositionFamily::PAYLOAD_KEY);
    at.push("attribs");
    at.push("extension_point");
    let mut finding = Diagnostic::new(
        DiagnosticCode::UnresolvedReference {
            text: Some(VanillaText::Formatted {
                template: NOT_DEFINED,
                args: vec![name.to_owned()],
            }),
        },
        Severity::Error,
        Phase::Validate,
        Location::new(
            Some(origin.pack.clone()),
            Some(origin.path.clone()),
            at,
            span,
        ),
    );
    finding.detail = Some(Box::new(DiagnosticDetail::Reference {
        kind: "extension point",
        id: name.to_owned(),
        hint: None,
    }));
    finding
}

/// `findings` with `finding` before the loader's closing line, which it adds when there is
/// none: the validator's messages come before the closing line.
fn before_closing_line(findings: &mut Diagnostics, finding: Diagnostic, origin: &Origin) {
    let closing = |f: &Diagnostic| {
        matches!(
            &f.code,
            DiagnosticCode::DocumentRejected {
                text: Some(VanillaText::Fixed(FAILED_TO_LOAD))
            }
        )
    };
    let mut kept: Vec<Diagnostic> = std::mem::take(findings)
        .into_iter()
        .filter(|f| !closing(f))
        .collect();
    kept.push(finding);
    let mut last = failed_to_load();
    last.location.pack = Some(origin.pack.clone());
    last.location.path = Some(origin.path.clone());
    kept.push(last);
    for finding in kept {
        findings.push(finding);
    }
}

/// The group, from every parsed file in walk order: every root, then every composition, a
/// composition checked against the extension points of the stored roots before it is stored.
pub(crate) fn combine(parsed: Vec<UiFile>, stack: &StackCx<'_>) -> DataDrivenUi {
    let mut roots = last_loaded_wins::<UiRoot>(stack);
    let mut styles = last_loaded_wins::<UiStyle>(stack);
    let mut pending = Vec::new();
    for file in parsed {
        match file {
            UiFile::Style(outcome) => match outcome.result {
                Ok(document) => {
                    let id = document.description.identifier.clone();
                    styles.push(Contribution::Stable(id, document));
                }
                Err(rejected) => {
                    styles.push(Contribution::Rejected(
                        outcome.identifier.map(Id::parse),
                        rejected,
                    ));
                }
            },
            UiFile::Root(outcome) => match outcome.result {
                Ok(document) => {
                    let id = document.description.identifier.clone();
                    roots.push(Contribution::Stable(id, document));
                }
                Err(rejected) => {
                    roots.push(Contribution::Rejected(
                        outcome.identifier.map(Id::parse),
                        rejected,
                    ));
                }
            },
            UiFile::Composition(outcome) => pending.push(*outcome),
        }
    }
    ignored_files(0, stack, &mut roots);
    let roots = roots.finish();
    // Every name any stored root declares. One set for the whole
    // stack, so a composition is one lookup however many roots there are.
    let declared: HashSet<&str> = roots
        .iter()
        .flat_map(|(_, root)| root.extension_points())
        .collect();
    let mut compositions = last_loaded_wins::<UiComposition>(stack);
    for outcome in pending {
        let FileOutcome {
            identifier,
            result,
            extension_point,
        } = outcome;
        let undeclared = extension_point.filter(|(name, _)| !declared.contains(name.as_str()));
        match (result, undeclared) {
            (Ok(document), None) => {
                let id = document.description.identifier.clone();
                compositions.push(Contribution::Stable(id, document));
            }
            (Ok(document), Some((name, span))) => {
                let origin = document.origin().clone();
                let mut diagnostics = document.diagnostics().clone();
                let finding = not_defined(&origin, &name, span);
                before_closing_line(&mut diagnostics, finding, &origin);
                let id = Some(document.description.identifier.clone());
                compositions.push(Contribution::Rejected(
                    id,
                    Rejected::new(origin, diagnostics),
                ));
            }
            (Err(mut rejected), undeclared) => {
                if let Some((name, span)) = undeclared {
                    let origin = rejected.origin.clone();
                    let finding = not_defined(&origin, &name, span);
                    before_closing_line(&mut rejected.diagnostics, finding, &origin);
                }
                compositions.push(Contribution::Rejected(identifier.map(Id::parse), rejected));
            }
        }
    }
    ignored_files(1, stack, &mut compositions);
    ignored_files(2, stack, &mut styles);
    DataDrivenUi {
        roots,
        compositions: compositions.finish(),
        styles: styles.finish(),
    }
}

/// The loader of `rp.data_driven_ui()`: the three directories of every resource pack, combined
/// into the group.
pub const fn load_group() -> CollectionLoader<DataDrivenUi, UiFile> {
    CollectionLoader {
        walk: WALK,
        parse: parse_walk_file,
        combine,
        needs: &[],
        then: None,
    }
}

impl Reported for DataDrivenUi {
    fn reported(&self) -> Box<dyn Iterator<Item = &Diagnostic> + '_> {
        Box::new(self.diagnostics())
    }
}
