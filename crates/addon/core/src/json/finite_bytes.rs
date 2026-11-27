//! A file's bytes with the numbers no double holds clamped, for a family whose own reader
//! keeps what it does not read: the bytes its reader is given, and the end of the read.

use std::borrow::Cow;

use super::{ClampPlan, ClampedText, JsonSlice, clamp_tokens, strip_bom};
use crate::diagnostic::{Diagnostics, JsonPath};
use crate::document::Origin;

/// The bytes of a file as a family's reader is to read them: as written, or, when a number
/// of the file is one no double holds (`1e999`, an integer of 400 digits), with each such
/// number replaced by the largest finite double of its sign ([`clamp_tokens`] with
/// [`ClampPlan::doubles`]).
///
/// An infinity kept for write-back (in an unknown member, in a member held as JSON) has no
/// JSON spelling: the document would load and could not be written. Read through this, the
/// document holds and writes the clamped number, and the read ends with an `InvalidValue`
/// warning at each number, on its token as written. A file without such a number costs one
/// walk of its tokens and is read as it is.
#[derive(Debug)]
pub struct FiniteBytes<'a> {
    bytes: Cow<'a, [u8]>,
    clamp: Option<Clamp>,
}

/// What a clamp did.
#[derive(Debug)]
struct Clamp {
    text: ClampedText,
    /// The warnings of the clamp, at the numbers as written.
    beyond: Diagnostics,
    /// Where the text starts in the file: after a byte-order mark.
    offset: usize,
}

impl<'a> FiniteBytes<'a> {
    /// The bytes of a file, clamped where they need it. A byte-order mark stays in front
    /// (the reader takes it with `json::blank_bom`, so that its spans count the mark as the
    /// clamp's do); a file that is no UTF-8 text or no JSON is left for the reader to
    /// refuse.
    #[must_use]
    pub fn of(bytes: &'a [u8]) -> Self {
        let stripped = strip_bom(bytes);
        let offset = bytes.len() - stripped.len();
        let mut beyond = Diagnostics::default();
        let clamped = std::str::from_utf8(stripped).ok().and_then(|text| {
            clamp_tokens(
                JsonSlice { text, offset },
                &ClampPlan::doubles(),
                &JsonPath::new(),
                &mut beyond,
            )
        });
        match clamped {
            Some(text) => {
                let mut read = bytes.get(..offset).unwrap_or_default().to_vec();
                read.extend_from_slice(text.text.as_bytes());
                Self {
                    bytes: Cow::Owned(read),
                    clamp: Some(Clamp {
                        text,
                        beyond,
                        offset,
                    }),
                }
            }
            None => Self {
                bytes: Cow::Borrowed(bytes),
                clamp: None,
            },
        }
    }

    /// The bytes to read.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Whether a number was replaced.
    #[must_use]
    pub fn is_clamped(&self) -> bool {
        self.clamp.is_some()
    }

    /// Ends the read: the findings `d` of the reading of [`Self::bytes`] get the places of
    /// the file as written (spans count from the start of the file), then come the warnings
    /// of the clamp, located in `origin`'s file. A warning is left out where the reader has
    /// given a finding at the same number: a number is told once (`ClampedText::settle`).
    pub fn settle(self, d: &mut Diagnostics, origin: Option<&Origin>) {
        self.settle_told(d, origin);
    }

    /// [`Self::settle`], giving the warnings of the clamp as it added them to `d`: for a
    /// reader whose members speak of their numbers only later (the merge of a stack's
    /// copies), which then ends with `json::drop_spoken_for`.
    pub fn settle_told(self, d: &mut Diagnostics, origin: Option<&Origin>) -> Diagnostics {
        let Some(clamp) = self.clamp else {
            return Diagnostics::default();
        };
        let mut beyond = Diagnostics::default();
        for mut finding in clamp.beyond {
            if let Some(origin) = origin {
                finding.location.pack = Some(origin.pack.clone());
                finding.location.path = Some(origin.path.clone());
            }
            beyond.push(finding);
        }
        clamp.text.settle(d, clamp.offset, beyond)
    }
}
