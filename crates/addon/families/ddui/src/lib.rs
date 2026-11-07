//! Data-driven UI: the three document kinds below `ddui/` of a resource pack — roots
//! (`ddui/root/`, `minecraft:ui-root`), compositions (`ddui/composition/`,
//! `minecraft:ui-composition`) and form styles (`ddui/styling/`, `minecraft:ui-style`) — loaded
//! together as the `DataDrivenUi` group. One struct per kind: roots and compositions at
//! `format_version` 1.21.130 exactly, styles at `"beta"` only; the control catalogue is keyed by
//! the engine version.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod codes;
pub mod history;
pub use catalogue::{Binding, Catalogue, ChildSet, ControlRule};
pub use component::Component;
pub use style::{
    BorderImageRepeat, BoxSize, FormLabelStyle, FormPanelStyle, FormTextFieldStyle, StyleColor,
    StyleComponents, StyleDescription, UiStyle, UiStyleId,
};

mod catalogue;
mod component;
mod read;
mod style;
mod types;
mod validate;
mod write;

#[cfg(test)]
mod tests;

pub use types::{
    CompositionAttribs, CompositionDescription, Control, Layout, RootAttribs, RootDescription,
    UiComposition, UiRoot,
};

use std::collections::HashSet;

use bedrock_addon_core::document::unversioned::origin_of;
use bedrock_addon_core::json::Prepared;
use bedrock_addon_core::load::{self, CollectionLoader, LoadContext, StackCx, Walk, WalkFile};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::known;

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticDetail, Diagnostics, JsonPath, Phase, Severity, Span,
};
use bedrock_addon_core::document::Registry;
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, JsonContent, JsonDefinition,
    JsonDocument, JsonPlaced, NoBeta, Notice, Origin, Placed, RegistryBuilder, Rejected,
    UnknownMembers, VersionInfo, default_path_flat_for_vanilla,
};
use bedrock_addon_core::id::{Id, UiCompositionId, UiRootId, kind};
use bedrock_addon_core::json::{
    EncodeError, JsonSlice, Value, WriteStyle, last_wins, probe_envelope, write_json,
};
use bedrock_addon_core::load::Reported;
use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput, PickedVersion};
use bedrock_addon_core::versioning::{
    BETA_VERSION, BelowFloor, BetaPermission, DocumentRead, Envelope, Family, InputKind, Loaded,
    MissingVersion, NoComponents, NoStable, ParseCx, Parsed, RangeModel, Single, ValidateCx,
    VersionStruct, Versioned, finish, no_parser_version, parse_as_written, write_envelope,
};
use history::{CompositionFamily, RootFamily, StyleFamily, StyleHistory};

/// The one `format_version` roots and compositions are read at.
pub const FORMAT_VERSION: FormatVersion = known::V1_21_130;

/// The data-driven UI of the resource packs (`rp.data_driven_ui()`): three collections loaded
/// and used together.
///
/// Roots and compositions are keyed by their `description.identifier`, compared byte for
/// byte. The client reads the stack from the top pack down and a later document replaces an
/// earlier one of the same identifier, so the **lowest** pack that defines an identifier
/// wins, whole; a document that failed replaces nothing. Every root of the stack is read
/// before the first composition, because a composition's `extension_point` must name an
/// `ExtensionPoint` control of a loaded root.
#[derive(Debug)]
#[non_exhaustive]
#[doc(alias = "ddui")]
#[doc(alias = "DataDrivenUIRepository")]
pub struct DataDrivenUi {
    /// The roots (`ddui/root/**/*.json`), in store order: by the first definition of each
    /// identifier, a replacement keeping the place.
    pub roots: Registry<UiRoot>,
    /// The compositions (`ddui/composition/**/*.json`), in store order.
    pub compositions: Registry<UiComposition>,
    /// The form styles (`ddui/styling/**/*.json`, engine 1.26.60 and later), in store order; the same
    /// combine rule as the roots'.
    pub styles: Registry<UiStyle>,
}

