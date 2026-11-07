//! `.lang` files: `texts/<code>.lang` of either pack side, the translations of one language
//! (`Translations`) and the pack keywords a manifest header names. The family is unversioned:
//! vanilla reads no `format_version` anywhere on this path, so there is no chain and no
//! `history`. A `.lang` file is read by a byte-level state machine
//! (`Localization::appendTranslations`), not a line splitter; `read.rs` reads what the machine finds.
//!
//! The language list a stack's translations are keyed by is the `languages` family's; the
//! stack composes the two.
//!
//! The reader stays text: the byte-level state machine above, with no JSON for a typed read to
//! take.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod codes;
mod format;
mod keyword;
mod language;
pub mod languages;
mod overlay;
mod read;
mod scan;
mod write;

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

pub use format::replace_tokens;
pub use keyword::{pack_key_prefix, pack_keyword_value};
pub use overlay::locale_overlay_paths;

pub use language::Language;
#[doc(hidden)]
pub use language::language_of;
pub use scan::{RESERVED_KEYS, is_pack_keyword};
use std::collections::btree_map::Entry;

use bedrock_addon_core::load::{Accept, CollectionLoader, StackCx, Walk, WalkFile};
use bedrock_addon_core::pack_identity::{PackPath, PackSide};
use indexmap::IndexMap;

use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::document::{
    Content, Document, Merge, Merged, Origin, Placed, Rejected, unversioned,
};
use bedrock_addon_core::json::{EncodeError, WriteStyle};
use bedrock_addon_core::load::LoadContext;

/// One `.lang` file, line by line as written (`texts/<code>.lang`).
///
/// The lines keep comments, blank lines and order, so a file that is read and written back
/// keeps its layout; line endings are written as LF. What the game makes of the file is
/// [`LangFile::translations`].
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[doc(alias = ".lang")]
pub struct LangFile {
    /// The language the file belongs to: the `<code>` of `texts/<code>.lang`.
    pub language: Language,
    /// The lines, in file order.
    pub lines: Vec<Line>,
    /// What the file had around its lines, so that it is written back as it was read.
    #[doc(hidden)]
    pub layout: LangLayout,
}

/// What a `.lang` file had around its lines: the presence record the writer honours, so a
/// file that is read and written back neither loses nor gains a byte order mark or its final
/// line feed. A file built in code has no byte order mark and ends its last line.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LangLayout {
    /// The file starts with a UTF-8 byte order mark.
    pub bom: bool,
    /// The last line ends with a line feed.
    pub final_line_feed: bool,
}

/// One line of a [`LangFile`].
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Line {
    /// An empty line, or one of only spaces and tabs. Written as an empty line.
    Blank,
    /// A comment line: the text after its `##`, up to the end of the line.
    Comment(String),
    /// `key=value`: a translation.
    Entry {
        /// The key as written. Vanilla lower-cases `A`–`Z` when it stores the key; spaces are
        /// part of the key, nothing is trimmed.
        key: String,
        /// The value, with `\n` (a line feed) and `\\` (a backslash) unescaped. Never empty in
        /// a file that was read: vanilla drops a line without a value.
        value: String,
        /// What follows the value on its line, from the tab or the `##` that ends it: the
        /// conventional `\t## comment`. Empty when the value runs to the end of the line.
        trailing: String,
    },
    /// A line vanilla drops, kept as written: a key without `=`, or an empty value.
    Dropped(String),
}

/// The translations of one language: what [`LangFile::translations`] gives for one file, and
/// what a stack merges for one code (`translations(&language)`), the highest pack's value
/// winning per key.
///
/// Values are stored as written; placeholders (`%s`, `%1`, `%key`) are expanded when a text
/// is looked up ([`Translations::format`]).
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Translations {
    /// Every key and its value. Keys are stored with `A`–`Z` lower-cased, as vanilla stores
    /// them; a pack's own `pack.name` and `pack.description` are stored under
    /// [`pack_key_prefix`] + the key.
    pub strings: BTreeMap<String, String>,
    /// A value contains `~`, so it may hold a `~LINEBREAK~` identifier that
    /// [`Translations::replace_identifiers`] rewrites when the language is chosen.
    pub may_contain_identifier: bool,
}

