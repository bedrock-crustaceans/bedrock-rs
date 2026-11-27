//! The lookups of the link pass: one method of [`LinkCx`] per kind of target a reference can
//! name. Most answer with a [`Target`] and compare in the **target's** canonical form only
//! (the identifier kind's own: a namespace default, a dropped `<event>` or `:aux` suffix, the
//! case folding of geometry and render-controller names). What a source member does to its text
//! before the lookup (lower-casing, adding a namespace, cutting a prefix) is the rule's to do.
//!
//! # What answers for the game's own content
//!
//! A name no pack of the stack defines can still be the game's. Who knows:
//!
//! 1. `LoadOptions::built_ins`, when the host gave one, is asked in place of the lists of
//!    point 2 and is believed, yes or no ([`LinkCx::built_in`]): `contains(kind, id)`. For a
//!    lookup of the `lookups!` table `kind` is the identifier kind's `Kind::NAME` and `id` the
//!    name in the kind's canonical form ([`canonical`]: `minecraft:` added to a bare name of
//!    a kind that defaults to it, an item without `:aux`, an entity without `<event>`, a
//!    geometry or render-controller name lower-cased). The special lookups and the rules
//!    that ask the host themselves pass the name in their own form (a loot table its
//!    lower-cased path, [`LinkCx::loot_table`]) and, where there is no identifier kind, a
//!    name of their own for the kind (`crafting tag` in `content/recipe_block.rs`). The
//!    stacks of a `StackSet` share their options; the behaviour stack's are read, the
//!    resource stack's when it has none.
//! 2. Without a host, the game's tables answer in its place
//!    ([`GeneratedBuiltIns`], over the vanilla data crate) for the lookups marked `[game]`:
//!    `block`, `item`, `entity`, `biome`, `dimension`, `camera preset`, `jigsaw structure`
//!    (the structure features of the game) and `feature` (the features the game registers
//!    in code, [`names::CODE_FEATURES`](crate::vanilla::names::CODE_FEATURES)). The
//!    aim-assist categories and preset of the game's behaviour pack (`world/aim_assist.rs`)
//!    and the level sound events ([`LinkCx::level_sound_event`]) answer from the same
//!    provider, in a host's place.
//! 3. Some names are the game's whatever a host says. The names a [`Names`] holds as legacy
//!    are tested before the host is asked ([`LinkCx::built_in`]): the block names of older
//!    versions and the entities no file defines; the item names of older versions are
//!    accepted after it ([`LinkCx::item`]), and the item tags of the game's recipes beside
//!    it ([`LinkCx::item_tag`]). Rules hold more such names: the crafting tags of the game's
//!    stations, its particle types, the two voxel shapes and the three block geometries it
//!    has in code, the default cubemap, and three lists of names the game's own files
//!    write that name nothing ([`names`](crate::vanilla::names)). The nine behavior trees of the
//!    game are documents of the stack's registry.
//!
//! # Which kinds need a host
//!
//! **The game's own packs** (every behaviour and resource pack of an install, with its app
//! data `definitions/`) link without a host for `block`, `item`, `entity`, `biome`,
//! `dimension`, `camera preset`, `jigsaw structure` and `feature`: the lists above and the
//! packs' own definitions answer. What is still told there without a host, and which kind a
//! host answers to quiet it:
//!
//! - `item tag`: the tags of the game's items beyond the sixteen its recipes use (the
//!   sulfur cube's `shareables` name twelve), told as informations;
//! - `entity`: the actors the game defines in code that have a client entity and no
//!   behaviour file (`minecraft:bed`, `minecraft:skull`, `minecraft:leash_knot`,
//!   `minecraft:decorated_pot`, `minecraft:trial_spawner`), informations;
//! - on a dedicated server, which ships no client resource data: `atlas key`, `texture`,
//!   `geometry`, `animation` and `render controller`;
//! - the references of the game's own files that name nothing (`minecraft:oak_button` in
//!   the aim-assist categories, a culling rule, a particle event, three loot tables): a
//!   host can answer for the kind, and nothing else will.
//!
//! **An add-on linked without the game's packs** needs a host for the names of the game of
//! the kinds a rule asks about and no list above answers for. `material` and `atlas key`
//! are judged by the view the pass has of them ([`View`]: [`LinkCx::material_view`],
//! [`LinkCx::terrain_atlas_view`], [`LinkCx::item_atlas_view`]). No rule of the pass calls
//! the lookups of `dimension`, `camera preset`, `structure set` and `function`, nor those
//! of commands and native script modules (the `music definition` one is the client biome's
//! music rule's).
// A host that answers for a kind is asked first and believed, yes or no; the crate's lists
// answer only where there is no host.

#![allow(
    dead_code,
    reason = "the frame of the rule packages: each calls the lookups its rows need, and which \
              exist depends on the enabled features"
)]

use std::borrow::Cow;
use std::cell::OnceCell;
use std::collections::HashMap;
use std::marker::PhantomData;

use bedrock_addon_core::diagnostic::ReferenceHint;
use bedrock_addon_core::document::{Definition, Origin, Registry};
#[allow(
    unused_imports,
    reason = "`kind` is unused when no content module is enabled"
)]
use bedrock_addon_core::id::{Id, Kind, Syntax, kind};
use bedrock_addon_core::load::BuiltIns;
#[cfg(feature = "material")]
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::pack_identity::{PackRef, PackSide};
use bedrock_addon_core::vanilla::{GeneratedBuiltIns, names};
use indexmap::IndexMap;
use uuid::Uuid;

#[cfg(any(feature = "material", feature = "texture_atlas"))]
use super::Findings;
use super::LinkCx;
use crate::families::with_families;

/// The names of the target kinds, as a finding's `DiagnosticDetail::Reference::kind` has them:
/// `Kind::NAME` of each identifier kind a lookup resolves, and the names of the targets that
/// have no identifier kind.
pub(crate) mod kinds {
    use bedrock_addon_core::id::{Kind, kind};

    macro_rules! names {
        ($($name:ident = $kind:ident;)*) => {
            $(
                #[doc = concat!("`", stringify!($kind), "`.")]
                pub(crate) const $name: &str = <kind::$kind as Kind>::NAME;
            )*
        };
    }

