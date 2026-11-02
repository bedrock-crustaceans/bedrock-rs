//! Attachables (`Attachable`): `attachables/**/*.json` of a resource pack (`minecraft:attachable`),
//! what a client draws for an item an actor holds or wears.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod binding;
pub mod history;
mod load;
mod new;
#[cfg(test)]
mod peek_tests;
mod read;
mod validate;
mod write;
pub use binding::{Condition, ItemBinding, ItemBindingEntry, ItemBindings};
#[doc(hidden)]
pub use load::load_registry;

use bedrock_addon_core::id::{
    AnimationRef, AttachableId, GeometryId, ItemId, MaterialId, ParticleId, SoundEventName,
    TexturePath,
};
use bedrock_addon_core::json::{StrictBool, StrictString, Value};
use bedrock_addon_core::version::{FormatVersion, SemVersion};
use bedrock_addon_core::visual::{
    ControllerUse, DescriptionSlots, HasVisuals, HeldItemScale, Scripts, SharedDescription,
    SpawnEgg, Visuals, description_min_engine_version,
};
// One enum for both families, declared in core's `visual`.
pub use bedrock_addon_core::visual::AnimationKind;
use facet::Facet;
use indexmap::IndexMap;

/// A resource-pack attachable (`attachables/**/*.json`, payload `minecraft:attachable`): what
/// a client draws for an item an actor holds or wears, in the newest shape (format version
/// 1.26.0 and up).
///
/// Everything is in [`description`](Self::description); the document has no components. Older
/// documents (1.8.0 and 1.10.0) are upgraded to this shape step by step ([`history`]).
///
/// An attachable is drawn for an item in one of two ways: `description.item` names the item,
/// with or without a condition ([`Attachable::binding`]), or the attachable's identifier is
/// the item's.
///
/// Built from [`Attachable::new`] with struct-update syntax:
///
/// ```
/// use bedrock_addon_core::id::{AttachableId, GeometryId};
/// use bedrock_addon_family_attachable::{Attachable, Description};
///
/// let id = AttachableId::parse("demo:wand");
/// let mut description = Description::new(id.clone());
/// description
///     .geometry
///     .insert("default".to_owned(), GeometryId::parse("geometry.wand"));
/// let wand = Attachable {
///     description,
///     ..Attachable::new(id)
/// };
/// assert_eq!(wand.identifier().as_str(), "demo:wand");
/// ```
#[doc(alias = "minecraft:attachable")]
#[doc(alias = "attachable")]
#[derive(Clone, Debug, PartialEq)]
pub struct Attachable {
    /// `format_version`: the version the document is written with, 1.26.0 or newer. A document
    /// upgraded from an older version has 1.26.0.
    pub format_version: FormatVersion,
    /// `description`: the identifier and everything the client draws the attachable with.
    pub description: Description,
    /// The item bindings a stack holds for this definition, in the order vanilla gathered
    /// them: its own, and those of every document merged into it or replaced by it. Filled
    /// by a stack load only: a document read from one file or built in code has none here
    /// (its own binding is [`Attachable::binding`]). Not part of the JSON.
    pub stack_bindings: Vec<ItemBindingEntry>,
}

/// `description` of an attachable (format version 1.26.0 and up).
///
/// Every member is kept as written. Vanilla lower-cases the identifier, every slot name and
/// the values of `materials`, `geometry`, `animations` and `particle_effects` when it loads
/// them; the accessors of [`Attachable`] give that reading. The keys of `item` are not
/// lower-cased. No member beyond these is allowed: vanilla skips a file that has one.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Description {
    /// `identifier`: the attachable's name, and the item it is drawn for when no binding
    /// selects another attachable. Compared in lower case.
    pub identifier: AttachableId,
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
    /// `spawn_egg`: accepted, and without meaning for an attachable.
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
    /// (`controller.animation.…`). A value with neither prefix is ignored. `None` when the
    /// member is absent, which reads as no animation.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub animations: Option<IndexMap<String, AnimationRef>>,
    /// `render_controllers`: the render controllers that draw the attachable, in order.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub render_controllers: Option<Vec<ControllerUse>>,
    /// `sound_effects`: short name → sound event. `None` when the member is absent.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub sound_effects: Option<IndexMap<String, SoundEventName>>,
    /// `enable_attachables`: read as for a client entity (it describes an actor that holds
    /// items); `false` when absent.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub enable_attachables: Option<StrictBool>,
    /// `held_item_ignores_lighting`: read as for a client entity; `false` when absent.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub held_item_ignores_lighting: Option<StrictBool>,
    /// `held_item_scale`: read as for a client entity; 1.0 when absent. A number at or below
    /// 0 is not used.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub held_item_scale: Option<HeldItemScale>,
    /// `hide_armor`: read as for a client entity; `false` when absent.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub hide_armor: Option<StrictBool>,
    /// `item`: the item this attachable is drawn for, always or under a condition. Absent,
    /// the attachable is drawn for the item whose identifier is its own.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub item: Option<ItemBinding>,
}

