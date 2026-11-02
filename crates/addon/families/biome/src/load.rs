//! Loading biomes: the family's version-chain parameters, `Content` / `Definition` of the
//! newest and beta documents, the world gate, the below-floor findings, the walk of `biomes/`
//! and the combine rule.

use facet_json::RawJson;
use indexmap::IndexMap;

use super::component::{BiomeComponent, child};
use super::history::{Any, BiomeFamily};
use super::parse::CEREAL;
use super::texts::{DUPLICATE, WORLD_GATE, fixed, text as formatted};
use super::{Beta, Biome, validate};
use bedrock_addon_core::__private::Sealed;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity, VanillaText,
};
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, NoBeta, Origin, Placed, Registry,
    RegistryBuilder, Rejected, default_path_flat_for_vanilla,
};
use bedrock_addon_core::id::{BiomeId, kind};
use bedrock_addon_core::json::NullRule;
use bedrock_addon_core::json::{
    EncodeError, JsonSlice, MAX_JSON_DEPTH, Value, WriteStyle, check_depth, strip_bom,
};
use bedrock_addon_core::load::{
    AppDataPlace, CollectionLoader, ListDepth, LoadContext, StackCx, Walk, WalkFile, WalkOrder,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput, SemVersion};
use bedrock_addon_core::versioning::{
    ComponentSection, ComponentTable, EnvelopeCheck, ErrorMask, Family, Identity, InputKind,
    Loaded, MissingVersion, Parsed, StructIndex, ValidateCx, finish, member_version,
    parse_as_written,
};

/// The directory biome files live in.
const DIRECTORY: &str = "biomes";
/// The floor: the first format vanilla loads; every earlier format was experimental.
const FLOOR: FormatVersion = FormatVersion::new(1, 21, 110);
/// The world version a non-base-game pack's biomes need, as the world-gate message says.
const WORLD_GATE_VERSION: SemVersion = SemVersion::new(1, 21, 110);
/// The first experimental format the below-floor hint names.
const EXPERIMENTAL_FORMATS: FormatVersion = FormatVersion::new(1, 13, 0);

impl Family for BiomeFamily {
    // What the document does not read is kept for write-back: a number no double holds is
    // the largest finite one, with a warning at it.
    const CLAMPS_BEYOND_DOUBLE: bool = true;
    const NAME: &'static str = "biome";
    const PAYLOAD_KEY: &'static str = "minecraft:biome";
    const DIRECTORIES: &'static [&'static str] = &[DIRECTORY];
    const FLOOR: FormatVersion = FLOOR;
    type Latest = Biome;
    type Beta = Beta;
    type Component = BiomeComponent;
    type Any = Any;
    // Below 1.21.120 the game reads the raw text of the whole document; from 1.21.120 the
    // envelope has exactly two members, `format_version` and `minecraft:biome`.
    const ENVELOPE: EnvelopeCheck = EnvelopeCheck::From(CEREAL);
    // A file without a `format_version` is rejected: "no valid version passed by the caller or
    // found in the input".
    const MISSING_VERSION: MissingVersion = MissingVersion::Reject;

    // Every format version reads with the strict error mask, the beta document included: an
    // unknown member or component refuses the document.
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
        BiomeComponent::TABLE
    }

    /// The 1.12 shape (`{ "<name>": { "format_version": …, … } }`) holds its version inside
    /// its only root member; it is read so the document is rejected as below the floor.
    // The game reports "no parser available for version 1.12.0" for such a file.
    fn payload_version(payload: JsonSlice<'_>) -> Option<FormatVersionInput> {
        let only = only_member(payload)?;
        member_version(only, "format_version")
    }

    fn component_sections(v: &mut Biome) -> Vec<ComponentSection<'_, BiomeComponent>> {
        let mut at = JsonPath::new();
        at.push("components");
        vec![ComponentSection::new(at, false, &mut v.components)]
    }

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        match v {
            Loaded::Stable(biome) => {
                validate::validate(&biome.description.identifier, &mut biome.components, cx);
            }
            Loaded::Beta(beta) => {
                validate::validate(&beta.description.identifier, &mut beta.components, cx);
            }
        }
    }
}

