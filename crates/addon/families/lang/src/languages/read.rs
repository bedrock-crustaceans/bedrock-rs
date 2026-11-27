//! `texts/languages.json` and `texts/language_names.json`: the language list, the display
//! names and the locale fallback.

use super::{Language, LanguageNames, Languages};
use crate::codes;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, JsonPath, Severity};
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::json::{Prepared, Value};
use bedrock_addon_core::schema::{self, ElementFailure, Schema, read_document, read_iterated};
use bedrock_addon_core::versioning::{MemberFailure, Reading};

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
    .map_err(|error| malformed(error.kind.to_string()))
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

/// The codes of `texts/languages.json`: the elements vanilla iterates of the root (an array's, an
/// object's member values in key order; a root that is no array is told a wrong type, a warning
/// of ours), each once.
///
/// An element that is no string is skipped with a warning (L9); vanilla turns it into a code
/// (`"true"`, `"false"` or `""`) and appends that. A code that is not of the form `xx_YY` is
/// kept with an info (L8); vanilla checks no form.
#[derive(facet::Facet, Schema)]
#[facet(transparent)]
struct Codes(#[facet(schema::read_with = unique_codes)] Vec<Language>);

// `_findAvailableLanguages`: appended unless an equal string is in the list already.
fn unique_codes(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<Language>> {
    let mut seen: Vec<Language> = Vec::new();
    // A repeated code is read as nothing: `Some(None)`.
    let read_code = |element: &Node<'_>, read: &mut Read<'_>| {
        let code = Language::new(read.string(element)?);
        if seen.contains(&code) {
            return Some(None);
        }
        // `vanilla = "accepted, no check"`.
        if !code.is_valid_code() {
            let mut finding = Diagnostic::of(&core_codes::INVALID_VALUE)
                .severity(Severity::Info)
                .at(element.path().clone());
            finding.location.span = element.span();
            read.push(finding);
        }
        seen.push(code.clone());
        Some(Some(code))
    };
    let read_all = read_iterated(node, read, ElementFailure::Drop, read_code)?;
    Some(read_all.into_iter().flatten().collect())
}

/// Reads `texts/languages.json`: the codes in order, each once.
pub(super) fn read_languages(bytes: &[u8]) -> Result<(Languages, Diagnostics), Box<Diagnostic>> {
    let root = root(bytes)?;
    let mut diagnostics = Diagnostics::default();
    let reading = Reading::OWN
        .strict()
        .with_on_member_failure(MemberFailure::DefaultAndTell);
    let codes = {
        let mut read = Read::new(reading, &mut diagnostics);
        read_document::<Codes>(&Node::new(&root, JsonPath::new()), &mut read)
    };
    let languages = Languages {
        codes: codes.map_or_default(|codes| codes.0),
        names: LanguageNames::default(),
    };
    Ok((languages, diagnostics))
}

/// The `[code, name]` pairs of `texts/language_names.json`, in order.
///
/// An element that is no array is skipped, and an array of another size than two gives its
/// first two elements (a missing one reads as empty); both with a warning that carries
/// vanilla's assert text (L10). The two elements are read as vanilla reads any value as a
/// string.
#[derive(facet::Facet, Schema)]
#[facet(transparent)]
struct Pairs(#[facet(schema::read_with = pairs)] Vec<(Language, String)>);

// Not an array → skipped; size ≠ 2 → `[0]` and `[1]` still used.
fn pairs(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<(Language, String)>> {
    read_iterated(node, read, ElementFailure::Drop, |element, read| {
        let at = element.path().clone();
        // `vanilla = "handleAssert only"`.
        let Some(pair) = element.value().as_array() else {
            read.push(Diagnostic::of(&codes::NAME_ENTRY_SHAPE_INVALID_TYPE).at(at));
            return None;
        };
        // Ours: an element that is no string is read as vanilla reads it (a bool
        // as its text, anything else as the empty text) and told at the element, since the
        // file is written with the text read.
        for (index, part) in pair.as_slice().iter().take(2).enumerate() {
            if !part.is_string() {
                read.push(
                    Diagnostic::of(&core_codes::INVALID_TYPE)
                        .severity(Severity::Info)
                        .at(at.index(index)),
                );
            }
        }
        if pair.len() != 2 {
            read.push(Diagnostic::of(&codes::NAME_ENTRY_SHAPE_INVALID_VALUE).at(at));
        }
        Some((
            Language::new(as_string(pair.first())),
            as_string(pair.get(1)),
        ))
    })
}

/// Reads `texts/language_names.json`: the `[code, name]` pairs in order.
pub(super) fn read_names(bytes: &[u8]) -> Result<(LanguageNames, Diagnostics), Box<Diagnostic>> {
    let root = root(bytes)?;
    let mut diagnostics = Diagnostics::default();
    let reading = Reading::OWN
        .strict()
        .with_on_member_failure(MemberFailure::DefaultAndTell);
    let pairs = {
        let mut read = Read::new(reading, &mut diagnostics);
        read_document::<Pairs>(&Node::new(&root, JsonPath::new()), &mut read)
    };
    let entries = pairs.map_or_default(|pairs| pairs.0);
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
    use bedrock_addon_core::diagnostic::Code;

    /// The finding of a copy that is no JSON, as `(code, span)`.
    fn refusal(bytes: &[u8]) -> (Code, Option<bedrock_addon_core::diagnostic::Span>) {
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
