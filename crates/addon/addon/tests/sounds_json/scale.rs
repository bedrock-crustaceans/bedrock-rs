//! The block `base` chains at scale: a table of 20 000 types that each name the next as their
//! base, and one whose bases close a cycle, resolve in time linear in the table.

use std::time::Duration;

use bedrock_addon::diagnostic::Severity;

use super::common::{BINDINGS, copies, found};

/// A `sounds.json` of `count` block sound types `b0` … each with `base` the next and one own
/// event; the last type's base is `end` (`""` for none).
fn chain(count: usize, end: &str) -> String {
    let mut text = String::from(r#"{"block_sounds": {"#);
    for index in 0..count {
        let base = if index + 1 < count {
            format!("b{}", index + 1)
        } else {
            end.to_owned()
        };
        let comma = if index == 0 { "" } else { "," };
        text.push_str(&format!(
            r#"{comma}"b{index}": {{"base": "{base}", "events": {{"hit": "hit.{index}"}}}}"#
        ));
    }
    text.push_str("}}");
    text
}

/// One load of `text` as the only copy of a stack: how long it took and the number of
/// findings named `code`.
fn load(text: &str, code: &str) -> (Duration, usize) {
    let stack = copies(BINDINGS, &[text]);
    let (sounds, took) = crate::cost::timed(|| stack.sounds_json());
    (took, found(sounds, code).len())
}

/// Requires the load of a chain of 20 000 types ending at `end` to take under eight times the
/// load of one of 5 000, by the crate's one protocol. The text is built before the
/// clock starts; `findings` says how many findings named `code` a chain of the size has.
#[track_caller]
fn linear(what: &str, end: &str, code: &str, findings: impl Fn(usize) -> usize) {
    crate::cost::assert_linear_timed_at(what, 5_000, |count| {
        let text = chain(count, end);
        let (took, found) = load(&text, code);
        assert_eq!(found, findings(count), "{what}: {count} types");
        took
    });
}

#[test]
fn a_base_chain_of_20_000_types_resolves_in_linear_time() {
    linear("block bases, a chain", "", "ReferenceCycle", |_| 0);
    let text = chain(20_000, "");
    let stack = copies(BINDINGS, &[&text]);
    let bindings = stack.sounds_json();
    assert_eq!(bindings.block_sounds.len(), 20_000);
    // Nothing is copied down the chain; the nearest definition answers.
    assert_eq!(bindings.block_sounds["b0"].sounds.events.len(), 1);
    assert_eq!(bindings.block_sound("b0", "hit").unwrap().sound, "hit.0");
    assert_eq!(stack.sounds_json().diagnostics().count(), 0);
}

#[test]
fn a_base_cycle_of_20_000_types_is_reported_for_each_in_linear_time() {
    // One finding per type of the cycle, as before.
    linear("block bases, a cycle", "b0", "ReferenceCycle", |count| {
        count
    });
    let text = chain(20_000, "b0");
    let stack = copies(BINDINGS, &[&text]);
    let sounds = stack.sounds_json();
    let first = found(sounds, "ReferenceCycle")[0];
    assert!(
        first
            .to_string()
            .contains("Block base sound 'b0' is recursively used")
    );
    assert_eq!(first.location.json_path.to_string(), "block_sounds/b0/base");
    assert!(first.severity >= Severity::Error);
    // A lookup in the cycle ends.
    assert!(sounds.block_sound("b0", "step").is_none());
}

#[test]
fn a_chain_into_a_missing_base_names_it_for_every_type() {
    let text = chain(2_000, "nope");
    let (_, findings) = load(&text, "UnresolvedReference");
    assert_eq!(findings, 2_000);
    let stack = copies(BINDINGS, &[&text]);
    let lines: Vec<String> = found(stack.sounds_json(), "UnresolvedReference")
        .iter()
        .map(|d| d.code.to_string())
        .collect();
    assert!(
        lines
            .iter()
            .all(|line| line == "Unknown block base sound 'nope'")
    );
}
