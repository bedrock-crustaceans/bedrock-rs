//! What a client needs to draw a model: the description slots client entities, attachables and
//! atomic client entities share, their member types and the resolver for vanilla's shared
//! identifier map. Always built, so `entity::client` and `item::client` never name `render`
//! or each other.
//!
//! The member types ([`ControllerUse`], [`AnimateEntry`], [`Scripts`], [`SpawnEgg`]) are the
//! newest shapes; [`history`] has the older `scripts` shapes. The resolver
//! ([`EngineVersionMerge`]) reproduces how vanilla combines client entities and attachables of
//! one identifier across packs: by `description.min_engine_version`, not by pack priority
//! alone.

mod condition;
mod held_item_scale;
pub mod history;
mod merge;
mod scripts;
mod spawn_egg;

use indexmap::IndexMap;

use crate::id::{AnimationRef, GeometryId, MaterialId, TexturePath};

pub use condition::{AnimateEntry, ControllerUse};
pub use held_item_scale::HeldItemScale;
pub use merge::{
    DescriptionSlots, EngineVersionMerge, MergeStep, ShadowReason, SharedDescription,
    description_min_engine_version, merge_slots, merge_step,
};
pub use scripts::Scripts;
#[doc(hidden)]
pub use scripts::is_molang_place;
pub use spawn_egg::{Color, SpawnEgg, SpawnEggInfo};

/// What an `animations` entry of a client entity or an attachable names, as vanilla tells
/// them apart by the value's prefix. The two families hand it out under their own modules
/// (`entity::client`, `item::client`); it is declared here, beside [`Visuals::animations`],
/// so that it is one type for whoever compares the two.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum AnimationKind {
    /// `animation.…`: an animation.
    Animation,
    /// `controller.animation.…`: an animation controller.
    Controller,
}

/// What a client needs to draw a model: the slots shared by client entities, attachables
/// and atomic client entities. Borrowed from the document, so it costs nothing to build.
///
/// A map is `None` when the document has no such member; a member written as an empty object
/// is `Some` of an empty map. [`Visuals::material`] and its siblings look a slot up
/// either way.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct Visuals<'a> {
    /// `materials`: slot name → material id.
    pub materials: Option<&'a IndexMap<String, MaterialId>>,
    /// `textures`: slot name → texture path.
    pub textures: Option<&'a IndexMap<String, TexturePath>>,
    /// `geometry`: slot name → geometry id.
    pub geometry: Option<&'a IndexMap<String, GeometryId>>,
    /// `animations`: short name → animation or animation controller id.
    pub animations: Option<&'a IndexMap<String, AnimationRef>>,
    /// `render_controllers`, in order; empty when the document has no such member.
    pub render_controllers: &'a [ControllerUse],
}

impl<'a> Visuals<'a> {
    /// The view over a document's slots.
    #[must_use]
    pub fn new(
        materials: Option<&'a IndexMap<String, MaterialId>>,
        textures: Option<&'a IndexMap<String, TexturePath>>,
        geometry: Option<&'a IndexMap<String, GeometryId>>,
        animations: Option<&'a IndexMap<String, AnimationRef>>,
        render_controllers: &'a [ControllerUse],
    ) -> Self {
        Self {
            materials,
            textures,
            geometry,
            animations,
            render_controllers,
        }
    }

    /// The material of the slot `name`, as written.
    #[must_use]
    pub fn material(&self, name: &str) -> Option<&'a MaterialId> {
        self.materials?.get(name)
    }

    /// The texture of the slot `name`, as written.
    #[must_use]
    pub fn texture(&self, name: &str) -> Option<&'a TexturePath> {
        self.textures?.get(name)
    }

    /// The geometry of the slot `name`, as written.
    #[must_use]
    pub fn geometry(&self, name: &str) -> Option<&'a GeometryId> {
        self.geometry?.get(name)
    }

    /// The animation or animation controller of the short name `name`, as written.
    #[must_use]
    pub fn animation(&self, name: &str) -> Option<&'a AnimationRef> {
        self.animations?.get(name)
    }
}

/// Implemented by `ClientEntity`, `Attachable` and `AtomicClientEntity`.
pub trait HasVisuals {
    /// The description slots a client draws from.
    fn visuals(&self) -> Visuals<'_>;
}
