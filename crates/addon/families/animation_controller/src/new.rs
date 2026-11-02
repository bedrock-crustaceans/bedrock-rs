//! Constructors and the inherent helpers: vanilla's defaults and its case-folded views.

use indexmap::IndexMap;

use super::{AnimationController, AnimationControllerFile, StateVariable};
use bedrock_addon_core::molang::Molang;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_support::animation::walk::atof;

impl AnimationControllerFile {
    /// An empty file at format 1.10.0.
    pub fn new() -> Self {
        Self {
            format_version: FormatVersion::new(1, 10, 0),
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
