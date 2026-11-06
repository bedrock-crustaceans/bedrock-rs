//! The fonts of a resource stack: `font/font_metadata.json`,
//! `font/emoticons.json`, the MSDF page metrics and remapping table (binary), and which pack
//! supplies each font file. No file carries a `format_version`; the metadata's own `version`
//! must be 1.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod binary;
mod emoticons;
mod fields;
mod metadata;
mod stack;

use std::ops::RangeInclusive;

use indexmap::IndexMap;

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics};
use bedrock_addon_core::document::{Merged, UnknownMembers};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::json::kit::Written;

use crate::stack::Index;
use bedrock_addon_core::diagnostic::builders::malformed;
use bedrock_addon_core::diagnostic::{DiagnosticCode, JsonPath, JsonPathSegment, VanillaText};
use bedrock_addon_core::document::{
    Content, Document, JsonContent, Merge, Origin, Placed, Rejected, unversioned,
};
use bedrock_addon_core::json::build::json_text;
use bedrock_addon_core::json::jsoncpp::read;
use bedrock_addon_core::json::kit::fixed_path::Authored;
use bedrock_addon_core::json::{EncodeError, WriteStyle};
use bedrock_addon_core::load::Reported;
use bedrock_addon_core::load::{CollectionLoader, LoadContext, StackCx, Walk, WalkFile};
use bedrock_addon_core::pack_identity::{PackPath, PackSide};
pub use binary::{GlyphRemapping, RemapEntry, check_glyph_sheet, png_size};
pub use stack::{FontFile, MsdfPage, SMOOTH_PREFIX, remapping_path};

/// The path of the font list, in every resource pack.
pub const FONT_METADATA_PATH: &str = "font/font_metadata.json";
/// The path of the emoticon list, in every resource pack.
pub const EMOTICONS_PATH: &str = "font/emoticons.json";
/// The fonts the game has without any file; a `font_reference` may name them, and an entry of
/// a `font_metadata.json` with one of these names replaces the built-in font.
pub const BUILT_IN_FONTS: [&str; 6] = [
    "DefaultFont",
    "RuneFont",
    "UIFont",
    "UnicodeFont",
    "SmoothFontLatin",
    "SmoothSmall",
];

/// The fonts of a resource stack (`rp.fonts()`): the font list and the emoticon list, each the
/// highest pack's file, whole, and the pack that supplies each font file.
///
/// Hand-written rather than a `load::merged` collection: it partitions two file kinds (the
/// font list and the emoticon list) into one collection, and reads the packs' listings for the
/// font files no walk reads.
///
/// The game finds every font file on its own: a page image, a page's metrics, a TrueType file
/// each come from the highest pack that has that file, so one page can mix packs. Only the two
/// JSON files are read; the other files are known by name.
#[derive(Debug)]
#[non_exhaustive]
pub struct Fonts {
    /// `font/font_metadata.json`: the highest pack's file. Empty when no pack has one or the
    /// highest copy is refused; the game then has its built-in fonts only.
    pub metadata: Merged<FontMetadata>,
    /// `font/emoticons.json`: the highest pack's file.
    pub emoticons: Merged<EmoticonList>,
    /// Every font file of the stack by the path the game asks for, with the highest pack that
    /// has it: the files below `font/`, below each `texts/<language>/font/`, and beside each
    /// MSDF font's page prefix.
    #[doc(hidden)]
    pub files: IndexMap<String, FontFile>,
    /// The pages of each MSDF page prefix (the built-in one and each `msdf` font's), by page.
    #[doc(hidden)]
    pub pages: IndexMap<String, Vec<MsdfPage>>,
    /// The findings about the files, after the two lists' own.
    pub(crate) diagnostics: Diagnostics,
}

impl Fonts {
    /// Every finding: the font list's, the emoticon list's, then the ones about font files.
    pub fn diagnostics(&self) -> impl Iterator<Item = &Diagnostic> + '_ {
        self.metadata
            .diagnostics()
            .chain(self.emoticons.diagnostics())
            .chain(self.diagnostics.iter())
    }
}

