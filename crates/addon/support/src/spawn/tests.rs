//! Unit tests of the module's private parts: the texts' arity.

use super::texts;

#[test]
fn every_formatted_text_has_one_placeholder() {
    for template in texts::FORMATTED {
        assert_eq!(template.matches("{}").count(), 1, "{template}");
    }
}
