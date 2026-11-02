//! Recipes: `Recipes`, `RecipeNamespace`, `Recipe`, `Shaped`, `Shapeless`, `Furnace`, `BrewingMix`,
//! `BrewingContainer`, `MaterialReduction`, `SmithingTransform`, `SmithingTrim`.
//!
//! Recipes: `recipes/**/*.json` of a behaviour pack. Not a version chain: vanilla reads every
//! recipe type with one hand-written walker at every `format_version`, so each type is one struct
//! and the two document gates are the predicates below.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod ingredient;
mod load;
mod new;
mod parse;
mod write;

use indexmap::IndexMap;

use bedrock_addon_core::diagnostic::Diagnostic;
use bedrock_addon_core::document::{Document, Extra, Registry, Rejected, registry};
use bedrock_addon_core::id::RecipeId;
use bedrock_addon_core::molang::MolangList;
use bedrock_addon_core::version::{FormatVersion, MinEngineVersion, SemVersion};

#[doc(hidden)]
pub use load::load_recipes;

/// The `format_version` from which a shaped or shapeless recipe must have `unlock`: the game
/// requires unlock data from 1.20.10 on.
const UNLOCK_REQUIRED_SINCE: FormatVersion = FormatVersion::new(1, 20, 10);

/// The newest pack `min_engine_version` whose input ingredients may carry `count`: the game
/// treats a recipe as legacy while the pack's version is not above 1.19.22, and only legacy
/// recipes keep `count` on inputs.
const LEGACY_RECIPE_UNTIL: SemVersion = SemVersion::new(1, 19, 22);

/// Whether a shaped or shapeless recipe written in `format_version` must have `unlock`: from
/// 1.20.10 a recipe without it is not loaded ("1.20+ Recipes require unlock data"); below, the
/// game fills in a default requirement.
pub fn unlock_required(format_version: FormatVersion) -> bool {
    format_version >= UNLOCK_REQUIRED_SINCE
}

/// Whether input ingredients (shaped `key` values, brewing and material-reduction `input`) may
/// carry `count` in a pack with this `min_engine_version`: only up to 1.19.22, inclusive
/// ("legacy recipes"). Reagents never may.
pub fn input_count_allowed(min_engine_version: &MinEngineVersion) -> bool {
    min_engine_version.version <= LEGACY_RECIPE_UNTIL
}

/// A recipe file (`minecraft:recipe_*`): one of eight recipe types, each with the members its
/// type reads.
///
/// JSON: `{"format_version": "1.20.10", "minecraft:recipe_shaped": {"description":
/// {"identifier": …}, "tags": [...], …}}`. The payload key picks the type; vanilla takes the
/// first present one in the order of [`RecipeKind`]. Vanilla registers shaped, shapeless and
/// furnace recipes once per tag in `tags`, so one without tags registers nothing, although its
/// identifier still overrides lower packs.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
#[doc(alias = "recipes")]
pub enum Recipe {
    /// `minecraft:recipe_shaped`.
    Shaped(Shaped),
    /// `minecraft:recipe_shapeless`.
    Shapeless(Shapeless),
    /// `minecraft:recipe_furnace`.
    Furnace(Furnace),
    /// `minecraft:recipe_brewing_mix`.
    BrewingMix(BrewingMix),
    /// `minecraft:recipe_brewing_container`.
    BrewingContainer(BrewingContainer),
    /// `minecraft:recipe_material_reduction`.
    MaterialReduction(MaterialReduction),
    /// `minecraft:recipe_smithing_transform`.
    SmithingTransform(SmithingTransform),
    /// `minecraft:recipe_smithing_trim`.
    SmithingTrim(SmithingTrim),
}

/// The type of a recipe: its payload key. Declared in the order vanilla probes the payload keys
/// (the first present one wins), which is also the order of vanilla's `RecipeType`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum RecipeKind {
    /// `minecraft:recipe_shaped`.
    Shaped,
    /// `minecraft:recipe_shapeless`.
    Shapeless,
    /// `minecraft:recipe_furnace`.
    Furnace,
    /// `minecraft:recipe_brewing_mix`.
    BrewingMix,
    /// `minecraft:recipe_brewing_container`.
    BrewingContainer,
    /// `minecraft:recipe_material_reduction`.
    MaterialReduction,
    /// `minecraft:recipe_smithing_transform`.
    SmithingTransform,
    /// `minecraft:recipe_smithing_trim`.
    SmithingTrim,
}