/// One `font/font_metadata.json`: the fonts a pack adds and the aliases that combine fonts.
///
/// JSON: `{"version": 1, "fonts": [{"font_format": "ttf", "font_name": "MinecraftTen",
/// "font_file": "font/minecraft-ten"}], "font_aliases": [{"alias": "MinecraftTenEmoticon",
/// "fonts": [{"font_reference": "MinecraftTen"}]}]}`. `version` must be the integer 1 or the
/// game ignores the file. Entries the game skips (a font without a string `font_format` or
/// `font_name`, an alias it drops) are not held. Where a member is written in a way the typed
/// field cannot say (a `version` that is no integer, a scale written `1`), the field is `None`
/// and the member is kept in `extra` as written.
#[derive(Clone, Debug, Default, PartialEq)]
#[doc(alias = "font_metadata.json")]
pub struct FontMetadata {
    /// JSON `fonts`, in order. A later entry replaces an earlier one of its name.
    pub fonts: Option<Vec<FontEntry>>,
    /// JSON `font_aliases`, in order, read after every font.
    pub font_aliases: Option<Vec<FontAlias>>,
    /// The other members (`version` is not one: a loaded file's is 1).
    pub extra: UnknownMembers,
    /// The root as the author wrote it, kept when the fields cannot say it (an entry the game
    /// skips, a list that is no array, a number the fields hold narrowed).
    /// `to_json` writes it for as long as the fields are still the game's reading of it; a
    /// value that was changed, or built by hand, writes its fields. The entries the game
    /// skipped are in this root alone, so one edit of any field drops every one of them from
    /// the written file, not only those beside the edited member. `Debug` shows the root;
    /// `PartialEq` leaves it out: a loaded value that kept one equals a hand-built value with
    /// the same fields.
    pub written: Written,
}

/// How a font's glyphs are stored.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum FontFormat {
    /// `bitmap`: glyph sheets (`font/default8`, `font/glyph_XX`).
    Bitmap,
    /// `ttf`: a TrueType or OpenType file, rasterised by the game.
    Ttf,
    /// `ttfmsdf`: a TrueType, OpenType or collection file, drawn as distance fields.
    TtfMsdf,
    /// `msdf`: distance-field atlas pages with their metrics files.
    Msdf,
}

/// One entry of `fonts` in a `font_metadata.json`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FontEntry {
    /// JSON `font_format`: `bitmap`, `ttf`, `ttfmsdf` or `msdf`, compared byte for byte; an
    /// entry with any other text loads nothing.
    pub font_format: String,
    /// JSON `font_name`: the name aliases and UI files refer to.
    pub font_name: String,
    /// JSON `lowPerformanceCompatible` when a boolean (see
    /// [`is_low_performance_compatible`](Self::is_low_performance_compatible)).
    pub low_performance_compatible: Option<bool>,
    /// JSON `generate_pages_on_start` when an array: the pages drawn at start.
    pub generate_pages_on_start: Option<Vec<PageSpec>>,
    /// JSON `ascii_font_file` (bitmap): the sheet of the first 256 characters, no extension.
    pub ascii_font_file: Option<String>,
    /// JSON `unicode_file_prefix` (bitmap): the prefix of the glyph pages.
    pub unicode_file_prefix: Option<String>,
    /// JSON `font_file`. For `ttf` and `ttfmsdf` the file without its extension; for `msdf`
    /// the prefix of the pages (`font/smooth/smooth_`).
    pub font_file: Option<String>,
    /// JSON `version` (`ttf`, `ttfmsdf`) when an integer; the game uses 1 otherwise.
    pub version: Option<i32>,
    /// JSON `target_font_render_size` (`ttf`) when an integer; the game uses 64 otherwise
    /// ([`render_size`](Self::render_size) gives the size it ends up with).
    pub target_font_render_size: Option<i32>,
    /// The other members, and the members above as written when the field cannot say them.
    pub extra: UnknownMembers,
}

/// One element of `generate_pages_on_start`.
#[derive(Clone, Debug, PartialEq)]
pub enum PageSpec {
    /// A page number.
    Page(i32),
    /// `{"first_page": 2, "last_page": 4}`: every page from the first to the last.
    Range {
        /// JSON `first_page`.
        first_page: i32,
        /// JSON `last_page`. A range whose last page is below its first has no page.
        last_page: i32,
    },
    /// Anything else, as written: the game skips it.
    Other(Value),
}

