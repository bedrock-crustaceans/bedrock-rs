//! Typed identifiers: an identifier is `Id<K>`, where the marker `K` says what it names, so a
//! registry indexed by its kind rejects an identifier of another kind at compile time. Kinds
//! live here, always built, so a reference type exists even when the target family's feature
//! is off.
//!
//! Parsing is lenient, construction is strict: [`Id::new`] rejects malformed text with an
//! [`IdError`], while [`Id::parse`] (and deserialising a document) keeps any text as written,
//! so a file vanilla accepts is never dropped by the type; [`Id::is_well_formed`] says which.

mod facet_proxy;
mod syntax;

use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::sync::Arc;

use syntax::{Canonical, canonical, split};

/// An identifier of kind `K`: the text as written, compared in the kind's canonical form.
///
/// Cheap to clone (static or `Arc<str>`). `Hash` / `Eq` use the canonical form (namespace
/// default and ASCII case folding where the kind's vanilla lookup folds), so `stone` and
/// `minecraft:stone` are equal `BlockId`s while `as_str` keeps what was written. An entity
/// identifier compares without its `<event>` suffix and a legacy item identifier without its
/// `:aux` suffix ([`Suffix`]).
///
/// In a document, an identifier is a JSON string, read leniently and written back as written.
#[derive(facet::Facet)]
#[facet(opaque, proxy = facet_proxy::IdProxy)]
pub struct Id<K: Kind> {
    text: Text,
    kind: PhantomData<fn() -> K>,
}

/// The text of an identifier: a literal or a shared allocation.
#[derive(Clone)]
enum Text {
    Static(&'static str),
    Shared(Arc<str>),
}

impl Text {
    fn as_str(&self) -> &str {
        match self {
            Self::Static(text) => text,
            Self::Shared(text) => text,
        }
    }
}

impl<K: Kind> Id<K> {
    /// Validates `text` against `K`'s syntax.
    pub fn new(text: impl Into<Arc<str>>) -> Result<Self, IdError> {
        let text = text.into();
        syntax::validate(&text, K::SYNTAX, K::SUFFIX)?;
        Ok(Self::shared(text))
    }

    /// Keeps `text` as written, whatever it is: the loader's constructor. A malformed text is an
    /// identifier that [`is_well_formed`](Self::is_well_formed) reports and that compares by
    /// the same canonical rules as a valid one.
    pub fn parse(text: impl Into<Arc<str>>) -> Self {
        Self::shared(text.into())
    }

    /// Validated at compile time: `const RUBY: BlockId = Id::from_static("demo:ruby_block");`
    ///
    /// Outside a `const` the check runs when the call does and panics on a malformed text.
    ///
    /// ```compile_fail
    /// use bedrock_addon_core::id::{BlockId, Id};
    /// const BAD: BlockId = Id::from_static("demo:bad block");
    /// ```
    pub const fn from_static(text: &'static str) -> Self {
        assert!(
            syntax::validate(text, K::SYNTAX, K::SUFFIX).is_ok(),
            "malformed identifier literal"
        );
        Self {
            text: Text::Static(text),
            kind: PhantomData,
        }
    }

    fn shared(text: Arc<str>) -> Self {
        Self {
            text: Text::Shared(text),
            kind: PhantomData,
        }
    }

    /// The text as written.
    pub fn as_str(&self) -> &str {
        self.text.as_str()
    }

    /// `Some("demo")` for `demo:x`; `None` for kinds without namespaces or a bare id.
    pub fn namespace(&self) -> Option<&str> {
        match K::SYNTAX {
            Syntax::Namespaced { .. } => split(self.as_str(), K::SUFFIX).0,
            _ => None,
        }
    }

    /// The part after the namespace (or the whole text). An entity's `<event>` and an item's
    /// `:aux` suffix are not part of it.
    pub fn path(&self) -> &str {
        match K::SYNTAX {
            Syntax::Namespaced { .. } => split(self.as_str(), K::SUFFIX).1,
            _ => self.as_str(),
        }
    }

    /// The namespace is `minecraft` (explicit or defaulted).
    pub fn is_vanilla(&self) -> bool {
        match K::SYNTAX {
            Syntax::Namespaced { default_vanilla } => match self.namespace() {
                Some(namespace) => namespace == VANILLA_NAMESPACE,
                None => default_vanilla,
            },
            _ => false,
        }
    }

    /// The text satisfies `K`'s syntax (the loader keeps malformed identifiers as written).
    pub fn is_well_formed(&self) -> bool {
        syntax::validate(self.as_str(), K::SYNTAX, K::SUFFIX).is_ok()
    }