/// Which entries of a `.lang` file reach a table, and under which key: the three arguments
/// vanilla's reader takes besides the text.
///
/// A key is matched byte for byte, lower-cased and without the prefix. The stack loader uses
/// [`KeyFilter::GLOBAL`]; a pack's keywords are read with [`KeyFilter::pack_keywords`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct KeyFilter<'a> {
    /// Keys that are skipped.
    pub exclude: &'a [&'a str],
    /// When not empty, the only keys that are kept.
    pub include: &'a [&'a str],
    /// Put in front of every stored key.
    pub prefix: &'a str,
}

/// Both stacks: a behaviour pack carries `texts/` for its manifest name.
const SIDES: &[PackSide] = &[PackSide::Behavior, PackSide::Resource];

/// The directory the stack loaders read.
const TEXTS: &str = "texts";

/// The layers top first; within one pack the later copy first. The walks are lowest pack
/// first, so that is the walk order reversed.
fn top_first<T>(layers: &mut [Document<T>]) {
    layers.reverse();
}

impl bedrock_addon_core::__private::Sealed for LangFile {}
impl bedrock_addon_core::__private::Sealed for Translations {}

/// Keyed by the stored key: `Merged::origin` names the pack whose value won.
impl Merge for Translations {
    type Key = String;
}

impl Content for LangFile {
    const NAME: &'static str = "lang file";
    const SIDES: &'static [PackSide] = SIDES;

    /// Reads one `.lang` file; its language is the file name of `path` without `.lang`
    /// (`texts/en_US.lang`). Never rejects: vanilla drops bad lines one at a time.
    fn parse(path: &PackPath, bytes: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
        let origin = unversioned::lone(path);
        let language = language_of_file(path);
        let (file, diagnostics) = LangFile::from_bytes(language, bytes);
        Ok(unversioned::document(file, origin, diagnostics))
    }

    /// Writes the `.lang` text ([`LangFile::write_text`]); the style is JSON's and unused.
    fn write_json(&self, out: &mut Vec<u8>, _: WriteStyle) -> Result<(), EncodeError> {
        self.write_text(out)
    }

    /// The `.lang` text ([`LangFile::to_text`]).
    fn to_json(&self) -> Result<String, EncodeError> {
        self.to_text()
    }
}

impl Placed for LangFile {
    /// `texts/<code>.lang`.
    fn default_path(&self) -> PackPath {
        PackPath::new(&format!("texts/{}.lang", self.language))
            .unwrap_or_else(|_| PackPath::from_static("texts/_.lang"))
    }
}

/// The language a single `.lang` file is read as: the code of `texts/<code>.lang`; for a file
/// anywhere else, its name without the extension.
fn language_of_file(path: &PackPath) -> Language {
    language_of(path).unwrap_or_else(|| {
        let name = path.segments().last().unwrap_or_default();
        Language::new(name.strip_suffix(".lang").unwrap_or(name))
    })
}

/// What the `translations` line reads of one `texts/<code>.lang`: the file's table for the
/// language, and the keywords its pack reads of it.
#[derive(Debug)]
#[doc(hidden)]
pub struct LangLayer {
    document: Document<Translations>,
    keywords: Translations,
}

/// The `Parse` job of `translations(&language)`: the file's table as the stack loader reads
/// it, and the keywords its pack reads of it.
fn parse_lang(file: &WalkFile, _: &LoadContext) -> LangLayer {
    let origin = unversioned::origin_of(file);
    let language = language_of(&file.key).unwrap_or_else(|| Language::new(""));
    let (lang, diagnostics) = LangFile::from_bytes(language, &file.bytes);
    LangLayer {
        keywords: lang.pack_keywords(file.pack.id()),
        document: unversioned::document(lang.translations(), origin, diagnostics),
    }
}

