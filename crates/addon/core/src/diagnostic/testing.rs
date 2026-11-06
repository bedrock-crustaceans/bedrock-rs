//! Checks for the tests of a family that moves a raise site from a legacy [`DiagnosticCode`] to a
//! [`CodeDef`].

use std::fmt::Write as _;

use super::def::{Code, kind_of};
use super::{CodeDef, Diagnostic, DiagnosticCode};

/// Asserts that `def` replaces `legacy` as [`CodeDef::was`] promises: it has the legacy
/// variant's [`name`](DiagnosticCode::name) and [`kind`](super::Kind).
///
/// # Panics
///
/// When the name or the kind differs.
#[allow(
    clippy::needless_pass_by_value,
    reason = "a test passes the legacy code as a literal"
)]
pub fn assert_replaces<const N: usize>(def: &CodeDef<N>, legacy: DiagnosticCode) {
    assert_eq!(
        def.name(),
        legacy.name(),
        "{} replaces {} under its name",
        def.id(),
        legacy.name()
    );
    assert_eq!(
        def.kind(),
        kind_of(&legacy),
        "{} keeps the kind of {}",
        def.id(),
        legacy.name()
    );
}

/// The `Debug` text of the legacy code a finding's specific code replaces: what a
/// characterization pin written before the raise site moved holds (the Debug-keyed `.txt` pins,
/// the literal strings of the kit's reader tests), so the pin renders the same line whichever of
/// the two the site raises. A legacy code renders as its own `Debug`.
///
/// The legacy code is the one the definition [`was`](CodeDef::was): a variant without fields
/// renders its name; a `text` field renders `None` for a definition without the game's sentence,
/// `Some(Fixed(..))` for one with a sentence and no argument (or the sentence given whole, a
/// `_WORDED` one), `Some(Formatted { .. })` for one with arguments; `MalformedInput`'s
/// `message` is the finding's [`syntax_message`](Diagnostic::syntax_message); the few lines of
/// a structured `VanillaText` render by their id. Goes with the
/// legacy enum: the pins are then re-rendered by id.
///
/// # Panics
///
/// For a specific code this does not know how to render: a definition of a structured text or
/// a typed payload not in its tables. A pin that meets one adds it here.
// hidden: a bridge for the pins of the migration, deleted with the legacy enum
#[doc(hidden)]
#[must_use]
pub fn legacy_debug(finding: &Diagnostic) -> String {
    let DiagnosticCode::Specific { code } = &finding.code else {
        return format!("{:?}", finding.code);
    };
    let name = code.name();
    let fields = DiagnosticCode::FIELDS
        .iter()
        .find_map(|(variant, fields)| (*variant == name).then_some(*fields))
        .unwrap_or_else(|| panic!("{} replaces no legacy variant", code.id()));
    let mut out = name.to_owned();
    match fields {
        [] => {}
        ["text"] => {
            let _ = write!(out, " {{ text: {} }}", text_debug(code));
        }
        ["message", "text"] => {
            let message = finding.syntax_message().unwrap_or_default();
            let _ = write!(
                out,
                " {{ message: {message:?}, text: {} }}",
                text_debug(code)
            );
        }
        fields => panic!(
            "{}: a payload of {fields:?} the bridge does not render",
            code.id()
        ),
    }
    out
}

/// The `Debug` of the legacy `text` field (an `Option<VanillaText>`) a definition without
/// arguments stands for: what a reading table pinned before a family's `MissingVersion` held a
/// definition (`None`, `Some(NoValidVersion)`, `Some(Fixed(".."))`).
// hidden: a bridge for the pins of the migration, deleted with the legacy enum
#[doc(hidden)]
#[must_use]
pub fn legacy_text_debug(def: &'static CodeDef<0>) -> String {
    text_debug(&Code::from(def))
}

/// The `Debug` of the legacy `text` field a specific code stands for.
fn text_debug(code: &Code) -> String {
    let args = code.args();
    if let Some(structured) = structured_debug(code.id(), args) {
        return format!("Some({structured})");
    }
    match code.vanilla_template() {
        None => "None".to_owned(),
        Some("{}") => format!("Some(Fixed({:?}))", args[0]),
        Some(template) if args.is_empty() => format!("Some(Fixed({template:?}))"),
        Some(template) => format!("Some(Formatted {{ template: {template:?}, args: {args:?} }})"),
    }
}