/// Where recipe identifiers must be unique across packs: shaped and shapeless recipes share one
/// namespace, every other type has its own, so a furnace recipe and a shaped recipe may share an
/// identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum RecipeNamespace {
    /// Shaped and shapeless recipes.
    Crafting,
    /// Furnace recipes.
    Furnace,
    /// Brewing mixes.
    BrewingMix,
    /// Brewing container recipes.
    BrewingContainer,
    /// Material-reduction recipes.
    MaterialReduction,
    /// Smithing transforms.
    SmithingTransform,
    /// Smithing trims.
    SmithingTrim,
}

/// The recipes of a behaviour stack (`bp.recipes()`): one registry per [`RecipeNamespace`],
/// each keyed by identifier, the highest pack's valid definition winning.
///
/// A file that fails does not reserve its identifier, so a lower pack's recipe of the same
/// namespace and identifier loads; of two files of one pack with the same identifier (of any
/// type), the first loads and the second is rejected.
#[non_exhaustive]
pub struct Recipes {
    /// Shaped and shapeless recipes.
    pub crafting: Registry<Recipe>,
    /// Furnace recipes.
    pub furnace: Registry<Recipe>,
    /// Brewing mixes.
    pub brewing_mix: Registry<Recipe>,
    /// Brewing container recipes.
    pub brewing_container: Registry<Recipe>,
    /// Material-reduction recipes.
    pub material_reduction: Registry<Recipe>,
    /// Smithing transforms.
    pub smithing_transform: Registry<Recipe>,
    /// Smithing trims.
    pub smithing_trim: Registry<Recipe>,
    /// Files rejected before their type was known (unreadable JSON, no payload key).
    untyped: Vec<Rejected>,
}

/// `description` of every recipe type.
#[derive(Clone, Debug, PartialEq)]
pub struct Description {
    /// The recipe's identifier, any non-empty string (JSON `identifier`).
    pub identifier: RecipeId,
    /// Unknown members.
    pub extra: Extra,
}

/// `tags`: the crafting stations that offer the recipe (`crafting_table`, `furnace`,
/// `smithing_table`, …), as one string or a list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Tags {
    /// `"tags": "material_reducer"`.
    One(String),
    /// `"tags": ["crafting_table", …]`.
    Many(Vec<String>),
}

/// A recipe ingredient or result: an item by name, or an object with `item` or `tag`.
///
/// The string form is `"namespace:name[:aux]"` (namespace `minecraft` when omitted, case
/// ignored, an `item.` prefix dropped) or a potion, `"minecraft:potion_type:<potion>"`.
#[derive(Clone, Debug, PartialEq)]
pub enum Ingredient {
    /// `"minecraft:stick"`, `"minecraft:wool:3"`, `"minecraft:potion_type:awkward"`.
    Name(String),
    /// `{"item": …, "data": …, "count": …, "tag": …}`.
    Object(IngredientObject),
}

/// The object form of an [`Ingredient`].
#[derive(Clone, Debug, Default)]
pub struct IngredientObject {
    /// The item, `"namespace:name[:aux]"`; an inline aux wins over `data` (JSON `item`).
    pub item: Option<String>,
    /// The aux value: a number, or a Molang expression vanilla evaluates once at load and
    /// truncates toward zero; absent means any aux (JSON `data`).
    pub data: Option<MolangList>,
    /// How many, default 1; never on a reagent, and on an input ingredient only in packs whose
    /// `min_engine_version` is at most 1.19.22 (JSON `count`).
    pub count: Option<i32>,
    /// An item tag standing for every item that carries it; read only where tags are allowed
    /// and `item` is not a string (JSON `tag`).
    pub tag: Option<String>,
    /// What `data` evaluated to when the recipe loaded: `None` when absent, unevaluated (an
    /// expression that does not fold to a constant) or a list. Derived at load, not written and
    /// not part of equality; read it through [`Ingredient::aux`].
    #[doc(hidden)]
    pub data_value: Option<i32>,
    /// Unknown members, and members vanilla ignores because of their JSON type.
    pub extra: Extra,
}

