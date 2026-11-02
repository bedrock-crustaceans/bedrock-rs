//! `languages()` and `translations(&language)` on a stack: the `stack/` fixture rows,
//! the merge rules across the pack stack, the pack keywords
//! and the stack-level lints L5–L7 and L12.

use bedrock_addon::diagnostic::{Diagnostic, DiagnosticCode, Severity};
use bedrock_addon::families::lang::Language;
use bedrock_addon::pack::{Behavior, Pack, PackStack, Resource, Side};

use super::common::*;

fn lang_of(code: &str) -> Language {
    Language::new(code)
}

/// The resource stack of fixture case `stack/<case>`.
fn case(case: &str) -> PackStack<Resource> {
    resource_stack(fixture_packs(&format!("stack/{case}"), "resources"))
}

/// A vanilla-like bottom pack listing `codes`, with these `.lang` files.
fn base_pack(codes: &str, langs: &[(&str, &str)]) -> Pack {
    let mut files = vec![("texts/languages.json", codes)];
    files.extend(langs.iter().copied());
    pack_with(1, "resources", &files)
}

fn codes<S: Side>(stack: &PackStack<S>) -> Vec<String> {
    stack
        .languages()
        .codes
        .iter()
        .map(|code| code.as_str().to_owned())
        .collect()
}

/// The findings of `languages()` with `code`'s name at `severity`, as `(pack index, path,
/// breadcrumb)`.
fn language_findings<S: Side>(
    stack: &PackStack<S>,
    code: &str,
    severity: Severity,
) -> Vec<(Option<usize>, String, String)> {
    stack
        .languages()
        .diagnostics()
        .filter(|d| d.code.name() == code && d.severity == severity)
        .map(place)
        .collect()
}

/// Where a finding is: `(pack index, path, breadcrumb)`.
fn place(diagnostic: &Diagnostic) -> (Option<usize>, String, String) {
    (
        diagnostic
            .location
            .pack
            .as_ref()
            .and_then(bedrock_addon::pack::PackRef::index),
        diagnostic
            .location
            .path
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default(),
        diagnostic.location.json_path.to_string(),
    )
}

// ---- the fixture rows ----------------------------------------------------------------------

#[test]
fn top_wins_per_key() {
    let stack = case("top_wins");
    let english = stack.translations(&lang_of("en_US")).expect("listed");
    assert_eq!(english.get("a"), Some("2"));
    assert_eq!(english.get("b"), Some("1"));
    // the winning pack of each key: the top pack is index 0
    assert_eq!(
        english.origin(&"a".to_owned()).unwrap().pack.index(),
        Some(0)
    );
    assert_eq!(
        english.origin(&"b".to_owned()).unwrap().pack.index(),
        Some(1)
    );
    assert_eq!(english.origin(&"c".to_owned()), None);
    // one layer per pack, top first, each with what its own file gave
    let layers = english.layers();
    assert_eq!(layers.len(), 2);
    assert_eq!(layers[0].origin().pack.index(), Some(0));
    assert_eq!(layers[0].origin().path.as_str(), "texts/en_US.lang");
    assert_eq!(layers[0].get("a"), Some("2"));
    assert_eq!(layers[0].get("b"), None);
    assert_eq!(layers[1].get("a"), Some("1"));
    assert_eq!(english.diagnostics().count(), 0);
}

#[test]
fn base_fallback_starts_a_language_from_the_base_table() {
    let stack = case("base_fallback");
    let languages = stack.languages();
    assert_eq!(languages.base(), Some(&lang_of("en_US")));
    let english = stack.translations(&lang_of("en_US")).unwrap();
    let german = stack.translations(&lang_of("de_DE")).unwrap();
    // the stack's table of a language holds its own files only
    assert_eq!(german.get("a"), None);
    let shown = german.with_base(english);
    assert_eq!(shown.get("a"), Some("1"));
    assert_eq!(shown.get("b"), Some("zwei"));
}

