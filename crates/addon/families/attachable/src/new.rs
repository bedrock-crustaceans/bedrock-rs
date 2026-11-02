//! Building an attachable in code.

use bedrock_addon_core::id::AttachableId;
use bedrock_addon_core::versioning::Versioned;
use indexmap::IndexMap;

use super::{Attachable, Description};

impl Attachable {
    /// An attachable for `identifier` with nothing to draw yet, at format version 1.26.0.
    pub fn new(identifier: AttachableId) -> Self {
        Self {
            format_version: Self::VERSION,
            description: Description::new(identifier),
            stack_bindings: Vec::new(),
        }
    }
}

impl Description {
    /// A description of `identifier` with every other member absent.
    pub fn new(identifier: AttachableId) -> Self {
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
            item: None,
        }
    }
}
