//! Lookup-time formatting (README "Lookup and formatting"): the `format/` fixtures,
//! each a list of cases (an id looked up in the file's `[keys]`, or a value given to
//! `replace_tokens`), and the edges the folder leaves to the implementation.

use bedrock_addon_family_lang::{Translations, replace_tokens};

use super::common::*;

/// Runs every `[[case]]` of `format/<name>.toml`; returns how many there were.
fn cases(name: &str) -> usize {
    let file = toml(&fixture_text(&format!("format/{name}.toml")));
    let mut table = Translations::new();
    for (key, value) in file.table("keys").strings() {
        table.insert(&key, value);
    }
    let cases = file.array("case");
    for case in &cases {
        let params = case.list("params");
        let params: Vec<&str> = params.iter().map(String::as_str).collect();
        let expected = case.str("expected").expect("a case has an expectation");
        let got = match (case.str("id"), case.str("value")) {
            (Some(id), None) => table
                .format(id, &params)
                .unwrap_or_else(|| panic!("{name}: `{id}` is not in the table")),
            (None, Some(value)) => replace_tokens(value, &params),
            other => panic!("{name}: a case has an `id` or a `value`: {other:?}"),
        };
        assert_eq!(got, expected, "{name}: {case:?}");
    }
    cases.len()
}

/// A table with these keys.
fn table(pairs: &[(&str, &str)]) -> Translations {
    let mut table = Translations::new();
    for (key, value) in pairs {
        table.insert(key, *value);
    }
    table
}

// ---- the fixture rows ----------------------------------------------------------------------

#[test]
fn ut_insert_key() {
    assert_eq!(cases("ut_insert_key"), 1);
}

#[test]
fn ut_param_order() {
    assert_eq!(cases("ut_param_order"), 1);
}

#[test]
fn ut_positional() {
    assert_eq!(cases("ut_positional"), 2);
}

#[test]
fn ut_reverse() {
    assert_eq!(cases("ut_reverse"), 1);
}

#[test]
fn ut_percent_escapes() {
    assert_eq!(cases("ut_percent_escapes"), 12);
}

#[test]
fn ut_replace_tokens() {
    assert_eq!(cases("ut_replace_tokens"), 1);
}

#[test]
fn dollar_s() {
    assert_eq!(cases("dollar_s"), 2);
}

#[test]
fn double_percent() {
    assert_eq!(cases("double_percent"), 1);
}

#[test]
fn mixed_offset() {
    assert_eq!(cases("mixed_offset"), 1);
}

#[test]
fn section_sign() {
    assert_eq!(cases("section_sign"), 1);
}

// ---- looking an id up ----------------------------------------------------------------------

#[test]
fn a_plain_id_is_its_stored_value_or_nothing() {
    let table = table(&[("key.1", "Value %s"), ("plain", "Text")]);
    assert_eq!(table.format("plain", &[]).as_deref(), Some("Text"));
    // without parameters the placeholders stay
    assert_eq!(table.format("key.1", &[]).as_deref(), Some("Value %s"));
    assert_eq!(table.format("Key.1", &["x"]).as_deref(), Some("Value x"));
    assert_eq!(table.format("missing", &[]), None);
    assert_eq!(table.format("missing", &["x"]), None);
}

#[test]
fn a_missing_key_in_an_id_is_its_bare_name() {
    let table = table(&[("key.1", "Value 1")]);
    assert_eq!(
        table.format("%key.1 and %key.2", &[]).as_deref(),
        Some("Value 1 and key.2")
    );
    // an id with `%` is never a lookup of the whole id
    assert_eq!(table.format("%missing", &[]).as_deref(), Some("missing"));
}

#[test]
fn a_key_name_in_an_id_is_letters_digits_dot_underscore_and_hyphen() {
    let table = table(&[("a-b_c.1", "X"), ("a", "A")]);
    assert_eq!(table.format("%a-b_c.1!", &[]).as_deref(), Some("X!"));
    assert_eq!(table.format("%a b", &[]).as_deref(), Some("A b"));
    assert_eq!(table.format("%a:b", &[]).as_deref(), Some("A:b"));
    assert_eq!(table.format("(%A)", &[]).as_deref(), Some("(A)"));
}

#[test]
fn a_percent_sign_with_no_name_after_it_is_consumed() {
    // the `%` is consumed and the empty name stands for it
    let table = Translations::new();
    assert_eq!(table.format("100% sure", &[]).as_deref(), Some("100 sure"));
    assert_eq!(table.format("50%", &[]).as_deref(), Some("50"));
    assert_eq!(table.format("%%", &[]).as_deref(), Some("%"));
    assert_eq!(table.format("%%%", &[]).as_deref(), Some("%"));
}

#[test]
fn a_parameter_that_names_a_key_is_translated() {
    let table = table(&[("item.name", "Ruby"), ("got", "You got %s")]);
    assert_eq!(
        table.format("got", &["%item.name"]).as_deref(),
        Some("You got Ruby")
    );
    // a parameter that names no key is kept as written, and a parameter without `%` is
    // never looked up
    assert_eq!(
        table.format("got", &["%item.other"]).as_deref(),
        Some("You got %item.other")
    );
    assert_eq!(
        table.format("got", &["item.name"]).as_deref(),
        Some("You got item.name")
    );
}

// ---- replace_tokens ------------------------------------------------------------------------

#[test]
fn without_parameters_nothing_is_replaced_or_erased() {
    assert_eq!(replace_tokens("%s %1 5$s %%", &[]), "%s %1 5$s %%");
}

#[test]
fn a_token_beyond_the_parameters_is_erased() {
    assert_eq!(replace_tokens("%s-%s-%d", &["a"]), "a--");
    assert_eq!(replace_tokens("%1-%2-%3", &["a", "b"]), "a-b-");
    assert_eq!(replace_tokens("%0|%9", &["a"]), "|");
}

#[test]
fn percent_d_takes_parameters_as_percent_s_does() {
    assert_eq!(replace_tokens("%d %s %d", &["1", "2", "3"]), "1 2 3");
}

#[test]
fn an_index_is_one_digit() {
    assert_eq!(replace_tokens("%10", &["a"]), "a0");
    assert_eq!(
        replace_tokens("%9", &["1", "2", "3", "4", "5", "6", "7", "8", "9"]),
        "9"
    );
}

#[test]
fn an_index_counts_from_the_parameters_the_sequential_tokens_left() {
    assert_eq!(
        replace_tokens("%s %s %1 %2", &["a", "b", "c", "d"]),
        "a b c d"
    );
    // every sequential token counts, the ones beyond the parameters too
    assert_eq!(replace_tokens("%s %s %s %1", &["a", "b"]), "a b  ");
}

#[test]
fn inserted_text_is_scanned_again_for_sequential_tokens() {
    // the search restarts from the start of the string after each insertion
    assert_eq!(replace_tokens("%s", &["%s", "x"]), "x");
    assert_eq!(replace_tokens("[%s]", &["%d%d", "x", "y"]), "[xy]");
}

#[test]
fn inserted_text_is_not_scanned_again_for_indices() {
    // one pass, left to right
    assert_eq!(replace_tokens("%1 %2", &["%2", "b"]), "%2 b");
}

#[test]
fn dollar_tokens_are_erased_before_anything_is_replaced() {
    assert_eq!(replace_tokens("%1$s %2$d", &["a", "b"]), "a b");
    assert_eq!(replace_tokens("$s$d", &["a"]), "");
    // a `$` before another byte stays
    assert_eq!(replace_tokens("5$ $x", &["a"]), "5$ $x");
}
