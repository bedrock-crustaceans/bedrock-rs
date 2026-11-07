//! Client biomes: what a biome looks and sounds like on the client, from the resource packs.
//! `ClientBiome` is `biomes/*.json` (`minecraft:client_biome`), one biome per file, at the
//! newest format (1.21.130); older files are upgraded when they load, and `history` has their
//! shapes; `Beta` is the `"format_version": "beta"` document.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod codes;
pub mod component;
pub mod history;

mod color;
mod read;
#[cfg(test)]
mod tests;
mod validate;
mod write;

pub use bedrock_addon_shared_biome_env::{
    BoolAttribute, BoolOperation, ColorAttribute, ColorOperation, EnvironmentAttributes,
    FloatAttribute, FloatOperation,
};
pub use color::{Color255Rgb, Color255Rgba};

use bedrock_addon_core::document::ComponentMap;
use bedrock_addon_core::id::BiomeId;

use component::ClientBiomeComponent;
use indexmap::IndexSet;

use crate::component::child;
use crate::history::{ClientBiomeFamily, ClientBiomeHistory, identifier_path};
use crate::read::{MODERN, PAYLOAD_KEY};
use std::borrow::Cow;

use crate::component::{DryFoliageColor, SkyColor, SoundAddition, WaterAppearance};
use bedrock_addon_core::__private::Sealed;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Code, Diagnostic, DiagnosticDetail, Diagnostics, JsonPath, Phase, Severity, builders,
};
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, JsonContent, NoBeta, Notice, Origin,
    Placed, Registry, RegistryBuilder, Rejected, default_path_flat_for_vanilla,
};
use bedrock_addon_core::id::SoundEventName;
use bedrock_addon_core::id::kind;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value, WriteStyle};
use bedrock_addon_core::json::{PrepareError, Prepared};
use bedrock_addon_core::load::DocumentSource;
use bedrock_addon_core::load::{CollectionLoader, LoadContext, StackCx, Walk, WalkFile};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::schema::{self, Schema};
use bedrock_addon_core::version::known;
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput};
use bedrock_addon_core::versioning::{
    BeyondWidth, ComponentSection, ComponentTable, Envelope, EnvelopeCheck, Family, Identity,
    InputKind, Loaded, MissingVersion, Parsed, Reading, Scalars, StructIndex, finish,
    parse_as_written,
};

/// A client biome (`minecraft:client_biome`), at the newest format version.
///
/// JSON: `{"format_version": "1.21.130", "minecraft:client_biome": {"description":
/// {"identifier"}, "components": {…}}}`. A document of an older version is upgraded to this
/// shape when it loads: the components vanilla's own older files left to code are inserted.
/// The client applies it only to a biome the server names.
#[derive(Clone, Debug, PartialEq)]
#[doc(alias = "minecraft:client_biome")]
#[doc(alias = "client_biome.json")]
pub struct ClientBiome {
    /// The biome's name (JSON `description`).
    pub description: Description,
    /// The components (JSON `components`, may be empty).
    pub components: ComponentMap<ClientBiomeComponent>,
}

/// `minecraft:client_biome/description`.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Description {
    /// The biome's identifier (JSON `identifier`): `namespace:name` in lower case from format
    /// 1.21.90 (matching `^[a-z0-9._%+-]+:[a-z0-9._%+-]+$`); an older document may name a
    /// vanilla biome without its namespace, which the upgrade adds.
    pub identifier: BiomeId,
}

/// A beta client biome (`"format_version": "beta"`): the newest components plus
/// `environment_attributes`. Loaded only from a base-game pack or with the Beta APIs toggle,
/// and never upgraded.
#[derive(Clone, Debug, PartialEq)]
#[doc(alias = "minecraft:client_biome")]
pub struct Beta {
    /// The biome's name (JSON `description`).
    pub description: Description,
    /// The components (JSON `components`, may be empty).
    pub components: ComponentMap<ClientBiomeComponent>,
    /// Experimental environment attributes (JSON `environment_attributes`).
    pub environment_attributes: Option<EnvironmentAttributes>,
}