/// One entry of `font_aliases`: a name for a list of fonts tried in order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FontAlias {
    /// JSON `alias`: the name.
    pub alias: String,
    /// JSON `fonts`: at least one.
    pub fonts: Vec<FontAliasRef>,
    /// JSON `font_scale_factor_override` when written as a decimal number; the game ignores an
    /// integer.
    pub font_scale_factor_override: Option<f64>,
    /// The other members, and `font_scale_factor_override` as written when it is no decimal.
    pub extra: UnknownMembers,
}

/// One font of an alias.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FontAliasRef {
    /// JSON `font_reference`: a built-in font, a `fonts` entry or an earlier alias, by its
    /// exact name.
    pub font_reference: String,
    /// JSON `font_language_code`: the language the font is used for. A value that is neither
    /// a string nor `null` reads as the empty code.
    pub font_language_code: Option<String>,
    /// JSON `font_ranges`: the code points the font is used for.
    pub font_ranges: Option<Vec<FontRange>>,
    /// JSON `font_scale` when written as a decimal number; the game ignores an integer.
    pub font_scale: Option<f64>,
    /// The other members, and the members above as written when the field cannot say them.
    pub extra: UnknownMembers,
}

/// A range of code points of an alias font: `{"first": 57344, "last": 63743}`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct FontRange {
    /// JSON `first`.
    pub first: i32,
    /// JSON `last`.
    pub last: i32,
}

impl FontRange {
    /// The code points of the range.
    #[must_use]
    pub fn code_points(self) -> RangeInclusive<i32> {
        self.first..=self.last
    }
}

/// One `font/emoticons.json`: names for private-use characters, typed in chat as `:name:`.
///
/// JSON: `[{"name": "food", "code": "0xE100"}]`. Elements the game skips (a value that is no
/// object, a `name` or `code` that is no string) are not held.
#[derive(Clone, Debug, Default, PartialEq)]
#[doc(alias = "emoticons.json")]
pub struct EmoticonList {
    /// The emoticons the game reads, in order: the root's elements with a string `name` and a
    /// string `code`.
    pub emoticons: Vec<Emoticon>,
    /// The root as the author wrote it, kept when the fields cannot say it (a root that is no
    /// array, an element the game skips).
    /// `to_json` writes it for as long as the fields are still the game's reading of it; a
    /// value that was changed, or built by hand, writes its fields. The entries the game
    /// skipped are in this root alone, so one edit of any field drops every one of them from
    /// the written file, not only those beside the edited member. `Debug` shows the root;
    /// `PartialEq` leaves it out: a loaded value that kept one equals a hand-built value with
    /// the same fields.
    pub written: Written,
}

/// One emoticon.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Emoticon {
    /// JSON `name`: matched in text between colons; only letters, digits and `_` can match.
    pub name: String,
    /// JSON `code`: the character as a base-16 number (`0xE100`); see
    /// [`code_point`](Self::code_point).
    pub code: String,
    /// The other members.
    pub extra: UnknownMembers,
}

/// The metrics of one MSDF font page (`font/smooth/smooth_41.fontdata`): how far each of the
/// page's 256 characters advances the pen.
///
/// Binary, little-endian: a `u32` that must be 0, then 256 `f32`, 1028 bytes in all.
#[derive(Clone, Debug, PartialEq)]
#[doc(alias = "fontdata")]
pub struct GlyphPage {
    /// The advance of character `page * 256 + i`, in units of the font height (8 pixels).
    pub advances: [f32; 256],
}

/// One copy of one of the two files: the document in its pack, or its rejection.
type FontFileCopy<T> = Result<Document<T>, Rejected>;

impl bedrock_addon_core::__private::Sealed for FontMetadata {}
impl bedrock_addon_core::__private::Sealed for EmoticonList {}

/// Keyed by a font's or an alias's name: `Merged::origin` names the pack whose file is read.
impl Merge for FontMetadata {
    type Key = String;
}

