//! Attachables (`Attachable`): `attachables/**/*.json` of a resource pack (`minecraft:attachable`),
//! what a client draws for an item an actor holds or wears.
//!
//! Read by the derived reader of core (`schema::FromJson`) with the texts vanilla's schema logs per
//! member (`unknown child schema option type`): the description, its eras, the file around it and
//! the rules between files are the shared `visual` crate's, which client entities read too. What is
//! this family's is `item`, its identifier, its loader's hand-off of client entities to the other
//! family and its item bindings.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod binding;
pub mod codes;
pub mod history;
#[cfg(test)]
mod peek_tests;
#[cfg(test)]
mod scalar_tests;
mod validate;
pub use binding::{Condition, ItemBinding, ItemBindingEntry, ItemBindings, ItemMember};

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Code, CodeDef, Diagnostic, Diagnostics, JsonPath, Phase, Severity,
};
use bedrock_addon_core::document::{
    CombineRule, Contribution, Definition, Document, JsonDocument, JsonPlaced, NoBeta, Origin,
    Registry, RegistryBuilder, Rejected, default_path,
};
use bedrock_addon_core::id::{AttachableId, ItemId, kind};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Prepared, Value};
use bedrock_addon_core::load::DocumentSource;
use bedrock_addon_core::load::{
    AppDataPlace, CollectionLoader, LoadContext, StackCx, Walk, WalkFile,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::schema::FromJson;
use bedrock_addon_core::version::known;
use bedrock_addon_core::version::{FormatVersion, SemVersion};
use bedrock_addon_core::versioning::{
    DocumentRead, Envelope, Family, Loaded, NoComponents, ParseCx, Reading, StructIndex,
    ValidateCx, VersionStruct, Versioned,
};
// One enum for both families, declared in the shared `visual` crate.
pub use bedrock_addon_shared_visual::AnimationKind;
use bedrock_addon_shared_visual::family::{
    ENVELOPE, READING, Shadowed, Visual, document, parse_newest,
};
use bedrock_addon_shared_visual::file as shared;
use bedrock_addon_shared_visual::history::v1_8_0::ControllerList;
use bedrock_addon_shared_visual::history::v1_10_0::FoldedLiterals;
use bedrock_addon_shared_visual::{
    AttachableRoot, HasVisuals, MergeStep, SharedDescription, VisualSlots, Visuals,
    description_min_engine_version, merge_step,
};
use indexmap::IndexMap;

use crate::history::{AttachableFamily, AttachableHistory};

/// The payload key.
const PAYLOAD: &str = "minecraft:attachable";
/// The payload key of the client entity family, which the same root schema declares.
const CLIENT_ENTITY: &str = "minecraft:client_entity";

/// A resource-pack attachable (`attachables/**/*.json`, payload `minecraft:attachable`): what
/// a client draws for an item an actor holds or wears, in the newest shape (format version
/// 1.26.0 and up).
///
/// Everything is in [`description`](Self::description); the document has no components. Older
/// documents (1.8.0 and 1.10.0) are upgraded to this shape step by step ([`history`]).
///
/// An attachable is drawn for an item in one of two ways: `description.item` names the item,
/// with or without a condition ([`Attachable::binding`]), or the attachable's identifier is
/// the item's.
///
/// Built from [`Attachable::new`] with struct-update syntax:
///
/// ```
/// use bedrock_addon_core::id::{AttachableId, GeometryId};
/// use bedrock_addon_family_attachable::{Attachable, Description};
///
/// let id = AttachableId::parse("demo:wand");
/// let mut description = Description::new(id.clone());
/// description
///     .slots
///     .geometry
///     .insert("default".to_owned(), GeometryId::parse("geometry.wand"));
/// let wand = Attachable {
///     description,
///     ..Attachable::new(id)
/// };
/// assert_eq!(wand.identifier().as_str(), "demo:wand");
/// ```
#[doc(alias = "minecraft:attachable")]
#[doc(alias = "attachable")]
#[derive(Clone, Debug, PartialEq)]
pub struct Attachable {
    /// `format_version`: the version the document is written with, 1.26.0 or newer. A document
    /// upgraded from an older version has 1.26.0.
    pub format_version: FormatVersion,
    /// `description`: the identifier and everything the client draws the attachable with.
    pub description: Description,
    /// The item bindings a stack holds for this definition, in the order vanilla gathered
    /// them: its own, and those of every document merged into it or replaced by it. Filled
    /// by a stack load only: a document read from one file or built in code has none here
    /// (its own binding is [`Attachable::binding`]). Not part of the JSON.
    pub stack_bindings: Vec<ItemBindingEntry>,
}

/// `description` of an attachable (format version 1.26.0 and up).
///
/// Every member is kept as written. Vanilla lower-cases the identifier, every slot name and
/// the values of `materials`, `geometry`, `animations` and `particle_effects` when it loads
/// them; the accessors of [`Attachable`] give that reading. The keys of `item` are not
/// lower-cased. No member beyond these is allowed: vanilla skips a file that has one.
///
/// `description.slots` holds the members attachables share with client entities, and
/// `description.own` the one that is its own: `item`.
pub type Description = bedrock_addon_shared_visual::Description<AttachableId, ItemMember>;

impl Attachable {
    /// The identifier as vanilla keys it: in ASCII lower case.
    pub fn identifier(&self) -> AttachableId {
        AttachableId::parse(self.description.identifier.as_str().to_ascii_lowercase())
    }

    /// The version this definition competes with across packs: `description.min_engine_version`
    /// read as a version, or 0.0.0 when it is absent or does not read.
    pub fn min_engine_version(&self) -> SemVersion {
        description_min_engine_version(
            self.format_version,
            self.description.min_engine_version.as_ref(),
        )
    }

    /// The item this document's `description.item` binds the attachable to, and the condition
    /// it is drawn under; `None` without the member. The item is as written: vanilla compares
    /// it with the item's full identifier byte for byte. Of an object with several members
    /// vanilla reads one: the one whose key sorts first byte by byte, not the first written.
    pub fn binding(&self) -> Option<(ItemId, Condition)> {
        self.description
            .own
            .item
            .as_ref()
            .and_then(ItemBinding::bound)
    }

    /// The item bindings the stack holds for this definition
    /// ([`stack_bindings`](Self::stack_bindings)), in the order vanilla gathered them; a
    /// client tries them from the last to the first. Empty outside a stack.
    pub fn bound_items(&self) -> &[ItemBindingEntry] {
        &self.stack_bindings
    }

    /// `enable_attachables`, `false` when absent.
    pub fn enable_attachables(&self) -> bool {
        self.description.slots.enable_attachables()
    }

    /// `held_item_ignores_lighting`, `false` when absent.
    pub fn held_item_ignores_lighting(&self) -> bool {
        self.description.slots.held_item_ignores_lighting()
    }

    /// `hide_armor`, `false` when absent.
    pub fn hide_armor(&self) -> bool {
        self.description.slots.hide_armor()
    }

    /// The constant scale of held items: the written number when it is above 0, else 1.0
    /// (also for a Molang scale, which only a client evaluates).
    pub fn held_item_scale(&self) -> f32 {
        self.description.slots.held_item_scale()
    }

    /// What the `animations` entry `name` is, by its value's prefix in lower case; `None` for
    /// an unknown name and for a value vanilla ignores.
    pub fn animation_kind(&self, name: &str) -> Option<AnimationKind> {
        self.description.slots.animation_kind(name)
    }

    /// The text vanilla parses for `initialize`: the strings joined with no separator; `None`
    /// when the member is absent or the text is empty.
    pub fn initialize(&self) -> Option<String> {
        self.description.slots.initialize()
    }

    /// The text vanilla parses for `pre_animation`: the strings joined with no separator;
    /// `None` when the member is absent or the text is empty.
    pub fn pre_animation(&self) -> Option<String> {
        self.description.slots.pre_animation()
    }
}

impl HasVisuals for Attachable {
    fn visuals(&self) -> Visuals<'_> {
        self.description.slots.visuals()
    }
}

