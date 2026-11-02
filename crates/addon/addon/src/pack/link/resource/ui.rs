//! The font files of the resource stack, checked from their bytes: the shape of each glyph
//! sheet, the metrics of each page of a distance-field font, and each font's remapping table.
//!
//! A stack load reads none of these files; it knows them by name. The pass reads the ones a
//! pack holds in memory (a file that is listed and not resident is told once by the pass and
//! skipped) and hands each to the font family's own reader, whose findings it tells at the
//! file. Every file is the one the game would load: the highest pack that has it, and for a
//! glyph sheet the language's copy before the pack's own.
// `GlyphSheetShape` (info) for a glyph sheet; `MalformedInput` for a page's metrics (error for
// the header, warning for the size) and for a remapping table (warning); the font family
// decides which reader each file goes to.

use std::collections::HashSet;

use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Severity};
use bedrock_addon_core::document::Origin;
use bedrock_addon_core::pack_identity::PackSide;

use super::super::{At, Findings, LinkCx};
use crate::families::font::{
    FontFile, FontFormat, Fonts, GlyphPage, GlyphRemapping, MsdfPage, check_glyph_sheet,
    remapping_path,
};

/// The page prefix of the game's built-in bitmap fonts (`font/glyph_4E`).
const GLYPH_PREFIX: &str = "font/glyph_";

/// The sheets of the game's built-in bitmap fonts that are no page: the default font's first
/// 256 characters and the enchanting-table alphabet.
const BUILT_IN_SHEETS: [&str; 2] = ["font/default8", "font/ascii_sga"];

/// What a finding calls a font file a remapping table needs and no pack has: the kind the
/// font family's own findings use.
const FONT_FILE: &str = "font file";

/// The font byte rules, in this order: the glyph sheets, then for each distance-field font
/// its remapping table and the metrics of its pages.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    let fonts = cx.resource().fonts();
    if cx.lints() {
        glyph_sheets(cx, out, fonts);
    }
    // A remapping table two fonts share is read once.
    let mut read = HashSet::new();
    for (prefix, pages) in &fonts.pages {
        if let Some(file) = fonts.file(&remapping_path(prefix)) {
            remapping(cx, out, prefix, pages, file, read.insert(key(file)));
        }
        for page in pages {
            metrics(cx, out, page);
        }
    }
}

/// What tells one font file of the stack from another: its pack's place and its path.
fn key(file: &FontFile) -> (Option<usize>, &str) {
    (file.pack.index(), file.path.as_str())
}

/// The bytes of a font file, when its pack holds them.
fn bytes(cx: &LinkCx<'_>, file: &FontFile, out: &mut Findings) -> Option<bytes::Bytes> {
    cx.bytes(PackSide::Resource, &file.pack, &file.path, out)
}

/// The findings a reader of the font family gave for the bytes of `file`, at that file.
fn at_file(findings: Diagnostics, file: &FontFile) -> impl Iterator<Item = Diagnostic> + '_ {
    findings.into_iter().map(|mut finding| {
        finding.location.pack = Some(file.pack.clone());
        finding.location.path = Some(file.path.clone());
        finding
    })
}

