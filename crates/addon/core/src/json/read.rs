//! The text entry points over [`parse`](super::parse::parse): a text as a [`Value`], with
//! comments ([`from_str_jsonc`]) or as plain JSON ([`from_str`]); where content follows the
//! root value ([`trailing_content`]); and what the Minecraft Document reader refuses of a text
//! a `JsonCpp` reader takes ([`document_json_refusal`]).

use super::Value;
use super::lex::{Kind, Lexer, Rules, scan};
use super::parse::{Fault, Options, parse};
use super::slice::JsonSlice;
use crate::diagnostic::codes as core_codes;
use crate::diagnostic::{Diagnostic, DiagnosticDetail, JsonPath, Location, Phase, Severity, Span};

/// `text` as JSON with comments: the value, or the first syntax fault. What every pack file is
/// read as.
///
/// # Errors
///
/// The [`Fault`] of the first syntax error of the text.
pub fn from_str_jsonc(text: &str) -> Result<Value, Fault> {
    parse(text, Options::default()).map(|parsed| parsed.value)
}

/// `text` as plain JSON: a comment is a byte that starts no token.
///
/// # Errors
///
/// The [`Fault`] of the first syntax error of the text.
pub fn from_str(text: &str) -> Result<Value, Fault> {
    parse(text, Options::default().without_comments()).map(|parsed| parsed.value)
}

/// Where the content after the root value of `text` lies (JSON with comments): from its first
/// byte to the end of the text, trailing white space left out. Vanilla's reader stops at the
/// end of the root value and never looks at it, so the
/// file loads; `None` when nothing follows, and when the text is not one facet is given.
pub fn trailing_content(text: &str) -> Option<Span> {
    let start = scan(text.as_bytes(), true).ok()?.trailing?;
    let len = text.get(start..)?.trim_end().len();
    Some(Span {
        offset: u32::try_from(start).ok()?,
        len: u32::try_from(len).ok()?,
    })
}

/// What `RapidJSON` refuses of `document` that a `JsonCpp` reader takes, for the Minecraft
/// Document reader: it parses iteratively, with comments and trailing commas allowed and
/// without stopping at the end of the root value, and on a parse error logs "Failed to parse
/// the Minecraft Document JSON: {}", an error in both of the game's log masks; the document
/// is not loaded. The two errors: a number no double holds (`RapidJSON`'s "number too big"
/// error) and content after the root (its "root not singular" error). The first in the text is
/// told, as `RapidJSON` stops there: a rejecting `MalformedInput` error with the game's text, on
/// the number or the content, at file offsets. `None` when the text has neither, and when facet
/// does not read it (its syntax error is the reader's to tell).
pub fn document_json_refusal(document: JsonSlice<'_>) -> Option<Diagnostic> {
    let number = super::first_beyond_double(document.text);
    let trailing = trailing_content(document.text);
    let number_first = match (&number, trailing) {
        (Some(number), Some(tail)) => number.start < tail.offset as usize,
        (number, _) => number.is_some(),
    };
    let (message, start, len) = match (number, trailing) {
        (Some(number), _) if number_first => (DOCUMENT_NUMBER_TOO_BIG, number.start, number.len()),
        (_, Some(tail)) => (
            DOCUMENT_ROOT_NOT_SINGULAR,
            tail.offset as usize,
            tail.len as usize,
        ),
        _ => return None,
    };
    let offset = document.offset.saturating_add(start);
    let finding = Diagnostic::new(
        core_codes::DOCUMENT_JSON.with([message]),
        Severity::Error,
        Phase::Parse,
        Location::new(
            None,
            None,
            JsonPath::new(),
            Some(Span {
                offset: u32::try_from(offset).unwrap_or(u32::MAX),
                len: u32::try_from(len).unwrap_or(u32::MAX),
            }),
        ),
    );
    Some(finding.detail(DiagnosticDetail::Syntax(message.to_owned())))
}

