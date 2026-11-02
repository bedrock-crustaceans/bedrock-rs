//! Defaults and derived views of recipes.

use super::{
    BrewingMix, Description, Furnace, Ingredient, Output, PotionMix, Recipe, RecipeKind,
    RecipeNamespace, Shaped, Shapeless, Tags, Unlock, UnlockContext, UnlockRequirement,
};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::id::RecipeId;
use bedrock_addon_core::version::FormatVersion;

/// The version vanilla reads a recipe without a readable `format_version` as.
pub(super) const MISSING_VERSION: FormatVersion = FormatVersion::new(1, 0, 0);

impl Default for Description {
    /// An empty identifier: not loadable until it is given one.
    fn default() -> Self {
        Self {
            identifier: RecipeId::parse(""),
            extra: Extra::default(),
        }
    }
}

impl Default for Output {
    /// No item.
    fn default() -> Self {
        Self::Many(Vec::new())
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
pub(super) use each;

impl Recipe {
    /// The recipe's type.
    #[must_use]
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
    #[must_use]
    pub fn namespace(&self) -> RecipeNamespace {
        self.kind().namespace()
    }

    /// `description`.
    #[must_use]
    pub fn description(&self) -> &Description {
        each!(self, r => &r.description)
    }

    /// `description.identifier`.
    #[must_use]
    pub fn identifier(&self) -> &RecipeId {
        &self.description().identifier
    }

    /// The tags (`tags`), empty when there are none.
    pub fn tags(&self) -> &[String] {
        each!(self, r => r.tags.as_ref().map_or(&[][..], Tags::as_slice))
    }

    /// The file's `format_version`; `None` when absent or unreadable.
    #[must_use]
    pub fn format_version(&self) -> Option<FormatVersion> {
        each!(self, r => r.format_version)
    }

    /// The version vanilla's gates compare against: `format_version`, or 1.0.0 without one.
    #[must_use]
    pub fn effective_version(&self) -> FormatVersion {
        self.format_version().unwrap_or(MISSING_VERSION)
    }

    /// The unlocking requirement of a shaped, shapeless or furnace recipe, after vanilla's
    /// defaults, as loaded in a base-game pack or not; `None` for the other types, which have
    /// none.
    ///
    /// Without `unlock`, a furnace recipe is always unlocked; so is a shaped or shapeless one
    /// below `format_version` 1.20.10, except in a base-game pack, where its context is `None`.
    #[must_use]
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
    #[must_use]
    pub fn assumes_symmetry(&self) -> bool {
        self.assume_symmetry.unwrap_or(true)
    }

    /// `priority`, default 0.
    #[must_use]
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
    #[must_use]
    pub fn result_item(&self) -> Option<&Ingredient> {
        self.result.as_slice().first()
    }

    /// `priority`, default 0.
    #[must_use]
    pub fn effective_priority(&self) -> i32 {
        self.priority.unwrap_or(0)
    }
}

impl Furnace {
    /// `priority`, default 0.
    #[must_use]
    pub fn effective_priority(&self) -> i32 {
        self.priority.unwrap_or(0)
    }
}

impl BrewingMix {
    /// The potion mix, when both `input` and `output` name potions: vanilla then adds it for
    /// the potion, splash and lingering containers ([`PotionMix::CONTAINERS`]).
    #[must_use]
    pub fn potion_mix(&self) -> Option<PotionMix> {
        Some(PotionMix {
            input: self.input.potion()?,
            output: self.output.potion()?,
        })
    }
}
