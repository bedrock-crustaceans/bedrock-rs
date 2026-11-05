//! Loading client biomes: the family's version-chain parameters, `Content` / `Definition` of
//! the newest and beta documents, vanilla's words for a version or a text the loader refuses,
//! the walk of `biomes/` and the combine rule.

use indexmap::IndexSet;

use super::component::{ClientBiomeComponent, child};
use super::history::{Any, ClientBiomeFamily, identifier_path};
use super::parse::{MODERN, PAYLOAD_KEY};
use super::texts::{
    CANNOT_LOAD, FAILED_TO_LOAD, INVALID_FILE, LEGACY_ROOT, LEGACY_SYNTAX, NO_VERSION_STRING,
    formatted,
};
use super::{Beta, ClientBiome, validate};
use std::borrow::Cow;

use bedrock_addon_core::__private::Sealed;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Phase, Severity, VanillaText,
};
use bedrock_addon_core::document::{
    CombineRule, ComponentMap, Content, Contribution, Definition, Document, JsonContent, NoBeta,
    Notice, Origin, Placed, Registry, RegistryBuilder, Rejected, default_path_flat_for_vanilla,
};
use bedrock_addon_core::id::{BiomeId, kind};
use bedrock_addon_core::json::NullRule;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value, WriteStyle};
use bedrock_addon_core::json::{PrepareError, Prepared};
use bedrock_addon_core::load::{
    Accept, AppDataPlace, CollectionLoader, ListDepth, LoadContext, StackCx, Walk, WalkFile,
    WalkOrder,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput};
use bedrock_addon_core::versioning::{
    ComponentSection, ComponentTable, EnvelopeCheck, ErrorMask, Family, Identity, InputKind,
    Loaded, MissingVersion, Parsed, StructIndex, finish, parse_as_written,
};

/// The directory client-biome files live in.
const DIRECTORY: &str = "biomes";
/// The floor: vanilla's lowest accepted version, and the first client-biome format.
const FLOOR: FormatVersion = FormatVersion::new(1, 21, 40);
/// Where the legacy range of formats starts: the game has no parser at all for a version below
/// it.
const LEGACY_SLICE: FormatVersion = FormatVersion::new(1, 0, 0);
/// The last struct of the legacy range.
const LAST_LEGACY: FormatVersion = FormatVersion::new(1, 21, 120);

