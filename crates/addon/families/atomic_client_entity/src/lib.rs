//! Atomic client entities (`atomic_client_entities/**/*.json`, payload
//! `minecraft:atomic_client_entity`): a client-only entity type that exists only as a beta
//! document.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod load;
mod read;

#[doc(hidden)]
pub use load::load_registry;

use bedrock_addon_core::id::{
    AnimationRef, AtomicClientEntityId, GeometryId, MaterialId, ParticleId, SoundEventName,
    TexturePath,
};
use bedrock_addon_core::json::{StrictBool, StrictFloat, StrictInt, StrictString};
use bedrock_addon_core::visual::{ControllerUse, HasVisuals, Visuals};
use facet::Facet;
use indexmap::IndexMap;

/// An atomic client entity: an entity that exists on clients only, with a model and optional
/// components (its box, and how a companion avoids and clips into blocks). It is no actor and has
/// no behaviour-pack counterpart, and its identifiers are a space of their own, apart from client
/// entities and attachables.
///
/// The format is beta-only: the one accepted `format_version` is `"beta"`, from any pack and
/// without the Beta APIs toggle. Every member not listed is an error; vanilla skips the file.
///
/// ```
/// use bedrock_addon_family_atomic_client_entity::{AabbDimensions, AtomicClientEntity, Components};
/// use bedrock_addon_core::id::AtomicClientEntityId;
///
/// let wisp = AtomicClientEntity {
///     components: Some(Components {
///         aabb_dimensions: Some(AabbDimensions::new(0.5, 0.5)),
///         ..Components::default()
///     }),
///     ..AtomicClientEntity::new(AtomicClientEntityId::parse("demo:wisp"))
/// };
/// assert_eq!(wisp.description.identifier.as_str(), "demo:wisp");
/// ```
#[doc(alias = "minecraft:atomic_client_entity")]
#[doc(alias = "atomic_client_entity")]
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct AtomicClientEntity {
    /// `description`: the identifier and what the client draws.
    pub description: AtomicDescription,
    /// `components`: the entity's components.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub components: Option<Components>,
}

/// `description` of an atomic client entity: the description of a client entity cut down to
/// the model. Names are kept as written; vanilla lower-cases nothing here.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct AtomicDescription {
    /// `identifier`: `namespace:name`; any namespace, `minecraft` included.
    pub identifier: AtomicClientEntityId,
    /// `materials`: slot name → material. `None`, here and in the members below, is a
    /// description without the member; an empty one the author wrote is kept.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub materials: Option<IndexMap<String, MaterialId>>,
    /// `textures`: slot name → texture path.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub textures: Option<IndexMap<String, TexturePath>>,
    /// `geometry`: slot name → geometry.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub geometry: Option<IndexMap<String, GeometryId>>,
    /// `animations`: short name → animation or animation controller.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub animations: Option<IndexMap<String, AnimationRef>>,
    /// `particle_effects`: short name → particle effect.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub particle_effects: Option<IndexMap<String, ParticleId>>,
    /// `sound_effects`: short name → sound event.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub sound_effects: Option<IndexMap<String, SoundEventName>>,
    /// `scripts`: the Molang the client runs and the animations it plays.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub scripts: Option<AtomicScripts>,
    /// `render_controllers`: the render controllers that draw the entity, in order. Names
    /// only: every entry is [`ControllerUse::Always`], and the `{ name: condition }` form is an
    /// error.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub render_controllers: Option<Vec<ControllerUse>>,
}

/// `scripts` of an atomic client entity: three lists of plain strings. The Molang statements
/// are kept as text; nothing compiles them when the document loads.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct AtomicScripts {
    /// `initialize`: Molang statements run once.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub initialize: Option<Vec<StrictString>>,
    /// `pre_animation`: Molang statements run before the animations.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub pre_animation: Option<Vec<StrictString>>,
    /// `animate`: the animations to play, by their `animations` short name; names only.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub animate: Option<Vec<StrictString>>,
}