/// The only member of a root without `minecraft:biome`, as a slice.
fn only_member(document: JsonSlice<'_>) -> Option<JsonSlice<'_>> {
    let members =
        bedrock_addon_core::json::from_str_borrowed_jsonc::<IndexMap<String, RawJson<'_>>>(
            document.text,
        )
        .ok()?;
    if members.len() != 1 || members.contains_key("minecraft:biome") {
        return None;
    }
    let (_, raw) = members.first()?;
    document.nested(raw)
}

impl Sealed for Biome {}

impl Content for Biome {
    const NAME: &'static str = "biome";
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    /// Parses one biome file, upgraded to the newest struct. A beta document is rejected here
    /// (it is a [`Beta`]).
    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        let origin = origin(PackRef::app_data(), PackSide::Behavior, path);
        match load(origin, bytes, cx).1? {
            (Loaded::Stable(biome), document) => Ok(document.with(biome)),
            (Loaded::Beta(_), document) => Err(beta_rejected(document)),
        }
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        self.write_into(out, style)
    }

    fn to_json(&self) -> Result<String, EncodeError> {
        let mut out = Vec::new();
        self.write_into(&mut out, WriteStyle::default())?;
        Ok(String::from_utf8(out).unwrap_or_default())
    }
}

impl Definition for Biome {
    type Kind = kind::Biome;
    type File = Biome;
    type Beta = Beta;
}

impl Placed for Biome {
    /// `biomes/<name>.json` for a `minecraft:` identifier (the `/` of the name as `_`), which
    /// a base-game pack's flat walk reads back; `biomes/<namespace>/<name>.json` for any
    /// other.
    fn default_path(&self) -> PackPath {
        default_path_flat_for_vanilla(DIRECTORY, &self.description.identifier)
    }
}

impl Sealed for Beta {}

impl Content for Beta {
    const NAME: &'static str = "beta biome";
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    /// Parses one beta biome file (`"format_version": "beta"`); any other version is rejected.
    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        let origin = origin(PackRef::app_data(), PackSide::Behavior, path);
        match load(origin, bytes, cx).1? {
            (Loaded::Beta(beta), document) => Ok(document.with(beta)),
            (Loaded::Stable(_), document) => {
                let Parts {
                    origin,
                    mut diagnostics,
                    ..
                } = document;
                diagnostics.push(file_error(DiagnosticCode::UnsupportedVersion {
                    text: None,
                }));
                locate(&mut diagnostics, &origin);
                Err(Rejected::new(origin, diagnostics))
            }
        }
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        self.write_into(out, style)
    }

    fn to_json(&self) -> Result<String, EncodeError> {
        let mut out = Vec::new();
        self.write_into(&mut out, WriteStyle::default())?;
        Ok(String::from_utf8(out).unwrap_or_default())
    }
}

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