impl SharedDescription for Attachable {
    fn slots_mut(&mut self) -> &mut VisualSlots {
        &mut self.description.slots
    }

    /// `item`: the later document's, when it has one.
    fn merge_own(&mut self, later: &mut Self) {
        if let Some(item) = later.description.own.item.take() {
            self.description.own.item = Some(item);
        }
    }
}

/// The directory attachable files live in.
const DIRECTORY: &str = "attachables";
/// The directory of client entity files, where an attachable may also be.
const ENTITY_DIRECTORY: &str = "entity";
/// The app's own directory of attachables, outside every pack.
const APP_DIRECTORY: &str = "definitions/attachables";

impl Family for AttachableFamily {
    const NAME: &'static str = "attachable";
    const PAYLOAD_KEY: &'static str = PAYLOAD;
    const FLOOR: FormatVersion = known::V1_8_0;
    type Latest = Attachable;
    type Beta = NoBeta<kind::Attachable>;
    type Component = NoComponents;
    type History = AttachableHistory;
    const ENVELOPE: Envelope = ENVELOPE;

    fn reading(_: StructIndex, _: DocumentSource) -> Reading {
        READING
    }

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(attachable) = v {
            validate::validate(attachable, cx);
        }
    }
}

impl Visual for AttachableFamily {
    type Id = AttachableId;
    type Own = ItemMember;
    type Root<D: FromJson> = AttachableRoot<D>;
    type Family = Self;
    type Latest = Attachable;
    type History = AttachableHistory;

