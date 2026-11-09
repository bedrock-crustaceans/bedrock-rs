//! `font/`: the font list, the emoticon list and the binary files, the round trips and the cost
//! of long lists. Parsing the font, scanning a character and drawing a text are no pack-file
//! rules.

#![allow(
    clippy::disallowed_methods,
    reason = "calls facet-json directly: serializes a value to compare it with the written text; not yet moved to the door"
)]

use bedrock_addon::diagnostic::{Diagnostic, DiagnosticDetail, Severity};
use bedrock_addon::document::{Content, Document, Placed};
use bedrock_addon::families::font::{
    EmoticonList, FontEntry, FontFormat, FontMetadata, GlyphPage, GlyphRemapping, PageSpec,
    RemapEntry, SMOOTH_PREFIX, check_glyph_sheet, png_size, remapping_path,
};
use bedrock_addon::json::last_wins;
use bedrock_addon::load::{LoadContext, LoadOptions};
use bedrock_addon::pack::{PackStack, Resource};
use bedrock_addon::storage::PackPath;

use super::common::{codes, json, located, pack_number, shown, stack_of, stack_with};

/// The paths a `FontFileMissing` lists as tried, each as its own element; none when the
/// detail is of another shape.
fn tried_paths(finding: &Diagnostic) -> Vec<String> {
    match finding.detail.as_deref() {
        Some(DiagnosticDetail::TriedPaths {
            kind: "font file",
            paths,
        }) => paths.clone(),
        _ => Vec::new(),
    }
}