impl Biome {
    /// Parses one biome file without upgrading it: the struct its `format_version` selects
    /// with the findings of the parse.
    ///
    /// # Errors
    ///
    /// Returns [`Rejected`] when the file is not text, is not JSON, or its findings include an
    /// error (the findings are in the [`Rejected`]).
    pub fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> Result<Document<Any>, Rejected> {
        let origin = origin(PackRef::app_data(), PackSide::Behavior, path);
        let (_, parsed) = parse(&origin, bytes, cx)?;
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
) -> Result<Document<Biome>, Rejected> {
    let origin = document.origin().clone();
    let version = document.version().clone();
    let found = document.diagnostics().clone();
    let value = document.into_value();
    let identity = Identity {
        identifier: Some(identifier_of(&value).to_owned()),
        runtime_identifier: None,
    };
    let at = child(&JsonPath::new(), &["minecraft:biome"]);
    let written = FormatVersionInput {
        version: (!version.beta).then_some(version.written.unwrap_or(version.effective)),
        beta_literal: version.beta,
        prerelease: false,
    };
    let parsed = Parsed::<BiomeFamily>::new(
        Some(value),
        version,
        written,
        identity,
        bedrock_addon_core::vanilla::ActorType::Mob,
        false,
        at,
        Diagnostics::default(),
    );
    let outcome = finish(parsed, cx);
    let mut diagnostics = found;
    let mut more = outcome.diagnostics;
    locate(&mut more, &origin);
    diagnostics.extend(more);
    match outcome.value {
        Some(Loaded::Stable(biome)) => {
            Ok(Document::new(biome, origin, outcome.version, diagnostics))
        }
        Some(Loaded::Beta(_)) => {
            let mut finding = file_error(DiagnosticCode::UnsupportedVersion {
                text: Some(VanillaText::NoParserForVersion {
                    version: "beta".to_owned(),
                }),
            });
            finding.location.pack = Some(origin.pack.clone());
            finding.location.path = Some(origin.path.clone());
            diagnostics.push(finding);
            Err(Rejected::new(origin, diagnostics))
        }
        None => Err(Rejected::new(origin, diagnostics)),
    }
}

/// The identifier of a parsed document, as written.
fn identifier_of(value: &Any) -> &str {
    match value {
        Any::V1_21_110(b) => b.description.identifier.as_str(),
        Any::V1_21_120(b) => b.description.identifier.as_str(),
        Any::V1_26_0(b) => b.description.identifier.as_str(),
        Any::V1_26_30(b) => b.description.identifier.as_str(),
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
}

/// The text of a file, after the BOM and the depth check.
fn text<'b>(bytes: &'b [u8], origin: &Origin) -> Result<&'b str, Rejected> {
    let malformed = |message: &str| {
        reject(
            origin.clone(),
            file_error(DiagnosticCode::MalformedInput {
                message: message.to_owned(),
                text: None,
            }),
        )
    };
    let bytes = strip_bom(bytes);
    let text = std::str::from_utf8(bytes).map_err(|_| malformed("the file is not UTF-8"))?;
    check_depth(bytes, MAX_JSON_DEPTH).map_err(|_| malformed("the JSON nests too deeply"))?;
    Ok(text)
}

/// The world gate: outside a base-game pack, biomes need a world of base-game version
/// 1.21.110 or later. A context without the world's version passes.
fn world_gate(cx: &LoadContext) -> Option<Diagnostic> {
    (!cx.is_base_game_pack && !cx.world_base_game_version.includes(&WORLD_GATE_VERSION)).then(
        || {
            file_error(DiagnosticCode::IgnoredFile {
                text: fixed(WORLD_GATE),
            })
        },
    )
}

/// The parse stage of one file: the world gate, then the chain's `parse_as_written`, with the
/// below-floor findings in vanilla's words. Also the identifier the payload names.
fn parse(
    origin: &Origin,
    bytes: &[u8],
    cx: &LoadContext,
) -> Result<(Option<BiomeId>, Parsed<BiomeFamily>), Rejected> {
    let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
    let text = text(bytes, origin)?;
    if let Some(gate) = world_gate(cx) {
        return Err(reject(origin.clone(), gate));
    }
    let mut parsed = parse_as_written::<BiomeFamily>(JsonSlice::new(text), cx);
    // The payload is under `minecraft:biome` on the raw-text path too: the checks after the
    // parse report below it.
    parsed.at = child(&JsonPath::new(), &["minecraft:biome"]);
    below_floor(&mut parsed, text);
    let identifier = parsed.identity.identifier.clone().map(BiomeId::parse);
    Ok((identifier, parsed))
}

/// One file through the chain: the identifier it names, and the loaded value with its parts
/// or the rejection.
#[allow(clippy::type_complexity)]
fn load(
    origin: Origin,
    bytes: &[u8],
    cx: &LoadContext,
) -> (
    Option<BiomeId>,
    Result<(Loaded<BiomeFamily>, Parts), Rejected>,
) {
    let (identifier, parsed) = match parse(&origin, bytes, cx) {
        Ok(parsed) => parsed,
        Err(rejected) => return (None, Err(rejected)),
    };
    let outcome = finish(parsed, cx);
    let mut diagnostics = outcome.diagnostics;
    locate(&mut diagnostics, &origin);
    let result = match outcome.value {
        Some(value) => Ok((
            value,
            Parts {
                origin,
                version: outcome.version,
                diagnostics,
            },
        )),
        None => Err(Rejected::new(origin, diagnostics)),
    };
    (identifier, result)
}

