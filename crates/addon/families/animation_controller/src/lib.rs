//! Animation controllers: `AnimationController`, `AnimationControllerFile`, `State`.
//!
//! Animation controllers (`animation_controllers/*.json`) of both pack kinds. Vanilla reads the
//! behaviour-pack and the resource-pack files with the same loader and the same schema, so one
//! type serves both stacks; what differs is runtime: the registry each side keeps, the Molang
//! query catalogue the compile check uses, and a few diagnostics of the crate's own.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod history;

mod load;
mod molang;
mod new;
mod parse;
mod validate;
mod write;

use indexmap::IndexMap;

use bedrock_addon_core::molang::{Molang, MolangSource};
use bedrock_addon_core::version::FormatVersion;

pub use bedrock_addon_support::animation::{EventString, ParticleEffect, SoundEffect};
#[doc(hidden)]
pub use load::load_controllers;

/// An animation controller file (`animation_controllers/*.json`): `{"format_version",
/// "animation_controllers": {"controller.animation.<name>": {…}}}`, at the newest format
/// version (1.10.0).
///
/// A file from 1.8.0 up to 1.10.0 is upgraded when it loads (its blend curves become state
/// `variables`); [`history`] has the 1.8 shape. Any undeclared member drops the whole file;
/// member names match ignoring case.
#[derive(Clone, Debug, PartialEq)]
#[doc(alias = "animation_controllers")]
pub struct AnimationControllerFile {
    /// `format_version`: as written; `1.10.0` after an upgrade.
    pub format_version: FormatVersion,
    /// `animation_controllers`: each controller by its key as written
    /// (`controller.animation.<name>`); the registry keys it lower-cased.
    pub animation_controllers: IndexMap<String, AnimationController>,
}

/// One animation controller: a state machine over animations.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AnimationController {
    /// `initial_state`: the state to start in, compared as written with the lower-cased state
    /// names; absent, the state `default`, else the first state.
    pub initial_state: Option<String>,
    /// `states`: each state by its name as written (vanilla lower-cases it).
    pub states: IndexMap<String, State>,
    /// Not JSON: the file's `format_version` was below 1.17.30, so vanilla lower-cases every
    /// transition expression (string literals included) before compiling it.
    pub lowercase_transitions: bool,
    /// Not JSON: the file's `format_version` was below 1.18.10, so the initial state's
    /// `on_entry` events do not fire when the controller starts.
    pub legacy_event_skipping: bool,
}

/// One state of an [`AnimationController`].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct State {
    /// `variables`: state variables by name (vanilla stores `variable.<lower-cased name>`).
    pub variables: Option<IndexMap<String, StateVariable>>,
    /// `animations`: the animations the state plays, by short name, with blend weights.
    pub animations: Option<Vec<StateAnimation>>,
    /// `transitions`: the states to move to and their conditions, in order.
    pub transitions: Option<Vec<Transition>>,
    /// `blend_transition`: the cross-fade when leaving the state.
    pub blend_transition: Option<BlendTransition>,
    /// `blend_via_shortest_path`: blend each Euler axis the short way round; only `true` may be
    /// written.
    pub blend_via_shortest_path: Option<bool>,
    /// `particle_effects`: particles played on entry.
    pub particle_effects: Option<Vec<ParticleEffect>>,
    /// `sound_effects`: sounds played on entry.
    pub sound_effects: Option<Vec<SoundEffect>>,
    /// `on_entry`: events fired on entering the state.
    pub on_entry: Option<Vec<EventString>>,
    /// `on_exit`: events fired on leaving the state.
    pub on_exit: Option<Vec<EventString>>,
}

/// A state variable (`variables.<name>`): `input` remapped through `remap_curve`.
#[derive(Clone, Debug, PartialEq)]
pub struct StateVariable {
    /// `input`: the value remapped.
    pub input: Molang,
    /// `remap_curve`: input value (key) to output value; absent, the input is stored as is.
    /// Vanilla evaluates the frames in its own member order (sorted key text), see
    /// [`StateVariable::frames`].
    pub remap_curve: Option<IndexMap<String, f32>>,
}

/// One entry of a state's `animations` array.
#[derive(Clone, Debug, PartialEq)]
pub enum StateAnimation {
    /// A short name, played at weight 1 (vanilla lower-cases it).
    Name(String),
    /// Short names with their blend weights (`{"walk": "query.modified_move_speed"}`).
    Weighted(IndexMap<String, Molang>),
}

/// One `transitions` entry: `{"<state>": "<condition>"}`.
#[derive(Clone, Debug, PartialEq)]
pub struct Transition {
    /// The target state, as written (vanilla lower-cases it).
    pub target: String,
    /// The condition, as written (see [`AnimationController::lowercase_transitions`]).
    pub condition: MolangSource,
}

/// A state's `blend_transition`.
#[derive(Clone, Debug, PartialEq)]
pub enum BlendTransition {
    /// A number: fade out over this many seconds (0: no blend).
    Seconds(f32),
    /// Seconds since the transition (key) to the old state's weight.
    Curve(IndexMap<String, f32>),
}
