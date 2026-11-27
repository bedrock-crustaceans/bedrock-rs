//! Write-back at the Lines level: comments, blank lines and order are kept, a
//! byte order mark and the final line feed are written as they were read, CRLF is written as
//! LF, and a written file reads back to the same file and is written again byte for byte.

use bedrock_addon_core::json::{EncodeError, Unrepresentable};
use bedrock_addon_family_lang::{LangFile, Line};

use super::common::*;

/// Reads `bytes`, writes the file, and requires the written text to read back to the same
/// lines and to be written again identically. Returns the written text.
fn round_trip(bytes: &[u8]) -> String {
    let (file, _) = LangFile::from_bytes(en_us(), bytes);
    let written = file.to_text().expect("a file that was read can be written");
    let (again, _) = LangFile::from_bytes(en_us(), written.as_bytes());
    assert_eq!(
        again, file,
        "the written text reads back differently:\n{written}"
    );
    assert_eq!(
        again.to_text().unwrap(),
        written,
        "the second write differs from the first"
    );
    assert_eq!(
        again.translations(),
        file.translations(),
        "the table changed"
    );
    written
}

/// A file with these lines.
fn file(lines: Vec<Line>) -> LangFile {
    LangFile {
        lines,
        ..LangFile::new(en_us())
    }
}

/// `bytes` with every CRLF as LF: the one loss of the Lines level.
fn crlf_as_lf(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    for (index, byte) in bytes.iter().enumerate() {
        if *byte == b'\r' && bytes.get(index + 1) == Some(&b'\n') {
            continue;
        }
        out.push(*byte);
    }
    out
}

/// The fixtures that hold something a line does not keep, with what.
const NOT_BYTE_IDENTICAL: [(&str, &str); 7] = [
    ("comment_line.lang", "blanks before a `##` comment line"),
    ("crlf.lang", "a carriage return inside a value"),
    (
        "double_backslash.lang",
        "`\\\\` before an ordinary byte is written as one backslash",
    ),
    ("leading_space.lang", "blanks before a key"),
    ("nul_mid_file.lang", "a NUL is written as a line feed"),
    ("ut_leading_whitespace.lang", "blanks before a key"),
    ("ut_multiline.lang", "blanks before a key"),
];

/// An entry line with what follows its value.
fn entry(key: &str, value: &str, trailing: &str) -> Line {
    Line::Entry {
        key: key.to_owned(),
        value: value.to_owned(),
        trailing: trailing.to_owned(),
    }
}

#[test]
fn every_lang_fixture_round_trips() {
    let mut names: Vec<String> = std::fs::read_dir(fixtures().join("lang"))
        .unwrap()
        .filter_map(|entry| entry.unwrap().file_name().into_string().ok())
        .filter(|name| {
            std::path::Path::new(name)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("lang"))
        })
        .collect();
    names.sort();
    assert!(names.len() >= 35, "{} fixtures", names.len());
    let mut identical = 0;
    for name in &names {
        let bytes = fixture(&format!("lang/{name}"));
        let written = round_trip(&bytes);
        if NOT_BYTE_IDENTICAL.iter().any(|(file, _)| file == name) {
            assert_ne!(
                written.as_bytes(),
                crlf_as_lf(&bytes),
                "{name} is identical after all"
            );
        } else {
            assert_eq!(
                written.as_bytes(),
                crlf_as_lf(&bytes),
                "{name}: the written text differs from the input beyond CRLF"
            );
            identical += 1;
        }
    }
    assert_eq!(identical, names.len() - NOT_BYTE_IDENTICAL.len());
}

#[test]
fn comments_blank_lines_and_order_are_kept_byte_for_byte() {
    let text = fixture_text("lang/ut_multiline_error.lang");
    assert_eq!(round_trip(text.as_bytes()), text);
}

#[test]
fn the_layout_of_a_whole_file_is_kept_but_for_the_indentation() {
    let text = fixture_text("lang/ut_multiline.lang");
    // the one difference: blanks before a key are not part of the line
    assert_eq!(
        round_trip(text.as_bytes()),
        text.replace("\tkey.indented", "key.indented")
    );
}