impl DataDrivenUi {
    /// Every finding of the three collections, roots first (the client's load order).
    pub fn diagnostics(&self) -> impl Iterator<Item = &Diagnostic> + '_ {
        self.roots
            .diagnostics()
            .chain(self.compositions.diagnostics())
            .chain(self.styles.diagnostics())
    }

    /// The controls spliced in at the extension point `name`: the markup of every
    /// composition whose `attribs.extension_point` is `name`, one composition after the other
    /// in store order, which is the order the client splices them in.
    pub fn extension_point_contents<'a>(
        &'a self,
        name: &'a str,
    ) -> impl Iterator<Item = &'a Control> + 'a {
        self.compositions
            .iter()
            .filter(move |(_, composition)| {
                composition.attribs.extension_point.as_deref() == Some(name)
            })
            .flat_map(|(_, composition)| composition.layout.markup.iter())
    }
}

/// `ddui/root`: `minecraft:ui-root` documents.
pub(crate) const ROOTS: &str = "ddui/root";
/// `ddui/composition`: `minecraft:ui-composition` documents.
pub(crate) const COMPOSITIONS: &str = "ddui/composition";
/// `ddui/styling`: `minecraft:ui-style` documents.
pub(crate) const STYLES: &str = "ddui/styling";

impl Family for RootFamily {
    const NAME: &'static str = "data-driven UI root";
    const PAYLOAD_KEY: &'static str = "minecraft:ui-root";
    // The loader reads exactly one version, so the range is closed at both ends:
    // [1.21.130, 1.21.130].
    const FLOOR: FormatVersion = FORMAT_VERSION;
    const ENVELOPE: Envelope = Envelope {
        // The loader reads a roots file through the document input: it takes the version as that
        // input reads it, and rule N2 holds.
        input: InputKind::Document,
        // The loader never reaches a document without a version; `stage` reports it with the
        // loader's own text.
        missing: MissingVersion::REJECT,
        // Below the first slice the versioned loader has no parser at all.
        below_floor: BelowFloor::NoParser,
        ..Envelope::DEFAULT
    };
    const RANGE_MODEL: RangeModel = RangeModel::ClosedToLatest;
    type Latest = UiRoot;
    type Beta = NoBeta<kind::UiRoot>;
    type Component = NoComponents;
    type History = Single<Self>;

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(root) = v {
            validate::namespace(root.description.identifier.as_str(), cx);
        }
    }
}

impl Family for CompositionFamily {
    const NAME: &'static str = "data-driven UI composition";
    const PAYLOAD_KEY: &'static str = "minecraft:ui-composition";
    // Exactly one version is read, as for roots.
    const FLOOR: FormatVersion = FORMAT_VERSION;
    const ENVELOPE: Envelope = Envelope {
        // The same document input as for roots: the version as it reads it, and rule N2.
        input: InputKind::Document,
        missing: MissingVersion::REJECT,
        // Below the first slice the versioned loader has no parser at all.
        below_floor: BelowFloor::NoParser,
        ..Envelope::DEFAULT
    };
    const RANGE_MODEL: RangeModel = RangeModel::ClosedToLatest;
    type Latest = UiComposition;
    type Beta = NoBeta<kind::UiComposition>;
    type Component = NoComponents;
    type History = Single<Self>;

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(composition) = v {
            // The namespace rule does not stop the validation: the extension-point rules run
            // after it either way.
            validate::namespace(composition.description.identifier.as_str(), cx);
            validate::external_extension_point(composition, cx);
        }
    }
}

/// The extension point a parsed composition asks for (`attribs.extension_point`), whether or
/// not the document then passes validation.
fn composition_point(any: &Single<CompositionFamily>) -> Option<String> {
    any.0.attribs.extension_point.clone()
}

/// The extension point of a document that asks for none.
fn no_point<F: Family>(_: &F::History) -> Option<String> {
    None
}

/// The stable struct of a loaded value (roots, compositions).
fn stable<F: Family>(loaded: Loaded<F>) -> Option<F::Latest> {
    match loaded {
        Loaded::Stable(value) => Some(value),
        Loaded::Beta(_) => None,
    }
}

/// The beta struct of a loaded style, the only one a style has.
fn beta(loaded: Loaded<StyleFamily>) -> Option<UiStyle> {
    match loaded {
        Loaded::Beta(style) => Some(style),
        Loaded::Stable(_) => None,
    }
}

