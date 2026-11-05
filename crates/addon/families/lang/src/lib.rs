//! `.lang` files: `texts/<code>.lang` of either pack side, the translations of one language
//! (`Translations`) and the pack keywords a manifest header names. The family is unversioned:
//! vanilla reads no `format_version` anywhere on this path, so there is no chain and no
//! `history`. A `.lang` file is read by a byte-level state machine
//! (`Localization::appendTranslations`), not a line splitter; `read.rs` reads what the machine finds.
//!
//! The language list a stack's translations are keyed by is the `languages` family's; the
//! stack composes the two.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod format;
mod keyword;
mod load;
mod new;
mod overlay;
mod read;
mod write;

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

pub use format::replace_tokens;
pub use keyword::{pack_key_prefix, pack_keyword_value};
pub use overlay::locale_overlay_paths;

pub use bedrock_addon_support::text::lang_scan::{RESERVED_KEYS, is_pack_keyword};
pub use bedrock_addon_support::text::language::Language;
#[doc(hidden)]
pub use bedrock_addon_support::text::language::language_of;

#[doc(hidden)]
pub use load::LOADER;

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