/// Equal when the members as written are equal: `data_value` is derived at load.
impl PartialEq for IngredientObject {
    fn eq(&self, other: &Self) -> bool {
        self.item == other.item
            && self.data == other.data
            && self.count == other.count
            && self.tag == other.tag
            && self.extra == other.extra
    }
}

/// An item name as vanilla splits a `"namespace:name[:aux]"` text.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct ItemName {
    /// The namespace, lower-cased; `minecraft` when the text has none.
    pub namespace: String,
    /// The name, lower-cased, without an `item.` prefix.
    pub name: String,
    /// The inline aux value, 0 to 255, when the text has a valid one.
    pub aux: Option<i32>,
    /// The potion a `minecraft:potion_type:<potion>` text names (`minecraft:awkward`); the item
    /// is then `minecraft:potion`.
    pub potion: Option<String>,
}

/// `result` (shaped, shapeless) and material-reduction `output`: one item or a list.
#[derive(Clone, Debug, PartialEq)]
pub enum Output {
    /// One item.
    One(Ingredient),
    /// A list of items.
    Many(Vec<Ingredient>),
}

/// `unlock`: what unlocks the recipe in the recipe book.
#[derive(Clone, Debug, PartialEq)]
pub enum Unlock {
    /// `[ingredient, …]`: holding any of these items.
    Ingredients(Vec<Ingredient>),
    /// `{"context": "…"}`.
    Context {
        /// The unlocking context (JSON `context`).
        context: UnlockContext,
        /// Unknown members.
        extra: Extra,
    },
    /// A list with an invalid element, or an object whose `context` names no context: vanilla
    /// logs it and reads it as context `None` with no items. Kept as written.
    Malformed(bedrock_addon_core::json::Value),
}

/// `unlock.context`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum UnlockContext {
    /// `"None"`: no context unlocks it.
    None,
    /// `"AlwaysUnlocked"`.
    AlwaysUnlocked,
    /// `"PlayerInWater"`.
    PlayerInWater,
    /// `"PlayerHasManyItems"`.
    PlayerHasManyItems,
}

/// A recipe's unlocking requirement after vanilla's defaults: the context and the items.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct UnlockRequirement {
    /// The context.
    pub context: UnlockContext,
    /// The items that unlock it.
    pub ingredients: Vec<Ingredient>,
}

/// A potion brewing mix: input and output are potions, so vanilla adds the mix for the potion,
/// splash and lingering containers alike.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct PotionMix {
    /// The input potion (`minecraft:awkward`).
    pub input: String,
    /// The output potion.
    pub output: String,
}

impl PotionMix {
    /// The containers a potion mix is added for.
    pub const CONTAINERS: [&'static str; 3] = [
        "minecraft:potion",
        "minecraft:splash_potion",
        "minecraft:lingering_potion",
    ];
}

/// `minecraft:recipe_shaped`: a crafting-grid pattern of at most 3×3 symbols.
#[derive(Clone, Debug, Default, PartialEq)]
#[doc(alias = "minecraft:recipe_shaped")]
pub struct Shaped {
    /// The file's `format_version`; `None` when absent or unreadable (vanilla then uses 1.0.0).
    pub format_version: Option<FormatVersion>,
    /// JSON `description`.
    pub description: Description,
    /// JSON `tags`.
    pub tags: Option<Tags>,
    /// Lower is tried first, default 0 (JSON `priority`).
    pub priority: Option<i32>,
    /// Whether the mirrored pattern also matches, default `true` (JSON `assume_symmetry`).
    pub assume_symmetry: Option<bool>,
    /// The grid rows, one symbol per character, at most 3 rows of 3 (JSON `pattern`).
    pub pattern: Vec<String>,
    /// The ingredient of each symbol; a member name's first byte is the symbol (JSON `key`).
    pub key: IndexMap<String, Ingredient>,
    /// What the recipe makes; several items are allowed (JSON `result`).
    pub result: Output,
    /// Required from `format_version` 1.20.10 (JSON `unlock`).
    pub unlock: Option<Unlock>,
    /// Unknown members of the payload.
    pub extra: Extra,
    /// Unknown root members (beside `format_version` and the payload).
    pub root_extra: Extra,
}