impl Family for StyleFamily {
    const NAME: &'static str = "data-driven UI style";
    const PAYLOAD_KEY: &'static str = "minecraft:ui-style";
    // Unused: there is no stable struct, so every version but `"beta"` has no parser.
    const FLOOR: FormatVersion = known::V0_0_0;
    type Latest = NoStable;
    type Beta = UiStyle;
    // Every pack may load the beta version, with or without the Beta APIs toggle.
    const BETA_PERMISSION: BetaPermission = BetaPermission::Always;
    type Component = NoComponents;
    type History = StyleHistory;
    const ENVELOPE: Envelope = Envelope {
        // The same document input as for roots: the version as it reads it, and rule N2.
        input: InputKind::Document,
        missing: MissingVersion::REJECT,
        ..Envelope::DEFAULT
    };

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        // Styles follow the namespace rule of roots and compositions, with the same two texts.
        if let Loaded::Beta(style) = v {
            validate::namespace(style.description.identifier.as_str(), cx);
        }
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

/// The path of the envelope's `format_version`.
fn version_at() -> JsonPath {
    JsonPath::of_keys(["format_version"])
}

/// The loader's "Missing object" error for family `F`.
fn missing_object<F: Family>() -> Diagnostic {
    Diagnostic::of(codes::MISSING_OBJECT.with([F::PAYLOAD_KEY]))
}

/// The loader's closing line.
fn failed_to_load() -> Diagnostic {
    Diagnostic::of(&codes::FAILED_TO_LOAD)
}

/// A document the envelope stage refused, with its findings.
fn refused<F: Family>(diagnostics: Diagnostics) -> Parsed<F> {
    Parsed::from_value(
        None,
        VersionInfo::new(None, false, F::FLOOR, F::FLOOR, false),
        FormatVersionInput {
            version: Some(F::FLOOR),
            beta_literal: false,
            prerelease: false,
        },
        JsonPath::new(),
        diagnostics,
    )
}

/// One file read up to its struct: the parse stage and whether the typed load was reached (its
/// failure gets the loader's closing line).
struct Stage<F: Family> {
    parsed: Parsed<F>,
    typed: bool,
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
fn stage<F: Family>(document: JsonSlice<'_>, cx: &LoadContext) -> Stage<F> {
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
        };
    };
    // The version is the first textual `"format_version"` of the file, a number read as its
    // text.
    let picked = FormatVersionInput::pick(document.text);
    let note = if envelope.member_count() != 2 {
        Some(
            Diagnostic::of(&core_codes::DOCUMENT_MEMBER_COUNT)
                .severity(Severity::Info)
                .detail(DiagnosticDetail::Syntax(format!(
                    "{} root members, not \"format_version\" and a payload member",
                    envelope.member_count()
                ))),
        )
    } else if picked.is_malformed() {
        Some(Diagnostic::of(&codes::MALFORMED_VERSION).at(version_at()))
    } else {
        None
    };
    let missing = note.is_some() || envelope.payload(F::PAYLOAD_KEY).is_none();
    let versioned = picked.version().is_some();
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
        };
    }
    let PickedVersion::Version { input, written } = picked else {
        // No version in the input and none from the caller.
        diagnostics.push(Diagnostic::of(&codes::NO_VALID_VERSION).at(version_at()));
        return Stage {
            parsed: refused(diagnostics),
            typed: true,
        };
    };
    let mut parsed = parse_as_written::<F>(document, cx);
    if parsed.value.is_none() {
        no_parser(
            &mut parsed.diagnostics,
            &no_parser_version(&input, &written),
        );
    }
    Stage {
        parsed,
        typed: true,
    }
}

/// The loader has one closed range and no beta version: every version outside it, the beta
/// literal included, is "no parser available for version …" (below the range the pipeline says
/// so, `ENVELOPE.below_floor`), where the generic check asks for the Beta APIs toggle.
fn no_parser(diagnostics: &mut Diagnostics, version: &str) {
    let found = std::mem::take(diagnostics);
    for mut finding in found {
        // By legacy name: the variant and the definitions that were it.
        if matches!(finding.code.name(), "BetaFeaturesNotEnabled") {
            finding.code = core_codes::NO_PARSER_FOR_VERSION.with([version]).into();
        }
        diagnostics.push(finding);
    }
}