/// `components` of an atomic client entity. The engine knows three components; any other key
/// is an error (the behaviour-pack entity's `minecraft:collision_box` among them).
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Components {
    /// `minecraft:aabb_dimensions`.
    #[facet(
        default,
        rename = "minecraft:aabb_dimensions",
        skip_serializing_if = Option::is_none
    )]
    pub aabb_dimensions: Option<AabbDimensions>,
    /// `minecraft:companion_block_avoidance`.
    #[facet(
        default,
        rename = "minecraft:companion_block_avoidance",
        skip_serializing_if = Option::is_none
    )]
    pub companion_block_avoidance: Option<CompanionBlockAvoidance>,
    /// `minecraft:companion_block_clipping`.
    #[facet(
        default,
        rename = "minecraft:companion_block_clipping",
        skip_serializing_if = Option::is_none
    )]
    pub companion_block_clipping: Option<CompanionBlockClipping>,
}

/// `minecraft:aabb_dimensions` of an atomic client entity ("AABB dimensions for Atomic Client
/// Entities."): a box `width` wide and deep and `height` high, standing on the entity's
/// position. Not the behaviour-pack entity's `minecraft:collision_box`. The client exposes the
/// box to Molang as `variable.aabb_dimension`, `variable.aabb_dimension_x` and
/// `variable.aabb_dimension_y`.
#[doc(alias = "minecraft:aabb_dimensions")]
#[derive(Clone, Copy, Debug, PartialEq, Facet)]
pub struct AabbDimensions {
    /// `width`: at least 0.
    pub width: StrictFloat<f32>,
    /// `height`: at least 0.
    pub height: StrictFloat<f32>,
}

/// An alias of [`AabbDimensions`], the box a `minecraft:aabb_dimensions` component holds.
pub type CollisionBox = AabbDimensions;

impl AabbDimensions {
    /// A box `width` wide and deep and `height` high.
    #[must_use]
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width: StrictFloat(width),
            height: StrictFloat(height),
        }
    }

    /// The box around the entity's position as vanilla builds it: `(min, max)` corners, half
    /// the width to each side and from 0 up to the height.
    #[must_use]
    pub fn bounds(&self) -> ([f32; 3], [f32; 3]) {
        // Half the width is the extent to each side; each corner is the component-wise minimum
        // and maximum of the two signed extents.
        let half = 0.5 * self.width.0;
        let (low, high) = (half.min(-half), half.max(-half));
        let (bottom, top) = (self.height.0.min(0.0), self.height.0.max(0.0));
        ([low, bottom, low], [high, top, high])
    }
}

/// `minecraft:companion_block_avoidance` ("Defines fallback anchor offsets used when the
/// primary companion follow target clips into blocks."). A member left out is `None` and the
/// client holds its default (the accessors); the write-back leaves it out.
#[doc(alias = "minecraft:companion_block_avoidance")]
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct CompanionBlockAvoidance {
    /// `backup_anchor_offsets`: the fallback offsets, each `[x, y, z]`; required.
    pub backup_anchor_offsets: Vec<[StrictFloat<f32>; 3]>,
    /// `require_hidden_before_fallback`: default false.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub require_hidden_before_fallback: Option<StrictBool>,
    /// `anchor_switch_cooldown`: an unsigned 16-bit count, default 200.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub anchor_switch_cooldown: Option<StrictInt<u16>>,
}

impl CompanionBlockAvoidance {
    /// `require_hidden_before_fallback` of a component that writes none.
    pub const DEFAULT_REQUIRE_HIDDEN_BEFORE_FALLBACK: bool = false;
    /// `anchor_switch_cooldown` of a component that writes none.
    pub const DEFAULT_ANCHOR_SWITCH_COOLDOWN: u16 = 200;

    /// A component with `offsets` and every other member left out.
    #[must_use]
    pub fn new(offsets: Vec<[f32; 3]>) -> Self {
        Self {
            backup_anchor_offsets: offsets
                .into_iter()
                .map(|offset| offset.map(StrictFloat))
                .collect(),
            require_hidden_before_fallback: None,
            anchor_switch_cooldown: None,
        }
    }

