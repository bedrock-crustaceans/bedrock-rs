//! The font files of a stack by name: which pack supplies each, and the rules that names alone
//! decide.

use std::collections::HashMap;

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::load::{ListedFile, StackPack};
use bedrock_addon_core::pack_identity::{PackPath, PackRef};
use indexmap::IndexMap;

use super::codes;
use super::{FontEntry, FontFormat, FontMetadata, Fonts};
use bedrock_addon_core::diagnostic::builders::file_finding;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticDetail, Diagnostics, JsonPath, Severity,
};

/// The page prefix of the game's built-in MSDF fonts (`SmoothFontLatin`, `SmoothSmall`).
pub const SMOOTH_PREFIX: &str = "font/smooth/smooth_";
/// The extension of an MSDF page's metrics.
const FONTDATA: &str = ".fontdata";
/// The extensions an image is looked up with, in order.
const IMAGE: [&str; 7] = ["", ".tga", ".png", ".jpg", ".jpeg", ".hdr", ".ktx"];

/// A font file of a stack: the highest pack that has it, and its path there.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontFile {
    /// The pack.
    pub pack: PackRef,
    /// The file's path in the pack (subpack folder included).
    pub path: PackPath,
}

impl FontFile {
    /// The file `path` of `pack`.
    pub(crate) fn new(pack: &PackRef, path: &PackPath) -> Self {
        Self {
            pack: pack.clone(),
            path: path.clone(),
        }
    }
}

/// One page of an MSDF font as a stack supplies it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MsdfPage {
    /// The page: the high byte of its characters.
    pub page: u8,
    /// The atlas image (`<prefix>XX.png`), from the highest pack that has one.
    pub image: Option<FontFile>,
    /// The metrics (`<prefix>XX.fontdata`), from the highest pack that has them.
    pub metrics: Option<FontFile>,
}

/// The path of the remapping table of an MSDF font with the page prefix `font_file`:
/// `remapping.dat` in the prefix's directory (`font/smooth/remapping.dat`).
// A prefix without a directory gives the pack root.
#[must_use]
pub fn remapping_path(font_file: &str) -> String {
    match font_file.rsplit_once('/') {
        Some((directory, _)) => format!("{directory}/remapping.dat"),
        None => "remapping.dat".to_owned(),
    }
}

/// The stack index of a pack; the app data sorts below every pack.
fn rank(pack: &PackRef) -> usize {
    pack.index().unwrap_or(usize::MAX)
}

/// Whether a smooth page is one the client draws: its built-in glyph list has pages
/// `00` to `04`, `20`, `21`, `2E` to `D7` and `F9` to `FF`.
fn smooth_page_is_drawn(page: u8) -> bool {
    matches!(page, 0x00..=0x04 | 0x20 | 0x21 | 0x2E..=0xD7 | 0xF9..=0xFF)
}

/// `XX`, two upper-case hex digits, as a page.
fn page_of(digits: &str) -> Option<u8> {
    let upper = digits
        .bytes()
        .all(|b| b.is_ascii_digit() || (b'A'..=b'F').contains(&b));
    (digits.len() == 2 && upper).then(|| u8::from_str_radix(digits, 16).ok())?
}

/// A path as a page file: its prefix, page, and the extension's place in the image lookup
/// order (`None` for the metrics).
fn page_file(path: &str) -> Option<(&str, u8, Option<usize>)> {
    let (stem, extension) = if let Some(stem) = path.strip_suffix(FONTDATA) {
        (stem, None)
    } else {
        let (stem, place) = IMAGE[1..]
            .iter()
            .enumerate()
            .find_map(|(i, extension)| Some((path.strip_suffix(extension)?, i + 1)))
            .unwrap_or((path, 0));
        (stem, Some(place))
    };
    let split = stem.len().checked_sub(2)?;
    let (prefix, digits) = (stem.get(..split)?, stem.get(split..)?);
    Some((prefix, page_of(digits)?, extension))
}