    /// Converts between kinds that share an identifier space (e.g. `EntityId` → spawn rules key).
    pub fn cast<L: Kind<Space = K::Space>>(self) -> Id<L> {
        Id {
            text: self.text,
            kind: PhantomData,
        }
    }

    fn canonical(&self) -> Canonical<'_> {
        canonical(self.as_str(), K::SYNTAX, K::SUFFIX)
    }
}

/// The namespace vanilla content uses and a bare identifier defaults to.
const VANILLA_NAMESPACE: &str = "minecraft";

impl Id<kind::Entity> {
    /// The spawn event of an `ns:name<ns:event>` reference (item `entity_placer.entity`, the
    /// aim-assist exclusion list): `Some("minecraft:entity_born")` for
    /// `minecraft:pig<minecraft:entity_born>`, `None` without a complete `<…>` suffix. The
    /// identifier compares without it.
    pub fn event(&self) -> Option<&str> {
        let text = self.as_str();
        let open = text.find('<')?;
        text[open + 1..].strip_suffix('>')
    }
}

impl Id<kind::Item> {
    /// The aux value of a legacy `ns:name:aux` item text (`minecraft:wool:3` → `Some(3)`):
    /// the last of three or more `:`-separated parts, when it is an `i16`. The identifier
    /// compares without that part, as vanilla's item reference registers it.
    pub fn aux(&self) -> Option<i16> {
        let text = self.as_str();
        if text.matches(':').count() < 2 {
            return None;
        }
        let (_, aux) = text.rsplit_once(':')?;
        aux.parse().ok()
    }
}

impl<K: Kind> Clone for Id<K> {
    fn clone(&self) -> Self {
        Self {
            text: self.text.clone(),
            kind: PhantomData,
        }
    }
}

impl<K: Kind> PartialEq for Id<K> {
    fn eq(&self, other: &Self) -> bool {
        self.canonical() == other.canonical()
    }
}

impl<K: Kind> Eq for Id<K> {}

impl<K: Kind> Hash for Id<K> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.canonical().hash(state);
    }
}

/// `Id<block>("stone")`: the kind's name and the text as written.
impl<K: Kind> fmt::Debug for Id<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Id<{}>({:?})", K::NAME, self.as_str())
    }
}

/// The text as written.
impl<K: Kind> fmt::Display for Id<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<K: Kind> AsRef<str> for Id<K> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// What an identifier names and how it is spelled. Implemented by the markers in [`kind`].
pub trait Kind: 'static {
    /// `"block"`, used in messages.
    const NAME: &'static str;
    /// How the identifier is spelled.
    const SYNTAX: Syntax;
    /// A suffix the canonical form ignores (entity events, item aux values); none by default.
    const SUFFIX: Suffix = Suffix::None;
    /// Kinds with the same `Space` share an identifier space and can [`Id::cast`]; they share
    /// `SYNTAX` and `SUFFIX` too, so a cast identifier keeps its key.
    type Space;
}

/// How an identifier kind is spelled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Syntax {
    /// `namespace:path`; `minecraft:` assumed when bare if `default_vanilla`, otherwise the
    /// namespace is required. Case-sensitive.
    Namespaced {
        /// A bare identifier reads as `minecraft:<path>`.
        default_vanilla: bool,
    },
    /// A dotted name with a fixed prefix: `geometry.`, `animation.`, `controller.animation.`,
    /// `controller.render.`.
    Prefixed {
        /// The prefix, with its trailing dot.
        prefix: &'static str,
        /// The vanilla lookup folds ASCII case (the prefix included).
        fold_case: bool,
    },
    /// A pack-relative path (`textures/blocks/stone`) under the pack-path rules: no `.` or `..`
    /// segment, no ASCII control character or `:`, at most 64 segments and 1023 bytes. Compared
    /// as `PackPath` normalises it (`\` and repeated separators are one `/`), case-sensitively;
    /// whether it carries an extension is the kind's convention.
    Path,
    /// Any non-empty string (sound events, material names, scene tags).
    Free,
}

/// A suffix a namespaced identifier may carry after its path, which the canonical form ignores.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Suffix {
    /// No suffix.
    None,
    /// An entity spawn event, `ns:name<ns:event>`: the identifier is the text before the first
    /// `<` ([`Id::event`]).
    Event,
    /// A legacy item aux value, `ns:name:aux`: with three or more `:`-separated parts the
    /// identifier is the text before the last `:` ([`Id::aux`]).
    Aux,
}