    /// The offsets as floats.
    #[must_use]
    pub fn to_offsets(&self) -> Vec<[f32; 3]> {
        self.backup_anchor_offsets
            .iter()
            .map(|offset| offset.map(|value| value.0))
            .collect()
    }

    /// `require_hidden_before_fallback` as the client holds it.
    #[must_use]
    pub fn require_hidden_before_fallback(&self) -> bool {
        self.require_hidden_before_fallback
            .map_or(Self::DEFAULT_REQUIRE_HIDDEN_BEFORE_FALLBACK, |value| {
                value.0
            })
    }

    /// `anchor_switch_cooldown` as the client holds it.
    #[must_use]
    pub fn anchor_switch_cooldown(&self) -> u16 {
        self.anchor_switch_cooldown
            .map_or(Self::DEFAULT_ANCHOR_SWITCH_COOLDOWN, |value| value.0)
    }
}

/// `minecraft:companion_block_clipping` ("Defines companion block clipping behavior,
/// transition effects, and hysteresis controls."). Every member may be left out; the
/// accessors give what the client holds. Each float is at least 0.
#[doc(alias = "minecraft:companion_block_clipping")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Facet)]
pub struct CompanionBlockClipping {
    /// `reveal_boundary`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub reveal_boundary: Option<VisibilityBoundary>,
    /// `hide_boundary`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub hide_boundary: Option<VisibilityBoundary>,
    /// `min_hidden_seconds`: default 2.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub min_hidden_seconds: Option<StrictFloat<f32>>,
    /// `min_visible_seconds`: default 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub min_visible_seconds: Option<StrictFloat<f32>>,
    /// `hide_delay_seconds`: default 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub hide_delay_seconds: Option<StrictFloat<f32>>,
    /// `hide_transition_seconds`: default 0.25.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub hide_transition_seconds: Option<StrictFloat<f32>>,
}

// The defaults the client holds for a component that writes none: the reveal and the hide
// boundary are each (0.5, 0.1, 1.0), then 2.0, 0.0, 0.0 and 0.25 seconds in declaration order.
impl CompanionBlockClipping {
    /// `min_hidden_seconds` of a component that writes none.
    pub const DEFAULT_MIN_HIDDEN_SECONDS: f32 = 2.0;
    /// `min_visible_seconds` of a component that writes none.
    pub const DEFAULT_MIN_VISIBLE_SECONDS: f32 = 0.0;
    /// `hide_delay_seconds` of a component that writes none.
    pub const DEFAULT_HIDE_DELAY_SECONDS: f32 = 0.0;
    /// `hide_transition_seconds` of a component that writes none.
    pub const DEFAULT_HIDE_TRANSITION_SECONDS: f32 = 0.25;

    /// `reveal_boundary` as the client holds it.
    #[must_use]
    pub fn reveal_boundary(&self) -> VisibilityBoundary {
        self.reveal_boundary.unwrap_or_default()
    }

    /// `hide_boundary` as the client holds it.
    #[must_use]
    pub fn hide_boundary(&self) -> VisibilityBoundary {
        self.hide_boundary.unwrap_or_default()
    }

    /// `min_hidden_seconds` as the client holds it.
    #[must_use]
    pub fn min_hidden_seconds(&self) -> f32 {
        self.min_hidden_seconds
            .map_or(Self::DEFAULT_MIN_HIDDEN_SECONDS, |value| value.0)
    }

    /// `min_visible_seconds` as the client holds it.
    #[must_use]
    pub fn min_visible_seconds(&self) -> f32 {
        self.min_visible_seconds
            .map_or(Self::DEFAULT_MIN_VISIBLE_SECONDS, |value| value.0)
    }

    /// `hide_delay_seconds` as the client holds it.
    #[must_use]
    pub fn hide_delay_seconds(&self) -> f32 {
        self.hide_delay_seconds
            .map_or(Self::DEFAULT_HIDE_DELAY_SECONDS, |value| value.0)
    }

