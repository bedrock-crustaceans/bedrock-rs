//! The three version steps of `sounds/sound_definitions.json`: the game's JSON fixer, which
//! runs on every pack's copy before the merge and once more on the merged root.

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Phase, VanillaText,
};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::version::{FormatVersion, SemVersion, known};
use bedrock_addon_support::sound::fork::{self, Stored};
use facet_value::VObject;

/// The first version with the `{ format_version, sound_definitions }` envelope.
pub(crate) const V1_14_0: FormatVersion = FormatVersion::new(1, 14, 0);
/// The first version that reads integer distances.
pub(crate) const V1_20_20: FormatVersion = FormatVersion::new(1, 20, 20);
/// The first version whose streamed entries may stream concurrently by default.
pub(crate) const V1_26_50: FormatVersion = known::V1_26_50;
/// The version of a file whose `format_version` is missing or unreadable.
pub(crate) const DEFAULT_VERSION: FormatVersion = FormatVersion::new(1, 13, 0);

/// Vanilla's text for a `sound_definitions` member that is no object.
pub(crate) const NOT_A_LIST: &str = "sound definition field isn't a list of sound definitions!";
/// Vanilla's text (sic, no space) for a definition that is no object.
pub(crate) const MALFORMED_SOUND: &str = "sound definitions has sound '{}'that is malformed";

/// The version the gates of a copy compare against: its `format_version` as a version string
/// or an array of three integers, and 1.13.0 when the member is missing or anything else.
pub(crate) fn gate_version(root: &Value) -> SemVersion {
    version_member(root).unwrap_or_else(|| SemVersion::from(DEFAULT_VERSION))
}

/// The root's `format_version`, when it reads as a version.
pub(crate) fn version_member(root: &Value) -> Option<SemVersion> {
    if !fork::has_member(root, "format_version") {
        return None;
    }
    SemVersion::from_json(fork::member(root, "format_version"))
}

/// Where an entry of a definition's `sounds` is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum EntryPlace {
    /// An element of the list.
    Index(usize),
    /// A member of an object (the fork walks an object by its member values).
    Key(String),
}