/// The directory client-biome files live in.
const DIRECTORY: &str = "biomes";
/// The floor: vanilla's lowest accepted version, and the first client-biome format.
const FLOOR: FormatVersion = FormatVersion::new(1, 21, 40);
/// Where the legacy range of formats starts: the game has no parser at all for a version below
/// it.
const LEGACY_SLICE: FormatVersion = FormatVersion::new(1, 0, 0);
/// The last struct of the legacy range.
const LAST_LEGACY: FormatVersion = known::V1_21_120;

/// How every client-biome document is read: the document-input profile, its scalars strictly
/// (a member takes its own JSON type alone, and a number beyond its float is a type error).
/// The components read by it too, also outside a versioned load.
pub(crate) fn reading() -> Reading {
    ClientBiomeFamily::ENVELOPE
        .input
        .profile()
        .with_scalars(Scalars::Strict)
        .with_beyond_width(BeyondWidth::Error)
}

impl Family for ClientBiomeFamily {
    const NAME: &'static str = "client_biome";
    const PAYLOAD_KEY: &'static str = PAYLOAD_KEY;
    const FLOOR: FormatVersion = FLOOR;
    type Latest = ClientBiome;
    type Beta = Beta;
    type Component = ClientBiomeComponent;
    type History = ClientBiomeHistory;
    const ENVELOPE: Envelope = Envelope {
        // The game reads every file as a document: the version is the first textual
        // `format_version` (a number is read as its text) and the size limits apply.
        input: InputKind::Document,
        // What the document does not read is kept for write-back: a number no double holds is
        // the largest finite one, with a warning at it.
        clamps_beyond_double: true,
        // Below 1.21.130 the game reads the raw text of the whole document; from 1.21.130 the
        // envelope has exactly two members, `format_version` and `minecraft:client_biome`.
        check: EnvelopeCheck::From(MODERN),
        // A file without a `format_version` is rejected: "no valid version passed by the caller or
        // found in the input".
        missing: MissingVersion::REJECT,
        ..Envelope::DEFAULT
    };

    /// [`reading`] for every slice.
    fn reading(_: StructIndex, _: DocumentSource) -> Reading {
        reading()
    }

    fn components() -> &'static ComponentTable<Self> {
        ClientBiomeComponent::TABLE
    }

    fn component_sections(v: &mut ClientBiome) -> Vec<ComponentSection<'_, ClientBiomeComponent>> {
        let at = JsonPath::of_keys(["components"]);
        vec![ComponentSection::new(at, false, &mut v.components)]
    }
}

/// The client's component checks on a document that claims its identifier: the document, or
/// its rejection with the checks' errors after its own findings.
fn component_checked<T>(
    document: Document<T>,
    components: impl Fn(&T) -> &ComponentMap<ClientBiomeComponent>,
) -> Result<Document<T>, Rejected> {
    let mut failed = validate::check_components(components(document.value()));
    if failed.is_empty() {
        return Ok(document);
    }
    let origin = document.origin().clone();
    failed.locate(&origin);
    let mut diagnostics = document.diagnostics().clone();
    diagnostics.extend(failed);
    Err(Rejected::new(origin, diagnostics))
}

// The quartet and the loader are hand-written. `Content::parse` runs the client's component
// checks on the document, and the walk's parse must not (only the claiming document of an
// identifier is checked, in `combine`), so the one `JsonDocument::read` both would share
// cannot be either; and a document that failed to load claims nothing here
// (`Contribution::Rejected(None, ..)`), which the registry constructor's rejection, naming
// the identifier, would change under `ClaimBeforeValidate`.
impl Sealed for ClientBiome {}

