//! Test JSON, read by the parser: [`json`] as the game reads a pack file (comments allowed),
//! [`strict`] as plain JSON (a comment is an error).

use bedrock_addon_core::json::parse::{Options, parse};
use bedrock_addon_core::json::{Value, from_str_jsonc};

/// `text` read as a JSON value by `from_str_jsonc`; panics with the text and the
/// error when it is no JSON.
#[track_caller]
pub fn json(text: &str) -> Value {
    from_str_jsonc(text).unwrap_or_else(|e| panic!("{text}: {e}"))
}

/// `text` read as plain JSON, which refuses a comment; panics with the text and the fault when
/// it is no plain JSON.
#[track_caller]
pub fn strict(text: &str) -> Value {
    match parse(text, Options::default().without_comments()) {
        Ok(parsed) => parsed.value,
        Err(fault) => panic!("{text}: {fault}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::catch_unwind;

    #[test]
    fn strict_refuses_a_comment_and_json_reads_it() {
        let text = "{\n  // a note\n  \"a\": 1\n}";
        assert!(
            catch_unwind(|| strict(text)).is_err(),
            "strict read a text with a comment"
        );
        assert_eq!(
            json(text),
            json(r#"{"a": 1}"#),
            "json reads past the comment"
        );
    }
}