impl Family for ClientBiomeFamily {
    // What the document does not read is kept for write-back: a number no double holds is
    // the largest finite one, with a warning at it.
    const CLAMPS_BEYOND_DOUBLE: bool = true;
    const NAME: &'static str = "client_biome";
    const PAYLOAD_KEY: &'static str = PAYLOAD_KEY;
    const DIRECTORIES: &'static [&'static str] = &[DIRECTORY];
    const FLOOR: FormatVersion = FLOOR;
    type Latest = ClientBiome;
    type Beta = Beta;
    type Component = ClientBiomeComponent;
    type Any = Any;
    // Below 1.21.130 the game reads the raw text of the whole document; from 1.21.130 the
    // envelope has exactly two members, `format_version` and `minecraft:client_biome`.
    const ENVELOPE: EnvelopeCheck = EnvelopeCheck::From(MODERN);
    // A file without a `format_version` is rejected: "no valid version passed by the caller or
    // found in the input".
    const MISSING_VERSION: MissingVersion = MissingVersion::Reject;

    // Every format version reads with the strict error mask, the legacy ones, 1.21.130 and the
    // beta document alike: an unknown member or component refuses the document.
    fn error_mask(_: StructIndex) -> ErrorMask {
        ErrorMask::Strict
    }

    // The game reads every file as a document: the version is the first textual
    // `format_version` (a number is read as its text) and the size limits apply.
    const INPUT: InputKind = InputKind::Document;

    // On every format version the game's reader tests no member for `null`, so a member
    // written `null` is present and fails its type.
    fn null_rule(_: StructIndex) -> Option<NullRule> {
        Some(NullRule::Rejected)
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
    ) -> Result<Document<Any>, Rejected> {
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
    document: Document<Any>,
    cx: &LoadContext,
) -> Result<Document<ClientBiome>, Rejected> {
    let origin = document.origin().clone();
    let version = document.version().clone();
    let found = document.diagnostics().clone();
    let value = document.into_value();
    let identity = Identity {
        identifier: Some(identifier_of(&value).to_owned()),
        runtime_identifier: None,
    };
    let written = FormatVersionInput {
        version: (!version.beta).then_some(version.written.unwrap_or(version.effective)),
        beta_literal: version.beta,
        prerelease: false,
    };
    let parsed = Parsed::<ClientBiomeFamily>::new(
        Some(value),
        version,
        written,
        identity,
        bedrock_addon_core::vanilla::ActorType::Mob,
        false,
        child(&JsonPath::new(), &[PAYLOAD_KEY]),
        Diagnostics::default(),
    );
    let outcome = finish(parsed, cx);
    let mut more = outcome.diagnostics;
    more.locate(&origin);
    let mut diagnostics = found;
    diagnostics.extend(more);
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
fn identifier_of(value: &Any) -> &str {
    match value {
        Any::V1_21_40(b) => b.description.identifier.as_str(),
        Any::V1_21_70(b) => b.description.identifier.as_str(),
        Any::V1_21_90(b) => b.description.identifier.as_str(),
        Any::V1_21_100(b) => b.description.identifier.as_str(),
        Any::V1_21_110(b) => b.description.identifier.as_str(),
        Any::V1_21_120(b) => b.description.identifier.as_str(),
        Any::V1_21_130(b) => b.description.identifier.as_str(),
        Any::Beta(b) => b.description.identifier.as_str(),
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
fn no_parser(version: &str) -> DiagnosticCode {
    DiagnosticCode::UnsupportedVersion {
        text: Some(VanillaText::NoParserForVersion {
            version: version.to_owned(),
        }),
    }
}

/// The text of a file, after the BOM, the UTF-8 and the depth check.
fn text<'b>(bytes: &'b [u8], origin: &Origin) -> Result<Cow<'b, str>, Rejected> {
    let malformed = |message: &str| {
        let mut diagnostics = Diagnostics::default();
        diagnostics.push(Diagnostic::error(DiagnosticCode::MalformedInput {
            message: message.to_owned(),
            text: None,
        }));
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
    let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
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
    for mut diagnostic in std::mem::take(&mut parsed.diagnostics) {
        let mut after = None;
        let reworded = match &diagnostic.code {
            // The game clears the input's version and then finds none.
            DiagnosticCode::MalformedFormatVersion { text: None } => {
                Some(DiagnosticCode::MalformedFormatVersion {
                    text: Some(VanillaText::NoValidVersion),
                })
            }
            // Below the floor the game refuses the version in the legacy loader's own words;
            // below the legacy range it has no parser at all.
            DiagnosticCode::UnsupportedVersion {
                text: Some(VanillaText::UseFormatVersion { .. }),
            } => written.filter(|version| *version < FLOOR).map(|version| {
                let version_text = version.to_string();
                if version < LEGACY_SLICE {
                    no_parser(&version_text)
                } else {
                    DiagnosticCode::UnsupportedVersion {
                        text: formatted(CANNOT_LOAD, &[path, &version_text]),
                    }
                }
            }),
            // The game takes the first textual `format_version`: a document below 1.21.130
            // that has it only below its root reaches the legacy loader, which finds none at
            // the root.
            DiagnosticCode::MissingRequiredField { text: None }
                if written.is_some_and(|version| version < MODERN)
                    && diagnostic.location.json_path.is_key("format_version") =>
            {
                Some(DiagnosticCode::MissingRequiredField {
                    text: formatted(NO_VERSION_STRING, &[path]),
                })
            }
            // A text the legacy loader cannot read as a JSON object.
            DiagnosticCode::MalformedInput {
                message,
                text: Some(VanillaText::DocumentJson { .. }),
            } if scanned_version(text).is_some_and(|version| version < MODERN) => {
                let vanilla = if bedrock_addon_core::json::from_str_jsonc::<Value>(text).is_ok() {
                    formatted(LEGACY_ROOT, &[path])
                } else {
                    formatted(LEGACY_SYNTAX, &[message])
                };
                Some(DiagnosticCode::MalformedInput {
                    message: message.clone(),
                    text: vanilla,
                })
            }
            // The content log line that follows "Invalid Client Biome file".
            DiagnosticCode::MalformedInput {
                text: Some(VanillaText::Fixed(INVALID_FILE)),
                ..
            } => {
                after = Some(Diagnostic::error(DiagnosticCode::MalformedInput {
                    message: "the document was not loaded".to_owned(),
                    text: formatted(FAILED_TO_LOAD, &[path]),
                }));
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
        found.push(Diagnostic::info(DiagnosticCode::UnusualFormatVersion).at(at));
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
fn is_json(path: &PackPath) -> bool {
    path.as_str().ends_with(".json")
}

/// Whether vanilla's walk reaches `path` in the pack of `cx`: the game lists `biomes/` of a
/// pack shipped in the app package without descending into sub-folders, and every other pack
/// with its sub-folders.
fn reached(path: &PackPath, cx: &LoadContext) -> bool {
    let nested = path.segments().count() > 2;
    !(cx.is_base_game_pack && nested)
}

/// `biomes/**/*.json` of every resource pack; `biomes/*.json` in a base-game pack.
const WALK: Walk = Walk {
    directories: &[DIRECTORY],
    depth: ListDepth::Recursive,
    accept: Accept::Fn(is_json),
    order: WalkOrder::TopDown,
    directory_major: false,
    app_data: &[],
    app_data_place: AppDataPlace::After,
    in_pack: Some(reached),
    shadowed: None,
    read_error: None,
};

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
fn notice(code: DiagnosticCode, severity: Severity, origin: &Origin) -> Diagnostic {
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
                code: DiagnosticCode::ClientBiomeShadowsLowerPacks,
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
                    DiagnosticCode::DuplicateIdentifier { text: None },
                    Severity::Info,
                    origin,
                ));
                continue;
            }
            if loaded && !claiming.contains(&index) {
                findings.push(notice(
                    DiagnosticCode::ShadowedDefinition { text: None },
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
pub const fn load_registry() -> CollectionLoader<Registry<ClientBiome>, Contribution<ClientBiome>> {
    CollectionLoader {
        walk: WALK,
        parse: parse_walk_file,
        combine,
        needs: &[],
        then: None,
    }
}
