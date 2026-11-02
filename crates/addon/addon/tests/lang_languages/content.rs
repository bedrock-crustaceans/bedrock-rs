//! `LangFile` as a file kind: `Content::parse` of one file, its place in a pack, and the
//! writer the trait names.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::document::{Content, Document, Placed};
use bedrock_addon::families::lang::{LangFile, Language, Line};
use bedrock_addon::families::languages::{LanguageNames, Languages};
use bedrock_addon::json::WriteStyle;
use bedrock_addon::load::LoadContext;
use bedrock_addon::pack::PackSide;
use bedrock_addon::storage::PackPath;

use super::common::*;

fn parse(path: &str, bytes: &[u8]) -> Document<LangFile> {
    LangFile::parse(
        &PackPath::new(path).unwrap(),
        bytes,
        &LoadContext::detached(),
    )
    .expect("a lang file is never rejected")
}

#[test]
fn the_language_of_a_parsed_file_is_the_code_in_its_path() {
    let document = parse("texts/de_DE.lang", b"a=b\n");
    assert_eq!(document.language, Language::new("de_DE"));
    assert_eq!(document.lines, [Line::entry("a", "b")]);
    assert_eq!(document.origin().path.as_str(), "texts/de_DE.lang");
    // an unversioned family: nothing was written and nothing upgraded
    assert!(document.version().written.is_none());
    assert!(!document.version().upgraded);
}

#[test]
fn a_file_anywhere_else_is_read_as_the_language_its_name_gives() {
    assert_eq!(parse("fr_FR.lang", b"").language, Language::new("fr_FR"));
    assert_eq!(
        parse("texts/ja_JP/texts/en_US.lang", b"").language,
        Language::new("en_US")
    );
    assert_eq!(parse("notes.txt", b"").language, Language::new("notes.txt"));
}

#[test]
fn a_file_with_bad_lines_still_loads_with_its_findings_at_the_file() {
    let document = parse("texts/en_US.lang", b"broken\nkey = v\n");
    assert_eq!(document.lines.len(), 2);
    let found: Vec<_> = document.diagnostics().iter().collect();
    assert_eq!(found.len(), 2);
    // vanilla's no-`=` warning, then the L2 information (ours)
    assert_eq!(found[0].severity, Severity::Warning);
    assert_eq!(found[1].severity, Severity::Info);
    for finding in found {
        assert_eq!(
            finding.location.path.as_ref().map(PackPath::as_str),
            Some("texts/en_US.lang")
        );
        assert!(finding.location.pack.is_some());
    }
}

#[test]
fn the_trait_writer_writes_the_lang_text() {
    let document = parse("texts/en_US.lang", b"## c\r\na=x\\ny\r\n");
    let mut out = Vec::new();
    document
        .write_json(&mut out, WriteStyle::default())
        .unwrap();
    assert_eq!(out, b"## c\na=x\\ny\n");
    assert_eq!(document.to_json().unwrap(), "## c\na=x\\ny\n");
    assert_eq!(document.to_text().unwrap(), "## c\na=x\\ny\n");
}

#[test]
#[expect(
    clippy::case_sensitive_file_extension_comparisons,
    reason = "the suffix is the exact `.lang` the writer appends, and `Path::extension` sees no extension in `.lang`"
)]
fn a_lang_file_is_placed_at_texts_code_lang() {
    let file = LangFile::new(Language::new("de_DE"));
    assert_eq!(file.default_path().as_str(), "texts/de_DE.lang");
    // a code that cannot be a file name still gives a path in `texts/`
    for code in ["..", "a:b", ""] {
        let path = LangFile::new(Language::new(code)).default_path();
        assert!(path.as_str().starts_with("texts/"), "{code:?}: {path}");
        assert!(path.as_str().ends_with(".lang"), "{code:?}: {path}");
    }
}

#[test]
fn the_three_files_are_read_on_both_sides() {
    let both = [PackSide::Behavior, PackSide::Resource];
    assert_eq!(<LangFile as Content>::SIDES, both);
    assert_eq!(<Languages as Content>::SIDES, both);
    assert_eq!(<LanguageNames as Content>::SIDES, both);
    assert_eq!(<LangFile as Content>::NAME, "lang file");
    assert_eq!(<Languages as Content>::NAME, "language list");
    assert_eq!(<LanguageNames as Content>::NAME, "language names");
}

#[test]
fn a_parsed_file_round_trips_through_the_trait() {
    let bytes = fixture("lang/ut_multiline_error.lang");
    let document = parse("texts/en_US.lang", &bytes);
    let written = document.to_json().unwrap();
    let again = parse("texts/en_US.lang", written.as_bytes());
    assert_eq!(again.value(), document.value());
    assert_eq!(written.as_bytes(), bytes);
}
