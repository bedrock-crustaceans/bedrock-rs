//! Unit tests of what only the crate can reach: the pack-keyword rules (L5, L6) over every
//! combination of header name and description, and the walk.

use bedrock_addon_core::pack_identity::{PackIdVersion, PackPath, PackRef, PackType};
use uuid::Uuid;

use super::Language;
use super::stack::{KeywordFacts, LangFacts, keyword_findings};
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Severity};
use bedrock_addon_core::version::SemVersion;

fn lang(code: &str) -> Language {
    Language::new(code)
}

// ---- L5 and L6 -----------------------------------------------------------------------------

fn pack() -> PackRef {
    let id = PackIdVersion::new(
        Uuid::from_u128(9),
        SemVersion::new(1, 0, 0),
        PackType::Resource,
    );
    PackRef::new(0, id, "a pack")
}

/// `texts/<code>.lang` defining `pack.name` and / or `pack.description`.
fn file(code: &str, name: bool, description: bool) -> LangFacts {
    LangFacts {
        language: lang(code),
        path: PackPath::new(&format!("texts/{code}.lang")).unwrap(),
        defines: [name, description],
    }
}

/// The findings for a pack with this header, these own codes and these files, as
/// `(code name, severity, path, breadcrumb)`.
fn findings(
    name: &str,
    description: Option<&str>,
    own_codes: &[&str],
    files: &[LangFacts],
) -> Vec<(&'static str, Severity, String, String)> {
    let pack = pack();
    let own_codes: Vec<Language> = own_codes.iter().map(|code| lang(code)).collect();
    let found = keyword_findings(&KeywordFacts {
        pack: &pack,
        name,
        description,
        own_codes: &own_codes,
        files,
    });
    found.iter().map(describe).collect()
}

fn describe(diagnostic: &Diagnostic) -> (&'static str, Severity, String, String) {
    assert_eq!(diagnostic.location.pack.as_ref(), Some(&pack()));
    (
        diagnostic.code.name(),
        diagnostic.severity,
        diagnostic
            .location
            .path
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default(),
        diagnostic.location.json_path.to_string(),
    )
}

fn unused(path: &str, key: &str) -> (&'static str, Severity, String, String) {
    (
        "IgnoredMember",
        Severity::Info,
        path.to_owned(),
        key.to_owned(),
    )
}

fn unresolved(field: &str) -> (&'static str, Severity, String, String) {
    (
        "UnresolvedLangKey",
        Severity::Info,
        "manifest.json".to_owned(),
        format!("header/{field}"),
    )
}

#[test]
fn the_findings_carry_no_vanilla_text() {
    let pack = pack();
    let found = keyword_findings(&KeywordFacts {
        pack: &pack,
        name: "pack.name",
        description: Some("plain"),
        own_codes: &[lang("en_US")],
        files: &[file("en_US", false, true)],
    });
    assert_eq!(found.len(), 2);
    assert!(matches!(
        found[0].code,
        DiagnosticCode::IgnoredMember { text: None }
    ));
    assert!(matches!(found[1].code, DiagnosticCode::UnresolvedLangKey));
}

#[test]
fn a_keyword_the_manifest_does_not_use_is_never_used() {
    // L5, first half
    assert_eq!(
        findings(
            "My Pack",
            Some("About it"),
            &["en_US"],
            &[file("en_US", true, true)]
        ),
        [
            unused("texts/en_US.lang", "pack.name"),
            unused("texts/en_US.lang", "pack.description"),
        ]
    );
    // each keyword on its own
    assert_eq!(
        findings(
            "pack.name",
            Some("About it"),
            &["en_US"],
            &[file("en_US", true, true)]
        ),
        [unused("texts/en_US.lang", "pack.description")]
    );
    assert_eq!(
        findings(
            "My Pack",
            Some("pack.description"),
            &["en_US"],
            &[file("en_US", true, true)]
        ),
        [unused("texts/en_US.lang", "pack.name")]
    );
}

#[test]
fn a_keyword_either_header_field_uses_is_used() {
    assert_eq!(
        findings(
            "pack.name",
            Some("pack.description"),
            &["en_US"],
            &[file("en_US", true, true)]
        ),
        []
    );
    // `isPackKeyword` is tested on both fields against both keywords
    assert_eq!(
        findings(
            "pack.description",
            Some("pack.name"),
            &["en_US"],
            &[file("en_US", true, true)]
        ),
        []
    );
}

#[test]
fn without_the_description_an_unused_keyword_is_not_judged() {
    assert_eq!(
        findings("My Pack", None, &["en_US"], &[file("en_US", true, true)]),
        []
    );
    assert_eq!(
        findings("pack.name", None, &["en_US"], &[file("en_US", true, true)]),
        []
    );
}