impl Content for ClientBiome {
    const NAME: &'static str = "client biome";
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// Parses one client-biome file, upgraded to the newest struct. A beta document is
    /// rejected here (it is a [`Beta`]).
    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        let origin = origin(PackRef::app_data(), path);
        match load(origin, bytes, cx)? {
            (Loaded::Stable(biome), parts) => {
                component_checked(parts.with(biome), |b| &b.components)
            }
            (Loaded::Beta(_), parts) => Err(parts.unsupported("beta")),
        }
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        self.write_into(out, style)
    }
}

impl JsonContent for ClientBiome {}

impl Definition for ClientBiome {
    type Kind = kind::Biome;
    type File = ClientBiome;
    type Beta = Beta;
}

impl Placed for ClientBiome {
    /// `biomes/<name>.json` for a `minecraft:` identifier (the `/` of the name as `_`), which
    /// a base-game pack's flat walk reads back; `biomes/<namespace>/<name>.json` for any
    /// other.
    fn default_path(&self) -> PackPath {
        default_path_flat_for_vanilla(DIRECTORY, &self.description.identifier)
    }
}

impl Sealed for Beta {}

impl Content for Beta {
    const NAME: &'static str = "beta client biome";
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// Parses one beta client-biome file (`"format_version": "beta"`); any other version is
    /// rejected.
    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        let origin = origin(PackRef::app_data(), path);
        match load(origin, bytes, cx)? {
            (Loaded::Beta(beta), parts) => component_checked(parts.with(beta), |b| &b.components),
            (Loaded::Stable(_), parts) => {
                let version = parts.version.effective.to_string();
                Err(parts.unsupported(&version))
            }
        }
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        self.write_into(out, style)
    }
}

impl JsonContent for Beta {}

impl Definition for Beta {
    type Kind = kind::Biome;
    type File = Beta;
    type Beta = NoBeta<kind::Biome>;
}

impl Placed for Beta {
    /// `biomes/<name>.json` for a `minecraft:` identifier (the `/` of the name as `_`), which
    /// a base-game pack's flat walk reads back; `biomes/<namespace>/<name>.json` for any
    /// other.
    fn default_path(&self) -> PackPath {
        default_path_flat_for_vanilla(DIRECTORY, &self.description.identifier)
    }
}

impl ClientBiome {
    /// Parses one client-biome file without upgrading it: the struct its `format_version`
    /// selects, with the findings of the parse.
    ///
    /// # Errors
    ///
    /// Returns [`Rejected`](bedrock_addon_core::document::Rejected) with the findings the game's content
    /// log would print when the bytes cannot be read as this file at all: not JSON, a
    /// `format_version` with no struct here, or a document that does not fit the struct it selects.
    pub fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> Result<Document<ClientBiomeHistory>, Rejected> {
        let origin = origin(PackRef::app_data(), path);
        let parsed = parse(&origin, bytes, cx)?;
        let mut diagnostics = parsed.diagnostics;
        diagnostics.locate(&origin);
        match parsed.value {
            Some(value) => Ok(Document::new(value, origin, parsed.version, diagnostics)),
            None => Err(Rejected::new(origin, diagnostics)),
        }
    }
}

/// `history::upgrade`: rebuilds the parse stage from the document and finishes it.
pub(crate) fn upgrade_document(
    document: Document<ClientBiomeHistory>,
    cx: &LoadContext,
) -> Result<Document<ClientBiome>, Rejected> {
    let (value, origin, version, mut found) = document.into_parsed_parts();
    // The steps start from no finding, with the unknown members to move and take.
    let mut rows = Diagnostics::default();
    let unknown = found.take_unknown();
    if !unknown.is_empty() {
        *rows.unknown_mut() = unknown;
    }
    let identity = Identity {
        identifier: Some(identifier_of(&value).to_owned()),
        runtime_identifier: None,
    };
    let written = FormatVersionInput {
        version: (!version.beta).then_some(version.written.unwrap_or(version.effective)),
        beta_literal: version.beta,
        prerelease: false,
    };
    let parsed = Parsed::<ClientBiomeFamily>::from_value(
        Some(value),
        version,
        written,
        child(&JsonPath::new(), &[PAYLOAD_KEY]),
        rows,
    )
    .with_identity(identity);
    let outcome = finish(parsed, cx);
    let mut more = outcome.diagnostics;
    more.locate(&origin);
    let mut diagnostics = found;
    diagnostics.extend_with_unknown(more);
    let parts = Parts {
        origin,
        version: outcome.version,
        diagnostics,
    };
    match outcome.value {
        Some(Loaded::Stable(biome)) => component_checked(parts.with(biome), |b| &b.components),
        Some(Loaded::Beta(_)) => Err(parts.unsupported("beta")),
        None => Err(Rejected::new(parts.origin, parts.diagnostics)),
    }
}