#[test]
fn crlf_is_written_as_lf() {
    assert_eq!(
        round_trip(b"a=b\r\n\r\n## c\r\nd=e\t## f\r\n"),
        "a=b\n\n## c\nd=e\t## f\n"
    );
}

#[test]
fn a_bom_and_a_missing_final_line_feed_are_written_as_they_were_read() {
    assert_eq!(round_trip(b"\xEF\xBB\xBFa=b\nc=d"), "\u{feff}a=b\nc=d");
    assert_eq!(round_trip(b"\xEF\xBB\xBFa=b\nc=d\n"), "\u{feff}a=b\nc=d\n");
    assert_eq!(round_trip(b"a=b\nc=d"), "a=b\nc=d");
    assert_eq!(round_trip(b"a=b\r\nc=d\r\n"), "a=b\nc=d\n");
    // nothing but a byte order mark, an empty file, and only line feeds
    assert_eq!(round_trip(b"\xEF\xBB\xBF"), "\u{feff}");
    assert_eq!(round_trip(b""), "");
    assert_eq!(round_trip(b"\n"), "\n");
    assert_eq!(round_trip(b"a=b\n\n"), "a=b\n\n");
}

#[test]
fn the_reader_records_the_bom_and_the_final_line_feed() {
    let layout = |bytes: &[u8]| {
        let layout = LangFile::from_bytes(en_us(), bytes).0.layout;
        (layout.bom, layout.final_line_feed)
    };
    assert_eq!(layout(b"a=b\n"), (false, true));
    assert_eq!(layout(b"a=b"), (false, false));
    assert_eq!(layout(b"\xEF\xBB\xBFa=b\r\n"), (true, true));
    assert_eq!(layout(b"\xEF\xBB\xBFa=b"), (true, false));
    // a byte order mark anywhere else is not one
    assert_eq!(layout(b"a=b\n\xEF\xBB\xBF\n"), (false, true));
    // a file built in code: no byte order mark, the last line ended
    let built = LangFile::new(en_us()).layout;
    assert_eq!((built.bom, built.final_line_feed), (false, true));
}

#[test]
fn a_last_line_of_only_blanks_is_written_as_an_empty_line() {
    // the blanks of a blank line are not kept, so the line needs its line feed to
    // stay a line
    assert_eq!(round_trip(b"a=b\n  "), "a=b\n\n");
    assert_eq!(round_trip(b" \t"), "\n");
}

#[test]
fn a_first_line_that_starts_with_a_bom_character_is_written_after_a_bom() {
    // two byte order marks: the first is the file's, the second starts the key
    let bytes = b"\xEF\xBB\xBF\xEF\xBB\xBFa=b\n";
    let (read, _) = LangFile::from_bytes(en_us(), bytes);
    assert_eq!(read.lines, [Line::entry("\u{feff}a", "b")]);
    assert_eq!(round_trip(bytes).as_bytes(), bytes);
    // and a first line vanilla drops
    let dropped = b"\xEF\xBB\xBF\xEF\xBB\xBFno equals\n";
    let (read, _) = LangFile::from_bytes(en_us(), dropped);
    assert_eq!(read.lines, [Line::Dropped("\u{feff}no equals".to_owned())]);
    assert_eq!(round_trip(dropped).as_bytes(), dropped);
}

#[test]
fn without_a_bom_a_first_line_that_starts_with_a_bom_character_is_written_after_a_carriage_return()
{
    // written as it stands, the reader would take the character for the file's byte order
    // mark; a carriage return in front, which the reader ignores, keeps it a key byte
    for line in [
        Line::entry("\u{feff}a", "b"),
        Line::Dropped("\u{feff}no equals".to_owned()),
        Line::Dropped("\u{feff}".to_owned()),
    ] {
        let built = file(vec![line.clone(), Line::entry("ok", "fine")]);
        let written = built.to_text().unwrap();
        assert!(written.starts_with("\r\u{feff}"), "{written:?}");
        assert_eq!(LangFile::from_bytes(en_us(), written.as_bytes()).0, built);
        // with the file's own byte order mark in front it needs none
        let mut with_bom = built.clone();
        with_bom.layout.bom = true;
        let written = with_bom.to_text().unwrap();
        assert!(written.starts_with("\u{feff}\u{feff}"), "{written:?}");
        assert_eq!(
            LangFile::from_bytes(en_us(), written.as_bytes()).0,
            with_bom
        );
        // on any later line the character is an ordinary one
        let later = file(vec![Line::entry("ok", "fine"), line]);
        let written = later.to_text().unwrap();
        assert!(!written.contains('\r'));
        assert_eq!(LangFile::from_bytes(en_us(), written.as_bytes()).0, later);
    }
}

