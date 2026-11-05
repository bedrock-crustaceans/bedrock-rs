//! Loading client entities: the family's version chain parameters, the version gate of
//! vanilla's loader, `Content` / `Definition`, the walk of `entity/` and the
//! combine by `description.min_engine_version`.

use bedrock_addon_core::__private::Sealed;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity, VanillaText,
};
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, EngineVersionDecision, NoBeta,
    Origin, Placed, Registry, RegistryBuilder, Rejected, default_path,
};
use bedrock_addon_core::id::{EntityId, kind};
use bedrock_addon_core::json::{
    EncodeError, JsonSlice, MAX_JSON_DEPTH, Value, WriteStyle, check_depth, strip_bom,
};
use bedrock_addon_core::load::{
    Accept, AppDataPlace, CollectionLoader, ListDepth, LoadContext, StackCx, Walk, WalkFile,
    WalkOrder,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput, SemVersion};
use bedrock_addon_core::versioning::{
    ComponentTable, EnvelopeCheck, Family, Identity, Loaded, NoComponents, Parsed, ValidateCx,
    finish, parse_as_written,
};
use bedrock_addon_core::visual::{MergeStep, SharedDescription, merge_slots, merge_step};

use super::history::{Any, ClientEntityFamily};
use super::read::{self, text};
use super::{ClientEntity, validate};

/// The directory client entity files live in.
const DIRECTORY: &str = "entity";

/// The floor: every explicit version below 1.8.0 is rejected by vanilla's own schema
const FLOOR: FormatVersion = FormatVersion::new(1, 8, 0);

/// Vanilla's warning for a document without `format_version`, with the crate's floor in place
/// of its 1.0.0.
const VERSION_NOT_GIVEN: &str = "Warning: The \"format_version\" field was not given. Fields from format version 1.8.0 and older will be loaded. Update your targeted \"format_version\" to use fields from newer versions.";
/// The message for a version that does not read, or one below 1.0.0.
const UNKNOWN_VERSION: &str =
    "Unknown/unsupported version value specified in format_version field in {}";
/// The minimum that message names: the `v1_10` schema's.
const SCHEMA_MINIMUM: &str = "1.10.0";
/// The message for a root that is null, no object or empty.
const NOTHING_TO_PARSE: &str = "nothing to parse in";

impl Family for ClientEntityFamily {
    const NAME: &'static str = "client_entity";
    const PAYLOAD_KEY: &'static str = read::PAYLOAD;
    const DIRECTORIES: &'static [&'static str] = &[DIRECTORY];
    const FLOOR: FormatVersion = FLOOR;
    type Latest = ClientEntity;
    type Beta = std::convert::Infallible;
    type Component = NoComponents;
    type Any = Any;
    // Vanilla's root schema declares `format_version`, `minecraft:client_entity` and
    // `minecraft:attachable`; a third member is "child '%s' not valid here.", not an envelope
    // error, so each struct reads the whole document.
    const ENVELOPE: EnvelopeCheck = EnvelopeCheck::Never;
    // The game logs this at level 2, the class the pack report shows as a warning: the default
    // `missing_version_severity`.
    const MISSING_VERSION_TEXT: Option<VanillaText> = Some(VanillaText::Fixed(VERSION_NOT_GIVEN));

    fn components() -> &'static ComponentTable<Self> {
        &ComponentTable::EMPTY
    }

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(entity) = v {
            validate::validate(entity, cx);
        }
    }
}

impl Sealed for ClientEntity {}