/// The numbers of `text` whose positive exponent is past `308` plus the digits of their
/// fraction: where `RapidJSON`'s number reader stops with its "number too big" error before it
/// has a value to compare, a finite one included (`0e999`, zero to a reader that computes it),
/// which [`document_json_refusal`] does not see. Each is the byte range of its token; strings
/// and comments are skipped, and the scan ends at a token the lexer refuses.
pub fn exponent_beyond_double(text: &str) -> Vec<std::ops::Range<usize>> {
    let mut lexer = Lexer::new(text.as_bytes(), Rules::strict(true));
    let mut found = Vec::new();
    while let Ok(Some(token)) = lexer.next_token() {
        if token.kind != Kind::Number {
            continue;
        }
        let number = text.get(token.range()).unwrap_or_default();
        let Some((mantissa, exponent)) = number.split_once(['e', 'E']) else {
            continue;
        };
        if exponent.starts_with('-') {
            continue;
        }
        let fraction = mantissa
            .split_once('.')
            .map_or(0, |(_, digits)| digits.len());
        let exponent = exponent
            .trim_start_matches('+')
            .bytes()
            .fold(0_usize, |n, digit| {
                n.saturating_mul(10)
                    .saturating_add(usize::from(digit - b'0'))
            });
        if exponent > 308 + fraction {
            found.push(token.range());
        }
    }
    found
}

/// `RapidJSON`'s text for a number too big for a double.
pub const DOCUMENT_NUMBER_TOO_BIG: &str = "Number too big to be stored in double.";
/// `RapidJSON`'s text for content after the document's root value.
pub const DOCUMENT_ROOT_NOT_SINGULAR: &str =
    "The document root must not be followed by other values.";

/// The message of [`trailing_content_finding`].
pub const TRAILING_CONTENT: &str = "content after the JSON value is ignored";

/// The finding for content after the root value of `document`: an information of the
/// crate's own on that content, at its file offset: the `JsonCpp` reader reads the root value,
/// skips the comments after it and returns without looking further, so it uses the file in
/// silence. The Minecraft Document reader refuses such a text instead (`RapidJSON`'s "root not
/// singular" error; the versioned load's own check). One per file: the
/// reader of the file's root raises it, not the readers of its slices.
pub fn trailing_content_finding(document: JsonSlice<'_>) -> Option<Diagnostic> {
    let span = trailing_content(document.text)?;
    let offset = u32::try_from(document.offset).unwrap_or(u32::MAX);
    Some(Diagnostic::new(
        core_codes::SYNTAX.with([TRAILING_CONTENT]),
        Severity::Info,
        Phase::Parse,
        Location::new(
            None,
            None,
            JsonPath::new(),
            Some(Span {
                offset: offset.saturating_add(span.offset),
                len: span.len,
            }),
        ),
    ))
}

/// The offset of the first syntax fault of `text` (JSON with comments), where the reader
/// reports it; `None` when the text is read to the end of its root value. The text is read as
/// its tokens: nothing a target adds (a type, a member written twice) is a fault.
pub(super) fn first_fault(text: &str) -> Option<usize> {
    parse(text, Options::default())
        .err()
        .map(|fault| fault.span.offset as usize)
}

#[cfg(test)]
mod tests {
    use super::exponent_beyond_double;

    /// The token of every number `RapidJSON`'s exponent rule refuses.
    fn refused(text: &str) -> Vec<&str> {
        exponent_beyond_double(text)
            .into_iter()
            .map(|range| &text[range])
            .collect()
    }

    #[test]
    fn an_exponent_past_308_and_the_fraction_s_digits_is_refused() {
        assert_eq!(
            refused("[0e999, 1e308, 1.5e309, 1.25e310, -2E+400, 1e-999, 7]"),
            ["0e999", "-2E+400"]
        );
    }

    #[test]
    fn strings_and_comments_hold_no_number() {
        let text = "{ \"a\": \"0e999 \\\" 1e999\", // 1e999\n /* 2e999 */ \"b\": 3e999 }";
        assert_eq!(refused(text), ["3e999"]);
    }
}
