//! Client entities (`ClientEntity`, `SpawnEgg`): what a client draws for an entity, from the
//! resource packs.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod history;
#[cfg(test)]
mod peek_tests;
mod read;
mod validate;
mod write;

use bedrock_addon_core::id::{
    AnimationRef, EntityId, GeometryId, MaterialId, ParticleId, SoundEventName, TexturePath,
};
use bedrock_addon_core::json::{StrictBool, StrictString, Value};
use bedrock_addon_core::version::{FormatVersion, SemVersion};
use bedrock_addon_core::visual::{
    ControllerUse, DescriptionSlots, HasVisuals, HeldItemScale, Scripts, SharedDescription,
    SpawnEgg, Visuals, description_min_engine_version,
};
// One enum serves client entities and attachables; core's `visual` module declares it.
pub use bedrock_addon_core::visual::AnimationKind;
use facet::Facet;
use indexmap::IndexMap;
use std::borrow::Cow;

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Phase, Severity, VanillaText,
};
use bedrock_addon_core::document::{
    CombineRule, Contribution, Definition, Document, EngineVersionDecision, JsonDocument,
    JsonPlaced, NoBeta, Origin, Registry, RegistryBuilder, Rejected, default_path,
};
use bedrock_addon_core::id::kind;
use bedrock_addon_core::json::{
    EncodeError, JsonSlice, Unrepresentable, WriteStyle, strip_bom, write_json,
};
use bedrock_addon_core::json::{PrepareError, Prepared};
use bedrock_addon_core::load::{
    AppDataPlace, CollectionLoader, LoadContext, StackCx, Walk, WalkFile,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::FormatVersionInput;
use bedrock_addon_core::version::known;
use bedrock_addon_core::versioning::{
    DocumentRead, Envelope, EnvelopeCheck, Family, Identity, Loaded, MissingVersion, NoComponents,
    Parsed, ValidateCx, VersionStruct, finish, parse_as_written,
};
use bedrock_addon_core::visual::{MergeStep, merge_slots, merge_step};

use crate::history::{Any, ClientEntityFamily};
use crate::read::text;
use bedrock_addon_core::versioning::Versioned;

/// A resource-pack client entity definition (`entity/**/*.json`, payload
/// `minecraft:client_entity`): what a client draws for an entity, in the newest shape (format
/// version 1.26.0 and up).
///
/// Everything is in [`description`](Self::description); the document has no components. Older
/// documents (1.8.0 and 1.10.0) are upgraded to this shape step by step ([`history`]).
///
/// Built from [`ClientEntity::new`] with struct-update syntax:
///
/// ```
/// use bedrock_addon_family_client_entity::{ClientEntity, Description};
/// use bedrock_addon_core::id::{EntityId, GeometryId};
///
/// let id = EntityId::parse("demo:ghost");
/// let mut description = Description::new(id.clone());
/// description
///     .geometry
///     .insert("default".to_owned(), GeometryId::parse("geometry.ghost"));
/// let entity = ClientEntity {
///     description,
///     ..ClientEntity::new(id)
/// };
/// assert_eq!(entity.identifier().as_str(), "demo:ghost");
/// ```
#[doc(alias = "minecraft:client_entity")]
#[doc(alias = "client_entity")]
#[derive(Clone, Debug, PartialEq)]
pub struct ClientEntity {
    /// `format_version`: the version the document is written with, 1.26.0 or newer. A document
    /// upgraded from an older version has 1.26.0.
    pub format_version: FormatVersion,
    /// `description`: the identifier and everything the client draws the entity with.
    pub description: Description,
}

/// `description` of a client entity (format version 1.26.0 and up).
///
/// Every member is kept as written. Vanilla lower-cases the identifier, every slot name and
/// the values of `materials`, `geometry`, `animations` and `particle_effects` when it loads
/// them; the accessors of [`ClientEntity`] give that reading. No member beyond these is
/// allowed: vanilla skips a file that has one.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Description {
    /// `identifier`: the entity this definition draws. Compared in lower case.
    pub identifier: EntityId,
    /// `min_engine_version`: a version string or `[major, minor, patch]`. It decides which
    /// definition of an identifier wins across packs, never which members are read; a text
    /// that is no version counts as 0.0.0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub min_engine_version: Option<Value>,
    /// `materials`: slot name → material.
    #[facet(default, skip_serializing_if = IndexMap::is_empty)]
    pub materials: IndexMap<String, MaterialId>,
    /// `textures`: slot name → texture path.
    #[facet(default, skip_serializing_if = IndexMap::is_empty)]
    pub textures: IndexMap<String, TexturePath>,
    /// `geometry`: slot name → geometry.
    #[facet(default, skip_serializing_if = IndexMap::is_empty)]
    pub geometry: IndexMap<String, GeometryId>,
    /// `queryable_geometry`: the geometry slot Molang's geometry queries read.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub queryable_geometry: Option<StrictString>,
    /// `spawn_egg`: how the entity's spawn egg looks.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub spawn_egg: Option<SpawnEgg>,
    /// `scripts`: the Molang the client runs for the model and the animations it plays.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub scripts: Option<Scripts>,
    /// `particle_effects`: short name → particle effect.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub particle_effects: Option<IndexMap<String, ParticleId>>,
    /// `particle_emitters`: accepted and read by nothing.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub particle_emitters: Option<IndexMap<String, StrictString>>,
    /// `animations`: short name → animation (`animation.…`) or animation controller
    /// (`controller.animation.…`). A value with neither prefix is ignored. `None` is a
    /// description without the member; an empty object the author wrote is kept.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub animations: Option<IndexMap<String, AnimationRef>>,
    /// `render_controllers`: the render controllers that draw the entity, in order.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub render_controllers: Option<Vec<ControllerUse>>,
    /// `sound_effects`: short name → sound event. `None` is a description without the
    /// member; an empty object the author wrote is kept.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub sound_effects: Option<IndexMap<String, SoundEventName>>,
    /// `enable_attachables`: draw held and worn attachables; `false` when absent.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub enable_attachables: Option<StrictBool>,
    /// `held_item_ignores_lighting`: draw held items fully lit; `false` when absent.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub held_item_ignores_lighting: Option<StrictBool>,
    /// `held_item_scale`: the scale of held items; 1.0 when absent. A number at or below 0 is
    /// not used.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub held_item_scale: Option<HeldItemScale>,
    /// `hide_armor`: do not draw worn armour; `false` when absent.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub hide_armor: Option<StrictBool>,
}