impl Content for ClientEntity {
    const NAME: &'static str = "client entity";
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// Parses one client entity file, upgraded to the newest shape. The rules between files
    /// (which definition of an identifier wins, by `description.min_engine_version`) apply to
    /// a stack, not to a single file.
    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        let root = root_of(bytes);
        parse_file(
            PackRef::app_data(),
            PackSide::Resource,
            path,
            bytes,
            root.as_ref(),
            cx,
        )
        .1
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

impl Definition for ClientEntity {
    type Kind = kind::Entity;
    type File = ClientEntity;
    type Beta = NoBeta<kind::Entity>;
}

impl Placed for ClientEntity {
    /// `entity/<namespace>/<name>.json`.
    fn default_path(&self) -> PackPath {
        default_path(DIRECTORY, &self.description.identifier)
    }
}

impl ClientEntity {
    /// Parses one client entity file without upgrading it: the struct its `format_version`
    /// selects.
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
        let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
        let origin = origin(PackRef::app_data(), PackSide::Resource, path);
        let text = admit(bytes, root_of(bytes).as_ref(), &origin, cx)?;
        let parsed = parse_as_written::<ClientEntityFamily>(JsonSlice::new(text), cx);
        let diagnostics = place(parsed.diagnostics, &origin);
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
) -> Result<Document<ClientEntity>, Rejected> {
    let origin = document.origin().clone();
    let version = document.version().clone();
    let diagnostics = document.diagnostics().clone();
    let value = document.into_value();
    let identifier = match &value {
        Any::V1_8_0(entity) => entity.description.identifier.as_str(),
        Any::V1_10_0(entity) => entity.description.identifier.as_str(),
        Any::V1_26_0(entity) => entity.description.identifier.as_str(),
    };
    let identity = Identity {
        identifier: Some(identifier.to_owned()),
        runtime_identifier: None,
    };
    let written = FormatVersionInput {
        version: Some(version.effective),
        beta_literal: false,
        prerelease: false,
    };
    let parsed = Parsed::<ClientEntityFamily>::new(
        Some(value),
        version,
        written,
        identity,
        ActorType::Mob,
        false,
        JsonPath::new(),
        Diagnostics::default(),
    );
    let outcome = finish(parsed, cx);
    let mut found = diagnostics;
    found.extend(place(outcome.diagnostics, &origin));
    match outcome.value {
        Some(Loaded::Stable(entity)) => Ok(Document::new(entity, origin, outcome.version, found)),
        _ => Err(Rejected::new(origin, found)),
    }
}

/// Where this loader refuses `text`, the text of a written client entity, read as the engine's
/// own content without a Molang compile; `None` when it loads.
pub(crate) fn refused_at(text: &[u8]) -> Option<JsonPath> {
    let cx = LoadContext::detached();
    let path = PackPath::from_static("entity/written.entity.json");
    let root = root_of(text);
    let outcome = parse_file(
        PackRef::app_data(),
        PackSide::Resource,
        &path,
        text,
        root.as_ref(),
        &cx,
    );
    let rejected = outcome.1.err()?;
    let at = rejected
        .diagnostics
        .iter()
        .find(|found| found.severity == Severity::Error)
        .map(|found| found.location.json_path.clone());
    Some(at.unwrap_or_default())
}

/// The origin of a file of `pack`.
fn origin(pack: PackRef, side: PackSide, path: &PackPath) -> Origin {
    Origin::new(pack, side, path.clone(), None, Vec::new())
}

/// A file-level error.
fn error(code: DiagnosticCode, at: JsonPath) -> Diagnostic {
    Diagnostic::new(
        code,
        Severity::Error,
        Phase::Parse,
        Location::new(None, None, at, None),
    )
}

/// A rejection with one finding.
fn reject(origin: &Origin, found: Diagnostic) -> Rejected {
    let diagnostics: Diagnostics = [found].into_iter().collect();
    Rejected::new(origin.clone(), place(diagnostics, origin))
}

/// Gives every finding the document's pack and file, and vanilla's texts the file's path.
fn place(found: Diagnostics, origin: &Origin) -> Diagnostics {
    found
        .into_iter()
        .map(|mut finding| {
            if let DiagnosticCode::InvalidValue {
                text: Some(VanillaText::Formatted { args, .. }),
            } = &mut finding.code
            {
                for arg in args.iter_mut().filter(|arg| *arg == validate::FILE) {
                    origin.path.as_str().clone_into(arg);
                }
            }
            finding.location.pack = Some(origin.pack.clone());
            finding.location.path = Some(origin.path.clone());
            finding
        })
        .collect()
}

/// The file's text when vanilla's loader would go on to the schema: UTF-8 within the nesting
/// limit, a root that is a non-empty object, and a `format_version` that is absent or within
/// 1.8.0 and the engine version. The rejections carry the loader's own texts, in its order.
/// `root` is [`root_of`] the same bytes.
fn admit<'b>(
    bytes: &'b [u8],
    root: Option<&Value>,
    origin: &Origin,
    cx: &LoadContext,
) -> Result<&'b str, Rejected> {
    let bytes = strip_bom(bytes);
    let malformed = |message: &str| {
        reject(
            origin,
            error(
                DiagnosticCode::MalformedInput {
                    message: message.to_owned(),
                    text: None,
                },
                JsonPath::new(),
            ),
        )
    };
    let text = std::str::from_utf8(bytes).map_err(|_| malformed("the file is not UTF-8"))?;
    check_depth(bytes, MAX_JSON_DEPTH).map_err(|_| malformed("the JSON nests too deeply"))?;
    // A text that is no JSON is the chain's to report (a parse error, or an empty document).
    let Some(root) = root else {
        return Ok(text);
    };
    // A root that is not an object, or an empty one, has nothing to parse.
    let Some(root) = root.as_object().filter(|root| !root.is_empty()) else {
        return Err(reject(
            origin,
            error(
                DiagnosticCode::MalformedInput {
                    message: NOTHING_TO_PARSE.to_owned(),
                    text: Some(VanillaText::Fixed(NOTHING_TO_PARSE)),
                },
                JsonPath::new(),
            ),
        ));
    };
    if let Some(version) = root.get("format_version")
        && let Some(code) = version_gate(version, origin.path.as_str(), cx)
    {
        return Err(reject(
            origin,
            error(code, read::below(&JsonPath::new(), &["format_version"])),
        ));
    }
    Ok(text)
}

