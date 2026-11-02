//! Client entities (`ClientEntity`, `SpawnEgg`): what a client draws for an entity, from the
//! resource packs.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod history;
mod load;
mod new;
#[cfg(test)]
mod peek_tests;
mod read;
mod validate;
mod write;
#[doc(hidden)]
pub use load::load_registry;

use bedrock_addon_core::id::{
    AnimationRef, EntityId, GeometryId, MaterialId, ParticleId, SoundEventName, TexturePath,
};
use bedrock_addon_core::json::{StrictBool, StrictString, Value};
use bedrock_addon_core::version::{FormatVersion, SemVersion};
use bedrock_addon_core::visual::{
    ControllerUse, DescriptionSlots, HasVisuals, HeldItemScale, Scripts, SharedDescription,
    SpawnEgg, Visuals, description_min_engine_version,
};
// One enum serves client entities and attachables; core's `visual` module declares it.
pub use bedrock_addon_core::visual::AnimationKind;
use facet::Facet;
use indexmap::IndexMap;

/// A resource-pack client entity definition (`entity/**/*.json`, payload
/// `minecraft:client_entity`): what a client draws for an entity, in the newest shape (format
/// version 1.26.0 and up).
///
/// Everything is in [`description`](Self::description); the document has no components. Older
/// documents (1.8.0 and 1.10.0) are upgraded to this shape step by step ([`history`]).
///
/// Built from [`ClientEntity::new`] with struct-update syntax:
///
/// ```
/// use bedrock_addon_family_client_entity::{ClientEntity, Description};
/// use bedrock_addon_core::id::{EntityId, GeometryId};
///
/// let id = EntityId::parse("demo:ghost");
/// let mut description = Description::new(id.clone());
/// description
///     .geometry
///     .insert("default".to_owned(), GeometryId::parse("geometry.ghost"));
/// let entity = ClientEntity {
///     description,
///     ..ClientEntity::new(id)
/// };
/// assert_eq!(entity.identifier().as_str(), "demo:ghost");
/// ```
#[doc(alias = "minecraft:client_entity")]
#[doc(alias = "client_entity")]
#[derive(Clone, Debug, PartialEq)]
pub struct ClientEntity {
    /// `format_version`: the version the document is written with, 1.26.0 or newer. A document
    /// upgraded from an older version has 1.26.0.
    pub format_version: FormatVersion,
    /// `description`: the identifier and everything the client draws the entity with.
    pub description: Description,
}

/// `description` of a client entity (format version 1.26.0 and up).
///
/// Every member is kept as written. Vanilla lower-cases the identifier, every slot name and
/// the values of `materials`, `geometry`, `animations` and `particle_effects` when it loads
/// them; the accessors of [`ClientEntity`] give that reading. No member beyond these is
/// allowed: vanilla skips a file that has one.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Description {
    /// `identifier`: the entity this definition draws. Compared in lower case.
    pub identifier: EntityId,
    /// `min_engine_version`: a version string or `[major, minor, patch]`. It decides which
    /// definition of an identifier wins across packs, never which members are read; a text
    /// that is no version counts as 0.0.0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub min_engine_version: Option<Value>,
    /// `materials`: slot name → material.
    #[facet(default, skip_serializing_if = IndexMap::is_empty)]
    pub materials: IndexMap<String, MaterialId>,
    /// `textures`: slot name → texture path.
    #[facet(default, skip_serializing_if = IndexMap::is_empty)]
    pub textures: IndexMap<String, TexturePath>,
    /// `geometry`: slot name → geometry.
    #[facet(default, skip_serializing_if = IndexMap::is_empty)]
    pub geometry: IndexMap<String, GeometryId>,
    /// `queryable_geometry`: the geometry slot Molang's geometry queries read.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub queryable_geometry: Option<StrictString>,
    /// `spawn_egg`: how the entity's spawn egg looks.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub spawn_egg: Option<SpawnEgg>,
    /// `scripts`: the Molang the client runs for the model and the animations it plays.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub scripts: Option<Scripts>,
    /// `particle_effects`: short name → particle effect.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub particle_effects: Option<IndexMap<String, ParticleId>>,
    /// `particle_emitters`: accepted and read by nothing.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub particle_emitters: Option<IndexMap<String, StrictString>>,
    /// `animations`: short name → animation (`animation.…`) or animation controller
    /// (`controller.animation.…`). A value with neither prefix is ignored. `None` is a
    /// description without the member; an empty object the author wrote is kept.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub animations: Option<IndexMap<String, AnimationRef>>,
    /// `render_controllers`: the render controllers that draw the entity, in order.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub render_controllers: Option<Vec<ControllerUse>>,
    /// `sound_effects`: short name → sound event. `None` is a description without the
    /// member; an empty object the author wrote is kept.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub sound_effects: Option<IndexMap<String, SoundEventName>>,
    /// `enable_attachables`: draw held and worn attachables; `false` when absent.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub enable_attachables: Option<StrictBool>,
    /// `held_item_ignores_lighting`: draw held items fully lit; `false` when absent.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub held_item_ignores_lighting: Option<StrictBool>,
    /// `held_item_scale`: the scale of held items; 1.0 when absent. A number at or below 0 is
    /// not used.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub held_item_scale: Option<HeldItemScale>,
    /// `hide_armor`: do not draw worn armour; `false` when absent.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub hide_armor: Option<StrictBool>,
}