    const PAYLOAD: &'static str = PAYLOAD;
    const TWIN_PAYLOAD: &'static str = CLIENT_ENTITY;
    // The client entity is loaded alone from a file with both payloads, and says so.
    const NOTES_TWIN: bool = false;
    // D2: a number no double holds in `min_engine_version` is kept as the largest finite one.
    const CLAMPS_VERSION: bool = true;
    // Vanilla's 1.8 → 1.10 step converts `minecraft:client_entity` only.
    const CONTROLLERS: ControllerList = ControllerList::Kept;
    // D4: a literal the lower-casing changes is folded and not told.
    const FOLDED: FoldedLiterals = FoldedLiterals::Silent;
    const SHADOWED: Shadowed = Shadowed {
        older: &core_codes::ATTACHABLE_SHADOWED_OLDER,
        above_stack: &core_codes::ATTACHABLE_SHADOWED_ABOVE_STACK,
        other: &core_codes::ATTACHABLE_SHADOWED,
    };

    fn wording(names_only: bool) -> fn(&JsonPath) -> Code {
        if names_only {
            wording::<true>
        } else {
            wording::<false>
        }
    }

    fn latest(format_version: FormatVersion, description: Description) -> Attachable {
        Attachable {
            format_version,
            description,
            stack_bindings: Vec::new(),
        }
    }

    fn identifier(history: &AttachableHistory) -> &str {
        match history {
            AttachableHistory::V1_8_0(attachable) => attachable.description.identifier.as_str(),
            AttachableHistory::V1_10_0(attachable) => attachable.description.identifier.as_str(),
            AttachableHistory::V1_26_0(attachable) => attachable.description.identifier.as_str(),
        }
    }

    fn min_engine_version(latest: &Attachable) -> SemVersion {
        latest.min_engine_version()
    }
}

/// The wording of a wrong type of an attachable: `item` is neither a string nor an object, its
/// conditions are Molang, and every other member and list entry is worded by its place.
fn wording<const NAMES_ONLY: bool>(path: &JsonPath) -> Code {
    use bedrock_addon_core::diagnostic::JsonPathSegment::Key;
    let mut below = path.segments().skip(2);
    match (below.next(), below.next()) {
        (Some(Key("item")), None) => codes::UNKNOWN_ITEM_TYPE.with(["string, object"]),
        (Some(Key("item")), Some(_)) => {
            bedrock_addon_core::diagnostic::cereal::unknown_child_option("molang")
        }
        _ => bedrock_addon_shared_visual::read::wording::<true, NAMES_ONLY>(path),
    }
}

impl Versioned for Attachable {
    const VERSION: FormatVersion = known::V1_26_0;
}

impl VersionStruct for Attachable {
    fn parse(document: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        // The 1.26 schema.
        parse_newest::<AttachableFamily>(document, cx, d)
    }

    /// The whole document (the struct reads the whole text, `EnvelopeCheck::Never`), at the
    /// version it is written with: its own, and never below 1.26.0, the oldest this shape is.
    fn write(&self) -> Result<Value, EncodeError> {
        let version = self.format_version.max(Self::VERSION);
        document::<AttachableFamily, _>(version, &self.description)
    }
}

impl JsonDocument for Attachable {
    type Family = AttachableFamily;
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// One attachable file, upgraded to the newest shape. A file that also holds a
    /// `minecraft:client_entity` object is a client entity to vanilla, and refused here (the
    /// walk hands such a file to the client entities instead). The rules between files
    /// (which definition of an identifier wins, by `description.min_engine_version`, and which
    /// item bindings a stack gathers) apply to a stack, not to a single file.
    fn read(origin: Origin, bytes: &[u8], cx: &LoadContext) -> DocumentRead<AttachableFamily> {
        let root = root_of(bytes);
        if let Err(rejected) = refuse_client_entity(root.as_ref(), &origin) {
            return DocumentRead::Rejected(None, rejected);
        }
        match shared::parse_file::<AttachableFamily>(
            origin.pack.clone(),
            origin.side,
            &origin.path,
            bytes,
            root.as_ref(),
            cx,
        ) {
            (_, Ok(document)) => DocumentRead::Stable(document),
            (identifier, Err(rejected)) => {
                DocumentRead::Rejected(identifier.map(|id| id.to_string()), rejected)
            }
        }
    }

