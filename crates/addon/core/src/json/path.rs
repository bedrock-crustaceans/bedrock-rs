//! The JSON member path of a token in a text (`path_at`), and what the walks of this crate ask
//! of a `diagnostic::JsonPath` beyond its public methods.

use super::lex::{Kind, Lexer, Rules, Structure, key_text};
use crate::diagnostic::{JsonPath, JsonPathSegment};

/// What the walks of this crate ask of a [`JsonPath`] beyond its public methods.
impl JsonPath {
    /// The key that follows `prefix` in this path, when this path starts with `prefix` and
    /// continues with an object member.
    pub(crate) fn key_after(&self, prefix: &JsonPath) -> Option<&str> {
        if !self.starts_with(prefix) {
            return None;
        }
        match self.segments().nth(prefix.len())? {
            JsonPathSegment::Key(key) => Some(key),
            JsonPathSegment::Index(_) => None,
        }
    }

    /// Whether `self` names an array element of `other`'s place: the keys agree step by step,
    /// the indices may differ, and the steps `self` has beyond `other` are array indices only.
    pub(crate) fn refines(&self, other: &JsonPath) -> bool {
        self.len() >= other.len()
            && self
                .segments()
                .zip(other.segments())
                .all(|pair| match pair {
                    (JsonPathSegment::Key(a), JsonPathSegment::Key(b)) => a == b,
                    (JsonPathSegment::Index(_), JsonPathSegment::Index(_)) => true,
                    _ => false,
                })
            && self
                .segments()
                .skip(other.len())
                .all(|step| matches!(step, JsonPathSegment::Index(_)))
    }
}

/// The members leading to the token at byte `offset` of `text`: the value that starts there, or,
/// for a member key, that member. A lexical scan (strings, comments, brackets), not a parse,
/// by the lenient reading of `lex` (a form feed is white space here); it runs only to place an
/// error facet reported, and stops at the offset.
pub(crate) fn path_at(text: &str, offset: usize) -> JsonPath {
    let mut lexer = Lexer::new(text.as_bytes(), Rules::lenient(true));
    let mut open = Structure::new();
    while let Ok(Some(token)) = lexer.next_token() {
        let at_key = matches!(token.kind, Kind::String { .. }) && open.awaits_key();
        if token.end > offset {
            let mut path = open.path().clone();
            if at_key {
                path.push(&key_text(&text[token.range()]));
            }
            return path;
        }
        match token.kind {
            Kind::OpenObject => open.open_object(),
            Kind::OpenArray => open.open_array(),
            Kind::CloseObject | Kind::CloseArray => {
                open.close();
            }
            Kind::Comma => open.comma(),
            Kind::String { .. } if at_key => open.enter_key(&key_text(&text[token.range()])),
            _ => {}
        }
    }
    open.path().clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(text: &str, needle: &str) -> String {
        path_at(text, text.find(needle).unwrap()).to_string()
    }

    #[test]
    fn a_key_token_names_its_member() {
        let text = r#"{ "a": { "b": 1, "c": 2 } }"#;
        assert_eq!(at(text, r#""c""#), "a/c");
        assert_eq!(at(text, r#""a""#), "a");
    }

    #[test]
    fn a_value_token_names_the_member_holding_it() {
        let text = r#"{ "a": [10, 20, { "b": true }] }"#;
        assert_eq!(at(text, "20"), "a/1");
        assert_eq!(at(text, "true"), "a/2/b");
        assert_eq!(at(text, "{ \"b\""), "a/2");
    }

    #[test]
    fn comments_and_strings_with_brackets_are_skipped() {
        let text = "{ /* { [ */ \"a\": \"]}\", // [\n \"b\": 1 }";
        assert_eq!(at(text, "1"), "b");
    }

    #[test]
    fn an_escaped_key_is_unescaped() {
        let text = r#"{ "a\"b": 1 }"#;
        assert_eq!(at(text, "1"), "a\"b");
    }

    #[test]
    fn an_escaped_slash_in_a_key_names_one_segment() {
        let text = r#"{ "a/b": { "c~d": 1 } }"#;
        assert_eq!(at(text, "1"), "a~1b/c~0d");
    }

    #[test]
    fn key_after_reads_the_next_key_of_a_matching_prefix() {
        let full = JsonPath::of_keys(["m", "k", "v"]);
        let mut prefix = JsonPath::of_keys(["m"]);
        assert_eq!(full.key_after(&prefix), Some("k"));
        prefix.pop();
        prefix.push("x");
        assert_eq!(full.key_after(&prefix), None);
    }
}