/// The font directory a key lies directly in: `font/` or `texts/<language>/font/`; gives the
/// file name.
fn in_font_directory(path: &str) -> Option<&str> {
    let name = if let Some(name) = path.strip_prefix("font/") {
        name
    } else {
        let (_, rest) = path.strip_prefix("texts/")?.split_once('/')?;
        rest.strip_prefix("font/")?
    };
    (!name.contains('/')).then_some(name)
}

/// What the stack supplies and what names alone say about it.
pub(super) struct Index {
    pub(super) files: IndexMap<String, FontFile>,
    pub(super) pages: IndexMap<String, Vec<MsdfPage>>,
    pub(super) diagnostics: Diagnostics,
}

/// A page being assembled: the image with its pack rank and extension place.
#[derive(Default)]
struct Slot {
    image: Option<(usize, usize, FontFile)>,
    metrics: Option<FontFile>,
}

impl Index {
    /// Indexes the font files of `packs` (top first) and runs the name rules, given the
    /// stack's font list and where it was read from.
    pub(super) fn build(
        packs: &[StackPack],
        metadata: &FontMetadata,
        metadata_file: Option<&FontFile>,
    ) -> Self {
        // The MSDF prefixes whose pages count: the built-in one and each `msdf` font's.
        let mut prefixes: IndexMap<&str, HashMap<u8, Slot>> = IndexMap::new();
        prefixes.insert(SMOOTH_PREFIX, HashMap::new());
        let mut directories: Vec<&str> = Vec::new();
        for font in metadata.fonts.iter().flatten() {
            if let (Some(FontFormat::Msdf), Some(prefix)) = (font.format(), &font.font_file)
                && !prefixes.contains_key(prefix.as_str())
            {
                prefixes.insert(prefix, HashMap::new());
                if let Some((directory, _)) = prefix.rsplit_once('/') {
                    directories.push(directory);
                }
            }
        }
        directories.sort_unstable();
        directories.dedup();
        // The directories read, none inside another: `font`, `texts`, then the prefix
        // directories outside both.
        // Sorted, a directory comes right after the ones it lies in, so the last root taken
        // is the only other candidate.
        let mut roots = vec!["font", "texts"];
        for directory in directories {
            let inside = |root: &str| {
                directory == root
                    || directory
                        .strip_prefix(root)
                        .is_some_and(|rest| rest.starts_with('/'))
            };
            let last = roots.last().copied().unwrap_or("font");
            if !(inside("font") || inside("texts") || inside(last)) {
                roots.push(directory);
            }
        }
        let mut files: IndexMap<String, FontFile> = IndexMap::new();
        let mut diagnostics = Diagnostics::default();
        for pack in packs {
            for root in &roots {
                for file in below(pack, root) {
                    let path = file.key.as_str();
                    if *root == "texts" && in_font_directory(path).is_none() {
                        continue;
                    }
                    if files.contains_key(path) {
                        continue;
                    }
                    let found = FontFile::new(&pack.pack, &file.path);
                    if let Some((prefix, page, extension)) = page_file(path)
                        && let Some(slots) = prefixes.get_mut(prefix)
                    {
                        let slot = slots.entry(page).or_default();
                        let here = rank(&pack.pack);
                        match extension {
                            None => slot.metrics = Some(found.clone()),
                            Some(place) => {
                                let better = slot
                                    .image
                                    .as_ref()
                                    .is_none_or(|(rank, at, _)| (here, place) < (*rank, *at));
                                if better {
                                    slot.image = Some((here, place, found.clone()));
                                }
                            }
                        }
                    }
                    check_glyph_name(path, &found, &mut diagnostics);
                    files.insert(path.to_owned(), found);
                }
            }
        }
        let mut pages: IndexMap<String, Vec<MsdfPage>> = IndexMap::new();
        for (prefix, slots) in prefixes {
            let mut list: Vec<MsdfPage> = slots
                .into_iter()
                .map(|(page, slot)| MsdfPage {
                    page,
                    image: slot.image.map(|(_, _, file)| file),
                    metrics: slot.metrics,
                })
                .collect();
            list.sort_by_key(|page| page.page);
            for page in &list {
                check_page(prefix, page, &mut diagnostics);
            }
            pages.insert(prefix.to_owned(), list);
        }
        let mut index = Self {
            files,
            pages,
            diagnostics,
        };
        if let Some(file) = metadata_file {
            index.check_true_type(metadata, file);
        }
        index
    }

