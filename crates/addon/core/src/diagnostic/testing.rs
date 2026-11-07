//! Helpers for the tests of a family: how a pin shows a finding, and the checks of a site that
//! holds the game's sentence.

use std::fmt::Write as _;

use super::def::Code;
use super::{CodeDef, Diagnostic, DiagnosticDetail};

/// A finding's code as a pin holds it: the definition's id, then its arguments when it has
/// any, then the reader's message when the finding carries one as its detail.
///
/// ```
/// use bedrock_addon_core::diagnostic::{Diagnostic, codes, testing::shown};
///
/// let finding = Diagnostic::error(codes::SYNTAX.with(["expected `,`"]));
/// assert_eq!(shown(&finding), r#"core/syntax ["expected `,`"]"#);
/// ```
#[must_use]
pub fn shown(finding: &Diagnostic) -> String {
    let code = &finding.code;
    let mut out = code.id().to_owned();
    if !code.args().is_empty() {
        let _ = write!(out, " {:?}", code.args());
    }
    if let Some(DiagnosticDetail::Syntax(message)) = finding.detail.as_deref() {
        let _ = write!(out, " syntax: {message:?}");
    }
    out
}

/// The `Debug` of the legacy `text` field (an `Option<VanillaText>`) a definition without
/// arguments stands for: what a reading table pinned before a family's `MissingVersion` held a
/// definition (`None`, `Some(NoValidVersion)`, `Some(Fixed(".."))`).
// hidden: a bridge for the reading tables, which still pin the legacy text shape
#[doc(hidden)]
#[must_use]
pub fn legacy_text_debug(def: &'static CodeDef<0>) -> String {
    text_debug(&Code::from(def))
}

/// The `Debug` of the legacy `text` field a definition without arguments stands for.
fn text_debug(code: &Code) -> String {
    if let Some(structured) = structured_debug(code.id()) {
        return format!("Some({structured})");
    }
    match code.vanilla_template() {
        None => "None".to_owned(),
        Some(template) => format!("Some(Fixed({template:?}))"),
    }
}

/// Whether `code` is of the definition `id` and tells `line` as its line and as its game's
/// sentence: what a site that held the game's sentence as a legacy text raises.
// hidden: a bridge for the pins of the migration
#[doc(hidden)]
#[must_use]
pub fn same_line(code: &Code, id: &str, line: &str) -> bool {
    code.id() == id && code.to_string() == line && code.vanilla().as_deref() == Some(line)
}

/// The `Debug` of a structured `VanillaText` a definition without arguments renders, by the
/// definition's id: the three a family's `MissingVersion` holds.
fn structured_debug(id: &str) -> Option<&'static str> {
    Some(match id {
        "core/no_valid_version" => "NoValidVersion",
        "core/no_format_version_field" => "NoFormatVersionField",
        "core/document_member_count" => "DocumentMemberCount",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::{Class, DiagnosticDetail, Kind, builders, codes};

    /// A definition with the game's sentence and two arguments, as a family declares one.
    static WORDED_TWICE: CodeDef<2> = CodeDef::new(
        "core/test_worded_twice",
        Kind::InvalidValue,
        Class::Error,
        "{} and {}",
    )
    .game("between {} and {}");

    fn finding(code: impl Into<Code>) -> Diagnostic {
        Diagnostic::error(code)
    }

    /// A worded definition with the sentence tells its line; another sentence, another
    /// definition or a line that is not the game's does not.
    #[test]
    fn a_worded_definition_tells_the_line_of_the_legacy_text() {
        let id = "core/unresolved_reference_worded";
        let worded = codes::UNRESOLVED_REFERENCE_WORDED.with(["no block a:b"]);
        assert!(same_line(&worded, id, "no block a:b"));
        let other = codes::UNRESOLVED_REFERENCE_WORDED.with(["no block c:d"]);
        assert!(!same_line(&other, id, "no block a:b"));
        let renamed = codes::INVALID_REFERENCE_WORDED.with(["no block a:b"]);
        assert!(!same_line(&renamed, id, "no block a:b"));
        let own = Code::from(&codes::UNRESOLVED_REFERENCE);
        assert!(!same_line(
            &own,
            "core/unresolved_reference",
            &own.to_string()
        ));
    }

    /// A pin shows a finding by its definition's id, its arguments and the reader's message.
    #[test]
    fn a_finding_is_shown_by_its_id_its_arguments_and_its_reader_s_message() {
        assert_eq!(shown(&finding(&codes::INVALID_VALUE)), "core/invalid_value");
        assert_eq!(
            shown(&finding(WORDED_TWICE.with(["1", "2"]))),
            r#"core/test_worded_twice ["1", "2"]"#
        );
        assert_eq!(
            shown(
                &finding(codes::MALFORMED_INPUT_WORDED.with(["line"]))
                    .detail(DiagnosticDetail::Syntax("expected `,`".to_owned()))
            ),
            r#"core/malformed_input_worded ["line"] syntax: "expected `,`""#
        );
    }

    /// The reader's message of a syntax finding, wherever it is held.
    #[test]
    fn the_syntax_message_is_read_from_the_detail_or_the_line() {
        let detailed = finding(codes::MALFORMED_INPUT_WORDED.with(["line"]))
            .detail(DiagnosticDetail::Syntax("m".to_owned()));
        assert_eq!(detailed.syntax_message(), Some("m"));
        assert_eq!(
            finding(codes::SYNTAX.with(["m"])).syntax_message(),
            Some("m")
        );
        assert_eq!(finding(&codes::MALFORMED_INPUT).syntax_message(), Some(""));
        assert_eq!(finding(&codes::INVALID_VALUE).syntax_message(), None);
    }

    /// `syntax` renders the reader's message, an empty one as the plain line.
    #[test]
    fn the_syntax_code_renders_the_readers_message() {
        for (message, line, id) in [
            ("", "malformed input", "core/malformed_input"),
            ("expected `,`", "expected `,`", "core/syntax"),
        ] {
            let code = builders::syntax(message);
            assert_eq!(code.to_string(), line);
            assert_eq!(code.id(), id);
        }
    }

    /// A code's game sentence: its definition's, with its arguments.
    #[test]
    fn a_code_s_game_sentence_is_its_definition_s() {
        assert_eq!(Code::from(&codes::INVALID_VALUE).vanilla(), None);
        assert_eq!(
            codes::INVALID_VALUE_WORDED
                .with(["line"])
                .vanilla()
                .as_deref(),
            Some("line")
        );
        assert_eq!(
            WORDED_TWICE.with(["1", "2"]).vanilla().as_deref(),
            Some("between 1 and 2")
        );
    }
}
