//! Loading attachables: the family's version chain parameters, the version gate of
//! vanilla's hand-written loader, `Content` / `Definition`, the walk of `attachables/` and the
//! combine by `description.min_engine_version`.

use bedrock_addon_core::__private::Sealed;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity, VanillaText,
};
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, EngineVersionDecision, NoBeta,
    Origin, Placed, Registry, RegistryBuilder, Rejected, default_path,
};
use bedrock_addon_core::id::{AttachableId, ItemId, kind};
use bedrock_addon_core::json::{
    EncodeError, JsonSlice, MAX_JSON_DEPTH, Value, WriteStyle, check_depth, strip_bom,
};
use bedrock_addon_core::load::{
    AppDataPlace, CollectionLoader, ListDepth, LoadContext, StackCx, Walk, WalkFile, WalkOrder,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput, SemVersion};
use bedrock_addon_core::versioning::{
    ComponentTable, EnvelopeCheck, Family, Identity, Loaded, NoComponents, Parsed, ValidateCx,
    finish, parse_as_written,
};
use bedrock_addon_core::visual::{MergeStep, SharedDescription, merge_slots, merge_step};
use indexmap::IndexMap;

use super::history::{Any, AttachableFamily};
use super::read::{self, text};
use super::{Attachable, Condition, ItemBindingEntry, validate};

/// The directory attachable files live in.
const DIRECTORY: &str = "attachables";
/// The directory of client entity files, where an attachable may also be.
const ENTITY_DIRECTORY: &str = "entity";
/// The app's own directory of attachables, outside every pack.
const APP_DIRECTORY: &str = "definitions/attachables";

/// The floor: every explicit version below 1.8.0 is rejected by vanilla's own schema.
const FLOOR: FormatVersion = FormatVersion::new(1, 8, 0);

/// Vanilla's warning for a document without `format_version`, with the crate's floor in place
/// of its 1.0.0.
const VERSION_NOT_GIVEN: &str = "Warning: The \"format_version\" field was not given. Fields from format version 1.8.0 and older will be loaded. Update your targeted \"format_version\" to use fields from newer versions.";
/// The text for a `format_version` that does not read, or is below 1.0.0.
const UNKNOWN_VERSION: &str =
    "Unknown/unsupported version value specified in format_version field in {}";
/// The minimum that message names: the `v1_10` schema's.
const SCHEMA_MINIMUM: &str = "1.10.0";
/// The text for a root that is null, no object or empty.
const NOTHING_TO_PARSE: &str = "nothing to parse in";

impl Family for AttachableFamily {
    const NAME: &'static str = "attachable";
    const PAYLOAD_KEY: &'static str = read::PAYLOAD;
    const DIRECTORIES: &'static [&'static str] = &[DIRECTORY];
    const FLOOR: FormatVersion = FLOOR;
    type Latest = Attachable;
    type Beta = std::convert::Infallible;
    type Component = NoComponents;
    type Any = Any;
    // Vanilla's root schema declares `format_version`, `minecraft:client_entity` and
    // `minecraft:attachable`; a third member is "child '%s' not valid here.", not an envelope
    // error, so each struct reads the whole document.
    const ENVELOPE: EnvelopeCheck = EnvelopeCheck::Never;
    // The game logs this at the content log's warning level, the default
    // `missing_version_severity`.
    const MISSING_VERSION_TEXT: Option<VanillaText> = Some(VanillaText::Fixed(VERSION_NOT_GIVEN));

    fn components() -> &'static ComponentTable<Self> {
        &ComponentTable::EMPTY
    }

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(attachable) = v {
            validate::validate(attachable, cx);
        }
    }
}

impl Sealed for Attachable {}