impl ClientEntity {
    /// The identifier as vanilla keys it: in ASCII lower case.
    pub fn identifier(&self) -> EntityId {
        EntityId::parse(self.description.identifier.as_str().to_ascii_lowercase())
    }

    /// The version this definition competes with across packs: `description.min_engine_version`
    /// read as a version, or 0.0.0 when it is absent or does not read.
    pub fn min_engine_version(&self) -> SemVersion {
        description_min_engine_version(
            self.format_version,
            self.description.min_engine_version.as_ref(),
        )
    }

    /// `enable_attachables`, `false` when absent.
    pub fn enable_attachables(&self) -> bool {
        self.description.enable_attachables.is_some_and(|b| b.0)
    }

    /// `held_item_ignores_lighting`, `false` when absent.
    pub fn held_item_ignores_lighting(&self) -> bool {
        self.description
            .held_item_ignores_lighting
            .is_some_and(|b| b.0)
    }

    /// `hide_armor`, `false` when absent.
    pub fn hide_armor(&self) -> bool {
        self.description.hide_armor.is_some_and(|b| b.0)
    }

    /// The constant scale of held items: the written number when it is above 0, else 1.0
    /// (also for a Molang scale, which only a client evaluates).
    pub fn held_item_scale(&self) -> f32 {
        // The game stores the value only when it is above 0; anything else keeps the default 1.0.
        match self.description.held_item_scale {
            Some(HeldItemScale::Number(scale)) if scale > 0.0 => scale,
            _ => 1.0,
        }
    }

    /// What the `animations` entry `name` is, by its value's prefix in lower case; `None` for
    /// an unknown name and for a value vanilla ignores.
    pub fn animation_kind(&self, name: &str) -> Option<AnimationKind> {
        // The game tests the `controller.animation.` prefix first, then `animation.`; a value
        // with neither prefix is ignored without a message.
        let value = self.description.animations.as_ref()?.get(name)?.as_str();
        let value = value.to_ascii_lowercase();
        if value.starts_with("controller.animation.") {
            Some(AnimationKind::Controller)
        } else if value.starts_with("animation.") {
            Some(AnimationKind::Animation)
        } else {
            None
        }
    }

