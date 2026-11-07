//! The `.lang` grammar: the `lang/` fixtures, each read as vanilla reads it and
//! compared with its `*.expected.toml` (stored key → value, findings in order).

use bedrock_addon_core::diagnostic::{Diagnostics, Severity};
use bedrock_addon_family_lang::{Line, codes};

use super::common::*;

// ---- keys ----------------------------------------------------------------------------------

#[test]
fn upper_key_is_stored_lower_cased() {
    let read = lang("upper_key");
    // the line keeps the key as written; only the table lower-cases it
    assert_eq!(read.file.lines, [Line::entry("Key.Name", "Value")]);
    assert_eq!(read.file.language, en_us());
}

#[test]
fn ut_regular_registers_the_value_without_a_log() {
    let read = lang("ut_regular");
    assert!(read.diagnostics.is_empty());
    assert_eq!(read.file.lines.len(), 1);
}

// ---- a tab ends the value ------------------------------------------------------------------

/// An entry line with what follows its value.
fn entry(key: &str, value: &str, trailing: &str) -> Line {
    Line::Entry {
        key: key.to_owned(),
        value: value.to_owned(),
        trailing: trailing.to_owned(),
    }
}

#[test]
fn a_tab_ends_the_value() {
    let read = lang("tab_ends_value");
    // the rest of the line is kept with the line, from the tab on
    assert_eq!(read.file.lines, [entry("a", "b", "\tc")]);
}

#[test]
fn ut_trailing_tab_registers_only_the_first_key() {
    let read = lang("ut_trailing_tab");
    assert!(!read.table.strings.contains_key("key.comment"));
    assert_eq!(
        read.file.lines,
        [entry(
            "key.trailing_whitespace",
            "Value followed by tab",
            "\tkey.comment=Not Registered"
        )]
    );
}

// ---- `##` comments -------------------------------------------------------------------------

#[test]
fn a_double_pound_ends_the_value_and_the_space_before_it_is_kept() {
    let read = lang("comment_in_value");
    assert_eq!(read.file.lines, [entry("a", "b ", "## c")]);
}

#[test]
fn ut_trailing_comment_registers_only_the_first_key() {
    let read = lang("ut_trailing_comment");
    assert!(!read.table.strings.contains_key("key.comment"));
    assert!(read.diagnostics.is_empty());
}

#[test]
fn ut_single_pound_is_part_of_the_value() {
    let read = lang("ut_single_pound");
    assert_eq!(
        read.file.lines,
        [Line::entry(
            "key.pound_value",
            "Value with single # is valid"
        )]
    );
}

#[test]
fn a_single_pound_at_the_line_start_is_a_key_character() {
    let read = lang("pound_key");
    assert_eq!(read.file.lines, [Line::entry("#a", "b")]);
}

#[test]
fn a_double_pound_at_the_line_start_is_a_comment_line() {
    let read = lang("comment_line");
    assert_eq!(
        read.file.lines,
        [
            Line::Comment(" a=b".to_owned()),
            // the blanks before the `##` are skipped
            Line::Comment(" c=d".to_owned()),
            Line::entry("x", "y"),
        ]
    );
}

// ---- blanks before the key -----------------------------------------------------------------

#[test]
fn ut_leading_whitespace_registers_the_key_without_the_tab() {
    let read = lang("ut_leading_whitespace");
    assert_eq!(
        read.file.lines,
        [Line::entry(
            "key.leading_whitespace",
            "Value with leading whitespace"
        )]
    );
}

#[test]
fn spaces_and_tabs_before_the_key_are_skipped() {
    let read = lang("leading_space");
    assert_eq!(read.file.lines, [Line::entry("a", "b")]);
}

// ---- escapes (value only) ------------------------------------------------------------------

#[test]
fn the_escape_backslash_n_becomes_a_line_feed() {
    let read = lang("escape_newline");
    assert_eq!(read.file.lines, [Line::entry("a", "x\ny")]);
}

#[test]
fn a_backslash_before_another_byte_is_kept() {
    let read = lang("backslash_kept");
    assert_eq!(read.file.lines, [Line::entry("a", "C:\\path")]);
}