fn fixture(path: &str) -> Vec<u8> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/ui/rp_misc");
    std::fs::read(root.join(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn metadata(text: &str) -> Document<FontMetadata> {
    FontMetadata::parse(
        &PackPath::from_static("font/font_metadata.json"),
        text.as_bytes(),
        &LoadContext::detached(),
    )
    .expect("a font list")
}

fn refused(text: &str) -> Vec<(&'static str, Severity, String)> {
    FontMetadata::parse(
        &PackPath::from_static("font/font_metadata.json"),
        text.as_bytes(),
        &LoadContext::detached(),
    )
    .expect_err("refused")
    .diagnostics
    .iter()
    .map(shown)
    .collect()
}

/// A font list with `fonts` and `aliases` as the two arrays' contents.
fn list(fonts: &str, aliases: &str) -> Document<FontMetadata> {
    metadata(&format!(
        r#"{{ "version": 1, "fonts": [{fonts}], "font_aliases": [{aliases}] }}"#
    ))
}

fn texts(doc: &Document<FontMetadata>) -> Vec<(String, Severity, String, String)> {
    doc.diagnostics()
        .iter()
        .map(|d| {
            (
                d.code.id().to_owned(),
                d.severity,
                d.location.json_path.to_string(),
                d.code.to_string(),
            )
        })
        .collect()
}

fn emoticons(text: &str) -> Document<EmoticonList> {
    EmoticonList::parse(
        &PackPath::from_static("font/emoticons.json"),
        text.as_bytes(),
        &LoadContext::detached(),
    )
    .expect("an emoticon list")
}

/// The start of a PNG file of the given size.
fn png(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = b"\x89PNG\r\n\x1a\n\x00\x00\x00\x0dIHDR".to_vec();
    bytes.extend(width.to_be_bytes());
    bytes.extend(height.to_be_bytes());
    bytes.extend([8, 6, 0, 0, 0]);
    bytes
}

/// The findings about font files as `(code, severity, pack, path)`.
fn file_findings(
    stack: &PackStack<Resource>,
    count: usize,
) -> Vec<(&'static str, Severity, usize, String)> {
    stack
        .fonts()
        .diagnostics()
        .map(|d: &Diagnostic| {
            (
                d.code.id(),
                d.severity,
                pack_number(d, count).unwrap(),
                d.location.path.as_ref().unwrap().to_string(),
            )
        })
        .collect()
}

const TTF: &str = r#"{ "font_format": "ttf", "font_name": "T", "font_file": "font/t" }"#;

// ---- glyph sheets ---------------------------------------------------------------------------

#[test]
fn fo01_glyph_page_names() {
    let stack = stack_of(&[&[
        ("font/glyph_4E.png", ""),
        ("font/glyph_4e.png", ""),
        ("font/glyph_100.png", ""),
        ("font/glyph_0A", ""),
        ("font/glyph_G0.tga", ""),
        ("font/default8.png", ""),
        ("font/sub/glyph_zz.png", ""),
        ("texts/ja_JP/font/glyph_3.png", ""),
        ("texts/ja_JP/other/glyph_3.png", ""),
    ]]);
    assert_eq!(
        file_findings(&stack, 1),
        [
            (
                "core/ignored_file",
                Severity::Info,
                0,
                "font/glyph_100.png".to_owned()
            ),
            (
                "core/ignored_file",
                Severity::Info,
                0,
                "font/glyph_4e.png".to_owned()
            ),
            (
                "core/ignored_file",
                Severity::Info,
                0,
                "font/glyph_G0.tga".to_owned()
            ),
            (
                "core/ignored_file",
                Severity::Info,
                0,
                "texts/ja_JP/font/glyph_3.png".to_owned()
            ),
        ]
    );
    let fonts = stack.fonts();
    assert_eq!(
        fonts
            .glyph_sheet("font/glyph_", 0x4E, None)
            .unwrap()
            .path
            .as_str(),
        "font/glyph_4E.png"
    );
    assert_eq!(
        fonts
            .glyph_sheet("font/glyph_", 0x0A, None)
            .unwrap()
            .path
            .as_str(),
        "font/glyph_0A"
    );
    assert!(fonts.glyph_sheet("font/glyph_", 0x4F, None).is_none());
}

#[test]
fn fo02_a_language_s_sheet_stands_before_the_pack_s_own() {
    let stack = stack_of(&[&[
        ("font/glyph_30.png", ""),
        ("texts/ja_JP/font/glyph_30.png", ""),
    ]]);
    let fonts = stack.fonts();
    let sheet = |language| {
        fonts
            .glyph_sheet("font/glyph_", 0x30, language)
            .unwrap()
            .path
            .as_str()
    };
    assert_eq!(sheet(Some("ja_JP")), "texts/ja_JP/font/glyph_30.png");
    assert_eq!(sheet(Some("en_US")), "font/glyph_30.png");
    assert_eq!(sheet(None), "font/glyph_30.png");
    assert_eq!(codes(fonts.diagnostics()), []);
}

#[test]
fn fo02_a_higher_pack_s_own_sheet_stands_before_a_lower_pack_s_language_sheet() {
    let low: &[(&str, &str)] = &[
        ("texts/ja_JP/font/glyph_30.png", ""),
        ("font/glyph_30.png", ""),
    ];
    let own: &[(&str, &str)] = &[("font/glyph_30.tga", "")];
    let both: &[(&str, &str)] = &[("texts/ja_JP/font/glyph_30.png", "")];
    let stack = stack_of(&[low, own]);
    let sheet = stack
        .fonts()
        .glyph_sheet("font/glyph_", 0x30, Some("ja_JP"))
        .unwrap();
    assert_eq!(
        (sheet.pack.index(), sheet.path.as_str()),
        (Some(0), "font/glyph_30.tga")
    );
    // Three packs: the top one has only the language's sheet.
    let stack = stack_of(&[low, own, both]);
    let fonts = stack.fonts();
    let sheet = fonts
        .glyph_sheet("font/glyph_", 0x30, Some("ja_JP"))
        .unwrap();
    assert_eq!(
        (sheet.pack.index(), sheet.path.as_str()),
        (Some(0), "texts/ja_JP/font/glyph_30.png")
    );
    let sheet = fonts.glyph_sheet("font/glyph_", 0x30, None).unwrap();
    assert_eq!(
        (sheet.pack.index(), sheet.path.as_str()),
        (Some(1), "font/glyph_30.tga")
    );
}

#[test]
fn fo03_fo25_sheet_shapes() {
    let shape = |width, height| codes(&check_glyph_sheet(&png(width, height)));
    assert_eq!(
        shape(200, 256),
        [("core/glyph_sheet_shape", Severity::Info)]
    );
    assert_eq!(
        shape(256, 200),
        [("core/glyph_sheet_shape", Severity::Info)]
    );
    assert_eq!(
        shape(200, 200),
        [("core/glyph_sheet_shape", Severity::Info)],
        "200 is no multiple of 16"
    );
    assert_eq!(shape(0, 0), [("core/glyph_sheet_shape", Severity::Info)]);
    for side in [16, 128, 256, 512, 1024] {
        assert_eq!(shape(side, side), [], "{side}");
    }
    assert_eq!(png_size(&png(512, 128)), Some((512, 128)));
    assert_eq!(png_size(b"\xff\xd8\xff\xe0 jpeg"), None);
    assert_eq!(
        codes(&check_glyph_sheet(b"\xff\xd8\xff\xe0 jpeg")),
        [],
        "not judged"
    );
    assert_eq!(png_size(&png(1, 1)[..20]), None, "a cut header");
}

// ---- font_metadata.json ---------------------------------------------------------------------

#[test]
fn fo04_a_bitmap_and_a_ttf_font() {
    let doc = list(
        &format!(
            r#"{{ "font_format": "bitmap", "font_name": "B", "ascii_font_file": "font/b" }}, {TTF}"#
        ),
        "",
    );
    assert_eq!(codes(doc.diagnostics()), []);
    let fonts = doc.fonts.as_ref().unwrap();
    assert_eq!(fonts.len(), 2);
    assert_eq!(
        (fonts[0].format(), fonts[1].format()),
        (Some(FontFormat::Bitmap), Some(FontFormat::Ttf))
    );
    assert!(fonts.iter().all(FontEntry::is_low_performance_compatible));
    assert_eq!(fonts[1].low_performance_compatible, None);
    assert_eq!(doc.font("T").unwrap().font_file.as_deref(), Some("font/t"));
}

#[test]
fn fo05_fo08_a_version_that_is_not_the_integer_one() {
    for version in ["2", "\"1\"", "1.0", "0", "null", "true", "[1]"] {
        let found = refused(&format!(r#"{{ "version": {version}, "fonts": [{TTF}] }}"#));
        assert_eq!(
            found,
            [(
                "font/version",
                Severity::Info,
                "Unsupported version number.".to_owned()
            )],
            "{version}"
        );
    }
    assert_eq!(refused(r#"{ "fonts": [] }"#)[0].0, "font/version", "absent");
    assert_eq!(
        refused("[1]")[0].0,
        "font/version",
        "a root that is no object"
    );
    assert_eq!(codes(metadata(r#"{ "version": 1 }"#).diagnostics()), []);
}

#[test]
fn f2_a_font_list_that_is_not_json() {
    assert_eq!(
        refused("{ nope"),
        [(
            "font/not_json",
            Severity::Info,
            "Couldn't parse font_metadata.json".to_owned()
        )]
    );
}

#[test]
fn fo06_fo11_entries_without_their_required_members() {
    let doc = list(
        r#"{ "font_name": "a" }, { "font_format": "ttf" }, { "font_format": 5, "font_name": "c" },
           { "font_format": "ttf", "font_name": 7 }, 3,
           { "font_format": "bitmap", "font_name": "e" },
           { "font_format": "bitmap", "font_name": "e2", "ascii_font_file": 4, "unicode_file_prefix": null },
           { "font_format": "ttf", "font_name": "f" },
           { "font_format": "ttfmsdf", "font_name": "g", "font_file": 1 },
           { "font_format": "msdf", "font_name": "h" }"#,
        "",
    );
    let warning = Severity::Info;
    let missing =
        |id: &str, at: &str, text: &str| (id.to_owned(), warning, at.to_owned(), text.to_owned());
    assert_eq!(
        texts(&doc),
        [
            missing(
                "font/no_format",
                "fonts/0/font_format",
                "Font is missing font_format."
            ),
            missing(
                "font/no_name",
                "fonts/1/font_name",
                "Font is missing font_name."
            ),
            missing(
                "font/no_format",
                "fonts/2/font_format",
                "Font is missing font_format."
            ),
            missing(
                "font/no_name",
                "fonts/3/font_name",
                "Font is missing font_name."
            ),
            missing(
                "font/no_format",
                "fonts/4/font_format",
                "Font is missing font_format."
            ),
            missing(
                "font/no_bitmap_file",
                "fonts/5",
                "Bitmap font must have either an ascii_font_file or a unicode_file_prefix specified"
            ),
            missing(
                "font/no_bitmap_file",
                "fonts/6",
                "Bitmap font must have either an ascii_font_file or a unicode_file_prefix specified"
            ),
            missing(
                "font/no_ttf_file",
                "fonts/7/font_file",
                "TTF is missing font_file."
            ),
            missing(
                "font/no_ttf_file",
                "fonts/8/font_file",
                "TTF is missing font_file."
            ),
            missing(
                "font/no_msdf_file",
                "fonts/9/font_file",
                "MSDF is missing font_file."
            ),
        ]
    );
    let names: Vec<&str> = doc
        .fonts
        .as_ref()
        .unwrap()
        .iter()
        .map(|f| f.font_name.as_str())
        .collect();
    assert_eq!(
        names,
        ["e", "e2", "f", "g", "h"],
        "the first five are skipped"
    );
    // A bitmap font needs only one of the two.
    for member in ["ascii_font_file", "unicode_file_prefix"] {
        let doc = list(
            &format!(r#"{{ "font_format": "bitmap", "font_name": "b", "{member}": "font/x" }}"#),
            "",
        );
        assert_eq!(codes(doc.diagnostics()), [], "{member}");
    }
}

#[test]
fn f6b_a_path_too_long_for_the_game_s_buffer_is_left_empty() {
    for (format, member, also) in [
        ("bitmap", "ascii_font_file", "font/no_bitmap_file"),
        ("bitmap", "unicode_file_prefix", "font/no_bitmap_file"),
        ("msdf", "font_file", "font/no_msdf_file"),
    ] {
        let entry = |length: usize| {
            format!(
                r#"{{ "font_format": "{format}", "font_name": "n", "{member}": "{}" }}"#,
                "p".repeat(length)
            )
        };
        assert_eq!(
            codes(list(&entry(1023), "").diagnostics()),
            [],
            "{member}: 1023 bytes fit"
        );
        let doc = list(&entry(1024), "");
        assert_eq!(
            located(doc.diagnostics())[0],
            (
                "core/invalid_value",
                Severity::Info,
                format!("fonts/0/{member}")
            ),
        );
        assert_eq!(
            codes(doc.diagnostics())[1].0,
            also,
            "{member}: the font then has no file"
        );
        let font = &doc.fonts.as_ref().unwrap()[0];
        assert_eq!(
            (
                &font.font_file,
                &font.ascii_font_file,
                &font.unicode_file_prefix
            ),
            (&None, &None, &None)
        );
        assert_eq!(
            json(&doc.to_json().unwrap()),
            json(&format!(
                r#"{{ "version": 1, "fonts": [{}], "font_aliases": [] }}"#,
                entry(1024)
            ))
        );
    }
}

#[test]
fn f8b_an_unknown_font_format_loads_nothing() {
    for format in ["BITMAP", "truetype", "", "msdf "] {
        let doc = list(
            &format!(r#"{{ "font_format": "{format}", "font_name": "n" }}"#),
            "",
        );
        assert_eq!(
            located(doc.diagnostics()),
            [(
                "core/invalid_value",
                Severity::Info,
                "fonts/0/font_format".to_owned()
            )]
        );
        assert_eq!(doc.fonts.as_ref().unwrap()[0].format(), None);
    }
}

#[test]
fn fo08b_generate_pages_on_start() {
    let doc = list(
        r#"{ "font_format": "ttf", "font_name": "a", "font_file": "f",
             "generate_pages_on_start": [0, { "first_page": 2, "last_page": 4 },
               { "first_page": 5, "last_page": 3 }, "x", { "first_page": 1.0, "last_page": 2 },
               { "first_page": 7, "last_page": 7 }, 2.5, { "last_page": 2 }] },
           { "font_format": "ttf", "font_name": "b", "font_file": "f",
             "generate_pages_on_start": { "first_page": 0, "last_page": 1 } }"#,
        "",
    );
    let fonts = doc.fonts.as_ref().unwrap();
    assert_eq!(fonts[0].start_pages(), [0..=0, 2..=4, 7..=7]);
    assert_eq!(fonts[1].start_pages(), [], "the member is ignored");
    assert_eq!(fonts[1].generate_pages_on_start, None);
    let invalid = |at: &str, text: &str| {
        (
            if text.starts_with("Invalid format") {
                "font/pages_format"
            } else {
                "core/invalid_value"
            }
            .to_owned(),
            Severity::Info,
            at.to_owned(),
            text.to_owned(),
        )
    };
    let pages = "Invalid format for the generate_pages_on_start json object.";
    assert_eq!(
        texts(&doc),
        [
            invalid("fonts/0/generate_pages_on_start/2", "invalid value"),
            invalid("fonts/0/generate_pages_on_start/3", pages),
            invalid("fonts/0/generate_pages_on_start/4", "invalid value"),
            invalid("fonts/0/generate_pages_on_start/6", pages),
            invalid("fonts/0/generate_pages_on_start/7", "invalid value"),
            invalid("fonts/1/generate_pages_on_start", "invalid value"),
        ]
    );
    assert_eq!(
        fonts[0].generate_pages_on_start.as_ref().unwrap()[0],
        PageSpec::Page(0)
    );
}

#[test]
fn fo08c_alias_failures() {
    let doc = list(
        TTF,
        r#"{ "alias": "A", "fonts": [{ "font_reference": "T" }, { "font_scale": 1.0 }] },
           { "alias": "B", "fonts": [{ "font_reference": "T", "font_ranges": [{ "first": 0, "last": "9" }] }] },
           { "alias": "C", "fonts": [{ "font_reference": "T", "font_ranges": 5, "font_language_code": 3 }] },
           { "alias": "D", "fonts": [] },
           { "fonts": [{ "font_reference": "T" }] },
           { "alias": "E", "fonts": { "font_reference": "T" } },
           { "alias": "F", "fonts": [{ "font_reference": 4 }] },
           { "alias": "G", "fonts": [{ "font_reference": "T", "font_ranges": null, "font_language_code": null }] },
           { "alias": "H", "fonts": [{ "font_reference": "T", "font_ranges": [{ "first": 1.0, "last": 2 }] }] }"#,
    );
    let aliases = doc.font_aliases.as_ref().unwrap();
    let names: Vec<&str> = aliases.iter().map(|a| a.alias.as_str()).collect();
    assert_eq!(names, ["C", "G"]);
    let c = &aliases[0].fonts[0];
    assert_eq!(
        (c.font_ranges.as_ref(), c.font_language_code.as_deref()),
        (None, Some(""))
    );
    let g = &aliases[1].fonts[0];
    assert_eq!(
        (g.font_ranges.as_ref(), g.font_language_code.as_deref()),
        (None, None)
    );
    let found: Vec<(String, String, String)> = texts(&doc)
        .into_iter()
        .map(|(code, _, at, text)| (code, at, text))
        .collect();
    let want = [
        (
            "font/no_reference",
            "font_aliases/0/fonts/1/font_reference",
            "Font alias missing font_reference",
        ),
        (
            "font/range_bounds",
            "font_aliases/1/fonts/0/font_ranges/0",
            "Font alias font_ranges first and last values must be integers.",
        ),
        (
            "font/ranges",
            "font_aliases/2/fonts/0/font_ranges",
            "Font alias font_ranges must be an array",
        ),
        (
            "font/language_code",
            "font_aliases/2/fonts/0/font_language_code",
            "Font alias font_language_code must be a string",
        ),
        (
            "core/invalid_value",
            "font_aliases/3/fonts",
            "invalid value",
        ),
        (
            "font/no_alias",
            "font_aliases/4/alias",
            "Font Alias missing alias.",
        ),
        (
            "font/alias_fonts",
            "font_aliases/5/fonts",
            "Font Alias property fonts must be an array.",
        ),
        (
            "font/no_reference",
            "font_aliases/6/fonts/0/font_reference",
            "Font alias missing font_reference",
        ),
        (
            "font/range_bounds",
            "font_aliases/8/fonts/0/font_ranges/0",
            "Font alias font_ranges first and last values must be integers.",
        ),
    ];
    assert_eq!(
        found,
        want.map(|(a, b, c)| (a.to_owned(), b.to_owned(), c.to_owned()))
    );
    assert!(
        doc.diagnostics()
            .iter()
            .all(|d| d.severity == Severity::Info)
    );
    // A dropped alias is no font a later alias can name.
    let doc = list(
        TTF,
        r#"{ "alias": "D", "fonts": [] }, { "alias": "X", "fonts": [{ "font_reference": "D" }] }"#,
    );
    assert_eq!(
        located(doc.diagnostics())[1].2,
        "font_aliases/1/fonts/0/font_reference"
    );
}

#[test]
fn fo08d_fo19b_low_performance_compatible_is_converted() {
    for (written, effective, typed) in [
        ("true", true, true),
        ("false", false, true),
        ("\"false\"", true, false),
        ("\"\"", false, false),
        ("0", false, false),
        ("2", true, false),
        ("0.0", false, false),
        ("null", true, false),
        ("[]", false, false),
        ("[0]", true, false),
        ("{}", false, false),
    ] {
        let doc = list(
            &format!(
                r#"{{ "font_format": "ttf", "font_name": "n", "font_file": "f", "lowPerformanceCompatible": {written} }}"#
            ),
            "",
        );
        let font = &doc.fonts.as_ref().unwrap()[0];
        assert_eq!(font.is_low_performance_compatible(), effective, "{written}");
        assert_eq!(
            font.low_performance_compatible.is_some(),
            typed,
            "{written}"
        );
        let want: &[_] = if typed {
            &[]
        } else {
            &[(
                "core/invalid_value",
                Severity::Info,
                "fonts/0/lowPerformanceCompatible".to_owned(),
            )]
        };
        assert_eq!(located(doc.diagnostics()), want, "{written}");
    }
}

#[test]
fn fo09_f18_a_scale_must_be_written_as_a_decimal_deviation() {
    let doc = list(
        TTF,
        r#"{ "alias": "a", "font_scale_factor_override": 1.5, "fonts": [{ "font_reference": "T", "font_scale": 0.65 }] },
           { "alias": "b", "font_scale_factor_override": 2, "fonts": [{ "font_reference": "T", "font_scale": 1 }] },
           { "alias": "c", "font_scale_factor_override": "1.5", "fonts": [{ "font_reference": "T", "font_scale": null }] },
           { "alias": "d", "font_scale_factor_override": null, "fonts": [{ "font_reference": "T", "font_scale": 2.0 }] }"#,
    );
    let aliases = doc.font_aliases.as_ref().unwrap();
    assert_eq!(aliases[0].font_scale_factor_override, Some(1.5));
    assert_eq!(aliases[0].fonts[0].font_scale, Some(0.65));
    assert_eq!(
        aliases[1].font_scale_factor_override, None,
        "an integer is ignored"
    );
    assert_eq!(aliases[1].fonts[0].font_scale, None);
    assert_eq!(aliases[3].fonts[0].font_scale, Some(2.0));
    assert_eq!(
        located(doc.diagnostics()),
        [
            (
                "core/invalid_value",
                Severity::Info,
                "font_aliases/1/fonts/0/font_scale".to_owned()
            ),
            (
                "core/invalid_value",
                Severity::Info,
                "font_aliases/1/font_scale_factor_override".to_owned()
            ),
            (
                "core/invalid_value",
                Severity::Info,
                "font_aliases/2/font_scale_factor_override".to_owned()
            ),
        ]
    );
    // What the game ignores is written back as it was, and a decimal stays a decimal.
    let again = metadata(&doc.to_json().unwrap());
    assert_eq!(again.value(), doc.value());
    assert_eq!(located(again.diagnostics()), located(doc.diagnostics()));
}

// The game's documentation does not say what the game makes of a scale beyond a float.
#[test]
fn a_scale_beyond_a_float_is_the_largest_finite_float_deviation() {
    for (huge, sign) in [
        ("1e999", 1.0),
        ("-1e999", -1.0),
        ("1e39", 1.0),
        ("-1e39", -1.0),
    ] {
        let doc = list(
            TTF,
            &format!(
                r#"{{ "alias": "a", "font_scale_factor_override": {huge}, "fonts": [{{ "font_reference": "T", "font_scale": {huge} }}] }}"#
            ),
        );
        let limit = f64::from(f32::MAX) * sign;
        let alias = &doc.font_aliases.as_ref().unwrap()[0];
        assert_eq!(alias.font_scale_factor_override, Some(limit), "{huge}");
        assert_eq!(alias.fonts[0].font_scale, Some(limit), "{huge}");
        assert_eq!(
            located(doc.diagnostics()),
            // `ClampedValue`: a finding of its own, not F18's `InvalidValue`.
            [
                (
                    "core/clamped_value",
                    Severity::Info,
                    "font_aliases/0/fonts/0/font_scale".to_owned()
                ),
                (
                    "core/clamped_value",
                    Severity::Info,
                    "font_aliases/0/font_scale_factor_override".to_owned()
                ),
            ],
            "{huge}"
        );
        let again = metadata(&doc.to_json().expect("writes"));
        assert_eq!(again.value(), doc.value(), "{huge}");
        assert_eq!(
            codes(again.diagnostics()),
            [],
            "the written number is within a float"
        );
    }
    // The largest float itself is no finding.
    let doc = list(
        TTF,
        r#"{ "alias": "a", "font_scale_factor_override": 3.4028234663852886e38, "fonts": [{ "font_reference": "T" }] }"#,
    );
    assert_eq!(codes(doc.diagnostics()), []);
}

#[test]
fn fo10_ttf_defaults_deviation() {
    let doc = list(
        r#"{ "font_format": "ttf", "font_name": "a", "font_file": "f" },
           { "font_format": "ttf", "font_name": "b", "font_file": "f", "version": "2", "target_font_render_size": 32.0 },
           { "font_format": "ttfmsdf", "font_name": "c", "font_file": "f", "version": 3, "target_font_render_size": "x" },
           { "font_format": "msdf", "font_name": "d", "font_file": "f", "version": "9" }"#,
        "",
    );
    let fonts = doc.fonts.as_ref().unwrap();
    assert_eq!((fonts[0].ttf_version(), fonts[0].render_size()), (1, 64));
    assert_eq!((fonts[1].ttf_version(), fonts[1].render_size()), (1, 64));
    assert_eq!(fonts[2].ttf_version(), 3);
    assert_eq!(
        located(doc.diagnostics()),
        [
            (
                "core/invalid_value",
                Severity::Info,
                "fonts/1/version".to_owned()
            ),
            (
                "core/invalid_value",
                Severity::Info,
                "fonts/1/target_font_render_size".to_owned()
            ),
        ],
        "the size is a ttf member only, the version a TrueType one"
    );
}

#[test]
fn fo20_the_render_size_is_narrowed_to_a_byte_deviation() {
    for (size, effective, reported) in [
        (300, 44, true),
        (256, 0, true),
        (255, 255, false),
        (1, 1, false),
        (0, 0, true),
        (-1, 255, true),
    ] {
        let doc = list(
            &format!(
                r#"{{ "font_format": "ttf", "font_name": "a", "font_file": "f", "target_font_render_size": {size} }}"#
            ),
            "",
        );
        assert_eq!(
            doc.fonts.as_ref().unwrap()[0].render_size(),
            effective,
            "{size}"
        );
        let want: &[_] = if reported {
            &[(
                "core/invalid_value",
                Severity::Info,
                "fonts/0/target_font_render_size".to_owned(),
            )]
        } else {
            &[]
        };
        assert_eq!(located(doc.diagnostics()), want, "{size}");
    }
}

#[test]
fn fo21_the_vanilla_font_list() {
    // crates/addon/addon/tests/fixtures/ui/rp_misc/fo21_alias_unknown_ref: gdk-retail 1.26.45
    // resource_packs/vanilla/font/font_metadata.json, unchanged.
    let bytes = fixture("fo21_alias_unknown_ref/pack/font/font_metadata.json");
    let doc = metadata(std::str::from_utf8(&bytes).unwrap());
    assert_eq!(
        located(doc.diagnostics()),
        [
            (
                "core/invalid_value",
                Severity::Info,
                "font_aliases/3/font_scale_factor_override".to_owned()
            ),
            (
                "core/invalid_value",
                Severity::Info,
                "font_aliases/2/fonts/1/font_reference".to_owned()
            ),
        ]
    );
    assert_eq!(doc.fonts.as_ref().unwrap().len(), 2);
    assert_eq!(doc.font_aliases.as_ref().unwrap().len(), 4);
    assert!(
        !doc.font("MinecraftTen")
            .unwrap()
            .is_low_performance_compatible()
    );
    assert_eq!(
        doc.alias("SmoothFontLatin").unwrap().fonts[1]
            .font_ranges
            .as_ref()
            .unwrap()
            .len(),
        7
    );
    assert_eq!(
        doc.alias("SmoothWithEmoticons").unwrap().fonts[1].font_scale,
        Some(0.65)
    );
    // The round trip: the written text reads back to the same list and writes the same text.
    let written = doc.to_json().unwrap();
    assert_eq!(json(&written), json(std::str::from_utf8(&bytes).unwrap()));
    assert_eq!(metadata(&written).to_json().unwrap(), written);
}

#[test]
fn fo22_f29_a_repeated_name_is_the_later_entry() {
    let doc = list(
        r#"{ "font_format": "bitmap", "font_name": "A", "ascii_font_file": "x" },
           { "font_format": "ttf", "font_name": "A", "font_file": "y" },
           { "font_format": "ttf", "font_name": "UIFont", "font_file": "y" }"#,
        r#"{ "alias": "L", "fonts": [{ "font_reference": "A" }] },
           { "alias": "L", "fonts": [{ "font_reference": "L" }] },
           { "alias": "A", "fonts": [{ "font_reference": "UIFont" }] },
           { "alias": "RuneFont", "fonts": [{ "font_reference": "A" }] }"#,
    );
    assert_eq!(doc.font("A").unwrap().format(), Some(FontFormat::Ttf));
    assert_eq!(doc.alias("L").unwrap().fonts[0].font_reference, "L");
    assert_eq!(
        located(doc.diagnostics()),
        [
            (
                "core/font_name_overridden",
                Severity::Info,
                "fonts/1/font_name".to_owned()
            ),
            (
                "core/font_name_overridden",
                Severity::Info,
                "font_aliases/1/alias".to_owned()
            ),
            (
                "core/font_name_overridden",
                Severity::Info,
                "font_aliases/2/alias".to_owned()
            ),
        ],
        "a name equal to a built-in font is not reported"
    );
}

/// F29 is reported at the entry's place in the file, whatever the game skipped before it.
#[test]
fn f29_is_reported_at_the_file_s_index() {
    let doc = list(
        r#"3, { "font_format": "ttf", "font_name": "A", "font_file": "y" }, { "font_name": "A" },
           { "font_format": "ttf", "font_name": "A", "font_file": "z" }"#,
        "",
    );
    let found = located(doc.diagnostics());
    assert_eq!(
        found.last().unwrap(),
        &(
            "core/font_name_overridden",
            Severity::Info,
            "fonts/3/font_name".to_owned()
        ),
        "{found:?}"
    );
    // Of an object `fonts`, the place is the member.
    let doc = metadata(
        r#"{ "version": 1, "fonts": { "a": 3, "b": { "font_format": "ttf", "font_name": "A", "font_file": "y" }, "c": { "font_format": "ttf", "font_name": "A", "font_file": "z" } } }"#,
    );
    let found = located(doc.diagnostics());
    assert_eq!(
        found.last().unwrap(),
        &(
            "core/font_name_overridden",
            Severity::Info,
            "fonts/c/font_name".to_owned()
        ),
        "{found:?}"
    );
}

#[test]
fn fo23_f28_a_reference_names_a_font_registered_before_it_deviation() {
    let doc = list(
        TTF,
        r#"{ "alias": "X", "fonts": [{ "font_reference": "Y" }, { "font_reference": "t" }, { "font_reference": "X" }] },
           { "alias": "Y", "fonts": [{ "font_reference": "X" }, { "font_reference": "T" }, { "font_reference": "UnicodeFont" }] }"#,
    );
    assert_eq!(
        located(doc.diagnostics()),
        [
            (
                "core/invalid_value",
                Severity::Info,
                "font_aliases/0/fonts/0/font_reference".to_owned()
            ),
            (
                "core/invalid_value",
                Severity::Info,
                "font_aliases/0/fonts/1/font_reference".to_owned()
            ),
            (
                "core/invalid_value",
                Severity::Info,
                "font_aliases/0/fonts/2/font_reference".to_owned()
            ),
        ],
        "a later alias, another case, and the alias itself name no font"
    );
    for built_in in bedrock_addon::families::font::BUILT_IN_FONTS {
        let doc = list(
            "",
            &format!(r#"{{ "alias": "a", "fonts": [{{ "font_reference": "{built_in}" }}] }}"#),
        );
        assert_eq!(codes(doc.diagnostics()), [], "{built_in}");
    }
}

#[test]
fn the_two_lists_are_iterated_as_the_game_does_and_unknown_members_are_kept() {
    let text = format!(
        r#"{{ "version": 1, "note": 1, "fonts": {{ "b": {TTF}, "a": {{ "font_format": "msdf", "font_name": "M", "font_file": "p_", "extra": [2] }} }}, "font_aliases": 4 }}"#
    );
    let doc = metadata(&text);
    let names: Vec<&str> = doc
        .fonts
        .as_ref()
        .unwrap()
        .iter()
        .map(|f| f.font_name.as_str())
        .collect();
    assert_eq!(names, ["M", "T"], "an object's values, by key");
    assert_eq!(doc.font_aliases.as_ref().unwrap().len(), 0);
    assert_eq!(
        located(doc.diagnostics()),
        [
            (
                "core/unrecognized_field",
                Severity::Info,
                "fonts/a/extra".to_owned()
            ),
            ("core/unrecognized_field", Severity::Info, "note".to_owned()),
        ]
    );
    assert_eq!(doc.extra.get("note"), Some(&json("1")));
    assert_eq!(
        doc.fonts.as_ref().unwrap()[0].extra.get("extra"),
        Some(&json("[2]"))
    );
}

#[test]
fn font_files_are_placed_below_font() {
    assert_eq!(
        FontMetadata::default().default_path().as_str(),
        "font/font_metadata.json"
    );
    assert_eq!(
        EmoticonList::default().default_path().as_str(),
        "font/emoticons.json"
    );
    assert_eq!(
        json(&FontMetadata::default().to_json().unwrap()),
        json(r#"{ "version": 1 }"#)
    );
    assert_eq!(EmoticonList::default().to_json().unwrap().trim(), "[]");
}

/// Values a generated member takes, as JSON text.
const GENERATED_VALUES: [&str; 16] = [
    "1",
    "1.5",
    "2.0",
    "-3",
    "\"s\"",
    "\"\"",
    "true",
    "false",
    "null",
    "[]",
    "{}",
    "[1, {\"first_page\": 1, \"last_page\": 3, \"k\": 0}, \"x\", {\"first_page\": 4, \"last_page\": 2}]",
    "[{\"first\": 1, \"last\": 2}]",
    "[{\"first\": 1}]",
    "1e39",
    "1e999",
];
const GENERATED_FONT_MEMBERS: [&str; 8] = [
    "lowPerformanceCompatible",
    "generate_pages_on_start",
    "ascii_font_file",
    "unicode_file_prefix",
    "font_file",
    "version",
    "target_font_render_size",
    "other",
];
const GENERATED_ALIAS_MEMBERS: [&str; 6] = [
    "font_scale_factor_override",
    "font_language_code",
    "font_ranges",
    "font_scale",
    "font_reference",
    "other",
];

/// A few thousand generated font lists: each that loads writes text that loads back to an
/// equal list with the same findings, and writes the same text again.
#[test]
fn generated_font_lists_round_trip() {
    let (mut count, mut rewritten, mut clamped, mut repeated) = (0, 0, 0, 0);
    let mut check = |text: String| {
        count += 1;
        let Ok(first) = FontMetadata::parse(
            &PackPath::from_static("font/font_metadata.json"),
            text.as_bytes(),
            &LoadContext::detached(),
        ) else {
            return;
        };
        let beyond_a_float = text.contains("1e39") || text.contains("1e999");
        let written = match first.to_json() {
            Ok(written) => written,
            // A list that loads is written: a number beyond a double in a member
            // kept as written is held as the largest finite double, with a warning (below).
            Err(error) => panic!("{text}: {error}"),
        };
        let second = metadata(&written);
        if json(&written) == json(&text) {
            // As authored.
            assert_eq!(first.value(), second.value(), "{text}");
        } else if json(&written) == json(&last_wins(&text)) {
            // Of a member written twice the last is the member; what is
            // written is the file as every JSON reader of the game takes it.
            assert_ne!(last_wins(&text), text, "{text}");
            assert_eq!(first.value(), second.value(), "{text}");
            repeated += 1;
        } else {
            // The one recorded rewrite: a scale beyond a float is written as the
            // largest float, and the load said so at the member.
            assert!(beyond_a_float, "{text} wrote {written}");
            let found = located(first.diagnostics());
            let of = |name: &str| -> Vec<&String> {
                found
                    .iter()
                    // the font clamp is an information, the wide-number line a warning
                    .filter(|(code, _, _)| *code == name)
                    .map(|(_, _, at)| at)
                    .collect()
            };
            // The scale's own finding is `ClampedValue`.
            let warned = of("core/invalid_value");
            if of("core/clamped_value")
                .iter()
                .any(|at| at.ends_with("font_scale") || at.ends_with("font_scale_factor_override"))
            {
                rewritten += 1;
            } else {
                // A number no double holds, in a member kept as written, is written
                // as the largest finite double, and the load said so at a member.
                assert!(text.contains("1e999") && !warned.is_empty(), "{text}");
                assert!(written.contains("1.7976931348623157e308"), "{written}");
                clamped += 1;
            }
            assert_eq!(first.value(), second.value(), "{text}");
        }
        assert_eq!(second.to_json().unwrap(), written, "{text}");
    };
    for format in ["bitmap", "ttf", "ttfmsdf", "msdf", "x"] {
        for member in GENERATED_FONT_MEMBERS {
            for value in GENERATED_VALUES {
                for second in GENERATED_VALUES
                    .iter()
                    .take(5)
                    .chain(&GENERATED_VALUES[14..])
                {
                    check(format!(
                        r#"{{ "version": 1, "fonts": [{{ "font_format": "{format}", "font_name": "n", "{member}": {value}, "font_file": {second} }}] }}"#
                    ));
                }
            }
        }
    }
    for member in GENERATED_ALIAS_MEMBERS {
        for value in GENERATED_VALUES {
            for second in GENERATED_VALUES {
                check(format!(
                    r#"{{ "version": 1, "fonts": [{TTF}], "font_aliases": [{{ "alias": "a", "{member}": {value}, "fonts": [{{ "font_reference": "T", "{member}": {second} }}] }}] }}"#
                ));
            }
        }
    }
    assert!(count >= 3000, "{count} documents");
    // Each class of exception occurs.
    assert!(
        rewritten > 0 && clamped > 0 && repeated > 0,
        "{rewritten} rewritten, {clamped} clamped, {repeated} with a repeated member"
    );
    println!("{count} font lists: {rewritten} rewritten, {clamped} clamped, {repeated} repeated");
}

// ---- emoticons.json -------------------------------------------------------------------------

#[test]
fn fo07_a_code_written_as_the_character_itself() {
    let doc = emoticons("[{ \"name\": \":a:\", \"code\": \"\u{E100}\" }, 3]");
    assert!(doc.code_points().is_empty(), "no emoticon is registered");
    assert_eq!(
        located(doc.diagnostics()),
        [
            ("core/invalid_value", Severity::Info, "0/code".to_owned()),
            ("core/invalid_value", Severity::Info, "0/name".to_owned()),
            ("core/invalid_value", Severity::Info, "1".to_owned()),
        ]
    );
    assert_eq!(
        doc.emoticons.len(),
        1,
        "the element that is no object is not held"
    );
}

#[test]
fn fo07b_emoticon_codes() {
    let doc = emoticons(
        r#"[{ "name": "a", "code": "0xE000" }, { "name": "b", "code": "E001zz" },
            { "name": "a", "code": "0xE002" }, { "name": "c", "code": "zz" }, { "name": "d", "code": "0x" }]"#,
    );
    let registered: Vec<(&str, i32)> = doc.code_points().into_iter().collect();
    assert_eq!(registered, [("a", 0xE000), ("b", 0xE001), ("d", 0)]);
    assert_eq!(
        located(doc.diagnostics()),
        [
            (
                "core/font_name_overridden",
                Severity::Info,
                "2/name".to_owned()
            ),
            ("core/invalid_value", Severity::Info, "3/code".to_owned()),
        ]
    );
}

#[test]
fn f17c_codes_are_read_as_strtol_reads_base_sixteen() {
    let code_point = |code: &str| {
        let text = format!(
            r#"[{{ "name": "n", "code": {} }}]"#,
            facet_json::to_string(&code.to_owned()).unwrap()
        );
        emoticons(&text).emoticons[0].code_point()
    };
    for (code, want) in [
        ("0xE100", Some(0xE100)),
        ("0XE100", Some(0xE100)),
        ("e100", Some(0xE100)),
        (" \t\n0x1f", Some(0x1F)),
        ("+7", Some(7)),
        ("-7", Some(-7)),
        ("-0x10", Some(-16)),
        ("0x", Some(0)),
        ("0xg", Some(0)),
        ("12 34", Some(0x12)),
        ("7FFFFFFF", Some(i32::MAX)),
        ("80000000", None),
        ("-80000000", Some(i32::MIN)),
        ("-80000001", None),
        ("FFFFFFFFFFFFFFFFFFFFFFFFFFFF", None),
        ("", None),
        ("x10", None),
        ("--1", None),
        ("0x-1", Some(0)),
        ("\u{E100}", None),
    ] {
        assert_eq!(code_point(code), want, "{code:?}");
    }
}

#[test]
fn f17b_f17d_entries_the_game_skips_or_never_matches() {
    const TEXT_F17B: &str = r#"[{ "name": 1, "code": "0x1" }, { "name": "a", "code": 1 }, { "name": "b" }, "s", null,
            { "name": "has space", "code": "0x1" }, { "name": "", "code": "0x2" },
            { "name": "ok_9Z", "code": "0x3", "tag": true }]"#;
    let doc = emoticons(TEXT_F17B);
    assert_eq!(
        located(doc.diagnostics()),
        [
            ("core/invalid_value", Severity::Info, "0".to_owned()),
            ("core/invalid_value", Severity::Info, "1".to_owned()),
            ("core/invalid_value", Severity::Info, "2".to_owned()),
            ("core/invalid_value", Severity::Info, "3".to_owned()),
            ("core/invalid_value", Severity::Info, "4".to_owned()),
            ("core/invalid_value", Severity::Info, "5/name".to_owned()),
            ("core/invalid_value", Severity::Info, "6/name".to_owned()),
            (
                "core/unrecognized_field",
                Severity::Info,
                "7/tag".to_owned()
            ),
        ]
    );
    assert_eq!(doc.emoticons.len(), 3);
    // The entries the game skips are not held, and are written back all the same.
    let written = doc.to_json().unwrap();
    assert_eq!(json(&written), json(TEXT_F17B));
    let held = EmoticonList {
        written: None.into(),
        ..doc.value().clone()
    };
    assert_eq!(
        json(&held.to_json().unwrap()),
        json(
            r#"[{ "name": "has space", "code": "0x1" }, { "name": "", "code": "0x2" }, { "name": "ok_9Z", "code": "0x3", "tag": true }]"#
        )
    );
    assert_eq!(emoticons(&written).value(), doc.value());
}

#[test]
fn f17_an_emoticon_file_the_game_cannot_use() {
    let doc = emoticons(r#"{ "name": "a", "code": "0x1" }"#);
    assert!(doc.emoticons.is_empty());
    assert_eq!(
        doc.diagnostics().iter().map(shown).collect::<Vec<_>>(),
        [(
            "core/syntax",
            Severity::Info,
            "the root is not an array".to_owned()
        )]
    );
    let rejected = EmoticonList::parse(
        &PackPath::from_static("font/emoticons.json"),
        b"[ nope",
        &LoadContext::detached(),
    )
    .unwrap_err();
    // The file is refused; the text is the game's developer-log assert: an information.
    assert_eq!(
        rejected.diagnostics.iter().map(shown).collect::<Vec<_>>(),
        [(
            "font/malformed",
            Severity::Info,
            "Malformed JSON".to_owned()
        )]
    );
}

#[test]
fn generated_emoticon_lists_round_trip() {
    let values = [
        "\"a\"",
        "\"0xE000\"",
        "\"zz\"",
        "\":x:\"",
        "1",
        "null",
        "[]",
        "{}",
        "true",
        "\"\"",
    ];
    let mut count = 0;
    for name in values {
        for code in values {
            for second in values {
                for extra in ["", ", \"k\": 1", ", \"extra\": {}"] {
                    let text = format!(
                        r#"[{{ "name": {name}, "code": {code}{extra} }}, {second}, {{ "name": {second}, "code": {code} }}]"#
                    );
                    let first = emoticons(&text);
                    let written = first.to_json().unwrap();
                    // As authored: no generated document is rewritten.
                    assert_eq!(json(&written), json(&text), "{text}");
                    let again = emoticons(&written);
                    assert_eq!(first.value(), again.value(), "{text}");
                    assert_eq!(first.code_points(), again.code_points(), "{text}");
                    assert_eq!(again.to_json().unwrap(), written, "{text}");
                    count += 1;
                }
            }
        }
    }
    assert!(count >= 3000, "{count} documents");
    println!("{count} generated documents written as authored");
}

// ---- .fontdata and remapping.dat ------------------------------------------------------------

fn fontdata(header: u32, advances: &[f32]) -> Vec<u8> {
    let mut bytes = header.to_le_bytes().to_vec();
    for advance in advances {
        bytes.extend(advance.to_le_bytes());
    }
    bytes
}

#[test]
fn fo12_the_vanilla_metrics_of_page_41() {
    // gdk-retail 1.26.45 resource_packs/vanilla/font/smooth/smooth_41.fontdata, unchanged.
    let bytes = fixture("fo12_fontdata_vanilla/pack/font/smooth/smooth_41.fontdata");
    assert_eq!(bytes.len(), 1028);
    let (page, diagnostics) = GlyphPage::from_bytes(&bytes).expect("header 0");
    assert_eq!(codes(&diagnostics), []);
    assert_eq!(page.advances.len(), 256);
    assert!(
        (page.width(0x41) - 7.26).abs() < 0.005,
        "U+4141 is {} wide",
        page.width(0x41)
    );
    assert_eq!(page.to_bytes(), bytes, "written back byte for byte");
}

#[test]
fn fo13_a_header_that_is_not_zero() {
    for header in [1, u32::MAX, 0x0100_0000] {
        let refused = GlyphPage::from_bytes(&fontdata(header, &[0.5; 256])).unwrap_err();
        assert_eq!(
            refused.iter().map(shown).collect::<Vec<_>>(),
            [(
                "font/fontdata_version",
                Severity::Info,
                "Version of MSDF data file should be 0".to_owned()
            )]
        );
    }
}

#[test]
fn fo14_a_short_file_leaves_the_missing_advances_at_zero_deviation() {
    let (page, diagnostics) = GlyphPage::from_bytes(&fontdata(0, &[0.5; 128])).expect("header 0");
    assert_eq!(page.advances[127], 0.5);
    assert!(page.advances[128..].iter().all(|advance| *advance == 0.0));
    assert_eq!(
        codes(&diagnostics),
        [("core/syntax", Severity::Info)],
        "the absent slots are not reported one by one"
    );
    // A cut float is no advance.
    let mut bytes = fontdata(0, &[0.5; 2]);
    bytes.truncate(4 + 4 + 3);
    let (page, _) = GlyphPage::from_bytes(&bytes).unwrap();
    assert_eq!(page.advances[..2], [0.5, 0.0]);
}

#[test]
fn f24_bytes_after_the_last_advance_are_ignored() {
    let mut bytes = fontdata(0, &[0.5; 256]);
    bytes.extend([9; 5]);
    let (page, diagnostics) = GlyphPage::from_bytes(&bytes).expect("header 0");
    assert_eq!(codes(&diagnostics), [("core/syntax", Severity::Info)]);
    assert_eq!(page.to_bytes(), fontdata(0, &[0.5; 256]));
    assert_eq!(
        codes(&GlyphPage::from_bytes(&fontdata(0, &[0.5; 256])).unwrap().1),
        [],
        "1028 bytes"
    );
}

#[test]
fn fo26_f25_an_advance_that_is_not_positive() {
    let mut advances = [0.5_f32; 256];
    advances[0x41] = 0.0;
    advances[0x42] = -1.0;
    advances[0x43] = f32::MIN_POSITIVE;
    let (page, diagnostics) = GlyphPage::from_bytes(&fontdata(0, &advances)).unwrap();
    assert_eq!(
        located(&diagnostics),
        [
            ("core/invalid_value", Severity::Info, "65".to_owned()),
            ("core/invalid_value", Severity::Info, "66".to_owned()),
        ]
    );
    assert_eq!(page.width(0x42), -8.0, "the stored value is the width");
}

#[test]
fn fo27_a_file_too_short_for_its_header() {
    for length in [0, 2, 3] {
        let refused = GlyphPage::from_bytes(&vec![0; length]).unwrap_err();
        assert_eq!(
            codes(&refused),
            [
                ("core/syntax", Severity::Info),
                ("font/fontdata_version", Severity::Info)
            ],
            "{length} bytes"
        );
    }
    let (page, diagnostics) = GlyphPage::from_bytes(&[0; 4]).expect("a whole header");
    assert_eq!(codes(&diagnostics), [("core/syntax", Severity::Info)]);
    assert!(page.advances.iter().all(|advance| *advance == 0.0));
}

#[test]
fn fo15_remapping_records() {
    let mut bytes = Vec::new();
    for (from, to) in [(0x4EBA_u16, 0x2100_u16), (0x4EBA, 0x2101), (0x0041, 0x2100)] {
        bytes.extend(from.to_le_bytes());
        bytes.extend(to.to_le_bytes());
    }
    let whole = bytes.clone();
    bytes.extend([1, 2]);
    let (remapping, diagnostics) = GlyphRemapping::from_bytes(&bytes);
    assert_eq!(remapping.0.len(), 3);
    assert_eq!(
        remapping.0[0],
        RemapEntry {
            from: 0x4EBA,
            to: 0x2100
        }
    );
    let map = remapping.to_map();
    assert_eq!(
        (map[&0x4EBA], map[&0x0041], map.len()),
        (0x2101, 0x2100, 2),
        "the last record of a character stands"
    );
    assert_eq!(
        located(&diagnostics),
        [
            ("core/syntax", Severity::Info, "1".to_owned()),
            ("core/syntax", Severity::Info, String::new()),
        ]
    );
    assert_eq!(remapping.to_bytes(), whole);
    assert_eq!(
        codes(&GlyphRemapping::from_bytes(&whole[4..]).1),
        [],
        "whole records, each character once"
    );
    assert_eq!(
        GlyphRemapping::from_bytes(&[1, 2, 3]).0,
        GlyphRemapping::default()
    );
}

// ---- the stack ------------------------------------------------------------------------------

fn font_list(fonts: &str) -> String {
    format!(r#"{{ "version": 1, "fonts": [{fonts}] }}"#)
}

#[test]
fn fo16_an_msdf_font_with_its_own_page_prefix_deviation() {
    let list = font_list(
        r#"{ "font_format": "msdf", "font_name": "Mine", "font_file": "font/myfont/page_" }"#,
    );
    let stack = stack_of(&[&[
        ("font/font_metadata.json", &list),
        ("font/myfont/page_00.png", ""),
        ("font/myfont/page_00.fontdata", ""),
        ("font/myfont/page_41.png", ""),
        ("font/myfont/page_41.fontdata", ""),
        ("font/myfont/remapping.dat", ""),
    ]]);
    assert_eq!(
        file_findings(&stack, 1),
        [(
            "core/ignored_file",
            Severity::Info,
            0,
            "font/myfont/page_41.png".to_owned()
        )],
        "only U+0000 to U+00FF of such a font are drawn"
    );
    let fonts = stack.fonts();
    let pages: Vec<u8> = fonts
        .msdf_pages("font/myfont/page_")
        .iter()
        .map(|page| page.page)
        .collect();
    assert_eq!(pages, [0x00, 0x41]);
    assert_eq!(
        remapping_path("font/myfont/page_"),
        "font/myfont/remapping.dat"
    );
    assert_eq!(remapping_path(SMOOTH_PREFIX), "font/smooth/remapping.dat");
    assert_eq!(remapping_path("page_"), "remapping.dat");
    assert!(fonts.file("font/myfont/remapping.dat").is_some());
    assert_eq!(
        fonts.metadata.font("Mine").unwrap().format(),
        Some(FontFormat::Msdf)
    );
}

#[test]
fn an_msdf_prefix_outside_font_is_found() {
    let list = font_list(
        r#"{ "font_format": "msdf", "font_name": "Mine", "font_file": "glyphs/sub/p" }, { "font_format": "msdf", "font_name": "Two", "font_file": "glyphs/q" }, { "font_format": "msdf", "font_name": "Root", "font_file": "r_" }"#,
    );
    let stack = stack_of(&[&[
        ("font/font_metadata.json", &list),
        ("glyphs/sub/p01.png", ""),
        ("glyphs/q02.fontdata", ""),
        ("r_03.png", ""),
    ]]);
    let fonts = stack.fonts();
    assert_eq!(fonts.msdf_pages("glyphs/sub/p").len(), 1);
    assert_eq!(fonts.msdf_pages("glyphs/q").len(), 1);
    assert_eq!(
        fonts.msdf_pages("r_").len(),
        0,
        "a prefix without a directory is not searched"
    );
    assert_eq!(
        file_findings(&stack, 1),
        [
            (
                "font/msdf_info_missing",
                Severity::Info,
                0,
                "glyphs/sub/p01.png".to_owned()
            ),
            (
                "core/ignored_file",
                Severity::Info,
                0,
                "glyphs/sub/p01.png".to_owned()
            ),
            (
                "core/ignored_file",
                Severity::Info,
                0,
                "glyphs/q02.fontdata".to_owned()
            ),
        ]
    );
}

#[test]
fn fo17_an_msdf_page_without_metrics() {
    let stack = stack_of(&[&[("other.txt", "")], &[("font/smooth/smooth_41.png", "")]]);
    assert_eq!(
        file_findings(&stack, 2),
        [(
            "font/msdf_info_missing",
            Severity::Info,
            1,
            "font/smooth/smooth_41.png".to_owned()
        )]
    );
    // F23's text is the game's.
    let texts: Vec<String> = stack
        .fonts()
        .diagnostics()
        .map(|d| d.code.to_string())
        .collect();
    assert_eq!(texts, ["Couldn't load MSDF Font information"]);
    // The finding names the path that was tried.
    let tried: Vec<Option<String>> = stack
        .fonts()
        .diagnostics()
        .map(|d| d.detail.as_ref().map(ToString::to_string))
        .collect();
    assert_eq!(
        tried,
        [Some(
            ": font file 'font/smooth/smooth_41.fontdata'".to_owned()
        )]
    );
    // One tried path is a list of one (K85).
    let listed: Vec<Vec<String>> = stack.fonts().diagnostics().map(tried_paths).collect();
    assert_eq!(listed, [vec!["font/smooth/smooth_41.fontdata"]]);
    // The metrics in any pack are enough (the pair is then mixed, fo24).
    let stack = stack_of(&[
        &[("font/smooth/smooth_41.fontdata", "")],
        &[("other.txt", "")],
        &[("font/smooth/smooth_41.png", "")],
    ]);
    assert_eq!(
        codes(stack.fonts().diagnostics()),
        [("core/mixed_font_page", Severity::Info)]
    );
    // Metrics without an atlas are no finding.
    let stack = stack_of(&[&[("font/smooth/smooth_41.fontdata", "")]]);
    assert_eq!(codes(stack.fonts().diagnostics()), []);
}

#[test]
fn fo24_f30_a_page_whose_files_come_from_two_packs() {
    let vanilla: &[(&str, &str)] = &[
        ("font/smooth/smooth_41.png", ""),
        ("font/smooth/smooth_41.fontdata", ""),
    ];
    let stack = stack_of(&[vanilla, &[("font/smooth/smooth_41.png", "")]]);
    let found: Vec<&Diagnostic> = stack.fonts().diagnostics().collect();
    assert_eq!(
        codes(found.iter().copied()),
        [("core/mixed_font_page", Severity::Info)]
    );
    assert_eq!(pack_number(found[0], 2), Some(1), "at the higher file");
    assert_eq!(
        found[0].location.path.as_ref().unwrap().as_str(),
        "font/smooth/smooth_41.png"
    );
    assert!(
        found[0]
            .to_string()
            .contains("pack 0: font/smooth/smooth_41.fontdata"),
        "{}",
        found[0]
    );
    let page = &stack.fonts().msdf_pages(SMOOTH_PREFIX)[0];
    assert_eq!(
        (
            page.image.as_ref().unwrap().pack.index(),
            page.metrics.as_ref().unwrap().pack.index()
        ),
        (Some(0), Some(1))
    );
    // The other way round, and both files overridden together.
    let stack = stack_of(&[vanilla, &[("font/smooth/smooth_41.fontdata", "")]]);
    let found: Vec<&Diagnostic> = stack.fonts().diagnostics().collect();
    assert_eq!(
        found[0].location.path.as_ref().unwrap().as_str(),
        "font/smooth/smooth_41.fontdata"
    );
    let stack = stack_of(&[vanilla, vanilla, vanilla]);
    assert_eq!(codes(stack.fonts().diagnostics()), []);
}

#[test]
fn f27_smooth_pages_the_client_does_not_draw_deviation() {
    for (page, drawn) in [
        ("00", true),
        ("04", true),
        ("05", false),
        ("1F", false),
        ("20", true),
        ("21", true),
        ("22", false),
        ("2D", false),
        ("2E", true),
        ("D7", true),
        ("D8", false),
        ("F8", false),
        ("F9", true),
        ("FF", true),
    ] {
        let (image, metrics) = (
            format!("font/smooth/smooth_{page}.png"),
            format!("font/smooth/smooth_{page}.fontdata"),
        );
        let stack = stack_of(&[&[(image.as_str(), ""), (metrics.as_str(), "")]]);
        let want: &[_] = if drawn {
            &[]
        } else {
            &[("core/ignored_file", Severity::Info)]
        };
        assert_eq!(codes(stack.fonts().diagnostics()), want, "{page}");
    }
    // Lower-case digits name no page.
    let stack = stack_of(&[&[("font/smooth/smooth_d8.png", "")]]);
    assert_eq!(codes(stack.fonts().diagnostics()), []);
    assert!(stack.fonts().msdf_pages(SMOOTH_PREFIX).is_empty());
}

/// A stack whose list names six true-type files, four of which no pack has.
fn true_type_files_no_pack_has() -> PackStack<Resource> {
    let list = font_list(
        r#"{ "font_format": "ttf", "font_name": "A", "font_file": "font/x.ttf" },
           { "font_format": "ttf", "font_name": "B", "font_file": "font/absent" },
           { "font_format": "ttf", "font_name": "C", "font_file": "font/c" },
           { "font_format": "ttf", "font_name": "D", "font_file": "font/collection" },
           { "font_format": "ttfmsdf", "font_name": "E", "font_file": "font/collection" },
           { "font_format": "ttfmsdf", "font_name": "F", "font_file": "font/c.otc" }"#,
    );
    stack_of(&[&[
        ("font/font_metadata.json", &list),
        ("font/x.ttf", ""),
        ("font/x.ttf.ttf", ""),
        ("font/c.otf", ""),
        ("font/collection.ttc", ""),
    ]])
}

#[test]
fn fo18_f21_a_true_type_file_no_pack_has() {
    let stack = true_type_files_no_pack_has();
    let found: Vec<(&str, Severity, String)> = located(stack.fonts().diagnostics());
    assert_eq!(
        found,
        [
            (
                "font/font_load_failed",
                Severity::Info,
                "fonts/0/font_file".to_owned()
            ),
            (
                "font/font_load_failed",
                Severity::Info,
                "fonts/1/font_file".to_owned()
            ),
            (
                "font/font_load_failed",
                Severity::Info,
                "fonts/3/font_file".to_owned()
            ),
            (
                "font/font_load_failed",
                Severity::Info,
                "fonts/5/font_file".to_owned()
            ),
        ],
        "an extension in font_file, a missing file, a collection for ttf, an extension again"
    );
    // F21's text is the game's.
    assert!(
        stack
            .fonts()
            .diagnostics()
            .filter(|d| d.code.id() == "font/font_load_failed")
            .all(|d| d.code.to_string() == "Failed to load font from resource pack")
    );
}

#[test]
fn fo18_f21_the_paths_the_game_tries_for_a_missing_true_type_file() {
    let stack = true_type_files_no_pack_has();
    // Each names the paths the game tries, in its order: `font_file` and one extension more,
    // also where `font_file` carries one already.
    let tried: Vec<String> = stack
        .fonts()
        .diagnostics()
        .filter(|d| d.code.id() == "font/font_load_failed")
        .map(|d| {
            d.detail
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default()
        })
        .collect();
    assert_eq!(
        tried,
        [
            ": font file 'font/x.ttf.ttf, font/x.ttf.otf'",
            ": font file 'font/absent.ttf, font/absent.otf'",
            ": font file 'font/collection.ttf, font/collection.otf'",
            ": font file 'font/c.otc.ttf, font/c.otc.otf, font/c.otc.ttc, font/c.otc.otc'",
        ]
    );
    // Each tried path is its own element of the detail, not one joined name (K85).
    let listed: Vec<Vec<String>> = stack
        .fonts()
        .diagnostics()
        .filter(|d| d.code.id() == "font/font_load_failed")
        .map(tried_paths)
        .collect();
    assert_eq!(
        listed,
        [
            vec!["font/x.ttf.ttf", "font/x.ttf.otf"],
            vec!["font/absent.ttf", "font/absent.otf"],
            vec!["font/collection.ttf", "font/collection.otf"],
            vec![
                "font/c.otc.ttf",
                "font/c.otc.otf",
                "font/c.otc.ttc",
                "font/c.otc.otc"
            ],
        ]
    );
    let at = stack
        .fonts()
        .diagnostics()
        .next()
        .unwrap()
        .location
        .path
        .clone()
        .unwrap();
    assert_eq!(at.as_str(), "font/font_metadata.json");
    let fonts = stack.fonts();
    assert_eq!(
        fonts
            .true_type_file(fonts.metadata.font("E").unwrap())
            .unwrap()
            .path
            .as_str(),
        "font/collection.ttc"
    );
    assert!(
        fonts
            .true_type_file(fonts.metadata.font("D").unwrap())
            .is_none()
    );
}

#[test]
fn fo19_a_higher_pack_s_otf_stands_before_a_lower_pack_s_ttf() {
    let list = font_list(
        r#"{ "font_format": "ttf", "font_name": "MinecraftTen", "font_file": "font/minecraft-ten" }"#,
    );
    let vanilla: &[(&str, &str)] = &[
        ("font/font_metadata.json", &list),
        ("font/minecraft-ten.ttf", ""),
        ("font/minecraft-ten.otf", ""),
    ];
    let stack = stack_of(&[
        vanilla,
        &[("other.txt", "")],
        &[("font/minecraft-ten.otf", "")],
    ]);
    let fonts = stack.fonts();
    let file = fonts
        .true_type_file(fonts.metadata.font("MinecraftTen").unwrap())
        .unwrap();
    assert_eq!(
        (file.pack.index(), file.path.as_str()),
        (Some(0), "font/minecraft-ten.otf")
    );
    assert_eq!(codes(fonts.diagnostics()), []);
    // Inside one pack the first extension stands.
    let stack = stack_of(&[vanilla]);
    let fonts = stack.fonts();
    assert_eq!(
        fonts
            .true_type_file(fonts.metadata.font("MinecraftTen").unwrap())
            .unwrap()
            .path
            .as_str(),
        "font/minecraft-ten.ttf"
    );
}

#[test]
fn the_highest_font_list_is_read_whole_and_a_refused_one_leaves_the_built_in_fonts() {
    let low = font_list(TTF);
    let high = font_list(r#"{ "font_format": "ttf", "font_name": "U", "font_file": "font/u" }"#);
    let stack = stack_of(&[
        &[
            ("font/font_metadata.json", &low),
            ("font/t.ttf", ""),
            (
                "font/emoticons.json",
                r#"[{ "name": "low", "code": "0x1" }]"#,
            ),
        ],
        &[
            ("font/font_metadata.json", "{ nope"),
            ("font/emoticons.json", "[ nope"),
        ],
        &[
            ("font/font_metadata.json", &high),
            ("font/u.ttf", ""),
            (
                "font/emoticons.json",
                r#"[{ "name": "high", "code": "0x2" }]"#,
            ),
        ],
    ]);
    let fonts = stack.fonts();
    assert!(fonts.metadata.font("T").is_none() && fonts.metadata.font("U").is_some());
    assert_eq!(
        fonts.metadata.origin(&"U".to_owned()).unwrap().pack.index(),
        Some(0)
    );
    assert_eq!(fonts.metadata.layers().len(), 2);
    assert_eq!(
        fonts
            .emoticons
            .code_points()
            .keys()
            .copied()
            .collect::<Vec<_>>(),
        ["high"]
    );
    assert_eq!(
        fonts
            .emoticons
            .origin(&"high".to_owned())
            .unwrap()
            .pack
            .index(),
        Some(0)
    );
    assert_eq!(
        codes(fonts.diagnostics()),
        [],
        "the lower copies report nothing"
    );
    // A refused highest copy: two and three packs, false and true versions.
    for refused in [r#"{ "version": 2, "fonts": [] }"#, "{ nope"] {
        for lower in [1, 2] {
            let mut packs: Vec<&[(&str, &str)]> = Vec::new();
            let low_pack: &[(&str, &str)] = &[
                ("font/font_metadata.json", &low),
                (
                    "font/emoticons.json",
                    r#"[{ "name": "low", "code": "0x1" }]"#,
                ),
            ];
            packs.resize(lower, low_pack);
            let top: &[(&str, &str)] = &[
                ("font/font_metadata.json", refused),
                ("font/emoticons.json", "[ nope"),
            ];
            packs.push(top);
            let stack = stack_of(&packs);
            let fonts = stack.fonts();
            assert_eq!(
                fonts.metadata.value(),
                &FontMetadata::default(),
                "{refused}"
            );
            assert!(fonts.emoticons.emoticons.is_empty());
            let found = codes(fonts.diagnostics());
            assert_eq!(found.len(), 2, "{found:?}");
            assert_eq!(found[0].1, Severity::Info);
            assert_eq!(found[1], ("font/malformed", Severity::Info));
            assert!(
                fonts
                    .diagnostics()
                    .all(|d| pack_number(d, lower + 1) == Some(lower))
            );
        }
    }
}

#[test]
fn f1_a_stack_without_font_files_has_nothing_to_say() {
    let stack = stack_with(&[&[]], LoadOptions::default());
    let fonts = stack.fonts();
    assert_eq!(fonts.metadata.value(), &FontMetadata::default());
    assert_eq!(codes(fonts.diagnostics()), []);
    assert!(fonts.file("font/default8.png").is_none());
}

/// A whole stack with a few hundred of everything, for its content alone: the cost of each
/// reader is held in the crate, on pre-built inputs.
#[test]
fn a_stack_of_three_hundred_fonts_aliases_emoticons_and_font_files_loads_whole() {
    let entries: usize = 300;
    {
        let fonts: Vec<String> = (0..entries)
            .map(|i| match i % 3 {
                0 => format!(r#"{{ "font_format": "ttf", "font_name": "t{i}", "font_file": "font/t{i}" }}"#),
                1 => format!(r#"{{ "font_format": "msdf", "font_name": "m{i}", "font_file": "pages/d{i}/p_" }}"#),
                _ => format!(r#"{{ "font_format": "bitmap", "font_name": "b{i}", "unicode_file_prefix": "font/g{i}_" }}"#),
            })
            .collect();
        let aliases: Vec<String> = (0..entries)
            .map(|i| format!(r#"{{ "alias": "a{i}", "fonts": [{{ "font_reference": "t{}" }}, {{ "font_reference": "a{}" }}] }}"#, i - i % 3, i.saturating_sub(1)))
            .collect();
        let list = format!(
            r#"{{ "version": 1, "fonts": [{}], "font_aliases": [{}] }}"#,
            fonts.join(","),
            aliases.join(",")
        );
        let emoticons: Vec<String> = (0..entries)
            .map(|i| format!(r#"{{ "name": "e{i}", "code": "0x{i:X}" }}"#))
            .collect();
        let emoticons = format!("[{}]", emoticons.join(","));
        let names: Vec<String> = (0..entries)
            .map(|i| match i % 3 {
                0 => format!("font/t{i}.otf"),
                1 => format!("pages/d{i}/p_{:02X}.png", i % 256),
                _ => format!("font/glyph_x{i}.png"),
            })
            .collect();
        let mut files: Vec<(&str, &[u8])> =
            names.iter().map(|name| (name.as_str(), &b""[..])).collect();
        files.push(("font/font_metadata.json", list.as_bytes()));
        files.push(("font/emoticons.json", emoticons.as_bytes()));
        let stack = stack_with(&[&files], LoadOptions::default());
        let fonts = stack.fonts();
        assert_eq!(fonts.metadata.fonts.as_ref().unwrap().len(), entries);
        assert_eq!(fonts.metadata.font_aliases.as_ref().unwrap().len(), entries);
        assert_eq!(fonts.emoticons.code_points().len(), entries);
        assert_eq!(
            fonts
                .true_type_file(fonts.metadata.font("t0").unwrap())
                .unwrap()
                .path
                .as_str(),
            "font/t0.otf"
        );
        assert_eq!(fonts.msdf_pages("pages/d1/p_").len(), 1);
        let found = codes(fonts.diagnostics());
        println!("{} findings", found.len());
        assert!(found.len() >= entries / 3 * 2, "{}", found.len());
        assert!(
            found
                .iter()
                .all(|(_, severity)| *severity != Severity::Error)
        );
    }
}
