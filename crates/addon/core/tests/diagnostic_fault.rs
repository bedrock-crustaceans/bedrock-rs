//! The code of a syntax fault (`Fault::code`): a fault of the parser is `MalformedInput` with
//! the fault's words, an end of input in a blank text `EmptyDocument`. (The rows of the parser
//! errors of the facet-json error-kind table this replaced; the kinds only facet-json's typed
//! read raised went with it.)

use bedrock_addon_core::diagnostic::{Code, codes};
use bedrock_addon_core::json::parse::{FaultKind, Options, parse, parse_bytes};

/// The fault of `input`, and its code.
fn fault(input: &str) -> (FaultKind, Code) {
    let fault = parse(input, Options::default()).expect_err("input must fail");
    let code = fault.code(input.trim().is_empty());
    (fault.kind, code)
}

fn is_malformed_input(code: &Code) -> bool {
    code.args_of(&codes::SYNTAX)
        .is_some_and(|message| !message[0].is_empty())
}

// ---- row 1: parser errors → MalformedInput (the fault's words) ----

#[test]
fn parser_unexpected_char_is_malformed_input() {
    let (kind, code) = fault("{\"value\": @}");
    assert!(matches!(kind, FaultKind::UnexpectedChar { .. }), "{kind:?}");
    assert!(is_malformed_input(&code), "{code:?}");
}

#[test]
fn parser_unexpected_eof_after_content_is_malformed_input() {
    let (kind, code) = fault("{\"value\": 1");
    assert!(matches!(kind, FaultKind::UnexpectedEof { .. }), "{kind:?}");
    assert!(is_malformed_input(&code), "{code:?}");
}

#[test]
fn parser_invalid_utf8_is_malformed_input() {
    let input = b"{\"value\": \"\xff\"}";
    let fault = parse_bytes(input, Options::default()).expect_err("invalid UTF-8");
    assert!(
        matches!(fault.kind, FaultKind::InvalidUtf8),
        "{:?}",
        fault.kind
    );
    assert!(is_malformed_input(&fault.code(false)), "{fault:?}");
}

#[test]
fn parser_unexpected_token_without_path_is_malformed_input() {
    let (kind, code) = fault("{\"value\" 1}");
    assert!(
        matches!(kind, FaultKind::UnexpectedToken { .. }),
        "{kind:?}"
    );
    assert!(is_malformed_input(&code), "{code:?}");
}

// ---- row 2: UnexpectedEof with nothing before it → EmptyDocument ----

#[test]
fn empty_file_is_empty_document() {
    let (kind, code) = fault("");
    assert!(matches!(kind, FaultKind::UnexpectedEof { .. }), "{kind:?}");
    assert!(code.is(&codes::EMPTY_DOCUMENT), "{code:?}");
}

#[test]
fn whitespace_only_file_is_empty_document() {
    let (kind, code) = fault(" \r\n\t ");
    assert!(matches!(kind, FaultKind::UnexpectedEof { .. }), "{kind:?}");
    assert!(code.is(&codes::EMPTY_DOCUMENT), "{code:?}");
}

/// The message `MalformedInput` carries is the fault's own words.
#[test]
fn malformed_input_carries_facets_message() {
    let (kind, code) = fault("{\"value\": @}");
    assert_eq!(code, codes::SYNTAX.with([kind.to_string()]));
}
