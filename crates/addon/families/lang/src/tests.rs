//! Unit tests of what only the crate can reach: the walk of the translations, and the
//! keyword-only scan against the full reader.

use bedrock_addon_core::pack_identity::PackPath;

use super::{KeyFilter, LangFile, Language, RESERVED_KEYS};
use crate::scan::defines_pack_keywords;

fn lang(code: &str) -> Language {
    Language::new(code)
}

// ---- the walk ---------------------------------------------------------------------

#[test]
fn the_translations_walk_reads_the_lang_files_directly_in_texts() {
    use bedrock_addon_core::load::WalkOrder;

    let walk = crate::LOADER.walk;
    let reads = |path: &str| walk.matches(&PackPath::new(path).unwrap()).is_some();
    assert!(reads("texts/en_US.lang"));
    for path in [
        "texts/en_US.LANG",
        "texts/ja_JP/texts/en_US.lang",
        "texts/languages.json",
        "Texts/en_US.lang",
        "en_US.lang",
    ] {
        assert!(!reads(path), "{path}");
    }
    // lowest pack first, as `loadAllVersionsOf` returns the copies
    assert_eq!(walk.order, WalkOrder::BottomUp);
    assert!(walk.app_data.is_empty());
}

// ---- the keyword-only scan ------------------------------------------------

/// What the full reader finds: the oracle of the keyword-only scan.
fn keywords_by_the_reader(bytes: &[u8]) -> [bool; 2] {
    let (file, _) = LangFile::from_bytes(lang("en_US"), bytes);
    let found = file.select(&KeyFilter::pack_keywords(""));
    RESERVED_KEYS.map(|keyword| found.strings.contains_key(keyword))
}

/// Every `.lang` file below `dir`, recursively.
#[allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "a test reads the fixture files"
)]
fn lang_fixtures(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display())) {
        let path = entry.unwrap().path();
        if path.is_dir() {
            lang_fixtures(&path, out);
        } else if path
            .extension()
            .is_some_and(|extension| extension == "lang")
        {
            out.push(path);
        }
    }
}

#[test]
#[allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "a test reads the fixture files"
)]
fn the_keyword_scan_agrees_with_the_reader_on_every_fixture() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../addon/tests/fixtures/text/text");
    let mut files = Vec::new();
    lang_fixtures(&root, &mut files);
    files.sort();
    assert!(files.len() >= 50, "{} lang fixtures", files.len());
    let mut with_keywords = 0;
    for path in &files {
        let bytes = std::fs::read(path).unwrap();
        let expected = keywords_by_the_reader(&bytes);
        assert_eq!(
            defines_pack_keywords(&bytes),
            expected,
            "{}",
            path.display()
        );
        with_keywords += usize::from(expected != [false; 2]);
    }
    assert!(
        with_keywords >= 4,
        "{with_keywords} fixtures define a keyword"
    );
}

#[test]
fn the_keyword_scan_reads_a_keyword_line_as_the_reader_does() {
    for (bytes, expected) in [
        (&b"pack.name=X\n"[..], [true, false]),
        (b"pack.description=Y", [false, true]),
        (
            b"other=1\nPack.Name=X\r\nPACK.DESCRIPTION=Y\r\n",
            [true, true],
        ),
        // blanks before the key, a BOM, a NUL as a line end, a carriage return in the key
        (b" \tpack.name=X\n", [true, false]),
        (b"\xEF\xBB\xBFpack.name=X\n", [true, false]),
        (b"a=b\0pack.description=Y\n", [false, true]),
        (b"p\rack.name=X\n", [true, false]),
        // a carriage return before the key, at the start of the file and after a line feed
        (b"\rpack.name=X", [true, false]),
        (b"a=b\n\rpack.description=Y\n", [false, true]),
        (b"\r \r\tPack.Name=X\r\n", [true, false]),
        // a value with an escape, a comment after the value
        (b"pack.name=a\\nb\t## c\n", [true, false]),
        // not a definition: no value, no `=`, another key, a comment, the text after a value
        (b"pack.name=\n", [false, false]),
        (b"pack.name=\tX\n", [false, false]),
        (b"pack.name=## X\n", [false, false]),
        (b"pack.name\n", [false, false]),
        (b"pack.name =X\n", [false, false]),
        (b"pack.names=X\n", [false, false]),
        (b"my.pack.name=X\n", [false, false]),
        (b"## pack.name=X\n", [false, false]),
        (b"a=b ## pack.name=X\n", [false, false]),
        (b"a=b\tpack.description=Y\n", [false, false]),
        (b"=pack.name=X\n", [false, false]),
        (b"", [false, false]),
    ] {
        assert_eq!(
            keywords_by_the_reader(bytes),
            expected,
            "the reader: {:?}",
            bytes.escape_ascii()
        );
        assert_eq!(
            defines_pack_keywords(bytes),
            expected,
            "the scan: {:?}",
            bytes.escape_ascii()
        );
    }
}

// ---- the byte machine -------------------------------------------------------------

#[test]
fn the_scan_types_implement_debug() {
    fn assert_debug<T: std::fmt::Debug>() {}
    assert_debug::<crate::scan::End>();
    assert_debug::<crate::scan::Kind>();
    assert_debug::<crate::scan::Segment>();
}
