//! Atomic client entities (`atomic_client_entities/**/*.json`, payload
//! `minecraft:atomic_client_entity`): a client-only entity type that exists only as a beta
//! document.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod codes;
mod read;
#[cfg(test)]
mod tests;

use bedrock_addon_core::__private::Sealed;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Code, Diagnostic, DiagnosticDetail, Diagnostics, Severity, builders,
};
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, JsonContent, NoBeta, Notice, Origin,
    Placed, Registry, RegistryBuilder, Rejected, UnknownMembers, default_path,
};
use bedrock_addon_core::id::kind;
use bedrock_addon_core::id::{
    AnimationRef, AtomicClientEntityId, GeometryId, MaterialId, ParticleId, SoundEventName,
    TexturePath,
};
use bedrock_addon_core::json::{
    EncodeError, JsonSlice, PrepareError, Prepared, Value, WriteStyle, last_wins, write_json,
};
use bedrock_addon_core::load::DocumentSource;
use bedrock_addon_core::load::{CollectionLoader, LoadContext, StackCx, Walk, WalkFile};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::schema::{self, Schema};
use bedrock_addon_core::version::{FormatVersion, known};
use bedrock_addon_core::versioning::{
    BETA_VERSION, BetaPermission, BeyondWidth, Envelope, Family, InputKind, Loaded, MissingVersion,
    NoComponents, NoStable, Reading, Scalars, StructIndex, load, write_envelope,
};
use bedrock_addon_shared_visual::{ControllerUse, HasVisuals, Visuals};
use facet::Facet;
use indexmap::IndexMap;

/// An atomic client entity: an entity that exists on clients only, with a model and optional
/// components (its box, and how a companion avoids and clips into blocks). It is no actor and has
/// no behaviour-pack counterpart, and its identifiers are a space of their own, apart from client
/// entities and attachables.
///
/// The format is beta-only: the one accepted `format_version` is `"beta"`, from any pack and
/// without the Beta APIs toggle. Every member not listed is an error; vanilla skips the file.
///
/// ```
/// use bedrock_addon_family_atomic_client_entity::{AabbDimensions, AtomicClientEntity, Components};
/// use bedrock_addon_core::id::AtomicClientEntityId;
///
/// let wisp = AtomicClientEntity {
///     components: Some(Components {
///         aabb_dimensions: Some(AabbDimensions::new(0.5, 0.5)),
///         ..Components::default()
///     }),
///     ..AtomicClientEntity::new(AtomicClientEntityId::parse("demo:wisp"))
/// };
/// assert_eq!(wisp.description.identifier.as_str(), "demo:wisp");
/// ```
#[doc(alias = "minecraft:atomic_client_entity")]
#[doc(alias = "atomic_client_entity")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct AtomicClientEntity {
    /// `description`: the identifier and what the client draws.
    pub description: AtomicDescription,
    /// `components`: the entity's components.
    pub components: Option<Components>,
}

/// `description` of an atomic client entity: the description of a client entity cut down to
/// the model. Names are kept as written; vanilla lower-cases nothing here.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct AtomicDescription {
    /// `identifier`: `namespace:name`; any namespace, `minecraft` included.
    pub identifier: AtomicClientEntityId,
    /// `materials`: slot name → material. `None`, here and in the members below, is a
    /// description without the member; an empty one the author wrote is kept.
    pub materials: Option<IndexMap<String, MaterialId>>,
    /// `textures`: slot name → texture path.
    pub textures: Option<IndexMap<String, TexturePath>>,
    /// `geometry`: slot name → geometry.
    pub geometry: Option<IndexMap<String, GeometryId>>,
    /// `animations`: short name → animation or animation controller.
    pub animations: Option<IndexMap<String, AnimationRef>>,
    /// `particle_effects`: short name → particle effect.
    pub particle_effects: Option<IndexMap<String, ParticleId>>,
    /// `sound_effects`: short name → sound event.
    pub sound_effects: Option<IndexMap<String, SoundEventName>>,
    /// `scripts`: the Molang the client runs and the animations it plays.
    pub scripts: Option<AtomicScripts>,
    /// `render_controllers`: the render controllers that draw the entity, in order. Names
    /// only: every entry is [`ControllerUse::Always`], and the `{ name: condition }` form is an
    /// error.
    #[facet(schema::molang_with = no_conditions)]
    pub render_controllers: Option<Vec<ControllerUse>>,
}

