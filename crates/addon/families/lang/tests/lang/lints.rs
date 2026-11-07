//! Our lints on one `.lang` file: each rule with
//! both of its ends, on inline bytes. Vanilla loads every one of these files without a word,
//! and the table is built exactly as vanilla builds it.

use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticDetail, Diagnostics, Severity};
use bedrock_addon_family_lang::LangFile;

use super::common::*;

/// The findings of a file with these bytes.
fn read(bytes: &[u8]) -> Diagnostics {
    LangFile::from_bytes(en_us(), bytes).1
}

/// The table of a file with these bytes.
fn table(bytes: &[u8]) -> std::collections::BTreeMap<String, String> {
    LangFile::from_bytes(en_us(), bytes)
        .0
        .select(&bedrock_addon_family_lang::KeyFilter::default())
        .strings
}

/// The key a finding names: its breadcrumb.
fn key_of(diagnostic: &Diagnostic) -> String {
    diagnostic.location.json_path.to_string()
}

// ---- L4: text after the tab that ends a value --------------------------------------------------

/// The L4 findings: `IgnoredMember` at info.
fn tab_cuts(diagnostics: &Diagnostics) -> Vec<&Diagnostic> {
    diagnostics
        .iter()
        .filter(|d| {
            d.is(&bedrock_addon_core::diagnostic::codes::IGNORED_MEMBER)
                && d.severity == Severity::Info
        })
        .collect()
}

#[test]
fn text_after_a_tab_is_reported_as_ignored_deviation() {
    // vanilla = "silent"
    let found = read(b"some.key=shown\thidden\n");
    let cuts = tab_cuts(&found);
    assert_eq!(cuts.len(), 1, "{found:#?}");
    assert_eq!(key_of(cuts[0]), "some.key");
    assert_eq!(found.len(), 1);
    assert_eq!(table(b"some.key=shown\thidden\n")["some.key"], "shown");
}

#[test]
fn a_comment_after_the_tab_is_the_convention_and_not_reported() {
    for bytes in [
        &b"a=b\t## c\n"[..],
        b"a=b\t# c\n",
        b"a=b\t#\n",
        // blanks between the tab and the comment
        b"a=b\t ## c\n",
        b"a=b\t\t \t## c\n",
    ] {
        let found = read(bytes);
        assert!(found.is_empty(), "{:?}: {found:#?}", bytes.escape_ascii());
        assert_eq!(table(bytes)["a"], "b");
    }
}

#[test]
fn a_tab_with_nothing_after_it_cuts_nothing() {
    for bytes in [&b"a=b\t\n"[..], b"a=b\t", b"a=b\t  \t\n"] {
        let found = read(bytes);
        assert!(found.is_empty(), "{:?}: {found:#?}", bytes.escape_ascii());
        assert_eq!(table(bytes)["a"], "b");
    }
}

#[test]
fn text_after_blanks_after_the_tab_is_still_reported() {
    let found = read(b"a=b\t c\n");
    assert_eq!(tab_cuts(&found).len(), 1, "{found:#?}");
}

// ---- L13: a key defined twice ------------------------------------------------------------------

/// The L13 findings: `DuplicateIdentifier` at info.
fn duplicates(diagnostics: &Diagnostics) -> Vec<&Diagnostic> {
    diagnostics
        .iter()
        .filter(|d| {
            d.is(&bedrock_addon_core::diagnostic::codes::DUPLICATE_IDENTIFIER)
                && d.severity == Severity::Info
        })
        .collect()
}

/// The line an L13 finding names as the earlier definition.
fn earlier_line(diagnostic: &Diagnostic) -> String {
    match diagnostic.detail.as_deref() {
        Some(DiagnosticDetail::Reference { kind, id, .. }) => {
            assert_eq!(*kind, "line");
            id.clone()
        }
        other => panic!("no earlier line: {other:?}"),
    }
}

#[test]
fn a_duplicate_key_names_both_lines_and_the_last_wins_deviation() {
    // vanilla = "silent, last wins"
    let bytes = b"## head\na=1\nb=2\na=3\n";
    let found = read(bytes);
    let twice = duplicates(&found);
    assert_eq!(twice.len(), 1, "{found:#?}");
    assert_eq!(found.len(), 1);
    assert_eq!(key_of(twice[0]), "a");
    // at the later definition (line 4), naming the earlier one (line 2)
    assert_eq!(
        twice[0]
            .location
            .span
            .map(|s| s.line_col("## head\na=1\nb=2\na=3\n")),
        Some((4, 1))
    );
    assert_eq!(earlier_line(twice[0]), "2");
    assert_eq!(table(bytes)["a"], "3");
}

#[test]
fn keys_that_differ_only_in_case_are_one_key() {
    let found = read(b"Some.Key=1\nsome.key=2\n");
    assert_eq!(duplicates(&found).len(), 1, "{found:#?}");
    assert_eq!(table(b"Some.Key=1\nsome.key=2\n")["some.key"], "2");
}

