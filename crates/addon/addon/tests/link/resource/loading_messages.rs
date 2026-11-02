//! The link rule of the loading messages: a loading-screen tip no language translates.

use bedrock_addon::diagnostic::{Diagnostic, DiagnosticDetail, Diagnostics, Phase, Severity};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{LinkOptions, Pack};

use crate::common::{hosted, resource_pack_of, stacks_of};

/// `bytes` for the life of the test process.
fn keep(bytes: &[u8]) -> &'static [u8] {
    Box::leak(bytes.to_vec().into_boxed_slice())
}

/// The resource pack `n` with `files`.
fn pack(n: u128, files: &[(&'static str, &[u8])]) -> Pack {
    let files: Vec<(&'static str, &'static [u8])> = files
        .iter()
        .map(|(path, bytes)| (*path, keep(bytes)))
        .collect();
    resource_pack_of(n, &files)
}

/// The link findings of a world of the resource packs `packs`, top first.
fn linked(packs: Vec<Pack>, load: LoadOptions, options: LinkOptions) -> Diagnostics {
    stacks_of(Vec::new(), packs, load).link_with(&options)
}

/// The link findings of one resource pack (`rp 2`) with `files`.
#[cfg(not(all(feature = "lang", feature = "languages")))]
fn link(files: &[(&'static str, &[u8])]) -> Diagnostics {
    linked(
        vec![pack(2, files)],
        LoadOptions::default(),
        &LinkOptions::default(),
    )
}

/// What a finding says: code, severity, pack, file, JSON path.
type Said = (&'static str, Severity, String, String, String);

fn said(finding: &Diagnostic) -> Said {
    assert_eq!(finding.phase, Phase::Link, "{finding}");
    let pack = finding
        .location
        .pack
        .as_ref()
        .map(bedrock_addon::pack::PackRef::name);
    let path = finding.location.path.as_ref().map(ToString::to_string);
    (
        finding.code.name(),
        finding.severity,
        pack.unwrap_or_default().to_owned(),
        path.unwrap_or_default(),
        finding.location.json_path.to_string(),
    )
}

fn all_said(found: &Diagnostics) -> Vec<Said> {
    found.iter().map(said).collect()
}

/// The kind and the name of a reference finding.
fn named(finding: &Diagnostic) -> (&'static str, String) {
    match finding.detail.as_deref() {
        Some(DiagnosticDetail::Reference { kind, id, .. }) => (*kind, id.clone()),
        other => panic!("no reference: {other:?}"),
    }
}

#[cfg(all(feature = "lang", feature = "languages"))]
mod tips {
    use super::*;

    const MESSAGES: &str = "loading_messages.json";
    const LANGUAGES: (&str, &[u8]) = ("texts/languages.json", br#"["en_US", "de_DE"]"#);
    const ENGLISH: (&str, &[u8]) = ("texts/en_US.lang", b"tips.known=A tip\n");
    const GERMAN: (&str, &[u8]) = ("texts/de_DE.lang", b"tips.nur=Ein Tipp\n");

    /// The findings of a world whose pack has the two languages and the tips file `text`.
    fn tips_with(text: &str, load: LoadOptions, options: LinkOptions) -> Diagnostics {
        let files = [(MESSAGES, text.as_bytes()), LANGUAGES, ENGLISH, GERMAN];
        linked(vec![pack(2, &files)], load, options)
    }

    fn tips(text: &str) -> Diagnostics {
        tips_with(text, LoadOptions::default(), LinkOptions::default())
    }

    /// `(JSON path, key)` of each finding, each held to the rule's code, severity and place.
    fn unresolved(found: &Diagnostics) -> Vec<(String, String)> {
        found
            .iter()
            .map(|finding| {
                let (code, severity, pack, file, path) = said(finding);
                assert_eq!(
                    (code, severity, pack.as_str(), file.as_str()),
                    ("UnresolvedLangKey", Severity::Info, "rp 2", MESSAGES),
                    "{finding}"
                );
                let (kind, key) = named(finding);
                assert_eq!(kind, "lang key");
                (path, key)
            })
            .collect()
    }

    fn at(path: &str, key: &str) -> (String, String) {
        (path.to_owned(), key.to_owned())
    }

    #[test]
    fn l9_a_tip_some_language_translates_is_no_finding() {
        // A key of the first language, and one only the second has.
        let found = tips(r#"{"beginner_loading_messages": ["tips.known", "tips.nur"]}"#);
        assert_eq!(unresolved(&found), []);
    }

    #[test]
    fn l9_a_tip_no_language_translates_is_an_unresolved_lang_key_information() {
        let found = tips(
            r#"{"beginner_loading_messages": ["tips.known", "tips.unknown", "A text shown as it is"]}"#,
        );
        assert_eq!(
            unresolved(&found),
            [
                at("beginner_loading_messages/1", "tips.unknown"),
                at("beginner_loading_messages/2", "A text shown as it is"),
            ]
        );
        assert_eq!(
            found.iter().next().unwrap().code.to_string(),
            "localisation key not defined"
        );
    }

    /// A key is looked up without ASCII case, as the game's translator does.
    #[test]
    fn l9_a_key_is_found_whatever_its_ascii_case() {
        let found = tips(r#"{"beginner_loading_messages": ["Tips.KNOWN", "tips.knöwn"]}"#);
        assert_eq!(
            unresolved(&found),
            [at("beginner_loading_messages/1", "tips.knöwn")]
        );
    }

    #[test]
    fn l9_every_list_the_game_reads_is_checked_at_its_own_member() {
        let found = tips(
            r#"{"beginner_loading_messages": ["b"], "mid_game_loading_messages": ["tips.known", "m"],
                "late_game_loading_messages": ["l"], "realms_loading_messages": ["r"],
                "addons_loading_messages": ["a"], "creative_loading_messages": ["c"],
                "editor_loading_messages": ["e"]}"#,
        );
        assert_eq!(
            unresolved(&found),
            [
                at("beginner_loading_messages/0", "b"),
                at("mid_game_loading_messages/1", "m"),
                at("late_game_loading_messages/0", "l"),
                at("realms_loading_messages/0", "r"),
                at("addons_loading_messages/0", "a"),
                at("creative_loading_messages/0", "c"),
                at("editor_loading_messages/0", "e"),
            ]
        );
    }

    /// While `loading_messages` has an element the game answers the three stages from it and
    /// never reads the stage lists.
    #[test]
    fn l9_the_stage_lists_are_not_checked_while_the_older_list_stands_for_them() {
        let found = tips(
            r#"{"loading_messages": ["old"], "beginner_loading_messages": ["b"],
                "mid_game_loading_messages": ["m"], "late_game_loading_messages": ["l"],
                "realms_loading_messages": ["r"]}"#,
        );
        assert_eq!(
            unresolved(&found),
            [
                at("loading_messages/0", "old"),
                at("realms_loading_messages/0", "r")
            ]
        );
        // An empty older list stands for nothing.
        let found = tips(r#"{"loading_messages": [], "beginner_loading_messages": ["b"]}"#);
        assert_eq!(unresolved(&found), [at("beginner_loading_messages/0", "b")]);
    }

    /// A conditional message is told at its `message`; one the game never shows (a condition
    /// it does not know), an element it skips and an empty text are no tips.
    #[test]
    fn l9_only_a_text_the_game_can_show_is_a_tip() {
        let found = tips(
            r#"{"beginner_loading_messages": [
                {"message": "shown", "condition": "$marketplace_not_disabled"},
                {"message": "never", "condition": "$other"},
                {"message": "plain"},
                7, "", {"message": "tips.known"}]}"#,
        );
        assert_eq!(
            unresolved(&found),
            [
                at("beginner_loading_messages/0/message", "shown"),
                at("beginner_loading_messages/2/message", "plain"),
            ]
        );
    }

    /// A list written as an object is read as the game iterates it, its member values in key
    /// order: a tip of it is told at the key the file has it under, not at a place in that
    /// order.
    #[test]
    fn l9_a_tip_of_a_list_written_as_an_object_is_told_at_its_key() {
        let found = tips(
            r#"{"beginner_loading_messages": {"b": "tips.unknown", "a": "tips.known",
                "d": {"message": "plain"}, "c": 7, "10": "ten"}}"#,
        );
        assert_eq!(
            unresolved(&found),
            [
                at("beginner_loading_messages/10", "ten"),
                at("beginner_loading_messages/b", "tips.unknown"),
                at("beginner_loading_messages/d/message", "plain"),
            ]
        );
    }

    /// One finding per member: a text two lists hold is told at both.
    #[test]
    fn l9_a_text_in_two_lists_is_told_at_each() {
        let found = tips(
            r#"{"beginner_loading_messages": ["twice", "twice"], "realms_loading_messages": ["twice"]}"#,
        );
        assert_eq!(
            unresolved(&found),
            [
                at("beginner_loading_messages/0", "twice"),
                at("beginner_loading_messages/1", "twice"),
                at("realms_loading_messages/0", "twice"),
            ]
        );
    }

    #[test]
    fn l9_a_key_of_the_game_resolves_with_a_host_that_has_it_and_not_without_one() {
        let text = r#"{"beginner_loading_messages": ["tips.game.1", "tips.known"]}"#;
        assert_eq!(
            unresolved(&tips(text)),
            [at("beginner_loading_messages/0", "tips.game.1")]
        );
        let options = LinkOptions::default();
        let host = hosted(&[("lang key", "tips.game.1")]);
        assert_eq!(unresolved(&tips_with(text, host, options)), []);
        // A host that lacks the key: the stack's own key still resolves.
        let other = hosted(&[("lang key", "tips.game.2")]);
        assert_eq!(
            unresolved(&tips_with(text, other, options)),
            [at("beginner_loading_messages/0", "tips.game.1")]
        );
    }

    #[test]
    fn l9_without_the_lints_nothing_is_told() {
        let quiet = LinkOptions {
            lints: false,
            ..Default::default()
        };
        let text = r#"{"beginner_loading_messages": ["tips.unknown"]}"#;
        let found = tips_with(text, LoadOptions::default(), quiet);
        assert_eq!(all_said(&found), []);
    }

    /// The tips are the highest pack's file, whole: a lower pack's tips are never shown and
    /// never told, and the languages of every pack translate.
    #[test]
    fn l9_only_the_highest_pack_s_file_is_checked_against_every_pack_s_languages() {
        let top = pack(
            2,
            &[(
                MESSAGES,
                br#"{"beginner_loading_messages": ["tips.known", "tips.top"]}"#,
            )],
        );
        let bottom = pack(
            3,
            &[
                (
                    MESSAGES,
                    br#"{"beginner_loading_messages": ["tips.bottom"]}"#,
                ),
                LANGUAGES,
                ENGLISH,
                GERMAN,
            ],
        );
        let found = linked(
            vec![top, bottom],
            LoadOptions::default(),
            LinkOptions::default(),
        );
        assert_eq!(
            unresolved(&found),
            [at("beginner_loading_messages/1", "tips.top")]
        );
    }
}

/// Without the `text` feature the stack has no translations and the rule does not exist.
#[cfg(not(all(feature = "lang", feature = "languages")))]
#[test]
fn l9_without_the_text_feature_no_tip_is_looked_up() {
    let found = link(&[(
        "loading_messages.json",
        br#"{"beginner_loading_messages": ["tips.unknown"]}"#,
    )]);
    assert!(
        found
            .iter()
            .all(|finding| finding.code.name() != "UnresolvedLangKey"),
        "{found:?}"
    );
}
