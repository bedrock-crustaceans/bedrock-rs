//! `texts/languages.json` and `texts/language_names.json`: the `languages/` and
//! `language_names/` fixtures, the locale fallback and the login code
//! check (V7).

use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, Severity};
use bedrock_addon_core::document::{Content, Document, Placed, Rejected};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_family_lang::Language;
use bedrock_addon_family_lang::languages::{LanguageNames, Languages};

use super::common::*;

/// Vanilla's assert text for a name entry of the wrong shape (V5).
const NAME_ENTRY: &str =
    "Languange name entries should be an array with 2 elements, a code and a name";

fn languages_bytes(bytes: &[u8]) -> Result<Document<Languages>, Rejected> {
    Languages::parse(
        &PackPath::from_static("texts/languages.json"),
        bytes,
        &LoadContext::detached(),
    )
}

/// `languages/<name>.json`, which must load.
fn languages(name: &str) -> Document<Languages> {
    languages_bytes(&fixture(&format!("languages/{name}.json")))
        .unwrap_or_else(|rejected| panic!("{name}: {:#?}", rejected.diagnostics))
}

fn names_bytes(bytes: &[u8]) -> Result<Document<LanguageNames>, Rejected> {
    LanguageNames::parse(
        &PackPath::from_static("texts/language_names.json"),
        bytes,
        &LoadContext::detached(),
    )
}

/// `language_names/<name>.json`, which must load.
fn names(name: &str) -> Document<LanguageNames> {
    names_bytes(&fixture(&format!("language_names/{name}.json")))
        .unwrap_or_else(|rejected| panic!("{name}: {:#?}", rejected.diagnostics))
}

fn codes(languages: &Languages) -> Vec<&str> {
    languages.codes.iter().map(Language::as_str).collect()
}

fn pairs(names: &LanguageNames) -> Vec<(&str, &str)> {
    names
        .entries
        .iter()
        .map(|(code, name)| (code.as_str(), name.as_str()))
        .collect()
}

fn lang_of(code: &str) -> Language {
    Language::new(code)
}

/// The path a finding names inside the file.
fn at(diagnostic: &Diagnostic) -> String {
    diagnostic.location.json_path.to_string()
}

fn only(diagnostics: &Diagnostics) -> &Diagnostic {
    assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
    diagnostics.iter().next().unwrap()
}

// ---- languages.json ------------------------------------------------------------------------

#[test]
fn the_vanilla_shape_reads_as_its_codes_in_order() {
    let document = languages("vanilla_shape");
    assert_eq!(codes(&document), ["en_US", "en_GB", "de_DE"]);
    assert!(document.diagnostics().is_empty());
    assert!(document.names.entries.is_empty());
    assert_eq!(document.origin().path.as_str(), "texts/languages.json");
    assert!(document.version().written.is_none());
}

#[test]
fn object_root_gives_its_member_values_in_key_order_deviation() {
    // vanilla iterates the object's values in key order and says nothing; ours: a warning
    // that the root is no array
    let document = languages("object_root");
    assert_eq!(codes(&document), ["en_US", "xx_YY"]);
    let finding = only(document.diagnostics());
    assert!(finding.is(&bedrock_addon_core::diagnostic::codes::INVALID_TYPE));
    assert_eq!(finding.severity, Severity::Info);
    assert_eq!(at(finding), "");
}

#[test]
fn scalar_root_gives_no_codes_deviation() {
    let document = languages("scalar_root");
    assert!(document.codes.is_empty());
    let finding = only(document.diagnostics());
    assert!(finding.is(&bedrock_addon_core::diagnostic::codes::INVALID_TYPE));
    assert_eq!(finding.severity, Severity::Info);
}