#[test]
fn each_later_definition_names_the_one_before_it() {
    let found = read(b"a=1\na=2\na=3\n");
    let lines: Vec<String> = duplicates(&found).into_iter().map(earlier_line).collect();
    assert_eq!(lines, ["1", "2"]);
}

#[test]
fn a_dropped_line_defines_nothing() {
    for bytes in [&b"a=\na=1\n"[..], b"a\na=1\n", b"a=1\nb=1\n"] {
        let found = read(bytes);
        assert!(duplicates(&found).is_empty(), "{found:#?}");
    }
}

// ---- L2: a space around the `=` ----------------------------------------------------------------

/// The L2 findings: `LangKeyTrailingSpace` at information. The game stores the key with its
/// space and logs nothing (its only lang-parse texts are the newline and NUL ones), so the lint
/// is the crate's own and stays below the warning channel.
fn spaces(diagnostics: &Diagnostics) -> Vec<&Diagnostic> {
    diagnostics
        .iter()
        .filter(|d| {
            d.is(&bedrock_addon_core::diagnostic::codes::LANG_KEY_TRAILING_SPACE)
                && d.severity == Severity::Info
        })
        .collect()
}

#[test]
fn spaces_around_the_equals_sign_name_the_stored_key_deviation() {
    // vanilla = "silent; key stored with the trailing space"
    let found = read(b"Key = v\n");
    let warned = spaces(&found);
    assert_eq!(warned.len(), 1, "{found:#?}");
    assert_eq!(found.len(), 1);
    assert_eq!(key_of(warned[0]), "key ");
    assert_eq!(table(b"Key = v\n")["key "], " v");
}

#[test]
fn a_key_that_ends_in_a_space_is_enough() {
    let found = read(b"key =v\n");
    assert_eq!(spaces(&found).len(), 1, "{found:#?}");
}

#[test]
fn a_value_that_starts_with_a_space_is_enough() {
    let found = read(b"key= v\n");
    assert_eq!(spaces(&found).len(), 1, "{found:#?}");
}

#[test]
fn spaces_elsewhere_are_not_reported() {
    for bytes in [
        &b"key=v\n"[..],
        b"my key=a b\n",
        // a value may end in a space, and blanks before the key are skipped
        b"key=v \n",
        b"  key=v\n",
        // a tab is not a space
        b"key\t=v\n",
    ] {
        let found = read(bytes);
        assert!(found.is_empty(), "{:?}: {found:#?}", bytes.escape_ascii());
    }
}

// ---- L3: `=` as the first byte of a line -------------------------------------------------------

/// The L3 findings: `InvalidValue` at warning.
fn leading_equals(diagnostics: &Diagnostics) -> Vec<&Diagnostic> {
    diagnostics
        .iter()
        .filter(|d| {
            d.is(&bedrock_addon_core::diagnostic::codes::INVALID_VALUE)
                && d.severity == Severity::Info
        })
        .collect()
}

#[test]
fn a_line_that_starts_with_equals_is_reported_deviation() {
    // vanilla = "silent; '=' becomes a key character"
    for bytes in [&b"=a=b\n"[..], b" \t=a=b\n"] {
        let found = read(bytes);
        let warned = leading_equals(&found);
        assert_eq!(warned.len(), 1, "{found:#?}");
        assert_eq!(found.len(), 1);
        assert_eq!(key_of(warned[0]), "=a");
        assert_eq!(table(bytes)["=a"], "b");
    }
}

#[test]
fn an_equals_sign_elsewhere_is_not_reported() {
    for bytes in [&b"a==b\n"[..], b"a=b=c\n", b"a=b\n"] {
        let found = read(bytes);
        assert!(leading_equals(&found).is_empty(), "{found:#?}");
    }
    assert_eq!(table(b"a==b\n")["a"], "=b");
    assert_eq!(table(b"a=b=c\n")["a"], "b=c");
}

#[test]
fn a_dropped_line_that_starts_with_equals_has_only_vanillas_warning() {
    let found = read(b"=a\n");
    assert!(leading_equals(&found).is_empty(), "{found:#?}");
    assert_eq!(found.len(), 1);
}

// ---- L11: placeholders the lookup will erase or keep -------------------------------------------

/// The L11 findings: `InvalidValue` at info.
fn odd_placeholders(diagnostics: &Diagnostics) -> Vec<&Diagnostic> {
    diagnostics
        .iter()
        .filter(|d| {
            d.is(&bedrock_addon_core::diagnostic::codes::INVALID_VALUE)
                && d.severity == Severity::Info
        })
        .collect()
}

/// The findings of the one-line file `k=<value>`.
fn read_value(value: &str) -> Diagnostics {
    read(format!("k={value}\n").as_bytes())
}