/// Gives every finding that names a member and no place in the file the place of that member
/// (the validators see the value, not the text). A file has a handful of them at most.
fn place(diagnostics: &mut Diagnostics, document: JsonSlice<'_>) {
    let found = std::mem::take(diagnostics);
    for mut finding in found {
        if finding.location.span.is_none() && !finding.location.json_path.is_empty() {
            finding.location.span =
                bedrock_addon_core::json::span_at(document, &finding.location.json_path);
        }
        diagnostics.push(finding);
    }
}

/// The rejection of a file that is no text the reader takes: not UTF-8, or nested too deep.
fn unreadable<F: Family>() -> Diagnostics {
    let message = "not UTF-8 JSON within the nesting limit".to_owned();
    let mut diagnostics = Diagnostics::default();
    diagnostics.push(
        Diagnostic::of(core_codes::DOCUMENT_JSON.with([message.as_str()]))
            .severity(Severity::Info)
            .detail(DiagnosticDetail::Syntax(message)),
    );
    diagnostics.push(missing_object::<F>());
    diagnostics
}

/// The extension point a composition asks for and where the file writes it, whether the
/// document loaded or a validator refused it; `None` when the struct was not read.
pub(crate) type AskedPoint = Option<(String, Option<Span>)>;

/// Loads the file at `origin` of family `F` as the client's loader reads it: the identifier the
/// payload names (when it is a string), the document of the struct `take` keeps or the
/// rejection, and the extension point `point` reads from the struct.
fn parse_file<F: Family, T>(
    origin: Origin,
    bytes: &[u8],
    cx: &LoadContext,
    take: fn(Loaded<F>) -> Option<T>,
    point: fn(&F::History) -> Option<String>,
) -> (Option<String>, Result<Document<T>, Rejected>, AskedPoint) {
    let Ok(prepared) = Prepared::new(bytes) else {
        let mut diagnostics = unreadable::<F>();
        diagnostics.locate(&origin);
        return (None, Err(Rejected::new(origin, diagnostics)), None);
    };
    let text = prepared.as_str();
    // Of a member written twice the last value stands, before any look at the text.
    let cleaned = last_wins(text);
    let document = JsonSlice::new(&cleaned);
    let Stage { parsed, typed } = stage::<F>(document, cx);
    let identifier = parsed.identity.identifier.clone();
    let extension_point = parsed.value.as_ref().and_then(point).map(|name| {
        let mut at = JsonPath::new();
        for member in [F::PAYLOAD_KEY, "attribs", "extension_point"] {
            at.push(member);
        }
        (name, bedrock_addon_core::json::span_at(document, &at))
    });
    let outcome = finish(parsed, cx);
    let mut diagnostics = outcome.diagnostics;
    place(&mut diagnostics, document);
    let mut version = outcome.version;
    if matches!(outcome.value, Some(Loaded::Beta(_))) {
        version.parsed_as = BETA_VERSION;
    }
    let result = if let Some(value) = outcome.value.and_then(take) {
        diagnostics.locate(&origin);
        Ok(Document::new(value, origin, version, diagnostics))
    } else {
        if typed {
            diagnostics.push(failed_to_load());
        }
        Err(Rejected::located(origin, diagnostics))
    };
    (identifier, result, extension_point)
}

/// `JsonDocument::read` of a stable family: the client's loader, not core's pipeline.
fn read_stable<F: Family>(origin: Origin, bytes: &[u8], cx: &LoadContext) -> DocumentRead<F> {
    match parse_file::<F, F::Latest>(origin, bytes, cx, stable, no_point::<F>) {
        (_, Ok(document), _) => DocumentRead::Stable(document),
        (identifier, Err(rejected), _) => DocumentRead::Rejected(identifier, rejected),
    }
}