    /// Parses one attachable file without upgrading it: the struct its `format_version`
    /// selects, after the same refusal and admit step as [`read`](JsonDocument::read).
    ///
    /// # Errors
    ///
    /// Returns [`Rejected`] when the file cannot be read into a value at all, such as bytes that
    /// are not JSON or a document the family refuses; it carries every finding about the file.
    fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> Result<Document<AttachableHistory>, Rejected> {
        let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
        let root = root_of(bytes);
        refuse_client_entity(
            root.as_ref(),
            &shared::origin(PackRef::app_data(), PackSide::Resource, path),
        )?;
        shared::parse_written::<AttachableFamily>(path, bytes, root.as_ref(), cx)
    }

    /// A value built in code may hold what no file can: an empty `item` object, a slot name
    /// outside its pattern, a version above the engine's. The document is written only when
    /// this loader reads it back, else refused with the path of what it would not read.
    /// Whether its Molang compiles is not asked: that depends on the pack.
    fn refuse_write(&self) -> Result<(), EncodeError> {
        shared::refuse_write::<AttachableFamily>(self, |text| {
            shared::refused_at::<AttachableFamily>(
                text,
                root_of(text).as_ref(),
                "attachables/written.json",
            )
        })
    }
}

impl JsonPlaced for Attachable {
    /// `attachables/<identifier, each ':' as '_'>.json`.
    fn place(&self) -> PackPath {
        default_path(DIRECTORY, &self.description.identifier)
    }
}

// A hand `Definition`, not a `JsonDefinition`: the registry keys an attachable by its
// identifier in lower case ([`Attachable::identifier`], a value, not a reference into the
// struct), and the loader is the family's own (`FileOutcome`: a file of the walk may be a
// client entity's, and the combine gathers the item bindings), so `load::registry` does not
// apply.
impl Definition for Attachable {
    type Kind = kind::Attachable;
    type File = Attachable;
    type Beta = NoBeta<kind::Attachable>;
}

/// The file's root as JSON, for the questions asked before the chain reads the text (which
/// payload the file holds, its `format_version`): parsed once per file. `None` for bytes that
/// are no UTF-8, no JSON, or JSON nested deeper than the limit.
pub(crate) fn root_of(bytes: &[u8]) -> Option<Value> {
    // Before any parse: a text that nests deeper than the limit overflows the reader's stack
    // (`admit` reports it).
    let prepared = Prepared::new(bytes).ok()?;
    let text = prepared.as_str();
    // Of a member written twice the last stands, whole. An untyped read of the text
    // as written merges two objects of one key, so the peek reads the cleaned text, as the
    // load does.
    bedrock_addon_core::json::from_str_jsonc(&bedrock_addon_core::json::last_wins(text)).ok()
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
    match (object(CLIENT_ENTITY), object(PAYLOAD)) {
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
        Payload::Both => PAYLOAD,
        Payload::ClientEntity => CLIENT_ENTITY,
        Payload::Attachable | Payload::Neither => return None,
    };
    Some(
        Diagnostic::of(&core_codes::IGNORED_PAYLOAD)
            .severity(severity)
            .at(JsonPath::of_keys([key]))
            .in_file(origin),
    )
}

/// The rejection of a file read alone that is a client entity, not an attachable.
fn refuse_client_entity(root: Option<&Value>, origin: &Origin) -> Result<(), Rejected> {
    match ignored_payload(root, origin, Severity::Error) {
        Some(found) => Err(Rejected::new(origin.clone(), [found].into_iter().collect())),
        None => Ok(()),
    }
}

/// The app's own `definitions/attachables`, then every `entity/**/*.json` and after them every
/// `attachables/**/*.json`, each over the packs from the lowest priority to the highest, as
/// vanilla loads them.
// The game loads its built-in definitions first, then the directories `entity` and
// `attachables`, each from the last pack of the stack to the first.
const WALK: Walk = Walk::json_tree_in(&[ENTITY_DIRECTORY, DIRECTORY])
    .directory_major()
    .app_data(&[APP_DIRECTORY], AppDataPlace::Before);