/// What one run of the steps did to a copy, for the caller's own findings.
#[derive(Clone, Debug, Default, PartialEq)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "four independent facts about one run"
)]
pub(crate) struct Upgraded {
    /// The 1.14.0 step ran.
    pub(crate) wrapped: bool,
    /// The 1.20.20 step ran.
    pub(crate) distances: bool,
    /// The root had a `sound_definitions` member before the 1.14.0 step wrapped it.
    pub(crate) wrapper_lost: bool,
    /// Definitions of a legacy copy that were no object and became one.
    pub(crate) replaced: Vec<String>,
    /// Definitions of a legacy copy whose own `__use_legacy_max_distance` was another value
    /// than the `"true"` the 1.14.0 step wrote over it.
    pub(crate) flag_replaced: Vec<String>,
    /// Definitions of a legacy copy that had no `sounds` and got `sounds: null`.
    pub(crate) sounds_inserted: Vec<String>,
    /// Integer distances the 1.20.20 step zeroed: the definition and the member.
    pub(crate) zeroed: Vec<(String, &'static str)>,
    /// The entries of `sounds` whose `pitch` the 1.14.0 step removed: the definition and
    /// the entry's place in `sounds` (an index of the list, or a key of an object).
    pub(crate) pitch_dropped: Vec<(String, EntryPlace)>,
    /// The 1.20.20 or the 1.26.50 step found no object under `sound_definitions` (each reads
    /// it with the non-const `operator[]`, so an absent one is now `null`).
    pub(crate) not_a_list: bool,
    /// The 1.26.50 step ran.
    pub(crate) streaming: bool,
}

/// The path of the definition `name`.
pub(crate) fn definition_path(name: &str) -> JsonPath {
    JsonPath::new().child("sound_definitions").child(name)
}

/// The 1.14.0 step on an object root: the root becomes `sound_definitions` of a new root
/// stamped `"1.14.0"`, every definition gets `__use_legacy_max_distance: "true"` (a definition
/// that is no object becomes one first) and a `sounds` member (`null` when it had none), and
/// every object entry of `sounds` loses its `pitch`.
pub(crate) fn wrap_legacy_root(root: &mut Value, report: &mut Upgraded) {
    report.wrapped = true;
    report.wrapper_lost = fork::has_member(root, "sound_definitions");
    let mut definitions = root.take();
    for name in fork::member_names(&definitions) {
        let definition = fork::index_mut(&mut definitions, &name);
        if !definition.is_object() {
            report.replaced.push(name.clone());
        }
        let flag = fork::index_mut(definition, "__use_legacy_max_distance");
        if !flag.is_null() && flag.as_string().is_none_or(|flag| flag.as_str() != "true") {
            report.flag_replaced.push(name.clone());
        }
        *flag = Value::from("true");
        if !fork::has_member(definition, "sounds") {
            report.sounds_inserted.push(name.clone());
        }
        let sounds = fork::index_mut(definition, "sounds");
        if let Some(entries) = sounds.as_array_mut() {
            for (index, entry) in entries.as_mut_slice().iter_mut().enumerate() {
                if let Some(entry) = entry.as_object_mut()
                    && entry.remove("pitch").is_some()
                {
                    report
                        .pitch_dropped
                        .push((name.clone(), EntryPlace::Index(index)));
                }
            }
        } else if let Some(entries) = sounds.as_object_mut() {
            // The fork walks an object by its member values.
            for (key, entry) in entries.iter_mut() {
                if let Some(entry) = entry.as_object_mut()
                    && entry.remove("pitch").is_some()
                {
                    report
                        .pitch_dropped
                        .push((name.clone(), EntryPlace::Key(key.as_str().to_owned())));
                }
            }
        }
    }
    let mut wrapped = VObject::new();
    wrapped.insert("format_version", Value::from("1.14.0"));
    wrapped.insert("sound_definitions", definitions);
    *root = Value::from(wrapped);
}

/// The 1.20.20 step on an object root: the root is stamped `"1.20.20"`; `sound_definitions`
/// must be an object, and each definition that is one gets its integer `max_distance` and
/// `min_distance` replaced by `0.0` and the absent ones inserted as `null`.
pub(crate) fn reset_integer_distances(
    root: &mut Value,
    report: &mut Upgraded,
    diagnostics: &mut Diagnostics,
) {
    report.distances = true;
    *fork::index_mut(root, "format_version") = Value::from("1.20.20");
    let definitions = fork::index_mut(root, "sound_definitions");
    if !definitions.is_object() {
        report.not_a_list = true;
        let path = JsonPath::new().child("sound_definitions");
        // The game writes an error line to the content log; the file is still the stack's copy
        // and loads: a kept error.
        diagnostics.push_kept_error(
            Diagnostic::error(DiagnosticCode::InvalidValue {
                text: Some(VanillaText::Fixed(NOT_A_LIST)),
            })
            .at(path)
            .phase(Phase::Upgrade),
        );
        return;
    }
    for name in fork::member_names(definitions) {
        let definition = fork::index_mut(definitions, &name);
        if !definition.is_object() {
            // The game writes an error line to the content log and the step goes on: a kept
            // error.
            diagnostics.push_kept_error(
                Diagnostic::error(DiagnosticCode::InvalidValue {
                    text: Some(VanillaText::Formatted {
                        template: MALFORMED_SOUND,
                        args: vec![name.clone()],
                    }),
                })
                .at(definition_path(&name))
                .phase(Phase::Upgrade),
            );
            continue;
        }
        for member in ["max_distance", "min_distance"] {
            let distance = fork::index_mut(definition, member);
            if fork::stored(distance) == Stored::Integer {
                *distance = Value::from(0.0_f64);
                report.zeroed.push((name.clone(), member));
            }
        }
    }
}

/// The 1.26.50 step on an object root: the root is stamped `"1.26.50"`, and every object entry
/// of a definition's `sounds` list that streams (`stream` read with `asBool(false)`) and has
/// no `allow_concurrent_streaming` gets `allow_concurrent_streaming: false`, so that an entry
/// written before the member existed keeps streaming alone. A root without
/// `sound_definitions` gets it as `null`. Silent; nothing is dropped.
// `sound_definitions` is read without a membership test, so an absent member is inserted as
// `null`; it is walked by its members (an object), its elements (a list), or nothing (anything
// else). A definition counts when it is an object whose `sounds` is a list; an entry counts
// when it is an object with `stream`. Nothing below the section is created.
pub(crate) fn disallow_concurrent_streaming(root: &mut Value, report: &mut Upgraded) {
    report.streaming = true;
    *fork::index_mut(root, "format_version") = Value::from("1.26.50");
    let definitions = fork::index_mut(root, "sound_definitions");
    if !definitions.is_object() {
        report.not_a_list = true;
    }
    let definitions: Vec<&mut Value> = if let Some(object) = definitions.as_object_mut() {
        object
            .iter_mut()
            .map(|(_, definition)| definition)
            .collect()
    } else if let Some(array) = definitions.as_array_mut() {
        array.as_mut_slice().iter_mut().collect()
    } else {
        Vec::new()
    };
    for definition in definitions {
        let Some(sounds) = definition
            .as_object_mut()
            .and_then(|definition| definition.get_mut("sounds"))
            .and_then(Value::as_array_mut)
        else {
            continue;
        };
        for entry in sounds.as_mut_slice().iter_mut() {
            let Some(entry) = entry.as_object_mut() else {
                continue;
            };
            let streams = entry
                .get("stream")
                .is_some_and(|stream| fork::as_bool(stream, false));
            if streams && !entry.contains_key("allow_concurrent_streaming") {
                entry.insert("allow_concurrent_streaming", Value::from(false));
            }
        }
    }
}

/// Runs the three steps: every gate compares the copy's original version, so a legacy
/// copy runs every step in one call. A root that is `null` or no object is left alone.
pub(crate) fn upgrade(root: &mut Value, diagnostics: &mut Diagnostics) -> Upgraded {
    let mut report = Upgraded::default();
    if !root.is_object() {
        return report;
    }
    let version = gate_version(root);
    if version < SemVersion::from(V1_14_0) {
        wrap_legacy_root(root, &mut report);
    }
    if version < SemVersion::from(V1_20_20) {
        reset_integer_distances(root, &mut report, diagnostics);
    }
    if version < SemVersion::from(V1_26_50) {
        disallow_concurrent_streaming(root, &mut report);
    }
    report
}

#[cfg(test)]
#[allow(clippy::float_cmp, reason = "the coercions give exact values")]
mod tests {
    use super::*;