    names! {
        BLOCK = Block;
        ITEM = Item;
        ENTITY = Entity;
        SPAWN_RULES = SpawnRules;
        BIOME = Biome;
        GEOMETRY = Geometry;
        TEXTURE = Texture;
        MATERIAL = Material;
        LOOT_TABLE = LootTable;
        STRUCTURE = Structure;
        ANIMATION = Animation;
        FEATURE = Feature;
        TEMPLATE_POOL = TemplatePool;
        PROCESSOR_LIST = ProcessorList;
        JIGSAW_STRUCTURE = JigsawStructure;
        STRUCTURE_SET = StructureSet;
        DIMENSION = Dimension;
        TRADE_TABLE = TradeTable;
        CAMERA_PRESET = CameraPreset;
        AIM_ASSIST_PRESET = AimAssistPreset;
        AIM_ASSIST_CATEGORY = AimAssistCategory;
        PARTICLE = Particle;
        SOUND_EVENT = SoundEvent;
        MUSIC_DEFINITION = MusicDefinition;
        FOG = Fog;
        LIGHTING_SETTINGS = LightingSettings;
        WATER_SETTINGS = WaterSettings;
        ATMOSPHERE_SETTINGS = AtmosphereSettings;
        COLOR_GRADING_SETTINGS = ColorGradingSettings;
        CUBEMAP_SETTINGS = CubemapSettings;
        RENDER_CONTROLLER = RenderController;
        ATLAS_KEY = AtlasTexture;
        FUNCTION = Function;
        LANG_KEY = Translation;
        SCRIPT_MODULE = ScriptModule;
        VOXEL_SHAPE = VoxelShape;
        BEHAVIOR_TREE = BehaviorTree;
        POI_TYPE = PoiType;
        POI_TAG = PoiTag;
        CULLING_RULES = CullingRules;
        LEVEL_SOUND_EVENT = LevelSoundEvent;
        ITEM_TAG = ItemTag;
    }

    /// A state of a block (`description.states` and the states its traits add).
    pub(crate) const BLOCK_STATE: &str = "block state";
    /// A bone of a geometry.
    pub(crate) const BONE: &str = "bone";
    /// A cube of a bone, by its index.
    pub(crate) const CUBE: &str = "cube";
    /// A locator of a geometry.
    pub(crate) const LOCATOR: &str = "locator";
    /// A block sound type: a key of `block_sounds` in `sounds.json`.
    pub(crate) const BLOCK_SOUND: &str = "block sound";
    /// A sound played by name: a key of `individual_named_sounds` in `sounds.json`.
    pub(crate) const NAMED_SOUND: &str = "named sound";
    /// A command, as typed without `/`.
    pub(crate) const COMMAND: &str = "command";
    /// A pack of the stack, by its header UUID.
    pub(crate) const PACK: &str = "pack";
}

/// What a name resolves to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Target<'a> {
    /// A pack of the stack defines it: where.
    Defined(&'a Origin),
    /// No pack defines it and it is the game's own.
    BuiltIn,
    /// No pack defines it, it is not known to be the game's, and a collection that could
    /// define it is compiled out: nobody can tell, and no rule reports it.
    NotInView,
    /// Nothing has it, with a near miss when there is one.
    Missing(Option<ReferenceHint>),
}

impl<'a> Target<'a> {
    /// Nothing has the name: the one answer a rule reports.
    pub(crate) fn is_missing(&self) -> bool {
        matches!(self, Self::Missing(_))
    }

    /// Where a pack defines it.
    pub(crate) fn origin(&self) -> Option<&'a Origin> {
        match self {
            Self::Defined(origin) => Some(origin),
            _ => None,
        }
    }

    /// The near miss of a missing name.
    pub(crate) fn hint(&self) -> Option<&ReferenceHint> {
        match self {
            Self::Missing(hint) => hint.as_ref(),
            _ => None,
        }
    }
}

/// How much of a kind the pass can see, for the kinds whose definitions are a registry of
/// the game that an add-on does not carry (`material`, `atlas key`): what a rule does with a
/// name nothing of the stack defines depends on it.
// A stack with no view of the kind says nothing about a name; a stack with a partial view
// tells a miss as an information.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum View {
    /// The stack holds no source of the kind and there is no host: a name of the game
    /// cannot be told from a missing one, and a rule says nothing of the kind.
    None,
    /// The stack holds a source of the kind, none of it from a pack the game ships, and
    /// there is no host: a name the stack lacks may be the game's. A miss is told as an
    /// information, with the row's code and detail.
    Partial,
    /// A pack the game ships provides a source of the kind that was read into the stack, or
    /// the stacks were built with a host (any host; it is then asked for the name): a miss
    /// is told with the row's severity. (A flipbook's tile is compared only when the stack
    /// has a terrain atlas: with a host and no atlas that rule says nothing.)
    Whole,
}

impl View {
    /// Whether a rule judges the kind at all.
    pub(crate) fn judges(self) -> bool {
        self != Self::None
    }

    /// The severity of a miss in a row whose severity is `row`: the row's with the whole
    /// view, an information with a partial one.
    pub(crate) fn severity(
        self,
        row: bedrock_addon_core::diagnostic::Severity,
    ) -> bedrock_addon_core::diagnostic::Severity {
        match self {
            Self::Whole => row,
            Self::None | Self::Partial => row.min(bedrock_addon_core::diagnostic::Severity::Info),
        }
    }
}

/// `id` in the canonical form of its kind, as a host's `BuiltIns` and the vanilla lists have
/// it: `minecraft:` in front of a bare name of a kind that defaults to it, no `<event>` or
/// `:aux` suffix, a case-folding kind lower-cased; any other name as written.
pub(crate) fn canonical<K: Kind>(id: &Id<K>) -> Cow<'_, str> {
    let text = id.as_str();
    match K::SYNTAX {
        Syntax::Namespaced { default_vanilla } => {
            let path = id.path();
            match id.namespace() {
                Some(namespace) if namespace.len() + 1 + path.len() == text.len() => {
                    Cow::Borrowed(text)
                }
                Some(namespace) => Cow::Owned(format!("{namespace}:{path}")),
                None if default_vanilla => Cow::Owned(format!("minecraft:{path}")),
                None => Cow::Borrowed(path),
            }
        }
        Syntax::Prefixed {
            fold_case: true, ..
        } if text.bytes().any(|byte| byte.is_ascii_uppercase()) => {
            Cow::Owned(text.to_ascii_lowercase())
        }
        _ => Cow::Borrowed(text),
    }
}

/// The definitions of one kind on the stack, by identifier, in load order: what a lookup of
/// that kind compares against. Built once per pass, on the first lookup of the kind.
pub(crate) struct Names<'a, K: Kind> {
    defined: IndexMap<Id<K>, &'a Origin>,
    /// Whether the game's tables (`GeneratedBuiltIns`) answer for the kind without a host.
    vanilla: bool,
    /// The names of the kind the game answers to beyond its lists
    /// ([`names`](crate::vanilla::names)): they stand with any host.
    legacy: Option<fn(&str) -> bool>,
    /// What a missing name is compared with for its near miss; built on the first miss.
    near: OnceCell<Near>,
}