    /// Finds every TrueType font whose file no pack has under any extension the game tries.
    fn check_true_type(&mut self, metadata: &FontMetadata, file: &FontFile) {
        let fonts_at = JsonPath::new().child("fonts");
        for (i, font) in metadata.fonts.iter().flatten().enumerate() {
            let (Some(format), Some(font_file)) = (font.format(), &font.font_file) else {
                continue;
            };
            let extensions = format.file_extensions();
            if extensions.is_empty() {
                continue;
            }
            // A `font_file` that already ends in an extension is still given one more.
            let carries = extensions.iter().any(|e| font_file.ends_with(e));
            if carries || resolve(&self.files, font_file, extensions).is_none() {
                let mut finding = file_finding(
                    // The game's text.
                    DiagnosticCode::from(&codes::FONT_LOAD_FAILED),
                    Severity::Info,
                    &file.pack,
                    &file.path,
                );
                finding.location.json_path = fonts_at.index(i).child("font_file");
                let paths = extensions.iter().map(|e| format!("{font_file}{e}"));
                finding.detail = Some(Box::new(tried(paths)));
                self.diagnostics.push(finding);
            }
        }
    }
}

/// The paths the game tried for a font's file and found in no pack, in its order, as the
/// detail of a `FontFileMissing`.
fn tried(paths: impl IntoIterator<Item = String>) -> DiagnosticDetail {
    // A list, each path its own element; it prints as the joined name.
    DiagnosticDetail::TriedPaths {
        kind: "font file",
        paths: paths.into_iter().collect(),
    }
}

/// The files of `pack` below the directory `directory`, by key.
fn below<'a>(pack: &'a StackPack, directory: &'a str) -> impl Iterator<Item = &'a ListedFile> {
    bedrock_addon_core::json::kit::fixed_path::below(pack, directory)
}

/// Checks a file named `glyph_…` directly in a font directory that is no page name the game
/// builds (`glyph_` and two upper-case hex digits).
fn check_glyph_name(path: &str, file: &FontFile, diagnostics: &mut Diagnostics) {
    let Some(name) = in_font_directory(path) else {
        return;
    };
    let Some(rest) = name.strip_prefix("glyph_") else {
        return;
    };
    let stem = IMAGE[1..]
        .iter()
        .find_map(|extension| rest.strip_suffix(extension))
        .unwrap_or(rest);
    if page_of(stem).is_none() {
        diagnostics.push(file_finding(
            DiagnosticCode::from(&core_codes::IGNORED_FILE),
            Severity::Info,
            &file.pack,
            &file.path,
        ));
    }
}

/// The checks of one page of the MSDF prefix `prefix`: an atlas without widths, a page the
/// client never draws, and the two files of a page coming from two packs.
fn check_page(prefix: &str, page: &MsdfPage, diagnostics: &mut Diagnostics) {
    let at = |code, severity, file: &FontFile| file_finding(code, severity, &file.pack, &file.path);
    match (&page.image, &page.metrics) {
        // An atlas without widths.
        (Some(image), None) => {
            let mut finding: Diagnostic = at(
                // The game's text.
                DiagnosticCode::from(&codes::MSDF_INFO_MISSING),
                Severity::Info,
                image,
            );
            finding.detail = Some(Box::new(tried([format!(
                "{prefix}{:02X}.fontdata",
                page.page
            )])));
            diagnostics.push(finding);
        }
        // The two files of a page from two packs: reported at the higher file, naming
        // the other.
        (Some(image), Some(metrics)) if image.pack != metrics.pack => {
            let (higher, lower) = if rank(&image.pack) < rank(&metrics.pack) {
                (image, metrics)
            } else {
                (metrics, image)
            };
            let mut finding: Diagnostic = at(
                DiagnosticCode::from(&core_codes::MIXED_FONT_PAGE),
                Severity::Info,
                higher,
            );
            finding.detail = Some(Box::new(DiagnosticDetail::Reference {
                kind: "font page file",
                id: format!("{}: {}", lower.pack.name(), lower.path),
                hint: None,
            }));
            diagnostics.push(finding);
        }
        _ => {}
    }
    // A page the client's built-in glyph list lacks is never drawn.
    let drawn = page.page == 0 || (prefix == SMOOTH_PREFIX && smooth_page_is_drawn(page.page));
    if !drawn && let Some(file) = page.image.as_ref().or(page.metrics.as_ref()) {
        let code = DiagnosticCode::from(&core_codes::IGNORED_FILE);
        diagnostics.push(at(code, Severity::Info, file));
    }
}