/// The `Debug` of a structured `VanillaText` a definition renders, by the definition's id.
fn structured_debug(id: &str, args: &[String]) -> Option<String> {
    Some(match id {
        "core/document_json" => format!("DocumentJson {{ message: {:?} }}", args[0]),
        "core/no_valid_version" => "NoValidVersion".to_owned(),
        "core/no_format_version_field" => "NoFormatVersionField".to_owned(),
        "core/no_parser_for_version" => format!("NoParserForVersion {{ version: {:?} }}", args[0]),
        "core/document_member_count" => "DocumentMemberCount".to_owned(),
        "core/member_deprecated" => format!("MemberDeprecated {{ member: {:?} }}", args[0]),
        "core/filter_test_not_found" => format!("FilterTestNotFound {{ name: {:?} }}", args[0]),
        // `NumericRange::code` gave the whole sentence as the one argument of `{}`.
        "core/outside_accepted_range" => format!(
            "Formatted {{ template: \"{{}}\", args: [{:?}] }}",
            format!("the input is outside of the accepted range {}", args[0])
        ),
        "core/number_bound" => format!(
            "Formatted {{ template: \"{{}}\", args: [{:?}] }}",
            format!("the input must be a number {}", args[0])
        ),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::{Class, DiagnosticDetail, Kind, VanillaText, builders, codes};

    /// A definition with the game's sentence and two arguments, as a family declares one.
    static WORDED_TWICE: CodeDef<2> = CodeDef::new(
        "core/test_worded_twice",
        Kind::InvalidValue,
        Class::Error,
        "{} and {}",
    )
    .game("between {} and {}")
    .was("InvalidValue");

    fn finding(code: impl Into<DiagnosticCode>) -> Diagnostic {
        Diagnostic::error(code.into())
    }

    /// A cereal range finding renders as the legacy code `NumericRange::code` gave: the whole
    /// sentence as the one argument of `{}`.
    #[test]
    fn a_range_definition_renders_as_the_whole_sentence_it_was() {
        let code = super::super::cereal::NumericRange::inclusive(0.0, 1.0).code();
        assert_eq!(
            legacy_debug(&finding(code)),
            r#"InvalidValue { text: Some(Formatted { template: "{}", args: ["the input is outside of the accepted range [0, 1]"] }) }"#
        );
    }

    /// Each specific code renders as the `Debug` of the legacy code it stands for.
    #[test]
    fn a_specific_code_renders_as_the_legacy_code_it_replaces() {
        let rows: Vec<(Diagnostic, DiagnosticCode)> = vec![
            (
                finding(&codes::INVALID_VALUE),
                DiagnosticCode::InvalidValue { text: None },
            ),
            (
                finding(codes::INVALID_VALUE_WORDED.with(["the game's line"])),
                DiagnosticCode::InvalidValue {
                    text: Some(VanillaText::Fixed("the game's line")),
                },
            ),
            (
                finding(WORDED_TWICE.with(["1", "2"])),
                DiagnosticCode::InvalidValue {
                    text: Some(VanillaText::Formatted {
                        template: "between {} and {}",
                        args: vec!["1".to_owned(), "2".to_owned()],
                    }),
                },
            ),
            (
                finding(&codes::LOSSY_CONVERSION),
                DiagnosticCode::LossyConversion,
            ),
            (
                finding(codes::SYNTAX.with(["expected `,`"])),
                DiagnosticCode::MalformedInput {
                    message: "expected `,`".to_owned(),
                    text: None,
                },
            ),
            (
                finding(codes::MALFORMED_INPUT_WORDED.with(["the game's line"]))
                    .detail(DiagnosticDetail::Syntax("expected `,`".to_owned())),
                DiagnosticCode::MalformedInput {
                    message: "expected `,`".to_owned(),
                    text: Some(VanillaText::Fixed("the game's line")),
                },
            ),
        ];
        for (specific, legacy) in rows {
            assert_eq!(legacy_debug(&specific), format!("{legacy:?}"));
            assert_eq!(
                legacy_debug(&finding(legacy.clone())),
                format!("{legacy:?}")
            );
        }
    }

    /// The reader's message of a syntax finding, wherever it is held.
    #[test]
    fn the_syntax_message_is_read_from_the_legacy_code_the_detail_or_the_line() {
        let legacy = finding(DiagnosticCode::MalformedInput {
            message: "m".to_owned(),
            text: None,
        });
        assert_eq!(legacy.syntax_message(), Some("m"));
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

    /// `syntax` renders as the legacy code without a text renders, an empty message included.
    #[test]
    fn the_syntax_code_renders_the_legacy_line() {
        for message in ["", "expected `,`"] {
            let legacy = DiagnosticCode::MalformedInput {
                message: message.to_owned(),
                text: None,
            };
            let code = DiagnosticCode::from(builders::syntax(message));
            assert_eq!(code.to_string(), legacy.to_string());
            assert_eq!(code.name(), legacy.name());
        }
    }

    /// A code's game sentence, of a legacy code and of a specific one.
    #[test]
    fn a_code_s_game_sentence_is_its_text_or_its_definition_s() {
        assert_eq!(DiagnosticCode::InvalidValue { text: None }.vanilla(), None);
        assert_eq!(
            DiagnosticCode::InvalidValue {
                text: Some(VanillaText::Fixed("line"))
            }
            .vanilla()
            .as_deref(),
            Some("line")
        );
        assert_eq!(DiagnosticCode::from(&codes::INVALID_VALUE).vanilla(), None);
        assert_eq!(
            DiagnosticCode::from(WORDED_TWICE.with(["1", "2"]))
                .vanilla()
                .as_deref(),
            Some("between 1 and 2")
        );
    }
}