/// The defined names by what a near miss shares with them; each value is the index of the
/// first such definition.
#[derive(Debug)]
struct Near {
    /// By the canonical text, ASCII lower-cased.
    folded: HashMap<String, usize>,
    /// By the text after the namespace, for a kind that has namespaces.
    paths: HashMap<String, usize>,
}

impl<K: Kind> std::fmt::Debug for Names<'_, K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Names")
            .field("defined", &self.defined.len())
            .field("vanilla", &self.vanilla)
            .field("legacy", &self.legacy.is_some())
            .field("near", &self.near)
            .finish_non_exhaustive()
    }
}

impl<'a, K: Kind> Names<'a, K> {
    /// The names `defined` gives, the first definition of a name kept.
    pub(crate) fn new(defined: impl IntoIterator<Item = (Id<K>, &'a Origin)>) -> Self {
        let mut names = IndexMap::new();
        for (id, origin) in defined {
            names.entry(id).or_insert(origin);
        }
        Self {
            defined: names,
            vanilla: false,
            legacy: None,
            near: OnceCell::new(),
        }
    }

    /// The same names, with the game's tables answering for the kind (`K::NAME`) without a
    /// host.
    pub(crate) fn with_vanilla(mut self) -> Self {
        self.vanilla = true;
        self
    }

    /// The same names, with `legacy` answering for the names of the kind the game has
    /// beyond its lists.
    pub(crate) fn with_legacy(mut self, legacy: fn(&str) -> bool) -> Self {
        self.legacy = Some(legacy);
        self
    }

    /// Whether `name` (canonical form) is one the game answers to beyond its lists.
    pub(crate) fn is_legacy(&self, name: &str) -> bool {
        self.legacy.is_some_and(|legacy| legacy(name))
    }

    /// Where the stack defines `id`.
    pub(crate) fn get(&self, id: &Id<K>) -> Option<&'a Origin> {
        self.defined.get(id).copied()
    }

    /// Every defined name with its place, in load order.
    pub(crate) fn iter(&self) -> impl ExactSizeIterator<Item = (&Id<K>, &'a Origin)> + '_ {
        self.defined.iter().map(|(id, origin)| (id, *origin))
    }

    /// How many names the stack defines.
    pub(crate) fn len(&self) -> usize {
        self.defined.len()
    }

    /// The stack defines no name of the kind.
    pub(crate) fn is_empty(&self) -> bool {
        self.defined.is_empty()
    }

    /// Whether the game's tables have `name` (canonical form) of the kind, when they answer
    /// for it.
    pub(crate) fn is_vanilla(&self, name: &str) -> bool {
        self.vanilla && GeneratedBuiltIns.contains(K::NAME, name)
    }

    /// The near miss of a name the stack does not define: a defined name that differs only in
    /// ASCII case, else, for a bare name of a kind with namespaces, that a defined name has
    /// it after its namespace.
    pub(crate) fn near_miss(&self, id: &Id<K>) -> Option<ReferenceHint> {
        let near = self.near.get_or_init(|| {
            let mut near = Near {
                folded: HashMap::new(),
                paths: HashMap::new(),
            };
            let namespaced = matches!(K::SYNTAX, Syntax::Namespaced { .. });
            for (index, name) in self.defined.keys().enumerate() {
                near.folded
                    .entry(canonical(name).to_ascii_lowercase())
                    .or_insert(index);
                if namespaced && name.namespace().is_some() {
                    near.paths.entry(name.path().to_owned()).or_insert(index);
                }
            }
            near
        });
        let name_of = |index: &usize| self.defined.get_index(*index).map(|(name, _)| name);
        if let Some(name) = near
            .folded
            .get(&canonical(id).to_ascii_lowercase())
            .and_then(name_of)
        {
            return Some(ReferenceHint::CaseMismatch(name.as_str().to_owned()));
        }
        let bare = matches!(K::SYNTAX, Syntax::Namespaced { .. }) && id.namespace().is_none();
        (bare && near.paths.contains_key(id.path())).then_some(ReferenceHint::MissingNamespace)
    }
}

/// Every effective definition of `registry`, stable then beta, with its place.
pub(crate) fn registered<T: Definition>(
    registry: &Registry<T>,
) -> impl Iterator<Item = (Id<T::Kind>, &Origin)> + '_ {
    let stable = registry
        .iter()
        .map(|(id, document)| (id.clone(), document.origin()));
    let beta = registry.beta().into_iter().flat_map(|beta| {
        beta.iter()
            .map(|(id, document)| (id.clone(), document.origin()))
    });
    stable.chain(beta)
}

/// The lookups that are one collection keyed by its identifier kind: a cache field per line
/// and two methods, `<name>(&Id<K>) -> Target` and `<name>_names() -> &Names<K>`.
macro_rules! lookups {
    ($(
        $(#[doc = $doc:literal])+
        #[cfg($cfg:meta)]
        $name:ident, $names:ident: $kind:ident $([$game:ident])? $(+ $legacy:ident)? = |$set:ident| $build:expr;
    )*) => {
        /// The names of each kind, each built on its first lookup.
        #[derive(Debug, Default)]
        pub(super) struct Caches<'a> {
            $(
                #[cfg($cfg)]
                $name: OnceCell<Names<'a, kind::$kind>>,
            )*
            pub(super) special: Special<'a>,
        }

        /// The name of every lookup of the table, compiled in or not, in table order.
        #[cfg(test)]
        const LOOKUPS: &[&str] = &[$(stringify!($name)),*];

        impl<'a> LinkCx<'a> {
            $(
                $(#[doc = $doc])+
                #[cfg($cfg)]
                pub(crate) fn $name(&self, id: &Id<kind::$kind>) -> Target<'a> {
                    self.resolve(self.$names(), id)
                }

                /// The names behind the lookup of the same name.
                #[cfg($cfg)]
                pub(crate) fn $names(&self) -> &Names<'a, kind::$kind> {
                    self.caches.$name.get_or_init(|| {
                        let $set = self.set();
                        let names = Names::new($build);
                        $(let names = lookups!(@game names, $game);)?
                        names $(.with_legacy(names::$legacy))?
                    })
                }
            )*
        }
    };
    // `[game]`: the game's tables answer for the kind without a host.
    (@game $names:ident, game) => { $names.with_vanilla() };
}

