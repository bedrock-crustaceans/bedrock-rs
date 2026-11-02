//! Audit of the rows of 20 whose lookup is the `sounds_json` family's own, at load: R40 (an
//! event key of `sounds.json` that is no level sound event) and R41 (a block sound type's
//! `base`). The load tells them, with the game's line, and the link pass says nothing more of
//! them. Each test here holds one finding to its code, its severity, its place and its text together;
//! the rows of the family's own tests are `tests/sounds_json/bindings.rs::sj03_…`, `::sj06_…`.

use bedrock_addon::diagnostic::{Diagnostic, Phase, Severity};
use bedrock_addon::load::LoadOptions;

use crate::common::stacks;

/// What a finding says: code, severity, JSON path and text.
fn said(finding: &Diagnostic) -> (&'static str, Severity, String, String) {
    (
        finding.code.name(),
        finding.severity,
        finding.location.json_path.to_string(),
        finding.code.to_string(),
    )
}

/// The load findings of the sounds of a world whose resource pack has the `sounds.json`
/// `text`, and that the link pass adds none about that file.
fn loaded(text: &str) -> Vec<(&'static str, Severity, String, String)> {
    let set = stacks(&[], &[("sounds.json", text)], LoadOptions::default());
    let found: Vec<_> = set.resource.sounds_json().diagnostics().map(said).collect();
    for finding in set.resource.sounds_json().diagnostics() {
        assert_ne!(finding.phase, Phase::Link, "{finding}");
    }
    let linked = set.link();
    assert!(
        crate::common::in_file(&linked, "sounds.json").is_empty(),
        "{linked:?}"
    );
    found
}

/// 20 R40 (sounds rule 18): the entry is dropped with the game's line, the key lower-cased as
/// the game compares it. A kept error: the game logs it at level 3 and keeps the rest
/// (was a warning).
#[test]
fn r40_an_event_key_that_is_no_level_sound_event_is_the_loads_kept_error() {
    let found =
        loaded(r#"{"individual_event_sounds": {"events": {"NotAnEvent": "x", "Break": "y"}}}"#);
    assert_eq!(
        found,
        [(
            "UnknownSound",
            Severity::Error,
            "individual_event_sounds/events/NotAnEvent".to_owned(),
            "Event name 'notanevent' is not a valid LevelSoundEvent".to_owned()
        )]
    );
}

/// 20 R41 (sounds rule 20): a `base` that names no block sound type is an
/// `UnresolvedReference` kept error with the game's line, and one that comes back to itself a
/// `ReferenceCycle` kept error.
#[test]
fn r41_a_base_that_is_no_block_sound_type_is_the_loads_kept_error() {
    let found = loaded(r#"{"block_sounds": {"d": {"base": "nope"}, "self": {"base": "self"}}}"#);
    assert_eq!(
        found,
        [
            (
                "UnresolvedReference",
                Severity::Error,
                "block_sounds/d/base".to_owned(),
                "Unknown block base sound 'nope'".to_owned()
            ),
            (
                "ReferenceCycle",
                Severity::Error,
                "block_sounds/self/base".to_owned(),
                "Block base sound 'self' is recursively used".to_owned()
            ),
        ]
    );
}