#[test]
fn new_language_is_appended_after_the_lower_codes_and_an_unlisted_file_is_never_loaded() {
    let stack = case("new_language");
    assert_eq!(codes(&stack), ["en_US", "xx_XX"]);
    let languages = stack.languages();
    // the pack that listed a code first is its origin
    assert_eq!(
        languages.origin(&lang_of("en_US")).unwrap().pack.index(),
        Some(1)
    );
    assert_eq!(
        languages.origin(&lang_of("xx_XX")).unwrap().pack.index(),
        Some(0)
    );
    let english = stack.translations(&lang_of("en_US")).unwrap();
    let custom = stack.translations(&lang_of("xx_XX")).unwrap();
    assert_eq!(custom.get("a"), Some("x"));
    assert_eq!(custom.with_base(english).get("b"), Some("1"));
    // `yy_YY.lang` is in no `languages.json`
    assert!(stack.translations(&lang_of("yy_YY")).is_none());
}

#[test]
fn a_lang_file_of_an_unlisted_code_is_reported_deviation() {
    // L7, vanilla = "silent"
    let stack = case("new_language");
    assert_eq!(
        language_findings(&stack, "IgnoredFile", Severity::Info),
        [(Some(0), "texts/yy_YY.lang".to_owned(), String::new())]
    );
    assert_eq!(stack.languages().diagnostics().count(), 1);
    // the stack report lists what the loaded collections found
    let report = stack.report();
    assert!(
        report
            .iter()
            .any(|d| matches!(d.code, DiagnosticCode::IgnoredFile { text: None })),
        "the report lacks the finding"
    );
}

#[test]
fn locale_fallback_through_the_stack() {
    let stack = case("locale_fallback");
    let languages = stack.languages();
    assert_eq!(languages.locale_for("fr_BE"), Some(&lang_of("fr_FR")));
    assert_eq!(languages.locale_for("xx_ZZ"), Some(&lang_of("en_US")));
}

/// What the one pack of `stack` shows for its header name and description in `en_US`.
fn shown<S: Side>(stack: &PackStack<S>, pack: &Pack) -> (String, String) {
    let english = lang_of("en_US");
    let tables: Vec<_> = stack
        .translations(&english)
        .map(|table| (&english, table.value()))
        .into_iter()
        .collect();
    let id = stack.packs().next().unwrap().id().clone();
    let header = &pack.manifest().header;
    (
        stack
            .pack_keyword_value(&header.name, &id, &tables)
            .to_owned(),
        stack
            .pack_keyword_value(
                header.description.as_deref().unwrap_or_default(),
                &id,
                &tables,
            )
            .to_owned(),
    )
}

#[test]
fn pack_name_is_localised_by_the_packs_own_lang_file() {
    let pack = fixture_packs("stack/pack_name", "resources").remove(1);
    assert_eq!(pack.manifest().header.name, "pack.name");
    let stack = resource_stack(vec![pack.clone()]);
    assert_eq!(
        shown(&stack, &pack),
        ("My Pack".to_owned(), "Mine".to_owned())
    );
    // the keywords never reach the language's own keys
    let english = stack.translations(&lang_of("en_US")).unwrap();
    assert_eq!(english.get("pack.name"), None);
    assert_eq!(english.get("pack.description"), None);
    assert_eq!(
        english.get("0f1e2d3c-4b5a-4978-8695-a4b3c2d1e0011.2.3pack.name"),
        Some("My Pack")
    );
    assert_eq!(english.strings.len(), 2);
    assert_eq!(stack.languages().diagnostics().count(), 0);
}

#[test]
fn pack_name_without_languages_json_shows_the_literal_keyword() {
    let pack = fixture_packs("stack/pack_name", "resources").remove(0);
    let stack = resource_stack(vec![pack.clone()]);
    assert_eq!(
        shown(&stack, &pack),
        ("pack.name".to_owned(), "pack.description".to_owned())
    );
    assert!(stack.translations(&lang_of("en_US")).is_none());
}

#[test]
fn pack_name_bp_is_localised_in_a_behavior_pack_too() {
    let pack = fixture_packs("stack/pack_name_bp", "data").remove(0);
    let stack: PackStack<Behavior> = behavior_stack(vec![pack.clone()]);
    assert_eq!(
        shown(&stack, &pack),
        ("My Pack".to_owned(), "Mine".to_owned())
    );
}

#[test]
fn texts_capital_is_not_read_by_the_stack_loader() {
    let stack = case("texts_capital");
    let english = stack.translations(&lang_of("en_US")).unwrap();
    assert_eq!(english.get("a"), Some("1"));
    assert_eq!(english.layers().len(), 1);
}

