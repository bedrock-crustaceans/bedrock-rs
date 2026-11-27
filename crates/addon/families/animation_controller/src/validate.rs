//! The findings that are not schema failures: the controller reference stage the game logs
//! (unknown state names), and the crate's own lints, where the game says nothing.

use super::codes;
use super::read::V1_17_30;
use super::{AnimationControllerFile, State};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, JsonPath, Phase};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::version::SemVersion;
use bedrock_addon_shared_animation::validate::{command_version_unset, events};
use bedrock_addon_shared_animation::walk::atof;

/// Whether a transition text has a single-quoted literal with an upper-case letter.
fn has_cased_literal(text: &str) -> bool {
    text.split('\'')
        .skip(1)
        .step_by(2)
        .any(|literal| literal.chars().any(char::is_uppercase))
}

/// The findings on a loaded controller file: the reference stage, and the
/// crate's own `ImplicitInitialState`, `CaseFoldedTransition`, `UnorderedRemapCurve`,
/// `UnrecognizedEventString` and `MissingMinEngineVersion`.
pub(crate) fn controller_findings(
    file: &AnimationControllerFile,
    written: &SemVersion,
    cx: &LoadContext,
    diagnostics: &mut Diagnostics,
) {
    let mut any_event = false;
    for (key, controller) in &file.animation_controllers {
        let base = JsonPath::new().child("animation_controllers").child(key);
        let names: Vec<String> = controller
            .states
            .keys()
            .map(|n| n.to_ascii_lowercase())
            .collect();
        let not_found = |name: &str| codes::STATE_NOT_FOUND.with([name, "0"]);
        match &controller.initial_state {
            // Compared as written with the lower-cased names.
            Some(initial) if !names.iter().any(|n| n == initial) => {
                let at = base.child("initial_state");
                // The game substitutes state 0 and keeps the controller. Its "Error: state %s
                // not found - defaulting to using state %d" fires when an entity binds the
                // controller, not at pack load, so the finding is an information with the text.
                diagnostics.push(
                    Diagnostic::of(not_found(initial))
                        .at(at)
                        .phase(Phase::Validate),
                );
            }
            None if !names.iter().any(|n| n == "default") => {
                diagnostics.push(
                    Diagnostic::of(&core_codes::IMPLICIT_INITIAL_STATE)
                        .at(base.clone())
                        .phase(Phase::Validate),
                );
            }
            Some(_) | None => {}
        }
        for (name, state) in &controller.states {
            let at = base.child("states").child(name);
            any_event |= state_findings(state, &at, &names, written, diagnostics);
        }
    }
    command_version_unset(cx, any_event, diagnostics);
}

fn state_findings(
    state: &State,
    base: &JsonPath,
    names: &[String],
    written: &SemVersion,
    diagnostics: &mut Diagnostics,
) -> bool {
    for (index, transition) in state.transitions.iter().flatten().enumerate() {
        let at = base
            .child("transitions")
            .index(index)
            .child(&transition.target);
        let target = transition.target.to_ascii_lowercase();
        if !names.contains(&target) {
            diagnostics.push(
                Diagnostic::of(codes::STATE_NOT_FOUND.with([&target, "0"]))
                    .at(at.clone())
                    .phase(Phase::Validate),
            );
        }
        if *written < V1_17_30 && has_cased_literal(transition.condition.as_str()) {
            diagnostics.push(
                Diagnostic::of(&core_codes::CASE_FOLDED_TRANSITION)
                    .at(at)
                    .phase(Phase::Validate),
            );
        }
    }
    for (name, variable) in state.variables.iter().flatten() {
        let Some(curve) = &variable.remap_curve else {
            continue;
        };
        // Vanilla adds the frames in jsoncpp member order (key text) and never sorts them.
        let mut keys: Vec<&String> = curve.keys().collect();
        keys.sort();
        let ordered = keys.windows(2).all(|w| atof(w[0]) <= atof(w[1]));
        if !ordered {
            let at = base.child("variables").child(name).child("remap_curve");
            diagnostics.push(
                Diagnostic::of(&core_codes::UNORDERED_REMAP_CURVE)
                    .at(at)
                    .phase(Phase::Validate),
            );
        }
    }
    let mut list = Vec::new();
    for (member, entries) in [("on_entry", &state.on_entry), ("on_exit", &state.on_exit)] {
        for (index, e) in entries.iter().flatten().enumerate() {
            let at = base.child(member).index(index);
            list.push((at, e));
        }
    }
    events(list, diagnostics)
}
