//! The link rules of the fonts: the font files of a stack checked from their bytes.

use bedrock_addon::diagnostic::{Diagnostic, DiagnosticDetail, Diagnostics, Phase, Severity};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{LinkOptions, Pack};

use crate::common::{resource_pack_of, stacks_of, unloaded_pack};

/// `bytes` for the life of the test process.
fn keep(bytes: &[u8]) -> &'static [u8] {
    Box::leak(bytes.to_vec().into_boxed_slice())
}

/// The resource pack `n` with `files`.
fn pack(n: u128, files: &[(&'static str, &[u8])]) -> Pack {
    let files: Vec<(&'static str, &'static [u8])> = files
        .iter()
        .map(|(path, bytes)| (*path, keep(bytes)))
        .collect();
    resource_pack_of(n, &files)
}

/// The link findings of a world of the resource packs `packs`, top first.
fn linked(packs: Vec<Pack>, load: LoadOptions, options: &LinkOptions) -> Diagnostics {
    stacks_of(Vec::new(), packs, load).link_with(options)
}

/// The link findings of one resource pack (`rp 2`) with `files`.
fn link(files: &[(&'static str, &[u8])]) -> Diagnostics {
    linked(
        vec![pack(2, files)],
        LoadOptions::default(),
        &LinkOptions::default(),
    )
}

/// What a finding says: code, severity, pack, file, JSON path.
type Said = (&'static str, Severity, String, String, String);

fn said(finding: &Diagnostic) -> Said {
    assert_eq!(finding.phase, Phase::Link, "{finding}");
    let pack = finding.location.pack.as_ref().map(|pack| pack.name());
    let path = finding.location.path.as_ref().map(ToString::to_string);
    (
        finding.code.name(),
        finding.severity,
        pack.unwrap_or_default().to_owned(),
        path.unwrap_or_default(),
        finding.location.json_path.to_string(),
    )
}

fn all_said(found: &Diagnostics) -> Vec<Said> {
    found.iter().map(said).collect()
}

/// The kind and the name of a reference finding.
fn named(finding: &Diagnostic) -> (&'static str, String) {
    match finding.detail.as_deref() {
        Some(DiagnosticDetail::Reference { kind, id, .. }) => (*kind, id.clone()),
        other => panic!("no reference: {other:?}"),
    }
}

/// A PNG header of the given size.
fn png(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = b"\x89PNG\r\n\x1a\n\x00\x00\x00\x0dIHDR".to_vec();
    bytes.extend(width.to_be_bytes());
    bytes.extend(height.to_be_bytes());
    bytes
}

/// A metrics file with the header `header` and 256 advances of 0.5.
fn fontdata(header: u32) -> Vec<u8> {
    let mut bytes = header.to_le_bytes().to_vec();
    for _ in 0..256 {
        bytes.extend(0.5_f32.to_le_bytes());
    }
    bytes
}

/// A remapping table of the records `(from, to)`.
fn remapping(records: &[(u16, u16)]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for (from, to) in records {
        bytes.extend(from.to_le_bytes());
        bytes.extend(to.to_le_bytes());
    }
    bytes
}

fn info(code: &'static str, pack: &str, file: &str, path: &str) -> Said {
    (
        code,
        Severity::Info,
        pack.to_owned(),
        file.to_owned(),
        path.to_owned(),
    )
}

const SMOOTH: &str = "font/smooth/smooth_";

#[test]
fn f16_a_glyph_sheet_that_is_a_16_by_16_grid_is_no_finding() {
    let found = link(&[
        ("font/glyph_E3.png", &png(512, 512)),
        ("font/glyph_00.png", &png(16, 16)),
        ("font/default8.png", &png(128, 128)),
        ("font/ascii_sga.png", &png(256, 256)),
    ]);
    assert_eq!(all_said(&found), []);
}

#[test]
fn f16_a_glyph_sheet_of_another_shape_is_a_glyph_sheet_shape_information() {
    let found = link(&[
        ("font/glyph_E2.png", &png(200, 256)),
        ("font/glyph_4E.png", &png(200, 200)),
        ("font/default8.png", &png(128, 64)),
        ("font/ascii_sga.png", &png(100, 100)),
        ("texts/ja_JP/font/glyph_30.png", &png(256, 255)),
        // No sheet the game asks for by a name it builds: not read.
        ("font/notes.png", &png(3, 5)),
    ]);
    assert_eq!(
        all_said(&found),
        [
            info("GlyphSheetShape", "rp 2", "font/default8.png", ""),
            info("GlyphSheetShape", "rp 2", "font/ascii_sga.png", ""),
            info("GlyphSheetShape", "rp 2", "font/glyph_4E.png", ""),
            info("GlyphSheetShape", "rp 2", "font/glyph_E2.png", ""),
            info(
                "GlyphSheetShape",
                "rp 2",
                "texts/ja_JP/font/glyph_30.png",
                ""
            ),
        ]
    );
}

/// The sheet checked is the one the game loads: the highest pack's, whatever a lower pack
/// has at the same name.
#[test]
fn f16_the_sheet_checked_is_the_highest_pack_s() {
    let good = pack(2, &[("font/glyph_E2.png", &png(256, 256))]);
    let bad = pack(3, &[("font/glyph_E2.png", &png(200, 256))]);
    let options = LinkOptions::default();
    let found = linked(
        vec![good.clone(), bad.clone()],
        LoadOptions::default(),
        &options,
    );
    assert_eq!(all_said(&found), []);
    let found = linked(vec![bad, good], LoadOptions::default(), &options);
    assert_eq!(
        all_said(&found),
        [info("GlyphSheetShape", "rp 3", "font/glyph_E2.png", "")]
    );
}

/// The sheets of a `bitmap` font of the font list are sheets too.
#[test]
fn f16_the_sheets_of_a_bitmap_font_of_the_font_list_are_checked() {
    let found = link(&[
        (
            "font/font_metadata.json",
            br#"{"version": 1, "fonts": [{"font_format": "bitmap", "font_name": "Mine",
                "ascii_font_file": "font/mine8", "unicode_file_prefix": "font/mine_"}]}"#,
        ),
        ("font/mine8.png", &png(64, 32)),
        ("font/mine_0A.png", &png(48, 48)),
        ("font/mine_0B.png", &png(40, 40)),
    ]);
    assert_eq!(
        all_said(&found),
        [
            info("GlyphSheetShape", "rp 2", "font/mine8.png", ""),
            info("GlyphSheetShape", "rp 2", "font/mine_0B.png", ""),
        ]
    );
}

#[test]
fn f16_and_the_other_font_rules_are_lints() {
    // F24 and F26 are the game's assert and its silence: informations, told with the lints
    // on only.
    let files: [(&'static str, &[u8]); 3] = [
        ("font/glyph_E2.png", &png(200, 256)),
        ("font/smooth/smooth_41.png", &png(1024, 1024)),
        ("font/smooth/smooth_41.fontdata", &fontdata(1)),
    ];
    let quiet = LinkOptions {
        lints: false,
        ..Default::default()
    };
    let found = linked(vec![pack(2, &files)], LoadOptions::default(), &quiet);
    assert_eq!(all_said(&found), []);
}

#[test]
fn f24_a_metrics_file_of_1028_bytes_with_the_header_0_is_no_finding() {
    let found = link(&[
        ("font/smooth/smooth_41.png", &png(1024, 1024)),
        ("font/smooth/smooth_41.fontdata", &fontdata(0)),
    ]);
    assert_eq!(all_said(&found), []);
}

#[test]
fn f24_a_header_that_is_not_0_and_another_size_are_informations() {
    let mut long = fontdata(0);
    long.push(0);
    let short = &fontdata(0)[..1024];
    let found = link(&[
        ("font/smooth/smooth_41.png", &png(1024, 1024)),
        ("font/smooth/smooth_41.fontdata", &fontdata(7)),
        ("font/smooth/smooth_42.png", &png(1024, 1024)),
        ("font/smooth/smooth_42.fontdata", &long),
        ("font/smooth/smooth_43.png", &png(1024, 1024)),
        ("font/smooth/smooth_43.fontdata", short),
    ]);
    // the header's assert and the unchecked size: informations
    assert_eq!(
        all_said(&found),
        [
            info(
                "MalformedInput",
                "rp 2",
                "font/smooth/smooth_41.fontdata",
                ""
            ),
            info(
                "MalformedInput",
                "rp 2",
                "font/smooth/smooth_42.fontdata",
                ""
            ),
            info(
                "MalformedInput",
                "rp 2",
                "font/smooth/smooth_43.fontdata",
                ""
            ),
        ]
    );
    // The header's finding carries the game's line.
    let header = found.iter().next().unwrap();
    assert_eq!(
        header.code.to_string(),
        "Version of MSDF data file should be 0"
    );
}

/// F25 comes with the reader of F24: an advance that is 0 or less is an information at its
/// place among the advances, and left out with the lints.
#[test]
fn f25_an_advance_that_is_not_positive_is_the_readers_information() {
    let mut metrics = fontdata(0);
    metrics[4 + 4 * 65..4 + 4 * 66].copy_from_slice(&0.0_f32.to_le_bytes());
    let files: [(&'static str, &[u8]); 2] = [
        ("font/smooth/smooth_41.png", &png(1024, 1024)),
        ("font/smooth/smooth_41.fontdata", &metrics),
    ];
    assert_eq!(
        all_said(&link(&files)),
        [info(
            "InvalidValue",
            "rp 2",
            "font/smooth/smooth_41.fontdata",
            "65"
        )]
    );
    let quiet = LinkOptions {
        lints: false,
        ..Default::default()
    };
    let found = linked(vec![pack(2, &files)], LoadOptions::default(), &quiet);
    assert_eq!(all_said(&found), []);
}

/// The pages of an `msdf` font of the font list are checked like the built-in font's, and
/// the metrics read are the highest pack's.
#[test]
fn f24_the_pages_of_an_msdf_font_of_the_font_list_are_checked_from_the_highest_pack() {
    let top = pack(
        2,
        &[
            (
                "font/font_metadata.json",
                br#"{"version": 1, "fonts": [{"font_format": "msdf", "font_name": "Mine",
                    "font_file": "font/mine/page_"}]}"#,
            ),
            ("font/mine/page_00.png", &png(1024, 1024)),
            ("font/mine/page_00.fontdata", &fontdata(3)),
        ],
    );
    let bottom = pack(3, &[("font/mine/page_00.fontdata", &fontdata(0))]);
    let found = linked(
        vec![top, bottom],
        LoadOptions::default(),
        &LinkOptions::default(),
    );
    let errors: Vec<Said> = all_said(&found)
        .into_iter()
        .filter(|said| said.0 == "MalformedInput")
        .collect();
    assert_eq!(
        errors,
        [(
            "MalformedInput",
            Severity::Info,
            "rp 2".to_owned(),
            "font/mine/page_00.fontdata".to_owned(),
            String::new()
        )]
    );
}

#[test]
fn f26_a_table_of_whole_records_onto_pages_the_font_has_is_no_finding() {
    let found = link(&[
        ("font/smooth/smooth_41.png", &png(1024, 1024)),
        ("font/smooth/smooth_41.fontdata", &fontdata(0)),
        (
            "font/smooth/remapping.dat",
            &remapping(&[(0x2000, 0x4101), (0x2001, 0x41FF)]),
        ),
    ]);
    assert_eq!(all_said(&found), []);
}

#[test]
fn f26_trailing_bytes_and_a_character_mapped_twice_are_informations() {
    let mut table = remapping(&[(0x2000, 0x4101), (0x2001, 0x4102), (0x2000, 0x4103)]);
    table.extend([9, 9]);
    let found = link(&[
        ("font/smooth/smooth_41.png", &png(1024, 1024)),
        ("font/smooth/smooth_41.fontdata", &fontdata(0)),
        ("font/smooth/remapping.dat", &table),
    ]);
    assert_eq!(
        all_said(&found),
        [
            info("MalformedInput", "rp 2", "font/smooth/remapping.dat", "2"),
            info("MalformedInput", "rp 2", "font/smooth/remapping.dat", ""),
        ]
    );
}

/// F26's third clause: a glyph on a page the font has no metrics or no atlas of draws
/// nothing. One finding for each file that is missing, at the first record that needs it,
/// naming that one path.
#[test]
fn f26_a_record_onto_a_page_without_a_file_names_each_missing_file_once() {
    let found = link(&[
        // Page 41 is whole, page 42 has no metrics, page 43 no atlas, page 44 neither.
        ("font/smooth/smooth_41.png", &png(1024, 1024)),
        ("font/smooth/smooth_41.fontdata", &fontdata(0)),
        ("font/smooth/smooth_42.png", &png(1024, 1024)),
        ("font/smooth/smooth_43.fontdata", &fontdata(0)),
        (
            "font/smooth/remapping.dat",
            &remapping(&[
                (0x2000, 0x4101),
                (0x2001, 0x4201),
                (0x2002, 0x4301),
                (0x2003, 0x4401),
                (0x2004, 0x4402),
                (0x2005, 0x4202),
            ]),
        ),
    ]);
    let table: Vec<&Diagnostic> = found
        .iter()
        .filter(|finding| {
            let path = finding.location.path.as_ref().map(|path| path.as_str());
            path == Some("font/smooth/remapping.dat")
        })
        .collect();
    let told: Vec<(Said, (&'static str, String))> = table
        .iter()
        .map(|finding| (said(finding), named(finding)))
        .collect();
    let missing = |index: &str, file: &str| {
        (
            // The game checks nothing: ours, an information.
            info("MalformedInput", "rp 2", "font/smooth/remapping.dat", index),
            ("font file", format!("{SMOOTH}{file}")),
        )
    };
    assert_eq!(
        told,
        [
            missing("1", "42.fontdata"),
            missing("2", "43.png"),
            missing("3", "44.fontdata"),
            missing("3", "44.png"),
        ]
    );
    // K85a: a finding names one path, never several joined.
    for (_, (_, name)) in &told {
        assert!(!name.contains(','), "{name}");
    }
    assert_eq!(
        table[0].code.to_string(),
        "character 0x2001 is drawn with a glyph of page 42, which the font has no file of"
    );
}

/// The page's files may be another pack's: the game finds each file on its own.
#[test]
fn f26_a_page_a_lower_pack_supplies_is_a_page_of_the_font() {
    let top = pack(
        2,
        &[("font/smooth/remapping.dat", &remapping(&[(0x2000, 0x4101)]))],
    );
    let bottom = pack(
        3,
        &[
            ("font/smooth/smooth_41.png", &png(1024, 1024)),
            ("font/smooth/smooth_41.fontdata", &fontdata(0)),
        ],
    );
    let options = LinkOptions::default();
    let found = linked(vec![top.clone(), bottom], LoadOptions::default(), &options);
    assert_eq!(all_said(&found), []);
    // Without that pack both files are missing.
    let found = linked(vec![top], LoadOptions::default(), &options);
    let names: Vec<String> = found.iter().map(|finding| named(finding).1).collect();
    assert_eq!(
        names,
        [format!("{SMOOTH}41.fontdata"), format!("{SMOOTH}41.png")]
    );
}

/// A font file a pack lists and does not hold in memory is not read: the frame says so once
/// for each, and no byte rule runs on it.
#[test]
fn the_font_files_of_a_pack_that_is_not_loaded_are_named_and_not_judged() {
    let glyph = png(200, 256);
    let metrics = fontdata(1);
    let atlas = png(1024, 1024);
    let unloaded = unloaded_pack(
        2,
        "resources",
        &[
            ("font/glyph_E2.png", glyph.as_slice()),
            ("font/smooth/smooth_41.png", atlas.as_slice()),
            ("font/smooth/smooth_41.fontdata", metrics.as_slice()),
        ],
    );
    let found = linked(
        vec![unloaded],
        LoadOptions::default(),
        &LinkOptions::default(),
    );
    let mut told: Vec<(&'static str, String)> = found
        .iter()
        .map(|finding| {
            let path = finding.location.path.as_ref().map(ToString::to_string);
            (finding.code.name(), path.unwrap_or_default())
        })
        .filter(|(_, path)| path.starts_with("font/"))
        .collect();
    told.sort();
    assert_eq!(
        told,
        [
            ("FileNotLoaded", "font/glyph_E2.png".to_owned()),
            ("FileNotLoaded", "font/smooth/smooth_41.fontdata".to_owned()),
        ]
    );
}
