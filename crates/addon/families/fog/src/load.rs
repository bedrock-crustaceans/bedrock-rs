//! Loading fog settings: the gates `registerFogFromJson` runs before either parser, the chain's
//! parameters, the rules after the parse, `Content` / `Definition` / `Placed`, the walk of
//! `fogs/` and the combine rule (per identifier the first registration wins, the stack walked
//! from its top).

use super::texts::{
    EMPTY_AFTER_PARSE, EMPTY_RESOURCE, FAILED_TO_VALIDATE, INVALID_VERSION, NO_IDENTIFIER,
    PARSE_ERROR, UNSUPPORTED_VERSION,
};
use super::{FLOOR, FogSettings, LATEST, rules};
use crate::history::{Any, FogFamily};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, VanillaText,
};
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, JsonContent, NoBeta, Notice, Origin,
    Placed, Registry, RegistryBuilder, Rejected, VersionInfo, default_path,
};
use bedrock_addon_core::id::{FogId, kind};
use bedrock_addon_core::json::{
    EncodeError, EnvelopeVersion, JsonSlice, MAX_JSON_DEPTH, Value, WriteStyle, check_depth,
    last_wins, parse_at, probe_envelope, strip_bom,
};
use bedrock_addon_core::load::{
    Accept, AppDataPlace, CollectionLoader, ListDepth, LoadContext, StackCx, Walk, WalkFile,
    WalkOrder,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput};
use bedrock_addon_core::versioning::{
    EnvelopeCheck, Family, Identity, Loaded, MissingVersion, NoComponents, ParseCx, Parsed,
    RangeModel, StructIndex, VersionChain, finish, parse_as_written,
};
use indexmap::IndexSet;

impl Family for FogFamily {
    const NAME: &'static str = "fog";
    const PAYLOAD_KEY: &'static str = "minecraft:fog_settings";
    const DIRECTORIES: &'static [&'static str] = &["fogs"];
    const FLOOR: FormatVersion = FLOOR;
    // The cereal parser is built with the 2-argument `CerealParserBase(ctx, VERSION)`: exactly
    // 1.26.20, whatever the engine.
    const RANGE_MODEL: RangeModel = RangeModel::ClosedToLatest;
    type Latest = FogSettings;
    type Beta = std::convert::Infallible;
    type Component = NoComponents;
    type Any = Any;
    // Below 1.26.20 the strict schema root reads the whole document (a third root member is an
    // unknown member of it); from 1.26.20 `MinecraftDocumentInput` wants exactly two members.
    // A pre-release of 1.26.20 sorts below it.
    const ENVELOPE: EnvelopeCheck = EnvelopeCheck::From(LATEST);
    const MISSING_VERSION: MissingVersion = MissingVersion::Reject;
    const MISSING_VERSION_TEXT: Option<VanillaText> = Some(VanillaText::Fixed(INVALID_VERSION));
}

/// The payload's path in the file.
fn payload_at() -> JsonPath {
    let mut at = JsonPath::new();
    at.push(FogFamily::PAYLOAD_KEY);
    at
}

/// An error of the gates, at the document root.
fn gate(code: DiagnosticCode) -> Diagnostics {
    let mut diagnostics = Diagnostics::default();
    diagnostics.push(Diagnostic::error(code));
    diagnostics
}

fn malformed(message: String) -> Diagnostics {
    gate(DiagnosticCode::MalformedInput {
        text: Some(VanillaText::Formatted {
            template: PARSE_ERROR,
            args: vec![message.clone()],
        }),
        message,
    })
}

fn empty(text: &'static str) -> Diagnostics {
    gate(DiagnosticCode::EmptyDocument {
        text: Some(VanillaText::Fixed(text)),
    })
}

/// `parseFogDefinitionIdentifier`: `minecraft:fog_settings.description.identifier` of the raw
/// JSON is a non-empty string.
fn has_identifier(root: &Value) -> bool {
    let identifier = root
        .as_object()
        .and_then(|root| root.get(FogFamily::PAYLOAD_KEY)?.as_object())
        .and_then(|settings| settings.get("description")?.as_object())
        .and_then(|description| description.get("identifier")?.as_string());
    identifier.is_some_and(|text| !text.as_str().is_empty())
}

