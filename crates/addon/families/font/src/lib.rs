//! The fonts of a resource stack: `font/font_metadata.json`,
//! `font/emoticons.json`, the MSDF page metrics and remapping table (binary), and which pack
//! supplies each font file. No file carries a `format_version`; the metadata's own `version`
//! must be 1.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod binary;
mod emoticons;
mod fields;
mod load;
mod metadata;
mod stack;

use std::ops::RangeInclusive;

use indexmap::IndexMap;

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics};
use bedrock_addon_core::document::{Extra, Merged};
use bedrock_addon_core::json::Value;

pub use binary::{GlyphRemapping, RemapEntry, check_glyph_sheet, png_size};
#[doc(hidden)]
pub use load::load_fonts;
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
    pub extra: Extra,
    /// The root as the author wrote it, kept when the fields cannot say it (an entry the game
    /// skips, a list that is no array, a number the fields hold narrowed).
    /// `to_json` writes it for as long as the fields are still the game's reading of it; a
    /// value that was changed, or built by hand, writes its fields. The entries the game
    /// skipped are in this root alone, so one edit of any field drops every one of them from
    /// the written file, not only those beside the edited member. `PartialEq` and `Debug`
    /// include the root: a loaded value that kept one differs from a hand-built value with
    /// the same fields.
    pub written: Option<Value>,
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
    pub extra: Extra,
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
    pub extra: Extra,
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
    pub extra: Extra,
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
    /// the written file, not only those beside the edited member. `PartialEq` and `Debug`
    /// include the root: a loaded value that kept one differs from a hand-built value with
    /// the same fields.
    pub written: Option<Value>,
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
    pub extra: Extra,
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