/// The `Combine` job of `translations(&language)`: each copy is appended on the lower ones,
/// so the highest pack's value wins per key; the pack keywords keep their first definition.
// `loadFromResourcePackManager`: `appendTranslations` once per copy, lowest priority first,
// each assigning. Pack-keyword locales merge with `addKeys`
// (`try_emplace` only).
fn combine_translations(parsed: Vec<LangLayer>, _: &StackCx<'_>) -> Merged<Translations> {
    let mut value = Translations::new();
    let mut origins: IndexMap<String, Origin> = IndexMap::new();
    let mut layers: Vec<Document<Translations>> = Vec::new();
    let mut diagnostics = Diagnostics::default();
    for LangLayer { document, keywords } in parsed {
        diagnostics.extend(document.diagnostics().iter().cloned());
        value.may_contain_identifier |=
            document.may_contain_identifier || keywords.may_contain_identifier;
        for (key, text) in &document.strings {
            value.strings.insert(key.clone(), text.clone());
            origins.insert(key.clone(), document.origin().clone());
        }
        for (key, text) in keywords.strings {
            if let Entry::Vacant(slot) = value.strings.entry(key) {
                origins.insert(slot.key().clone(), document.origin().clone());
                slot.insert(text);
            }
        }
        layers.push(document);
    }
    top_first(&mut layers);
    Merged::new(value, origins, layers, diagnostics)
}

/// The loader of `translations(&language)`, on either stack: `texts/*.lang`, which the
/// accessor splits by language ([`language_of`]). The family's own: each copy carries the
/// pack keywords its pack reads of it beside its table (`LangLayer`), which `load::merged`'s
/// `FileCopy` cannot carry.
#[doc(hidden)]
pub const LOADER: CollectionLoader<Merged<Translations>, LangLayer> = CollectionLoader {
    // A `.lang` file, by the extension vanilla builds the path with (`getLangFilePath`
    // appends `.lang` as written).
    walk: Walk::json_children_in(&[TEXTS]).accept(Accept::Ext("lang")),
    parse: parse_lang,
    combine: combine_translations,
    needs: &[],
    then: None,
};

impl LangFile {
    /// An empty file of `language`.
    pub fn new(language: Language) -> Self {
        Self {
            language,
            lines: Vec::new(),
            layout: LangLayout::default(),
        }
    }
}

/// No byte order mark, and a line feed after the last line.
impl Default for LangLayout {
    fn default() -> Self {
        Self {
            bom: false,
            final_line_feed: true,
        }
    }
}

impl Line {
    /// The line `key=value`, with nothing after the value.
    pub fn entry(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self::Entry {
            key: key.into(),
            value: value.into(),
            trailing: String::new(),
        }
    }
}

impl Translations {
    /// An empty table.
    pub fn new() -> Self {
        Self::default()
    }
}

impl KeyFilter<'static> {
    /// What the stack loader reads of a file: every key but [`RESERVED_KEYS`].
    // `loadFromResourcePackManager`: `appendTranslations(content, blocked = mPackReservedKeys,
    // allowed = {}, prefix = "")`.
    pub const GLOBAL: KeyFilter<'static> = KeyFilter {
        exclude: &RESERVED_KEYS,
        include: &[],
        prefix: "",
    };
}

impl<'a> KeyFilter<'a> {
    /// What a pack reads of its own file for its manifest: only [`RESERVED_KEYS`], stored
    /// under `prefix` (the pack's [`pack_key_prefix`]).
    // The game's pack loader allows its reserved pack keys plus extra ones. The extra keys (format-3 settings and subpack names) are the
    // caller's to add: `include` is a public field.
    pub const fn pack_keywords(prefix: &'a str) -> Self {
        KeyFilter {
            exclude: &[],
            include: &RESERVED_KEYS,
            prefix,
        }
    }
}