/// `minecraft:recipe_shapeless`: ingredients in any arrangement.
#[derive(Clone, Debug, Default, PartialEq)]
#[doc(alias = "minecraft:recipe_shapeless")]
pub struct Shapeless {
    /// The file's `format_version`; `None` when absent or unreadable.
    pub format_version: Option<FormatVersion>,
    /// JSON `description`.
    pub description: Description,
    /// JSON `tags`.
    pub tags: Option<Tags>,
    /// Lower is tried first, default 0 (JSON `priority`).
    pub priority: Option<i32>,
    /// The ingredients; one with `count` n fills n slots, none when n ≤ 0 (JSON
    /// `ingredients`).
    pub ingredients: Vec<Ingredient>,
    /// What the recipe makes; only the first item of a list is used (JSON `result`).
    pub result: Output,
    /// Required from `format_version` 1.20.10 (JSON `unlock`).
    pub unlock: Option<Unlock>,
    /// Unknown members of the payload.
    pub extra: Extra,
    /// Unknown root members.
    pub root_extra: Extra,
}

/// `minecraft:recipe_furnace`: one input smelted into one output, once per station tag
/// (`furnace`, `smoker`, `blast_furnace`, `campfire`, `soul_campfire`).
#[derive(Clone, Debug, Default, PartialEq)]
#[doc(alias = "minecraft:recipe_furnace")]
pub struct Furnace {
    /// The file's `format_version`; `None` when absent or unreadable.
    pub format_version: Option<FormatVersion>,
    /// JSON `description`.
    pub description: Description,
    /// JSON `tags`.
    pub tags: Option<Tags>,
    /// Lower is tried first, default 0 (JSON `priority`).
    pub priority: Option<i32>,
    /// What is smelted, one item, never with `count` (JSON `input`).
    pub input: Ingredient,
    /// What it smelts into (JSON `output`).
    pub output: Ingredient,
    /// Optional at every version; absent means always unlocked (JSON `unlock`).
    pub unlock: Option<Unlock>,
    /// Unknown members of the payload.
    pub extra: Extra,
    /// Unknown root members.
    pub root_extra: Extra,
}

/// `minecraft:recipe_brewing_mix`: a reagent turns one potion (or item) into another.
#[derive(Clone, Debug, Default, PartialEq)]
#[doc(alias = "minecraft:recipe_brewing_mix")]
pub struct BrewingMix {
    /// The file's `format_version`; `None` when absent or unreadable.
    pub format_version: Option<FormatVersion>,
    /// JSON `description`.
    pub description: Description,
    /// JSON `tags`.
    pub tags: Option<Tags>,
    /// The potion or item brewed (JSON `input`).
    pub input: Ingredient,
    /// What is added: one item, without `count` or `tag` (JSON `reagent`).
    pub reagent: Ingredient,
    /// The potion or item it becomes (JSON `output`).
    pub output: Ingredient,
    /// Unknown members of the payload.
    pub extra: Extra,
    /// Unknown root members.
    pub root_extra: Extra,
}

/// `minecraft:recipe_brewing_container`: a reagent turns one potion container into another.
#[derive(Clone, Debug, Default, PartialEq)]
#[doc(alias = "minecraft:recipe_brewing_container")]
pub struct BrewingContainer {
    /// The file's `format_version`; `None` when absent or unreadable.
    pub format_version: Option<FormatVersion>,
    /// JSON `description`.
    pub description: Description,
    /// JSON `tags`.
    pub tags: Option<Tags>,
    /// The container brewed (JSON `input`).
    pub input: Ingredient,
    /// What is added: one item, without `count` or `tag` (JSON `reagent`).
    pub reagent: Ingredient,
    /// The container it becomes (JSON `output`).
    pub output: Ingredient,
    /// Unknown members of the payload.
    pub extra: Extra,
    /// Unknown root members.
    pub root_extra: Extra,
}

/// `minecraft:recipe_material_reduction`: what the material reducer splits an item into.
#[derive(Clone, Debug, Default, PartialEq)]
#[doc(alias = "minecraft:recipe_material_reduction")]
pub struct MaterialReduction {
    /// The file's `format_version`; `None` when absent or unreadable.
    pub format_version: Option<FormatVersion>,
    /// JSON `description`.
    pub description: Description,
    /// JSON `tags`.
    pub tags: Option<Tags>,
    /// The item reduced (JSON `input`).
    pub input: Ingredient,
    /// At most nine items, as one object or a list (JSON `output`).
    pub output: Option<Output>,
    /// Unknown members of the payload.
    pub extra: Extra,
    /// Unknown root members.
    pub root_extra: Extra,
}