/// Every glyph sheet the game loads is a square PNG whose side is a multiple of 16; one
/// that is not is a `GlyphSheetShape` information at the file (the game cuts a sheet into
/// 16 by 16 cells of a sixteenth of its width). The sheets are the built-in fonts' and those
/// of each `bitmap` font of the font list: the 256 pages of each page prefix and the sheet
/// of the first 256 characters, each as the game finds it for no language and for every
/// language a pack has a font folder for. A lint: not run when the lints are off.
fn glyph_sheets<'f>(cx: &LinkCx<'_>, out: &mut Findings, fonts: &'f Fonts) {
    let mut prefixes = vec![GLYPH_PREFIX];
    let mut sheets = BUILT_IN_SHEETS.to_vec();
    for font in fonts.metadata.value().fonts.iter().flatten() {
        if font.format() != Some(FontFormat::Bitmap) {
            continue;
        }
        if let Some(prefix) = font.unicode_file_prefix.as_deref()
            && !prefixes.contains(&prefix)
        {
            prefixes.push(prefix);
        }
        if let Some(sheet) = font.ascii_font_file.as_deref()
            && !sheets.contains(&sheet)
        {
            sheets.push(sheet);
        }
    }
    // The languages some pack has a font folder for (`texts/<language>/font/`), after the
    // client without one.
    let mut languages: Vec<Option<&str>> = vec![None];
    for path in fonts.files.keys() {
        let language = path
            .strip_prefix("texts/")
            .and_then(|rest| rest.split_once('/'))
            .map(|(language, _)| language);
        if language.is_some() && !languages.contains(&language) {
            languages.push(language);
        }
    }
    // One file serves every language without a copy of its own: each is read once.
    let mut checked = HashSet::new();
    let mut check = |file: Option<&'f FontFile>, out: &mut Findings| {
        if let Some(file) = file
            && checked.insert(key(file))
            && let Some(bytes) = bytes(cx, file, out)
        {
            out.extend(at_file(check_glyph_sheet(&bytes), file));
        }
    };
    for language in languages {
        for sheet in &sheets {
            check(fonts.sheet(sheet, language), out);
        }
        for prefix in &prefixes {
            for page in 0..=u8::MAX {
                check(fonts.glyph_sheet(prefix, page, language), out);
            }
        }
    }
}

/// The metrics file of a page of a distance-field
/// font is 1028 bytes with a header of 0. A header that is not 0 is a `MalformedInput` error
/// (the page has no widths), another size a `MalformedInput` warning, and an advance that is
/// 0 or less an `InvalidValue` information, each at the file.
fn metrics(cx: &LinkCx<'_>, out: &mut Findings, page: &MsdfPage) {
    let Some(file) = &page.metrics else {
        return;
    };
    let Some(bytes) = bytes(cx, file, out) else {
        return;
    };
    let findings = match GlyphPage::from_bytes(&bytes) {
        Ok((_, findings)) | Err(findings) => findings,
    };
    out.extend(at_file(findings, file));
}

/// The remapping table of the distance-field font with the page prefix `prefix`. Bytes
/// after the last whole record and a second record for one character are each a
/// `MalformedInput` warning of the table's reader, told when `first` (the table has not been
/// read for another font). A record whose glyph is on a page the stack lacks a file of draws
/// nothing: a `MalformedInput` information at the first such record of each missing file, which
/// the finding names by the one path the game asks for (`<prefix>XX.fontdata`,
/// `<prefix>XX.png`), a finding for each.
// A path per finding, never several joined in one name.
fn remapping(
    cx: &LinkCx<'_>,
    out: &mut Findings,
    prefix: &str,
    pages: &[MsdfPage],
    file: &FontFile,
    first: bool,
) {
    let Some(bytes) = bytes(cx, file, out) else {
        return;
    };
    let (table, findings) = GlyphRemapping::from_bytes(&bytes);
    if first {
        out.extend(at_file(findings, file));
    }
    let origin = Origin::new(
        file.pack.clone(),
        PackSide::Resource,
        file.path.clone(),
        None,
        Vec::new(),
    );
    // The pages are in page order.
    let page_of = |page: u8| {
        let found = pages.binary_search_by_key(&page, |held| held.page).ok()?;
        pages.get(found)
    };
    let mut told: HashSet<(u8, &str)> = HashSet::new();
    for (index, record) in table.0.iter().enumerate() {
        let [page, _] = record.to.to_be_bytes();
        let held = page_of(page);
        let lacks = [
            (held.is_none_or(|held| held.metrics.is_none()), ".fontdata"),
            (held.is_none_or(|held| held.image.is_none()), ".png"),
        ];
        for (lacking, extension) in lacks {
            if !lacking || !told.insert((page, extension)) {
                continue;
            }
            let mut path = JsonPath::new();
            path.push_index(index);
            let code = DiagnosticCode::MalformedInput {
                message: format!(
                    "character {:#06X} is drawn with a glyph of page {page:02X}, which the font has no file of",
                    record.from
                ),
                text: None,
            };
            let missing = format!("{prefix}{page:02X}{extension}");
            // The game reads the remapping records unchecked and the font family has no
            // content-log line for this: the crate reports an information.
            let at = At::new(&origin, path, Severity::Info);
            out.push(at.reference(code, FONT_FILE, &missing, None));
        }
    }
}