#[test]
fn backslash_t_stays_two_bytes() {
    let read = lang("backslash_t");
    assert_eq!(read.table.strings["a"].as_bytes(), b"x\\ty");
}

#[test]
fn ut_escape_newline_reads_one_and_two_line_feeds() {
    let read = lang("ut_escape_newline");
    assert_eq!(read.table.strings["key.1"], "Text before\nand after");
    assert_eq!(read.table.strings["key.2"], "Two\n\nlines");
}

#[test]
fn a_double_backslash_is_one_backslash_and_ends_the_escape() {
    let read = lang("double_backslash");
    assert_eq!(read.table.strings["a"].as_bytes(), b"x\\y");
    // `\\n` is a backslash and an `n`, not a line feed
    assert_eq!(read.table.strings["b"].as_bytes(), b"x\\ny");
}

#[test]
fn a_backslash_as_the_last_byte_of_the_file_is_kept() {
    let read = lang("trailing_backslash");
    assert_eq!(read.table.strings["a"].as_bytes(), b"x\\");
}

// ---- a line without `=` (V1, V2) -----------------------------------------------------------

/// The template and arguments of the one `MalformedInput` warning of a file.
fn no_equals_warning(diagnostics: &Diagnostics) -> (&'static str, Vec<String>) {
    let found = with_code(diagnostics, "MalformedInput", Severity::Warning);
    assert_eq!(found.len(), 1, "{diagnostics:#?}");
    for line in [&codes::NEW_LINE_IN_KEY, &codes::NULL_IN_KEY] {
        if let Some(args) = found[0].code.args_of(line) {
            return (line.template(), args.to_vec());
        }
    }
    panic!("not vanilla's formatted text: {:?}", found[0].code);
}

const NEW_LINE: &str =
    "Line: {} - Invalid lang file format. New line character was found while parsing key: '{}'.";
const NULL: &str =
    "Line: {} - Invalid lang file format. Null character was found while parsing key: '{}'.";

#[test]
fn a_line_without_equals_is_dropped_with_the_new_line_warning() {
    let read = lang("noequals");
    // the line is kept as written; the text names the lower-cased partial key
    assert_eq!(read.file.lines, [Line::Dropped("NoEquals".to_owned())]);
    assert_eq!(
        no_equals_warning(&read.diagnostics),
        (NEW_LINE, vec!["1".to_owned(), "noequals".to_owned()])
    );
    let warning = read.diagnostics.iter().next().unwrap();
    assert_eq!(warning.location.json_path.to_string(), "noequals");
    assert_eq!(
        warning.location.span.map(|s| (s.offset, s.len)),
        Some((0, 8))
    );
}

#[test]
fn ut_no_equal_is_the_null_character_warning_at_the_end_of_the_file() {
    let read = lang("ut_no_equal");
    assert_eq!(
        no_equals_warning(&read.diagnostics),
        (NULL, vec!["1".to_owned(), "key.no_equal".to_owned()])
    );
    assert_eq!(read.file.lines, [Line::Dropped("key.no_equal".to_owned())]);
}

#[test]
fn ut_multiline_error_counts_line_feeds_and_keeps_reading() {
    let read = lang("ut_multiline_error");
    assert_eq!(
        no_equals_warning(&read.diagnostics),
        (NEW_LINE, vec!["6".to_owned(), "error".to_owned()])
    );
    assert_eq!(
        read.file.lines,
        [
            Line::Blank,
            Line::entry("key.1", "Value 1"),
            Line::entry("key.2", "Value 1"),
            Line::Blank,
            Line::Comment(" comment and newlines".to_owned()),
            Line::Dropped("error".to_owned()),
            Line::entry("key.3", "Value 3"),
        ]
    );
}

#[test]
fn a_pound_as_the_last_byte_of_the_file_starts_a_key() {
    let read = lang("pound_last_byte");
    assert_eq!(
        no_equals_warning(&read.diagnostics),
        (NULL, vec!["2".to_owned(), "#".to_owned()])
    );
    assert_eq!(
        read.file.lines,
        [Line::entry("a", "b#"), Line::Dropped("#".to_owned())]
    );
}

// ---- an empty value (V3) -------------------------------------------------------------------