#[test]
fn a_texts_folder_with_a_capital_t_is_reported_once_per_pack_deviation() {
    // L12, vanilla = "only the single-pack appendLanguageStrings path falls back to Texts/"
    let stack = case("texts_capital");
    assert_eq!(
        language_findings(&stack, "IgnoredFile", Severity::Info),
        [(Some(0), "Texts".to_owned(), String::new())]
    );
    let two_files = resource_stack(vec![pack_with(
        1,
        "resources",
        &[
            ("Texts/en_US.lang", "a=1\n"),
            ("Texts/languages.json", "[\"en_US\"]"),
        ],
    )]);
    assert_eq!(
        language_findings(&two_files, "IgnoredFile", Severity::Info).len(),
        1
    );
    // its `languages.json` is not read either
    assert!(codes(&two_files).is_empty());
}

// ---- languages ----------------------------------------------------------------------------

#[test]
fn the_codes_are_the_union_in_first_seen_order_lowest_pack_first() {
    let stack = resource_stack(vec![
        pack_with(
            3,
            "resources",
            &[("texts/languages.json", r#"["zz_ZZ", "en_US", "de_DE"]"#)],
        ),
        pack_with(
            2,
            "resources",
            &[("texts/languages.json", r#"["de_DE", "fr_FR"]"#)],
        ),
        base_pack(r#"["en_US", "de_DE"]"#, &[]),
    ]);
    assert_eq!(codes(&stack), ["en_US", "de_DE", "fr_FR", "zz_ZZ"]);
    // a pack cannot move the base language
    assert_eq!(stack.languages().base(), Some(&lang_of("en_US")));
    // one layer per file, top first
    let layers = stack.languages().layers();
    assert_eq!(layers.len(), 3);
    assert_eq!(layers[0].origin().pack.index(), Some(0));
    assert_eq!(layers[0].codes.len(), 3);
    assert_eq!(layers[2].origin().pack.index(), Some(2));
}

#[test]
fn empty_name_in_a_higher_pack_keeps_the_lower_packs_name() {
    let top = String::from_utf8(fixture("language_names/empty_name_top.json")).unwrap();
    let bottom = String::from_utf8(fixture("language_names/empty_name_bottom.json")).unwrap();
    let stack = resource_stack(vec![
        pack_with(2, "resources", &[("texts/language_names.json", &top)]),
        pack_with(
            1,
            "resources",
            &[
                ("texts/languages.json", r#"["en_US"]"#),
                ("texts/language_names.json", &bottom),
            ],
        ),
    ]);
    assert_eq!(stack.languages().name(&lang_of("en_US")), Some("English"));
    assert_eq!(stack.languages().layers().len(), 3);
}

#[test]
fn the_highest_packs_name_wins() {
    let stack = resource_stack(vec![
        pack_with(
            2,
            "resources",
            &[(
                "texts/language_names.json",
                r#"[["en_US", "English (United States)"]]"#,
            )],
        ),
        pack_with(
            1,
            "resources",
            &[
                ("texts/languages.json", r#"["en_US"]"#),
                (
                    "texts/language_names.json",
                    r#"[["en_US", "English (US)"], ["de_DE", "Deutsch"]]"#,
                ),
            ],
        ),
    ]);
    let languages = stack.languages();
    assert_eq!(
        languages.name(&lang_of("en_US")),
        Some("English (United States)")
    );
    assert_eq!(languages.name(&lang_of("de_DE")), Some("Deutsch"));
    assert_eq!(languages.names.entries.len(), 2);
}

#[test]
fn a_copy_that_does_not_parse_is_skipped_and_the_other_packs_copies_count() {
    // V6: vanilla skips it silently; here it is the copy's error
    let stack = resource_stack(vec![
        pack_with(
            2,
            "resources",
            &[
                ("texts/languages.json", r#"["xx_XX""#),
                ("texts/language_names.json", "nope"),
            ],
        ),
        base_pack(r#"["en_US"]"#, &[]),
    ]);
    assert_eq!(codes(&stack), ["en_US"]);
    assert_eq!(stack.languages().layers().len(), 1);
    let errors = language_findings(&stack, "MalformedInput", Severity::Info);
    assert_eq!(
        errors,
        [
            (
                Some(0),
                "texts/language_names.json".to_owned(),
                String::new()
            ),
            (Some(0), "texts/languages.json".to_owned(), String::new()),
        ]
    );
}

#[test]
fn a_stack_without_languages_json_lists_nothing_and_has_no_translations() {
    let stack = resource_stack(vec![pack_with(
        1,
        "resources",
        &[("texts/en_US.lang", "a=1\n")],
    )]);
    assert!(codes(&stack).is_empty());
    assert!(stack.translations(&lang_of("en_US")).is_none());
    // L7: the file is never loaded
    assert_eq!(
        language_findings(&stack, "IgnoredFile", Severity::Info),
        [(Some(0), "texts/en_US.lang".to_owned(), String::new())]
    );
}

#[test]
fn the_findings_of_a_languages_file_come_with_its_pack_and_path() {
    let stack = resource_stack(vec![base_pack(r#"["en_US", 5, "english"]"#, &[])]);
    assert_eq!(
        language_findings(&stack, "InvalidType", Severity::Info),
        [(Some(0), "texts/languages.json".to_owned(), "1".to_owned())]
    );
    assert_eq!(
        language_findings(&stack, "InvalidValue", Severity::Info),
        [(Some(0), "texts/languages.json".to_owned(), "2".to_owned())]
    );
}

// ---- translations -------------------------------------------------------------------------

#[test]
fn only_lang_files_directly_in_texts_are_a_languages_files() {
    let stack = resource_stack(vec![base_pack(
        r#"["en_US"]"#,
        &[
            ("texts/en_US.lang", "a=1\n"),
            ("texts/en_US.txt", "a=2\n"),
            ("texts/en_US.LANG", "a=3\n"),
            ("texts/en_US/en_US.lang", "a=4\n"),
            ("texts/en_US.lang.bak", "a=5\n"),
        ],
    )]);
    let english = stack.translations(&lang_of("en_US")).unwrap();
    assert_eq!(english.get("a"), Some("1"));
    assert_eq!(english.layers().len(), 1);
}

#[test]
fn a_lang_files_findings_come_with_its_pack_and_path() {
    let stack = resource_stack(vec![
        pack_with(2, "resources", &[("texts/en_US.lang", "broken\na=1\n")]),
        base_pack(r#"["en_US"]"#, &[("texts/en_US.lang", "a = 0\n")]),
    ]);
    let english = stack.translations(&lang_of("en_US")).unwrap();
    let found: Vec<_> = english.diagnostics().map(place).collect();
    // in walk order: the lowest pack first
    assert_eq!(
        found,
        [
            (Some(1), "texts/en_US.lang".to_owned(), "a ".to_owned()),
            (Some(0), "texts/en_US.lang".to_owned(), "broken".to_owned()),
        ]
    );
    assert_eq!(english.get("a"), Some("1"));
    assert_eq!(english.get("a "), Some(" 0"));
}

#[test]
fn a_tilde_in_any_packs_file_sets_the_merged_flag() {
    let stack = resource_stack(vec![
        pack_with(2, "resources", &[("texts/en_US.lang", "b=plain\n")]),
        base_pack(r#"["en_US"]"#, &[("texts/en_US.lang", "a=x~LINEBREAK~y\n")]),
    ]);
    let english = stack.translations(&lang_of("en_US")).unwrap();
    assert!(english.may_contain_identifier);
    let plain = resource_stack(vec![base_pack(
        r#"["en_US"]"#,
        &[("texts/en_US.lang", "a=x\n")],
    )]);
    assert!(
        !plain
            .translations(&lang_of("en_US"))
            .unwrap()
            .may_contain_identifier
    );
}

#[test]
fn a_listed_language_without_a_file_is_an_empty_table() {
    let stack = resource_stack(vec![base_pack(
        r#"["en_US", "de_DE"]"#,
        &[("texts/en_US.lang", "a=1\n")],
    )]);
    let german = stack.translations(&lang_of("de_DE")).expect("listed");
    assert!(german.strings.is_empty());
    assert!(german.layers().is_empty());
}

// ---- pack keywords ------------------------------------------------------------------------

/// A resource pack whose header name is `pack.name`, with these text files.
fn keyword_pack(n: u128, files: &[(&str, &str)]) -> Pack {
    let mut all = vec![(
        "manifest.json".to_owned(),
        manifest(n, "resources", "pack.name").into_bytes(),
    )];
    all.extend(
        files
            .iter()
            .map(|(path, text)| ((*path).to_owned(), text.as_bytes().to_vec())),
    );
    pack_of(n, "resources", all)
}

/// What the top pack of `stack` shows for `text` when the language is `current`.
fn keyword(stack: &PackStack<Resource>, text: &str, current: &str) -> String {
    let languages = stack.languages();
    let current = lang_of(current);
    let base = languages
        .base()
        .cloned()
        .unwrap_or_else(|| lang_of("en_US"));
    let tables: Vec<_> = [&current, &base]
        .into_iter()
        .filter_map(|code| Some((code, stack.translations(code)?.value())))
        .collect();
    let id = stack.packs().next().unwrap().id().clone();
    stack.pack_keyword_value(text, &id, &tables).to_owned()
}

#[test]
fn a_keyword_falls_back_from_the_current_language_to_the_base() {
    let stack = resource_stack(vec![keyword_pack(
        2,
        &[
            ("texts/languages.json", r#"["en_US", "de_DE", "fr_FR"]"#),
            ("texts/en_US.lang", "pack.name=My Pack\n"),
            ("texts/de_DE.lang", "pack.name=Mein Paket\n"),
            ("texts/fr_FR.lang", "other=x\n"),
        ],
    )]);
    assert_eq!(keyword(&stack, "pack.name", "de_DE"), "Mein Paket");
    assert_eq!(keyword(&stack, "pack.name", "fr_FR"), "My Pack");
    // no table has the description: the keyword itself is shown
    assert_eq!(
        keyword(&stack, "pack.description", "de_DE"),
        "pack.description"
    );
    // a text that is no keyword is shown as it is
    assert_eq!(keyword(&stack, "Plain Name", "de_DE"), "Plain Name");
    assert_eq!(keyword(&stack, "Pack.Name", "de_DE"), "Pack.Name");
}

#[test]
fn a_keyword_is_read_only_in_the_languages_the_pack_lists_itself() {
    // the pack ships `de_DE.lang` but lists only `en_US`; `de_DE` is listed by the pack below
    let stack = resource_stack(vec![
        keyword_pack(
            2,
            &[
                ("texts/languages.json", r#"["en_US"]"#),
                ("texts/en_US.lang", "pack.name=My Pack\n"),
                ("texts/de_DE.lang", "pack.name=Mein Paket\n"),
            ],
        ),
        base_pack(r#"["en_US", "de_DE"]"#, &[]),
    ]);
    assert_eq!(keyword(&stack, "pack.name", "de_DE"), "My Pack");
}

#[test]
fn a_keyword_is_the_packs_own_never_another_packs() {
    let stack = resource_stack(vec![
        keyword_pack(2, &[("texts/languages.json", r#"["en_US"]"#)]),
        keyword_pack(
            1,
            &[
                ("texts/languages.json", r#"["en_US"]"#),
                ("texts/en_US.lang", "pack.name=The Other Pack\n"),
            ],
        ),
    ]);
    assert_eq!(keyword(&stack, "pack.name", "en_US"), "pack.name");
    // both packs' keywords are in the table, each under its own prefix
    let english = stack.translations(&lang_of("en_US")).unwrap();
    assert_eq!(english.strings.len(), 1);
    let lower = stack.packs().nth(1).unwrap();
    assert_eq!(
        english.pack_keyword(lower.id(), "pack.name"),
        Some("The Other Pack")
    );
}

// ---- L5, L6: keywords the loader can judge ----------------------------------------------------

#[test]
fn a_name_keyword_no_listed_lang_file_defines_is_reported_deviation() {
    // L6, vanilla = "displays the literal keyword"
    let missing_key = resource_stack(vec![keyword_pack(
        1,
        &[
            ("texts/languages.json", r#"["en_US"]"#),
            ("texts/en_US.lang", "other=x\n"),
        ],
    )]);
    assert_eq!(
        language_findings(&missing_key, "UnresolvedLangKey", Severity::Info),
        [(
            Some(0),
            "manifest.json".to_owned(),
            "header/name".to_owned()
        )]
    );
    let no_languages = resource_stack(vec![keyword_pack(1, &[])]);
    assert_eq!(
        language_findings(&no_languages, "UnresolvedLangKey", Severity::Info).len(),
        1
    );
}

#[test]
fn a_name_keyword_a_listed_lang_file_defines_is_not_reported() {
    let stack = resource_stack(vec![keyword_pack(
        1,
        &[
            ("texts/languages.json", r#"["en_US", "de_DE"]"#),
            ("texts/de_DE.lang", "pack.name=Mein Paket\n"),
        ],
    )]);
    assert!(language_findings(&stack, "UnresolvedLangKey", Severity::Info).is_empty());
    // a plain name is no keyword
    let plain = resource_stack(vec![base_pack(r#"["en_US"]"#, &[])]);
    assert_eq!(plain.languages().diagnostics().count(), 0);
}

#[test]
fn a_keyword_in_a_lang_file_the_pack_does_not_list_is_reported_deviation() {
    // L5, vanilla = "silent": the pack's keywords are read only for its own listed codes
    let stack = resource_stack(vec![
        keyword_pack(
            2,
            &[
                ("texts/languages.json", r#"["en_US"]"#),
                ("texts/en_US.lang", "pack.name=My Pack\n"),
                (
                    "texts/de_DE.lang",
                    "pack.name=Mein Paket\npack.description=Meins\n",
                ),
            ],
        ),
        base_pack(r#"["en_US", "de_DE"]"#, &[]),
    ]);
    assert_eq!(
        language_findings(&stack, "IgnoredMember", Severity::Info),
        [
            (
                Some(0),
                "texts/de_DE.lang".to_owned(),
                "pack.name".to_owned()
            ),
            (
                Some(0),
                "texts/de_DE.lang".to_owned(),
                "pack.description".to_owned()
            ),
        ]
    );
}

/// A resource pack with this header `name` and `description` member (`None`: no member), with
/// these text files.
fn described_pack(n: u128, name: &str, description: Option<&str>, files: &[(&str, &str)]) -> Pack {
    let uuid = uuid::Uuid::from_u128(n);
    let module = uuid::Uuid::from_u128(n ^ (0xabcd_u128 << 64));
    let description = description
        .map(|text| format!(r#""description": "{text}", "#))
        .unwrap_or_default();
    let manifest = format!(
        r#"{{"format_version": 2, "header": {{"name": "{name}", {description}"uuid": "{uuid}", "version": [1, 0, 0], "min_engine_version": [1, 21, 0]}}, "modules": [{{"type": "resources", "uuid": "{module}", "version": [1, 0, 0]}}]}}"#
    );
    let mut all = vec![("manifest.json".to_owned(), manifest.into_bytes())];
    all.extend(
        files
            .iter()
            .map(|(path, text)| ((*path).to_owned(), text.as_bytes().to_vec())),
    );
    pack_of(n, "resources", all)
}

const LISTS_ENGLISH: (&str, &str) = ("texts/languages.json", r#"["en_US"]"#);

#[test]
fn a_keyword_neither_header_field_uses_is_reported_deviation() {
    // L5, vanilla = "silent": the loader sees the header description
    let both = "pack.name=Unused\npack.description=Unused too\n";
    for description in [Some("About this pack"), Some(""), None] {
        let stack = resource_stack(vec![described_pack(
            1,
            "My Pack",
            description,
            &[LISTS_ENGLISH, ("texts/en_US.lang", both)],
        )]);
        assert_eq!(
            language_findings(&stack, "IgnoredMember", Severity::Info),
            [
                (
                    Some(0),
                    "texts/en_US.lang".to_owned(),
                    "pack.name".to_owned()
                ),
                (
                    Some(0),
                    "texts/en_US.lang".to_owned(),
                    "pack.description".to_owned()
                ),
            ],
            "{description:?}"
        );
        assert_eq!(stack.languages().diagnostics().count(), 2);
    }
}

#[test]
fn a_description_keyword_is_used_by_the_manifest() {
    let stack = resource_stack(vec![described_pack(
        1,
        "My Pack",
        Some("pack.description"),
        &[
            LISTS_ENGLISH,
            (
                "texts/en_US.lang",
                "pack.name=Unused\npack.description=Mine\n",
            ),
        ],
    )]);
    // only the keyword no header field names is unused
    assert_eq!(
        language_findings(&stack, "IgnoredMember", Severity::Info),
        [(
            Some(0),
            "texts/en_US.lang".to_owned(),
            "pack.name".to_owned()
        )]
    );
    assert_eq!(stack.languages().diagnostics().count(), 1);
    // both header fields in use: nothing to report
    let both = resource_stack(vec![described_pack(
        1,
        "pack.name",
        Some("pack.description"),
        &[
            LISTS_ENGLISH,
            (
                "texts/en_US.lang",
                "pack.name=My Pack\npack.description=Mine\n",
            ),
        ],
    )]);
    assert_eq!(both.languages().diagnostics().count(), 0);
}

#[test]
fn a_description_keyword_no_listed_lang_file_defines_is_reported_deviation() {
    // L6 for the description, vanilla = "displays the literal keyword"
    let stack = resource_stack(vec![described_pack(
        1,
        "pack.name",
        Some("pack.description"),
        &[LISTS_ENGLISH, ("texts/en_US.lang", "pack.name=My Pack\n")],
    )]);
    assert_eq!(
        language_findings(&stack, "UnresolvedLangKey", Severity::Info),
        [(
            Some(0),
            "manifest.json".to_owned(),
            "header/description".to_owned()
        )]
    );
    assert_eq!(stack.languages().diagnostics().count(), 1);
    // a description that is no keyword, or no description, needs no definition
    for description in [Some("About this pack"), None] {
        let plain = resource_stack(vec![described_pack(
            1,
            "pack.name",
            description,
            &[LISTS_ENGLISH, ("texts/en_US.lang", "pack.name=My Pack\n")],
        )]);
        assert_eq!(
            plain.languages().diagnostics().count(),
            0,
            "{description:?}"
        );
    }
}

// ---- a full load ---------------------------------------------------------------------------

#[test]
fn a_full_load_answers_what_the_lazy_accessors_answer() {
    use bedrock_addon::pack::StackSet;

    let packs = |kind: &str| fixture_packs("stack/new_language", kind);
    let set = StackSet {
        behavior: behavior_stack(packs("data")),
        resource: resource_stack(packs("resources")),
    };
    let report = set.load_all();
    // both stacks report the unlisted `yy_YY.lang`
    let ignored = report
        .iter()
        .filter(|d| matches!(d.code, DiagnosticCode::IgnoredFile { text: None }))
        .count();
    assert_eq!(ignored, 2);
    assert!(!report.has_errors());

    let lazy = resource_stack(packs("resources"));
    for code in ["en_US", "xx_XX"] {
        let loaded = set.resource.translations(&lang_of(code)).expect("listed");
        let on_demand = lazy.translations(&lang_of(code)).expect("listed");
        assert_eq!(loaded.value(), on_demand.value(), "{code}");
        assert_eq!(loaded.layers().len(), on_demand.layers().len(), "{code}");
        let behavior = set.behavior.translations(&lang_of(code)).expect("listed");
        assert_eq!(behavior.value(), on_demand.value(), "{code}");
    }
    // a full load parses `yy_YY.lang` too, and the accessor still answers listed codes only
    assert!(set.resource.translations(&lang_of("yy_YY")).is_none());
    assert_eq!(set.resource.languages().value(), lazy.languages().value());
}

// ---- rules the mutation pass found no test for ------------------------------------

#[test]
fn a_text_that_is_no_keyword_is_never_looked_up() {
    use bedrock_addon::families::lang::{Translations, pack_key_prefix};

    let stack = resource_stack(vec![keyword_pack(
        2,
        &[("texts/languages.json", r#"["en_US"]"#)],
    )]);
    let id = stack.packs().next().unwrap().id().clone();
    // a table built by hand that holds the pack's prefix in front of an ordinary text
    let mut table = Translations::new();
    let prefix = pack_key_prefix(&id);
    table
        .strings
        .insert(format!("{prefix}Plain Name"), "looked up".to_owned());
    table
        .strings
        .insert(format!("{prefix}pack.name"), "My Pack".to_owned());
    let english = lang_of("en_US");
    let tables = [(&english, &table)];
    assert_eq!(
        stack.pack_keyword_value("Plain Name", &id, &tables),
        "Plain Name"
    );
    assert_eq!(
        stack.pack_keyword_value("pack.name", &id, &tables),
        "My Pack"
    );
}

#[test]
fn the_first_definition_of_a_pack_keyword_is_kept() {
    // the same pack identity twice on one stack: the keys share their prefix, and the copy
    // that is merged first (the lower one) keeps the key
    let same_pack = |name: &str| {
        keyword_pack(
            7,
            &[
                ("texts/languages.json", r#"["en_US"]"#),
                ("texts/en_US.lang", name),
            ],
        )
    };
    let stack = resource_stack(vec![
        same_pack("pack.name=Second\na=top\n"),
        same_pack("pack.name=First\na=bottom\n"),
    ]);
    let english = stack.translations(&lang_of("en_US")).unwrap();
    let id = stack.packs().next().unwrap().id().clone();
    assert_eq!(english.pack_keyword(&id, "pack.name"), Some("First"));
    // an ordinary key is the top pack's, as ever
    assert_eq!(english.get("a"), Some("top"));
    let keyword = english
        .strings
        .keys()
        .find(|key| key.ends_with("pack.name"))
        .unwrap();
    assert_eq!(english.origin(keyword).unwrap().pack.index(), Some(1));
}

#[test]
fn the_layers_of_one_pack_are_its_later_file_first() {
    let stack = resource_stack(vec![
        pack_with(
            2,
            "resources",
            &[
                ("texts/languages.json", r#"["de_DE"]"#),
                ("texts/language_names.json", r#"[["de_DE", "Deutsch"]]"#),
            ],
        ),
        pack_with(
            1,
            "resources",
            &[
                ("texts/languages.json", r#"["en_US"]"#),
                ("texts/language_names.json", r#"[["en_US", "English"]]"#),
            ],
        ),
    ]);
    let places: Vec<(Option<usize>, &str)> = stack
        .languages()
        .layers()
        .iter()
        .map(|layer| (layer.origin().pack.index(), layer.origin().path.as_str()))
        .collect();
    // top pack first; within a pack the file the walk reaches later comes first
    assert_eq!(
        places,
        [
            (Some(0), "texts/languages.json"),
            (Some(0), "texts/language_names.json"),
            (Some(1), "texts/languages.json"),
            (Some(1), "texts/language_names.json"),
        ]
    );
}

#[test]
fn a_keyword_another_pack_defines_does_not_resolve_this_packs_name() {
    let stack = resource_stack(vec![
        keyword_pack(2, &[("texts/languages.json", r#"["en_US"]"#)]),
        keyword_pack(
            1,
            &[
                ("texts/languages.json", r#"["en_US"]"#),
                ("texts/en_US.lang", "pack.name=The Other Pack\n"),
            ],
        ),
    ]);
    // L6 for the top pack only: the lower pack's own file resolves the lower pack's name
    assert_eq!(
        language_findings(&stack, "UnresolvedLangKey", Severity::Info),
        [(
            Some(0),
            "manifest.json".to_owned(),
            "header/name".to_owned()
        )]
    );
}

// `languages()` reads the two lists and the `.lang` files; every other file
// below `texts/` and `Texts/` is looked at by its path alone, from the pack's file list, so
// a stack load never asks for a glyph sheet or a stray file.
#[test]
fn a_stack_load_of_the_languages_asks_for_the_lists_and_the_lang_files_only() {
    use bedrock_addon::load::Collection;

    use crate::responder::Order;

    let files: &[(&str, &[u8])] = &[
        ("texts/languages.json", br#"["en_US"]"#),
        ("texts/en_US.lang", b"a=b\n"),
        ("texts/en_US/font/glyph_E0.png", b"\x89PNG"),
        ("texts/en_US/other.json", b"{}"),
        ("texts/zz_ZZ/x.png", b"\x89PNG"),
        ("texts/notes.txt", b"a note"),
        ("Texts/readme.txt", b"a note"),
    ];
    for order in Order::ALL {
        let (set, mut asked) =
            crate::requests::stack_load("resources", &[files], &[Collection::Languages], order);
        asked.sort();
        assert_eq!(
            asked,
            ["p0/texts/en_US.lang", "p0/texts/languages.json"],
            "{order:?}"
        );
        // The findings about the unread files are raised from their paths all the same.
        let found: Vec<(&str, &str)> = set
            .resource
            .languages()
            .diagnostics()
            .map(|f| {
                let path = f.location.path.as_ref().map_or("", |path| path.as_str());
                (f.code.name(), path)
            })
            .collect();
        assert!(
            found.contains(&("ShadowedDefinition", "texts/en_US/other.json")),
            "{order:?}: {found:?}"
        );
        assert!(
            found.contains(&("IgnoredFile", "texts/zz_ZZ")),
            "{order:?}: {found:?}"
        );
        assert!(
            found.iter().any(|(_, path)| path.starts_with("Texts")),
            "{order:?}: {found:?}"
        );
        assert!(
            !found.iter().any(|(_, path)| path.contains("glyph_E0")),
            "{order:?}: {found:?}"
        );
    }
}