/// Why `Id::new` rejected a text.
#[derive(Clone, Debug, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum IdError {
    /// The text, or a part the syntax needs (namespace, path, the name after a prefix), is empty.
    #[error("empty identifier")]
    Empty,
    /// The kind needs `namespace:path` and no namespace was written.
    #[error("missing namespace")]
    MissingNamespace,
    /// A character the kind's syntax does not allow (ASCII whitespace or an ASCII control
    /// character, a second `:`, a `<` / `>` outside an entity event, a `.` / `..` path segment, a
    /// bad aux value).
    #[error("invalid character {ch:?} at byte {index}")]
    InvalidChar {
        /// Byte index of the character.
        index: usize,
        /// The character.
        ch: char,
    },
    /// The kind needs a fixed prefix (`geometry.`) and the text starts with another.
    #[error("expected the prefix {expected:?}")]
    WrongPrefix {
        /// The prefix the kind requires.
        expected: &'static str,
    },
    /// The text exceeds the kind's length limit: 65 535 bytes, or the pack-path limits for a
    /// path kind.
    #[error("identifier too long")]
    TooLong,
}

/// The identifier kinds: one marker per thing an identifier can name.
pub mod kind {
    use super::{Kind, Suffix, Syntax};

    const VANILLA: Syntax = Syntax::Namespaced {
        default_vanilla: true,
    };
    const NAMESPACED: Syntax = Syntax::Namespaced {
        default_vanilla: false,
    };