/// The version checks of the game's loader and the schema's minimum, in vanilla's order: a
/// value that is no version (or the any-version, or the `"beta"` literal) is unknown; a
/// version above the engine exceeds the game version; below 1.0.0 it is unknown again; from
/// 1.0.0 up to 1.8.0 it is below the schema's minimum.
fn version_gate(version: &Value, path: &str, cx: &LoadContext) -> Option<DiagnosticCode> {
    // The text that is no version at all is the first of the checks.
    let unknown = || DiagnosticCode::UnsupportedVersion {
        text: text(UNKNOWN_VERSION, &[path]),
    };
    let literal = version.as_string().map(facet_value::VString::as_str);
    let Some(read) =
        SemVersion::from_json(version).filter(|read| !read.is_any() && literal != Some("beta"))
    else {
        return Some(unknown());
    };
    let written = literal.map_or_else(|| read.to_string(), str::to_owned);
    let engine = cx.engine_version;
    if read > SemVersion::new(engine.major, engine.minor, engine.patch) {
        return Some(DiagnosticCode::UnsupportedVersion {
            text: Some(VanillaText::ActorAboveEngine {
                version: written,
                engine,
            }),
        });
    }
    if read < SemVersion::new(1, 0, 0) {
        return Some(unknown());
    }
    if read < SemVersion::new(FLOOR.major, FLOOR.minor, FLOOR.patch) {
        return Some(bedrock_addon_core::diagnostic::cereal::version_below_min(
            &written,
            SCHEMA_MINIMUM,
        ));
    }
    None
}

/// The file's root as JSON, for the questions asked before the chain reads the text (which
/// payload the file holds, its `format_version`): parsed once per file. `None` for bytes that
/// are no UTF-8 or no JSON.
pub(super) fn root_of(bytes: &[u8]) -> Option<Value> {
    let text = std::str::from_utf8(strip_bom(bytes)).ok()?;
    // Of a member written twice the last stands, whole. An untyped read of the text as
    // written merges two objects of one key, so the peek reads the cleaned text, as the load
    // does.
    bedrock_addon_core::json::from_str_jsonc::<Value>(&bedrock_addon_core::json::last_wins(text))
        .ok()
}

/// Whether the document is an attachable: no `minecraft:client_entity` object, and a
/// `minecraft:attachable` one. Vanilla reads `minecraft:client_entity` first, then
/// `minecraft:attachable`, from either directory.
fn is_attachable(root: &Value) -> bool {
    let object = |key: &str| {
        root.as_object()
            .and_then(|root| root.get(key))
            .is_some_and(|value| value.as_object().is_some())
    };
    !object(read::PAYLOAD) && object(read::ATTACHABLE)
}

/// Whether the document holds a `minecraft:client_entity` object.
fn is_client_entity(root: &Value) -> bool {
    root.as_object()
        .and_then(|root| root.get(read::PAYLOAD))
        .is_some_and(|value| value.as_object().is_some())
}

/// Loads one client entity file: the identifier it names, in lower case (when the payload has
/// one), and the document or its rejection.
pub(crate) fn parse_file(
    pack: PackRef,
    side: PackSide,
    path: &PackPath,
    bytes: &[u8],
    root: Option<&Value>,
    cx: &LoadContext,
) -> (Option<EntityId>, Result<Document<ClientEntity>, Rejected>) {
    let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
    let origin = origin(pack, side, path);
    let text = match admit(bytes, root, &origin, cx) {
        Ok(text) => text,
        Err(rejected) => return (None, Err(rejected)),
    };
    let parsed = parse_as_written::<ClientEntityFamily>(JsonSlice::new(text), cx);
    let identifier = parsed
        .identity
        .identifier
        .as_deref()
        .map(|id| EntityId::parse(id.to_ascii_lowercase()));
    let outcome = finish(parsed, cx);
    let diagnostics = place(outcome.diagnostics, &origin);
    let result = match outcome.value {
        Some(Loaded::Stable(entity)) => {
            Ok(Document::new(entity, origin, outcome.version, diagnostics))
        }
        _ => Err(Rejected::new(origin, diagnostics)),
    };
    (identifier, result)
}