    /// `hide_transition_seconds` as the client holds it.
    #[must_use]
    pub fn hide_transition_seconds(&self) -> f32 {
        self.hide_transition_seconds
            .map_or(Self::DEFAULT_HIDE_TRANSITION_SECONDS, |value| value.0)
    }
}

/// A companion visibility boundary (`minecraft:companion_visibility_boundary`): the
/// `reveal_boundary` and `hide_boundary` of [`CompanionBlockClipping`]. Every member may be
/// left out; each is at least 0.
#[derive(Clone, Copy, Debug, Default, PartialEq, Facet)]
pub struct VisibilityBoundary {
    /// `margin`: default 0.5.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub margin: Option<StrictFloat<f32>>,
    /// `lookahead_per_speed`: default 0.1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub lookahead_per_speed: Option<StrictFloat<f32>>,
    /// `lookahead_max`: default 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub lookahead_max: Option<StrictFloat<f32>>,
}

// The defaults the client holds for a boundary that writes none: a margin of 0.5, a lookahead
// per speed of 0.1 and a lookahead maximum of 1.0.
impl VisibilityBoundary {
    /// `margin` of a boundary that writes none.
    pub const DEFAULT_MARGIN: f32 = 0.5;
    /// `lookahead_per_speed` of a boundary that writes none.
    pub const DEFAULT_LOOKAHEAD_PER_SPEED: f32 = 0.1;
    /// `lookahead_max` of a boundary that writes none.
    pub const DEFAULT_LOOKAHEAD_MAX: f32 = 1.0;

    /// `margin` as the client holds it.
    #[must_use]
    pub fn margin(&self) -> f32 {
        self.margin.map_or(Self::DEFAULT_MARGIN, |value| value.0)
    }

    /// `lookahead_per_speed` as the client holds it.
    #[must_use]
    pub fn lookahead_per_speed(&self) -> f32 {
        self.lookahead_per_speed
            .map_or(Self::DEFAULT_LOOKAHEAD_PER_SPEED, |value| value.0)
    }

    /// `lookahead_max` as the client holds it.
    #[must_use]
    pub fn lookahead_max(&self) -> f32 {
        self.lookahead_max
            .map_or(Self::DEFAULT_LOOKAHEAD_MAX, |value| value.0)
    }
}

impl AtomicClientEntity {
    /// An atomic client entity for `identifier` with nothing to draw and no component.
    #[must_use]
    pub fn new(identifier: AtomicClientEntityId) -> Self {
        Self {
            description: AtomicDescription::new(identifier),
            components: None,
        }
    }

    /// `minecraft:aabb_dimensions`, when the entity has one.
    #[must_use]
    pub fn aabb_dimensions(&self) -> Option<&AabbDimensions> {
        self.components.as_ref()?.aabb_dimensions.as_ref()
    }

    /// `minecraft:companion_block_avoidance`, when the entity has one.
    #[must_use]
    pub fn companion_block_avoidance(&self) -> Option<&CompanionBlockAvoidance> {
        self.components.as_ref()?.companion_block_avoidance.as_ref()
    }

    /// `minecraft:companion_block_clipping`, when the entity has one.
    #[must_use]
    pub fn companion_block_clipping(&self) -> Option<&CompanionBlockClipping> {
        self.components.as_ref()?.companion_block_clipping.as_ref()
    }
}

impl AtomicDescription {
    /// A description of `identifier` with every other member absent.
    #[must_use]
    pub fn new(identifier: AtomicClientEntityId) -> Self {
        Self {
            identifier,
            materials: None,
            textures: None,
            geometry: None,
            animations: None,
            particle_effects: None,
            sound_effects: None,
            scripts: None,
            render_controllers: None,
        }
    }
}

impl HasVisuals for AtomicClientEntity {
    fn visuals(&self) -> Visuals<'_> {
        let d = &self.description;
        Visuals::new(
            d.materials.as_ref(),
            d.textures.as_ref(),
            d.geometry.as_ref(),
            d.animations.as_ref(),
            d.render_controllers.as_deref().unwrap_or_default(),
        )
    }
}
