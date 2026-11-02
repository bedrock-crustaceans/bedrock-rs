//! Unit tests of the module's private parts: the texts' arity.

use super::texts;

#[test]
fn every_formatted_text_has_a_placeholder_per_argument() {
    for (template, arguments) in texts::FORMATTED {
        assert_eq!(template.matches("{}").count(), *arguments, "{template}");
    }
}

#[test]
fn the_hinted_texts_extend_the_unknown_command_text() {
    for hinted in [texts::UNKNOWN_COMMAND_COMMENT, texts::UNKNOWN_COMMAND_SLASH] {
        assert!(hinted.starts_with(texts::UNKNOWN_COMMAND), "{hinted}");
        assert!(hinted.ends_with(')'), "{hinted}");
    }
}