/// The `lookups { … }` blocks of the family list, each entry behind its family's feature, as one
/// `lookups!` table.
macro_rules! family_lookups {
    ($($family:literal $module:ident = $krate:ident $(:: $path:ident)* { $($accessors:tt)* }
        $(lookups {
            $(
                $(#[doc = $doc:literal])+
                $name:ident, $names:ident: $kind:ident $([$game:ident])? $(+ $legacy:ident)?
                    = |$set:ident| $build:expr;
            )*
        })?
    )*) => {
        lookups! {
            $($($(
                $(#[doc = $doc])+
                #[cfg(feature = $family)]
                $name, $names: $kind $([$game])? $(+ $legacy)? = |$set| $build;
            )*)?)*
        }
    };
}

// The lookup table: the `lookups` blocks of the family list. A new identifier kind adds its
// entry to its family's group in `families/list.rs`.
with_families!(family_lookups);

/// The caches of the lookups that are not one collection by identifier.
#[derive(Debug, Default)]
pub(super) struct Special<'a> {
    #[cfg(feature = "loot")]
    loot_tables: OnceCell<HashMap<String, &'a Origin>>,
    #[cfg(feature = "item")]
    item_tags: OnceCell<HashMap<&'a str, &'a Origin>>,
    #[cfg(feature = "geometry")]
    geometry_parts: OnceCell<HashMap<Id<kind::Geometry>, GeometryParts<'a>>>,
    #[cfg(feature = "material")]
    resolved_materials:
        [OnceCell<IndexMap<String, crate::families::material::ResolvedMaterial>>; 3],
    #[cfg(all(feature = "lang", feature = "languages"))]
    unlisted_languages: OnceCell<UnlistedLanguages>,
    /// The views of the materials, the terrain atlas and the item atlas, each found on its
    /// first use in the pass.
    views: [OnceCell<View>; 3],
    lifetime: PhantomData<&'a ()>,
}

/// The keys of the `.lang` files of the game's languages that no `texts/languages.json` of
/// their stack lists (so that `translations` does not hold them), read from the packs.
#[cfg(all(feature = "lang", feature = "languages"))]
#[derive(Debug, Default)]
struct UnlistedLanguages {
    /// Each key, lower-cased as the game stores it, with the first file that defines it:
    /// the resource stack's files before the behaviour stack's, the game's languages in
    /// their order.
    keys: HashMap<String, Origin>,
    /// Such a file of the base language (`en_US`) is listed by its pack and not resident:
    /// the keys every language falls back to are not known.
    unread: bool,
}

/// The own bones and locators of one geometry, by lower-cased name.
#[cfg(feature = "geometry")]
#[derive(Debug, Default)]
struct GeometryParts<'a> {
    bones: HashMap<String, &'a crate::families::geometry::Bone>,
    locators: std::collections::HashSet<String>,
}

