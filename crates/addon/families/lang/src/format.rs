//! Looking a text up: the table lookup and the placeholder expansion of vanilla's
//! `Localization::_get`.

use super::Translations;

/// The identifier a chosen language rewrites to a line feed.
const LINEBREAK: &str = "~LINEBREAK~";

impl Translations {
    /// The value of `key` as stored: the exact key, then the key with `A`–`Z` lower-cased, so
    /// lookups are case-insensitive for ASCII as vanilla's are.
    // `_getSimple`: the exact id, then `Util::toLower(id)`.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.strings
            .get(key)
            .or_else(|| self.strings.get(&key.to_ascii_lowercase()))
            .map(String::as_str)
    }

    /// Stores `value` under `key`, the key lower-cased (`A`–`Z`) as the reader stores it; a
    /// later value replaces an earlier one.
    pub fn insert(&mut self, key: &str, value: impl Into<String>) {
        let value = value.into();
        self.may_contain_identifier |= value.contains('~');
        self.strings.insert(key.to_ascii_lowercase(), value);
    }

    /// Puts every entry of `other` on top of this table: its values replace the ones here.
    // `appendTranslations(Localization const&)` assigns.
    pub fn append(&mut self, other: &Translations) {
        self.may_contain_identifier |= other.may_contain_identifier;
        for (key, value) in &other.strings {
            self.strings.insert(key.clone(), value.clone());
        }
    }

    /// This language on top of the base language: a copy of `base` with this table's entries
    /// replacing its own, so every key this language lacks falls back to the base.
    // `Localization(code, parent)` copies the parent's `mStrings`; the language's own files
    // are appended on top. The `~` flag travels with the values it is
    // about.
    #[must_use]
    pub fn with_base(&self, base: &Translations) -> Translations {
        let mut table = base.clone();
        table.append(self);
        table
    }

    /// The text of `id` with `params` filled in, as the game shows it; `None` when `id` is a
    /// plain key the table does not have.
    ///
    /// An id that contains `%` is not looked up as a whole: each `%name` in it is replaced by
    /// the value of the key `name`, or by the bare name when the table has no such key, and
    /// `%%` is one `%`. A parameter that starts with `%` and names a key is that key's value.
    /// The parameters then fill the placeholders ([`replace_tokens`]).
    // `Localization::_get`.
    #[must_use]
    pub fn format(&self, id: &str, params: &[&str]) -> Option<String> {
        let text = if id.contains('%') {
            self.expand_keys(id)
        } else {
            self.get(id)?.to_owned()
        };
        let params: Vec<&str> = params
            .iter()
            .map(|param| {
                param
                    .strip_prefix('%')
                    .and_then(|key| self.get(key))
                    .unwrap_or(param)
            })
            .collect();
        Some(replace_tokens(&text, &params))
    }

    /// An id with `%` in it, each `%name` replaced by the key's value or the bare name.
    // `_parseFormattedString`: a name is a run of `[A-Za-z0-9._-]`;
    // the `%` is consumed; `%%1` gives `%1`. A `%` with no name after it is consumed like
    // any other, the empty name standing for it.
    fn expand_keys(&self, id: &str) -> String {
        let mut out = String::with_capacity(id.len());
        let mut rest = id;
        while let Some(at) = rest.find('%') {
            out.push_str(&rest[..at]);
            let after = &rest[at + 1..];
            if let Some(tail) = after.strip_prefix('%') {
                out.push('%');
                rest = tail;
                continue;
            }
            let end = after
                .find(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-')))
                .unwrap_or(after.len());
            let name = &after[..end];
            out.push_str(self.get(name).unwrap_or(name));
            rest = &after[end..];
        }
        out.push_str(rest);
        out
    }

    /// Rewrites the identifiers a chosen language expands: every `~LINEBREAK~` becomes a line
    /// feed. Does nothing unless [`may_contain_identifier`](Self::may_contain_identifier).
    ///
    /// Vanilla also rewrites `~UTC_DATETIME=<time>~` to a local date and time; that needs a
    /// clock and a time zone, which this crate does not have, so such a token stays as written.
    // `_chooseLanguage` calls `replaceIdentifiers` when `mMayContainIdentifier` is set
    pub fn replace_identifiers(&mut self) {
        if !self.may_contain_identifier {
            return;
        }
        for value in self.strings.values_mut() {
            if value.contains(LINEBREAK) {
                *value = value.replace(LINEBREAK, "\n");
            }
        }
    }
}

/// Fills `params` into the placeholders of `value`, as vanilla's `replaceTokens` does.
///
/// Without parameters the value is returned as it is. With at least one:
/// 1. every `$s` and `$d` is erased, wherever it stands (`%1$s` becomes `%1`);
/// 2. the `%s` and `%d` tokens take the parameters left to right, a token beyond the
///    parameters being erased; inserted text is scanned again;
/// 3. `%1` to `%9` (one digit) take the parameter at that position counted after the ones
///    step 2 consumed; an index without a parameter, `%0` included, is erased.
///
/// `%%` is not an escape: `+%d%% %s` with `5` and `Speed` is `+5%% Speed`.
// `Localization::replaceTokens`.
#[must_use]
pub fn replace_tokens(value: &str, params: &[&str]) -> String {
    if params.is_empty() {
        return value.to_owned();
    }
    let mut text = erase_dollar_tokens(value);
    // The search restarts from the start of the string after each insertion.
    let mut consumed = 0_usize;
    while let Some(at) = sequential_token(&text) {
        let param = params.get(consumed).copied().unwrap_or_default();
        text.replace_range(at..at + 2, param);
        consumed += 1;
    }
    fill_indices(&text, params, consumed)
}

/// `value` without its `$s` and `$d`, in one pass.
fn erase_dollar_tokens(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(at) = rest.find('$') {
        out.push_str(&rest[..at]);
        let after = &rest[at + 1..];
        if let Some(tail) = after.strip_prefix(['s', 'd']) {
            rest = tail;
        } else {
            out.push('$');
            rest = after;
        }
    }
    out.push_str(rest);
    out
}

/// Where the first `%s` or `%d` of `text` starts.
fn sequential_token(text: &str) -> Option<usize> {
    match (text.find("%s"), text.find("%d")) {
        (Some(s), Some(d)) => Some(s.min(d)),
        (s, d) => s.or(d),
    }
}

/// `text` with each `%N` replaced by `params[N - 1 + consumed]`, or erased when there is no
/// such parameter. One pass, left to right: inserted text is not scanned again.
fn fill_indices(text: &str, params: &[&str], consumed: usize) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find('%') {
        out.push_str(&rest[..at]);
        let after = &rest[at + 1..];
        if let Some(digit) = after.bytes().next().filter(u8::is_ascii_digit) {
            let position = usize::from(digit - b'0');
            let param = position
                .checked_sub(1)
                .and_then(|index| params.get(index + consumed));
            out.push_str(param.copied().unwrap_or_default());
            rest = &after[1..];
        } else {
            out.push('%');
            rest = after;
        }
    }
    out.push_str(rest);
    out
}