    macro_rules! kinds {
        ($( $(#[$meta:meta])* $name:ident = $text:literal, $syntax:expr $(, suffix = $suffix:expr)? $(, space = $space:ty)?; )*) => {
            $(
                $(#[$meta])*
                #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
                pub enum $name {}

                impl Kind for $name {
                    const NAME: &'static str = $text;
                    const SYNTAX: Syntax = $syntax;
                    $(const SUFFIX: Suffix = $suffix;)?
                    type Space = kinds!(@space $name $(, $space)?);
                }
            )*
        };
        (@space $name:ident) => { $name };
        (@space $name:ident, $space:ty) => { $space };
    }

    // The syntax of each kind: what the game accepts for that kind of key.
    kinds! {
        /// A block (`minecraft:stone`).
        Block = "block", VANILLA;
        /// An item (`minecraft:apple`; legacy `minecraft:wool:3`).
        Item = "item", VANILLA, suffix = Suffix::Aux;
        /// An entity (`minecraft:zombie`; with a spawn event `minecraft:pig<minecraft:entity_born>`).
        Entity = "entity", VANILLA, suffix = Suffix::Event;
        /// The spawn rules of an entity: the entity's identifier space and suffix rule.
        SpawnRules = "spawn rules", VANILLA, suffix = Suffix::Event, space = Entity;
        /// A biome (`minecraft:plains`).
        Biome = "biome", VANILLA;
        /// A geometry (`geometry.zombie`).
        Geometry = "geometry", Syntax::Prefixed { prefix: "geometry.", fold_case: true };
        /// A texture path (`textures/blocks/stone`).
        Texture = "texture", Syntax::Path;
        /// A material name.
        Material = "material", Syntax::Free;
        /// A loot table path (`loot_tables/chests/simple_dungeon.json`).
        LootTable = "loot table", Syntax::Path;
        /// A structure (`mystructure:house`).
        Structure = "structure", NAMESPACED;
        /// An animation or animation controller (`animation.zombie.walk`, `controller.animation.x`).
        Animation = "animation", Syntax::Free;
        /// A world-generation feature (`minecraft:oak_tree_feature`).
        Feature = "feature", NAMESPACED;
        /// A feature rule (`minecraft:overworld_after_surface_flowers_feature_rules`).
        FeatureRule = "feature rule", NAMESPACED;
        /// A jigsaw template pool (`minecraft:trail_ruins/tower`).
        TemplatePool = "template pool", NAMESPACED;
        /// A jigsaw processor list.
        ProcessorList = "processor list", NAMESPACED;
        /// A jigsaw structure.
        JigsawStructure = "jigsaw structure", NAMESPACED;
        /// A jigsaw structure set.
        StructureSet = "structure set", NAMESPACED;
        /// A dimension (`minecraft:overworld`).
        Dimension = "dimension", NAMESPACED;
        /// A villager trade table path (`trading/economy_trades/farmer_trades.json`).
        TradeTable = "trade table", Syntax::Path;
        /// A camera preset (`minecraft:free`).
        CameraPreset = "camera preset", NAMESPACED;
        /// A camera list.
        CameraList = "camera list", NAMESPACED;
        /// An aim-assist preset.
        AimAssistPreset = "aim assist preset", NAMESPACED;
        /// An aim-assist category (`minecraft:default`).
        AimAssistCategory = "aim assist category", NAMESPACED;
        /// A particle effect (`minecraft:basic_flame_particle`).
        Particle = "particle", Syntax::Free;
        /// A sound event of `sound_definitions.json` (`random.click`).
        SoundEvent = "sound event", Syntax::Free;
        /// A music definition of `music_definitions.json` (`game`).
        MusicDefinition = "music definition", Syntax::Free;
        /// Fog settings (`minecraft:fog_default`).
        Fog = "fog", Syntax::Free;
        /// Deferred lighting settings.
        LightingSettings = "lighting settings", Syntax::Free;
        /// Deferred water settings.
        WaterSettings = "water settings", Syntax::Free;
        /// Deferred atmosphere settings.
        AtmosphereSettings = "atmosphere settings", Syntax::Free;
        /// Deferred colour-grading settings.
        ColorGradingSettings = "color grading settings", Syntax::Free;
        /// Deferred cubemap settings.
        CubemapSettings = "cubemap settings", Syntax::Free;
        /// A render controller (`controller.render.zombie`).
        RenderController = "render controller", Syntax::Prefixed { prefix: "controller.render.", fold_case: true };
        /// A key of the terrain or item texture atlas (`stone`, `apple`).
        AtlasTexture = "atlas key", Syntax::Free;
        /// A dialogue scene tag.
        DialogueScene = "dialogue scene", Syntax::Free;
        /// A function path under `functions/`, without extension (`folder/name`).
        Function = "function", Syntax::Path;
        /// A translation key of the `.lang` files (`item.apple.name`).
        Translation = "lang key", Syntax::Free;
        /// A native script module (`@minecraft/server`).
        ScriptModule = "script module", Syntax::Free;
        /// A data-driven UI extension point.
        ExtensionPoint = "extension point", Syntax::Free;
        /// A voxel shape (`minecraft:unit_cube`).
        VoxelShape = "voxel shape", Syntax::Free;
        /// A recipe.
        Recipe = "recipe", Syntax::Free;
        /// A behavior tree (`minecraft:name`).
        BehaviorTree = "behavior tree", Syntax::Free;
        /// A point-of-interest type (`poi_block`).
        PoiType = "poi type", NAMESPACED;
        /// A point-of-interest tag.
        PoiTag = "poi tag", NAMESPACED;
        /// Block culling rules.
        CullingRules = "culling rules", NAMESPACED;
        /// An attachable.
        Attachable = "attachable", Syntax::Free;
        /// An atomic client entity (its own identifier space, not the entity one).
        AtomicClientEntity = "atomic client entity", NAMESPACED;
        /// A raid spawn group (`minecraft:raid_group_1`).
        SpawnGroup = "spawn group", Syntax::Free;
        /// A data-driven UI root.
        UiRoot = "ui root", NAMESPACED;
        /// A data-driven UI composition.
        UiComposition = "ui composition", NAMESPACED;
        /// A data-driven UI form style (`ddui/styling/`): `namespace:name`, no default
        /// namespace, a space of its own.
        UiStyle = "ui style", NAMESPACED;
        /// An armour trim pattern (`ward`): a bare name, compared as written.
        TrimPattern = "trim pattern", Syntax::Free;
        /// An armour trim material (`quartz`): a bare name, compared as written.
        TrimMaterial = "trim material", Syntax::Free;
        /// A level sound event of the game (`LevelSoundEvent`: `ambient`, `step`), by its
        /// lower-case name: what `sounds.json` binds sounds to and what a sound member of an
        /// entity or an item names.
        LevelSoundEvent = "level sound event", Syntax::Free;
        /// An item tag (`minecraft:is_food`): declared by the items that carry it, compared
        /// as written.
        ItemTag = "item tag", Syntax::Free;
    }
}

/// A block identifier.
pub type BlockId = Id<kind::Block>;
/// An item identifier.
pub type ItemId = Id<kind::Item>;
/// An entity identifier.
pub type EntityId = Id<kind::Entity>;
/// The key of an entity's spawn rules (castable from an [`EntityId`]).
pub type SpawnRulesId = Id<kind::SpawnRules>;
/// A biome identifier.
pub type BiomeId = Id<kind::Biome>;
/// A geometry identifier.
pub type GeometryId = Id<kind::Geometry>;
/// A texture path.
pub type TexturePath = Id<kind::Texture>;
/// A material identifier.
pub type MaterialId = Id<kind::Material>;
/// A loot table path.
pub type LootTablePath = Id<kind::LootTable>;
/// A structure identifier.
pub type StructureId = Id<kind::Structure>;
/// An animation or animation controller reference.
pub type AnimationRef = Id<kind::Animation>;
/// A feature identifier.
pub type FeatureId = Id<kind::Feature>;
/// A feature rule identifier.
pub type FeatureRuleId = Id<kind::FeatureRule>;
/// A jigsaw template pool identifier.
pub type TemplatePoolId = Id<kind::TemplatePool>;
/// A jigsaw processor list identifier.
pub type ProcessorListId = Id<kind::ProcessorList>;
/// A jigsaw structure identifier.
pub type JigsawStructureId = Id<kind::JigsawStructure>;
/// A jigsaw structure set identifier.
pub type StructureSetId = Id<kind::StructureSet>;
/// A dimension identifier.
pub type DimensionId = Id<kind::Dimension>;
/// A trade table path.
pub type TradeTablePath = Id<kind::TradeTable>;
/// A camera preset identifier.
pub type CameraPresetId = Id<kind::CameraPreset>;
/// A camera list identifier.
pub type CameraListId = Id<kind::CameraList>;
/// An aim-assist preset identifier.
pub type AimAssistPresetId = Id<kind::AimAssistPreset>;
/// An aim-assist category identifier.
pub type AimAssistCategoryId = Id<kind::AimAssistCategory>;
/// A particle effect identifier.
pub type ParticleId = Id<kind::Particle>;
/// A sound event name.
pub type SoundEventName = Id<kind::SoundEvent>;
/// A music definition identifier.
pub type MusicDefinitionId = Id<kind::MusicDefinition>;
/// A fog settings identifier.
pub type FogId = Id<kind::Fog>;
/// A lighting settings identifier (a render config).
pub type LightingSettingsId = Id<kind::LightingSettings>;
/// A water settings identifier (a render config).
pub type WaterSettingsId = Id<kind::WaterSettings>;
/// An atmosphere settings identifier (a render config).
pub type AtmosphereSettingsId = Id<kind::AtmosphereSettings>;
/// A colour-grading settings identifier (a render config).
pub type ColorGradingSettingsId = Id<kind::ColorGradingSettings>;
/// A cubemap settings identifier (a render config).
pub type CubemapSettingsId = Id<kind::CubemapSettings>;
/// A render controller identifier.
pub type RenderControllerId = Id<kind::RenderController>;
/// A texture atlas key.
pub type AtlasKey = Id<kind::AtlasTexture>;
/// A dialogue scene tag.
pub type DialogueSceneTag = Id<kind::DialogueScene>;
/// A function path.
pub type FunctionPath = Id<kind::Function>;
/// A translation key.
pub type LangKey = Id<kind::Translation>;
/// A native script module name.
pub type ScriptModuleName = Id<kind::ScriptModule>;
/// A data-driven UI extension point name.
pub type ExtensionPointName = Id<kind::ExtensionPoint>;
/// A voxel shape identifier.
pub type VoxelShapeId = Id<kind::VoxelShape>;
/// A recipe identifier.
pub type RecipeId = Id<kind::Recipe>;
/// A behavior tree identifier.
pub type BehaviorTreeId = Id<kind::BehaviorTree>;
/// A point-of-interest type identifier.
pub type PoiTypeId = Id<kind::PoiType>;
/// A point-of-interest tag identifier.
pub type PoiTagId = Id<kind::PoiTag>;
/// A block culling rules identifier.
pub type CullingRulesId = Id<kind::CullingRules>;
/// An attachable identifier.
pub type AttachableId = Id<kind::Attachable>;
/// An atomic client entity identifier.
pub type AtomicClientEntityId = Id<kind::AtomicClientEntity>;
/// A raid spawn group identifier.
pub type SpawnGroupId = Id<kind::SpawnGroup>;
/// A data-driven UI root identifier.
pub type UiRootId = Id<kind::UiRoot>;
/// A data-driven UI composition identifier.
pub type UiCompositionId = Id<kind::UiComposition>;
/// A data-driven UI form style identifier.
pub type UiStyleId = Id<kind::UiStyle>;
/// An armour trim pattern identifier (`ward`).
pub type TrimPatternId = Id<kind::TrimPattern>;
/// An armour trim material identifier (`quartz`).
pub type TrimMaterialId = Id<kind::TrimMaterial>;
/// The name of a level sound event of the game.
pub type LevelSoundEventName = Id<kind::LevelSoundEvent>;
/// An item tag.
pub type ItemTagId = Id<kind::ItemTag>;
