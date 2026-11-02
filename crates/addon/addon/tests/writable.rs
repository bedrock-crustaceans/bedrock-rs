#![allow(clippy::pedantic, missing_docs)]
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg(feature = "full")]
//! Every document that loads is writable, and no value is rewritten or dropped in silence.
//!
//! The invariant, for every document type with a writer: for any text `T`, if the load of `T`
//! yields a document, then `to_json` of it is `Ok(W)`, the load of `W` yields a document
//! without an error (but one the load of `T` gave at the same place: an entry `blocks.json`
//! refuses and keeps as written), and `to_json` of that is `W` again; and every difference between `T` and
//! `W` that changes a value (a number clamped, a member dropped, a type changed) has a finding
//! at that place in the load of `T`.
//!
//! One test per document type (the list of `common/document_types.rs`, which `non_finite.rs`
//! walks with unknown members; this probe walks the members the documents HAVE). Each test
//! takes the type's small document and, when `BEDROCK_ADDON_CORPUS` is set, a sample of the
//! vanilla files of the family, and replaces one value at a time:
//!
//! * every **number** by each of [`NUMBERS`] (`0.5` only where an integer stood);
//! * every **string** and **bool** by the number `7`, and every **object** or **array** below
//!   the root by `7` (the wrong-shape probes): at every place of a small document, once per
//!   distinct member name of the family in the corpus files.
//!
//! For each text one of these must hold, or the type's test fails:
//!
//! * **refused**: the load yields no document;
//! * **held**: the written text holds the replacement at the replaced place, as the value it
//!   spells (a number as its number, whatever the token; a float member holds it as the
//!   nearest `f32`);
//! * **absent**: for the families of [`NULL_IS_ABSENT`] alone, the replacement is `null` in
//!   an object, and the written object has no such member (a JSON `null` spells absence: `Option` members read it so by the rule of the
//!   strict wrappers);
//! * **said**: a finding the unchanged document does not have (the same code, text and path)
//!   is at the replaced place, or one it does not have (by code and path) is at an enclosing
//!   member (which it drops or rejects) or below
//!   the place (the member of the long form that a short form stands for), or a finding is
//!   at an enclosing member the written text does not have (dropped whole, as it says). A
//!   finding at the place or below it alone is "said" only where the written text loses
//!   nothing BESIDE the member that the unchanged document's written text holds (a
//!   component that goes whole for one member is told at the component; an element of a list
//!   is told of as its list is);
//! * **held elsewhere**: the written text holds the replacement at a place the unchanged
//!   document's written text has no such value (an upgrade step moved the member);
//! * **not written, as the load says**: for the families of [`TOLD_NOT_WRITTEN`] alone, each
//!   with its D- entry in the table (a component-based item whose every component the load
//!   dropped; block writes a document the newest format refuses in its older
//!   format); and **exempt**, the places of [`EXEMPT`], each with its D- entry.
//!
//! # The sizes
//!
//! The small documents are probed in full. Of the corpus, each family takes at most
//! [`FILES`] files (evenly spaced over the sorted list), at most [`NUMBERS_PER_FILE`] numbers
//! of a file, at most [`PER_SIGNATURE`] places of one signature (the last two member names of
//! the path) and at most [`BUDGET`] bytes of probe text over the family, so that a family's
//! test stays within about two minutes in the dev profile. `writable:` lines (shown with
//! `--show-output`) give the counts, with the places the caps left out.
//!
//! # Four binaries
//!
//! A test binary runs under a limit of CPU seconds, and the probe of all the document types
//! is far beyond it on a loaded machine: the twelve types of the most texts are probed by
//! three binaries of their own (`writable_2.rs`: animation, atmosphere, fog,
//! music_definitions, particle; `writable_3.rs`: entity, feature, item, ui_global_variables;
//! `writable_4.rs`: geometry, material_file, ui_file), every other type here. The harness
//! (`writable/harness.rs`) is one, and `the_binaries_of_the_probe_share_out_every_document_type`
//! holds the four lists against each other.

#[macro_use]
#[path = "common/document_types.rs"]
mod document_types;
#[path = "common/places.rs"]
mod places;

/// The document types this binary probes: all but those of `writable_2.rs`, `writable_3.rs`
/// and `writable_4.rs`.
macro_rules! in_this_binary {
    (animation $body:tt) => {};
    (geometry $body:tt) => {};
    (entity $body:tt) => {};
    (atmosphere $body:tt) => {};
    (feature $body:tt) => {};
    (ui_global_variables $body:tt) => {};
    (ui_file $body:tt) => {};
    (music_definitions $body:tt) => {};
    (material_file $body:tt) => {};
    (particle $body:tt) => {};
    (item $body:tt) => {};
    (fog $body:tt) => {};
    ($other:ident { $($body:tt)* }) => { $($body)* };
}

include!("writable/harness.rs");

