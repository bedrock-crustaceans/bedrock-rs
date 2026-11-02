//! Pack keywords: the `pack.name` / `pack.description` a manifest header names and the pack's
//! own `.lang` files translate.

use bedrock_addon_core::pack_identity::PackIdVersion;

use super::{KeyFilter, LangFile, Language, Translations, is_pack_keyword};

/// The prefix a pack's keywords are stored under: the header UUID followed directly by the
/// header version, with no separator (`ee649bcf-256c-4013-9068-6a802b89d7560.0.1`).
pub fn pack_key_prefix(pack: &PackIdVersion) -> String {
    // `_generatePackKeyPrefix`: `getIdentity().mId.asString()` followed directly by
    // `getIdentity().mVersion.asString()`. The two text formats are
    // those of `mce::UUID::asString` and `SemVersion::asString`.
    format!("{}{}", pack.uuid, pack.version)
}

/// What a pack shows for a header `name` or `description`: `text` itself unless it is a pack
/// keyword; for a keyword, its value in the first of `tables` (the current language, then the
/// base language) that the pack's own `texts/languages.json` lists (`listed`) and whose table
/// has it; otherwise the keyword itself.
pub fn pack_keyword_value<'a>(
    text: &'a str,
    pack: &PackIdVersion,
    listed: impl Fn(&Language) -> bool,
    tables: &[(&Language, &'a Translations)],
) -> &'a str {
    if !is_pack_keyword(text) {
        return text;
    }
    // The codes come from this pack's own `texts/languages.json`
    // (`getLanguageCodesFromPack`, stored in `mLanguageCodesForPackKeywords`).
    // `getPackKeywordValue` starts from the keyword text and tries the current pack-meta
    // language, then `mLanguages[0]`.
    tables
        .iter()
        .filter(|(language, _)| listed(language))
        .find_map(|(_, table)| table.pack_keyword(pack, text))
        .unwrap_or(text)
}

impl LangFile {
    /// The keywords `pack` reads of this file for its manifest: `pack.name` and
    /// `pack.description`, stored under the pack's [`pack_key_prefix`].
    pub fn pack_keywords(&self, pack: &PackIdVersion) -> Translations {
        self.select(&KeyFilter::pack_keywords(&pack_key_prefix(pack)))
    }
}

impl Translations {
    /// The value of the keyword `keyword` (`pack.name`, `pack.description`) that `pack`
    /// defines in this language.
    pub fn pack_keyword(&self, pack: &PackIdVersion, keyword: &str) -> Option<&str> {
        self.strings
            .get(&format!("{}{keyword}", pack_key_prefix(pack)))
            .map(String::as_str)
    }
}
