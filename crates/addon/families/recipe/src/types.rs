//! The recipe types, the `Recipes` collection, and their defaults and derived views.

use indexmap::IndexMap;

use bedrock_addon_core::diagnostic::Diagnostic;
use bedrock_addon_core::document::{Document, Registry, Rejected, registry};
use bedrock_addon_core::id::RecipeId;
use bedrock_addon_core::molang::MolangList;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_shared_shapes::OneOrMany;

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
///
/// Hand-written: it partitions one file kind into a registry per namespace, and a recipe is not
/// a version chain (vanilla's one walker reads every `format_version`), so no `JsonDocument`.
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
    pub(crate) untyped: Vec<Rejected>,
}

/// `description` of every recipe type.
#[derive(Clone, Debug, PartialEq)]
pub struct Description {
    /// The recipe's identifier, any non-empty string (JSON `identifier`).
    pub identifier: RecipeId,
}

/// `tags`: the crafting stations that offer the recipe (`crafting_table`, `furnace`,
/// `smithing_table`, …), as one string or a list.
///
/// `"tags": "material_reducer"` or `"tags": ["crafting_table", …]`.
pub type Tags = OneOrMany<String>;

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
}

/// Equal when the members as written are equal: `data_value` is derived at load.
impl PartialEq for IngredientObject {
    fn eq(&self, other: &Self) -> bool {
        self.item == other.item
            && self.data == other.data
            && self.count == other.count
            && self.tag == other.tag
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
/// No item is an empty list ([`OneOrMany::default`]).
pub type Output = OneOrMany<Ingredient>;

/// `unlock`: what unlocks the recipe in the recipe book.
#[derive(Clone, Debug, PartialEq)]
pub enum Unlock {
    /// `[ingredient, …]`: holding any of these items.
    Ingredients(Vec<Ingredient>),
    /// `{"context": "…"}`.
    Context {
        /// The unlocking context (JSON `context`).
        context: UnlockContext,
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

/// The version vanilla reads a recipe without a readable `format_version` as.
pub(crate) const MISSING_VERSION: FormatVersion = FormatVersion::new(1, 0, 0);

impl Default for Description {
    /// An empty identifier: not loadable until it is given one.
    fn default() -> Self {
        Self {
            identifier: RecipeId::parse(""),
        }
    }
}

/// Applies `$body` to the variant's struct, bound to `$r`.
macro_rules! each {
    ($recipe:expr, $r:ident => $body:expr) => {
        match $recipe {
            Recipe::Shaped($r) => $body,
            Recipe::Shapeless($r) => $body,
            Recipe::Furnace($r) => $body,
            Recipe::BrewingMix($r) => $body,
            Recipe::BrewingContainer($r) => $body,
            Recipe::MaterialReduction($r) => $body,
            Recipe::SmithingTransform($r) => $body,
            Recipe::SmithingTrim($r) => $body,
        }
    };
}
pub(crate) use each;

impl Recipe {
    /// The recipe's type.
    pub fn kind(&self) -> RecipeKind {
        match self {
            Self::Shaped(_) => RecipeKind::Shaped,
            Self::Shapeless(_) => RecipeKind::Shapeless,
            Self::Furnace(_) => RecipeKind::Furnace,
            Self::BrewingMix(_) => RecipeKind::BrewingMix,
            Self::BrewingContainer(_) => RecipeKind::BrewingContainer,
            Self::MaterialReduction(_) => RecipeKind::MaterialReduction,
            Self::SmithingTransform(_) => RecipeKind::SmithingTransform,
            Self::SmithingTrim(_) => RecipeKind::SmithingTrim,
        }
    }

    /// The namespace its identifier lives in.
    pub fn namespace(&self) -> RecipeNamespace {
        self.kind().namespace()
    }

    /// `description`.
    pub fn description(&self) -> &Description {
        each!(self, r => &r.description)
    }

    /// `description.identifier`.
    pub fn identifier(&self) -> &RecipeId {
        &self.description().identifier
    }

    /// The tags (`tags`), empty when there are none.
    pub fn tags(&self) -> &[String] {
        each!(self, r => r.tags.as_ref().map_or(&[][..], Tags::as_slice))
    }

    /// The file's `format_version`; `None` when absent or unreadable.
    pub fn format_version(&self) -> Option<FormatVersion> {
        each!(self, r => r.format_version)
    }

    /// The version vanilla's gates compare against: `format_version`, or 1.0.0 without one.
    pub fn effective_version(&self) -> FormatVersion {
        self.format_version().unwrap_or(MISSING_VERSION)
    }

    /// The unlocking requirement of a shaped, shapeless or furnace recipe, after vanilla's
    /// defaults, as loaded in a base-game pack or not; `None` for the other types, which have
    /// none.
    ///
    /// Without `unlock`, a furnace recipe is always unlocked; so is a shaped or shapeless one
    /// below `format_version` 1.20.10, except in a base-game pack, where its context is `None`.
    pub fn unlock_requirement(&self, is_base_game_pack: bool) -> Option<UnlockRequirement> {
        let (unlock, gated) = match self {
            Self::Shaped(r) => (r.unlock.as_ref(), true),
            Self::Shapeless(r) => (r.unlock.as_ref(), true),
            Self::Furnace(r) => (r.unlock.as_ref(), false),
            _ => return None,
        };
        let requirement = match unlock {
            Some(Unlock::Ingredients(ingredients)) => UnlockRequirement {
                context: UnlockContext::None,
                ingredients: ingredients.clone(),
            },
            Some(Unlock::Malformed(_)) => UnlockRequirement {
                context: UnlockContext::None,
                ingredients: Vec::new(),
            },
            Some(Unlock::Context { context, .. }) => UnlockRequirement {
                context: *context,
                ingredients: Vec::new(),
            },
            // Without `unlock` the game picks the context itself: a shaped or shapeless recipe of
            // a base-game pack gets `None`, every other recipe is always unlocked.
            None => UnlockRequirement {
                context: if gated && is_base_game_pack {
                    UnlockContext::None
                } else {
                    UnlockContext::AlwaysUnlocked
                },
                ingredients: Vec::new(),
            },
        };
        Some(requirement)
    }
}

impl Shaped {
    /// `assume_symmetry`, default `true`.
    pub fn assumes_symmetry(&self) -> bool {
        self.assume_symmetry.unwrap_or(true)
    }

    /// `priority`, default 0.
    pub fn effective_priority(&self) -> i32 {
        self.priority.unwrap_or(0)
    }
}

impl Shapeless {
    /// The crafting slots the ingredients fill: each ingredient once per unit of its `count`,
    /// none for a `count` of 0 or less.
    pub fn slots(&self) -> impl Iterator<Item = &Ingredient> + '_ {
        self.ingredients.iter().flat_map(|ingredient| {
            let count = usize::try_from(ingredient.count()).unwrap_or(0);
            std::iter::repeat_n(ingredient, count)
        })
    }

    /// The item made: the first of `result`.
    pub fn result_item(&self) -> Option<&Ingredient> {
        self.result.as_slice().first()
    }

    /// `priority`, default 0.
    pub fn effective_priority(&self) -> i32 {
        self.priority.unwrap_or(0)
    }
}

impl Furnace {
    /// `priority`, default 0.
    pub fn effective_priority(&self) -> i32 {
        self.priority.unwrap_or(0)
    }
}

impl BrewingMix {
    /// The potion mix, when both `input` and `output` name potions: vanilla then adds it for
    /// the potion, splash and lingering containers ([`PotionMix::CONTAINERS`]).
    pub fn potion_mix(&self) -> Option<PotionMix> {
        Some(PotionMix {
            input: self.input.potion()?,
            output: self.output.potion()?,
        })
    }
}