    fn json(text: &str) -> Value {
        bedrock_addon_core::json::from_str::<Value>(text).expect("valid JSON")
    }

    fn upgraded(text: &str) -> (Value, Upgraded, Diagnostics) {
        let mut root = json(text);
        let mut diagnostics = Diagnostics::default();
        let report = upgrade(&mut root, &mut diagnostics);
        (root, report, diagnostics)
    }

    #[test]
    fn the_gate_version_reads_a_string_or_three_integers_and_defaults_to_1_13_0() {
        let v = |text: &str| gate_version(&json(text));
        assert_eq!(
            v(r#"{"format_version": "1.20.20"}"#),
            SemVersion::new(1, 20, 20)
        );
        assert_eq!(
            v(r#"{"format_version": [1, 20, 20]}"#),
            SemVersion::new(1, 20, 20)
        );
        for other in [
            "{}",
            r#"{"format_version": [1, 20]}"#,
            r#"{"format_version": "banana"}"#,
            r#"{"format_version": 1.2}"#,
            r#"{"format_version": true}"#,
            r#"{"format_version": {"a": 1}}"#,
        ] {
            assert_eq!(v(other), SemVersion::new(1, 13, 0), "{other}");
        }
    }

    #[test]
    fn a_prerelease_of_1_20_20_still_runs_the_distance_step() {
        let (root, report, _) = upgraded(
            r#"{"format_version": "1.20.20-beta.1", "sound_definitions": {"x": {"max_distance": 4}}}"#,
        );
        assert!(report.distances && !report.wrapped);
        assert_eq!(
            fork::stored(fork::member(
                fork::member(fork::member(&root, "sound_definitions"), "x"),
                "max_distance"
            )),
            Stored::Real
        );
    }

    #[test]
    fn a_version_at_each_gate_skips_that_step() {
        let (_, report, _) = upgraded(r#"{"format_version": "1.14.0", "sound_definitions": {}}"#);
        assert!(!report.wrapped && report.distances);
        let (_, report, _) = upgraded(r#"{"format_version": "1.13.99", "a": {}}"#);
        assert!(report.wrapped && report.distances);
        let (root, report, _) = upgraded(
            r#"{"format_version": "1.20.20", "sound_definitions": {"x": {"max_distance": 4}}}"#,
        );
        assert!(!report.wrapped && !report.distances && report.streaming);
        assert_eq!(
            root,
            json(
                r#"{"format_version": "1.26.50", "sound_definitions": {"x": {"max_distance": 4}}}"#
            )
        );
        let (_, report, _) = upgraded(r#"{"format_version": "1.20.19", "sound_definitions": {}}"#);
        assert!(report.distances && report.streaming);
        let text = r#"{"format_version": "1.26.50", "sound_definitions": {"x": {"sounds": [{"name": "a", "stream": true}]}}}"#;
        let (root, report, _) = upgraded(text);
        assert_eq!(report, Upgraded::default());
        assert_eq!(root, json(text));
        let (_, report, _) = upgraded(r#"{"format_version": "1.26.50-beta.1"}"#);
        assert!(report.streaming, "a prerelease of 1.26.50 orders before it");
    }

    /// The `sounds` of the definition `x` after the steps.
    fn x_sounds(text: &str) -> Value {
        let (root, _, diagnostics) = upgraded(text);
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        fork::member(
            fork::member(fork::member(&root, "sound_definitions"), "x"),
            "sounds",
        )
        .clone()
    }

    #[test]
    fn the_1_26_50_step_marks_streamed_object_entries_only() {
        let sounds = x_sounds(
            r#"{"format_version": "1.20.20", "sound_definitions": {"x": {"sounds": [
                {"name": "a", "stream": true},
                {"name": "b", "stream": false},
                {"name": "c"},
                "d",
                {"name": "e", "stream": true, "allow_concurrent_streaming": true},
                {"name": "f", "stream": 1},
                {"name": "g", "stream": "yes"},
                {"name": "h", "stream": null},
                {"name": "i", "stream": true, "allow_concurrent_streaming": null}
            ]}}}"#,
        );
        assert_eq!(
            sounds,
            json(
                r#"[
                {"name": "a", "stream": true, "allow_concurrent_streaming": false},
                {"name": "b", "stream": false},
                {"name": "c"},
                "d",
                {"name": "e", "stream": true, "allow_concurrent_streaming": true},
                {"name": "f", "stream": 1, "allow_concurrent_streaming": false},
                {"name": "g", "stream": "yes", "allow_concurrent_streaming": false},
                {"name": "h", "stream": null},
                {"name": "i", "stream": true, "allow_concurrent_streaming": null}
            ]"#
            )
        );
    }

    #[test]
    fn the_1_26_50_step_walks_only_lists_of_sounds_and_creates_only_the_section() {
        // `sounds` that is an object is not walked (the step needs a list).
        let keyed = r#"{"format_version": "1.20.20", "sound_definitions": {"x": {"sounds": {"k": {"name": "a", "stream": true}}}}}"#;
        assert_eq!(
            x_sounds(keyed),
            json(r#"{"k": {"name": "a", "stream": true}}"#)
        );
        // Without `sound_definitions` the step inserts it as `null`, and the walk finds nothing
        // in it.
        let (root, report, _) = upgraded(r#"{"format_version": "1.20.20"}"#);
        assert_eq!(
            root,
            json(r#"{"format_version": "1.26.50", "sound_definitions": null}"#)
        );
        assert!(report.not_a_list);
        // A list of definitions is walked by its elements, as the iterator does.
        let (root, _, _) = upgraded(
            r#"{"format_version": "1.20.20", "sound_definitions": [{"sounds": [{"name": "a", "stream": true}]}, 3]}"#,
        );
        assert_eq!(
            root,
            json(
                r#"{"format_version": "1.26.50", "sound_definitions": [{"sounds": [{"name": "a", "stream": true, "allow_concurrent_streaming": false}]}, 3]}"#
            )
        );
    }

    #[test]
    fn a_legacy_root_runs_all_three_steps() {
        let (root, report, _) = upgraded(r#"{"x": {"sounds": [{"name": "a", "stream": true}]}}"#);
        assert!(report.wrapped && report.distances && report.streaming);
        assert_eq!(
            root,
            json(
                r#"{"format_version": "1.26.50", "sound_definitions": {"x": {"__use_legacy_max_distance": "true", "sounds": [{"name": "a", "stream": true, "allow_concurrent_streaming": false}], "max_distance": null, "min_distance": null}}}"#
            )
        );
    }

    #[test]
    fn a_root_that_is_no_object_runs_no_step() {
        for text in ["null", "[1]", "\"x\"", "3"] {
            let (root, report, diagnostics) = upgraded(text);
            assert_eq!(root, json(text));
            assert_eq!(report, Upgraded::default());
            assert!(diagnostics.is_empty());
        }
    }

    #[test]
    fn an_integral_float_distance_is_not_zeroed() {
        let (root, report, _) = upgraded(
            r#"{"format_version": "1.14.0", "sound_definitions": {"x": {"max_distance": 16.0, "min_distance": 2}}}"#,
        );
        let x = fork::member(fork::member(&root, "sound_definitions"), "x");
        assert_eq!(fork::as_float(fork::member(x, "max_distance"), -1.0), 16.0);
        assert_eq!(fork::as_float(fork::member(x, "min_distance"), -1.0), 0.0);
        assert_eq!(report.zeroed, vec![("x".to_owned(), "min_distance")]);
    }

    #[test]
    fn the_pitch_of_an_object_entry_goes_and_a_definition_level_pitch_stays() {
        let (root, _, _) =
            upgraded(r#"{"a": {"pitch": 2, "sounds": [{"name": "n", "pitch": 1.2}, "s"]}}"#);
        let a = fork::member(fork::member(&root, "sound_definitions"), "a");
        assert!(fork::has_member(a, "pitch"));
        assert_eq!(*fork::member(a, "sounds"), json(r#"[{"name": "n"}, "s"]"#));
    }
}
