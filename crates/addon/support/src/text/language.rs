//! The language code, which `languages.json` lists and `texts/<code>.lang` is named by.

use std::fmt;

use bedrock_addon_core::pack_identity::PackPath;

/// A language code: an element of `texts/languages.json` and the stem of `texts/<code>.lang`
/// (`en_US`).
///
/// Vanilla checks no format when it reads a code from a pack, so any text is a code; the form
/// `xx_YY` is only required of the code a client logs in with ([`Language::is_valid_code`]).
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Language(String);

impl Language {
    /// The language with this code. Any text is a code: vanilla checks no format when it reads
    /// one from a pack.
    pub fn new(code: impl Into<String>) -> Self {
        Self(code.into())
    }

    /// The code as written (`en_US`).
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Whether the code has the form a client may log in with: exactly five bytes,
    /// `[a-z][a-z]_[A-Z][A-Z]` (`en_US`). A server keeps its default language for a player
    /// whose login code fails this; codes read from a pack are never checked.
    // Only the code a client logs in with is checked, when a player is created; codes read
    // from a pack are never checked.
    #[must_use]
    pub fn is_valid_code(&self) -> bool {
        match self.as_str().as_bytes() {
            [a, b, b'_', c, d] => {
                a.is_ascii_lowercase()
                    && b.is_ascii_lowercase()
                    && c.is_ascii_uppercase()
                    && d.is_ascii_uppercase()
            }
            _ => false,
        }
    }
}

/// The code as written.
impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for Language {
    fn from(code: &str) -> Self {
        Self::new(code)
    }
}

impl From<String> for Language {
    fn from(code: String) -> Self {
        Self::new(code)
    }
}

impl AsRef<str> for Language {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Language {
    /// The language of `texts/<code>.lang`: the code vanilla builds the path from.
    // The path of a language's file is `texts/` + code + `.lang`.
    #[must_use]
    pub fn from_path(path: &PackPath) -> Option<Self> {
        let code = path
            .as_str()
            .strip_prefix("texts/")?
            .strip_suffix(".lang")?;
        (!code.is_empty() && !code.contains('/')).then(|| Self::new(code))
    }
}

/// The language of `texts/<code>.lang`, as the function the stack accessors take as a key
/// reader: [`Language::from_path`].
#[must_use]
pub fn language_of(path: &PackPath) -> Option<Language> {
    Language::from_path(path)
}

#[cfg(test)]
mod tests {
    use bedrock_addon_core::pack_identity::PackPath;

    use super::Language;

    fn lang(code: &str) -> Language {
        Language::new(code)
    }

    #[test]
    fn the_language_of_a_lang_file_is_its_stem_directly_in_texts() {
        let of = |path: &str| Language::from_path(&PackPath::new(path).unwrap());
        assert_eq!(of("texts/en_US.lang"), Some(lang("en_US")));
        // vanilla checks no form
        assert_eq!(of("texts/klingon.lang"), Some(lang("klingon")));
        assert_eq!(of("texts/my lang.lang"), Some(lang("my lang")));
    }

    #[test]
    fn the_free_function_is_the_constructor() {
        let path = PackPath::new("texts/de_DE.lang").unwrap();
        assert_eq!(super::language_of(&path), Language::from_path(&path));
    }

    #[test]
    fn any_other_path_is_no_languages_file() {
        let of = |path: &str| Language::from_path(&PackPath::new(path).unwrap());
        for path in [
            "texts/languages.json",
            "texts/en_US.LANG",
            "texts/en_US.lang.bak",
            "texts/en_US.txt",
            "texts/.lang",
            "texts/ja_JP/en_US.lang",
            "Texts/en_US.lang",
            "en_US.lang",
            "other/en_US.lang",
            "texts",
        ] {
            assert_eq!(of(path), None, "{path}");
        }
    }
}
