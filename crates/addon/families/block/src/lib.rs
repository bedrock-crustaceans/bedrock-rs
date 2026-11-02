//! Blocks: `Block`, `Permutation`, `StateValues`, `Traits`, `component::*`, `history::*`.
//!
//! Custom blocks: `blocks/**/*.json` of a behaviour pack (`minecraft:block`). `Block` is the
//! newest document (format 1.26.20); older files are upgraded when they load, and `history` has
//! the older shapes.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![recursion_limit = "512"]

pub mod component;
pub mod history;

pub use states::{
    Connection, MultiBlock, PlacementDirection, PlacementPosition, StateValues, Traits,
};

mod load;
mod parse;
mod states;
mod validate;
mod write;

#[cfg(feature = "nbt")]
mod network;
mod new;

use indexmap::IndexMap;

use bedrock_addon_core::document::{ComponentMap, Extra};
use bedrock_addon_core::id::BlockId;
use bedrock_addon_core::json::{StrictBool, StrictFloat, StrictInt};
use bedrock_addon_core::molang::Molang;
use bedrock_addon_core::shared::MenuCategory;
use bedrock_addon_core::version::FormatVersion;

use component::{BlockComponent, DeprecatedComponents, DeprecatedDefinitionEvent};

#[doc(hidden)]
pub use load::load_registry;

/// A custom block (`minecraft:block`), at the newest format version.
///
/// JSON: `{"format_version": "1.26.20", "minecraft:block": {"description", "components",
/// "permutations"?}}`. The base components always apply; each permutation's components replace
/// them while its condition holds.
#[derive(Clone, Debug, PartialEq)]
#[doc(alias = "minecraft:block")]
pub struct Block {
    /// What the block is (JSON `description`).
    pub description: Description,
    /// The base components (JSON `components`).
    pub components: ComponentMap<BlockComponent>,
    /// Legacy components an upgraded document kept (JSON `deprecated`, the crate's own member; only
    /// allow-listed packs below 1.26.20 have them).
    pub deprecated: DeprecatedComponents,
    /// Component overrides that apply while their condition holds (JSON `permutations`).
    pub permutations: Vec<Permutation>,
    /// The legacy events of an allow-listed pack's document below 1.26.20 (JSON
    /// `deprecated_event_handlers`, the crate's own member; the game's name for them is `events`).
    pub deprecated_event_handlers: IndexMap<String, DeprecatedDefinitionEvent>,
    /// The order of the 1.21.60 `minecraft:custom_components` list the base components came from
    /// (JSON `ordered_v1_custom_components`).
    pub ordered_v1_custom_components: Vec<String>,
    /// The document's own `format_version` (the effective one when absent): what the version
    /// gates of validation and the client's runtime flags compare against. Not a JSON member;
    /// the writer writes it as `format_version` when it is 1.26.20 or newer.
    pub source_format_version: FormatVersion,
    /// Unknown members of `minecraft:block`.
    pub extra: Extra,
    /// Where the file has the members that a rule of 1.26.20 refuses of a block read from a
    /// file below it, where an upgrade step moved them: `(the member of the block, the
    /// member of the file)`. The write's refusal names the file's member, so the author sees where to fix it. Not a
    /// JSON member; empty for every block that is written.
    #[doc(hidden)]
    pub(crate) refused_at: Vec<(
        bedrock_addon_core::diagnostic::JsonPath,
        bedrock_addon_core::diagnostic::JsonPath,
    )>,
}

/// `minecraft:block/description`.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
pub struct Description {
    /// The block's identifier, `namespace:name`, never in the `minecraft` namespace outside the
    /// base game (JSON `identifier`).
    pub identifier: BlockId,
    /// Where the block appears in the creative menu (JSON `menu_category`).
    #[facet(default)]
    pub menu_category: Option<MenuCategory>,
    /// The block's own states and their values (JSON `states`).
    #[facet(default, skip_serializing_if = IndexMap::is_empty)]
    pub states: IndexMap<String, StateValues>,
    /// Traits, each of which adds states of its own (JSON `traits`).
    #[facet(default, skip_serializing_if = Traits::is_empty)]
    pub traits: Traits,
    /// Engine data of base-game blocks (JSON `internal_vanilla_data`; base-game packs only,
    /// an unknown member elsewhere).
    #[facet(default)]
    pub internal_vanilla_data: Option<VanillaBlockData>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `description.internal_vanilla_data`: what the engine needs of a base-game block.
#[derive(Clone, Debug, Default, PartialEq, facet::Facet)]
pub struct VanillaBlockData {
    /// The legacy numeric block id (JSON `block_id`).
    #[facet(default)]
    pub block_id: Option<StrictInt<i32>>,
    /// The block material (JSON `material`).
    #[facet(default)]
    pub material: Option<MaterialType>,
    /// The archetype, one entry keyed by its name, as written (JSON `block_archetype`).
    #[facet(default)]
    pub block_archetype: Option<IndexMap<String, bedrock_addon_core::json::Value>>,
    /// JSON `translucency`.
    #[facet(default)]
    pub translucency: Option<StrictFloat<f32>>,
    /// JSON `can_dampen_vibrations`.
    #[facet(default)]
    pub can_dampen_vibrations: Option<StrictBool>,
    /// JSON `can_occlude_vibrations`.
    #[facet(default)]
    pub can_occlude_vibrations: Option<StrictBool>,
    /// JSON `requires_correct_tool_for_drops` (engine 1.26.50).
    #[facet(default)]
    pub requires_correct_tool_for_drops: Option<StrictBool>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl VanillaBlockData {
    /// The legacy numeric block id: `block_id`, −1 when absent (the game's constructor default).
    #[must_use]
    pub fn block_id(&self) -> i32 {
        self.block_id.map_or(-1, |id| id.0)
    }

    /// The material: `material`, dirt when absent (the game's constructor default).
    #[must_use]
    pub fn material(&self) -> MaterialType {
        self.material.unwrap_or(MaterialType::Dirt)
    }

    /// `requires_correct_tool_for_drops`, `false` when absent, as the game's block loader
    /// defaults it.
    #[must_use]
    pub fn requires_correct_tool_for_drops(&self) -> bool {
        self.requires_correct_tool_for_drops.is_some_and(|b| b.0)
    }
}

/// One entry of `minecraft:block/permutations`: components that apply while `condition` holds.
#[derive(Clone, Debug, PartialEq)]
pub struct Permutation {
    /// A Molang expression over `query.block_state` (JSON `condition`).
    pub condition: Molang,
    /// The components it applies (JSON `components`).
    pub components: ComponentMap<BlockComponent>,
    /// Legacy components an upgraded permutation kept (JSON `deprecated`, the crate's own member).
    pub deprecated: DeprecatedComponents,
    /// The order of the permutation's 1.21.60 `custom_components` list (JSON
    /// `ordered_v1_custom_components`).
    pub ordered_v1_custom_components: Vec<String>,
    /// Unknown members.
    pub extra: Extra,
}

/// A block's material (ids 0..=25), `internal_vanilla_data.material`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, facet::Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
#[allow(missing_docs, reason = "each variant is its JSON spelling")]
pub enum MaterialType {
    Air,
    Dirt,
    Wood,
    Metal,
    Grate,
    Water,
    Lava,
    Leaves,
    Plant,
    SolidPlant,
    Fire,
    Glass,
    Explosive,
    Ice,
    PowderSnow,
    Cactus,
    Portal,
    StoneDecoration,
    Bubble,
    Barrier,
    DecorationSolid,
    ClientRequestPlaceholder,
    StructureVoid,
    Solid,
    NonSolid,
    Any,
}
