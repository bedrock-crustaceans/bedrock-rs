//! Constructors of the entity types.

use bedrock_addon_core::document::{ComponentMap, Extra};
use bedrock_addon_core::id::EntityId;
use bedrock_addon_core::json::Value;
use indexmap::IndexMap;

use super::{Description, Entity, Event, Written};

impl Entity {
    /// An entity named `id` with an empty description, no components, groups or events.
    #[must_use]
    pub fn new(id: EntityId) -> Self {
        Self {
            description: Description::new(id),
            components: ComponentMap::new(),
            component_groups: IndexMap::new(),
            events: IndexMap::new(),
            brain: None,
            upgrades: IndexMap::new(),
            deprecated: None,
            extra: Extra::default(),
            written: Written {
                components: true,
                ..Written::default()
            },
        }
    }
}

impl Description {
    /// A description naming `id` and nothing else.
    #[must_use]
    pub fn new(id: EntityId) -> Self {
        Self {
            identifier: id,
            runtime_identifier: None,
            is_spawnable: None,
            is_summonable: None,
            properties: IndexMap::new(),
            animations: IndexMap::new(),
            scripts: None,
            spawn_category: None,
            property_memory_mapping: None,
            entity_version: None,
            extra: Extra::default(),
        }
    }
}

impl Event {
    /// The event `name` with the body `node` (`{ "add": { "component_groups": [...] } }`, …).
    pub fn new(name: impl Into<String>, node: Value) -> Self {
        Self {
            name: name.into(),
            node,
        }
    }
}