#[test]
fn an_empty_value_is_dropped_silently() {
    let read = lang("empty_value");
    assert!(read.diagnostics.is_empty());
    assert_eq!(
        read.file.lines,
        [Line::Dropped("a=".to_owned()), Line::entry("b", "c")]
    );
}

#[test]
fn ut_key_without_value_is_not_registered_and_not_logged() {
    let read = lang("ut_key_without_value");
    assert!(read.diagnostics.is_empty());
    assert_eq!(read.file.lines, [Line::Dropped("key.no_value=".to_owned())]);
}

#[test]
fn a_value_that_a_tab_or_a_comment_ends_at_once_is_empty() {
    let read = lang("empty_value_cut");
    // dropped lines carry no lint: the text after the tab belongs to no entry
    assert!(read.diagnostics.is_empty());
    assert_eq!(
        read.file.lines,
        [
            Line::Dropped("a=\tb".to_owned()),
            Line::Dropped("c=## d".to_owned()),
            Line::entry("e", "f"),
        ]
    );
}

// ---- BOM, carriage returns, the end of the file --------------------------------------------

/// The lines of a file with these bytes.
fn lines(bytes: &[u8]) -> Vec<Line> {
    bedrock_addon_family_lang::LangFile::from_bytes(en_us(), bytes)
        .0
        .lines
}

#[test]
fn a_bom_at_the_start_of_the_file_is_skipped() {
    let read = lang("bom");
    assert_eq!(read.file.lines, [Line::entry("a", "b")]);
}

#[test]
fn a_bom_anywhere_else_is_three_ordinary_bytes() {
    assert_eq!(
        lines(b"x=y\n\xEF\xBB\xBFa=b\n"),
        [Line::entry("x", "y"), Line::entry("\u{feff}a", "b")]
    );
}

#[test]
fn carriage_returns_are_ignored_everywhere() {
    let read = lang("crlf");
    assert_eq!(
        read.file.lines,
        [
            Line::entry("a", "b"),
            Line::entry("c", "d"),
            Line::entry("e", "xy"),
        ]
    );
}

#[test]
fn a_carriage_return_is_in_no_part_of_a_line() {
    assert_eq!(
        lines(b"k\rey=v\r\n## c\r\nnoeq\r\na=b\t# c\r\n\r\nz=\r\n"),
        [
            Line::entry("key", "v"),
            Line::Comment(" c".to_owned()),
            Line::Dropped("noeq".to_owned()),
            entry("a", "b", "\t# c"),
            Line::Blank,
            Line::Dropped("z=".to_owned()),
        ]
    );
}

#[test]
fn the_end_of_the_file_ends_the_last_line() {
    let read = lang("no_trailing_newline");
    assert_eq!(
        read.file.lines,
        [Line::entry("a", "b"), Line::entry("c", "d")]
    );
    // a final line feed starts no further line; two do
    assert_eq!(lines(b"a=b\n"), [Line::entry("a", "b")]);
    assert_eq!(lines(b"a=b\n\n"), [Line::entry("a", "b"), Line::Blank]);
    assert_eq!(lines(b"a=b\n  "), [Line::entry("a", "b"), Line::Blank]);
    assert_eq!(lines(b""), []);
}

// ---- duplicate keys ------------------------------------------------------------------------

#[test]
fn the_last_definition_of_a_duplicate_key_wins() {
    let read = lang("duplicate");
    assert_eq!(read.table.strings["a"], "3");
    // every definition stays a line of the file
    assert_eq!(
        read.file.lines,
        [
            Line::entry("a", "1"),
            Line::entry("b", "2"),
            Line::entry("a", "3"),
        ]
    );
}

// ---- nothing is trimmed --------------------------------------------------------------------

#[test]
fn equals_as_the_first_byte_is_a_key_character() {
    let read = lang("equals_first");
    assert_eq!(read.file.lines, [Line::entry("=a", "b")]);
}

#[test]
fn spaces_around_the_equals_sign_belong_to_the_key_and_the_value() {
    let read = lang("spaces_around_equals");
    assert_eq!(read.file.lines, [Line::entry("key ", " v")]);
}

#[test]
fn a_tab_in_a_key_is_an_ordinary_byte() {
    let read = lang("tab_in_key");
    assert_eq!(read.file.lines, [Line::entry("a\tb", "c")]);
}