#[test]
fn a_file_whose_first_key_starts_with_a_bom_character_after_another_byte_is_writable() {
    // Bytes before the byte order mark make it a key, and such a file that
    // was read is written and reads back
    for bytes in [
        &b"\r\xEF\xBB\xBFa=b\n"[..],
        b"\0\xEF\xBB\xBFa=b\n",
        b" \xEF\xBB\xBFa=b\n",
        b"\r\n\xEF\xBB\xBFa=b\n",
    ] {
        let (read, _) = LangFile::from_bytes(en_us(), bytes);
        assert!(!read.layout.bom);
        assert!(
            read.lines.contains(&Line::entry("\u{feff}a", "b")),
            "{:?}",
            read.lines
        );
        let written = read.to_text().expect("a file that was read can be written");
        assert_eq!(
            LangFile::from_bytes(en_us(), written.as_bytes()).0,
            read,
            "{:?}",
            bytes.escape_ascii()
        );
    }
    // the first of them is written back byte for byte
    assert_eq!(round_trip(b"\r\xEF\xBB\xBFa=b\n"), "\r\u{feff}a=b\n");
}

#[test]
fn a_blank_last_line_is_always_ended() {
    // An empty last line is only a line with its line feed, whatever the
    // record says
    let mut built = file(vec![Line::entry("a", "b"), Line::Blank]);
    built.layout.final_line_feed = false;
    assert_eq!(built.to_text().unwrap(), "a=b\n\n");
    let mut unended = file(vec![Line::entry("a", "b"), Line::entry("c", "d")]);
    unended.layout.final_line_feed = false;
    assert_eq!(unended.to_text().unwrap(), "a=b\nc=d");
}

#[test]
fn a_nul_after_the_final_line_feed_does_not_drop_it() {
    // What follows the last line feed is no line, so the line feed is final
    assert_eq!(round_trip(b"a=b\n\0"), "a=b\n");
    assert_eq!(round_trip(b"a=b\n\r\0\r"), "a=b\n");
    assert_eq!(round_trip(b"a=b\r\n\0\0"), "a=b\n");
    // a NUL that ends the last line is written as nothing: the line stays unended
    assert_eq!(round_trip(b"a=b\0"), "a=b");
}

#[test]
fn a_value_is_escaped_only_where_it_would_read_differently() {
    // a line feed is written as `\n`
    assert_eq!(round_trip(b"a=x\\ny\n"), "a=x\\ny\n");
    // a backslash before an ordinary byte stays single
    assert_eq!(round_trip(b"a=C:\\path\\to\n"), "a=C:\\path\\to\n");
    // a backslash before `n` or another backslash is doubled, so it is not read as an escape
    assert_eq!(round_trip(b"a=x\\\\ny\n"), "a=x\\\\ny\n");
    // of two backslashes only the first stands before a backslash
    assert_eq!(round_trip(b"a=x\\\\\\\\y\n"), "a=x\\\\\\y\n");
    // a backslash at the end of the value needs no escape, before a comment either
    assert_eq!(round_trip(b"a=x\\"), "a=x\\");
    assert_eq!(round_trip(b"a=x\\\t# c\n"), "a=x\\\t# c\n");
    assert_eq!(round_trip(b"a=x\\## c\n"), "a=x\\## c\n");
}

#[test]
fn a_backslash_before_a_line_feed_in_a_value_is_doubled() {
    let built = file(vec![Line::entry("a", "x\\\ny")]);
    let written = built.to_text().unwrap();
    assert_eq!(written, "a=x\\\\\\ny\n");
    assert_eq!(LangFile::from_bytes(en_us(), written.as_bytes()).0, built);
}