/// The saved pack settings have an entry of their own (`SavedSettings::parse`).
#[test]
fn pack_settings() {
    use bedrock_addon::pack::settings::SavedSettings;
    let read = |_: &PackPath, text: &str| {
        outcome(SavedSettings::parse(text.as_bytes()), |value| {
            value.to_json().map_err(|error| format!("{error:?}"))
        })
    };
    audit(
        "pack_settings",
        "pack_settings.json",
        &read,
        &read,
        r#"{"format_version": "1.21.100", "minecraft:pack_settings": {"settings": [{"pack_id": "d3349771-6782-4420-875a-fcd6179a8915", "values": {"banana:slider": 2.5}}]}}"#,
    );
}

// --- the harness itself ----------------------------------------------------------------------

/// The document types the `in_this_binary!` of `source` names, one per rule; the rule for
/// every other type, which begins with a variable, names none. A line of the list that is no
/// rule of one line fails: no rule is passed over.
fn named_in_this_binary(source: &str) -> Vec<&str> {
    let (_, rules) = source
        .split_once("macro_rules! in_this_binary {")
        .expect("the binary's list of document types");
    let (rules, _) = rules.split_once("\n}").expect("the end of the list");
    rules
        .lines()
        .map(str::trim)
        .filter(|rule| !rule.is_empty())
        .filter_map(|rule| {
            let matcher = rule
                .strip_prefix('(')
                .unwrap_or_else(|| panic!("no rule of one line: `{rule}`"))
                .trim_start();
            assert!(rule.ends_with(';'), "no rule of one line: `{rule}`");
            if matcher.starts_with('$') {
                return None;
            }
            let end = matcher
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .unwrap_or(matcher.len());
            assert!(end > 0, "a rule that names no type: `{rule}`");
            Some(&matcher[..end])
        })
        .collect()
}

/// Every document type is probed by one binary: the types this binary leaves out are those
/// the other three take, each taken by one of them, and each a row of the list of document
/// types (a name that is no row would leave a type out here and probe it nowhere).
#[test]
fn the_binaries_of_the_probe_share_out_every_document_type() {
    let left_out = named_in_this_binary(include_str!("writable.rs"));
    let taken: Vec<&str> = [
        include_str!("writable_2.rs"),
        include_str!("writable_3.rs"),
        include_str!("writable_4.rs"),
    ]
    .into_iter()
    .flat_map(named_in_this_binary)
    .collect();
    let set = |names: &[&'static str]| names.iter().copied().collect::<BTreeSet<_>>();
    assert_eq!(
        set(&taken).len(),
        taken.len(),
        "a type taken by two binaries: {taken:?}"
    );
    assert_eq!(
        set(&left_out).len(),
        left_out.len(),
        "a type left out twice: {left_out:?}"
    );
    assert_eq!(set(&left_out), set(&taken));
    assert!(!taken.is_empty());
    let rows = include_str!("common/document_types.rs");
    for name in left_out {
        assert!(
            rows.contains(&format!("\n        {name}: bedrock_addon::")),
            "{name} is no row of the document types"
        );
    }
}

/// The reader of the lists takes a rule however its name is spaced, and passes no line over.
#[test]
fn the_reader_of_a_binary_s_list_takes_every_rule() {
    let list = "macro_rules! in_this_binary {\n    (fog $body:tt) => {};\n    ( item { $($body:tt)* }) => { $($body)* };\n\n    ($other:ident $body:tt) => {};\n}\n";
    assert_eq!(named_in_this_binary(list), ["fog", "item"]);
    for broken in [
        "macro_rules! in_this_binary {\n    (fog\n        $body:tt) => {};\n}\n",
        "macro_rules! in_this_binary {\n    [fog $body:tt] => {};\n}\n",
        "macro_rules! in_this_binary {\n    ({ fog }) => {};\n}\n",
    ] {
        assert!(
            std::panic::catch_unwind(|| named_in_this_binary(broken)).is_err(),
            "{broken}"
        );
    }
}

#[test]
fn every_value_of_a_document_is_a_place() {
    let found = nodes(r#"{"a": {"b": [1, {"c": "}\""}, -2.5e3]}, "d": null}"#);
    let shown: Vec<String> = found
        .iter()
        .map(|node| format!("{} {:?}", node.shown(), node.kind))
        .collect();
    assert_eq!(
        shown,
        [
            " Object",
            "a Object",
            "a/b Array",
            "a/b/0 Number",
            "a/b/1 Object",
            "a/b/1/c String",
            "a/b/2 Number",
            "d Other"
        ]
    );
    assert_eq!(found[5].signature(), "b/*/c");
    assert_eq!(found[6].name(), "b/*");
    assert_eq!(found[3].name(), "b/*");
    assert_eq!(found[1].name(), "a");
}

#[test]
fn a_number_is_compared_as_the_number_it_spells() {
    let spell = |text: &str| scalar(&parse(text).expect("JSON"));
    assert_eq!(spell("7"), spell("7.0"));
    assert_eq!(
        spell("1e39"),
        spell("1000000000000000000000000000000000000000")
    );
    assert_ne!(spell("7"), spell("\"7\""));
    assert_ne!(spell("-2147483649"), spell("-2147483648"));
}