// ---- the collection ----------------------------------------------------------------------

/// A client entity file: a `.json` file below `entity/`.
fn is_json(path: &PackPath) -> bool {
    path.as_str()
        .rsplit_once('.')
        .is_some_and(|(_, extension)| extension == "json")
}

/// The folder of the attachables, in a pack and in the app's own data: a file there that
/// holds a `minecraft:client_entity` object is a client entity.
const ATTACHABLES: &str = "attachables";
const APP_ATTACHABLES: &str = "definitions/attachables";

/// The app's own `definitions/attachables`, then every `entity/**/*.json` and after them every
/// `attachables/**/*.json`, each over the packs from the lowest priority to the highest, as
/// vanilla loads them: the walk of the attachables, since one loader reads both payloads from
/// either folder. The game loads its built-in definitions first, then the `entity` and
/// `attachables` folders in that order, each from the last pack of the stack to the first.
const WALK: Walk = Walk {
    directories: &[DIRECTORY, ATTACHABLES],
    depth: ListDepth::Recursive,
    accept: Accept::Fn(is_json),
    order: WalkOrder::BottomUp,
    directory_major: true,
    app_data: &[APP_ATTACHABLES],
    app_data_place: AppDataPlace::Before,
    in_pack: None,
    shadowed: None,
    read_error: None,
};

/// What one walked file is.
#[derive(Debug)]
pub enum FileOutcome {
    /// The attachable family's: a `minecraft:attachable` document under `entity/`, and
    /// whatever under `attachables/` or the app's own folder is no client entity.
    Attachable,
    /// A client entity, or a file that fails as one.
    Entity(Box<Contribution<ClientEntity>>),
}

fn parse_walk_file(file: &WalkFile, cx: &LoadContext) -> FileOutcome {
    let root = root_of(&file.bytes);
    if root.as_ref().is_some_and(is_attachable) {
        return FileOutcome::Attachable;
    }
    // Outside `entity/` only a file with a `minecraft:client_entity` object is this family's: what is
    // wrong with any other file there, the attachables say.
    let in_entity = WALK.matches(&file.key) == Some(0) && !file.pack.is_app_data();
    if !in_entity && !root.as_ref().is_some_and(is_client_entity) {
        return FileOutcome::Attachable;
    }
    let contribution = match parse_file(
        file.pack.clone(),
        file.side,
        &file.path,
        &file.bytes,
        root.as_ref(),
        cx,
    ) {
        (_, Ok(document)) => Contribution::Stable(document.identifier(), document),
        (id, Err(rejected)) => Contribution::Rejected(id, rejected),
    };
    FileOutcome::Entity(Box::new(contribution))
}

/// One definition per identifier, by `description.min_engine_version` and not by pack priority
/// alone: in walk order a document above the stack's highest pack `min_engine_version` is
/// ignored; the first of an identifier defines it; a later one with a lower version is
/// ignored, with an equal one is merged into the definition, with a higher one replaces it.
/// Every ignored document gets a `ClientEntityShadowed` info (the crate's own: vanilla is
/// silent). The choice of merge step and the merge itself are `visual`'s, shared with attachables.
fn combine(parsed: Vec<FileOutcome>, stack: &StackCx<'_>) -> Registry<ClientEntity> {
    let full_stack = stack.full_stack_min_engine_version();
    let decide = move |existing: Option<&Document<ClientEntity>>,
                       incoming: &Document<ClientEntity>,
                       d: &mut Diagnostics| {
        let existing = existing.map(|existing| existing.min_engine_version());
        match merge_step(
            &full_stack,
            &incoming.min_engine_version(),
            existing.as_ref(),
        ) {
            MergeStep::Merge => EngineVersionDecision::Merge,
            MergeStep::Ignore(reason) => {
                let origin = incoming.origin();
                d.push(Diagnostic::new(
                    // The reason is in the finding's text.
                    DiagnosticCode::ClientEntityShadowed {
                        reason: Some(reason),
                    },
                    Severity::Info,
                    Phase::Validate,
                    Location::new(
                        Some(origin.pack.clone()),
                        Some(origin.path.clone()),
                        JsonPath::new(),
                        None,
                    ),
                ));
                EngineVersionDecision::Ignore
            }
            // `Create`, `Replace`, and any step a newer core adds.
            _ => EngineVersionDecision::Replace,
        }
    };
    let mut builder = RegistryBuilder::new(
        CombineRule::EngineVersionMerge {
            decide: Box::new(decide),
            merge: |earlier, mut later| {
                merge_slots(&mut earlier.slots(), &mut later.slots());
            },
        },
        CombineRule::top_wins(),
        stack.options,
    );
    for outcome in parsed {
        if let FileOutcome::Entity(contribution) = outcome {
            builder.push(*contribution);
        }
    }
    builder.finish()
}