/// The render controllers' walk: the list holds names only, so no condition is compiled, the
/// object form a hand-built value may hold included.
fn no_conditions(
    _: &mut Option<Vec<ControllerUse>>,
    _: &mut bedrock_addon_core::molang::MolangVisitor<'_>,
) {
}

/// `scripts` of an atomic client entity: three lists of plain strings. The Molang statements
/// are kept as text; nothing compiles them when the document loads.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct AtomicScripts {
    /// `initialize`: Molang statements run once.
    pub initialize: Option<Vec<String>>,
    /// `pre_animation`: Molang statements run before the animations.
    pub pre_animation: Option<Vec<String>>,
    /// `animate`: the animations to play, by their `animations` short name; names only.
    pub animate: Option<Vec<String>>,
}

/// `components` of an atomic client entity. The engine knows three components; any other key
/// is an error (the behaviour-pack entity's `minecraft:collision_box` among them).
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct Components {
    /// `minecraft:aabb_dimensions`.
    #[facet(rename = "minecraft:aabb_dimensions")]
    pub aabb_dimensions: Option<AabbDimensions>,
    /// `minecraft:companion_block_avoidance`.
    #[facet(rename = "minecraft:companion_block_avoidance")]
    pub companion_block_avoidance: Option<CompanionBlockAvoidance>,
    /// `minecraft:companion_block_clipping`.
    #[facet(rename = "minecraft:companion_block_clipping")]
    pub companion_block_clipping: Option<CompanionBlockClipping>,
}

/// `minecraft:aabb_dimensions` of an atomic client entity ("AABB dimensions for Atomic Client
/// Entities."): a box `width` wide and deep and `height` high, standing on the entity's
/// position. Not the behaviour-pack entity's `minecraft:collision_box`. The client exposes the
/// box to Molang as `variable.aabb_dimension`, `variable.aabb_dimension_x` and
/// `variable.aabb_dimension_y`.
#[doc(alias = "minecraft:aabb_dimensions")]
#[derive(Clone, Copy, Debug, PartialEq, Facet, Schema)]
pub struct AabbDimensions {
    /// `width`: at least 0.
    pub width: f32,
    /// `height`: at least 0.
    pub height: f32,
}

/// An alias of [`AabbDimensions`], the box a `minecraft:aabb_dimensions` component holds.
pub type CollisionBox = AabbDimensions;

impl AabbDimensions {
    /// A box `width` wide and deep and `height` high.
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// The box around the entity's position as vanilla builds it: `(min, max)` corners, half
    /// the width to each side and from 0 up to the height.
    pub fn bounds(&self) -> ([f32; 3], [f32; 3]) {
        // Half the width is the extent to each side; each corner is the component-wise minimum
        // and maximum of the two signed extents.
        let half = 0.5 * self.width;
        let (low, high) = (half.min(-half), half.max(-half));
        let (bottom, top) = (self.height.min(0.0), self.height.max(0.0));
        ([low, bottom, low], [high, top, high])
    }
}

/// `minecraft:companion_block_avoidance` ("Defines fallback anchor offsets used when the
/// primary companion follow target clips into blocks."). A member left out is `None` and the
/// client holds its default (the accessors); the write-back leaves it out.
#[doc(alias = "minecraft:companion_block_avoidance")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct CompanionBlockAvoidance {
    /// `backup_anchor_offsets`: the fallback offsets, each `[x, y, z]`; required.
    pub backup_anchor_offsets: Vec<[f32; 3]>,
    /// `require_hidden_before_fallback`: default false.
    #[facet(schema::fallback = Self::DEFAULT_REQUIRE_HIDDEN_BEFORE_FALLBACK)]
    pub require_hidden_before_fallback: Option<bool>,
    /// `anchor_switch_cooldown`: an unsigned 16-bit count, default 200.
    #[facet(schema::fallback = Self::DEFAULT_ANCHOR_SWITCH_COOLDOWN)]
    pub anchor_switch_cooldown: Option<u16>,
}

