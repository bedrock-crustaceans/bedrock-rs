//! The audio files a sound definition names, judged by the link pass (`UnresolvedReference` for
//! a name that resolves to no file, `InvalidValue` for a name that ends in an extension): the
//! rules of the three families `sound_definitions` and `sound_file` together, over packs in a
//! stack set. Every rule here is ours: vanilla checks nothing at load.

use std::sync::Arc;

use bedrock_addon::diagnostic::{
    Diagnostic, DiagnosticDetail, Diagnostics, Phase, ReferenceHint, Severity,
};
use bedrock_addon::families::sound_file::SOUND_FILE_KIND;
use bedrock_addon::load::{BuiltIns, LoadOptions};
use bedrock_addon::pack::{Pack, PackOrigin, StackSet};

use super::common::{
    DEFINITIONS, Findable, describe, expect_clean, expect_none, files, found, linked_user, pack_of,
    pack_with_origin, plays, stack_set, vorbis,
};

/// Built-ins that know the listed sound files.
struct VanillaIndex(&'static [&'static str]);

impl BuiltIns for VanillaIndex {
    fn contains(&self, kind: &'static str, id: &str) -> bool {
        kind == SOUND_FILE_KIND && self.0.contains(&id)
    }
}

/// The stack set of `packs` (lowest first, all served ones) over a client whose files are `index`.
fn with_index(packs: Vec<Pack>, index: &'static [&'static str]) -> StackSet {
    stack_set(
        packs
            .into_iter()
            .map(|pack| (pack, PackOrigin::User))
            .collect(),
        LoadOptions {
            built_ins: Some(Arc::new(VanillaIndex(index))),
            ..LoadOptions::default()
        },
    )
}

/// The findings of the sound definitions of `set` and of its link pass.
fn judged<'a>(set: &'a StackSet, link: &'a Diagnostics) -> Vec<&'a Diagnostic> {
    set.resource
        .sound_definitions()
        .diagnostics()
        .chain(link.iter())
        .collect()
}

/// The names of the unresolved sound files, with the case hint of each.
fn unresolved(findings: &impl Findable) -> Vec<(String, Option<String>)> {
    found(findings, "UnresolvedReference")
        .iter()
        .map(|d| match d.detail.as_deref() {
            Some(DiagnosticDetail::Reference { kind, id, hint }) => {
                assert_eq!(*kind, SOUND_FILE_KIND);
                assert_eq!(d.severity, Severity::Info);
                assert_eq!(d.phase, Phase::Link);
                let hint = hint.as_ref().map(|hint| match hint {
                    ReferenceHint::CaseMismatch(path) => path.clone(),
                    other => panic!("{other:?}"),
                });
                (id.clone(), hint)
            }
            other => panic!("{other:?}"),
        })
        .collect()
}

/// A resource pack of the client's own (`PackOrigin::Package`) with the files.
fn client_pack(uuid: u128, files: &[(&str, &[u8])]) -> Pack {
    pack_with_origin(uuid, PackOrigin::Package, files)
}

const PLAYS_X: &[u8] =
    br#"{"format_version": "1.20.20", "sound_definitions": {"a": {"sounds": ["sounds/x"]}}}"#;

/// Built-ins that know `sounds/x.fsb`.
struct Client;

impl BuiltIns for Client {
    fn contains(&self, kind: &'static str, id: &str) -> bool {
        kind == SOUND_FILE_KIND && id == "sounds/x.fsb"
    }
}

#[test]
fn as03_a_name_without_a_file_deviation() {
    // vanilla = "no load-time check; the sound silently does not play".
    let packs = || {
        vec![files(
            10,
            &[(DEFINITIONS, plays(&["sounds/nope", ""]).into_bytes())],
        )]
    };
    let set = with_index(packs(), &[]);
    let link = set.link();
    assert_eq!(unresolved(&link), [("sounds/nope".to_owned(), None)]);
    let finding = found(&link, "UnresolvedReference")[0];
    assert_eq!(
        finding.location.json_path.to_string(),
        "sound_definitions/a/sounds/0"
    );
    assert_eq!(pack_of(finding), Some(0));
    // Without the client's own files in view (no built-ins, no base-game pack that lists
    // audio) nothing is said: an add-on's listing is partial by nature.
    expect_none(&linked_user(packs()), "UnresolvedReference");
    let mut with_audio = packs();
    with_audio.push(files(20, &[("sounds/other.ogg", vorbis())]));
    expect_none(&linked_user(with_audio), "UnresolvedReference");
}

#[test]
fn as04_the_vanilla_index_resolves_without_a_format_check() {
    let set = with_index(
        vec![files(
            10,
            &[(
                DEFINITIONS,
                plays(&["sounds/random/click", "sounds/w"]).into_bytes(),
            )],
        )],
        &["sounds/random/click.fsb", "sounds/w.wav"],
    );
    let link = set.link();
    let findings = judged(&set, &link);
    assert_eq!(findings.len(), 0, "{:#?}", describe(&findings));
    assert!(
        set.resource
            .sound_files()
            .get("sounds/random/click")
            .is_none()
    );
}

#[test]
fn as05_a_case_mismatch_does_not_resolve() {
    let set = with_index(
        vec![files(
            10,
            &[
                (DEFINITIONS, plays(&["sounds/boom"]).into_bytes()),
                ("sounds/Boom.ogg", vorbis()),
            ],
        )],
        &[],
    );
    assert_eq!(
        unresolved(&set.link()),
        [("sounds/boom".to_owned(), Some("sounds/Boom.ogg".to_owned()))]
    );
    let files = set.resource.sound_files();
    assert!(files.get("sounds/boom").is_none());
    assert!(files.get("sounds/Boom").is_some());
}