impl<'a> LinkCx<'a> {
    /// The answer for `id` among `names`: defined, else the game's own, else missing with the
    /// near miss.
    pub(crate) fn resolve<'n, K: Kind>(&self, names: &Names<'n, K>, id: &Id<K>) -> Target<'n> {
        if let Some(origin) = names.get(id) {
            return Target::Defined(origin);
        }
        if self.built_in(names, id) {
            return Target::BuiltIn;
        }
        Target::Missing(names.near_miss(id))
    }

    /// Whether `id` is the game's own: a name the game answers to beyond its lists (with any
    /// host), else what the host says, or without a host the crate's list of the kind.
    pub(crate) fn built_in<K: Kind>(&self, names: &Names<'_, K>, id: &Id<K>) -> bool {
        let name = canonical(id);
        if names.is_legacy(&name) {
            return true;
        }
        match self.host_built_ins() {
            Some(host) => host.contains(K::NAME, &name),
            None => names.is_vanilla(&name),
        }
    }

    /// Whether the host says the game has `name` of `kind`; `false` without a host.
    pub(crate) fn host_has(&self, kind: &'static str, name: &str) -> bool {
        self.host_built_ins()
            .is_some_and(|host| host.contains(kind, name))
    }

    /// An item: one a behaviour pack defines, the item of a block of the stack (every block
    /// has an item of its name), one of the game's, or a legacy name the game's own files
    /// still use ([`LEGACY_ITEM_NAMES`](crate::vanilla::names::LEGACY_ITEM_NAMES): compared without
    /// ASCII case, with any host). An `:aux` suffix is not part of the name.
    ///
    /// Without the `block` feature a pack's blocks cannot be known: a name outside the
    /// `minecraft` namespace that is no item is [`Target::NotInView`].
    // The item of a block of the stack is an item of the block's name; the legacy names are the
    // ones the game's own files still use.
    #[cfg(feature = "item")]
    pub(crate) fn item(&self, id: &Id<kind::Item>) -> Target<'a> {
        let names = self.item_names();
        if let Some(origin) = names.get(id) {
            return Target::Defined(origin);
        }
        let name = canonical(id);
        #[cfg(feature = "block")]
        if let Some(origin) = self.block_names().get(&Id::parse(name.as_ref())) {
            return Target::Defined(origin);
        }
        // spawnable actor (`is_spawnable`) a pack defines.
        #[cfg(feature = "entity")]
        if let Some(entity) = name.strip_suffix("_spawn_egg")
            && let Some(document) = self.set().behavior.entities().get(&Id::parse(entity))
            && document.value().is_spawnable()
        {
            return Target::Defined(document.origin());
        }
        if self.built_in(names, id) || names::is_legacy_item_name(&name) {
            return Target::BuiltIn;
        }
        #[cfg(not(feature = "block"))]
        if !id.is_vanilla() {
            return Target::NotInView;
        }
        Target::Missing(names.near_miss(id))
    }

    /// An item tag: a tag an item of the behaviour stack carries (`minecraft:tags`), one the
    /// game's own recipes take an ingredient by
    /// ([`VANILLA_ITEM_TAGS`](crate::vanilla::names::VANILLA_ITEM_TAGS), with any host), or one the
    /// host says the game's items carry. Compared as written.
    #[cfg(feature = "item")]
    pub(crate) fn item_tag(&self, tag: &str) -> Target<'a> {
        let tags = self.caches.special.item_tags.get_or_init(|| {
            let mut tags = HashMap::new();
            for (_, document) in self.set().behavior.items() {
                let carried = document
                    .value()
                    .components
                    .get::<crate::families::item::component::Tags>();
                for tag in carried.iter().flat_map(|tags| tags.tags.iter()) {
                    tags.entry(tag.as_str()).or_insert(document.origin());
                }
            }
            tags
        });
        match tags.get(tag) {
            Some(origin) => Target::Defined(origin),
            None if names::is_vanilla_item_tag(tag) || self.host_has(kinds::ITEM_TAG, tag) => {
                Target::BuiltIn
            }
            None => Target::Missing(None),
        }
    }

    /// The states of a block of the stack with the values each can take: its own
    /// `description.states` and the states its traits add. `None` for a block no pack defines
    /// (the states of the game's own blocks are not known here, and a rule checks none).
    #[cfg(feature = "block")]
    pub(crate) fn block_states(
        &self,
        id: &Id<kind::Block>,
    ) -> Option<IndexMap<String, crate::families::block::StateValues>> {
        let block = self.set().behavior.blocks().get(id)?;
        Some(block.value().states_with_traits())
    }

    /// A loot table, by a reference as written: the text up to its last `:` is dropped, the
    /// rest is compared without case with the tables' paths, and a reference without `.json`
    /// names the file with it (the game's own tables write `loot_tables/gameplay/fishing/fish`).
    // The reference is stripped to the text after its last `:` and lower-cased, and one without
    // `.json` names the file with it.
    #[cfg(feature = "loot")]
    pub(crate) fn loot_table(&self, reference: &str) -> Target<'a> {
        let tables = self.caches.special.loot_tables.get_or_init(|| {
            let mut tables = HashMap::new();
            for (path, document) in self.set().behavior.loot_tables() {
                tables
                    .entry(path.as_str().to_lowercase())
                    .or_insert(document.origin());
            }
            tables
        });
        let path = reference
            .rsplit_once(':')
            .map_or(reference, |(_, path)| path)
            .to_lowercase();
        let found = tables
            .get(&path)
            .or_else(|| tables.get(&format!("{path}.json")));
        match found {
            Some(origin) => Target::Defined(origin),
            None if self.host_has(kinds::LOOT_TABLE, &path) => Target::BuiltIn,
            None => Target::Missing(None),
        }
    }

    /// The structure template a jigsaw pool element names: `structures/<name>.mcstructure`,
    /// `<name>` being the location without its namespace.
    #[cfg(feature = "structure")]
    pub(crate) fn structure_template(&self, location: &Id<kind::Structure>) -> Target<'a> {
        let path = format!("structures/{}.mcstructure", location.path());
        let found = crate::storage::PackPath::new(&path)
            .ok()
            .and_then(|path| self.set().behavior.structures().get(&path));
        match found {
            Some(document) => Target::Defined(document.origin()),
            None if self.host_has(kinds::STRUCTURE, &canonical(location)) => Target::BuiltIn,
            None => Target::Missing(None),
        }
    }

    /// An animation or animation controller of `side`, the prefix of the name picking which:
    /// `controller.animation.` a controller, `animation.` an animation. `None` for a name
    /// with neither prefix, which the game ignores.
    // The prefix of the name picks the kind; other prefixes are ignored.
    #[cfg(all(feature = "animation", feature = "animation_controller"))]
    pub(crate) fn animation(
        &self,
        side: crate::pack::PackSide,
        id: &Id<kind::Animation>,
    ) -> Option<Target<'a>> {
        use crate::pack::PackSide::{Behavior, Resource};
        let name = id.as_str();
        let names = if name.starts_with("controller.animation.") {
            match side {
                Behavior => self.behavior_animation_controller_names(),
                Resource => self.client_animation_controller_names(),
            }
        } else if name.starts_with("animation.") {
            match side {
                Behavior => self.behavior_animation_names(),
                Resource => self.client_animation_names(),
            }
        } else {
            return None;
        };
        Some(self.resolve(names, id))
    }

    /// A bone of the geometry `id`, by name without case: one of its own, else one it
    /// inherits (a geometry older than 1.12 with a parent), nearest parent first. `None` when
    /// the stack has no such geometry or it has no such bone. The bone has its cubes and its
    /// locators.
    ///
    /// An own bone costs one lookup; an inherited one the bones of the parents up to it.
    #[cfg(feature = "geometry")]
    pub(crate) fn geometry_bone(
        &self,
        id: &Id<kind::Geometry>,
        bone: &str,
    ) -> Option<&'a crate::families::geometry::Bone> {
        let name = bone.to_ascii_lowercase();
        if let Some(found) = self.geometry_parts().get(id)?.bones.get(&name) {
            return Some(found);
        }
        let geometry = self.set().resource.geometries().get(id)?.value();
        let mut parent = geometry.inherited.as_deref();
        while let Some(link) = parent {
            let found = link
                .bones()
                .iter()
                .find(|bone| bone.name.eq_ignore_ascii_case(&name));
            if found.is_some() {
                return found;
            }
            parent = link.parent();
        }
        None
    }

    /// Whether the geometry `id` has the locator `name` (compared without case) on a bone of
    /// its own or on one it inherits.
    #[cfg(feature = "geometry")]
    pub(crate) fn geometry_has_locator(&self, id: &Id<kind::Geometry>, name: &str) -> bool {
        let name = name.to_ascii_lowercase();
        let Some(parts) = self.geometry_parts().get(id) else {
            return false;
        };
        if parts.locators.contains(&name) {
            return true;
        }
        let Some(geometry) = self.set().resource.geometries().get(id) else {
            return false;
        };
        let mut parent = geometry.value().inherited.as_deref();
        while let Some(link) = parent {
            let has = link.bones().iter().any(|bone| {
                bone.locators
                    .iter()
                    .flat_map(|locators| locators.keys())
                    .any(|locator| locator.eq_ignore_ascii_case(&name))
            });
            if has {
                return true;
            }
            parent = link.parent();
        }
        false
    }

    #[cfg(feature = "geometry")]
    fn geometry_parts(&self) -> &HashMap<Id<kind::Geometry>, GeometryParts<'a>> {
        self.caches.special.geometry_parts.get_or_init(|| {
            let mut all = HashMap::new();
            for (id, document) in self.set().resource.geometries() {
                let mut parts = GeometryParts::default();
                for bone in document.value().bones() {
                    parts
                        .bones
                        .entry(bone.name.to_ascii_lowercase())
                        .or_insert(bone);
                    for locator in bone.locators.iter().flat_map(|locators| locators.keys()) {
                        parts.locators.insert(locator.to_ascii_lowercase());
                    }
                }
                all.insert(id.clone(), parts);
            }
            all
        })
    }

    /// A material: any of the three groups defines the name, as a material or as a variant
    /// `<name>.<variant>`; compared as written. The place is the file that defines the
    /// material (its key's, in the merged file).
    ///
    /// A miss costs the length of the name. A hit also looks for the material's key among the
    /// keys of the file that defines it, so it costs the materials of that file: a rule that
    /// resolves many references to materials of one large file is not linear by this lookup
    /// alone and has its own cost test (or reads the place once per material).
    #[cfg(feature = "material")]
    pub(crate) fn material(&self, id: &Id<kind::Material>) -> Target<'a> {
        let materials = self.set().resource.materials();
        if !materials.contains(id) {
            return if self.host_has(kinds::MATERIAL, id.as_str()) {
                Target::BuiltIn
            } else {
                Target::Missing(None)
            };
        }
        // The material itself, or the one a variant name belongs to: the longest name before
        // a dot that is a material.
        let name = id.as_str();
        let owner = std::iter::once(name)
            .chain(name.rmatch_indices('.').map(|(at, _)| &name[..at]))
            .find_map(|name| materials.get(name));
        let origin = owner.and_then(|material| {
            let file = materials.files.get(&material.file)?;
            let written = file
                .value()
                .materials
                .keys()
                .find(|key| key.name == material.name && !key.extend)?;
            file.origin(written)
        });
        // A material without a place it can name (its key is another file's): the game's.
        origin.map_or(Target::BuiltIn, Target::Defined)
    }

    /// Whether a material named `id` exists: any of the three groups defines the name, as a
    /// material or as a variant `<name>.<variant>` (compared as written), or the host says
    /// the game has it. The answer of [`material`](Self::material) without the place.
    ///
    /// Costs the length of the name, hit or miss: the lookup for a rule that resolves many
    /// references and has no use for where the material is defined.
    // For a rule that resolves many references and does not need where the material is defined.
    #[cfg(feature = "material")]
    pub(crate) fn material_exists(&self, id: &Id<kind::Material>) -> bool {
        self.set().resource.materials().contains(id) || self.host_has(kinds::MATERIAL, id.as_str())
    }

    /// The view the pass has of the materials. A resource pack loaded without the game's
    /// `materials/` creates none (a `.material` file of its own creates materials only
    /// through a list file), and then a material of the game cannot be told from a missing
    /// one.
    ///
    /// - [`View::Whole`]: a host answers, or a pack the game ships created a material of the
    ///   stack (its list and its material file were read);
    /// - [`View::Partial`]: materials are created, none of them by a pack the game ships;
    /// - [`View::None`]: no pack of the resource stack creates a material and there is no
    ///   host.
    ///
    /// A file below `materials/` of a pack the game ships that creates nothing is no view of
    /// the game's materials. A list file of such a pack that the stack did not read because
    /// its bytes are not resident, and a `.material` file beside a list file likewise, is
    /// told to `out` once per pass (`FileNotLoaded`), so that the caller knows why nothing,
    /// or less, was judged. The view is found once per pass.
    // The view is found from what the stack read: a source that was read counts, and the check
    // runs whenever the family produced an index for the stack.
    #[cfg(feature = "material")]
    pub(crate) fn material_view(&self, out: &mut Findings) -> View {
        use crate::families::material::MaterialGroupKind;
        *self.caches.special.views[0].get_or_init(|| {
            if self.host_built_ins().is_some() {
                return View::Whole;
            }
            let materials = self.set().resource.materials();
            let groups = || {
                MaterialGroupKind::ALL
                    .into_iter()
                    .map(|kind| materials.group(kind))
            };
            // The file of a material is merged over the packs that have it: the game
            // created the material when one of them is a pack the game ships.
            let read_from = |file: &PackPath, pack: &PackRef| {
                materials.files.get(file).is_some_and(|merged| {
                    merged
                        .layers()
                        .iter()
                        .any(|layer| layer.origin.pack == *pack)
                })
            };
            let game_file = |file: &PackPath| {
                materials.files.get(file).is_some_and(|merged| {
                    merged
                        .layers()
                        .iter()
                        .any(|layer| self.is_game_pack(&layer.origin.pack))
                })
            };
            let mut files = std::collections::HashSet::new();
            let by_game = groups().any(|group| {
                group
                    .iter()
                    .any(|material| files.insert(&material.file) && game_file(&material.file))
            });
            if by_game {
                return View::Whole;
            }
            // What a pack the game ships has below `materials/` and the stack did not read:
            // its list files, and beside a list file its `.material` files (a `.material`
            // file creates materials only through a list). Asking for the bytes tells one
            // that is not resident, once per pass.
            for pack in self.set().resource.stack_packs() {
                if !pack.context.is_base_game_pack {
                    continue;
                }
                let listed = pack.files_below("materials");
                let is_list = |path: &str| {
                    path.rsplit_once('.')
                        .is_some_and(|(_, extension)| extension == "json")
                };
                if !listed.iter().any(|file| is_list(file.key.as_str())) {
                    continue;
                }
                for file in listed {
                    let read = if is_list(file.key.as_str()) {
                        materials.lists.iter().any(|list| {
                            list.origin().pack == pack.pack && list.origin().path == file.path
                        })
                    } else {
                        read_from(&file.key, &pack.pack)
                    };
                    if !read {
                        let _ = self.bytes(PackSide::Resource, &pack.pack, &file.path, out);
                    }
                }
            }
            if groups().any(|group| !group.is_empty()) {
                View::Partial
            } else {
                View::None
            }
        })
    }

    /// The view the pass has of the terrain atlas (`textures/terrain_texture.json`): whole
    /// when a host answers, or the stack builds the atlas and the copy of a pack the game
    /// ships was merged into it and names its `texture_name` (the game's own atlas builds:
    /// its tiles were read); partial when a pack that is not the game's has the file; none
    /// otherwise. A copy of the game's pack that the stack did not read because its bytes
    /// are not resident is told to `out` once per pass (`FileNotLoaded`).
    #[cfg(feature = "texture_atlas")]
    pub(crate) fn terrain_atlas_view(&self, out: &mut Findings) -> View {
        *self.caches.special.views[1].get_or_init(|| {
            let textures = self.set().resource.texture_atlases();
            let built = textures.terrain().is_some();
            let file = "textures/terrain_texture.json";
            self.atlas_view(file, &textures.terrain_atlas, built, out)
        })
    }

    /// The view the pass has of the item atlas (`textures/item_texture.json`), by the rule
    /// of [`terrain_atlas_view`](Self::terrain_atlas_view).
    #[cfg(feature = "texture_atlas")]
    pub(crate) fn item_atlas_view(&self, out: &mut Findings) -> View {
        *self.caches.special.views[2].get_or_init(|| {
            let textures = self.set().resource.texture_atlases();
            let built = textures.item().is_some();
            let file = "textures/item_texture.json";
            self.atlas_view(file, &textures.item_atlas, built, out)
        })
    }

    /// The view of the atlas `atlas`, whose file is `file` and which the stack builds when
    /// `built`: one rule for both atlases.
    #[cfg(feature = "texture_atlas")]
    fn atlas_view(
        &self,
        file: &str,
        atlas: &bedrock_addon_core::document::Merged<
            crate::families::texture_atlas::TextureAtlas,
            bedrock_addon_core::document::RawLayer,
        >,
        built: bool,
        out: &mut Findings,
    ) -> View {
        if self.host_built_ins().is_some() {
            return View::Whole;
        }
        let (mut own, mut game_read) = (false, false);
        for pack in self.set().resource.stack_packs() {
            let listed = pack.files_below("textures");
            let Ok(at) = listed.binary_search_by(|listed| listed.key.as_str().cmp(file)) else {
                continue;
            };
            if !pack.context.is_base_game_pack {
                own = true;
                continue;
            }
            // The copy of this pack as the stack merged it.
            let copy = atlas
                .layers()
                .iter()
                .find(|layer| layer.origin.pack == pack.pack);
            match copy {
                // The game's tiles were read when its own copy builds an atlas: it names
                // its `texture_name`.
                Some(layer) => {
                    game_read |= layer
                        .json
                        .as_object()
                        .and_then(|object| object.get("texture_name"))
                        .is_some_and(|name| name.as_string().is_some());
                }
                // Not merged: asking for the bytes tells a file that is not resident, once
                // per pass (one that is resident did not load, and its load said so).
                None => {
                    let _ = self.bytes(PackSide::Resource, &pack.pack, &listed[at].path, out);
                }
            }
        }
        if game_read && built {
            View::Whole
        } else if own {
            View::Partial
        } else {
            View::None
        }
    }

    /// Whether `pack` is a pack of the resource stack that the game ships (a base-game
    /// pack).
    fn is_game_pack(&self, pack: &PackRef) -> bool {
        self.stack_pack(PackSide::Resource, pack)
            .is_some_and(|entry| entry.context.is_base_game_pack)
    }

    /// Every material of the group `kind` with everything it inherits applied, by name:
    /// resolved once per pass and group, at the cost of the group. A rule that needs the
    /// members of many materials (a shader, a define) looks them up here, never one
    /// `Materials::resolve` per reference.
    #[cfg(feature = "material")]
    pub(crate) fn resolved_materials(
        &self,
        kind: crate::families::material::MaterialGroupKind,
    ) -> &IndexMap<String, crate::families::material::ResolvedMaterial> {
        use crate::families::material::MaterialGroupKind::{Common, Fancy, Sad};
        let [common, fancy, sad] = &self.caches.special.resolved_materials;
        let group = match kind {
            Common => common,
            Fancy => fancy,
            // `Sad` is "every other graphics mode"; the kind is `#[non_exhaustive]` now that
            // its crate is another.
            Sad | _ => sad,
        };
        group.get_or_init(|| self.set().resource.materials().resolve_all(kind))
    }

    /// What the client loads for a texture path: [`PackStack::resolve_texture`](crate::pack::PackStack::resolve_texture),
    /// the one resolver of every texture reference. `None` is a missing texture
    /// (`TextureNotFound`); a file whose `is_missing()` holds is a texture set that did not
    /// load, which the load has reported: a rule says nothing more of it.
    #[cfg(all(feature = "texture_file", feature = "texture_set"))]
    pub(crate) fn texture_file(
        &self,
        path: &Id<kind::Texture>,
    ) -> Option<crate::pack::TextureFile<'a>> {
        self.set().resource.resolve_texture(path)
    }

    /// A block sound type: a key of `block_sounds` in the merged `sounds.json`, compared as
    /// written. The place is the pack whose `block_sounds` won.
    #[cfg(feature = "sounds_json")]
    pub(crate) fn block_sound(&self, name: &str) -> Target<'a> {
        let bindings = self.set().resource.sounds_json();
        if !bindings.value().block_sounds.contains_key(name) {
            return Target::Missing(None);
        }
        bindings
            .origin(&"block_sounds".to_owned())
            .map_or(Target::BuiltIn, Target::Defined)
    }

    /// A sound played by name: a key of `individual_named_sounds` in the merged `sounds.json`,
    /// compared with case.
    #[cfg(feature = "sounds_json")]
    pub(crate) fn named_sound(&self, name: &str) -> Target<'a> {
        let bindings = self.set().resource.sounds_json();
        if !bindings
            .value()
            .individual_named_sounds
            .sounds
            .contains_key(name)
        {
            if self.host_has(kinds::NAMED_SOUND, name) {
                return Target::BuiltIn;
            }
            return Target::Missing(None);
        }
        bindings
            .origin(&"individual_named_sounds".to_owned())
            .map_or(Target::BuiltIn, Target::Defined)
    }

    /// A level sound event of the game (`LevelSoundEvent`), by its name as written in lower
    /// case: the host's answer, or without a host the game's tables.
    #[cfg(any(feature = "sounds_json", feature = "entity"))]
    pub(crate) fn level_sound_event(&self, name: &str) -> Target<'a> {
        let known = match self.host_built_ins() {
            Some(host) => host.contains(kinds::LEVEL_SOUND_EVENT, name),
            None => GeneratedBuiltIns.contains(kinds::LEVEL_SOUND_EVENT, name),
        };
        if known {
            Target::BuiltIn
        } else {
            Target::Missing(None)
        }
    }

    /// A localisation key: a key some language the game loads defines in either stack,
    /// compared without ASCII case. The resource stack's languages are asked first, each
    /// stack's in the order its `languages.json` lists them.
    ///
    /// The game loads the languages that some `texts/languages.json` of the stack lists, each
    /// from every pack. Its own packs, which are on every stack the game builds, list its 29
    /// languages ([`VANILLA_LANGUAGES`](crate::vanilla::names::VANILLA_LANGUAGES)): so the
    /// `texts/<code>.lang` files of those codes count too when no pack of the stack lists the
    /// code (the usual add-on has `texts/en_US.lang` and no `languages.json`). A `.lang` file
    /// of any other code that no pack lists is never loaded, and its keys are no keys.
    ///
    /// When such a file of the base language (`en_US`, which every other language starts
    /// from and falls back to) is listed by its pack and its bytes are not resident, a key
    /// nothing else defines is [`Target::NotInView`]: nobody can tell. An unread file of
    /// another language leaves nothing unjudged: a key is held to the base language.
    ///
    /// A hit in the first language loads that language alone. A miss has asked every listed
    /// language of both stacks, loading each on its first use in the pass, and has read the
    /// unlisted files of the game's languages once: the first miss costs the load of all of
    /// them, each later one a probe per language. A rule over many keys has its own cost test.
    // Language lookups are ASCII case-insensitive. The game loads the languages that some
    // `texts/languages.json` of the stack lists, each from every pack, and its own packs list
    // their languages.
    #[cfg(all(feature = "lang", feature = "languages"))]
    pub(crate) fn lang_key(&self, key: &str) -> Target<'_> {
        fn find<'a, S: crate::pack::Side>(
            stack: &'a crate::pack::PackStack<S>,
            key: &String,
        ) -> Option<Target<'a>> {
            stack.languages().value().codes.iter().find_map(|code| {
                let table = stack.translations(code)?;
                table
                    .value()
                    .strings
                    .contains_key(key)
                    .then(|| table.origin(key).map_or(Target::BuiltIn, Target::Defined))
            })
        }
        let key = key.to_ascii_lowercase();
        let set = self.set();
        if let Some(found) = find(&set.resource, &key).or_else(|| find(&set.behavior, &key)) {
            return found;
        }
        let unlisted = self.unlisted_languages();
        if let Some(origin) = unlisted.keys.get(&key) {
            Target::Defined(origin)
        } else if self.host_has(kinds::LANG_KEY, &key) {
            Target::BuiltIn
        } else if unlisted.unread {
            Target::NotInView
        } else {
            Target::Missing(None)
        }
    }

    /// The keys of the `.lang` files of the game's languages that their stack does not list,
    /// read once per pass.
    #[cfg(all(feature = "lang", feature = "languages"))]
    fn unlisted_languages(&self) -> &UnlistedLanguages {
        fn read<S: crate::pack::Side>(
            stack: &crate::pack::PackStack<S>,
            side: crate::pack::PackSide,
            into: &mut UnlistedLanguages,
        ) {
            use crate::families::lang::{LangFile, Language};
            use crate::storage::{PackPath, StorageError};
            let listed = &stack.languages().value().codes;
            for code in names::VANILLA_LANGUAGES {
                if listed.iter().any(|language| language.as_str() == code) {
                    continue;
                }
                let Ok(path) = PackPath::new(&format!("texts/{code}.lang")) else {
                    continue;
                };
                // The packs are top first: a key is placed at the file of the highest pack.
                let packs = stack.stack_packs().iter().zip(stack.instances());
                for (entry, instance) in packs {
                    let has = entry
                        .files_below("texts")
                        .binary_search_by(|file| file.key.as_str().cmp(path.as_str()))
                        .is_ok();
                    if !has {
                        continue;
                    }
                    match instance.pack.resident(&path) {
                        Ok(bytes) => {
                            let (file, _) = LangFile::from_bytes(Language::new(code), &bytes);
                            for (key, _) in file.translations().strings {
                                into.keys.entry(key).or_insert_with(|| {
                                    Origin::file(entry.pack.clone(), side, path.clone())
                                });
                            }
                        }
                        // Every language starts from the base language and falls back to
                        // it: only its unread file leaves a key unknown.
                        Err(StorageError::NotResident) => {
                            into.unread |= code == names::VANILLA_LANGUAGES[0];
                        }
                        Err(_) => {}
                    }
                }
            }
        }
        self.caches.special.unlisted_languages.get_or_init(|| {
            use crate::pack::PackSide;
            let mut unlisted = UnlistedLanguages::default();
            read(&self.set().resource, PackSide::Resource, &mut unlisted);
            read(&self.set().behavior, PackSide::Behavior, &mut unlisted);
            unlisted
        })
    }

    /// A particle type of the game (`ParticleType`): what a legacy particle member takes in
    /// place of the identifier of a particle effect. `name` is passed in lower case, as the
    /// game compares it.
    #[allow(clippy::unused_self, reason = "a lookup like the others")]
    pub(crate) fn particle_type(&self, name: &str) -> Target<'a> {
        if names::is_particle_type(name) {
            Target::BuiltIn
        } else {
            Target::Missing(None)
        }
    }

    /// A native script module (`@minecraft/server`), by its name byte for byte: one the
    /// crate's module metadata lists.
    #[cfg(feature = "script")]
    #[allow(clippy::unused_self, reason = "a lookup like the others")]
    pub(crate) fn script_module(&self, name: &str) -> Target<'a> {
        if crate::families::script::meta::module(name).is_some() {
            Target::BuiltIn
        } else {
            Target::Missing(None)
        }
    }

    /// A command, as typed without `/`: its name or one of its aliases.
    #[allow(clippy::unused_self, reason = "a lookup like the others")]
    pub(crate) fn command(&self, name: &str) -> Target<'a> {
        if bedrock_addon_core::vanilla::command(name).is_some() {
            Target::BuiltIn
        } else {
            Target::Missing(None)
        }
    }

    /// The pack of either stack whose header UUID is `uuid`: the behaviour stack's first.
    pub(crate) fn pack(&self, uuid: Uuid) -> Option<PackRef> {
        let set = self.set();
        set.behavior
            .packs()
            .chain(set.resource.packs())
            .find(|pack| pack.id().uuid == uuid)
    }
}