impl ClientEntity {
    /// The identifier as vanilla keys it: in ASCII lower case.
    pub fn identifier(&self) -> EntityId {
        EntityId::parse(self.description.identifier.as_str().to_ascii_lowercase())
    }

    /// The version this definition competes with across packs: `description.min_engine_version`
    /// read as a version, or 0.0.0 when it is absent or does not read.
    pub fn min_engine_version(&self) -> SemVersion {
        description_min_engine_version(
            self.format_version,
            self.description.min_engine_version.as_ref(),
        )
    }

    /// `enable_attachables`, `false` when absent.
    pub fn enable_attachables(&self) -> bool {
        self.description.enable_attachables.is_some_and(|b| b.0)
    }

    /// `held_item_ignores_lighting`, `false` when absent.
    pub fn held_item_ignores_lighting(&self) -> bool {
        self.description
            .held_item_ignores_lighting
            .is_some_and(|b| b.0)
    }

    /// `hide_armor`, `false` when absent.
    pub fn hide_armor(&self) -> bool {
        self.description.hide_armor.is_some_and(|b| b.0)
    }

    /// The constant scale of held items: the written number when it is above 0, else 1.0
    /// (also for a Molang scale, which only a client evaluates).
    pub fn held_item_scale(&self) -> f32 {
        // The game stores the value only when it is above 0; anything else keeps the default 1.0.
        match self.description.held_item_scale {
            Some(HeldItemScale::Number(scale)) if scale > 0.0 => scale,
            _ => 1.0,
        }
    }

    /// What the `animations` entry `name` is, by its value's prefix in lower case; `None` for
    /// an unknown name and for a value vanilla ignores.
    pub fn animation_kind(&self, name: &str) -> Option<AnimationKind> {
        // The game tests the `controller.animation.` prefix first, then `animation.`; a value
        // with neither prefix is ignored without a message.
        let value = self.description.animations.as_ref()?.get(name)?.as_str();
        let value = value.to_ascii_lowercase();
        if value.starts_with("controller.animation.") {
            Some(AnimationKind::Controller)
        } else if value.starts_with("animation.") {
            Some(AnimationKind::Animation)
        } else {
            None
        }
    }

    /// The text vanilla parses for `initialize`: the strings joined with no separator; `None`
    /// when the member is absent or the text is empty.
    pub fn initialize(&self) -> Option<String> {
        let scripts = self.description.scripts.as_ref()?;
        validate::joined(scripts.initialize.as_deref())
    }

    /// The text vanilla parses for `pre_animation`: the strings joined with no separator;
    /// `None` when the member is absent or the text is empty.
    pub fn pre_animation(&self) -> Option<String> {
        let scripts = self.description.scripts.as_ref()?;
        validate::joined(scripts.pre_animation.as_deref())
    }
}

impl HasVisuals for ClientEntity {
    fn visuals(&self) -> Visuals<'_> {
        let d = &self.description;
        Visuals::new(
            Some(&d.materials),
            Some(&d.textures),
            Some(&d.geometry),
            d.animations.as_ref(),
            d.render_controllers.as_deref().unwrap_or_default(),
        )
    }
}

impl SharedDescription for ClientEntity {
    fn slots(&mut self) -> DescriptionSlots<'_> {
        let d = &mut self.description;
        DescriptionSlots {
            materials: &mut d.materials,
            textures: &mut d.textures,
            geometry: &mut d.geometry,
            animations: &mut d.animations,
            sound_effects: &mut d.sound_effects,
            particle_effects: &mut d.particle_effects,
            queryable_geometry: &mut d.queryable_geometry,
            render_controllers: &mut d.render_controllers,
            spawn_egg: &mut d.spawn_egg,
            scripts: &mut d.scripts,
            enable_attachables: &mut d.enable_attachables,
            held_item_ignores_lighting: &mut d.held_item_ignores_lighting,
            held_item_scale: &mut d.held_item_scale,
            hide_armor: &mut d.hide_armor,
        }
    }
}