#[test]
fn a_keyword_in_a_file_of_a_code_the_pack_does_not_list_is_never_loaded() {
    // L5, second half: also when the manifest uses the keyword, and without the description
    for description in [Some("pack.description"), None] {
        assert_eq!(
            findings(
                "pack.name",
                description,
                &["en_US"],
                &[file("en_US", true, true), file("de_DE", true, false),]
            ),
            [unused("texts/de_DE.lang", "pack.name")]
        );
    }
    // a pack without `languages.json` lists nothing
    assert_eq!(
        findings("My Pack", None, &[], &[file("en_US", false, true)]),
        [unused("texts/en_US.lang", "pack.description")]
    );
}

#[test]
fn a_file_without_keywords_is_never_reported() {
    assert_eq!(
        findings(
            "My Pack",
            Some("About"),
            &[],
            &[file("en_US", false, false)]
        ),
        []
    );
}

#[test]
fn a_used_keyword_no_listed_file_defines_is_unresolved() {
    // L6
    assert_eq!(
        findings(
            "pack.name",
            Some("pack.description"),
            &["en_US"],
            &[file("en_US", false, false)]
        ),
        [unresolved("name"), unresolved("description")]
    );
    assert_eq!(
        findings(
            "pack.name",
            Some("pack.description"),
            &["en_US"],
            &[file("en_US", true, false)]
        ),
        [unresolved("description")]
    );
    assert_eq!(
        findings(
            "pack.name",
            Some("pack.description"),
            &["en_US"],
            &[file("en_US", false, true)]
        ),
        [unresolved("name")]
    );
    // without `languages.json`, or without any file
    assert_eq!(
        findings("pack.name", Some("x"), &[], &[]),
        [unresolved("name")]
    );
    assert_eq!(
        findings("x", Some("pack.description"), &["en_US"], &[]),
        [unresolved("description")]
    );
}

#[test]
fn a_definition_in_any_listed_language_resolves_the_keyword() {
    assert_eq!(
        findings(
            "pack.name",
            Some("About"),
            &["en_US", "de_DE"],
            &[file("en_US", false, false), file("de_DE", true, false),]
        ),
        []
    );
}

#[test]
fn a_definition_in_an_unlisted_language_resolves_nothing() {
    assert_eq!(
        findings(
            "pack.name",
            Some("About"),
            &["en_US"],
            &[file("de_DE", true, false)]
        ),
        [unused("texts/de_DE.lang", "pack.name"), unresolved("name")]
    );
}

#[test]
fn without_the_description_only_the_name_is_resolved() {
    assert_eq!(
        findings(
            "pack.name",
            None,
            &["en_US"],
            &[file("en_US", false, false)]
        ),
        [unresolved("name")]
    );
    assert_eq!(
        findings("My Pack", None, &["en_US"], &[file("en_US", false, false)]),
        []
    );
}

#[test]
fn a_field_that_names_the_other_keyword_is_resolved_by_that_keyword() {
    // the header name is `pack.description`: the value shown is the file's `pack.description`
    assert_eq!(
        findings(
            "pack.description",
            Some("About"),
            &["en_US"],
            &[file("en_US", false, true)]
        ),
        []
    );
    assert_eq!(
        findings(
            "pack.description",
            Some("About"),
            &["en_US"],
            &[file("en_US", true, false)]
        ),
        [unused("texts/en_US.lang", "pack.name"), unresolved("name")]
    );
}

// ---- the walk ---------------------------------------------------------------------

#[test]
fn the_languages_walk_reads_the_two_lists_and_the_lang_files_only() {
    use bedrock_addon_core::load::WalkOrder;

    let walk = super::stack::LOADER.walk;
    let reads = |path: &str| walk.matches(&PackPath::new(path).unwrap()).is_some();
    for path in [
        "texts/languages.json",
        "texts/language_names.json",
        "texts/en_US.lang",
    ] {
        assert!(reads(path), "{path}");
    }
    // every other file below `texts/` and `Texts/` counts by its path alone (the
    // packs' file lists); before it the walk read them all.
    for path in [
        "texts/ja_JP/font/glyph_30.png",
        "texts/ja_JP/en_US.lang",
        "texts/notes.txt",
        "Texts/en_US.lang",
        "font/glyph_30.png",
        "manifest.json",
        "textures/texts/a.png",
        "TEXTS/en_US.lang",
        "texts",
    ] {
        assert!(!reads(path), "{path}");
    }
    assert_eq!(walk.order, WalkOrder::BottomUp);
    assert!(walk.app_data.is_empty());
}