impl CompanionBlockAvoidance {
    /// `require_hidden_before_fallback` of a component that writes none.
    pub const DEFAULT_REQUIRE_HIDDEN_BEFORE_FALLBACK: bool = false;
    /// `anchor_switch_cooldown` of a component that writes none.
    pub const DEFAULT_ANCHOR_SWITCH_COOLDOWN: u16 = 200;

    /// A component with `offsets` and every other member left out.
    pub fn new(offsets: Vec<[f32; 3]>) -> Self {
        Self {
            backup_anchor_offsets: offsets,
            require_hidden_before_fallback: None,
            anchor_switch_cooldown: None,
        }
    }

    /// The offsets as floats.
    pub fn to_offsets(&self) -> Vec<[f32; 3]> {
        self.backup_anchor_offsets.clone()
    }
}

/// `minecraft:companion_block_clipping` ("Defines companion block clipping behavior,
/// transition effects, and hysteresis controls."). Every member may be left out; the
/// accessors give what the client holds. Each float is at least 0.
#[doc(alias = "minecraft:companion_block_clipping")]
#[derive(Clone, Copy, Debug, PartialEq, Facet, Schema)]
pub struct CompanionBlockClipping {
    /// `reveal_boundary`.
    #[facet(schema::fallback = VisibilityBoundary::default())]
    pub reveal_boundary: Option<VisibilityBoundary>,
    /// `hide_boundary`.
    #[facet(schema::fallback = VisibilityBoundary::default())]
    pub hide_boundary: Option<VisibilityBoundary>,
    /// `min_hidden_seconds`: default 2.
    #[facet(schema::fallback = Self::DEFAULT_MIN_HIDDEN_SECONDS)]
    pub min_hidden_seconds: Option<f32>,
    /// `min_visible_seconds`: default 0.
    #[facet(schema::fallback = Self::DEFAULT_MIN_VISIBLE_SECONDS)]
    pub min_visible_seconds: Option<f32>,
    /// `hide_delay_seconds`: default 0.
    #[facet(schema::fallback = Self::DEFAULT_HIDE_DELAY_SECONDS)]
    pub hide_delay_seconds: Option<f32>,
    /// `hide_transition_seconds`: default 0.25.
    #[facet(schema::fallback = Self::DEFAULT_HIDE_TRANSITION_SECONDS)]
    pub hide_transition_seconds: Option<f32>,
}

// The defaults the client holds for a component that writes none: the reveal and the hide
// boundary are each (0.5, 0.1, 1.0), then 2.0, 0.0, 0.0 and 0.25 seconds in declaration order.
impl CompanionBlockClipping {
    /// `min_hidden_seconds` of a component that writes none.
    pub const DEFAULT_MIN_HIDDEN_SECONDS: f32 = 2.0;
    /// `min_visible_seconds` of a component that writes none.
    pub const DEFAULT_MIN_VISIBLE_SECONDS: f32 = 0.0;
    /// `hide_delay_seconds` of a component that writes none.
    pub const DEFAULT_HIDE_DELAY_SECONDS: f32 = 0.0;
    /// `hide_transition_seconds` of a component that writes none.
    pub const DEFAULT_HIDE_TRANSITION_SECONDS: f32 = 0.25;
}

/// A companion visibility boundary (`minecraft:companion_visibility_boundary`): the
/// `reveal_boundary` and `hide_boundary` of [`CompanionBlockClipping`]. Every member may be
/// left out; each is at least 0.
#[derive(Clone, Copy, Debug, PartialEq, Facet, Schema)]
pub struct VisibilityBoundary {
    /// `margin`: default 0.5.
    #[facet(schema::fallback = Self::DEFAULT_MARGIN)]
    pub margin: Option<f32>,
    /// `lookahead_per_speed`: default 0.1.
    #[facet(schema::fallback = Self::DEFAULT_LOOKAHEAD_PER_SPEED)]
    pub lookahead_per_speed: Option<f32>,
    /// `lookahead_max`: default 1.
    #[facet(schema::fallback = Self::DEFAULT_LOOKAHEAD_MAX)]
    pub lookahead_max: Option<f32>,
}

