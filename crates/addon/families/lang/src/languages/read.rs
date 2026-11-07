//! `texts/languages.json` and `texts/language_names.json`: the language list, the display
//! names and the locale fallback.

use super::{Language, LanguageNames, Languages};
use crate::codes;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, JsonPath, Severity};
use bedrock_addon_core::json::{Prepared, Value};

impl Languages {
    /// The base language: the first code. Every other language starts from a copy of its
    /// table, and it is the last fallback of a locale lookup. `None` for an empty list.
    // `mLanguageCodes[0]` is loaded first, with no parent.
    pub fn base(&self) -> Option<&Language> {
        self.codes.first()
    }

    /// Whether `language` is listed.
    pub fn contains(&self, language: &Language) -> bool {
        self.codes.contains(language)
    }

    /// The listed language the game uses for a requested code: the code itself when listed;
    /// otherwise the first listed code that contains the requested code's part before its
    /// first `_` (the whole code when it has none) anywhere in it; otherwise the base
    /// language. `None` for an empty list.
    // `I18nImpl::_getLocaleCodeFor`: `std::string::find`, not a prefix
    // match.
    pub fn locale_for(&self, requested: &str) -> Option<&Language> {
        if let Some(exact) = self.codes.iter().find(|code| code.as_str() == requested) {
            return Some(exact);
        }
        let prefix = requested.split('_').next().unwrap_or(requested);
        self.codes
            .iter()
            .find(|code| code.as_str().contains(prefix))
            .or_else(|| self.base())
    }

    /// The display name of `language` ([`LanguageNames::get`]).
    pub fn name(&self, language: &Language) -> Option<&str> {
        self.names.get(language)
    }
}

impl LanguageNames {
    /// Gives `language` the name `name` as vanilla's reader does: a name replaces the code's
    /// name; an empty name only makes the code known and never clears a name.
    // A non-empty name overwrites the map entry; an empty one only creates it when absent
    pub fn set(&mut self, language: Language, name: &str) {
        match self.entries.iter_mut().find(|(code, _)| *code == language) {
            Some((_, current)) if !name.is_empty() => name.clone_into(current),
            Some(_) => {}
            None => self.entries.push((language, name.to_owned())),
        }
    }

    /// Puts the entries of `other` (a higher pack's file) on top of these, each by
    /// [`set`](Self::set).
    pub fn append(&mut self, other: &LanguageNames) {
        for (language, name) in &other.entries {
            self.set(language.clone(), name);
        }
    }

    /// The display name of `language`: the last non-empty name among its entries; `""` when
    /// its entries all have an empty name; `None` when it has no entry.
    pub fn get(&self, language: &Language) -> Option<&str> {
        let mut names = self
            .entries
            .iter()
            .filter(|(code, _)| code == language)
            .map(|(_, name)| name.as_str());
        let first = names.next()?;
        Some(names.rfind(|name| !name.is_empty()).unwrap_or(first))
    }
}

/// The root of a JSON text file; a file that does not parse is an error. Vanilla skips such a
/// copy without a word (V6); the other packs' copies still count.
// `Json::Reader::parse` fails → the copy is skipped. Comments are
// accepted as jsoncpp accepts them.
fn root(bytes: &[u8]) -> Result<Value, Box<Diagnostic>> {
    let malformed = |message: String| {
        // vanilla skips such a copy without a word: ours, an information, the copy still
        // refused
        Box::new(
            Diagnostic::of(bedrock_addon_core::diagnostic::builders::syntax(message))
                .severity(Severity::Info),
        )
    };
    // Either failure of the gate is the one text.
    let prepared = Prepared::new(bytes)
        .map_err(|_| malformed("not UTF-8 JSON within the nesting limit".to_owned()))?;
    // of a member written twice the last stands.
    bedrock_addon_core::json::from_str_jsonc(&bedrock_addon_core::json::last_wins(
        prepared.as_str(),
    ))
    .map_err(|error| malformed(error.to_string()))
}

/// The elements vanilla iterates of a root, each with its path: the elements of an array; the
/// member values of an object in key order (the object is a `std::map`); nothing for any
/// other root. A root that is no array is a warning of ours.
// The game's JSON iteration is empty for every type but array and object.
// That the key order is byte order is assumed.
fn elements<'v>(root: &'v Value, diagnostics: &mut Diagnostics) -> Vec<(JsonPath, &'v Value)> {
    if let Some(array) = root.as_array() {
        return array
            .iter()
            .enumerate()
            .map(|(index, element)| {
                let at = JsonPath::new().index(index);
                (at, element)
            })
            .collect();
    }
    diagnostics.push(Diagnostic::of(&core_codes::INVALID_TYPE).severity(Severity::Info));
    let Some(object) = root.as_object() else {
        return Vec::new();
    };
    let mut members: Vec<(&str, &Value)> = object
        .iter()
        .map(|(key, value)| (key.as_str(), value))
        .collect();
    members.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
    members
        .into_iter()
        .map(|(key, value)| {
            let at = JsonPath::new().child(key);
            (at, value)
        })
        .collect()
}