#[test]
fn dropped_lines_are_written_as_they_were() {
    assert_eq!(
        round_trip(b"  no equals here\nempty=\nempty.too=\t## c\n"),
        "  no equals here\nempty=\nempty.too=\t## c\n"
    );
}

#[test]
fn the_parts_a_nul_separates_are_written_as_lines() {
    assert_eq!(round_trip(b"a=b\0c=d\ne"), "a=b\nc=d\ne");
}

#[test]
fn a_file_built_by_hand_is_written_and_reads_back() {
    let built = file(vec![
        Line::Comment(" Custom items".to_owned()),
        Line::Blank,
        Line::entry("item.demo:ruby.name", "Ruby"),
        entry("tile.demo:ore.name", "Ruby Ore", "\t## the block"),
        Line::entry("=odd", "kept"),
    ]);
    let written = built.to_text().unwrap();
    assert_eq!(
        written,
        "## Custom items\n\nitem.demo:ruby.name=Ruby\ntile.demo:ore.name=Ruby Ore\t## the block\n=odd=kept\n"
    );
    assert_eq!(LangFile::from_bytes(en_us(), written.as_bytes()).0, built);
    assert_eq!(LangFile::new(en_us()).to_text().unwrap(), "");
}

/// Whether writing `line` fails as something the format cannot hold, leaving the output
/// untouched.
fn unrepresentable(line: Line) -> bool {
    let mut out = b"before".to_vec();
    let result = file(vec![Line::entry("ok", "fine"), line]).write_text(&mut out);
    if result.is_err() {
        assert_eq!(out, b"before", "a failed write appended to the output");
    } else {
        assert!(
            out.starts_with(b"beforeok=fine\n"),
            "the output is appended to"
        );
    }
    matches!(
        result,
        Err(EncodeError::NotRepresentable {
            what: Unrepresentable::Unsupported,
            ..
        })
    )
}

#[test]
fn a_key_the_format_cannot_hold_is_refused() {
    for key in [
        "",
        " a",
        "\ta",
        "a=b",
        "a\nb",
        "a\rb",
        "a\0b",
        // a later `=` after a first character of more than one byte
        "\u{e9}=",
        "\u{e9}=b",
        "\u{4e2d}a=b",
        "\u{feff}a=b",
        "\u{feff}=",
    ] {
        assert!(unrepresentable(Line::entry(key, "v")), "{key:?}");
    }
    // one `#` and a leading `=` are ordinary key bytes
    for key in ["#a", "=a", "a b", "a\tb", "a#b", "a##b", "##a", "###"] {
        assert!(!unrepresentable(Line::entry(key, "v")), "{key:?}");
        reads_back(Line::entry(key, "v"));
    }
}

#[test]
fn a_value_the_format_cannot_hold_is_refused() {
    for value in ["", "a\tb", "a\rb", "a\0b"] {
        assert!(unrepresentable(Line::entry("k", value)), "{value:?}");
    }
    for value in [
        "a#b", "a=b", " a ", "a\nb", "\\", "a#", "a##b", "##", "###", "#\n#",
    ] {
        assert!(!unrepresentable(Line::entry("k", value)), "{value:?}");
        reads_back(Line::entry("k", value));
    }
}

/// A file with `line` between two others is written and reads back to itself.
fn reads_back(line: Line) {
    let built = file(vec![
        Line::entry("before", "x"),
        line,
        Line::entry("after", "y"),
    ]);
    let written = built.to_text().unwrap();
    assert_eq!(
        LangFile::from_bytes(en_us(), written.as_bytes()).0,
        built,
        "{written:?}"
    );
}

