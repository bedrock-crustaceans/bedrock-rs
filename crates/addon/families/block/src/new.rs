//! Constructors and derived views of blocks.

use indexmap::IndexMap;

use super::component::{BlockComponent, DeprecatedComponents};
use super::parse::CEREAL;
use super::states::implied_states;
use super::{Block, Description, Permutation, StateValues, Traits};
use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
use bedrock_addon_core::document::{ComponentMap, Extra};
use bedrock_addon_core::id::BlockId;
use bedrock_addon_core::json::StrictString;
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::molang::{Molang, MolangSource, MolangVersion};

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
            extra: Extra::default(),
            refused_at: Vec::new(),
        }
    }

    /// Runs the family's validation rules as if the block were loaded from a pack with `cx`:
    /// identifier and state rules, the permutation rules, component gates, the cross-component
    /// and trait rules. The block is not changed.
    pub fn validate(&self, cx: &LoadContext) -> Diagnostics {
        let mut block = self.clone();
        let mut diagnostics = Diagnostics::default();
        let mut at = JsonPath::new();
        at.push("minecraft:block");
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
            extra: Extra::default(),
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
            extra: Extra::default(),
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