#[cfg(test)]
mod tests {
    use super::LOOKUPS;

    #[test]
    fn the_lookup_table_holds_every_identifier_lookup() {
        let mut listed = LOOKUPS.to_vec();
        listed.sort_unstable();
        let before = listed.len();
        listed.dedup();
        assert_eq!(listed.len(), before, "a lookup is listed twice");
        assert_eq!(
            listed,
            [
                "aim_assist_category",
                "aim_assist_preset",
                "atmosphere_settings",
                "behavior_animation",
                "behavior_animation_controller",
                "behavior_tree",
                "biome",
                "block",
                "camera_preset",
                "client_animation",
                "client_animation_controller",
                "client_block",
                "client_entity",
                "color_grading_settings",
                "cubemap_settings",
                "culling_rules",
                "dimension",
                "entity",
                "feature",
                "fog",
                "function",
                "geometry",
                "item_atlas_key",
                "item_definition",
                "jigsaw_structure",
                "lighting_settings",
                "music_definition",
                "particle",
                "poi_tag",
                "poi_type",
                "processor_list",
                "render_controller",
                "sound_definition",
                "spawn_rules",
                "structure",
                "structure_set",
                "template_pool",
                "terrain_atlas_key",
                "trade_table",
                "voxel_shape",
                "water_settings",
            ],
            "the identifier lookups differ from the table's"
        );
    }
}
