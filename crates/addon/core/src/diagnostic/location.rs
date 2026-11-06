//! Where a finding is: [`Location`] and the byte [`Span`] in the file.

use super::JsonPath;
use crate::pack_identity::{PackPath, PackRef};

/// Where a finding is.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct Location {
    /// The pack, when known.
    pub pack: Option<PackRef>,
    /// The file, when known.
    pub path: Option<PackPath>,
    /// Breadcrumb inside the file: `minecraft:block/components/minecraft:friction`.
    pub json_path: JsonPath,
    /// Byte range in the file, when the reader knows it (typed findings yes, `Value` findings no).
    pub span: Option<Span>,
}

impl Location {
    /// A location from its four parts: the pack, the file in it, the JSON path in the file and
    /// the byte range, each as far as it is known.
    ///
    /// The struct is `#[non_exhaustive]`, so this is the plain constructor outside the crate;
    /// [`Location::default`] is the location with none of the parts.
    pub fn new(
        pack: Option<PackRef>,
        path: Option<PackPath>,
        json_path: JsonPath,
        span: Option<Span>,
    ) -> Self {
        Self {
            pack,
            path,
            json_path,
            span,
        }
    }
}

/// A byte range in a file.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Span {
    /// Byte offset of the start.
    pub offset: u32,
    /// Length in bytes.
    pub len: u32,
}

impl Span {
    /// The one-based line and column of the span's start in `text`, the file as written.
    ///
    /// Lines end at `\n`, so LF and CRLF text give the same line numbers; columns count
    /// characters, not bytes. An offset past the end, or inside a character, counts up to the
    /// last whole character before it. A byte-order mark at the start of `text` is no
    /// column: the first character after it is column 1, as an editor shows it (the span's
    /// offset does count the mark's three bytes).
    pub fn line_col(&self, text: &str) -> (u32, u32) {
        let mut end = usize::try_from(self.offset).map_or(text.len(), |o| o.min(text.len()));
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        let before = &text[..end];
        let line_start = before.rfind('\n').map_or(0, |i| i + 1);
        let line = before.bytes().filter(|&b| b == b'\n').count() + 1;
        let on_line = &before[line_start..];
        let on_line = if line_start == 0 {
            on_line.strip_prefix('\u{feff}').unwrap_or(on_line)
        } else {
            on_line
        };
        let column = on_line.chars().count() + 1;
        (saturate(line), saturate(column))
    }
}

fn saturate(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}
