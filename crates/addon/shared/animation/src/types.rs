//! The values an animation and an animation controller state both hold: particle and sound
//! events and event strings.

use bedrock_addon_core::molang::{Molang, MolangSource};

/// A particle event (`particle_effects` of an animation or a controller state).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ParticleEffect {
    /// `effect`: the short name from the client entity's `particle_effects` map (vanilla
    /// lower-cases it); kept as written.
    pub effect: String,
    /// `locator`: the locator the effect plays at (vanilla lower-cases it); kept as written.
    pub locator: Option<String>,
    /// `pre_effect_script`: Molang run when the emitter is created; absent, none.
    pub pre_effect_script: Option<Molang>,
    /// `bind_to_actor`: absent the effect follows the actor; only `false` may be written.
    pub bind_to_actor: Option<bool>,
}

/// A sound event (`sound_effects` of an animation or a controller state).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SoundEffect {
    /// `effect`: the short name from the client entity's `sound_effects` map (vanilla
    /// lower-cases it); kept as written.
    pub effect: String,
    /// `locator`: where the sound plays (vanilla lower-cases it); kept as written.
    pub locator: Option<String>,
}

/// An event string of a `timeline` or a controller state's `on_entry` / `on_exit`, split the way
/// the game splits it.
#[derive(Clone, Debug, PartialEq)]
pub enum EventString {
    /// `/<command>`: a slash command, run with the pack's command version.
    Command {
        /// The command without its slash, case kept.
        command: String,
        /// The command version of the pack's `min_engine_version`.
        command_version: u8,
    },
    /// `@s <event>`: the entity's event; the name after the three bytes `@s `, unchanged.
    EntityEvent(String),
    /// Anything else not empty and not starting with `@`: a Molang expression.
    Molang(MolangSource),
    /// `""`, or `@…` other than `@s `: nothing happens; the text as written.
    Ignored(String),
}

impl ParticleEffect {
    /// A particle event playing `effect`, every other member absent.
    pub fn new(effect: impl Into<String>) -> Self {
        Self {
            effect: effect.into(),
            ..Self::default()
        }
    }

    /// `effect` as vanilla stores it: lower-cased.
    pub fn to_effect_name(&self) -> String {
        self.effect.to_ascii_lowercase()
    }

    /// `locator` as vanilla stores it: lower-cased, empty when absent.
    pub fn to_locator_name(&self) -> String {
        self.locator
            .as_deref()
            .unwrap_or_default()
            .to_ascii_lowercase()
    }

    /// `bind_to_actor`, with its default (bound).
    pub fn binds_to_actor(&self) -> bool {
        self.bind_to_actor.unwrap_or(true)
    }
}

impl SoundEffect {
    /// A sound event playing `effect`.
    pub fn new(effect: impl Into<String>) -> Self {
        Self {
            effect: effect.into(),
            locator: None,
        }
    }

    /// `effect` as vanilla stores it: lower-cased.
    pub fn to_effect_name(&self) -> String {
        self.effect.to_ascii_lowercase()
    }
}
