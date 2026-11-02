//! Constructors and the small value methods of the text types.

use super::{KeyFilter, LangFile, LangLayout, Language, Line, RESERVED_KEYS, Translations};

impl LangFile {
    /// An empty file of `language`.
    #[must_use]
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
    #[must_use]
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
    /// under `prefix` (the pack's [`pack_key_prefix`](super::pack_key_prefix)).
    // The game's pack loader allows its reserved pack keys plus extra ones. The extra keys (format-3 settings and subpack names) are the
    // caller's to add: `include` is a public field.
    #[must_use]
    pub const fn pack_keywords(prefix: &'a str) -> Self {
        KeyFilter {
            exclude: &[],
            include: &RESERVED_KEYS,
            prefix,
        }
    }
}
