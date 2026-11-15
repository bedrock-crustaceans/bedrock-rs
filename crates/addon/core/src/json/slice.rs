//! A slice of a document's text with its file offset: [`JsonSlice`].

/// A slice of a document's text with its file offset, so a nested parse reports file offsets.
/// Internal: the versioning machinery names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct JsonSlice<'a> {
    /// The text.
    pub text: &'a str,
    /// The byte offset of `text` in the file.
    pub offset: usize,
}

impl<'a> JsonSlice<'a> {
    /// A whole file's text, at offset 0.
    pub fn new(text: &'a str) -> Self {
        Self { text, offset: 0 }
    }

    /// Whether the text, after whitespace and comments, opens an object: the object form of a
    /// component that also reads a bare value.
    pub fn opens_an_object(&self) -> bool {
        self.first_byte() == Some(b'{')
    }

    /// The first byte of the text that is neither white space nor inside a comment: what
    /// kind of value it holds, by a look at its start. White space is `str::trim_start`'s
    /// (Unicode's, wider than JSON's: a peek, not a read); a `/*` comment without an end runs
    /// to the end of the text. `None` for a text of white space and comments only.
    pub fn first_byte(&self) -> Option<u8> {
        let mut rest = self.text;
        loop {
            rest = rest.trim_start();
            if let Some(after) = rest.strip_prefix("//") {
                rest = after.split_once('\n').map_or("", |(_, tail)| tail);
            } else if let Some(after) = rest.strip_prefix("/*") {
                rest = after.split_once("*/").map_or("", |(_, tail)| tail);
            } else {
                return rest.bytes().next();
            }
        }
    }
}