/// The contribution of a file the walk read: its document under its identifier, or its
/// rejection under the identifier the payload names.
fn contribution<T: Definition>(
    identifier: Option<String>,
    result: Result<Document<T>, Rejected>,
    id: fn(&T) -> &Id<T::Kind>,
) -> Contribution<T> {
    match result {
        Ok(document) => Contribution::Stable(id(&document).clone(), document),
        Err(rejected) => Contribution::Rejected(identifier.map(Id::parse), rejected),
    }
}

/// Parses a file of family `F` without upgrading it: the struct the file's `format_version`
/// selects.
fn as_written<F: Family>(
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> Result<Document<F::History>, Rejected> {
    let origin = Origin::file(PackRef::app_data(), PackSide::Resource, path.clone());
    let Ok(prepared) = Prepared::new(bytes) else {
        let mut diagnostics = unreadable::<F>();
        diagnostics.locate(&origin);
        return Err(Rejected::new(origin, diagnostics));
    };
    let text = prepared.as_str();
    let cleaned = last_wins(text);
    let document = JsonSlice::new(&cleaned);
    let Stage { parsed, typed } = stage::<F>(document, cx);
    let mut diagnostics = parsed.diagnostics;
    place(&mut diagnostics, document);
    if let Some(value) = parsed.value {
        diagnostics.locate(&origin);
        Ok(Document::new(value, origin, parsed.version, diagnostics))
    } else {
        if typed {
            diagnostics.push(failed_to_load());
        }
        Err(Rejected::located(origin, diagnostics))
    }
}

// ---- the documents -------------------------------------------------------------------------

impl JsonDocument for UiRoot {
    type Family = RootFamily;
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// One file outside a stack: everything the client checks in the file itself, read as its
    /// loader reads it (the envelope stage and the closing line are the family's own).
    fn read(origin: Origin, bytes: &[u8], cx: &LoadContext) -> DocumentRead<RootFamily> {
        read_stable(origin, bytes, cx)
    }

    /// The family's own envelope stage, not core's.
    fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> Result<Document<Single<RootFamily>>, Rejected> {
        as_written::<RootFamily>(path, bytes, cx)
    }
}

impl JsonPlaced for UiRoot {
    /// `ddui/root/<name>.json` for a `minecraft:` identifier (the `/` of the name as `_`),
    /// which a shipped pack's flat walk reads back; `ddui/root/<namespace>/<name>.json` for
    /// any other.
    fn place(&self) -> PackPath {
        default_path_flat_for_vanilla(ROOTS, &self.description.identifier)
    }
}

impl JsonDefinition for UiRoot {
    type Kind = kind::UiRoot;
    fn identifier(&self) -> &UiRootId {
        &self.description.identifier
    }
}

impl JsonDocument for UiComposition {
    type Family = CompositionFamily;
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// One file outside a stack: everything the client checks in the file itself. Whether a
    /// composition's `extension_point` is declared by a root is known only to a stack
    /// (`PackStack::data_driven_ui`).
    fn read(origin: Origin, bytes: &[u8], cx: &LoadContext) -> DocumentRead<CompositionFamily> {
        read_stable(origin, bytes, cx)
    }

    /// The family's own envelope stage, not core's.
    fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> Result<Document<Single<CompositionFamily>>, Rejected> {
        as_written::<CompositionFamily>(path, bytes, cx)
    }
}

impl JsonPlaced for UiComposition {
    /// `ddui/composition/<name>.json` for a `minecraft:` identifier (the `/` of the name as
    /// `_`), which a shipped pack's flat walk reads back;
    /// `ddui/composition/<namespace>/<name>.json` for any other.
    fn place(&self) -> PackPath {
        default_path_flat_for_vanilla(COMPOSITIONS, &self.description.identifier)
    }
}

impl JsonDefinition for UiComposition {
    type Kind = kind::UiComposition;
    fn identifier(&self) -> &UiCompositionId {
        &self.description.identifier
    }
}

// A style is a beta struct whose family's newest stable struct is `NoStable`: it keeps hand
// impls beside the blanket ones (`JsonDocument` needs `Family::Latest = Self`).
impl bedrock_addon_core::__private::Sealed for UiStyle {}

impl Content for UiStyle {
    const NAME: &'static str = "data-driven UI style";
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// One file outside a stack: everything the client checks in the file itself.
    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        let origin = Origin::file(PackRef::app_data(), PackSide::Resource, path.clone());
        parse_file(origin, bytes, cx, beta, no_point::<StyleFamily>).1
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        let payload = style::payload(self)?;
        write_json(
            &write_envelope::<StyleFamily>(BETA_VERSION, payload, &UnknownMembers::default()),
            out,
            style,
        )
    }
}