/// The identifier of a parsed document, as written.
fn identifier_of(value: &ClientBiomeHistory) -> &str {
    match value {
        ClientBiomeHistory::V1_21_40(b) => b.description.identifier.as_str(),
        ClientBiomeHistory::V1_21_70(b) => b.description.identifier.as_str(),
        ClientBiomeHistory::V1_21_90(b) => b.description.identifier.as_str(),
        ClientBiomeHistory::V1_21_100(b) => b.description.identifier.as_str(),
        ClientBiomeHistory::V1_21_110(b) => b.description.identifier.as_str(),
        ClientBiomeHistory::V1_21_120(b) => b.description.identifier.as_str(),
        ClientBiomeHistory::V1_21_130(b) => b.description.identifier.as_str(),
        ClientBiomeHistory::Beta(b) => b.description.identifier.as_str(),
    }
}

/// A loaded document's parts, before the value goes into a [`Document`].
struct Parts {
    origin: Origin,
    version: bedrock_addon_core::document::VersionInfo,
    diagnostics: Diagnostics,
}

impl Parts {
    fn with<T>(self, value: T) -> Document<T> {
        Document::new(value, self.origin, self.version, self.diagnostics)
    }

    /// The rejection of a document whose struct the asked type is not: "unsupported version:
    /// no parser available for version %s".
    fn unsupported(self, version: &str) -> Rejected {
        let Self {
            origin,
            mut diagnostics,
            ..
        } = self;
        let finding = Diagnostic::error(no_parser(version)).in_file(&origin);
        diagnostics.push(finding);
        Rejected::new(origin, diagnostics)
    }
}

/// "unsupported version: no parser available for version %s".
fn no_parser(version: &str) -> Code {
    core_codes::NO_PARSER_FOR_VERSION.with([version.to_string()])
}

/// The text of a file, after the BOM, the UTF-8 and the depth check.
fn text<'b>(bytes: &'b [u8], origin: &Origin) -> Result<Cow<'b, str>, Rejected> {
    let malformed = |message: &str| {
        let mut diagnostics = Diagnostics::default();
        diagnostics.push(Diagnostic::of(builders::syntax(message)));
        diagnostics.locate(origin);
        Rejected::new(origin.clone(), diagnostics)
    };
    // `new` never refuses an empty text: that is the chain's to report.
    let text = Prepared::new(bytes)
        .map_err(|error| match error {
            PrepareError::NotUtf8(_) => malformed("the file is not UTF-8"),
            _ => malformed("the JSON nests too deeply"),
        })?
        .into_text();
    Ok(text)
}

/// The parse stage of one file: the chain's `parse_as_written`, with the findings in the words
/// vanilla's client-biome loader has for them.
fn parse(
    origin: &Origin,
    bytes: &[u8],
    cx: &LoadContext,
) -> Result<Parsed<ClientBiomeFamily>, Rejected> {
    let text = text(bytes, origin)?;
    let mut parsed = parse_as_written::<ClientBiomeFamily>(JsonSlice::new(&text), cx);
    // The payload is under `minecraft:client_biome` on the raw-text path too: the checks after
    // the parse report below it.
    parsed.at = child(&JsonPath::new(), &[PAYLOAD_KEY]);
    reword(&mut parsed, &text, origin.path.as_str());
    Ok(parsed)
}