#[test]
fn number_element_is_skipped_with_a_warning_deviation() {
    // vanilla = `asString("")`: the code `""` is appended
    let document = languages("number_element");
    assert!(document.codes.is_empty());
    let finding = only(document.diagnostics());
    assert!(finding.is(&bedrock_addon_core::diagnostic::codes::INVALID_TYPE));
    assert_eq!(finding.severity, Severity::Info);
    assert_eq!(at(finding), "0");
}

#[test]
fn bool_element_is_skipped_with_a_warning_deviation() {
    // vanilla = `asString("")`: the code `"true"` is appended
    let document = languages("bool_element");
    assert!(document.codes.is_empty());
    let finding = only(document.diagnostics());
    assert!(finding.is(&bedrock_addon_core::diagnostic::codes::INVALID_TYPE));
    assert_eq!(finding.severity, Severity::Info);
}

#[test]
fn every_element_that_is_no_string_is_skipped_and_the_strings_are_kept() {
    let document =
        languages_bytes(br#"["en_US", null, 1.5, false, ["de_DE"], {"a": "fr_FR"}, "ja_JP"]"#)
            .unwrap();
    assert_eq!(codes(&document), ["en_US", "ja_JP"]);
    let found: Vec<String> = document.diagnostics().iter().map(at).collect();
    assert_eq!(found, ["1", "2", "3", "4", "5"]);
    assert!(document.diagnostics().iter().all(|d| {
        d.is(&bedrock_addon_core::diagnostic::codes::INVALID_TYPE) && d.severity == Severity::Info
    }));
}

#[test]
fn a_code_listed_twice_counts_once() {
    let document = languages_bytes(br#"["en_US", "de_DE", "en_US"]"#).unwrap();
    assert_eq!(codes(&document), ["en_US", "de_DE"]);
    assert!(document.diagnostics().is_empty());
    // codes are compared byte for byte
    let cased = languages_bytes(br#"["en_US", "en_us"]"#).unwrap();
    assert_eq!(codes(&cased), ["en_US", "en_us"]);
}

#[test]
fn a_code_of_another_form_is_accepted_with_an_info_deviation() {
    // vanilla = "accepted, no check" (L8)
    let document = languages_bytes(br#"["en_US", "english", "zz_ZZ", ""]"#).unwrap();
    assert_eq!(codes(&document), ["en_US", "english", "zz_ZZ", ""]);
    let found: Vec<&Diagnostic> = document.diagnostics().iter().collect();
    assert_eq!(found.len(), 2, "{found:#?}");
    for finding in &found {
        assert!(finding.is(&bedrock_addon_core::diagnostic::codes::INVALID_VALUE));
        assert_eq!(finding.severity, Severity::Info);
    }
    assert_eq!(at(found[0]), "1");
    assert_eq!(at(found[1]), "3");
}

#[test]
fn a_login_code_is_two_lower_case_letters_an_underscore_and_two_upper_case_letters() {
    for code in ["en_US", "aa_AA", "zz_ZZ"] {
        assert!(lang_of(code).is_valid_code(), "{code}");
    }
    for code in [
        "",
        "en",
        "en_U",
        "en_USA",
        "en-US",
        "enUS_",
        "En_US",
        "eN_US",
        "en_uS",
        "en_Us",
        "e1_US",
        "en_U1",
        "`n_US",
        "{n_US",
        "en_@S",
        "en_[S",
        "en US",
        "\u{e9}_US",
    ] {
        assert!(!lang_of(code).is_valid_code(), "{code:?}");
    }
}

#[test]
fn a_file_that_does_not_parse_is_rejected() {
    for bytes in [&b"[\"en_US\""[..], b"", b"not json", b"\xFF\xFE"] {
        let rejected = languages_bytes(bytes).expect_err("no JSON");
        let finding = only(&rejected.diagnostics);
        assert!(finding.is(&bedrock_addon_core::diagnostic::codes::SYNTAX));
        assert_eq!(finding.severity, Severity::Info);
        assert!(names_bytes(bytes).is_err());
    }
}

#[test]
fn comments_a_bom_and_trailing_commas_are_read() {
    let document =
        languages_bytes(b"\xEF\xBB\xBF// the languages\n[\"en_US\", /* and */ \"de_DE\",]")
            .unwrap();
    assert_eq!(codes(&document), ["en_US", "de_DE"]);
}

#[test]
fn the_codes_are_written_as_an_array_and_read_back() {
    let document = languages("vanilla_shape");
    let written = document.to_json().unwrap();
    assert_eq!(written, "[\n  \"en_US\",\n  \"en_GB\",\n  \"de_DE\"\n]\n");
    let again = languages_bytes(written.as_bytes()).unwrap();
    assert_eq!(again.value(), document.value());
    assert_eq!(again.to_json().unwrap(), written);
    assert_eq!(document.default_path().as_str(), "texts/languages.json");
    assert_eq!(Languages::default().to_json().unwrap(), "[]\n");
}

// ---- the locale fallback -------------------------------------------------------------------

fn listed(codes: &[&str]) -> Languages {
    Languages {
        codes: codes.iter().map(|code| lang_of(code)).collect(),
        names: LanguageNames::default(),
    }
}

#[test]
fn locale_fallback_takes_the_exact_code_then_the_prefix_then_the_base() {
    let languages = listed(&["en_US", "fr_FR", "fr_CA"]);
    let pick = |requested: &str| languages.locale_for(requested).map(Language::as_str);
    // `stack/locale_fallback`
    assert_eq!(pick("fr_BE"), Some("fr_FR"));
    assert_eq!(pick("xx_ZZ"), Some("en_US"));
    // the exact code wins over an earlier code with the same prefix
    assert_eq!(pick("fr_CA"), Some("fr_CA"));
    // without `_` the whole request is the prefix
    assert_eq!(pick("fr"), Some("fr_FR"));
    assert_eq!(languages.base(), Some(&lang_of("en_US")));
    assert!(languages.contains(&lang_of("fr_CA")));
    assert!(!languages.contains(&lang_of("fr_BE")));
}

#[test]
fn the_prefix_is_searched_anywhere_in_a_code() {
    let languages = listed(&["en_US", "de_DE"]);
    let pick = |requested: &str| languages.locale_for(requested).map(Language::as_str);
    // `find`, not a prefix match: `DE` is found in `de_DE`
    assert_eq!(pick("DE_xx"), Some("de_DE"));
    assert_eq!(pick("US"), Some("en_US"));
    // an empty prefix is found in the first code
    assert_eq!(pick("_DE"), Some("en_US"));
    assert_eq!(pick(""), Some("en_US"));
}

#[test]
fn an_empty_list_has_no_base_and_no_locale() {
    let languages = Languages::default();
    assert_eq!(languages.base(), None);
    assert_eq!(languages.locale_for("en_US"), None);
    assert!(!languages.contains(&lang_of("en_US")));
}

// ---- language_names.json -------------------------------------------------------------------

#[test]
fn the_vanilla_shape_reads_as_its_pairs_in_order() {
    let document = names("vanilla_shape");
    assert_eq!(
        pairs(&document),
        [
            ("en_US", "English (United States)"),
            ("en_GB", "English (United Kingdom)"),
            ("de_DE", "Deutsch (Deutschland)"),
        ]
    );
    assert!(document.diagnostics().is_empty());
    assert_eq!(
        document.get(&lang_of("de_DE")),
        Some("Deutsch (Deutschland)")
    );
    assert_eq!(document.get(&lang_of("fr_FR")), None);
}

/// The one L10 warning of `diagnostics`: its code and path.
fn name_entry_warning(diagnostics: &Diagnostics) -> (&DiagnosticCode, String) {
    let finding = only(diagnostics);
    assert_eq!(finding.severity, Severity::Info);
    (&finding.code, at(finding))
}

#[test]
fn three_elements_uses_the_first_two_with_vanillas_assert_text_deviation() {
    // vanilla = "handleAssert, first two elements used"
    let document = names("three_elements");
    assert_eq!(pairs(&document), [("en_US", "English")]);
    let (code, path) = name_entry_warning(document.diagnostics());
    assert_eq!(path, "0");
    assert!(
        code.is(&bedrock_addon_family_lang::codes::NAME_ENTRY_SHAPE_INVALID_VALUE),
        "{code:?}"
    );
    assert_eq!(code.to_string(), NAME_ENTRY);
}

#[test]
fn an_entry_that_is_no_array_is_skipped_deviation() {
    let document = names_bytes(br#"["en_US", ["de_DE", "Deutsch"], {"a": 1}]"#).unwrap();
    assert_eq!(pairs(&document), [("de_DE", "Deutsch")]);
    let found: Vec<&Diagnostic> = document.diagnostics().iter().collect();
    assert_eq!(found.len(), 2, "{found:#?}");
    assert_eq!(at(found[0]), "0");
    assert_eq!(at(found[1]), "2");
    for finding in found {
        assert_eq!(finding.severity, Severity::Info);
        assert!(
            finding
                .code
                .is(&bedrock_addon_family_lang::codes::NAME_ENTRY_SHAPE_INVALID_TYPE),
            "{:?}",
            finding.code
        );
    }
}

#[test]
fn a_short_entry_reads_its_missing_elements_as_empty() {
    let one = names_bytes(br#"[["en_US"]]"#).unwrap();
    assert_eq!(pairs(&one), [("en_US", "")]);
    assert!(
        name_entry_warning(one.diagnostics())
            .0
            .is(&bedrock_addon_family_lang::codes::NAME_ENTRY_SHAPE_INVALID_VALUE)
    );
    let none = names_bytes(br"[[]]").unwrap();
    assert_eq!(pairs(&none), [("", "")]);
    assert!(
        name_entry_warning(none.diagnostics())
            .0
            .is(&bedrock_addon_family_lang::codes::NAME_ENTRY_SHAPE_INVALID_VALUE)
    );
}

#[test]
fn the_elements_of_a_pair_are_read_as_vanilla_reads_any_value_as_a_string() {
    // `asString("")`: a boolean is `true` / `false`, every other non-string is empty
    let document =
        names_bytes(br#"[[true, false], ["de_DE", 5], [null, "x"], [["a"], {"b": 1}]]"#).unwrap();
    assert_eq!(
        pairs(&document),
        [("true", "false"), ("de_DE", ""), ("", "x"), ("", "")]
    );
    // Each element that is no string is told.
    let told: Vec<String> = document
        .diagnostics()
        .iter()
        .filter(|f| {
            f.is(&bedrock_addon_core::diagnostic::codes::INVALID_TYPE)
                && f.severity == Severity::Info
        })
        .map(|f| f.location.json_path.to_string())
        .collect();
    assert_eq!(told, ["0/0", "0/1", "1/1", "2/0", "3/0", "3/1"]);
    assert_eq!(document.diagnostics().len(), told.len());
}

#[test]
fn a_names_root_that_is_no_array_is_read_as_vanilla_iterates_it_deviation() {
    let object = names_bytes(br#"{"b": ["xx_YY", "Second"], "a": ["en_US", "First"]}"#).unwrap();
    assert_eq!(pairs(&object), [("en_US", "First"), ("xx_YY", "Second")]);
    let finding = only(object.diagnostics());
    assert!(finding.is(&bedrock_addon_core::diagnostic::codes::INVALID_TYPE));
    assert_eq!(finding.severity, Severity::Info);
    let scalar = names_bytes(b"5").unwrap();
    assert!(scalar.entries.is_empty());
    assert_eq!(scalar.diagnostics().len(), 1);
}

#[test]
fn empty_name_never_clears_a_name() {
    // `language_names/empty_name`: the higher pack's empty name keeps the lower
    // pack's `English`
    let bottom = names("empty_name_bottom");
    let top = names("empty_name_top");
    let mut merged = LanguageNames::default();
    merged.append(&bottom);
    merged.append(&top);
    assert_eq!(merged.get(&lang_of("en_US")), Some("English"));
    assert_eq!(pairs(&merged), [("en_US", "English")]);
}

#[test]
fn a_name_replaces_a_name_and_an_empty_name_only_makes_the_code_known() {
    let mut names = LanguageNames::default();
    names.set(lang_of("en_US"), "");
    assert_eq!(names.get(&lang_of("en_US")), Some(""));
    names.set(lang_of("en_US"), "English (US)");
    names.set(lang_of("de_DE"), "Deutsch");
    names.set(lang_of("en_US"), "English (United States)");
    names.set(lang_of("de_DE"), "");
    assert_eq!(
        pairs(&names),
        [("en_US", "English (United States)"), ("de_DE", "Deutsch")]
    );
    assert_eq!(names.get(&lang_of("fr_FR")), None);
}

#[test]
fn get_applies_the_rule_to_a_file_that_names_a_code_twice() {
    let document =
        names_bytes(br#"[["en_US", "First"], ["en_US", ""], ["en_US", "Last"], ["en_US", ""]]"#)
            .unwrap();
    assert_eq!(document.entries.len(), 4);
    assert_eq!(document.get(&lang_of("en_US")), Some("Last"));
    // of several names the last one, wherever the empty ones stand
    let several =
        names_bytes(br#"[["en_US", "First"], ["en_US", "Second"], ["en_US", "Third"]]"#).unwrap();
    assert_eq!(several.get(&lang_of("en_US")), Some("Third"));
    let trailing_empty = names_bytes(br#"[["en_US", "Only"], ["en_US", ""]]"#).unwrap();
    assert_eq!(trailing_empty.get(&lang_of("en_US")), Some("Only"));
}

#[test]
fn the_pairs_are_written_as_an_array_and_read_back() {
    let document = names("vanilla_shape");
    let written = document.to_json().unwrap();
    assert!(
        written.starts_with("[\n  [\n    \"en_US\",\n    \"English (United States)\"\n  ],"),
        "{written}"
    );
    let again = names_bytes(written.as_bytes()).unwrap();
    assert_eq!(again.value(), document.value());
    assert_eq!(again.to_json().unwrap(), written);
    assert_eq!(
        document.default_path().as_str(),
        "texts/language_names.json"
    );
}

#[test]
fn languages_shows_the_name_of_a_code() {
    let mut languages = listed(&["en_US", "de_DE"]);
    languages.names.set(lang_of("en_US"), "English");
    assert_eq!(languages.name(&lang_of("en_US")), Some("English"));
    assert_eq!(languages.name(&lang_of("de_DE")), None);
}

/// an element of a pair that is no string is read as vanilla reads it and told at the
/// element; a pair of two strings says nothing.
#[test]
fn a_name_or_code_that_is_no_string_is_read_as_vanilla_reads_it_and_told() {
    let document =
        names_bytes(br#"[["en_US", 7], [true, "Deutsch"], ["fr_FR", "Francais"]]"#).unwrap();
    let told: Vec<(&str, Severity, String)> = document
        .diagnostics()
        .iter()
        .map(|f| (f.code.name(), f.severity, f.location.json_path.to_string()))
        .collect();
    assert_eq!(
        told,
        [
            ("InvalidType", Severity::Info, "0/1".to_owned()),
            ("InvalidType", Severity::Info, "1/0".to_owned())
        ]
    );
    let entries: Vec<(&str, &str)> = document
        .value()
        .entries
        .iter()
        .map(|(code, name)| (code.as_str(), name.as_str()))
        .collect();
    assert_eq!(
        entries,
        [("en_US", ""), ("true", "Deutsch"), ("fr_FR", "Francais")]
    );
}