    /// The text vanilla parses for `initialize`: the strings joined with no separator; `None`
    /// when the member is absent or the text is empty.
    pub fn initialize(&self) -> Option<String> {
        let scripts = self.description.scripts.as_ref()?;
        validate::joined(scripts.initialize.as_deref())
    }

    /// The text vanilla parses for `pre_animation`: the strings joined with no separator;
    /// `None` when the member is absent or the text is empty.
    pub fn pre_animation(&self) -> Option<String> {
        let scripts = self.description.scripts.as_ref()?;
        validate::joined(scripts.pre_animation.as_deref())
    }
}

impl HasVisuals for ClientEntity {
    fn visuals(&self) -> Visuals<'_> {
        let d = &self.description;
        Visuals::new(
            Some(&d.materials),
            Some(&d.textures),
            Some(&d.geometry),
            d.animations.as_ref(),
            d.render_controllers.as_deref().unwrap_or_default(),
        )
    }
}

impl SharedDescription for ClientEntity {
    fn slots(&mut self) -> DescriptionSlots<'_> {
        let d = &mut self.description;
        DescriptionSlots {
            materials: &mut d.materials,
            textures: &mut d.textures,
            geometry: &mut d.geometry,
            animations: &mut d.animations,
            sound_effects: &mut d.sound_effects,
            particle_effects: &mut d.particle_effects,
            queryable_geometry: &mut d.queryable_geometry,
            render_controllers: &mut d.render_controllers,
            spawn_egg: &mut d.spawn_egg,
            scripts: &mut d.scripts,
            enable_attachables: &mut d.enable_attachables,
            held_item_ignores_lighting: &mut d.held_item_ignores_lighting,
            held_item_scale: &mut d.held_item_scale,
            hide_armor: &mut d.hide_armor,
        }
    }
}

/// The directory client entity files live in.
const DIRECTORY: &str = "entity";

/// The floor: every explicit version below 1.8.0 is rejected by vanilla's own schema
const FLOOR: FormatVersion = known::V1_8_0;

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
    const FLOOR: FormatVersion = FLOOR;
    type Latest = ClientEntity;
    type Beta = NoBeta<kind::Entity>;
    type Component = NoComponents;
    type Any = Any;
    const ENVELOPE: Envelope = Envelope {
        // Vanilla's root schema declares `format_version`, `minecraft:client_entity` and
        // `minecraft:attachable`; a third member is "child '%s' not valid here.", not an envelope
        // error, so each struct reads the whole document.
        check: EnvelopeCheck::Never,
        // The game logs this at level 2, the class the pack report shows as a warning.
        missing: MissingVersion::Floor {
            text: Some(VanillaText::Fixed(VERSION_NOT_GIVEN)),
            severity: Some(Severity::Warning),
        },
        ..Envelope::DEFAULT
    };

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(entity) = v {
            validate::validate(entity, cx);
        }
    }
}

impl JsonDocument for ClientEntity {
    type Family = ClientEntityFamily;
    const SIDES: &'static [PackSide] = &[PackSide::Resource];
    const NAME: &'static str = "client entity";