/// `minecraft:recipe_smithing_transform`: a smithing-table upgrade of `base` with `addition`.
#[derive(Clone, Debug, Default, PartialEq)]
#[doc(alias = "minecraft:recipe_smithing_transform")]
pub struct SmithingTransform {
    /// The file's `format_version`; `None` when absent or unreadable.
    pub format_version: Option<FormatVersion>,
    /// JSON `description`.
    pub description: Description,
    /// Exactly `["smithing_table"]` (JSON `tags`).
    pub tags: Option<Tags>,
    /// The smithing template; not read in worlds older than 1.20 (JSON `template`).
    pub template: Option<Ingredient>,
    /// The armour or tool upgraded (JSON `base`).
    pub base: Ingredient,
    /// The material: an item tagged `minecraft:transform_materials` (JSON `addition`).
    pub addition: Ingredient,
    /// What it becomes, always one item (JSON `result`).
    pub result: Ingredient,
    /// Unknown members of the payload.
    pub extra: Extra,
    /// Unknown root members.
    pub root_extra: Extra,
}

/// `minecraft:recipe_smithing_trim`: a smithing-table armour trim, only in worlds from 1.20.
#[derive(Clone, Debug, Default, PartialEq)]
#[doc(alias = "minecraft:recipe_smithing_trim")]
pub struct SmithingTrim {
    /// The file's `format_version`; `None` when absent or unreadable.
    pub format_version: Option<FormatVersion>,
    /// JSON `description`.
    pub description: Description,
    /// Exactly `["smithing_table"]` (JSON `tags`).
    pub tags: Option<Tags>,
    /// The trim pattern template (JSON `template`).
    pub template: Ingredient,
    /// The armour trimmed (JSON `base`).
    pub base: Ingredient,
    /// The trim material (JSON `addition`).
    pub addition: Ingredient,
    /// Unknown members of the payload.
    pub extra: Extra,
    /// Unknown root members.
    pub root_extra: Extra,
}

impl Recipes {
    /// The recipe `id` of `namespace`.
    pub fn get(&self, namespace: RecipeNamespace, id: &RecipeId) -> Option<&Document<Recipe>> {
        self.registry(namespace).get(id)
    }

    /// The registry of `namespace`.
    pub fn registry(&self, namespace: RecipeNamespace) -> &Registry<Recipe> {
        match namespace {
            RecipeNamespace::Crafting => &self.crafting,
            RecipeNamespace::Furnace => &self.furnace,
            RecipeNamespace::BrewingMix => &self.brewing_mix,
            RecipeNamespace::BrewingContainer => &self.brewing_container,
            RecipeNamespace::MaterialReduction => &self.material_reduction,
            RecipeNamespace::SmithingTransform => &self.smithing_transform,
            RecipeNamespace::SmithingTrim => &self.smithing_trim,
        }
    }

    /// Every effective recipe: namespace by namespace in the order of [`RecipeNamespace`], each
    /// in load order.
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            recipes: self,
            next_namespace: 0,
            current: None,
        }
    }

    /// How many effective recipes there are, over every namespace.
    pub fn len(&self) -> usize {
        RecipeNamespace::ALL
            .iter()
            .map(|namespace| self.registry(*namespace).len())
            .sum()
    }

    /// No recipe loaded.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Every rejected file: those rejected before their type was known, then each namespace's.
    pub fn rejected(&self) -> impl Iterator<Item = &Rejected> + '_ {
        self.untyped.iter().chain(
            RecipeNamespace::ALL
                .iter()
                .flat_map(|namespace| self.registry(*namespace).rejected()),
        )
    }

    /// Every finding: those of the files rejected before their type was known, then each
    /// namespace's.
    pub fn diagnostics(&self) -> impl Iterator<Item = &Diagnostic> + '_ {
        self.untyped
            .iter()
            .flat_map(|rejected| rejected.diagnostics.iter())
            .chain(
                RecipeNamespace::ALL
                    .iter()
                    .flat_map(|namespace| self.registry(*namespace).diagnostics()),
            )
    }
}

impl RecipeNamespace {
    /// Every namespace, in declaration order.
    pub const ALL: &'static [RecipeNamespace] = &[
        RecipeNamespace::Crafting,
        RecipeNamespace::Furnace,
        RecipeNamespace::BrewingMix,
        RecipeNamespace::BrewingContainer,
        RecipeNamespace::MaterialReduction,
        RecipeNamespace::SmithingTransform,
        RecipeNamespace::SmithingTrim,
    ];
}

