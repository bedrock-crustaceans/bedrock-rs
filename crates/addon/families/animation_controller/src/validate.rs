//! The findings that are not schema failures: the controller reference stage the game logs
//! (unknown state names), and the crate's own lints, where the game says nothing.

use super::parse::V1_17_30;
use super::{AnimationControllerFile, State};
use bedrock_addon_core::diagnostic::{DiagnosticCode, Diagnostics, JsonPath, Severity};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::version::SemVersion;
use bedrock_addon_support::animation::validate::{command_version_unset, events, push};
use bedrock_addon_support::animation::walk::{atof, text};

/// `Error: state %s not found - defaulting to using state %d`.
pub(crate) const STATE_NOT_FOUND: &str = "Error: state {} not found - defaulting to using state {}";

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
        let mut base = JsonPath::new();
        base.push("animation_controllers");
        base.push(key);
        let names: Vec<String> = controller
            .states
            .keys()
            .map(|n| n.to_ascii_lowercase())
            .collect();
        let not_found = |name: &str| DiagnosticCode::UnresolvedReference {
            text: text(STATE_NOT_FOUND, &[name, "0"]),
        };
        match &controller.initial_state {
            // Compared as written with the lower-cased names.
            Some(initial) if !names.iter().any(|n| n == initial) => {
                let mut at = base.clone();
                at.push("initial_state");
                // The game substitutes state 0 and keeps the controller. Its "Error: state %s
                // not found - defaulting to using state %d" fires when an entity binds the
                // controller, not at pack load, so the finding is an information with the text.
                push(diagnostics, not_found(initial), Severity::Info, at);
            }
            None if !names.iter().any(|n| n == "default") => {
                push(
                    diagnostics,
                    DiagnosticCode::ImplicitInitialState,
                    Severity::Info,
                    base.clone(),
                );
            }
            Some(_) | None => {}
        }
        for (name, state) in &controller.states {
            let mut at = base.clone();
            at.push("states");
            at.push(name);
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
        let mut at = base.clone();
        at.push("transitions");
        at.push_index(index);
        at.push(&transition.target);
        let target = transition.target.to_ascii_lowercase();
        if !names.contains(&target) {
            push(
                diagnostics,
                DiagnosticCode::UnresolvedReference {
                    text: text(STATE_NOT_FOUND, &[&target, "0"]),
                },
                Severity::Info,
                at.clone(),
            );
        }
        if *written < V1_17_30 && has_cased_literal(transition.condition.as_str()) {
            push(
                diagnostics,
                DiagnosticCode::CaseFoldedTransition,
                Severity::Info,
                at,
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
            let mut at = base.clone();
            at.push("variables");
            at.push(name);
            at.push("remap_curve");
            push(
                diagnostics,
                DiagnosticCode::UnorderedRemapCurve,
                Severity::Info,
                at,
            );
        }
    }
    let mut list = Vec::new();
    for (member, entries) in [("on_entry", &state.on_entry), ("on_exit", &state.on_exit)] {
        for (index, e) in entries.iter().flatten().enumerate() {
            let mut at = base.clone();
            at.push(member);
            at.push_index(index);
            list.push((at, e));
        }
    }
    events(list, diagnostics)
}
