//! The full syntax check of the `script-syntax` feature: the file parsed as an ES module.
// The parser accepts ES2025; the engine's
// QuickJS build does not accept all of it (import attributes), which stays the lexer's rule.

use oxc_parser::Parser;

use super::{ScriptPosition, ScriptSyntax, ScriptSyntaxKind};

/// The place of byte `offset` of `text`.
fn position(text: &str, offset: usize) -> ScriptPosition {
    let mut offset = offset.min(text.len());
    while !text.is_char_boundary(offset) {
        offset -= 1;
    }
    let before = &text[..offset];
    let line_start = before.rfind('\n').map_or(0, |at| at + 1);
    let line = before.bytes().filter(|byte| *byte == b'\n').count() + 1;
    // A byte order mark is no character of the first line.
    let column = before[line_start..]
        .chars()
        .filter(|c| *c != '\u{feff}')
        .count()
        + 1;
    ScriptPosition {
        offset: offset as u64,
        line: u32::try_from(line).unwrap_or(u32::MAX),
        column: u32::try_from(column).unwrap_or(u32::MAX),
    }
}

/// The first error the parser reports for `text` read as a module; `None` for a file that
/// parses.
#[allow(
    clippy::default_trait_access,
    reason = "the types are of crates this one does not depend on by name"
)]
pub(super) fn first_error(text: &str) -> Option<ScriptSyntax> {
    // The allocator and the source type (an ES module, the default) are named by inference:
    // `oxc_parser` is the only parser crate this crate depends on.
    let allocator = Default::default();
    let parsed = Parser::new(&allocator, text, Default::default()).parse();
    let error = parsed.diagnostics.errors().next()?;
    let offset = error.labels.first().map_or(0, |label| {
        usize::try_from(label.offset()).unwrap_or(usize::MAX)
    });
    Some(ScriptSyntax {
        kind: ScriptSyntaxKind::Parser {
            message: error.message.as_ref().into(),
        },
        at: position(text, offset),
    })
}

/// The specifiers of the static imports and re-exports of `text`, sorted, as the parser's
/// module record has them: what the lexer's sites are checked against.
#[cfg(test)]
#[allow(
    clippy::default_trait_access,
    reason = "the types are of crates this one does not depend on by name"
)]
pub(super) fn requested_modules(text: &str) -> Option<Vec<String>> {
    let allocator = Default::default();
    let parsed = Parser::new(&allocator, text, Default::default()).parse();
    if parsed.diagnostics.has_errors() {
        return None;
    }
    let mut names: Vec<String> = parsed
        .module_record
        .requested_modules
        .keys()
        .map(|name| name.as_str().to_owned())
        .collect();
    names.sort();
    Some(names)
}