#[test]
fn placeholders_the_lookup_erases_or_keeps_are_reported_deviation() {
    // vanilla = "silent"
    for value in [
        // `$s` / `$d` outside a `%N$s` form: the lookup erases them
        "cost 5$s",
        "cost 5$d",
        "$s",
        "%s$s",
        "%$s",
        // `%%` is not an escape: the lookup keeps both bytes
        "+%d%% %s",
        "100%%",
        // `%0` and two-digit indices: the lookup knows `%1` to `%9` only
        "%0",
        "page %10 of %2",
        "%0$s",
        "%12$s",
    ] {
        let found = read_value(value);
        let odd = odd_placeholders(&found);
        assert_eq!(odd.len(), 1, "{value:?}: {found:#?}");
        assert_eq!(found.len(), 1, "{value:?}");
        assert_eq!(key_of(odd[0]), "k");
    }
}

#[test]
fn the_placeholders_the_lookup_knows_are_not_reported() {
    for value in [
        "%s and %d",
        "%1 of %9",
        "Page %1$s of %2$d",
        "%key.name here",
        // a lone percent sign and a lone dollar sign are ordinary bytes
        "100% sure",
        "50%",
        "5$ each",
        "$",
        "s$",
    ] {
        let found = read_value(value);
        assert!(found.is_empty(), "{value:?}: {found:#?}");
    }
}

#[test]
fn one_value_is_one_placeholder_finding() {
    let found = read_value("%0 %% 5$s %10");
    assert_eq!(odd_placeholders(&found).len(), 1, "{found:#?}");
}

// ---- L14: bytes that are not UTF-8 -------------------------------------------------------------

/// The L14 findings: `MalformedInput` of ours at info.
fn not_utf8(diagnostics: &Diagnostics) -> Vec<&Diagnostic> {
    diagnostics
        .iter()
        .filter(|d| {
            d.is(&bedrock_addon_core::diagnostic::codes::SYNTAX) && d.severity == Severity::Info
        })
        .collect()
}

#[test]
fn invalid_utf8_in_a_value_is_reported_and_decoded_lossily_deviation() {
    // vanilla = "bytes copied, no decoding"; here the table holds text, so each invalid
    // sequence becomes U+FFFD
    let bytes = b"k=caf\xE9 au lait\n";
    let found = read(bytes);
    assert_eq!(not_utf8(&found).len(), 1, "{found:#?}");
    assert_eq!(found.len(), 1);
    assert_eq!(key_of(not_utf8(&found)[0]), "k");
    assert_eq!(table(bytes)["k"], "caf\u{fffd} au lait");
}

#[test]
fn invalid_utf8_in_a_key_is_reported_once_per_line() {
    let found = read(b"k\xFF=v\n");
    assert_eq!(not_utf8(&found).len(), 1, "{found:#?}");
    assert_eq!(key_of(not_utf8(&found)[0]), "k\u{fffd}");
    let both = read(b"k\xFF=v\xFE\n");
    assert_eq!(not_utf8(&both).len(), 1, "{both:#?}");
}

#[test]
fn valid_utf8_and_bytes_outside_keys_and_values_are_not_reported() {
    for bytes in [
        "k=caf\u{e9} \u{2713} \u{a7}a\n".as_bytes(),
        "cl\u{e9}=v\n".as_bytes(),
        // a comment line and the text after a value are not read
        b"## \xFF\n",
        b"k=v\t# \xFF\n",
    ] {
        let found = read(bytes);
        assert!(found.is_empty(), "{:?}: {found:#?}", bytes.escape_ascii());
    }
}

#[test]
fn invalid_utf8_outside_keys_and_values_is_the_replacement_character_without_a_finding_deviation() {
    // L14 covers keys and values; a comment line, the text after a value and a line vanilla
    // drops hold text too, so their invalid bytes are read as U+FFFD, silently
    let bytes = b"## c\xFF\nk=v\t# t\xFE\nempty\xFD=\n";
    let (file, found) = LangFile::from_bytes(en_us(), bytes);
    assert!(found.is_empty(), "{found:#?}");
    assert_eq!(
        file.lines,
        [
            bedrock_addon_family_lang::Line::Comment(" c\u{fffd}".to_owned()),
            bedrock_addon_family_lang::Line::Entry {
                key: "k".to_owned(),
                value: "v".to_owned(),
                trailing: "\t# t\u{fffd}".to_owned(),
            },
            bedrock_addon_family_lang::Line::Dropped("empty\u{fffd}=".to_owned()),
        ]
    );
    // a line without `=` has vanilla's warning only, with the key as read
    let (file, found) = LangFile::from_bytes(en_us(), b"no\xFFequals\n");
    assert_eq!(found.len(), 1);
    assert!(not_utf8(&found).is_empty());
    assert_eq!(
        file.lines,
        [bedrock_addon_family_lang::Line::Dropped(
            "no\u{fffd}equals".to_owned()
        )]
    );
}