#[test]
fn a_double_pound_that_is_no_comment_is_written_with_a_carriage_return_between() {
    // the reader tests the byte right after a `#` as stored and drops the carriage return
    // afterwards, so `#\r#` is how a file holds `##` outside a comment
    assert_eq!(
        file(vec![Line::entry("k", "a##b")]).to_text().unwrap(),
        "k=a#\r#b\n"
    );
    assert_eq!(
        file(vec![Line::entry("k", "###")]).to_text().unwrap(),
        "k=#\r#\r#\n"
    );
    assert_eq!(
        file(vec![Line::entry("##k", "v")]).to_text().unwrap(),
        "#\r#k=v\n"
    );
    // a value that ends in `#` before a `##` comment
    assert_eq!(
        file(vec![entry("k", "v#", "## c")]).to_text().unwrap(),
        "k=v#\r## c\n"
    );
    // a line vanilla drops whose key starts with `##`: written as a comment it would read as one
    assert_eq!(
        file(vec![Line::Dropped(" ##no equals".to_owned())])
            .to_text()
            .unwrap(),
        " #\r#no equals\n"
    );
    reads_back(Line::Dropped("##no equals".to_owned()));
    reads_back(Line::Dropped("\t ##k=".to_owned()));
    reads_back(entry("k", "v#", "## c"));
    // a single `#`, and a `##` that is a comment, need none
    assert_eq!(
        file(vec![
            entry("k", "a#b", "## c"),
            Line::Comment("# d".to_owned())
        ])
        .to_text()
        .unwrap(),
        "k=a#b## c\n### d\n"
    );
}

#[test]
fn text_after_a_value_must_start_where_the_value_ends() {
    for trailing in ["c", " ## c", "#c", "\tc\nd", "## c\rd", "\t\0"] {
        assert!(unrepresentable(entry("k", "v", trailing)), "{trailing:?}");
    }
    for trailing in ["", "\t", "\tc", "\t## c", "## c", "##"] {
        assert!(!unrepresentable(entry("k", "v", trailing)), "{trailing:?}");
    }
    assert!(!unrepresentable(entry("k", "v#", "## c")));
    assert!(!unrepresentable(entry("k", "v#", "\t## c")));
}

#[test]
fn a_comment_or_dropped_line_cannot_span_lines() {
    for text in ["a\nb", "a\rb", "a\0b"] {
        assert!(unrepresentable(Line::Comment(text.to_owned())), "{text:?}");
        assert!(unrepresentable(Line::Dropped(text.to_owned())), "{text:?}");
    }
    assert!(!unrepresentable(Line::Comment(String::new())));
}

#[test]
fn a_dropped_line_must_read_back_as_a_dropped_line() {
    // The reader is the oracle. Text that would read as a blank line or as
    // an entry is no dropped line
    for text in [
        "",
        " ",
        " \t",
        "a=b",
        "anything = goes",
        "=a=b",
        "  k=v\t## c",
    ] {
        assert!(unrepresentable(Line::Dropped(text.to_owned())), "{text:?}");
    }
    // what vanilla drops: a key without `=`, an empty value, and a `##` that starts a key
    for text in [
        "no equals",
        " \tno equals",
        "k=",
        "k=\t## c",
        "k=## c",
        "=",
        "#",
        "##x",
        "## c",
        "\u{e9}t\u{e9}",
    ] {
        assert!(!unrepresentable(Line::Dropped(text.to_owned())), "{text:?}");
        reads_back(Line::Dropped(text.to_owned()));
    }
}

#[test]
fn a_key_with_a_first_character_of_several_bytes_is_written() {
    for key in [
        "\u{e9}",
        "\u{e9}t\u{e9}",
        "\u{4e2d}\u{6587}",
        "=\u{e9}",
        "\u{1f600}k",
    ] {
        assert!(!unrepresentable(Line::entry(key, "v")), "{key:?}");
        reads_back(Line::entry(key, "v"));
    }
}

#[test]
fn the_blanks_a_line_does_not_keep_deviation() {
    // the blanks of a line of only blanks, and the blanks before a `##` comment line
    // or before a key (vanilla skips all three), are not part of a line and are not written
    assert_eq!(
        round_trip(b" \t\n\t ## c\n  a=b\n \tno equals\n"),
        "\n## c\na=b\n \tno equals\n"
    );
    // a line vanilla drops is kept as written, its blanks included; so are the blanks
    // inside a key, a value, a comment and the text after a value
    assert_eq!(
        round_trip(b"a b = c d \t ## e \n##  f \n"),
        "a b = c d \t ## e \n##  f \n"
    );
}
