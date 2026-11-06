//! What runs over the raw bytes before facet sees them: the BOM and the nesting limit.

use super::lex::{Kind, Lexer, Rules};

/// The deepest nesting of arrays and objects a document may have: facet-json has no limit of its
/// own and a deeper document overflows the parser's stack.
///
/// What a stack must hold for it: nothing that grows with the text. An untyped `Value` is
/// read without a call for each level, and the scans beside it follow a text no deeper than
/// this. A typed target is read by facet's recursion, a call for each level of the type and
/// of the text below an untyped member or a type that holds itself, and a value is written
/// the same way. The readers and writers of `json` run such a walk on the calling thread's
/// stack where enough of it is left for the depth at hand, and on a stack made for the
/// walk, on the same thread, where it is not. So every reader reads a text this deep, and
/// refuses a deeper one with an error, on a thread with a megabyte of stack, in a debug
/// build as in any other.
///
/// Three things are outside that rule:
///
/// - WebAssembly. A module has the one stack it was linked with, and none is made: an
///   embedder that reads typed targets there sizes it for the types it reads (an optimised
///   build takes up to 11 KiB a level; an unoptimised one up to 87).
/// - A target on which `stacker`, the crate the stack is made with, has no way to give a
///   call another stack: the walk then runs where it is, as on WebAssembly, and nothing
///   says so.
/// - A host that cannot map the stack: `stacker` panics where the mapping fails (an
///   address-space limit such as `RLIMIT_AS`, strict overcommit, a full 32-bit address
///   space). The stack for a text at this limit is 65 MiB of address space; its pages are
///   taken as they are touched.
pub const MAX_JSON_DEPTH: usize = 128;

/// `bytes` without one leading UTF-8 byte order mark: vanilla's reader skips it, facet-json
/// rejects it.
pub fn strip_bom(bytes: &[u8]) -> &[u8] {
    // jsoncpp skips a leading UTF-8 byte order mark.
    bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes)
}

/// The bytes of a file as a reader of JSON text is given them: `bytes`, with one leading
/// UTF-8 byte order mark replaced by three spaces. Vanilla's reader skips the mark
/// and facet-json rejects it; as white space it is skipped by every reader, and every offset
/// into the text read is an offset into the file as written, so a finding's span covers in
/// the file the text it is about. A file without a mark is borrowed.
///
/// For a reader whose findings have spans. [`strip_bom`] is for text that is not JSON (white
/// space at its start may mean something) and for a reader that counts the mark itself.
#[must_use]
pub fn blank_bom(bytes: &[u8]) -> std::borrow::Cow<'_, [u8]> {
    match bytes.strip_prefix(b"\xEF\xBB\xBF") {
        Some(rest) => {
            let mut read = Vec::with_capacity(bytes.len());
            read.extend_from_slice(b"   ");
            read.extend_from_slice(rest);
            std::borrow::Cow::Owned(read)
        }
        None => std::borrow::Cow::Borrowed(bytes),
    }
}

/// Checks that no array or object in `bytes` is nested deeper than `max` levels, without
/// parsing: a bracket counter that skips strings and comments. Brackets that do not balance are
/// left for the parser to report.
///
/// The count ends where the root value does: vanilla's reader stops there, so what follows a
/// root value is not the document's nesting (the file loads). A root that is no array
/// or object has none.
///
/// # Errors
///
/// Returns `TooDeep` with the offset of the first bracket nested past `max`.
pub fn check_depth(bytes: &[u8], max: usize) -> Result<(), TooDeep> {
    // Read loosely: a string's escapes, an unended comment and what is no JSON are the
    // parser's to report, once the nesting is known to be within the limit.
    let mut lexer = Lexer::new(bytes, Rules::lenient(false));
    let mut depth = 0usize;
    while let Ok(Some(token)) = lexer.next_token() {
        match token.kind {
            Kind::OpenArray | Kind::OpenObject => {
                depth += 1;
                if depth > max {
                    return Err(TooDeep {
                        offset: token.start,
                    });
                }
            }
            Kind::CloseArray | Kind::CloseObject => {
                // The bracket that closes the root, or one that closes nothing: either way
                // no root value goes on from here.
                if depth <= 1 {
                    return Ok(());
                }
                depth -= 1;
            }
            // A root that is a string, a number or a literal, or no JSON at all.
            _ if depth == 0 => return Ok(()),
            _ => {}
        }
    }
    Ok(())
}

/// A document nests arrays and objects deeper than the limit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("JSON nesting deeper than the limit at byte {offset}")]
pub struct TooDeep {
    /// The byte offset of the first bracket past the limit.
    pub offset: usize,
}