#[test]
fn as06_a_name_with_an_extension() {
    let set = with_index(
        vec![files(
            10,
            &[
                (
                    DEFINITIONS,
                    plays(&["sounds/x.ogg", "sounds/y.MP3", "sounds/z.oggx"]).into_bytes(),
                ),
                ("sounds/x.ogg", vorbis()),
            ],
        )],
        &[],
    );
    let link = set.link();
    let warned: Vec<String> = found(&link, "InvalidValue")
        .iter()
        .map(|d| d.location.json_path.to_string())
        .collect();
    assert_eq!(
        warned,
        [
            "sound_definitions/a/sounds/0",
            "sound_definitions/a/sounds/1"
        ]
    );
    assert!(
        found(&link, "InvalidValue")
            .iter()
            .all(|d| d.severity == Severity::Info && d.phase == Phase::Link)
    );
    // The doubled path does not exist either.
    assert_eq!(unresolved(&link).len(), 3);
    // A doubled file resolves the name, and the extension is still reported.
    let set = with_index(
        vec![files(
            10,
            &[
                (DEFINITIONS, plays(&["sounds/x.ogg"]).into_bytes()),
                ("sounds/x.ogg.ogg", vorbis()),
            ],
        )],
        &[],
    );
    let link = set.link();
    assert!(unresolved(&link).is_empty());
    assert_eq!(found(&link, "InvalidValue").len(), 1);
}

#[test]
fn a_pack_whose_audio_is_all_ignored_leaves_no_name_to_judge() {
    // Nothing to resolve against: the stack ships no file the client loads.
    let set = with_index(
        vec![files(
            10,
            &[
                (DEFINITIONS, plays(&["sounds/x"]).into_bytes()),
                ("sounds/x.mp3", b"ID3".to_vec()),
                ("sounds/y.OGG", vorbis()),
                ("sounds/notes.json", b"{}".to_vec()),
            ],
        )],
        &[],
    );
    assert!(set.resource.sound_files().by_name.is_empty());
    let served = linked_user(vec![files(
        10,
        &[
            (DEFINITIONS, plays(&["sounds/x"]).into_bytes()),
            ("sounds/x.mp3", b"ID3".to_vec()),
        ],
    )]);
    assert!(unresolved(&served).is_empty());
}

#[test]
fn as11_the_sounds_of_an_external_event_are_not_checked() {
    let text = r#"{"format_version": "1.20.20", "sound_definitions": {"a": {"external_event_name": "ev", "sounds": ["sounds/nope"]}}}"#;
    let set = with_index(
        vec![files(10, &[(DEFINITIONS, text.as_bytes().to_vec())])],
        &[],
    );
    let link = set.link();
    let findings = judged(&set, &link);
    assert!(unresolved(&findings).is_empty());
    assert_eq!(
        found(&findings, "InvalidValue").len(),
        1,
        "rule 8's error only"
    );
    assert_eq!(findings.len(), 1);
}

#[test]
fn a_name_too_long_for_the_lookup_buffer_is_unresolved_even_with_its_file() {
    let long = format!("sounds/{}", "a".repeat(1013));
    assert_eq!(long.len(), 1020);
    let fits = format!("sounds/{}", "a".repeat(1012));
    let (long_file, fits_file) = (format!("{long}.ogg"), format!("{fits}.ogg"));
    // Path limits of the pack layer may refuse such a file; the built-ins answer for it.
    let index: &'static [&'static str] = Box::leak(
        vec![
            &*Box::leak(long_file.into_boxed_str()),
            &*Box::leak(fits_file.into_boxed_str()),
        ]
        .into_boxed_slice(),
    );
    let set = with_index(
        vec![files(
            10,
            &[(DEFINITIONS, plays(&[&long, &fits]).into_bytes())],
        )],
        index,
    );
    assert_eq!(unresolved(&set.link()), [(long, None)]);
}

#[test]
fn a_dangling_name_is_judged_only_with_the_clients_files_in_view() {
    let files: &[(&str, &[u8])] = &[(DEFINITIONS, PLAYS_X), ("sounds/other.ogg", b"")];
    let served = || {
        vec![(
            pack_with_origin(10, PackOrigin::User, files),
            PackOrigin::User,
        )]
    };
    let link = |packs, options| stack_set(packs, options).link();
    // A served pack's own listing is partial: nothing is said.
    expect_clean(&link(served(), LoadOptions::default()));
    // A base-game pack that lists audio is the client's own files: the name is dangling. The
    // base-game pack is below the served one.
    let client: &[(&str, &[u8])] = &[("sounds/client.fsb", b"")];
    let mut with_client = vec![(client_pack(20, client), PackOrigin::Package)];
    with_client.extend(served());
    assert_eq!(
        found(
            &link(with_client, LoadOptions::default()),
            "UnresolvedReference"
        )
        .len(),
        1
    );
    // A base-game pack without audio is not.
    let mut without = vec![(
        client_pack(20, &[("sounds.json", b"{}")]),
        PackOrigin::Package,
    )];
    without.extend(served());
    expect_clean(&link(without, LoadOptions::default()));
    // Built-ins are the client's files too: they answer for `sounds/x`, and judge another.
    let options = LoadOptions {
        built_ins: Some(Arc::new(Client)),
        ..LoadOptions::default()
    };
    expect_clean(&link(served(), options));
}
