//! 20 R44: the sound events the behaviour packs give server metadata to
//! (`sounds/sound_definitions.json` of a behaviour pack, beta) against the sound events the
//! resource packs define (the file of the same name on the resource side). A lint.

use bedrock_addon::diagnostic::{Diagnostic, DiagnosticDetail, Kind, Phase, Severity};
use bedrock_addon::load::{BuiltIns, LoadOptions};
use bedrock_addon::pack::LinkOptions;
use std::sync::Arc;

use crate::common::{Listed, behavior_pack, resource_pack, stacks_of, unloaded_pack};

const PATH: &str = "sounds/sound_definitions.json";

/// A server sound definitions file with a definition of each of `names`, in order.
fn server(names: &[&str]) -> String {
    let definitions: Vec<String> = names
        .iter()
        .map(|name| format!(r#"{{"sound_event_name": "{name}"}}"#))
        .collect();
    format!(
        r#"{{"format_version": "1.26.60", "minecraft:server_sound_definitions": [{}]}}"#,
        definitions.join(", ")
    )
}

/// A resource pack's sound definitions file that defines each of `names`.
fn client(names: &[&str]) -> String {
    let definitions: Vec<String> = names
        .iter()
        .map(|name| format!(r#""{name}": {{"category": "neutral", "sounds": []}}"#))
        .collect();
    format!(
        r#"{{"format_version": "1.20.20", "sound_definitions": {{{}}}}}"#,
        definitions.join(", ")
    )
}

/// The load options of a world that reads beta documents, with `host` when given.
fn beta(host: Option<&'static [(&'static str, &'static str)]>) -> LoadOptions {
    LoadOptions {
        beta_apis: true,
        built_ins: host.map(|list| Arc::new(Listed(list)) as Arc<dyn BuiltIns>),
        ..LoadOptions::default()
    }
}

/// The `UnknownSound` findings of a world whose behaviour pack declares `declared` and
/// whose resource pack defines `defined`.
fn unknown_with(
    declared: &[&str],
    defined: &[&str],
    load: LoadOptions,
    options: LinkOptions,
) -> Vec<Diagnostic> {
    let set = stacks_of(
        vec![behavior_pack(1, &[(PATH, &server(declared))])],
        vec![resource_pack(2, &[(PATH, &client(defined))])],
        load,
    );
    let found = set.link_with(&options);
    found
        .iter()
        .filter(|finding| finding.code.id() == "core/unknown_sound")
        .cloned()
        .collect()
}

fn unknown(declared: &[&str], defined: &[&str]) -> Vec<Diagnostic> {
    unknown_with(declared, defined, beta(None), LinkOptions::default())
}

/// The JSON path and the name of each finding.
fn named(found: &[Diagnostic]) -> Vec<(String, String)> {
    found
        .iter()
        .map(|finding| {
            let Some(DiagnosticDetail::Reference { kind, id, hint }) = finding.detail.as_deref()
            else {
                panic!("no reference: {finding}");
            };
            assert_eq!(*kind, "sound event");
            assert!(hint.is_none());
            (finding.location.json_path.to_string(), id.clone())
        })
        .collect()
}

fn at(index: usize, name: &str) -> (String, String) {
    (
        format!("minecraft:server_sound_definitions/{index}/sound_event_name"),
        name.to_owned(),
    )
}

#[test]
fn r44_a_name_the_resource_packs_define_is_no_finding() {
    assert!(unknown(&["demo:a", "demo:b"], &["demo:b", "demo:a", "demo:c"]).is_empty());
}

#[test]
fn r44_a_name_no_resource_pack_defines_is_an_unknown_sound_information() {
    let found = unknown(
        &["demo:present", "demo:missing", "demo:gone"],
        &["demo:present"],
    );
    assert_eq!(named(&found), [at(1, "demo:missing"), at(2, "demo:gone")]);
    for finding in &found {
        assert_eq!(finding.severity, Severity::Info);
        assert_eq!(finding.phase, Phase::Link);
        let pack = finding
            .location
            .pack
            .as_ref()
            .map(bedrock_addon::pack::PackRef::name);
        assert_eq!(pack, Some("bp 1"));
        let path = finding
            .location
            .path
            .as_ref()
            .map(bedrock_addon::storage::PackPath::as_str);
        assert_eq!(path, Some(PATH));
        // Ours: vanilla checks nothing, so the finding has no text of its.
        assert_eq!(finding.code.to_string(), "unknown sound event");
    }
}

/// A sound event name is compared byte for byte, like the server's own duplicate check.
#[test]
fn r44_a_name_is_compared_as_written() {
    let found = unknown(&["demo:A", "demo:a", "demo:a.b"], &["demo:a", "demo:A.B"]);
    assert_eq!(named(&found), [at(0, "demo:A"), at(2, "demo:a.b")]);
}

#[test]
fn r44_a_sound_of_the_game_resolves_with_a_host_that_has_it_and_not_without_one() {
    // Without a host the crate has no list of the game's sound definitions.
    let found = unknown(&["game:click"], &[]);
    assert_eq!(named(&found), [at(0, "game:click")]);
    let options = LinkOptions::default();
    let host = beta(Some(&[("sound event", "game:click")]));
    assert!(unknown_with(&["game:click"], &[], host, options).is_empty());
    // A host is believed when it says no; the resource pack's own definition still resolves.
    let other = beta(Some(&[("sound event", "game:pop")]));
    let found = unknown_with(&["game:click", "demo:a"], &["demo:a"], other, options);
    assert_eq!(named(&found), [at(0, "game:click")]);
}

#[test]
fn r44_without_the_lints_nothing_is_told() {
    let quiet = LinkOptions {
        lints: false,
        ..Default::default()
    };
    assert!(unknown_with(&["demo:missing"], &[], beta(None), quiet).is_empty());
}

/// The index is the definition's place in its file, also after a definition the server
/// dropped as a duplicate.
#[test]
fn r44_the_index_is_the_files_also_after_a_dropped_duplicate() {
    let found = unknown(&["demo:a", "demo:a", "demo:b"], &[]);
    assert_eq!(named(&found), [at(0, "demo:a"), at(2, "demo:b")]);
}

/// Each behaviour pack's file has its own indices, and the lowest pack's definition of a
/// name is the one the server keeps and the one told.
#[test]
fn r44_each_pack_s_definitions_are_told_at_their_own_file() {
    let set = stacks_of(
        vec![
            behavior_pack(1, &[(PATH, &server(&["demo:low", "demo:top"]))]),
            behavior_pack(3, &[(PATH, &server(&["demo:other", "demo:low"]))]),
        ],
        vec![resource_pack(2, &[(PATH, &client(&[]))])],
        beta(None),
    );
    let found = set.link();
    let mut told: Vec<(String, String, String)> = found
        .iter()
        .filter(|finding| finding.code.id() == "core/unknown_sound")
        .map(|finding| {
            let pack = finding.location.pack.as_ref().expect("a pack").name();
            let (path, name) = named(std::slice::from_ref(finding)).remove(0);
            (pack.to_owned(), path, name)
        })
        .collect();
    told.sort();
    let place = |pack: &str, index: usize, name: &str| {
        let (path, name) = at(index, name);
        (pack.to_owned(), path, name)
    };
    assert_eq!(
        told,
        [
            place("bp 1", 1, "demo:top"),
            place("bp 3", 0, "demo:other"),
            place("bp 3", 1, "demo:low"),
        ]
    );
}

/// A world without a resource pack defines no sound: every declared name is told.
#[test]
fn r44_without_a_resource_pack_every_name_is_told() {
    let set = stacks_of(
        vec![behavior_pack(1, &[(PATH, &server(&["demo:a"]))])],
        Vec::new(),
        beta(None),
    );
    let found = set.link();
    let unknown: Vec<Diagnostic> = found
        .iter()
        .filter(|finding| finding.code.id() == "core/unknown_sound")
        .cloned()
        .collect();
    assert_eq!(named(&unknown), [at(0, "demo:a")]);
}

/// A pack whose files are not in memory is not read by the pass: nothing of it is loaded,
/// so nothing is told of its definitions.
#[test]
fn r44_a_file_that_is_not_resident_is_not_read() {
    let text = server(&["demo:missing"]);
    let set = stacks_of(
        vec![unloaded_pack(1, "data", &[(PATH, text.as_bytes())])],
        Vec::new(),
        beta(None),
    );
    let found = set.link();
    assert!(
        found
            .iter()
            .all(|finding| finding.code.kind() != Kind::UnresolvedReference),
        "{found:?}"
    );
}
