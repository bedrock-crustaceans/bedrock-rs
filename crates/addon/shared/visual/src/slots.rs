//! [`VisualSlots`]: the members of the newest `description` client entities and attachables
//! share, as one field set each family flattens into its own `Description`.

use bedrock_addon_core::id::{
    AnimationRef, GeometryId, MaterialId, ParticleId, SoundEventName, TexturePath,
};
use facet::Facet;
use indexmap::IndexMap;

use crate::{AnimationKind, ControllerUse, HeldItemScale, Scripts, SpawnEgg, Visuals};

/// The members a client entity's and an attachable's `description` share (format version
/// 1.26.0 and up), between `min_engine_version` and the family's own members. A family
/// carries them as `#[facet(flatten)] pub slots: VisualSlots`, so they are read and written
/// at the description's own level, in this order.
///
/// Every member is kept as written. Vanilla lower-cases every slot name and the values of
/// `materials`, `geometry`, `animations` and `particle_effects` when it loads them.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct VisualSlots {
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
    pub queryable_geometry: Option<String>,
    /// `spawn_egg`: how the entity's spawn egg looks (accepted, and without meaning, for an
    /// attachable).
    pub spawn_egg: Option<SpawnEgg>,
    /// `scripts`: the Molang the client runs for the model and the animations it plays.
    pub scripts: Option<Scripts>,
    /// `particle_effects`: short name → particle effect.
    pub particle_effects: Option<IndexMap<String, ParticleId>>,
    /// `particle_emitters`: accepted and read by nothing.
    pub particle_emitters: Option<IndexMap<String, String>>,
    /// `animations`: short name → animation (`animation.…`) or animation controller
    /// (`controller.animation.…`). A value with neither prefix is ignored. `None` is a
    /// description without the member; an empty object the author wrote is kept.
    pub animations: Option<IndexMap<String, AnimationRef>>,
    /// `render_controllers`: the render controllers that draw the model, in order.
    pub render_controllers: Option<Vec<ControllerUse>>,
    /// `sound_effects`: short name → sound event. `None` is a description without the
    /// member; an empty object the author wrote is kept.
    pub sound_effects: Option<IndexMap<String, SoundEventName>>,
    /// `enable_attachables`: draw held and worn attachables; `false` when absent.
    pub enable_attachables: Option<bool>,
    /// `held_item_ignores_lighting`: draw held items fully lit; `false` when absent.
    pub held_item_ignores_lighting: Option<bool>,
    /// `held_item_scale`: the scale of held items; 1.0 when absent. A number at or below 0 is
    /// not used.
    pub held_item_scale: Option<HeldItemScale>,
    /// `hide_armor`: do not draw worn armour; `false` when absent.
    pub hide_armor: Option<bool>,
}

impl VisualSlots {
    /// `enable_attachables`, `false` when absent.
    pub fn enable_attachables(&self) -> bool {
        self.enable_attachables.unwrap_or(false)
    }

    /// `held_item_ignores_lighting`, `false` when absent.
    pub fn held_item_ignores_lighting(&self) -> bool {
        self.held_item_ignores_lighting.unwrap_or(false)
    }

    /// `hide_armor`, `false` when absent.
    pub fn hide_armor(&self) -> bool {
        self.hide_armor.unwrap_or(false)
    }

    /// The constant scale of held items: the written number when it is above 0, else 1.0
    /// (also for a Molang scale, which only a client evaluates).
    pub fn held_item_scale(&self) -> f32 {
        // The game stores the value only when it is above 0; anything else keeps the default 1.0.
        match self.held_item_scale {
            Some(HeldItemScale::Number(scale)) if scale > 0.0 => scale,
            _ => 1.0,
        }
    }

    /// What the `animations` entry `name` is, by its value's prefix in lower case; `None` for
    /// an unknown name and for a value vanilla ignores.
    pub fn animation_kind(&self, name: &str) -> Option<AnimationKind> {
        // The game tests the `controller.animation.` prefix first, then `animation.`; a value
        // with neither prefix is ignored without a message.
        let value = self.animations.as_ref()?.get(name)?.as_str();
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
        joined(self.scripts.as_ref()?.initialize.as_deref())
    }

    /// The text vanilla parses for `pre_animation`: the strings joined with no separator;
    /// `None` when the member is absent or the text is empty.
    pub fn pre_animation(&self) -> Option<String> {
        joined(self.scripts.as_ref()?.pre_animation.as_deref())
    }

    /// The slots a client draws from, borrowed.
    pub fn visuals(&self) -> Visuals<'_> {
        Visuals::new(
            Some(&self.materials),
            Some(&self.textures),
            Some(&self.geometry),
            self.animations.as_ref(),
            self.render_controllers.as_deref().unwrap_or_default(),
        )
    }

    /// Merges the `later` description's slots into these, as vanilla merges two documents
    /// with the same identifier and `min_engine_version`:
    /// - the maps `materials`, `textures`, `geometry`, `animations`, `sound_effects` and
    ///   `scripts.variables` key by key, a later key replacing an earlier one that differs only
    ///   in ASCII case (vanilla lower-cases keys); the merged definition has `animations` or
    ///   `sound_effects` when one of the two documents has the member;
    /// - `particle_effects`, `render_controllers`, `spawn_egg` (whole, so its absent members
    ///   fall back to their defaults) and `scripts.animate` replaced when the later document
    ///   has them;
    /// - the booleans and `queryable_geometry` replaced when the later document has them, so a
    ///   later `false` overrides an earlier `true`;
    /// - a named script replaced when the later document's member builds a script (a non-empty
    ///   expression; a number or a boolean does not), `scaleX` / `scalex` as one script;
    /// - `held_item_scale` replaced unless the later value is a number ≤ 0, which vanilla does
    ///   not store;
    /// - `particle_emitters`, which nothing reads, is not merged.
    ///
    /// `later` is consumed member by member. The caller removes scripts that fail to compile
    /// before merging: vanilla drops them.
    pub fn merge(&mut self, later: &mut VisualSlots) {
        crate::merge::merge_slots(self, later);
    }
}

/// The text the loader builds for `initialize` or `pre_animation`: the strings joined with no
/// separator. `None` when the member is absent or the text is empty.
#[doc(hidden)]
pub fn joined(parts: Option<&[String]>) -> Option<String> {
    // An array of strings is read as the elements concatenated with no separator.
    let text: String = parts?.iter().map(String::as_str).collect();
    (!text.is_empty()).then_some(text)
}
