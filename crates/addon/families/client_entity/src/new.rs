//! Building a client entity in code.

use bedrock_addon_core::id::EntityId;
use bedrock_addon_core::versioning::Versioned;
use indexmap::IndexMap;

use super::{ClientEntity, Description};

impl ClientEntity {
    /// A client entity for `identifier` with nothing to draw yet, at format version 1.26.0.
    #[must_use]
    pub fn new(identifier: EntityId) -> Self {
        Self {
            format_version: Self::VERSION,
            description: Description::new(identifier),
        }
    }
}

impl Description {
    /// A description of `identifier` with every other member absent.
    #[must_use]
    pub fn new(identifier: EntityId) -> Self {
        Self {
            identifier,
            min_engine_version: None,
            materials: IndexMap::new(),
            textures: IndexMap::new(),
            geometry: IndexMap::new(),
            queryable_geometry: None,
            spawn_egg: None,
            scripts: None,
            particle_effects: None,
            particle_emitters: None,
            animations: None,
            render_controllers: None,
            sound_effects: None,
            enable_attachables: None,
            held_item_ignores_lighting: None,
            held_item_scale: None,
            hide_armor: None,
        }
    }
}