/// Below the floor vanilla has no parser: the floor's generic text becomes "unsupported
/// version: no parser available for version %s", followed by the crate's own hint for the old
/// formats.
fn below_floor(parsed: &mut Parsed<BiomeFamily>, text: &str) {
    let Some(written) = parsed.version.written.filter(|v| *v < FLOOR) else {
        return;
    };
    let mut found = Diagnostics::default();
    for mut diagnostic in std::mem::take(&mut parsed.diagnostics) {
        let floor = matches!(
            diagnostic.code,
            DiagnosticCode::UnsupportedVersion {
                text: Some(VanillaText::UseFormatVersion { .. })
            }
        );
        if floor {
            diagnostic.code = DiagnosticCode::UnsupportedVersion {
                text: Some(VanillaText::NoParserForVersion {
                    version: written.to_string(),
                }),
            };
            found.push(diagnostic);
            if let Some(hint) = hint(text, written) {
                found.push(Diagnostic::new(
                    hint,
                    Severity::Info,
                    Phase::Parse,
                    Location::new(None, None, JsonPath::new(), None),
                ));
            }
        } else {
            found.push(diagnostic);
        }
    }
    parsed.diagnostics = found;
}

/// The crate's own hint for a document of an old format: the 1.12 name-keyed shape, or an
/// experimental format from 1.13.0 (with the surface sentence when the 1.20.60-era surface
/// components are used).
fn hint(text: &str, version: FormatVersion) -> Option<DiagnosticCode> {
    let root = bedrock_addon_core::json::from_str_jsonc::<Value>(
        &bedrock_addon_core::json::last_wins(text),
    )
    .ok()?;
    let root = root.as_object()?;
    let biome = root.get("minecraft:biome");
    if biome.is_none() && root.len() == 1 {
        let only = root.iter().next().map(|(_, v)| v)?;
        if only
            .as_object()
            .is_some_and(|o| o.get("format_version").is_some())
        {
            return Some(DiagnosticCode::BiomeFormatHint {
                version: None,
                surface_components: false,
            });
        }
    }
    if version < EXPERIMENTAL_FORMATS {
        return None;
    }
    // `minecraft:surface_parameters` and `minecraft:*_surface`.
    let surface_components = biome
        .and_then(Value::as_object)
        .and_then(|b| b.get("components"))
        .and_then(Value::as_object)
        .is_some_and(|components| {
            components.iter().any(|(key, _)| {
                let key = key.as_str();
                key == "minecraft:surface_parameters"
                    || (key.starts_with("minecraft:") && key.ends_with("_surface"))
            })
        });
    Some(DiagnosticCode::BiomeFormatHint {
        version: Some(version),
        surface_components,
    })
}

/// The rejection of a beta document by the stable document's `parse`.
fn beta_rejected(parts: Parts) -> Rejected {
    let Parts {
        origin,
        mut diagnostics,
        ..
    } = parts;
    let mut finding = file_error(DiagnosticCode::UnsupportedVersion {
        text: Some(VanillaText::NoParserForVersion {
            version: "beta".to_owned(),
        }),
    });
    finding.location.pack = Some(origin.pack.clone());
    finding.location.path = Some(origin.path.clone());
    diagnostics.push(finding);
    Rejected::new(origin, diagnostics)
}

/// A file-level error.
fn file_error(code: DiagnosticCode) -> Diagnostic {
    Diagnostic::new(
        code,
        Severity::Error,
        Phase::Parse,
        Location::new(None, None, JsonPath::new(), None),
    )
}

