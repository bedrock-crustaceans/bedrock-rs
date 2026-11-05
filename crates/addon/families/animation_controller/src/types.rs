//! The family's types and their constructors: vanilla's defaults and its case-folded views.

use indexmap::IndexMap;

use bedrock_addon_core::molang::{Molang, MolangSource};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::known;
use bedrock_addon_support::animation::walk::atof;

use super::{EventString, ParticleEffect, SoundEffect};

/// An animation controller file (`animation_controllers/*.json`): `{"format_version",
/// "animation_controllers": {"controller.animation.<name>": {…}}}`, at the newest format
/// version (1.10.0).
///
/// A file from 1.8.0 up to 1.10.0 is upgraded when it loads (its blend curves become state
/// `variables`); [`history`](crate::history) has the 1.8 shape. Any undeclared member drops the whole file;
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

impl AnimationControllerFile {
    /// An empty file at format 1.10.0.
    pub fn new() -> Self {
        Self {
            format_version: known::V1_10_0,
            animation_controllers: IndexMap::new(),
        }
    }
}

impl Default for AnimationControllerFile {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimationController {
    /// A controller with no states.
    pub fn new() -> Self {
        Self::default()
    }

    /// The state vanilla starts in, by its name as written: `initial_state` when it names a
    /// state (compared as written with the lower-cased names), the state `default` when
    /// `initial_state` is absent, else the first state.
    pub fn initial_state_name(&self) -> Option<&str> {
        let find = |wanted: &str| {
            self.states
                .keys()
                .find(|name| name.to_ascii_lowercase() == wanted)
        };
        let chosen = match &self.initial_state {
            Some(initial) => find(initial),
            None => find("default"),
        };
        chosen
            .or_else(|| self.states.keys().next())
            .map(String::as_str)
    }
}

impl StateVariable {
    /// A variable that stores `input` unchanged.
    pub fn new(input: Molang) -> Self {
        Self {
            input,
            remap_curve: None,
        }
    }

    /// The remap curve's frames `(input, output)` in the order vanilla adds them: member order of
    /// its JSON reader, which is the key text sorted; vanilla never re-sorts them.
    pub fn frames(&self) -> Vec<(f32, f32)> {
        let Some(curve) = &self.remap_curve else {
            return Vec::new();
        };
        let mut keys: Vec<_> = curve.iter().collect();
        keys.sort_by(|a, b| a.0.cmp(b.0));
        keys.into_iter()
            .map(|(key, out)| (atof(key), *out))
            .collect()
    }
}

impl AnimationController {
    /// The text vanilla compiles for `transition`: lower-cased whole (string literals included)
    /// when [`lowercase_transitions`](Self::lowercase_transitions) is set, else as written.
    pub fn transition_expression(&self, transition: &super::Transition) -> String {
        let text = transition.condition.as_str();
        if self.lowercase_transitions {
            text.to_lowercase()
        } else {
            text.to_owned()
        }
    }
}

impl AnimationController {
    /// The state vanilla registers as `name`: state names are lower-cased, so the lookup ignores
    /// case.
    pub fn state(&self, name: &str) -> Option<&super::State> {
        self.states
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, state)| state)
    }
}

impl super::State {
    /// A state with no members set.
    pub fn new() -> Self {
        Self::default()
    }

    /// The Molang variables the state sets, as vanilla names them: `variable.<lower-cased
    /// name>`.
    pub fn variable_names(&self) -> Vec<String> {
        self.variables
            .iter()
            .flat_map(IndexMap::keys)
            .map(|name| format!("variable.{}", name.to_ascii_lowercase()))
            .collect()
    }
}

impl super::StateAnimation {
    /// The short name of a plain-string entry as vanilla stores it (lower-cased); `None` for a
    /// weighted entry.
    pub fn name(&self) -> Option<String> {
        match self {
            Self::Name(name) => Some(name.to_ascii_lowercase()),
            Self::Weighted(_) => None,
        }
    }
}

impl super::BlendTransition {
    /// The blend key frames `(seconds, weight)` vanilla builds, sorted by time: a number `t`
    /// gives `(0, 1), (t, 0)` (none for 0); a curve gives its keys read with `atof`, plus
    /// `(0, 1)` when no key is at time 0.
    pub fn frames(&self) -> Vec<(f32, f32)> {
        let mut frames = match self {
            Self::Seconds(t) if *t == 0.0 => Vec::new(),
            Self::Seconds(t) => vec![(0.0, 1.0), (*t, 0.0)],
            Self::Curve(curve) => {
                let mut frames: Vec<_> = curve
                    .iter()
                    .map(|(key, weight)| (atof(key), *weight))
                    .collect();
                if !frames.is_empty() && !frames.iter().any(|(t, _)| *t == 0.0) {
                    frames.push((0.0, 1.0));
                }
                frames
            }
        };
        frames.sort_by(|a, b| a.0.total_cmp(&b.0));
        frames
    }
}

#[cfg(test)]
mod tests {
    use super::super::State;

    #[test]
    fn a_new_state_has_no_members_set() {
        assert_eq!(State::new(), State::default());
        assert!(State::new().transitions.is_none());
    }
}