/// Puts the parse stage's generic findings in the client-biome loader's words
/// and adds the crate's own note for a version between two structs.
fn reword(parsed: &mut Parsed<ClientBiomeFamily>, text: &str, path: &str) {
    let written = parsed.version.written;
    let mut found = Diagnostics::default();
    // The findings are told again in other words; the unknown members the read kept stay.
    let kept = parsed.diagnostics.take_unknown();
    if !kept.is_empty() {
        found.unknown_mut().absorb(&JsonPath::new(), kept);
    }
    for mut diagnostic in std::mem::take(&mut parsed.diagnostics) {
        let mut after = None;
        // The pipeline's lines, on core's definitions.
        let plain_malformed_version = diagnostic.is(&core_codes::MALFORMED_FORMAT_VERSION);
        let use_format = diagnostic.is(&core_codes::USE_FORMAT_VERSION);
        let plain_missing = diagnostic.is(&core_codes::MISSING_REQUIRED_FIELD);
        let reworded = match &diagnostic.code {
            // The game clears the input's version and then finds none.
            _ if plain_malformed_version => Some(Code::from(&codes::NO_VALID_VERSION)),
            // Below the floor the game refuses the version in the legacy loader's own words;
            // below the legacy range it has no parser at all.
            _ if use_format => written.filter(|version| *version < FLOOR).map(|version| {
                let version_text = version.to_string();
                if version < LEGACY_SLICE {
                    no_parser(&version_text)
                } else {
                    codes::CANNOT_LOAD.with([path, version_text.as_str()])
                }
            }),
            // The game takes the first textual `format_version`: a document below 1.21.130
            // that has it only below its root reaches the legacy loader, which finds none at
            // the root.
            _ if plain_missing
                && written.is_some_and(|version| version < MODERN)
                && diagnostic.location.json_path.is_key("format_version") =>
            {
                Some(codes::NO_VERSION_STRING.with([path]))
            }
            // A text the legacy loader cannot read as a JSON object; the reader's message stays
            // the finding's detail.
            _ if diagnostic.is(&core_codes::DOCUMENT_JSON)
                && scanned_version(text).is_some_and(|version| version < MODERN) =>
            {
                let code = if bedrock_addon_core::json::from_str_jsonc::<Value>(text).is_ok() {
                    codes::LEGACY_ROOT.with([path])
                } else {
                    let message = diagnostic.syntax_message().unwrap_or_default();
                    codes::LEGACY_SYNTAX.with([message])
                };
                Some(code)
            }
            // The content log line that follows "Invalid Client Biome file".
            _ if diagnostic.is(&codes::INVALID_FILE) => {
                after = Some(Diagnostic::of(codes::FAILED_TO_LOAD.with([path])).detail(
                    DiagnosticDetail::Syntax("the document was not loaded".to_owned()),
                ));
                None
            }
            _ => None,
        };
        if let Some(code) = reworded {
            diagnostic.code = code;
        }
        found.push(diagnostic);
        found.extend(after);
    }
    // 1.21.121 … 1.21.129 are read as 1.21.120 documents (the game does not check that the
    // version stays within the struct's range).
    if parsed.value.is_some()
        && written.is_some_and(|version| version > LAST_LEGACY && version < MODERN)
    {
        let mut at = JsonPath::new();
        at.push("format_version");
        found.push(Diagnostic::of(&core_codes::UNUSUAL_FORMAT_VERSION).at(at));
    }
    parsed.diagnostics = found;
}

/// The version after the first `"format_version"` key of the raw text, for a text that is no
/// JSON object: the game scans the raw text for it before any parse.
fn scanned_version(text: &str) -> Option<FormatVersion> {
    let (_, rest) = text.split_once("\"format_version\"")?;
    let rest = rest.trim_start().strip_prefix(':')?.trim_start();
    let (version, _) = rest.strip_prefix('"')?.split_once('"')?;
    FormatVersionInput::parse(version)?.version
}