impl JsonContent for UiStyle {}

impl UiStyle {
    /// Parses one file without upgrading it: the struct its `format_version` selects, the beta
    /// struct.
    ///
    /// # Errors
    ///
    /// Returns [`Rejected`] when the file cannot be read into a value at all, such as bytes
    /// that are not JSON or a document the family refuses; it carries every finding about the
    /// file.
    pub fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> Result<Document<StyleHistory>, Rejected> {
        as_written::<StyleFamily>(path, bytes, cx)
    }
}

impl Definition for UiStyle {
    type Kind = kind::UiStyle;
    type File = UiStyle;
    type Beta = NoBeta<kind::UiStyle>;
}

impl Placed for UiStyle {
    /// `ddui/styling/<name>.json` for a `minecraft:` identifier (the `/` of the name as `_`),
    /// `ddui/styling/<namespace>/<name>.json` for any other, as roots.
    fn default_path(&self) -> PackPath {
        default_path_flat_for_vanilla(STYLES, &self.description.identifier)
    }
}

// ---- the group -----------------------------------------------------------------------------

#[doc(hidden)]
/// What one walked file parses to.
#[derive(Debug)]
pub enum UiFile {
    /// A file of `ddui/root`.
    Root(Box<Contribution<UiRoot>>),
    /// A file of `ddui/composition`, with the extension point it asks for.
    Composition(Box<Contribution<UiComposition>>, AskedPoint),
    /// A file of `ddui/styling`.
    Style(Box<Contribution<UiStyle>>),
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
const WALK: Walk = Walk::json_tree_in(&[ROOTS, COMPOSITIONS, STYLES])
    .top_down()
    .directory_major()
    .in_pack(reached)
    .read_error(codes::UNREAD.template());

/// Parses one walked file by its directory.
fn parse_walk_file(walked: &WalkFile, cx: &LoadContext) -> UiFile {
    match WALK.matches(&walked.key) {
        // The parse of the registry constructor, through `JsonDocument::read`.
        Some(0) => UiFile::Root(Box::new((load::registry(WALK).parse)(walked, cx))),
        Some(1) => {
            let (identifier, result, point) = parse_file(
                origin_of(walked),
                &walked.bytes,
                cx,
                stable::<CompositionFamily>,
                composition_point,
            );
            let id = <UiComposition as JsonDefinition>::identifier;
            UiFile::Composition(Box::new(contribution(identifier, result, id)), point)
        }
        _ => {
            let (identifier, result, _) = parse_file(
                origin_of(walked),
                &walked.bytes,
                cx,
                beta,
                no_point::<StyleFamily>,
            );
            let id: fn(&UiStyle) -> &Id<kind::UiStyle> = |style| &style.description.identifier;
            UiFile::Style(Box::new(contribution(identifier, result, id)))
        }
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
            if !key.starts_with(&prefix) || file.key.is_json() || !reached(&file.key, &pack.context)
            {
                continue;
            }
            let ignored = Origin::file(pack.pack.clone(), PackSide::Resource, file.path.clone());
            registry.push_diagnostic(
                Diagnostic::of(&core_codes::IGNORED_FILE)
                    .phase(Phase::Validate)
                    .in_file(&ignored),
            );
        }
    }
}

/// The error of a composition whose extension point no stored root declares.
fn not_defined(origin: &Origin, name: &str, span: Option<Span>) -> Diagnostic {
    let mut at = JsonPath::of_keys([CompositionFamily::PAYLOAD_KEY]);
    at.push("attribs");
    at.push("extension_point");
    let finding = Diagnostic::of(codes::NOT_DEFINED.with([name]))
        .at(at)
        .phase(Phase::Validate)
        .in_file(origin)
        .detail(DiagnosticDetail::Reference {
            kind: "extension point",
            id: name.to_owned(),
            hint: None,
        });
    match span {
        Some(span) => finding.span(span),
        None => finding,
    }
}