/// `Json::Value::asString("")`: a string is its text, a boolean `true` / `false`, and null,
/// numbers, arrays and objects are empty.
fn as_string(value: Option<&Value>) -> String {
    let Some(value) = value else {
        return String::new();
    };
    if let Some(text) = value.as_string() {
        text.as_str().to_owned()
    } else if let Some(flag) = value.as_bool() {
        flag.to_string()
    } else {
        String::new()
    }
}

/// Reads `texts/languages.json`: the codes in order, each once.
///
/// An element that is no string is skipped with a warning (L9); vanilla turns it into a code
/// (`"true"`, `"false"` or `""`) and appends that. A code that is not of the form `xx_YY` is
/// kept with an info (L8); vanilla checks no form.
// `_findAvailableLanguages`: appended unless an equal string is in
// the list already.
pub(super) fn read_languages(bytes: &[u8]) -> Result<(Languages, Diagnostics), Box<Diagnostic>> {
    let root = root(bytes)?;
    let mut diagnostics = Diagnostics::default();
    let mut codes: Vec<Language> = Vec::new();
    for (at, element) in elements(&root, &mut diagnostics) {
        // `vanilla = "asString(\"\"): … appended without an emptiness
        // check"`.
        let Some(code) = element.as_string() else {
            diagnostics.push(
                Diagnostic::of(&core_codes::INVALID_TYPE)
                    .severity(Severity::Info)
                    .at(at),
            );
            continue;
        };
        let code = Language::new(code.as_str());
        if codes.contains(&code) {
            continue;
        }
        // `vanilla = "accepted, no check"`.
        if !code.is_valid_code() {
            diagnostics.push(
                Diagnostic::of(&core_codes::INVALID_VALUE)
                    .severity(Severity::Info)
                    .at(at),
            );
        }
        codes.push(code);
    }
    let languages = Languages {
        codes,
        names: LanguageNames::default(),
    };
    Ok((languages, diagnostics))
}

/// Reads `texts/language_names.json`: the `[code, name]` pairs in order.
///
/// An element that is no array is skipped, and an array of another size than two gives its
/// first two elements (a missing one reads as empty); both with a warning that carries
/// vanilla's assert text (L10). The two elements are read as vanilla reads any value as a
/// string.
// Not an array → skipped;
// size ≠ 2 → `[0]` and `[1]` still used.
pub(super) fn read_names(bytes: &[u8]) -> Result<(LanguageNames, Diagnostics), Box<Diagnostic>> {
    let root = root(bytes)?;
    let mut diagnostics = Diagnostics::default();
    let mut entries = Vec::new();
    for (at, element) in elements(&root, &mut diagnostics) {
        // `vanilla = "handleAssert only"`.
        let Some(pair) = element.as_array() else {
            diagnostics.push(Diagnostic::of(&codes::NAME_ENTRY_SHAPE_INVALID_TYPE).at(at));
            continue;
        };
        // Ours: an element that is no string is read as vanilla reads it (a bool
        // as its text, anything else as the empty text) and told at the element, since the
        // file is written with the text read.
        for (index, part) in pair.as_slice().iter().take(2).enumerate() {
            if !part.is_string() {
                let at = at.index(index);
                diagnostics.push(
                    Diagnostic::of(&core_codes::INVALID_TYPE)
                        .severity(Severity::Info)
                        .at(at),
                );
            }
        }
        if pair.len() != 2 {
            diagnostics.push(Diagnostic::of(&codes::NAME_ENTRY_SHAPE_INVALID_VALUE).at(at));
        }
        entries.push((
            Language::new(as_string(pair.first())),
            as_string(pair.get(1)),
        ));
    }
    Ok((LanguageNames { entries }, diagnostics))
}

/// The codes as the JSON array `languages.json` holds.
pub(super) fn codes_json(languages: &Languages) -> Vec<String> {
    languages
        .codes
        .iter()
        .map(|code| code.as_str().to_owned())
        .collect()
}

/// The pairs as the JSON array of arrays `language_names.json` holds.
pub(super) fn names_json(names: &LanguageNames) -> Vec<Vec<String>> {
    names
        .entries
        .iter()
        .map(|(code, name)| vec![code.as_str().to_owned(), name.clone()])
        .collect()
}
#[cfg(test)]
mod gate_tests {
    use super::*;
    use bedrock_addon_core::diagnostic::DiagnosticCode;

    /// The finding of a copy that is no JSON, as `(code, span)`.
    fn refusal(bytes: &[u8]) -> (DiagnosticCode, Option<bedrock_addon_core::diagnostic::Span>) {
        let finding = root(bytes).unwrap_err();
        (finding.code.clone(), finding.location.span)
    }

    #[test]
    fn a_byte_order_mark_changes_neither_the_root_nor_the_refusal() {
        assert_eq!(
            root(b"\xEF\xBB\xBF[\"en_US\"]").unwrap(),
            root(b"[\"en_US\"]").unwrap()
        );
        assert_eq!(refusal(b"\xEF\xBB\xBF{"), refusal(b"{"));
        assert_eq!(refusal(b"\xEF\xBB\xBF\"\xff\""), refusal(b"\"\xff\""));
        let deep = [b'['; 4096];
        assert_eq!(
            refusal(&deep),
            refusal(b"\"\xff\""),
            "one text for both failures"
        );
    }
}