// The defaults the client holds for a boundary that writes none: a margin of 0.5, a lookahead
// per speed of 0.1 and a lookahead maximum of 1.0.
impl VisibilityBoundary {
    /// `margin` of a boundary that writes none.
    pub const DEFAULT_MARGIN: f32 = 0.5;
    /// `lookahead_per_speed` of a boundary that writes none.
    pub const DEFAULT_LOOKAHEAD_PER_SPEED: f32 = 0.1;
    /// `lookahead_max` of a boundary that writes none.
    pub const DEFAULT_LOOKAHEAD_MAX: f32 = 1.0;
}

impl AtomicClientEntity {
    /// An atomic client entity for `identifier` with nothing to draw and no component.
    pub fn new(identifier: AtomicClientEntityId) -> Self {
        Self {
            description: AtomicDescription::new(identifier),
            components: None,
        }
    }

    /// `minecraft:aabb_dimensions`, when the entity has one.
    pub fn aabb_dimensions(&self) -> Option<&AabbDimensions> {
        self.components.as_ref()?.aabb_dimensions.as_ref()
    }

    /// `minecraft:companion_block_avoidance`, when the entity has one.
    pub fn companion_block_avoidance(&self) -> Option<&CompanionBlockAvoidance> {
        self.components.as_ref()?.companion_block_avoidance.as_ref()
    }

    /// `minecraft:companion_block_clipping`, when the entity has one.
    pub fn companion_block_clipping(&self) -> Option<&CompanionBlockClipping> {
        self.components.as_ref()?.companion_block_clipping.as_ref()
    }
}

impl AtomicDescription {
    /// A description of `identifier` with every other member absent.
    pub fn new(identifier: AtomicClientEntityId) -> Self {
        Self {
            identifier,
            materials: None,
            textures: None,
            geometry: None,
            animations: None,
            particle_effects: None,
            sound_effects: None,
            scripts: None,
            render_controllers: None,
        }
    }
}

impl HasVisuals for AtomicClientEntity {
    fn visuals(&self) -> Visuals<'_> {
        let d = &self.description;
        Visuals::new(
            d.materials.as_ref(),
            d.textures.as_ref(),
            d.geometry.as_ref(),
            d.animations.as_ref(),
            d.render_controllers.as_deref().unwrap_or_default(),
        )
    }
}

/// The directory atomic client entity files live in.
const DIRECTORY: &str = "atomic_client_entities";
/// The payload key.
const PAYLOAD: &str = "minecraft:atomic_client_entity";

/// The atomic client entity family of the version-chain machinery (hidden: only the loader
/// names it).
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct AtomicFamily;

bedrock_addon_core::version_chain! {
    family = AtomicFamily;
    /// An atomic client entity document as parsed: the beta struct is the only one.
    #[doc(hidden)]
    #[derive(Clone, Debug, PartialEq)]
    pub enum AtomicHistory {}
    beta =
        /// `"format_version": "beta"`.
        Beta(AtomicClientEntity);
}

impl Family for AtomicFamily {
    const NAME: &'static str = "atomic_client_entity";
    const PAYLOAD_KEY: &'static str = PAYLOAD;
    // Unused: there is no stable struct, so every version but `"beta"` has no parser.
    const FLOOR: FormatVersion = known::V0_0_0;
    type Latest = NoStable;
    type Beta = AtomicClientEntity;
    // The game builds the load context with the beta permission on, whatever the pack and the
    // experiments.
    const BETA_PERMISSION: BetaPermission = BetaPermission::Always;
    // The one component is a member of `Components`; the generic component checks run on a
    // stable struct only, which this family does not have.
    type Component = NoComponents;
    type History = AtomicHistory;
    const ENVELOPE: Envelope = Envelope {
        // The game reads the file through the document-input reader of the cereal-based families:
        // the version is read from the document, and a member written `null` fails its type (the
        // reader applies that rule itself).
        input: InputKind::Document,
        // A document without a version fails with "no valid version passed by the caller or found
        // in the input".
        missing: MissingVersion::REJECT,
        ..Envelope::DEFAULT
    };
    // An empty file is skipped silently.
    const EMPTY_DOCUMENT: Severity = Severity::Info;

