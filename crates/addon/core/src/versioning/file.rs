//! The write-back sub-trait of a one-file family ([`FileFamily`]) and the policy below a chain's
//! floor ([`BelowFloor`]).

use std::convert::Infallible;

use super::{Family, VersionStruct};
use crate::diagnostic::Diagnostics;
use crate::json::{EncodeError, JsonSlice, Value};
use crate::pack_identity::PackPath;

/// A family whose files are one document each, with no beta struct, written back whole: the
/// write-back sub-trait of [`Family`].
///
/// Its [`close`](Self::close) is the hook the support crate's one-file load closes with; it
/// delegates to [`Family::close`] by default, so a family states its hook once on either trait.
/// A family overriding both gets this one from `support::chain_file`, which loads through the
/// unclosed stages (`parse_as_written`, `finish`) and closes itself: a file is closed once,
/// never by both.
pub trait FileFamily: Family<Beta = Infallible, Latest: VersionStruct + Clone> {
    /// Rewrites the generic findings of a parse into the family's vanilla texts and, when the
    /// file `failed`, adds the family's closing line. `document` is the text, when it is one.
    /// Default [`Family::close`].
    fn close(
        diagnostics: &mut Diagnostics,
        document: Option<JsonSlice<'_>>,
        path: &PackPath,
        failed: bool,
    ) {
        <Self as Family>::close(diagnostics, document, path, failed);
    }

    /// The whole document of `value`, `format_version` included.
    ///
    /// # Errors
    ///
    /// `EncodeError::NotRepresentable` when `value` holds something JSON cannot spell, such as a
    /// non-finite number.
    fn document(value: &Self::Latest) -> Result<Value, EncodeError>;
}

/// What a document below the chain's floor is told (`Family::BELOW_FLOOR`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum BelowFloor {
    /// The generic floor line: "To use … use json format version … or higher"
    /// (`VanillaText::UseFormatVersion`). The default.
    #[default]
    UseFormatVersion,
    /// The game's versioned loader has no slice below its first one: "no parser available"
    /// (`VanillaText::NoParserForVersion`), the version rendered as the text above the engine
    /// renders it (`versioning::no_parser_version`: the full triple, a pre-release as
    /// written).
    NoParser,
}