/// Keyed by an emoticon's name.
impl Merge for EmoticonList {
    type Key = String;
}

/// Reads one `font/font_metadata.json`.
fn read_metadata(origin: Origin, bytes: &[u8]) -> FontFileCopy<FontMetadata> {
    let mut root = match read(bytes) {
        Ok(root) => root,
        Err(message) => {
            // The game keeps its default fonts and writes its line to the developer log, not the
            // content log: an information with its text, the file still refused.
            let code = malformed(message.to_string(), Some(metadata::NOT_JSON));
            return Err(unversioned::reject(origin, Diagnostic::info(code)));
        }
    };
    if !FontMetadata::version_is_supported(&root) {
        // The file is ignored; the game's line goes to the developer log, so this is an
        // information, and the file is still refused.
        let code = DiagnosticCode::UnsupportedVersion {
            text: Some(VanillaText::Fixed(metadata::VERSION)),
        };
        let at = JsonPath::new().child("version");
        return Err(unversioned::reject(origin, Diagnostic::info(code).at(at)));
    }
    let mut diagnostics = Diagnostics::default();
    // A number no double holds is read as the largest finite one, with a warning.
    let mut beyond = Diagnostics::default();
    bedrock_addon_core::json::clamp_non_finite_warned(
        &mut root,
        None,
        &JsonPath::new(),
        &mut beyond,
    );
    let metadata = FontMetadata::from_value(&root, &mut diagnostics);
    // A scale is held at the width of a float and says so itself at the number.
    for finding in beyond {
        let at = &finding.location.json_path;
        let is_scale = matches!(
            at.segments().last(),
            Some(JsonPathSegment::Key(
                "font_scale" | "font_scale_factor_override"
            ))
        );
        let said = is_scale
            && diagnostics.iter().any(|own| {
                matches!(own.code, DiagnosticCode::ClampedValue { .. })
                    && own.location.json_path == *at
            });
        if !said {
            diagnostics.push(finding);
        }
    }
    Ok(unversioned::document(metadata, origin, diagnostics))
}

/// Reads one `font/emoticons.json`.
fn read_emoticons(origin: Origin, bytes: &[u8]) -> FontFileCopy<EmoticonList> {
    let mut root = match read(bytes) {
        Ok(root) => root,
        Err(message) => {
            // No emoticons; the game's "Malformed JSON" is a developer assert: an information
            // with its text, the file still refused.
            let code = malformed(message.to_string(), Some(emoticons::MALFORMED));
            return Err(unversioned::reject(origin, Diagnostic::info(code)));
        }
    };
    let mut diagnostics = Diagnostics::default();
    // A number no double holds is read as the largest finite one, with a warning.
    bedrock_addon_core::json::clamp_non_finite_warned(
        &mut root,
        None,
        &JsonPath::new(),
        &mut diagnostics,
    );
    let list = EmoticonList::from_value(&root, &mut diagnostics);
    Ok(unversioned::document(list, origin, diagnostics))
}

impl Content for FontMetadata {
    const NAME: &'static str = "font metadata";
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// Reads one `font/font_metadata.json`. A file that is not JSON, and one whose `version`
    /// is not the integer 1 (`1.0` and `"1"` are not), is rejected: the game ignores it.
    fn parse(path: &PackPath, bytes: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
        read_metadata(unversioned::lone(path), bytes)
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        json_text(&self.to_value(), out, style)
    }
}

impl JsonContent for FontMetadata {}

impl Placed for FontMetadata {
    /// `font/font_metadata.json`.
    fn default_path(&self) -> PackPath {
        PackPath::from_static(FONT_METADATA_PATH)
    }
}

impl Content for EmoticonList {
    const NAME: &'static str = "emoticon list";
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// Reads one `font/emoticons.json`. Only a file that is not JSON is rejected, with an
    /// error; a root that is no array is a list without emoticons, with an information.
    fn parse(path: &PackPath, bytes: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
        read_emoticons(unversioned::lone(path), bytes)
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        json_text(&self.to_value(), out, style)
    }
}

impl JsonContent for EmoticonList {}