/// One file through the chain: the loaded value with its parts, or the rejection. The
/// client's component checks are not run here (see `validate`).
fn load(
    origin: Origin,
    bytes: &[u8],
    cx: &LoadContext,
) -> Result<(Loaded<ClientBiomeFamily>, Parts), Rejected> {
    let parsed = parse(&origin, bytes, cx)?;
    let outcome = finish(parsed, cx);
    let mut diagnostics = outcome.diagnostics;
    diagnostics.locate(&origin);
    match outcome.value {
        Some(value) => Ok((
            value,
            Parts {
                origin,
                version: outcome.version,
                diagnostics,
            },
        )),
        None => Err(Rejected::new(origin, diagnostics)),
    }
}

/// The origin of a resource-pack file of `pack`.
fn origin(pack: PackRef, path: &PackPath) -> Origin {
    Origin::file(pack, PackSide::Resource, path.clone())
}

// ---- the collection ----------------------------------------------------------------------

/// A client-biome file: any `.json` file below `biomes/` (vanilla's own are named
/// `<name>.client_biome.json`, but the loader asks only for the extension).
#[allow(
    clippy::case_sensitive_file_extension_comparisons,
    reason = "vanilla's directory walk matches `.json` as written"
)]
/// Whether vanilla's walk reaches `path` in the pack of `cx`: the game lists `biomes/` of a
/// pack shipped in the app package without descending into sub-folders, and every other pack
/// with its sub-folders.
fn reached(path: &PackPath, cx: &LoadContext) -> bool {
    let nested = path.segments().count() > 2;
    !(cx.is_base_game_pack && nested)
}

/// `biomes/**/*.json` of every resource pack; `biomes/*.json` in a base-game pack.
const WALK: Walk = Walk::json_tree_in(&[DIRECTORY]).top_down().in_pack(reached);

/// One file's contribution.
fn parse_walk_file(file: &WalkFile, cx: &LoadContext) -> Contribution<ClientBiome> {
    let origin = Origin::file(file.pack.clone(), file.side, file.path.clone());
    match load(origin, &file.bytes, cx) {
        Ok((Loaded::Stable(biome), parts)) => {
            let id = biome.description.identifier.clone();
            Contribution::Stable(id, parts.with(biome))
        }
        Ok((Loaded::Beta(beta), parts)) => {
            let id = beta.description.identifier.clone();
            Contribution::Beta(id, parts.with(beta))
        }
        // A document that failed to load never reaches the client's claim.
        Err(rejected) => Contribution::Rejected(None, rejected),
    }
}

/// The identifier the file at `origin` names when it loads as a client biome, stable or beta,
/// whatever the client's component checks say of it: what the link pass asks of a claiming
/// document those checks refused and of a document a higher pack shadows, neither of which
/// the registry keeps. `None` for a file that does not load.
#[doc(hidden)]
pub fn identifier_in(origin: &Origin, bytes: &[u8], cx: &LoadContext) -> Option<BiomeId> {
    match load(origin.clone(), bytes, cx).ok()?.0 {
        Loaded::Stable(biome) => Some(biome.description.identifier),
        Loaded::Beta(beta) => Some(beta.description.identifier),
    }
}

/// The priority rank of a pack: 0 is the top; the app-data layer is below every pack.
fn rank(pack: &PackRef) -> usize {
    pack.index().unwrap_or(usize::MAX)
}

/// The identifier a contribution claims, where it came from, and whether it is a loaded
/// document (not a claiming one the component checks rejected).
fn claim_of(contribution: &Contribution<ClientBiome>) -> Option<(&BiomeId, &Origin, bool)> {
    match contribution {
        Contribution::Stable(id, document) => Some((id, document.origin(), true)),
        Contribution::Beta(id, document) => Some((id, document.origin(), true)),
        Contribution::Rejected(Some(id), rejected) => Some((id, &rejected.origin, false)),
        Contribution::Rejected(None, _) => None,
    }
}