/// The loader of `rp.client_entities()`: `entity/**/*.json` of every resource pack and the
/// client entities of `attachables/**/*.json` and of the app's `definitions/attachables`, one
/// definition per identifier by `description.min_engine_version`.
pub const fn load_registry() -> CollectionLoader<Registry<ClientEntity>, FileOutcome> {
    CollectionLoader {
        walk: WALK,
        parse: parse_walk_file,
        combine,
        needs: &[],
        then: None,
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bedrock_addon_core::load::LoadOptions;

    use super::*;
    use bedrock_addon_core::test_cost::{assert_linear_timed_at, timed};

    /// The documents the walk of a two-pack stack hands the combine for `identifiers` client
    /// entities, lower pack first: every identifier is defined twice, the second time
    /// alternately with the same `min_engine_version` (merged) and with a lower one (ignored).
    fn colliding(identifiers: usize) -> Vec<FileOutcome> {
        let cx = LoadContext::detached();
        let template = |mev: &str, texture: &str| {
            let text = format!(
                r#"{{"format_version": "1.26.0", "minecraft:client_entity": {{"description": {{"identifier": "demo:template", "min_engine_version": "{mev}", "textures": {{"default": "textures/{texture}"}}}}}}}}"#
            );
            ClientEntity::parse(
                &PackPath::from_static("entity/t.json"),
                text.as_bytes(),
                &cx,
            )
            .expect("the template loads")
        };
        let lower = template("1.20.0", "lower");
        let merged = template("1.20.0", "upper");
        let ignored = template("1.10.0", "upper");
        let outcome = |template: &Document<ClientEntity>, i: usize| {
            let mut entity = template.value().clone();
            entity.description.identifier = EntityId::parse(format!("demo:e{i}"));
            let path = PackPath::new(&format!("entity/e{i}.json")).expect("a pack path");
            let document = Document::new(
                entity,
                origin(PackRef::app_data(), PackSide::Resource, &path),
                template.version().clone(),
                Diagnostics::default(),
            );
            FileOutcome::Entity(Box::new(Contribution::Stable(
                document.identifier(),
                document,
            )))
        };
        let mut outcomes = Vec::with_capacity(identifiers * 2);
        outcomes.extend((0..identifiers).map(|i| outcome(&lower, i)));
        outcomes.extend(
            (0..identifiers).map(|i| outcome(if i % 2 == 0 { &merged } else { &ignored }, i)),
        );
        outcomes
    }

    /// Combines [`colliding`] of `identifiers`, checks the registry, and gives the time the
    /// combine took.
    fn combine_colliding(identifiers: usize) -> Duration {
        let parsed = colliding(identifiers);
        let options = LoadOptions::default();
        let (registry, took) = timed(|| combine(parsed, &StackCx::new(&options, &[])));
        assert_eq!(registry.len(), identifiers);
        assert_eq!(registry.diagnostics().count(), identifiers / 2);
        // Both packs are one `PackRef` here, so no contributor is recorded: the stack test
        // (`tests/client_entity/stack.rs`) holds that part.
        let texture = |i: usize| {
            let entity = registry
                .get(&EntityId::parse(format!("demo:e{i}")))
                .expect("every identifier is defined");
            entity.description.textures["default"].as_str().to_owned()
        };
        for i in [0, identifiers - 2] {
            assert_eq!(texture(i), "textures/upper");
        }
        for i in [1, identifiers - 1] {
            assert_eq!(texture(i), "textures/lower");
        }
        took
    }

    #[test]
    fn the_merge_of_colliding_identifiers_is_linear() {
        // 4 000 identifiers against 16 000, every one defined twice: the combine looks each
        // identifier up once, so four times the documents take about four times the time. A
        // combine that scanned what it had seen on every collision would take sixteen times.
        assert_linear_timed_at("client entity combine", 4_000, combine_colliding);
    }
}