    /// The document-input profile, its scalars read strictly: a member takes its own JSON type
    /// alone, and a number beyond its float is a type error (the floats with a minimum are
    /// clamped in the text before the read).
    fn reading(_: StructIndex, _: DocumentSource) -> Reading {
        AtomicFamily::ENVELOPE
            .input
            .profile()
            .with_scalars(Scalars::Strict)
            .with_beyond_width(BeyondWidth::Error)
    }
}

// The quartet is hand-written: the family is beta-only (`Latest = NoStable`), so the struct is
// its `Beta`, which `JsonDocument` (`Family<Latest = Self>`) cannot name, and `load::registry`
// (a `JsonDefinition`) does not apply.
impl Sealed for AtomicClientEntity {}

impl Content for AtomicClientEntity {
    const NAME: &'static str = "atomic client entity";
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// Parses one atomic client entity file. Only `"format_version": "beta"` loads.
    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        parse_file(PackRef::app_data(), PackSide::Resource, path, bytes, cx).1
    }

    /// `{"format_version": "beta", "minecraft:atomic_client_entity": ..}`.
    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        let payload = bedrock_addon_core::versioning::write_payload(self)?;
        let root =
            write_envelope::<AtomicFamily>(BETA_VERSION, payload, &UnknownMembers::default());
        write_json(&root, out, style)
    }
}

impl JsonContent for AtomicClientEntity {}

impl Definition for AtomicClientEntity {
    type Kind = kind::AtomicClientEntity;
    type File = AtomicClientEntity;
    type Beta = NoBeta<kind::AtomicClientEntity>;
}

impl Placed for AtomicClientEntity {
    /// `atomic_client_entities/<namespace>/<name>.json`.
    fn default_path(&self) -> PackPath {
        default_path(DIRECTORY, &self.description.identifier)
    }
}

/// Gives every finding the document's pack and file, and the envelope's findings the game's
/// texts: a missing payload its text, and a malformed version the "no valid version" line that
/// follows it.
fn place(found: Diagnostics, origin: &Origin) -> Diagnostics {
    let mut placed = Diagnostics::default();
    for mut finding in found {
        // By legacy name: the variant and the definitions that were it.
        let malformed = matches!(finding.code.name(), "MalformedFormatVersion");
        if finding.is(&core_codes::MISSING_PAYLOAD)
            || finding
                .code
                .is(&bedrock_addon_core::diagnostic::codes::MISSING_PAYLOAD)
        {
            // A missing payload gets the game's text.
            finding.code = bedrock_addon_core::diagnostic::cereal::payload_missing();
        }
        let location = finding.location.clone();
        placed.push(finding);
        if malformed {
            // The game writes "Malformed format version", then the "no valid version" text.
            let mut missing = Diagnostic::of(&core_codes::NO_VALID_VERSION);
            missing.location = location;
            placed.push(missing);
        }
    }
    placed.located(origin)
}

/// The envelope's member-count rule for a root without `format_version`: a root that holds
/// only the payload fails "exactly two members" before the missing version is looked at.
fn member_count(text: &str) -> Option<Diagnostic> {
    let cleaned = last_wins(text);
    let root = bedrock_addon_core::json::from_str_jsonc::<Value>(&cleaned).ok()?;
    let root = root.as_object()?;
    (root.get("format_version").is_none() && root.len() != 2).then(|| {
        Diagnostic::of(&core_codes::DOCUMENT_MEMBER_COUNT).detail(DiagnosticDetail::Syntax(
            format!(
                "{} root members, not \"format_version\" and a payload member",
                root.len()
            ),
        ))
    })
}

