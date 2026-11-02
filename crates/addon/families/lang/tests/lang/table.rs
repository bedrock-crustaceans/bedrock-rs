//! `Translations`: the case-insensitive lookup, the base-language copy
//! (README "Load order", step 3), and the `~` identifiers a chosen language expands.

use std::collections::BTreeMap;

use bedrock_addon_family_lang::{KeyFilter, LangFile, Translations};

use super::common::*;

/// A table with exactly these stored keys.
fn table(pairs: &[(&str, &str)], may_contain_identifier: bool) -> Translations {
    Translations {
        strings: pairs
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect::<BTreeMap<_, _>>(),
        may_contain_identifier,
    }
}

// ---- lookup --------------------------------------------------------------------------------

#[test]
fn get_is_case_insensitive_for_ascii_as_vanilla_is() {
    let table = lang("upper_key").table;
    assert_eq!(table.get("key.name"), Some("Value"));
    assert_eq!(table.get("Key.Name"), Some("Value"));
    assert_eq!(table.get("KEY.NAME"), Some("Value"));
    assert_eq!(table.get("key.other"), None);
}

#[test]
fn get_tries_the_exact_key_before_the_lower_cased_one() {
    let table = table(&[("Key", "exact"), ("key", "lower")], false);
    assert_eq!(table.get("Key"), Some("exact"));
    assert_eq!(table.get("KEY"), Some("lower"));
}

#[test]
fn get_folds_only_ascii_letters() {
    let table = table(&[("\u{e9}t\u{e9}", "summer")], false);
    assert_eq!(table.get("\u{e9}T\u{e9}"), Some("summer"));
    assert_eq!(table.get("\u{c9}t\u{e9}"), None);
}

#[test]
fn insert_stores_the_key_lower_cased_and_replaces() {
    let mut table = Translations::new();
    table.insert("Key.Name", "first");
    table.insert("KEY.NAME", "second");
    assert_eq!(
        table.strings.iter().collect::<Vec<_>>(),
        [(&"key.name".to_owned(), &"second".to_owned())]
    );
    assert!(!table.may_contain_identifier);
    table.insert("other", "a~b");
    assert!(table.may_contain_identifier);
}

#[test]
fn append_puts_the_other_tables_values_on_top() {
    let mut lower = table(&[("a", "1"), ("b", "2")], false);
    lower.append(&table(&[("b", "3"), ("c", "~")], true));
    assert_eq!(lower, table(&[("a", "1"), ("b", "3"), ("c", "~")], true));
    // the flag is never cleared
    lower.append(&table(&[("d", "4")], false));
    assert!(lower.may_contain_identifier);
    assert_eq!(lower.strings.len(), 4);
}

#[test]
fn with_base_starts_from_a_copy_of_the_base_table() {
    let base = table(&[("a", "1"), ("b", "2")], false);
    let german = table(&[("b", "zwei"), ("c", "drei")], false);
    assert_eq!(
        german.with_base(&base),
        table(&[("a", "1"), ("b", "zwei"), ("c", "drei")], false)
    );
    // neither table changes, and the base's `~` flag comes along with its values
    assert_eq!(german.strings.len(), 2);
    assert!(
        german
            .with_base(&table(&[("a", "~")], true))
            .may_contain_identifier
    );
    assert!(
        table(&[("a", "~")], true)
            .with_base(&base)
            .may_contain_identifier
    );
}

// ---- `~` identifiers -----------------------------------------------------------------------

#[test]
fn a_tilde_sets_the_flag_and_linebreak_is_rewritten_when_chosen() {
    let mut table = lang("tilde_flag").table;
    assert!(table.may_contain_identifier);
    table.replace_identifiers();
    assert_eq!(table.get("a"), Some("x\ny"));
    // a malformed identifier is left as it is
    assert_eq!(table.get("b"), Some("p~LINEBEAK~q"));
}

#[test]
fn the_flag_is_set_when_the_value_is_read_even_if_the_entry_is_filtered_out() {
    let (file, _) = LangFile::from_bytes(en_us(), b"a=x~y\nb=z\n");
    let filter = KeyFilter {
        exclude: &["a"],
        ..KeyFilter::default()
    };
    let selected = file.select(&filter);
    assert_eq!(selected.strings.len(), 1);
    assert!(selected.may_contain_identifier);
    // a `~` in a key or in a comment is not a value byte
    let (file, _) = LangFile::from_bytes(en_us(), b"a~=x\n## ~\nb=y\t~\n");
    assert!(!file.translations().may_contain_identifier);
}

#[test]
fn identifiers_are_rewritten_only_when_the_flag_is_set() {
    let mut unflagged = table(&[("a", "x~LINEBREAK~y")], false);
    unflagged.replace_identifiers();
    assert_eq!(unflagged.get("a"), Some("x~LINEBREAK~y"));

    let mut flagged = table(
        &[("a", "~LINEBREAK~x~LINEBREAK~~LINEBREAK~"), ("b", "plain")],
        true,
    );
    flagged.replace_identifiers();
    assert_eq!(flagged.get("a"), Some("\nx\n\n"));
    assert_eq!(flagged.get("b"), Some("plain"));
}

#[test]
fn utc_datetime_identifiers_are_not_expanded_deviation() {
    // vanilla rewrites `~UTC_DATETIME=<time>~` to the local date and time through its
    // `DateManager`; this crate has no clock or time zone
    let mut table = table(
        &[("a", "at ~UTC_DATETIME=2026-01-01T00:00:00Z~ sharp")],
        true,
    );
    table.replace_identifiers();
    assert_eq!(
        table.get("a"),
        Some("at ~UTC_DATETIME=2026-01-01T00:00:00Z~ sharp")
    );
}