/// What one walked file is.
#[derive(Debug)]
#[doc(hidden)]
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
#[doc(hidden)]
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
            let origin = shared::origin(file.pack.clone(), file.side, &file.path);
            // The crate's own note that the attachable part is not loaded: the game has no
            // line for it, so it is an information.
            if let Some(found) = ignored_payload(root.as_ref(), &origin, Severity::Info) {
                return FileOutcome::Ignored(Box::new(found));
            }
        }
    }
    let (contribution, binding) = match shared::parse_file::<AttachableFamily>(
        file.pack.clone(),
        file.side,
        &file.path,
        &file.bytes,
        root.as_ref(),
        cx,
    ) {
        (_, Ok(document)) => {
            let item = document.description.own.item.as_ref();
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
        let note = |code: &'static CodeDef<0>, entry: &ItemBindingEntry| {
            let file = Origin::file(entry.pack.clone(), PackSide::Resource, entry.file.clone());
            Diagnostic::of(code)
                .severity(Severity::Info)
                .at(JsonPath::of_keys([PAYLOAD, "description", "item"]))
                .phase(Phase::Validate)
                .in_file(&file)
        };
        let mut notes = Vec::new();
        let mut held = Vec::with_capacity(self.by_identifier.len());
        for (_, mut gathering) in self.by_identifier {
            for entry in gathering.bindings.iter_mut().take(gathering.live_from) {
                entry.shadowed = true;
                notes.push(note(&core_codes::ITEM_BINDING_SHADOWED, entry));
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
                notes.push(note(&core_codes::ITEM_BINDING_OVERLAP, earlier));
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
        shared::decide::<AttachableFamily>(&full_stack, existing, incoming, d)
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
    earlier.slots_mut().merge(later.slots_mut());
    earlier.merge_own(&mut later);
}

/// The loader of `rp.attachables()`: the app's `definitions/attachables`, then the
/// `minecraft:attachable` documents of `entity/**/*.json` and `attachables/**/*.json` of every
/// resource pack; one definition per identifier by `description.min_engine_version`.
#[doc(hidden)]
pub const LOADER: CollectionLoader<Registry<Attachable>, FileOutcome> = CollectionLoader {
    walk: WALK,
    parse: parse_walk_file,
    combine,
    needs: &[],
    then: None,
};

impl Attachable {
    /// An attachable for `identifier` with nothing to draw yet, at format version 1.26.0.
    pub fn new(identifier: AttachableId) -> Self {
        Self {
            format_version: Self::VERSION,
            description: Description::new(identifier),
            stack_bindings: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bedrock_addon_core::document::Content;
    use bedrock_addon_core::load::LoadOptions;

    use super::*;
    use bedrock_addon_core::test_cost::{assert_linear_timed_at, timed};

    /// A loaded 1.26.0 attachable to build the walked files of a stack from.
    fn template() -> Document<Attachable> {
        let text = r#"{"format_version": "1.26.0", "minecraft:attachable": {"description": {"identifier": "demo:template", "textures": {"default": "textures/t"}}}}"#;
        <Attachable as Content>::parse(
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
            shared::origin(PackRef::app_data(), PackSide::Resource, &path),
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
                .filter(|found| found.code.id() == name)
                .count()
        };
        let bindings: usize = registry
            .iter()
            .map(|(_, document)| document.bound_items().len())
            .sum();
        match shape {
            Shape::Colliding => {
                assert_eq!(registry.len(), n / 2);
                assert_eq!(count("core/attachable_shadowed_older"), n / 4);
                // Each identifier's own binding, and the merged half's second one.
                assert_eq!(bindings, n / 2 + n / 4);
                assert_eq!(count("core/item_binding_overlap"), n / 2 - 1);
            }
            Shape::Replaced => {
                assert_eq!(registry.len(), 1);
                assert_eq!(bindings, n);
                assert_eq!(count("core/item_binding_shadowed"), n - 1);
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
                assert_eq!(count("core/item_binding_overlap"), n - 1);
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
            let (index, took) = timed(|| crate::ItemBindings::from_registry(&registry));
            assert_eq!(index.get("demo:blade").len(), n);
            took
        });
    }
}

#[cfg(test)]
mod reading_table {
    //! The family's reading table: what the pipeline reads each of its files and structs by.

    use bedrock_addon_core::versioning::testing::{assert_reading_table, reading_table};

    #[test]
    fn attachable_reads_as_pinned() {
        assert_reading_table(
            "AttachableFamily",
            &reading_table::<super::AttachableFamily>(),
            r#"
                text: Root, clamps beyond a double: false, duplicates kept in: []
                envelope: Never
                missing version: at the floor, told at Warning: Some(Fixed("Warning: The \"format_version\" field was not given. Fields from format version 1.8.0 and older will be loaded. Update your targeted \"format_version\" to use fields from newer versions."))
                below floor: UseFormatVersion
                empty document: Error
                1.8.0..=1.26.0 (0..=2): json Own/Strict/Error/Warn/reports/lenient/Refuse, mcb Own/Strict/Error/Warn/reports/lenient/Refuse
            "#,
        );
    }
}