impl Attachable {
    /// The identifier as vanilla keys it: in ASCII lower case.
    #[must_use]
    pub fn identifier(&self) -> AttachableId {
        AttachableId::parse(self.description.identifier.as_str().to_ascii_lowercase())
    }

    /// The version this definition competes with across packs: `description.min_engine_version`
    /// read as a version, or 0.0.0 when it is absent or does not read.
    #[must_use]
    pub fn min_engine_version(&self) -> SemVersion {
        description_min_engine_version(
            self.format_version,
            self.description.min_engine_version.as_ref(),
        )
    }

    /// The item this document's `description.item` binds the attachable to, and the condition
    /// it is drawn under; `None` without the member. The item is as written: vanilla compares
    /// it with the item's full identifier byte for byte. Of an object with several members
    /// vanilla reads one: the one whose key sorts first byte by byte, not the first written.
    pub fn binding(&self) -> Option<(ItemId, Condition)> {
        self.description.item.as_ref().and_then(ItemBinding::bound)
    }

    /// The item bindings the stack holds for this definition
    /// ([`stack_bindings`](Self::stack_bindings)), in the order vanilla gathered them; a
    /// client tries them from the last to the first. Empty outside a stack.
    #[must_use]
    pub fn bound_items(&self) -> &[ItemBindingEntry] {
        &self.stack_bindings
    }

    /// `enable_attachables`, `false` when absent.
    #[must_use]
    pub fn enable_attachables(&self) -> bool {
        self.description.enable_attachables.is_some_and(|b| b.0)
    }

    /// `held_item_ignores_lighting`, `false` when absent.
    #[must_use]
    pub fn held_item_ignores_lighting(&self) -> bool {
        self.description
            .held_item_ignores_lighting
            .is_some_and(|b| b.0)
    }

    /// `hide_armor`, `false` when absent.
    #[must_use]
    pub fn hide_armor(&self) -> bool {
        self.description.hide_armor.is_some_and(|b| b.0)
    }

    /// The constant scale of held items: the written number when it is above 0, else 1.0
    /// (also for a Molang scale, which only a client evaluates).
    #[must_use]
    pub fn held_item_scale(&self) -> f32 {
        // The game stores the value only when it is above 0.
        match self.description.held_item_scale {
            Some(HeldItemScale::Number(scale)) if scale > 0.0 => scale,
            _ => 1.0,
        }
    }

    /// What the `animations` entry `name` is, by its value's prefix in lower case; `None` for
    /// an unknown name and for a value vanilla ignores.
    #[must_use]
    pub fn animation_kind(&self, name: &str) -> Option<AnimationKind> {
        // The game checks for `controller.animation.` first, then `animation.`; anything
        // else is ignored silently.
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
    #[must_use]
    pub fn initialize(&self) -> Option<String> {
        let scripts = self.description.scripts.as_ref()?;
        validate::joined(scripts.initialize.as_deref())
    }

    /// The text vanilla parses for `pre_animation`: the strings joined with no separator;
    /// `None` when the member is absent or the text is empty.
    #[must_use]
    pub fn pre_animation(&self) -> Option<String> {
        let scripts = self.description.scripts.as_ref()?;
        validate::joined(scripts.pre_animation.as_deref())
    }
}

impl HasVisuals for Attachable {
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

impl SharedDescription for Attachable {
    /// The members the shared merge borrows. `animations` and `sound_effects` are borrowed
    /// as maps, so an absent one becomes an empty one here; the load's merge takes that back.
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

    /// `item`: the later document's, when it has one.
    fn merge_own(&mut self, later: &mut Self) {
        if let Some(item) = later.description.item.take() {
            self.description.item = Some(item);
        }
    }
}