/// Steps 1 to 7 of `registerFogFromJson` on a file's bytes: the text with duplicate members
/// settled, or the findings of the gate that skipped the file.
fn gates(bytes: &[u8]) -> Result<String, Diagnostics> {
    if bytes.is_empty() {
        return Err(empty(EMPTY_RESOURCE));
    }
    let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
    let bytes = strip_bom(bytes);
    check_depth(bytes, MAX_JSON_DEPTH).map_err(|_| malformed("nesting too deep".to_owned()))?;
    let text = std::str::from_utf8(bytes).map_err(|e| malformed(e.to_string()))?;
    // Of a member written twice the last value stands, before anything looks at the text.
    let cleaned = last_wins(text).into_owned();
    let root =
        parse_at::<Value>(JsonSlice::new(&cleaned)).map_err(|e| malformed(e.error.to_string()))?;
    // `Json::Value::empty()`: null, or an array or object without members.
    let is_empty = root.is_null()
        || root.as_object().is_some_and(facet_value::VObject::is_empty)
        || root.as_array().is_some_and(facet_value::VArray::is_empty);
    if is_empty {
        return Err(empty(EMPTY_AFTER_PARSE));
    }
    if !has_identifier(&root) {
        return Err(gate(DiagnosticCode::MissingIdentifier {
            text: Some(VanillaText::Fixed(NO_IDENTIFIER)),
        }));
    }
    let invalid = || {
        gate(DiagnosticCode::MissingFormatVersion {
            text: Some(VanillaText::Fixed(INVALID_VERSION)),
        })
    };
    let envelope = probe_envelope(JsonSlice::new(&cleaned)).map_err(|_| invalid())?;
    match &envelope.format_version {
        EnvelopeVersion::String(written) => {
            let version = FormatVersionInput::parse(written).ok_or_else(invalid)?;
            // The literal "beta" goes on to the chain, which has no beta struct.
            let below = !version.beta_literal
                && version
                    .version
                    .is_none_or(|v| v < FLOOR || (version.prerelease && v == FLOOR));
            if below {
                return Err(gate(DiagnosticCode::UnsupportedVersion {
                    text: Some(VanillaText::Fixed(UNSUPPORTED_VERSION)),
                }));
            }
        }
        // V5a (ours): the dispatcher takes a three-element array, and then either parser
        // refuses it; it is refused here.
        EnvelopeVersion::Array(_) => {
            return Err(gate(DiagnosticCode::MalformedFormatVersion { text: None }));
        }
        _ => return Err(invalid()),
    }
    Ok(cleaned)
}

/// The closing line of a document either parser refused.
fn failed_to_validate(diagnostics: &mut Diagnostics) {
    diagnostics.push(Diagnostic::error(DiagnosticCode::DocumentRejected {
        text: Some(VanillaText::Fixed(FAILED_TO_VALIDATE)),
    }));
}

/// The upgrade walk, then what the game checks on the definition: the keyframe gate and, for a
/// document read at 1.26.20, the cereal parser's validation rules, either failing with the
/// closing line; then the namespace rule.
fn finish_parsed(
    parsed: Parsed<FogFamily>,
    cx: &LoadContext,
) -> (Option<FogSettings>, VersionInfo, Diagnostics) {
    let outcome = finish(parsed, cx);
    let version = outcome.version;
    let mut diagnostics = outcome.diagnostics;
    let Some(Loaded::Stable(settings)) = outcome.value else {
        // The literal "beta" never reached a parser: "Invalid 'format_version' …" in the game.
        if !version.beta {
            failed_to_validate(&mut diagnostics);
        }
        return (None, version, diagnostics);
    };
    let at = payload_at();
    rules::keyframe_gate(&settings, cx.engine_version, &at, &mut diagnostics);
    if version.parsed_as == LATEST {
        rules::format_version(
            &version.effective.to_string(),
            &JsonPath::new(),
            &mut diagnostics,
        );
    }
    // V15–V20 are every era's: the 1.26.20 validation lambda's, and the strict schema's
    // typed nodes' own (the same nodes in all three older schemas). There V15 and V17–V20 are
    // level 3 and fail the validation, so the file is refused through "Failed to validate fog
    // schema" below, and V16–V16b are level 2.
    rules::cereal_rules(&settings, &at, &mut diagnostics);
    if diagnostics.has_errors() {
        failed_to_validate(&mut diagnostics);
        return (None, version, diagnostics);
    }
    rules::namespace(
        settings.description.identifier.as_str(),
        cx.is_base_game_pack,
        &at,
        &mut diagnostics,
    );
    let settings = (!diagnostics.has_errors()).then_some(settings);
    (settings, version, diagnostics)
}

