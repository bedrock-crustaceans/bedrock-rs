//! Particles: `ParticleEffect`.
//!
//! Particle effects: `particles/**/*.json` of a resource pack (`particle_effect`).
//! `ParticleEffect` is the newest document (format 1.20.80, which the game reads up to its own
//! version); a file below 1.20.80 is read by the legacy parser's rules and upgraded when it
//! loads, and `history` has that older struct.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![recursion_limit = "512"]

pub mod component;
pub mod history;
pub use curve::{BezierChainCurve, BezierChainNode, Curve, SimpleCurve, SimpleCurveType};
pub use event::{
    EventNode, ParticleEffectType, RandomEventNode, SoundEffectEvent, VisualEffectEvent,
};

mod curve;
mod event;
#[cfg(test)]
mod linear;
mod load;
mod parse;
mod read;
mod texts;
mod write;

use indexmap::IndexMap;

use bedrock_addon_core::document::ComponentMap;
use bedrock_addon_core::id::ParticleId;

use component::ParticleComponent;
use read::{Opt, object};

#[doc(hidden)]
pub use load::load_registry;

/// A particle effect (`particle_effect`), at the newest format version.
///
/// JSON: `{"format_version": "1.20.80", "particle_effect": {"description", "components",
/// "curves"?, "events"?}}`. The components say how the emitter lives, how many particles it
/// makes, where they appear, and how each particle moves and looks; curves and events are the
/// named parts the components and their Molang refer to.
///
/// Every Molang member is a number, a string (compiled at the Molang version of the pack's
/// `min_engine_version`) or `{"expression", "version"}` (compiled at its own version).
///
/// An effect read from a file below 1.20.80 may be a partial one: where the game's legacy
/// parser stops (no `basic_render_parameters`, no material, no texture, no `components`) the
/// game registers what it had read, and so does this type, with the parser's line as a
/// warning of the load. The value does not say that it is partial: the members never reached
/// are at their defaults (an empty `material` or `texture`, no component, no curve or
/// event), and [`to_json`](bedrock_addon_core::document::Content::to_json) writes the effect whole at
/// those defaults, as a 1.20.80 document that loads again without that warning.
#[derive(Clone, Debug, PartialEq)]
#[doc(alias = "particle_effect")]
pub struct ParticleEffect {
    /// What the effect is (JSON `description`).
    pub description: Description,
    /// The components (JSON `components`).
    pub components: ComponentMap<ParticleComponent>,
    /// Curves by the Molang variable each sets (JSON `curves`).
    pub curves: Option<IndexMap<String, Curve>>,
    /// Events by name (JSON `events`).
    pub events: Option<IndexMap<String, EventNode>>,
    /// Unknown members of `particle_effect`.
    pub extra: bedrock_addon_core::document::Extra,
}

impl ParticleEffect {
    /// An effect named `identifier` drawn with `material` and `texture`, without components.
    pub fn new(
        identifier: ParticleId,
        material: impl Into<String>,
        texture: impl Into<String>,
    ) -> Self {
        Self {
            description: Description {
                identifier,
                basic_render_parameters: BasicRenderParameters {
                    material: material.into(),
                    texture: texture.into(),
                    extra: bedrock_addon_core::document::Extra::default(),
                },
                extra: bedrock_addon_core::document::Extra::default(),
            },
            components: ComponentMap::new(),
            curves: None,
            events: None,
            extra: bedrock_addon_core::document::Extra::default(),
        }
    }
}

object! {
    /// `particle_effect/description`.
    pub struct Description {
        /// The effect's identifier, as other files name it (JSON `identifier`, required).
        req identifier: ParticleId = "identifier", Opt::NONE;
        /// How the particles are drawn (JSON `basic_render_parameters`, required).
        req basic_render_parameters: BasicRenderParameters = "basic_render_parameters",
            Opt::NONE;
    }
}

object! {
    /// `description/basic_render_parameters`: the material and the texture of every particle
    /// of the effect.
    pub struct BasicRenderParameters finish = parse::render_parameters {
        /// The material, a name of the pack stack's material files (JSON `material`,
        /// required).
        req material: String = "material", Opt::NONE;
        /// The texture, a path in the pack without its extension, at most 256 characters
        /// (JSON `texture`, required).
        req texture: String = "texture", Opt::NONE;
    }
}
