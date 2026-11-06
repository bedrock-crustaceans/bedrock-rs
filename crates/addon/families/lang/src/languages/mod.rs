//! `texts/languages.json` and `texts/language_names.json` of either pack side: the language
//! codes a pack lists, their display names and the locale fallback chain. The family is
//! unversioned: vanilla reads no `format_version` anywhere on this path, so there is no chain
//! and no `history`.
//!
//! Beside the two lists the load looks at what else lies below `texts/` (a `.lang` file of a
//! code no list names, the locale overlays, the pack keywords) and says what the game
//! ignores.
//!
//! The reader stays a hand walk over the two JSON lists, read as vanilla's JSON library reads them:
//! no schema stands behind them.

mod content;
mod read;
mod stack;

#[cfg(test)]
mod tests;

pub use crate::Language;

#[doc(hidden)]
pub use stack::LOADER;

/// The languages of a pack or a stack: the codes of `texts/languages.json` and the display
/// names of `texts/language_names.json`.
///
/// JSON (`languages.json`): `["en_US", "en_GB", …]`. As a stack's merged value (`languages()`),
/// `codes` is the union of every pack's list in first-seen order, lowest pack first, so a
/// pack can add a language but cannot move the first one, which is the base every other
/// language falls back to; `names` holds one name per code, the highest pack's non-empty name.
/// As one pack's layer, the value holds what that one file gave: the codes of its
/// `languages.json`, or the names of its `language_names.json`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[doc(alias = "languages.json")]
pub struct Languages {
    /// The language codes, in order (JSON: the elements of the root array).
    pub codes: Vec<Language>,
    /// The display names (`texts/language_names.json`); not written to `languages.json`.
    pub names: LanguageNames,
}

/// The display names of languages (`texts/language_names.json`).
///
/// JSON: `[["en_US", "English (United States)"], …]`, an array of `[code, name]` pairs. An
/// entry with a name sets the code's name; an entry with an empty name only makes the code
/// known and never clears a name ([`LanguageNames::get`]).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[doc(alias = "language_names.json")]
pub struct LanguageNames {
    /// The `[code, name]` pairs, in order (JSON: the elements of the root array).
    pub entries: Vec<(Language, String)>,
}