/// Loads one fog file of `pack`.
pub(crate) fn parse_file(
    pack: PackRef,
    side: PackSide,
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> Result<Document<FogSettings>, Rejected> {
    let origin = Origin::file(pack, side, path.clone());
    let text = match gates(bytes) {
        Ok(text) => text,
        Err(diagnostics) => {
            return Err(Rejected::located(origin, diagnostics));
        }
    };
    let parsed = as_written(&text, cx);
    let (settings, version, diagnostics) = finish_parsed(parsed, cx);
    let diagnostics = diagnostics.located(&origin);
    match settings {
        Some(settings) => Ok(Document::new(settings, origin, version, diagnostics)),
        None => Err(Rejected::new(origin, diagnostics)),
    }
}

/// The struct of the chain that reads the 1.21.90 root.
const V1_21_90: usize = 2;

/// The parse stage of a text the gates let through: the chain's, except for a pre-release of
/// 1.26.20. The game's dispatcher compares with a plain `<`, so `1.26.20-beta.1` is below
/// 1.26.20 and read by the 1.21.90 root; the chain's range model, built for the game's other loaders,
/// covers no pre-release of its newest struct, so that one case is routed here.
fn as_written(text: &str, cx: &LoadContext) -> Parsed<FogFamily> {
    let document = JsonSlice::new(text);
    let written = match probe_envelope(document).map(|envelope| envelope.format_version) {
        Ok(EnvelopeVersion::String(written)) => FormatVersionInput::parse(&written)
            .filter(|input| input.prerelease && input.version == Some(LATEST))
            .map(|input| (written, input)),
        _ => None,
    };
    let Some((written, input)) = written else {
        return parse_as_written::<FogFamily>(document, cx);
    };
    let mut diagnostics = Diagnostics::default();
    let mut at = JsonPath::new();
    at.push("format_version");
    diagnostics.push(
        Diagnostic::info(DiagnosticCode::PrereleaseFormatVersion { version: written }).at(at),
    );
    let versions = <Any as VersionChain<FogFamily>>::VERSIONS;
    let parsed_as = versions.get(V1_21_90).copied().unwrap_or(FLOOR);
    let parse = ParseCx {
        load: cx,
        written: input,
        original_version: LATEST,
        at: JsonPath::new(),
    };
    let value = <Any as VersionChain<FogFamily>>::parse_struct(
        document,
        StructIndex::Stable(V1_21_90),
        &parse,
        &mut diagnostics,
    )
    .filter(|_| !diagnostics.has_errors());
    Parsed::new(
        value,
        VersionInfo::new(Some(LATEST), false, LATEST, parsed_as, false),
        input,
        Identity::default(),
        ActorType::Mob,
        false,
        JsonPath::new(),
        diagnostics,
    )
}

impl bedrock_addon_core::__private::Sealed for FogSettings {}

impl JsonContent for FogSettings {}

impl Content for FogSettings {
    const NAME: &'static str = "fog settings";
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        parse_file(PackRef::app_data(), PackSide::Resource, path, bytes, cx)
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        self.write_into(out, style)
    }
}

impl Definition for FogSettings {
    type Kind = kind::Fog;
    type File = FogSettings;
    type Beta = NoBeta<kind::Fog>;
}

impl Placed for FogSettings {
    /// `fogs/<identifier>.json`, the identifier's `:` written `_`
    /// (`fogs/minecraft_fog_bamboo_jungle.json`).
    fn default_path(&self) -> PackPath {
        default_path("fogs", &self.description.identifier)
    }
}

impl FogSettings {
    /// Parses one fog file without upgrading it: the struct its `format_version` selects;
    /// [`history::upgrade`](crate::history::upgrade) finishes it.
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
        let origin = Origin::file(PackRef::app_data(), PackSide::Resource, path.clone());
        let text = match gates(bytes) {
            Ok(text) => text,
            Err(diagnostics) => {
                return Err(Rejected::located(origin, diagnostics));
            }
        };
        let parsed = as_written(&text, cx);
        let mut diagnostics = parsed.diagnostics;
        if let Some(any) = parsed.value {
            return Ok(Document::new(
                any,
                origin.clone(),
                parsed.version,
                diagnostics.located(&origin),
            ));
        }
        if !parsed.version.beta {
            failed_to_validate(&mut diagnostics);
        }
        Err(Rejected::located(origin, diagnostics))
    }
}