/// The findings a loaded document carries.
fn findings_of(contribution: &Contribution<ClientBiome>) -> Diagnostics {
    match contribution {
        Contribution::Stable(_, document) => document.diagnostics().clone(),
        Contribution::Beta(_, document) => document.diagnostics().clone(),
        Contribution::Rejected(_, rejected) => rejected.diagnostics.clone(),
    }
}

/// A combine-time finding at a document.
fn notice(code: Code, severity: Severity, origin: &Origin) -> Diagnostic {
    Diagnostic::error(code)
        .severity(severity)
        .at(identifier_path())
        .phase(Phase::Validate)
        .in_file(origin)
}

/// The client's component checks on the document that claims an identifier: a document they
/// reject keeps its claim (`Rejected` with the identifier).
fn check_claim(contribution: Contribution<ClientBiome>) -> Contribution<ClientBiome> {
    match contribution {
        Contribution::Stable(id, document) => {
            match component_checked(document, |biome| &biome.components) {
                Ok(document) => Contribution::Stable(id, document),
                Err(rejected) => Contribution::Rejected(Some(id), rejected),
            }
        }
        Contribution::Beta(id, document) => {
            match component_checked(document, |beta| &beta.components) {
                Ok(document) => Contribution::Beta(id, document),
                Err(rejected) => Contribution::Rejected(Some(id), rejected),
            }
        }
        rejected @ Contribution::Rejected(..) => rejected,
    }
}

/// The registry, in the client's order. The stack is walked top first and the
/// first loaded document of an identifier claims it; only that document's components are
/// checked. The highest pack's document wins whole; when its component checks fail the biome
/// has no client data and the lower packs are still shadowed
/// (`ClientBiomeShadowsLowerPacks`); a document that failed to load claims nothing. A document
/// a higher pack shadows is a `ShadowedDefinition` info and is never checked. A second
/// document of one identifier in one pack is a `DuplicateIdentifier` information: it was read,
/// so the findings of its parse stay, but the client ignores it before any check; the file
/// first by name wins (the crate's own rule: vanilla takes the file its directory walk meets
/// first). The client logs neither the duplicate nor the shadowing (the "Multiple biome
/// documents with same identifier: %s" line belongs to the server's biome loader), so both are
/// informations of the crate's own.
fn combine(parsed: Vec<Contribution<ClientBiome>>, stack: &StackCx<'_>) -> Registry<ClientBiome> {
    let mut builder = RegistryBuilder::new(
        CombineRule::ClaimBeforeValidate {
            claimed: Some(Notice {
                code: Code::from(&core_codes::CLIENT_BIOME_SHADOWS_LOWER_PACKS),
                severity: Severity::Info,
            }),
        },
        CombineRule::top_wins(),
        stack.options,
    );
    // Per identifier, the claiming document: the first, in walk order, of the highest pack
    // that has a loaded one.
    let mut claims: indexmap::IndexMap<BiomeId, (usize, usize)> = indexmap::IndexMap::new();
    for (index, contribution) in parsed.iter().enumerate() {
        if let Some((id, origin, _)) = claim_of(contribution) {
            let rank = rank(&origin.pack);
            let claim = claims.entry(id.clone()).or_insert((rank, index));
            if rank < claim.0 {
                *claim = (rank, index);
            }
        }
    }
    let claiming: IndexSet<usize> = claims.values().map(|&(_, index)| index).collect();
    let mut seen: IndexSet<(usize, BiomeId)> = IndexSet::new();
    let mut findings = Vec::new();
    for (index, contribution) in parsed.into_iter().enumerate() {
        let contribution = if claiming.contains(&index) {
            check_claim(contribution)
        } else {
            contribution
        };
        if let Some((id, origin, loaded)) = claim_of(&contribution) {
            let rank = rank(&origin.pack);
            if !seen.insert((rank, id.clone())) {
                findings.extend(findings_of(&contribution));
                findings.push(notice(
                    Code::from(&core_codes::DUPLICATE_IDENTIFIER),
                    Severity::Info,
                    origin,
                ));
                continue;
            }
            if loaded && !claiming.contains(&index) {
                findings.push(notice(
                    Code::from(&core_codes::SHADOWED_DEFINITION),
                    Severity::Info,
                    origin,
                ));
            }
        }
        builder.push(contribution);
    }
    for finding in findings {
        builder.push_diagnostic(finding);
    }
    builder.finish()
}