impl RecipeKind {
    /// Every type, in the order vanilla probes the payload keys: a file with several payload
    /// keys is read as the first type in this list.
    pub const ALL: &'static [RecipeKind] = &[
        RecipeKind::Shaped,
        RecipeKind::Shapeless,
        RecipeKind::Furnace,
        RecipeKind::BrewingMix,
        RecipeKind::BrewingContainer,
        RecipeKind::MaterialReduction,
        RecipeKind::SmithingTransform,
        RecipeKind::SmithingTrim,
    ];

    /// The payload key (`minecraft:recipe_shaped`).
    pub fn payload_key(self) -> &'static str {
        match self {
            Self::Shaped => "minecraft:recipe_shaped",
            Self::Shapeless => "minecraft:recipe_shapeless",
            Self::Furnace => "minecraft:recipe_furnace",
            Self::BrewingMix => "minecraft:recipe_brewing_mix",
            Self::BrewingContainer => "minecraft:recipe_brewing_container",
            Self::MaterialReduction => "minecraft:recipe_material_reduction",
            Self::SmithingTransform => "minecraft:recipe_smithing_transform",
            Self::SmithingTrim => "minecraft:recipe_smithing_trim",
        }
    }

    /// The type of a payload key.
    pub fn from_payload_key(key: &str) -> Option<RecipeKind> {
        Self::ALL
            .iter()
            .copied()
            .find(|kind| kind.payload_key() == key)
    }

    /// The namespace its identifiers live in.
    pub fn namespace(self) -> RecipeNamespace {
        match self {
            Self::Shaped | Self::Shapeless => RecipeNamespace::Crafting,
            Self::Furnace => RecipeNamespace::Furnace,
            Self::BrewingMix => RecipeNamespace::BrewingMix,
            Self::BrewingContainer => RecipeNamespace::BrewingContainer,
            Self::MaterialReduction => RecipeNamespace::MaterialReduction,
            Self::SmithingTransform => RecipeNamespace::SmithingTransform,
            Self::SmithingTrim => RecipeNamespace::SmithingTrim,
        }
    }
}

impl Tags {
    /// The tags, in order.
    pub fn as_slice(&self) -> &[String] {
        match self {
            Self::One(tag) => std::slice::from_ref(tag),
            Self::Many(tags) => tags,
        }
    }
}

impl Output {
    /// The items, in order.
    pub fn as_slice(&self) -> &[Ingredient] {
        match self {
            Self::One(item) => std::slice::from_ref(item),
            Self::Many(items) => items,
        }
    }
}

impl UnlockContext {
    /// The JSON spelling (`"AlwaysUnlocked"`).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::AlwaysUnlocked => "AlwaysUnlocked",
            Self::PlayerInWater => "PlayerInWater",
            Self::PlayerHasManyItems => "PlayerHasManyItems",
        }
    }

    /// The context a JSON spelling names; case matters.
    pub fn from_name(name: &str) -> Option<UnlockContext> {
        match name {
            "None" => Some(Self::None),
            "AlwaysUnlocked" => Some(Self::AlwaysUnlocked),
            "PlayerInWater" => Some(Self::PlayerInWater),
            "PlayerHasManyItems" => Some(Self::PlayerHasManyItems),
            _ => None,
        }
    }
}

/// Every effective recipe of [`Recipes`], namespace by namespace; see [`Recipes::iter`].
pub struct Iter<'a> {
    recipes: &'a Recipes,
    next_namespace: usize,
    current: Option<registry::Iter<'a, Recipe>>,
}

impl std::fmt::Debug for Iter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Iter").finish_non_exhaustive()
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a RecipeId, &'a Document<Recipe>);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(item) = self.current.as_mut().and_then(Iterator::next) {
                return Some(item);
            }
            let namespace = *RecipeNamespace::ALL.get(self.next_namespace)?;
            self.next_namespace += 1;
            self.current = Some(self.recipes.registry(namespace).iter());
        }
    }
}

impl std::iter::FusedIterator for Iter<'_> {}

impl<'a> IntoIterator for &'a Recipes {
    type Item = (&'a RecipeId, &'a Document<Recipe>);
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
