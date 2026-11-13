//! Test JSON, read through the door: [`json`] as the game reads a pack file (comments allowed),
//! [`strict`] as plain JSON (a comment is an error).

use bedrock_addon_core::json::{Value, from_str, from_str_jsonc};

/// `text` read as a JSON value by the door's jsonc entry point; panics with the text and the
/// error when it is no JSON.
#[track_caller]
pub fn json(text: &str) -> Value {
    from_str_jsonc(text).unwrap_or_else(|e| panic!("{text}: {e}"))
}

/// `text` read as a JSON value by the door's plain entry point, which refuses a comment; panics
/// with the text and the error when it is no plain JSON.
#[track_caller]
pub fn strict(text: &str) -> Value {
    from_str(text).unwrap_or_else(|e| panic!("{text}: {e}"))
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
