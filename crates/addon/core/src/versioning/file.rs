//! The write-back sub-trait of a one-file family ([`FileFamily`]) and the policy below a chain's
//! floor ([`BelowFloor`]).

use std::convert::Infallible;

use super::{Family, VersionStruct};
use crate::json::{EncodeError, Value};

/// A family whose files are one document each, with no beta struct, written back whole: the
/// write-back sub-trait of [`Family`].
///
/// Its files close with [`Family::close`], run once per file by the closed entry points
/// (`versioning::load_file`, `parse_file_as_written`, `finish_file`) the support crate's
/// one-file load goes through.
pub trait FileFamily: Family<Beta = Infallible, Latest: VersionStruct + Clone> {
    /// The whole document of `value`, `format_version` included.
    ///
    /// # Errors
    ///
    /// `EncodeError::NotRepresentable` when `value` holds something JSON cannot spell, such as a
    /// non-finite number.
    fn document(value: &Self::Latest) -> Result<Value, EncodeError>;
}

/// What a document below the chain's floor is told (`Family::ENVELOPE.below_floor`).
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