impl Content for Attachable {
    const NAME: &'static str = "attachable";
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// Parses one attachable file, upgraded to the newest shape. A file that also holds a
    /// `minecraft:client_entity` object is a client entity to vanilla, and refused here. The
    /// rules between files (which definition of an identifier wins, by
    /// `description.min_engine_version`, and which item bindings a stack gathers) apply to a
    /// stack, not to a single file.
    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        let root = root_of(bytes);
        refuse_client_entity(
            root.as_ref(),
            &origin(PackRef::app_data(), PackSide::Resource, path),
        )?;
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

impl Definition for Attachable {
    type Kind = kind::Attachable;
    type File = Attachable;
    type Beta = NoBeta<kind::Attachable>;
}

impl Placed for Attachable {
    /// `attachables/<identifier, each ':' as '_'>.json`.
    fn default_path(&self) -> PackPath {
        default_path(DIRECTORY, &self.description.identifier)
    }
}

impl Attachable {
    /// Parses one attachable file without upgrading it: the struct its `format_version`
    /// selects.
    pub fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> Result<Document<Any>, Rejected> {
        let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
        let origin = origin(PackRef::app_data(), PackSide::Resource, path);
        let root = root_of(bytes);
        refuse_client_entity(root.as_ref(), &origin)?;
        let text = admit(bytes, root.as_ref(), &origin, cx)?;
        let parsed = parse_as_written::<AttachableFamily>(JsonSlice::new(text), cx);
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
) -> Result<Document<Attachable>, Rejected> {
    let origin = document.origin().clone();
    let version = document.version().clone();
    let diagnostics = document.diagnostics().clone();
    let value = document.into_value();
    let identifier = match &value {
        Any::V1_8_0(attachable) => attachable.description.identifier.as_str(),
        Any::V1_10_0(attachable) => attachable.description.identifier.as_str(),
        Any::V1_26_0(attachable) => attachable.description.identifier.as_str(),
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
    let parsed = Parsed::<AttachableFamily>::new(
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
        Some(Loaded::Stable(attachable)) => {
            Ok(Document::new(attachable, origin, outcome.version, found))
        }
        _ => Err(Rejected::new(origin, found)),
    }
}

/// Where this loader refuses `text`, the text of a written attachable, read as the engine's
/// own content without a Molang compile; `None` when it loads.
pub(crate) fn refused_at(text: &[u8]) -> Option<JsonPath> {
    let cx = LoadContext::detached();
    let path = PackPath::from_static("attachables/written.json");
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
    // A root that is null, no object or empty has nothing to parse.
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

/// `loadActorResourceDefinitionFormatVersion` and the schema's minimum, in vanilla's order: a
/// value that is no version (or the any-version, or the `"beta"` literal) is unknown; a
/// version above the engine exceeds the game version; below 1.0.0 it is unknown again; from
/// 1.0.0 up to 1.8.0 it is below the schema's minimum.
fn version_gate(version: &Value, path: &str, cx: &LoadContext) -> Option<DiagnosticCode> {
    // A value the game cannot read as a version, or that is the `beta` literal, is unknown.
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
/// are no UTF-8, no JSON, or JSON nested deeper than the limit.
pub(super) fn root_of(bytes: &[u8]) -> Option<Value> {
    let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
    let bytes = strip_bom(bytes);
    // Before any parse: a text that nests deeper than the limit overflows the reader's stack
    // (`admit` reports it).
    check_depth(bytes, MAX_JSON_DEPTH).ok()?;
    let text = std::str::from_utf8(bytes).ok()?;
    // Of a member written twice the last stands, whole. An untyped read of the text
    // as written merges two objects of one key, so the peek reads the cleaned text, as the
    // load does.
    bedrock_addon_core::json::from_str_jsonc::<Value>(&bedrock_addon_core::json::last_wins(text))
        .ok()
}

/// Loads one attachable file: the identifier it names, in lower case (when the payload has
/// one), and the document or its rejection.
pub(crate) fn parse_file(
    pack: PackRef,
    side: PackSide,
    path: &PackPath,
    bytes: &[u8],
    root: Option<&Value>,
    cx: &LoadContext,
) -> (Option<AttachableId>, Result<Document<Attachable>, Rejected>) {
    let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
    let origin = origin(pack, side, path);
    let text = match admit(bytes, root, &origin, cx) {
        Ok(text) => text,
        Err(rejected) => return (None, Err(rejected)),
    };
    let parsed = parse_as_written::<AttachableFamily>(JsonSlice::new(text), cx);
    let identifier = parsed
        .identity
        .identifier
        .as_deref()
        .map(|id| AttachableId::parse(id.to_ascii_lowercase()));
    let outcome = finish(parsed, cx);
    let diagnostics = place(outcome.diagnostics, &origin);
    let result = match outcome.value {
        Some(Loaded::Stable(attachable)) => Ok(Document::new(
            attachable,
            origin,
            outcome.version,
            diagnostics,
        )),
        _ => Err(Rejected::new(origin, diagnostics)),
    };
    (identifier, result)
}

// ---- the collection ----------------------------------------------------------------------

/// What the root of a file says the file is. Vanilla reads `minecraft:client_entity` first,
/// then `minecraft:attachable`, whatever the directory.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Payload {
    /// A `minecraft:attachable` object and no `minecraft:client_entity` object.
    Attachable,
    /// A `minecraft:client_entity` object alone: a client entity.
    ClientEntity,
    /// Both objects: a client entity, whose attachable part vanilla never reads.
    Both,
    /// Neither object.
    Neither,
}

/// The payload of the file whose root is `root`.
fn payload_of(root: &Value) -> Payload {
    // Which payload members the file has decides whose file it is.
    let object = |key: &str| {
        root.as_object()
            .and_then(|root| root.get(key))
            .is_some_and(|value| value.as_object().is_some())
    };
    match (object(read::CLIENT_ENTITY), object(read::PAYLOAD)) {
        (true, true) => Payload::Both,
        (true, false) => Payload::ClientEntity,
        (false, true) => Payload::Attachable,
        (false, false) => Payload::Neither,
    }
}

/// `IgnoredPayload` for a file that is a client entity: at its attachable payload when it has
/// both, else at the client entity payload this family does not load.
fn ignored_payload(
    root: Option<&Value>,
    origin: &Origin,
    severity: Severity,
) -> Option<Diagnostic> {
    let key = match root.map(payload_of)? {
        Payload::Both => read::PAYLOAD,
        Payload::ClientEntity => read::CLIENT_ENTITY,
        Payload::Attachable | Payload::Neither => return None,
    };
    Some(Diagnostic::new(
        DiagnosticCode::IgnoredPayload,
        severity,
        Phase::Parse,
        Location::new(
            Some(origin.pack.clone()),
            Some(origin.path.clone()),
            read::below(&JsonPath::new(), &[key]),
            None,
        ),
    ))
}

/// The rejection of a file read alone that is a client entity, not an attachable.
fn refuse_client_entity(root: Option<&Value>, origin: &Origin) -> Result<(), Rejected> {
    match ignored_payload(root, origin, Severity::Error) {
        Some(found) => Err(Rejected::new(origin.clone(), [found].into_iter().collect())),
        None => Ok(()),
    }
}

/// An attachable file: a `.json` file.
fn is_json(path: &PackPath) -> bool {
    path.as_str()
        .rsplit_once('.')
        .is_some_and(|(_, extension)| extension == "json")
}

/// The app's own `definitions/attachables`, then every `entity/**/*.json` and after them every
/// `attachables/**/*.json`, each over the packs from the lowest priority to the highest, as
/// vanilla loads them.
// The game loads its built-in definitions first, then the directories `entity` and
// `attachables`, each from the last pack of the stack to the first.
const WALK: Walk = Walk {
    directories: &[ENTITY_DIRECTORY, DIRECTORY],
    depth: ListDepth::Recursive,
    accept: is_json,
    order: WalkOrder::BottomUp,
    directory_major: true,
    app_data: &[APP_DIRECTORY],
    app_data_place: AppDataPlace::Before,
    in_pack: None,
    shadowed: None,
    read_error: None,
};

/// What one walked file is.
#[derive(Debug)]
pub enum FileOutcome {
    /// An `entity/` file that is no attachable, or a file of any folder that is a client
    /// entity alone: the client entity family's, which loads it or reports what is wrong
    /// with it.
    NotOurs,
    /// A client entity where an attachable could be: the `IgnoredPayload` warning.
    Ignored(Box<Diagnostic>),
    /// An attachable, or a file that fails as one.
    Document(Box<LoadedFile>),
}

/// One file read as an attachable.
#[derive(Debug)]
pub struct LoadedFile {
    /// The document or its rejection.
    contribution: Contribution<Attachable>,
    /// What the document's `item` binds, a condition that is no expression at its pack's
    /// Molang version.
    binding: Option<(ItemId, Condition)>,
}

fn parse_walk_file(file: &WalkFile, cx: &LoadContext) -> FileOutcome {
    let root = root_of(&file.bytes);
    let in_entity = file
        .key
        .as_str()
        .strip_prefix(ENTITY_DIRECTORY)
        .is_some_and(|rest| rest.starts_with('/'));
    let payload = root.as_ref().map(payload_of);
    if payload != Some(Payload::Attachable) {
        // Whatever is wrong with an `entity/` file that is no attachable, the client entity
        // family says it.
        if in_entity && !matches!(payload, Some(Payload::Both)) {
            return FileOutcome::NotOurs;
        }
        // A client entity alone, in whatever folder, is the client entity family's file,
        // which loads it: nothing to say here.
        if matches!(payload, Some(Payload::ClientEntity)) {
            return FileOutcome::NotOurs;
        }
        if matches!(payload, Some(Payload::Both)) {
            let origin = origin(file.pack.clone(), file.side, &file.path);
            // The crate's own note that the attachable part is not loaded: the game has no
            // line for it, so it is an information.
            if let Some(found) = ignored_payload(root.as_ref(), &origin, Severity::Info) {
                return FileOutcome::Ignored(Box::new(found));
            }
        }
    }
    let (contribution, binding) = match parse_file(
        file.pack.clone(),
        file.side,
        &file.path,
        &file.bytes,
        root.as_ref(),
        cx,
    ) {
        (_, Ok(document)) => {
            let item = document.description.item.as_ref();
            let binding = item.and_then(|item| item.bound_at(Some(cx.molang_version)));
            (
                Contribution::Stable(document.identifier(), document),
                binding,
            )
        }
        (id, Err(rejected)) => (Contribution::Rejected(id, rejected), None),
    };
    FileOutcome::Document(Box::new(LoadedFile {
        contribution,
        binding,
    }))
}

/// The bindings vanilla holds for one identifier while the stack loads.
struct Gathering {
    /// The `min_engine_version` of the definition as it stands.
    version: SemVersion,
    /// Which walked file is the definition (the first document, or the last that replaced).
    holder: usize,
    /// The bindings of every document that counted, in load order.
    bindings: Vec<ItemBindingEntry>,
    /// Where the bindings of the definition as it stands begin: those before it were written
    /// for a definition since replaced.
    live_from: usize,
}

/// The item bindings of a stack, gathered as vanilla appends them: one for every accepted
/// document with `item`, never removed, none for a document the resolver ignores.
#[derive(Default)]
struct Bindings {
    by_identifier: IndexMap<AttachableId, Gathering>,
    gathered: usize,
}

impl Bindings {
    /// Offers the walked file `index`, a loaded `document` of `identifier` whose `item` binds
    /// `binding`: what the shared identifier map does with the document decides what becomes
    /// of the binding.
    fn offer(
        &mut self,
        index: usize,
        identifier: &AttachableId,
        document: &Document<Attachable>,
        binding: Option<(ItemId, Condition)>,
        full_stack: &SemVersion,
    ) {
        // The binding is appended after the definition is found.
        let offered = document.min_engine_version();
        let existing = self.by_identifier.get(identifier);
        let step = merge_step(
            full_stack,
            &offered,
            existing.map(|gathering| &gathering.version),
        );
        if matches!(step, MergeStep::Ignore(_)) {
            // Vanilla returns before it reads `item`.
            return;
        }
        let entry = binding.map(|(item, condition)| {
            let sequence = self.gathered;
            self.gathered += 1;
            ItemBindingEntry {
                item,
                attachable: identifier.clone(),
                condition,
                pack: document.origin().pack.clone(),
                file: document.origin().path.clone(),
                shadowed: false,
                sequence,
            }
        });
        match (step, self.by_identifier.get_mut(identifier)) {
            (MergeStep::Merge, Some(gathering)) => gathering.bindings.extend(entry),
            (MergeStep::Replace, Some(gathering)) => {
                // The earlier bindings stay and now select the replacement; they are marked
                // once, at the end.
                gathering.live_from = gathering.bindings.len();
                gathering.version = offered;
                gathering.holder = index;
                gathering.bindings.extend(entry);
            }
            // `Create`, and any step a newer core adds.
            _ => {
                self.by_identifier.insert(
                    identifier.clone(),
                    Gathering {
                        version: offered,
                        holder: index,
                        bindings: entry.into_iter().collect(),
                        live_from: 0,
                    },
                );
            }
        }
    }

    /// The bindings of each definition with the walked file that holds it, and the notes:
    /// `ItemBindingShadowed` for each binding whose definition was replaced, and
    /// `ItemBindingOverlap` for each always-drawn binding of an item that has a later
    /// always-drawn one (a client tries the later first, and it always holds).
    fn finish(self) -> (Vec<(usize, Vec<ItemBindingEntry>)>, Vec<Diagnostic>) {
        let note = |code: DiagnosticCode, entry: &ItemBindingEntry| {
            Diagnostic::new(
                code,
                Severity::Info,
                Phase::Validate,
                Location::new(
                    Some(entry.pack.clone()),
                    Some(entry.file.clone()),
                    read::below(&JsonPath::new(), &[read::PAYLOAD, "description", "item"]),
                    None,
                ),
            )
        };
        let mut notes = Vec::new();
        let mut held = Vec::with_capacity(self.by_identifier.len());
        for (_, mut gathering) in self.by_identifier {
            for entry in gathering.bindings.iter_mut().take(gathering.live_from) {
                entry.shadowed = true;
                notes.push(note(DiagnosticCode::ItemBindingShadowed, entry));
            }
            held.push((gathering.holder, gathering.bindings));
        }
        // Over every binding in load order, each sequence number is used once.
        let mut in_order: Vec<Option<&ItemBindingEntry>> = vec![None; self.gathered];
        for entry in held.iter().flat_map(|(_, bindings)| bindings) {
            if let Some(place) = in_order.get_mut(entry.sequence) {
                *place = Some(entry);
            }
        }
        let mut last_always: IndexMap<&str, &ItemBindingEntry> = IndexMap::new();
        for entry in in_order.into_iter().flatten() {
            if entry.condition.is_always()
                && let Some(earlier) = last_always.insert(entry.item.as_str(), entry)
            {
                notes.push(note(DiagnosticCode::ItemBindingOverlap, earlier));
            }
        }
        (held, notes)
    }
}

/// One definition per identifier, by `description.min_engine_version` and not by pack priority
/// alone: in walk order a document above the stack's highest pack `min_engine_version` is
/// ignored; the first of an identifier defines it; a later one with a lower version is
/// ignored, with an equal one is merged into the definition, with a higher one replaces it.
/// Every ignored document gets an `AttachableShadowed` info (the crate's own: vanilla is silent). The
/// item bindings do not follow the definitions: every document that is not ignored adds its
/// own, and the definition as it stands holds them all ([`Attachable::bound_items`]).
// The choice of which document defines the identifier and the merge are `visual`'s, shared with client entities.
fn combine(parsed: Vec<FileOutcome>, stack: &StackCx<'_>) -> Registry<Attachable> {
    let full_stack = stack.full_stack_min_engine_version();
    let mut notes = Vec::new();
    let mut files: Vec<Option<Contribution<Attachable>>> = Vec::with_capacity(parsed.len());
    let mut bindings = Bindings::default();
    for outcome in parsed {
        match outcome {
            FileOutcome::NotOurs => {}
            FileOutcome::Ignored(found) => notes.push(*found),
            FileOutcome::Document(file) => {
                let LoadedFile {
                    contribution,
                    binding,
                } = *file;
                if let Contribution::Stable(identifier, document) = &contribution {
                    bindings.offer(files.len(), identifier, document, binding, &full_stack);
                }
                files.push(Some(contribution));
            }
        }
    }
    let (held, binding_notes) = bindings.finish();
    notes.extend(binding_notes);
    for (holder, stack_bindings) in held {
        let Some(slot) = files.get_mut(holder) else {
            continue;
        };
        *slot = match slot.take() {
            Some(Contribution::Stable(identifier, document)) => Some(Contribution::Stable(
                identifier,
                document.map(|attachable| Attachable {
                    stack_bindings,
                    ..attachable
                }),
            )),
            other => other,
        };
    }
    let decide = move |existing: Option<&Document<Attachable>>,
                       incoming: &Document<Attachable>,
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
                    DiagnosticCode::AttachableShadowed {
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
            merge: merge_documents,
        },
        CombineRule::top_wins(),
        stack.options,
    );
    for contribution in files.into_iter().flatten() {
        builder.push(contribution);
    }
    for found in notes {
        builder.push_diagnostic(found);
    }
    builder.finish()
}

/// Merges `later` into `earlier`, two documents of one identifier and `min_engine_version`:
/// the shared members as `visual` merges them (the merged definition has `animations` and
/// `sound_effects` only when one of the two documents has), then `item`.
fn merge_documents(earlier: &mut Attachable, mut later: Attachable) {
    // The presence of the two maps is core's to keep, so an authored empty member is written
    // back.
    merge_slots(&mut earlier.slots(), &mut later.slots());
    earlier.merge_own(&mut later);
}

/// The loader of `rp.attachables()`: the app's `definitions/attachables`, then the
/// `minecraft:attachable` documents of `entity/**/*.json` and `attachables/**/*.json` of every
/// resource pack; one definition per identifier by `description.min_engine_version`.
pub const fn load_registry() -> CollectionLoader<Registry<Attachable>, FileOutcome> {
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

    /// A loaded 1.26.0 attachable to build the walked files of a stack from.
    fn template() -> Document<Attachable> {
        let text = r#"{"format_version": "1.26.0", "minecraft:attachable": {"description": {"identifier": "demo:template", "textures": {"default": "textures/t"}}}}"#;
        Attachable::parse(
            &PackPath::from_static("attachables/t.json"),
            text.as_bytes(),
            &LoadContext::detached(),
        )
        .expect("the template loads")
    }

    /// The walked file `index`: `template` as `identifier` at `min_engine_version` `version`,
    /// bound always to `item`.
    fn walked(
        template: &Document<Attachable>,
        index: usize,
        identifier: &str,
        version: &str,
        item: &str,
    ) -> FileOutcome {
        let mut attachable = template.value().clone();
        attachable.description.identifier = AttachableId::parse(identifier);
        attachable.description.min_engine_version = Some(Value::from(version));
        let path = PackPath::new(&format!("attachables/a{index}.json")).expect("a pack path");
        let document = Document::new(
            attachable,
            origin(PackRef::app_data(), PackSide::Resource, &path),
            template.version().clone(),
            Diagnostics::default(),
        );
        FileOutcome::Document(Box::new(LoadedFile {
            contribution: Contribution::Stable(document.identifier(), document),
            binding: Some((ItemId::parse(item), Condition::Always)),
        }))
    }

    /// The shapes of stack the combine is timed on.
    #[derive(Clone, Copy, Debug)]
    enum Shape {
        /// Every identifier defined twice: merged or ignored, in turn.
        Colliding,
        /// One identifier, each document replacing the one before it and binding an item.
        Replaced,
        /// As many identifiers, all bound always to one item.
        OneItem,
    }

    /// The walked files of `shape` for `n` documents.
    fn files(shape: Shape, n: usize) -> Vec<FileOutcome> {
        let template = template();
        match shape {
            Shape::Colliding => {
                let lower = (0..n / 2)
                    .map(|i| walked(&template, i, &format!("demo:a{i}"), "1.20.0", "demo:x"));
                let upper = (0..n / 2).map(|i| {
                    let version = if i % 2 == 0 { "1.20.0" } else { "1.10.0" };
                    let item = format!("demo:y{i}");
                    walked(&template, n / 2 + i, &format!("demo:a{i}"), version, &item)
                });
                lower.chain(upper).collect()
            }
            Shape::Replaced => (0..n)
                .map(|i| {
                    let version = format!("0.{}.{}", i / 1000, i % 1000);
                    walked(&template, i, "demo:one", &version, &format!("demo:i{i}"))
                })
                .collect(),
            Shape::OneItem => (0..n)
                .map(|i| walked(&template, i, &format!("demo:a{i}"), "1.20.0", "demo:blade"))
                .collect(),
        }
    }

    /// Combines [`files`] of `shape` and `n`, checks the registry, and gives the time the
    /// combine took.
    fn combined(shape: Shape, n: usize) -> Duration {
        let parsed = files(shape, n);
        let options = LoadOptions::default();
        let (registry, took) = timed(|| combine(parsed, &StackCx::new(&options, &[])));
        let count = |name: &str| {
            registry
                .diagnostics()
                .filter(|found| found.code.name() == name)
                .count()
        };
        let bindings: usize = registry
            .iter()
            .map(|(_, document)| document.bound_items().len())
            .sum();
        match shape {
            Shape::Colliding => {
                assert_eq!(registry.len(), n / 2);
                assert_eq!(count("AttachableShadowed"), n / 4);
                // Each identifier's own binding, and the merged half's second one.
                assert_eq!(bindings, n / 2 + n / 4);
                assert_eq!(count("ItemBindingOverlap"), n / 2 - 1);
            }
            Shape::Replaced => {
                assert_eq!(registry.len(), 1);
                assert_eq!(bindings, n);
                assert_eq!(count("ItemBindingShadowed"), n - 1);
                let (_, one) = registry.iter().next().expect("one definition");
                assert!(
                    one.bound_items()[..n - 1]
                        .iter()
                        .all(|entry| entry.shadowed)
                );
                assert!(!one.bound_items()[n - 1].shadowed);
            }
            Shape::OneItem => {
                assert_eq!(registry.len(), n);
                assert_eq!(bindings, n);
                assert_eq!(count("ItemBindingOverlap"), n - 1);
            }
        }
        took
    }

    #[test]
    fn the_combine_is_linear_in_the_documents_of_colliding_identifiers() {
        assert_linear_timed_at("attachable combine, colliding", 5_000, |n| {
            combined(Shape::Colliding, n)
        });
    }

    #[test]
    fn the_combine_is_linear_in_a_chain_of_replacements_that_each_bind_an_item() {
        assert_linear_timed_at("attachable combine, replaced", 5_000, |n| {
            combined(Shape::Replaced, n)
        });
    }

    #[test]
    fn the_combine_is_linear_in_the_bindings_of_one_item() {
        assert_linear_timed_at("attachable combine, one item", 5_000, |n| {
            combined(Shape::OneItem, n)
        });
    }

    #[test]
    fn the_item_index_is_linear_in_the_bindings() {
        assert_linear_timed_at("attachable item index", 50_000, |n| {
            let options = LoadOptions::default();
            let registry = combine(files(Shape::OneItem, n), &StackCx::new(&options, &[]));
            let (index, took) = timed(|| super::super::ItemBindings::from_registry(&registry));
            assert_eq!(index.get("demo:blade").len(), n);
            took
        });
    }
}