// ---- a NUL in the file ---------------------------------------------------------------------

#[test]
fn a_nul_ends_the_line_without_advancing_the_line_counter() {
    let read = lang("nul_mid_file");
    assert_eq!(
        no_equals_warning(&read.diagnostics),
        (NULL, vec!["2".to_owned(), "e".to_owned()])
    );
    // each part the NUL separates is a line of its own
    assert_eq!(
        read.file.lines,
        [
            Line::entry("a", "b"),
            Line::entry("c", "d"),
            Line::Dropped("e".to_owned()),
        ]
    );
}

#[test]
fn a_nul_while_a_key_is_read_is_the_null_character_warning() {
    let (file, diagnostics) =
        bedrock_addon_family_lang::LangFile::from_bytes(en_us(), b"x=y\nAb\0c=d\n");
    assert_eq!(
        no_equals_warning(&diagnostics),
        (NULL, vec!["2".to_owned(), "ab".to_owned()])
    );
    assert_eq!(
        file.lines,
        [
            Line::entry("x", "y"),
            Line::Dropped("Ab".to_owned()),
            Line::entry("c", "d"),
        ]
    );
}

#[test]
fn a_nul_right_after_a_line_end_starts_no_line() {
    assert_eq!(
        lines(b"a=b\0\0c=d\n\0"),
        [Line::entry("a", "b"), Line::entry("c", "d")]
    );
}

// ---- a whole file --------------------------------------------------------------------------

#[test]
fn ut_multiline_registers_all_six_keys_without_a_log() {
    let read = lang("ut_multiline");
    assert!(read.diagnostics.is_empty(), "{:#?}", read.diagnostics);
    assert_eq!(read.table.strings.len(), 6);
    assert_eq!(read.file.lines.len(), 10);
    assert_eq!(
        read.file.lines[3],
        entry("key.second", "Second value", "\t ## a comment after a tab")
    );
    assert_eq!(read.file.lines[5], Line::Blank);
    // non-ASCII bytes and `§` codes are copied like any other byte
    assert_eq!(
        read.table.get("key.unicode_entry"),
        Some("\u{dc}n\u{ef}c\u{f6}d\u{e9} \u{2713} \u{a7}aGreen")
    );
}

// ---- a carriage return between the two `#` --------------------------------

#[test]
fn a_carriage_return_between_two_pounds_is_no_comment() {
    // vanilla tests the byte right after a `#` as stored, and drops the carriage return
    // afterwards: the value holds `##`
    assert_eq!(lines(b"a=b#\r#c\n"), [Line::entry("a", "b##c")]);
    assert_eq!(lines(b"a=b#\r\r#c\n"), [Line::entry("a", "b##c")]);
    assert_eq!(lines(b"a=b#\r## c\n"), [entry("a", "b#", "## c")]);
    // at the line start it is a key, here one without `=`
    assert_eq!(
        lines(b"#\r#x\n#\r#k=v\n"),
        [Line::Dropped("##x".to_owned()), Line::entry("##k", "v")]
    );
    assert_eq!(lines(b"a=b#\r\n"), [Line::entry("a", "b#")]);
}

#[test]
fn a_file_with_a_carriage_return_between_two_pounds_is_written_back_byte_for_byte() {
    use bedrock_addon_family_lang::LangFile;

    // the writer puts the carriage return back wherever a `##` must not start a comment, so
    // these lone carriage returns survive; only CRLF becomes LF
    for (bytes, expected) in [
        (&b"a=b#\r#c\r\n"[..], &b"a=b#\r#c\n"[..]),
        (b"#\r#x\r\nk=v\r\n", b"#\r#x\nk=v\n"),
        (b"#\r#k=v#\r## c\n", b"#\r#k=v#\r## c\n"),
        (b" #\r#k=\n", b" #\r#k=\n"),
    ] {
        let (file, _) = LangFile::from_bytes(en_us(), bytes);
        let written = file.to_text().expect("a file that was read can be written");
        assert_eq!(written.as_bytes(), expected, "{:?}", bytes.escape_ascii());
        assert_eq!(LangFile::from_bytes(en_us(), written.as_bytes()).0, file);
    }
}