/// A rejection with one finding.
fn reject(origin: Origin, finding: Diagnostic) -> Rejected {
    let mut diagnostics = Diagnostics::default();
    diagnostics.push(finding);
    locate(&mut diagnostics, &origin);
    Rejected::new(origin, diagnostics)
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

// ---- the collection ----------------------------------------------------------------------

/// A biome file: a `.json` file below `biomes/`.
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

/// `biomes/**/*.json`, packs bottom-up; `biomes/*.json` in a base-game pack.
const WALK: Walk = Walk {
    directories: &[DIRECTORY],
    depth: ListDepth::Recursive,
    accept: is_json,
    order: WalkOrder::BottomUp,
    directory_major: false,
    app_data: &[],
    app_data_place: AppDataPlace::After,
    in_pack: Some(reached),
    shadowed: None,
    read_error: None,
};

/// One file's contribution.
fn parse_walk_file(file: &WalkFile, cx: &LoadContext) -> Contribution<Biome> {
    let origin = origin(file.pack.clone(), file.side, &file.path);
    match load(origin, &file.bytes, cx) {
        (_, Ok((Loaded::Stable(biome), parts))) => {
            let id = biome.description.identifier.clone();
            Contribution::Stable(id, parts.with(biome))
        }
        (_, Ok((Loaded::Beta(beta), parts))) => {
            let id = beta.description.identifier.clone();
            Contribution::Beta(id, parts.with(beta))
        }
        (id, Err(rejected)) => Contribution::Rejected(id, rejected),
    }
}

/// The registry. Vanilla keeps the first document of an identifier and drops later ones;
/// with the stack walked top first that is the highest-priority pack's, so the rule is
/// `TopWins`. A dropped document from a pack other than a base-game pack is vanilla's error
/// "Multiple biome documents with same identifier: %s".
fn combine(parsed: Vec<Contribution<Biome>>, stack: &StackCx<'_>) -> Registry<Biome> {
    let mut duplicates = Vec::new();
    {
        // The highest priority (lowest rank) loaded per identifier, stable and beta alike.
        let mut top: IndexMap<&BiomeId, usize> = IndexMap::new();
        let loaded = parsed.iter().filter_map(|c| match c {
            Contribution::Stable(id, d) => Some((id, d.origin())),
            Contribution::Beta(id, d) => Some((id, d.origin())),
            Contribution::Rejected(..) => None,
        });
        let loaded: Vec<_> = loaded.collect();
        for (id, origin) in &loaded {
            let rank = rank(&origin.pack);
            top.entry(id)
                .and_modify(|r| *r = (*r).min(rank))
                .or_insert(rank);
        }
        for (id, origin) in &loaded {
            let dropped = top.get(id).is_some_and(|&r| rank(&origin.pack) > r);
            if dropped && !is_base_game(stack, &origin.pack) {
                duplicates.push(Diagnostic::new(
                    DiagnosticCode::DuplicateIdentifier {
                        text: formatted(DUPLICATE, &[id.as_str()]),
                    },
                    Severity::Error,
                    Phase::Validate,
                    Location::new(
                        Some(origin.pack.clone()),
                        Some(origin.path.clone()),
                        child(
                            &JsonPath::new(),
                            &["minecraft:biome", "description", "identifier"],
                        ),
                        None,
                    ),
                ));
            }
        }
    }
    let mut builder = RegistryBuilder::new(
        CombineRule::top_wins(),
        CombineRule::top_wins(),
        stack.options,
    );
    for contribution in parsed {
        builder.push(contribution);
    }
    for finding in duplicates {
        builder.push_diagnostic(finding);
    }
    builder.finish()
}

/// The priority rank of a pack: 0 is the top; the app-data layer is below every pack.
fn rank(pack: &PackRef) -> usize {
    pack.index().unwrap_or(usize::MAX)
}

/// Whether `pack` is a base-game pack of the stack.
fn is_base_game(stack: &StackCx<'_>, pack: &PackRef) -> bool {
    pack.index()
        .and_then(|i| stack.packs.get(i))
        .is_some_and(|p| p.context.is_base_game_pack)
}

/// The loader of `bp.biomes()`: `biomes/**/*.json` of every behaviour pack, combined top pack
/// wins.
#[must_use]
pub const fn load_registry() -> CollectionLoader<Registry<Biome>, Contribution<Biome>> {
    CollectionLoader {
        walk: WALK,
        parse: parse_walk_file,
        combine,
        needs: &[],
        then: None,
    }
}