    /// One client entity file, upgraded to the newest shape: vanilla's admit step (the text,
    /// the root, the version gate), then the chain. The rules between files (which definition
    /// of an identifier wins, by `description.min_engine_version`) apply to a stack, not to a
    /// single file.
    fn read(origin: Origin, bytes: &[u8], cx: &LoadContext) -> DocumentRead<ClientEntityFamily> {
        let root = root_of(bytes);
        match parse_file(
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

    /// Parses one client entity file without upgrading it: the struct its `format_version`
    /// selects, after the same admit step as [`read`](JsonDocument::read).
    ///
    /// # Errors
    ///
    /// Returns [`Rejected`](bedrock_addon_core::document::Rejected) with the findings the game's content
    /// log would print when the bytes cannot be read as this file at all: not JSON, a
    /// `format_version` with no struct here, or a document that does not fit the struct it selects.
    fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> Result<Document<Any>, Rejected> {
        let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
        let origin = origin(PackRef::app_data(), PackSide::Resource, path);
        let text = admit(bytes, root_of(bytes).as_ref(), &origin, cx)?;
        let parsed = parse_as_written::<ClientEntityFamily>(JsonSlice::new(&text), cx);
        let diagnostics = place(parsed.diagnostics, &origin);
        match parsed.value {
            Some(value) => Ok(Document::new(value, origin, parsed.version, diagnostics)),
            None => Err(Rejected::new(origin, diagnostics)),
        }
    }

    /// A value built in code may hold what no file can: an empty identifier, a slot name
    /// outside its pattern, a version above the engine's. The document is written only when
    /// this loader reads it back, else refused with the path of what it would not read.
    /// Whether its Molang compiles is not asked: that depends on the pack.
    fn refuse_write(&self) -> Result<(), EncodeError> {
        let mut text = Vec::new();
        write_json(
            &VersionStruct::write(self)?,
            &mut text,
            WriteStyle::default(),
        )?;
        match refused_at(&text) {
            Some(at) => Err(EncodeError::NotRepresentable {
                at,
                what: Unrepresentable::Unsupported,
            }),
            None => Ok(()),
        }
    }
}

impl JsonPlaced for ClientEntity {
    /// `entity/<namespace>/<name>.json`.
    fn place(&self) -> PackPath {
        default_path(DIRECTORY, &self.description.identifier)
    }
}

// A hand `Definition`, not a `JsonDefinition`: the registry keys a client entity by its
// identifier in lower case ([`ClientEntity::identifier`], a value, not a reference into the
// struct), and the loader is the family's own (`FileOutcome`: a file of the walk may be an
// attachable's), so `load::registry` does not apply.
impl Definition for ClientEntity {
    type Kind = kind::Entity;
    type File = ClientEntity;
    type Beta = NoBeta<kind::Entity>;
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
    let parsed = Parsed::<ClientEntityFamily>::from_value(
        Some(value),
        version,
        written,
        JsonPath::new(),
        Diagnostics::default(),
    )
    .with_identity(identity);
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
    Origin::file(pack, side, path.clone())
}

/// A rejection with one finding.
fn reject(origin: &Origin, found: Diagnostic) -> Rejected {
    let diagnostics: Diagnostics = [found].into_iter().collect();
    Rejected::new(origin.clone(), place(diagnostics, origin))
}

/// Gives every finding the document's pack and file, and vanilla's texts the file's path.
fn place(mut found: Diagnostics, origin: &Origin) -> Diagnostics {
    for finding in found.iter_mut() {
        if let DiagnosticCode::InvalidValue {
            text: Some(VanillaText::Formatted { args, .. }),
        } = &mut finding.code
        {
            for arg in args.iter_mut().filter(|arg| *arg == validate::FILE) {
                origin.path.as_str().clone_into(arg);
            }
        }
    }
    // In place: the list keeps where the members of the upgraded entity are in the file.
    found.located(origin)
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
) -> Result<Cow<'b, str>, Rejected> {
    let malformed = |message: &str| {
        reject(
            origin,
            Diagnostic::error(DiagnosticCode::MalformedInput {
                message: message.to_owned(),
                text: None,
            }),
        )
    };
    // `new` never refuses an empty text: that is the chain's to report.
    let text = Prepared::new(bytes)
        .map_err(|error| match error {
            PrepareError::NotUtf8(_) => malformed("the file is not UTF-8"),
            _ => malformed("the JSON nests too deeply"),
        })?
        .into_text();
    // A text that is no JSON is the chain's to report (a parse error, or an empty document).
    let Some(root) = root else {
        return Ok(text);
    };
    // A root that is not an object, or an empty one, has nothing to parse.
    let Some(root) = root.as_object().filter(|root| !root.is_empty()) else {
        return Err(reject(
            origin,
            Diagnostic::error(DiagnosticCode::MalformedInput {
                message: NOTHING_TO_PARSE.to_owned(),
                text: Some(VanillaText::Fixed(NOTHING_TO_PARSE)),
            }),
        ));
    };
    if let Some(version) = root.get("format_version")
        && let Some(code) = version_gate(version, origin.path.as_str(), cx)
    {
        return Err(reject(
            origin,
            Diagnostic::error(code).at(JsonPath::of_keys(["format_version"])),
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
pub(crate) fn root_of(bytes: &[u8]) -> Option<Value> {
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
    let parsed = parse_as_written::<ClientEntityFamily>(JsonSlice::new(&text), cx);
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

/// The folder of the attachables, in a pack and in the app's own data: a file there that
/// holds a `minecraft:client_entity` object is a client entity.
const ATTACHABLES: &str = "attachables";
const APP_ATTACHABLES: &str = "definitions/attachables";

/// The app's own `definitions/attachables`, then every `entity/**/*.json` and after them every
/// `attachables/**/*.json`, each over the packs from the lowest priority to the highest, as
/// vanilla loads them: the walk of the attachables, since one loader reads both payloads from
/// either folder. The game loads its built-in definitions first, then the `entity` and
/// `attachables` folders in that order, each from the last pack of the stack to the first.
const WALK: Walk = Walk::json_tree_in(&[DIRECTORY, ATTACHABLES])
    .directory_major()
    .app_data(&[APP_ATTACHABLES], AppDataPlace::Before);

/// What one walked file is.
#[derive(Debug)]
#[doc(hidden)]
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
                // The reason is in the finding's text.
                d.push(
                    Diagnostic::info(DiagnosticCode::ClientEntityShadowed {
                        reason: Some(reason),
                    })
                    .phase(Phase::Validate)
                    .in_file(origin),
                );
                EngineVersionDecision::Ignore
            }
            // `Create`, `Replace`, and any step a newer core adds.
            _ => EngineVersionDecision::Replace,
        }
    };
    let contributions = parsed.into_iter().filter_map(|outcome| match outcome {
        FileOutcome::Entity(contribution) => Some(*contribution),
        FileOutcome::Attachable => None,
    });
    RegistryBuilder::combine(
        contributions,
        CombineRule::EngineVersionMerge {
            decide: Box::new(decide),
            merge: |earlier, mut later| {
                merge_slots(&mut earlier.slots(), &mut later.slots());
            },
        },
        CombineRule::top_wins(),
        stack.options,
    )
}

/// The loader of `rp.client_entities()`: `entity/**/*.json` of every resource pack and the
/// client entities of `attachables/**/*.json` and of the app's `definitions/attachables`, one
/// definition per identifier by `description.min_engine_version`.
#[doc(hidden)]
pub const LOADER: CollectionLoader<Registry<ClientEntity>, FileOutcome> = CollectionLoader {
    walk: WALK,
    parse: parse_walk_file,
    combine,
    needs: &[],
    then: None,
};

impl ClientEntity {
    /// A client entity for `identifier` with nothing to draw yet, at format version 1.26.0.
    pub fn new(identifier: EntityId) -> Self {
        Self {
            format_version: Self::VERSION,
            description: Description::new(identifier),
        }
    }
}

impl Description {
    /// A description of `identifier` with every other member absent.
    pub fn new(identifier: EntityId) -> Self {
        Self {
            identifier,
            min_engine_version: None,
            materials: IndexMap::new(),
            textures: IndexMap::new(),
            geometry: IndexMap::new(),
            queryable_geometry: None,
            spawn_egg: None,
            scripts: None,
            particle_effects: None,
            particle_emitters: None,
            animations: None,
            render_controllers: None,
            sound_effects: None,
            enable_attachables: None,
            held_item_ignores_lighting: None,
            held_item_scale: None,
            hide_armor: None,
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

    /// The documents the walk of a two-pack stack hands the combine for `identifiers` client
    /// entities, lower pack first: every identifier is defined twice, the second time
    /// alternately with the same `min_engine_version` (merged) and with a lower one (ignored).
    fn colliding(identifiers: usize) -> Vec<FileOutcome> {
        let cx = LoadContext::detached();
        let template = |mev: &str, texture: &str| {
            let text = format!(
                r#"{{"format_version": "1.26.0", "minecraft:client_entity": {{"description": {{"identifier": "demo:template", "min_engine_version": "{mev}", "textures": {{"default": "textures/{texture}"}}}}}}}}"#
            );
            <ClientEntity as Content>::parse(
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

#[cfg(test)]
mod reading_table {
    //! The family's reading table: what the pipeline reads each of its files and structs by.

    use bedrock_addon_core::versioning::testing::{assert_reading_table, reading_table};

    #[test]
    fn client_entity_reads_as_pinned() {
        assert_reading_table(
            "ClientEntityFamily",
            &reading_table::<super::ClientEntityFamily>(),
            r#"
                text: Root, clamps beyond a double: false, duplicates kept in: []
                envelope: Never
                missing version: at the floor, told at Warning: Some(Fixed("Warning: The \"format_version\" field was not given. Fields from format version 1.8.0 and older will be loaded. Update your targeted \"format_version\" to use fields from newer versions."))
                below floor: UseFormatVersion
                empty document: Error
                1.8.0..=1.26.0 (0..=2): json Own/Strict/Warn/lenient/Refuse, mcb Own/Strict/Warn/lenient/Refuse
            "#,
        );
    }
}