/// `history::upgrade`: the remaining steps and the rules on a document
/// [`FogSettings::parse_as_written`] gave.
///
/// # Errors
///
/// Returns [`Rejected`] when an upgrade step or a check refuses the document; it carries every
/// finding about the file.
pub fn upgrade_document(
    document: Document<Any>,
    cx: &LoadContext,
) -> Result<Document<FogSettings>, Rejected> {
    let origin = document.origin().clone();
    let version = document.version().clone();
    let diagnostics = document.diagnostics().clone();
    let at = if version.parsed_as == LATEST {
        payload_at()
    } else {
        JsonPath::new()
    };
    let written = FormatVersionInput {
        version: Some(version.effective),
        beta_literal: false,
        prerelease: false,
    };
    let parsed = Parsed::<FogFamily>::new(
        Some(document.into_value()),
        version,
        written,
        Identity::default(),
        ActorType::Mob,
        false,
        at,
        diagnostics,
    );
    let (settings, version, diagnostics) = finish_parsed(parsed, cx);
    // The findings the document carried are located already; the new ones are not.
    let diagnostics = diagnostics.located(&origin);
    match settings {
        Some(settings) => Ok(Document::new(settings, origin, version, diagnostics)),
        None => Err(Rejected::new(origin, diagnostics)),
    }
}

// ---- the collection ----------------------------------------------------------------------------

/// A fog file: a `.json` file directly in `fogs/`.
fn is_json(path: &PackPath) -> bool {
    path.as_str()
        .rsplit_once('.')
        .is_some_and(|(_, extension)| extension == "json")
}

/// `fogs/*.json`, packs top-down: the game walks its stack from index 0 and the first file to
/// register an identifier keeps it.
// The game collects them with `getResourcesOfGroup(pack, "fogs")`.
const WALK: Walk = Walk {
    directories: &["fogs"],
    depth: ListDepth::Children,
    accept: Accept::Fn(is_json),
    order: WalkOrder::TopDown,
    directory_major: false,
    app_data: &[],
    app_data_place: AppDataPlace::After,
    in_pack: None,
    shadowed: None,
    read_error: None,
};

fn parse_walk_file(file: &WalkFile, cx: &LoadContext) -> Contribution<FogSettings> {
    match parse_file(file.pack.clone(), file.side, &file.path, &file.bytes, cx) {
        Ok(document) => Contribution::Stable(document.description.identifier.clone(), document),
        Err(rejected) => Contribution::Rejected(None, rejected),
    }
}

/// The registry: per identifier the highest pack's definition; each lower one is noted with
/// `ShadowedDefinition` at info (ours; the game skips it silently). A file that failed
/// registers nothing, so a lower pack's definition of its identifier still does.
fn combine(parsed: Vec<Contribution<FogSettings>>, stack: &StackCx<'_>) -> Registry<FogSettings> {
    let mut builder = RegistryBuilder::new(
        CombineRule::TopWins {
            shadowed: Some(Notice::shadowed()),
        },
        CombineRule::top_wins(),
        stack.options,
    );
    // Inside one pack the first file to register an identifier keeps it too (the walk gives a
    // pack's files in path order): a later one is skipped with the same info, and keeps the
    // findings of its parse, which the game ran before it looked the identifier up.
    let mut registered: IndexSet<(Option<usize>, FogId)> = IndexSet::new();
    for contribution in parsed {
        if let Contribution::Stable(id, document) = &contribution {
            let origin = document.origin();
            if !registered.insert((origin.pack.index(), id.clone())) {
                for finding in document.diagnostics().iter().cloned() {
                    builder.push_diagnostic(finding);
                }
                let notice = Notice::shadowed();
                builder.push_diagnostic(Diagnostic::new(
                    notice.code,
                    notice.severity,
                    Phase::Validate,
                    Location::new(
                        Some(origin.pack.clone()),
                        Some(origin.path.clone()),
                        JsonPath::new(),
                        None,
                    ),
                ));
                continue;
            }
        }
        builder.push(contribution);
    }
    builder.finish()
}

/// The loader of `rp.fogs()`: `fogs/*.json` of every resource pack, the highest pack's
/// definition winning per identifier.
pub const fn load_registry() -> CollectionLoader<Registry<FogSettings>, Contribution<FogSettings>> {
    CollectionLoader {
        walk: WALK,
        parse: parse_walk_file,
        combine,
        needs: &[],
        then: None,
    }
}