/// Loads one atomic client entity file: the document or its rejection.
fn parse_file(
    pack: PackRef,
    side: PackSide,
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> (
    Option<AtomicClientEntityId>,
    Result<Document<AtomicClientEntity>, Rejected>,
) {
    let origin = Origin::file(pack, side, path.clone());
    let reject = |found: Diagnostic| {
        let diagnostics: Diagnostics = [found].into_iter().collect();
        Rejected::new(origin.clone(), place(diagnostics, &origin))
    };
    let malformed = |message: &str| reject(Diagnostic::of(builders::syntax(message)));
    let prepared = match Prepared::new(bytes) {
        Ok(prepared) => prepared,
        Err(PrepareError::NotUtf8(_)) => {
            return (None, Err(malformed("the file is not UTF-8")));
        }
        Err(PrepareError::TooDeep(_) | PrepareError::Empty) => {
            // `new` never refuses empty input.
            return (None, Err(malformed("the JSON nests too deeply")));
        }
    };
    let text = prepared.as_str();
    if let Some(found) = member_count(text) {
        return (None, Err(reject(found)));
    }
    let outcome = load::<AtomicFamily>(JsonSlice::new(text), cx);
    let diagnostics = place(outcome.diagnostics, &origin);
    match outcome.value {
        Some(Loaded::Beta(entity)) => {
            let id = entity.description.identifier.clone();
            let mut version = outcome.version;
            version.parsed_as = BETA_VERSION;
            (
                Some(id),
                Ok(Document::new(entity, origin, version, diagnostics)),
            )
        }
        _ => (None, Err(Rejected::new(origin, diagnostics))),
    }
}

// ---- the collection ----------------------------------------------------------------------

/// `atomic_client_entities/**/*.json`, packs from the lowest priority to the highest: the
/// extension is exactly `json`; any other file of the directory is skipped silently.
const WALK: Walk = Walk::json_tree_in(&[DIRECTORY]);

fn parse_walk_file(file: &WalkFile, cx: &LoadContext) -> Contribution<AtomicClientEntity> {
    match parse_file(file.pack.clone(), file.side, &file.path, &file.bytes, cx) {
        (Some(id), Ok(document)) => Contribution::Stable(id, document),
        (id, Err(rejected)) => Contribution::Rejected(id, rejected),
        // A loaded document always has its identifier.
        (None, Ok(document)) => {
            Contribution::Stable(document.description.identifier.clone(), document)
        }
    }
}

/// One definition per identifier: every loaded document assigns its identifier, so the last
/// one in walk order replaces the earlier one whole (the highest-priority pack's, and inside a
/// pack the last file's). No member is merged and `min_engine_version` plays no part. Each
/// replaced document gets an `AtomicClientEntityShadowed` info (the game is silent here, so
/// the finding is the crate's own). The registry assigns by identifier, so the later document
/// replaces the earlier one.
fn combine(
    parsed: Vec<Contribution<AtomicClientEntity>>,
    stack: &StackCx<'_>,
) -> Registry<AtomicClientEntity> {
    RegistryBuilder::combine(
        parsed,
        CombineRule::LastLoadedWins {
            replaced: Some(Notice {
                code: Code::from(&core_codes::ATOMIC_CLIENT_ENTITY_SHADOWED),
                severity: Severity::Info,
            }),
        },
        CombineRule::top_wins(),
        stack.options,
    )
}

/// The loader of `rp.atomic_client_entities()`: `atomic_client_entities/**/*.json` of every
/// resource pack, the last document of an identifier winning.
#[doc(hidden)]
pub const LOADER: CollectionLoader<Registry<AtomicClientEntity>, Contribution<AtomicClientEntity>> =
    CollectionLoader {
        walk: WALK,
        parse: parse_walk_file,
        combine,
        needs: &[],
        then: None,
    };

#[cfg(test)]
mod reading_table {
    //! The family's reading table: what the pipeline reads each of its files and structs by.

    use bedrock_addon_core::versioning::testing::{assert_reading_table, reading_table};

    #[test]
    fn atomic_client_entity_reads_as_pinned() {
        assert_reading_table(
            "AtomicFamily",
            &reading_table::<super::AtomicFamily>(),
            r"
                text: Document, clamps beyond a double: false, duplicates kept in: []
                envelope: Always
                missing version: rejected: NoValidVersion
                below floor: UseFormatVersion
                empty document: Info
                beta: json Rejected/Strict/Error/Error/reports/fatal/Refuse, mcb Own/Strict/Error/Error/reports/fatal/Refuse
            ",
        );
    }
}