/// `findings` with `finding` before the loader's closing line, which it adds when there is
/// none: the validator's messages come before the closing line.
fn before_closing_line(findings: &mut Diagnostics, finding: Diagnostic, origin: &Origin) {
    let closing = |f: &Diagnostic| f.is(&codes::FAILED_TO_LOAD);
    let mut kept: Vec<Diagnostic> = std::mem::take(findings)
        .into_iter()
        .filter(|f| !closing(f))
        .collect();
    kept.push(finding);
    let last = failed_to_load().in_file(origin);
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
            UiFile::Style(contribution) => styles.push(*contribution),
            UiFile::Root(contribution) => roots.push(*contribution),
            UiFile::Composition(contribution, point) => pending.push((*contribution, point)),
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
    for (contribution, extension_point) in pending {
        let undeclared = extension_point.filter(|(name, _)| !declared.contains(name.as_str()));
        match (contribution, undeclared) {
            (Contribution::Stable(id, document), Some((name, span))) => {
                let origin = document.origin().clone();
                let mut diagnostics = document.diagnostics().clone();
                let finding = not_defined(&origin, &name, span);
                before_closing_line(&mut diagnostics, finding, &origin);
                compositions.push(Contribution::Rejected(
                    Some(id),
                    Rejected::new(origin, diagnostics),
                ));
            }
            (Contribution::Rejected(id, mut rejected), Some((name, span))) => {
                let origin = rejected.origin.clone();
                let finding = not_defined(&origin, &name, span);
                before_closing_line(&mut rejected.diagnostics, finding, &origin);
                compositions.push(Contribution::Rejected(id, rejected));
            }
            (contribution, _) => compositions.push(contribution),
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

#[doc(hidden)]
/// The loader of `rp.data_driven_ui()`: the three directories of every resource pack, combined
/// into the group.
pub const LOADER: CollectionLoader<DataDrivenUi, UiFile> = CollectionLoader {
    walk: WALK,
    parse: parse_walk_file,
    combine,
    needs: &[],
    then: None,
};

impl Reported for DataDrivenUi {
    fn reported(&self) -> Box<dyn Iterator<Item = &Diagnostic> + '_> {
        Box::new(self.diagnostics())
    }
}

#[cfg(test)]
mod reading_table {
    //! The family's reading table: what the pipeline reads each of its files and structs by.

    use bedrock_addon_core::versioning::testing::{assert_reading_table, reading_table};

    #[test]
    fn root_reads_as_pinned() {
        assert_reading_table(
            "RootFamily",
            &reading_table::<super::RootFamily>(),
            r"
                text: Document, clamps beyond a double: false, duplicates kept in: []
                envelope: Always
                missing version: rejected: NoValidVersion
                below floor: NoParser
                empty document: Error
                1.21.130..=1.21.130 (0..=0): json Rejected/ConvertAndTell/ClampAndTell/Error/reports/fatal/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Error/reports/fatal/Refuse
            ",
        );
    }

    #[test]
    fn composition_reads_as_pinned() {
        assert_reading_table(
            "CompositionFamily",
            &reading_table::<super::CompositionFamily>(),
            r"
                text: Document, clamps beyond a double: false, duplicates kept in: []
                envelope: Always
                missing version: rejected: NoValidVersion
                below floor: NoParser
                empty document: Error
                1.21.130..=1.21.130 (0..=0): json Rejected/ConvertAndTell/ClampAndTell/Error/reports/fatal/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Error/reports/fatal/Refuse
            ",
        );
    }

    #[test]
    fn style_reads_as_pinned() {
        assert_reading_table(
            "StyleFamily",
            &reading_table::<super::StyleFamily>(),
            r"
                text: Document, clamps beyond a double: false, duplicates kept in: []
                envelope: Always
                missing version: rejected: NoValidVersion
                below floor: UseFormatVersion
                empty document: Error
                beta: json Rejected/ConvertAndTell/ClampAndTell/Error/reports/fatal/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Error/reports/fatal/Refuse
            ",
        );
    }
}