impl Placed for EmoticonList {
    /// `font/emoticons.json`.
    fn default_path(&self) -> PackPath {
        PackPath::from_static(EMOTICONS_PATH)
    }
}

/// One of the two JSON files of `font/`, read.
#[derive(Debug)]
pub enum FontCopy {
    /// `font/font_metadata.json`.
    Metadata(FontFileCopy<FontMetadata>),
    /// `font/emoticons.json`.
    Emoticons(FontFileCopy<EmoticonList>),
}

fn parse_font(file: &WalkFile, _: &LoadContext) -> FontCopy {
    let origin = unversioned::origin_of(file);
    if file.key.as_str() == FONT_METADATA_PATH {
        FontCopy::Metadata(read_metadata(origin, &file.bytes))
    } else {
        FontCopy::Emoticons(read_emoticons(origin, &file.bytes))
    }
}

/// The highest copy of a file read whole: its value, the origin of each of its `names`, every
/// copy that reads as a layer, and the highest copy's findings alone. A highest copy that is
/// refused leaves the default value: the game does not turn to a lower pack's file.
fn highest<T: Merge<Key = String> + Clone + Default>(
    copies: Vec<FontFileCopy<T>>,
    names: impl Fn(&T) -> Vec<String>,
) -> Merged<T> {
    let mut layers = Vec::new();
    let mut diagnostics = Diagnostics::default();
    let mut origins = IndexMap::new();
    let mut value = T::default();
    let count = copies.len();
    for (place, copy) in copies.into_iter().enumerate() {
        let top = place + 1 == count;
        match copy {
            Ok(layer) => {
                if top {
                    diagnostics.extend(layer.diagnostics().iter().cloned());
                    value = layer.value().clone();
                    for name in names(&value) {
                        origins.insert(name, layer.origin().clone());
                    }
                }
                layers.push(layer);
            }
            Err(rejected) if top => diagnostics.extend(rejected.diagnostics),
            Err(_) => {}
        }
    }
    layers.reverse();
    Merged::new(value, origins, layers, diagnostics)
}

/// The `Combine` job of `rp.fonts()`.
fn combine(parsed: Vec<FontCopy>, cx: &StackCx<'_>) -> Fonts {
    let mut metadata_copies = Vec::new();
    let mut emoticon_copies = Vec::new();
    // Where the highest font list lies, read or refused.
    let mut metadata_file = None;
    for copy in parsed {
        match copy {
            FontCopy::Metadata(copy) => {
                let origin = match &copy {
                    Ok(layer) => layer.origin(),
                    Err(rejected) => &rejected.origin,
                };
                metadata_file = Some(FontFile::new(&origin.pack, &origin.path));
                metadata_copies.push(copy);
            }
            FontCopy::Emoticons(copy) => emoticon_copies.push(copy),
        }
    }
    let metadata = highest(metadata_copies, |metadata: &FontMetadata| {
        let fonts = metadata.fonts.iter().flatten().map(|f| f.font_name.clone());
        let aliases = metadata.font_aliases.iter().flatten();
        fonts.chain(aliases.map(|a| a.alias.clone())).collect()
    });
    let emoticons = highest(emoticon_copies, |list: &EmoticonList| {
        list.emoticons
            .iter()
            .map(|emoticon| emoticon.name.clone())
            .collect()
    });
    let index = Index::build(cx.packs, metadata.value(), metadata_file.as_ref());
    Fonts {
        metadata,
        emoticons,
        files: index.files,
        pages: index.pages,
        diagnostics: index.diagnostics,
    }
}

impl Reported for Fonts {
    fn reported(&self) -> Box<dyn Iterator<Item = &Diagnostic> + '_> {
        Box::new(self.diagnostics())
    }
}

/// The loader of `rp.fonts()`: `font/font_metadata.json` and `font/emoticons.json` of every
/// resource pack. The other font files are known from each pack's listing; none is read.
#[doc(hidden)]
pub const LOADER: CollectionLoader<Fonts, FontCopy> = CollectionLoader {
    walk: Walk::exact_files(&[FONT_METADATA_PATH, EMOTICONS_PATH]),
    parse: parse_font,
    combine,
    needs: &[],
    then: None,
};