/// The loader of `rp.client_biomes()`: `biomes/**/*.json` of every resource pack, the highest
/// pack's document claiming its identifier.
#[doc(hidden)]
pub const LOADER: CollectionLoader<Registry<ClientBiome>, Contribution<ClientBiome>> =
    CollectionLoader {
        walk: WALK,
        parse: parse_walk_file,
        combine,
        needs: &[],
        then: None,
    };

impl Description {
    /// The description of `identifier`.
    pub fn new(identifier: BiomeId) -> Self {
        Self { identifier }
    }
}

impl ClientBiome {
    /// A client biome named `identifier` without components.
    pub fn new(identifier: BiomeId) -> Self {
        Self {
            description: Description::new(identifier),
            components: ComponentMap::new(),
        }
    }

    /// The surface colour (RGBA channels from 0 to 1) and opacity its
    /// `minecraft:water_appearance` sets; `None` for a member it leaves out. The start of the
    /// water surface `biomes_client.json` then overrides.
    pub fn water_surface(&self) -> (Option<[f32; 4]>, Option<f32>) {
        let Some(water) = self.components.get::<WaterAppearance>() else {
            return (None, None);
        };
        let color = water
            .surface_color
            .as_ref()
            .and_then(Color255Rgb::rgb)
            .map(|[r, g, b]| [r, g, b].map(|channel| f32::from(channel) / 255.0))
            .map(|[r, g, b]| [r, g, b, 1.0]);
        (color, water.surface_opacity)
    }
}

impl Beta {
    /// A beta client biome named `identifier` without components or environment attributes.
    pub fn new(identifier: BiomeId) -> Self {
        Self {
            description: Description::new(identifier),
            components: ComponentMap::new(),
            environment_attributes: None,
        }
    }
}

impl SkyColor {
    /// `minecraft:sky_color` with `sky_color`.
    pub fn new(sky_color: Color255Rgb) -> Self {
        Self { sky_color }
    }
}

impl DryFoliageColor {
    /// `minecraft:dry_foliage_color` with `color`.
    pub fn new(color: Color255Rgb) -> Self {
        Self { color }
    }
}

impl SoundAddition {
    /// The sound `asset` at `chance` (from 0 to 1).
    pub fn new(asset: SoundEventName, chance: f32) -> Self {
        Self { asset, chance }
    }
}

#[cfg(test)]
mod reading_table {
    //! The family's reading table: what the pipeline reads each of its files and structs by.

    use bedrock_addon_core::versioning::testing::{assert_reading_table, reading_table};

    #[test]
    fn client_biome_reads_as_pinned() {
        assert_reading_table(
            "ClientBiomeFamily",
            &reading_table::<super::ClientBiomeFamily>(),
            r"
                text: Document, clamps beyond a double: true, duplicates kept in: []
                envelope: From(FormatVersion { major: 1, minor: 21, patch: 130 })
                missing version: rejected: NoValidVersion
                below floor: UseFormatVersion
                empty document: Error
                1.21.40..=1.21.130 (0..=6): json Rejected/Strict/Error/Error/reports/fatal/Refuse, mcb Own/Strict/Error/Error/reports/fatal/Refuse
                beta: json Rejected/Strict/Error/Error/reports/fatal/Refuse, mcb Own/Strict/Error/Error/reports/fatal/Refuse
            ",
        );
    }
}
