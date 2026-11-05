//! The family's types, their constructors and derived views.

use indexmap::IndexMap;

use super::component::{BlockComponent, DeprecatedComponents, DeprecatedDefinitionEvent};
use super::read::CEREAL;
use super::states::implied_states;
use super::{StateValues, Traits};
use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
use bedrock_addon_core::document::ComponentMap;
use bedrock_addon_core::id::BlockId;
use bedrock_addon_core::json::{StrictBool, StrictFloat, StrictInt, StrictString};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::molang::{Molang, MolangSource, MolangVersion};
use bedrock_addon_core::schema;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_shared_shapes::MenuCategory;

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
#[facet(schema::unknown = "keep")]
pub struct Description {
    /// The block's identifier, `namespace:name`, never in the `minecraft` namespace outside the
    /// base game (JSON `identifier`).
    pub identifier: BlockId,
    /// Where the block appears in the creative menu (JSON `menu_category`).
    pub menu_category: Option<MenuCategory>,
    /// The block's own states and their values (JSON `states`).
    #[facet(default, skip_serializing_if = IndexMap::is_empty)]
    pub states: IndexMap<String, StateValues>,
    /// Traits, each of which adds states of its own (JSON `traits`).
    #[facet(default, skip_serializing_if = Traits::is_empty)]
    pub traits: Traits,
    /// Engine data of base-game blocks (JSON `internal_vanilla_data`; base-game packs only,
    /// an unknown member elsewhere).
    pub internal_vanilla_data: Option<VanillaBlockData>,
}

/// `description.internal_vanilla_data`: what the engine needs of a base-game block.
#[derive(Clone, Debug, Default, PartialEq, facet::Facet)]
#[facet(schema::unknown = "keep")]
pub struct VanillaBlockData {
    /// The legacy numeric block id (JSON `block_id`).
    pub block_id: Option<StrictInt<i32>>,
    /// The block material (JSON `material`).
    pub material: Option<MaterialType>,
    /// The archetype, one entry keyed by its name, as written (JSON `block_archetype`).
    pub block_archetype: Option<IndexMap<String, bedrock_addon_core::json::Value>>,
    /// JSON `translucency`.
    pub translucency: Option<StrictFloat<f32>>,
    /// JSON `can_dampen_vibrations`.
    pub can_dampen_vibrations: Option<StrictBool>,
    /// JSON `can_occlude_vibrations`.
    pub can_occlude_vibrations: Option<StrictBool>,
    /// JSON `requires_correct_tool_for_drops` (engine 1.26.50).
    pub requires_correct_tool_for_drops: Option<StrictBool>,
}

impl VanillaBlockData {
    /// The legacy numeric block id: `block_id`, −1 when absent (the game's constructor default).
    pub fn block_id(&self) -> i32 {
        self.block_id.map_or(-1, |id| id.0)
    }

    /// The material: `material`, dirt when absent (the game's constructor default).
    pub fn material(&self) -> MaterialType {
        self.material.unwrap_or(MaterialType::Dirt)
    }

    /// `requires_correct_tool_for_drops`, `false` when absent, as the game's block loader
    /// defaults it.
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

impl Block {
    /// A block with this identifier, no states, no components, no permutations, at the newest
    /// format version.
    pub fn new(id: BlockId) -> Block {
        Block {
            description: Description::new(id),
            components: ComponentMap::new(),
            deprecated: DeprecatedComponents::default(),
            permutations: Vec::new(),
            deprecated_event_handlers: IndexMap::new(),
            ordered_v1_custom_components: Vec::new(),
            source_format_version: CEREAL,
            refused_at: Vec::new(),
        }
    }

    /// Runs the family's validation rules as if the block were loaded from a pack with `cx`:
    /// identifier and state rules, the permutation rules, component gates, the cross-component
    /// and trait rules. The block is not changed.
    pub fn validate(&self, cx: &LoadContext) -> Diagnostics {
        let mut block = self.clone();
        let mut diagnostics = Diagnostics::default();
        let at = JsonPath::new().child("minecraft:block");
        super::validate::validate_detached(&mut block, cx, &at, &mut diagnostics);
        diagnostics
    }

    /// The states, the block's own followed by the ones its traits imply, by name.
    pub fn states_with_traits(&self) -> IndexMap<String, StateValues> {
        let mut states = self.description.states.clone();
        for (name, values) in self.description.traits.implied_states() {
            states.entry(name.to_owned()).or_insert(values);
        }
        states
    }

    /// How many permutations of state values the block has: the product of every state's
    /// value count, the trait states included.
    pub fn permutation_count(&self) -> u64 {
        self.states_with_traits()
            .values()
            .map(|values| values.len() as u64)
            .product()
    }
}

impl Description {
    /// A description with this identifier and nothing else.
    pub fn new(identifier: BlockId) -> Description {
        Description {
            identifier,
            menu_category: None,
            states: IndexMap::new(),
            traits: Traits::default(),
            internal_vanilla_data: None,
        }
    }
}

impl Traits {
    /// The states the enabled traits add, in trait order.
    pub(crate) fn implied_states(&self) -> Vec<(&'static str, StateValues)> {
        let mut states = Vec::new();
        let mut add = |enabled: &[StrictString], parts: u8| {
            for name in enabled {
                for state in implied_states(&name.0, parts) {
                    if !states.iter().any(|(n, _)| *n == state.0) {
                        states.push(state);
                    }
                }
            }
        };
        if let Some(t) = &self.placement_direction {
            add(&t.enabled_states, 2);
        }
        if let Some(t) = &self.placement_position {
            add(&t.enabled_states, 2);
        }
        if let Some(t) = &self.connection {
            add(&t.enabled_states, 2);
        }
        if let Some(t) = &self.multi_block {
            add(&t.enabled_states, t.parts());
        }
        states
    }
}

impl Permutation {
    /// A permutation applied when `condition` is true, with no components yet.
    pub fn new(condition: Molang) -> Permutation {
        Permutation {
            condition,
            components: ComponentMap::new(),
            deprecated: DeprecatedComponents::default(),
            ordered_v1_custom_components: Vec::new(),
        }
    }

    /// [`Permutation::new`] from expression text, versioned as a block file of 1.26.20 or
    /// newer (Molang version 13).
    pub fn when(condition: &str) -> Permutation {
        Permutation::new(Molang::Expr(MolangSource::string(
            condition,
            MolangVersion::V13,
        )))
    }

    /// The permutation with `component` added (or replaced).
    #[must_use]
    pub fn with(mut self, component: impl Into<BlockComponent>) -> Permutation {
        self.components.insert_typed(component.into(), false);
        self
    }
}