/// The file the game finds for `stem` with one of `extensions`: the highest pack that has
/// one, and inside a pack the first extension in order.
fn resolve<'a>(
    files: &'a IndexMap<String, FontFile>,
    stem: &str,
    extensions: &[&str],
) -> Option<&'a FontFile> {
    extensions
        .iter()
        .enumerate()
        .filter_map(|(place, extension)| {
            let file = files.get(format!("{stem}{extension}").as_str())?;
            Some(((rank(&file.pack), place), file))
        })
        .min_by_key(|(order, _)| *order)
        .map(|(_, file)| file)
}

impl Fonts {
    /// The font file at `path` (`font/smooth/remapping.dat`): the highest pack that has it.
    /// Only font files are known here: the files below `font/`, in a `texts/<language>/font/`
    /// and in the directory of an MSDF font's pages.
    pub fn file(&self, path: &str) -> Option<&FontFile> {
        self.files.get(path)
    }

    /// The glyph sheet the game loads for `page` of a bitmap font with the page prefix
    /// `prefix` (`font/glyph_`), for a client in `language`: the highest pack that has the
    /// page, and in that pack the language's copy (`texts/<language>/font/glyph_XX`) before
    /// the pack's own.
    pub fn glyph_sheet(&self, prefix: &str, page: u8, language: Option<&str>) -> Option<&FontFile> {
        let stem = format!("{prefix}{page:02X}");
        let own = resolve(&self.files, &stem, &IMAGE);
        let overlay = language
            .and_then(|language| resolve(&self.files, &format!("texts/{language}/{stem}"), &IMAGE));
        match (overlay, own) {
            (Some(overlay), Some(own)) if rank(&own.pack) < rank(&overlay.pack) => Some(own),
            (Some(overlay), _) => Some(overlay),
            (None, own) => own,
        }
    }

    /// The image the game loads for the glyph sheet `stem` (`font/default8`, a page's
    /// `font/glyph_4E`) for a client in `language`, by the rule of
    /// [`glyph_sheet`](Self::glyph_sheet): what a pass that checks the sheets' bytes asks for
    /// a sheet that is no page.
    pub fn sheet(&self, stem: &str, language: Option<&str>) -> Option<&FontFile> {
        let own = resolve(&self.files, stem, &IMAGE);
        let overlay = language
            .and_then(|language| resolve(&self.files, &format!("texts/{language}/{stem}"), &IMAGE));
        match (overlay, own) {
            (Some(overlay), Some(own)) if rank(&own.pack) < rank(&overlay.pack) => Some(own),
            (Some(overlay), _) => Some(overlay),
            (None, own) => own,
        }
    }

    /// The file the game loads for a `ttf` or `ttfmsdf` font: `font_file` with the first
    /// extension the highest pack has (`.ttf`, `.otf`; for `ttfmsdf` also `.ttc`, `.otc`), so
    /// a higher pack's `.otf` stands before a lower pack's `.ttf`. `None` for another format
    /// and when no pack has the file.
    pub fn true_type_file(&self, font: &FontEntry) -> Option<&FontFile> {
        let extensions = font.format()?.file_extensions();
        resolve(&self.files, font.font_file.as_deref()?, extensions)
    }

    /// The pages the stack has of the MSDF font with the page prefix `prefix`, by page: the
    /// built-in [`SMOOTH_PREFIX`] or the `font_file` of an `msdf` font of the font list.
    pub fn msdf_pages(&self, prefix: &str) -> &[MsdfPage] {
        self.pages.get(prefix).map_or(&[], Vec::as_slice)
    }
}
