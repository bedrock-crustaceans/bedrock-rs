//! The animation controller file as written in older format versions, and the step that
//! upgrades it to [`super::AnimationControllerFile`].
//!
//! The chain is `v1_8_0 → v1_10_0`: a file from 1.8.0 up to 1.10.0 parses as
//! [`v1_8_0::AnimationControllerFile`] and its blend curves become state variables; a file from
//! 1.10.0 up to the engine version parses as the newest struct. Below 1.8.0 or above the engine
//! a controller file is not loaded. Animation files have one struct and no history.

pub mod v1_8_0;

use super::AnimationControllerFile;
use bedrock_addon_core::document::{Document, Rejected};
use bedrock_addon_core::json::{EncodeError, WriteStyle};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::version::FormatVersion;

/// A controller file parsed by the struct its `format_version` selects, not upgraded.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Any {
    /// Format 1.8.0 up to 1.10.0.
    V1_8_0(v1_8_0::AnimationControllerFile),
    /// Format 1.10.0 and later: the newest document.
    V1_10_0(AnimationControllerFile),
}

impl Any {
    /// The version of the struct that parsed the file.
    #[must_use]
    pub fn parsed_version(&self) -> FormatVersion {
        match self {
            Self::V1_8_0(_) => FormatVersion::new(1, 8, 0),
            Self::V1_10_0(_) => FormatVersion::new(1, 10, 0),
        }
    }

    /// Writes the file in its own struct's shape (a round trip without upgrading).
    ///
    /// # Errors
    ///
    /// Returns [`EncodeError`](bedrock_addon_core::json::EncodeError) when a value has no JSON
    /// spelling, such as a non-finite number.
    pub fn to_json(&self) -> Result<String, EncodeError> {
        let mut out = Vec::new();
        let value = match self {
            Self::V1_8_0(file) => file.to_value(),
            Self::V1_10_0(file) => file.to_value(),
        }?;
        bedrock_addon_core::json::write_json(&value, &mut out, WriteStyle::default())?;
        Ok(String::from_utf8(out).unwrap_or_default())
    }
}

/// Runs the remaining upgrade step and the checks of a document
/// [`AnimationControllerFile::parse_as_written`] gave: the same result
/// [`AnimationControllerFile::parse`](bedrock_addon_core::document::Content::parse) gives for its
/// file. The step runs over the document, which holds the origin the step needs.
///
/// # Errors
///
/// Returns [`Rejected`](bedrock_addon_core::document::Rejected) when the upgrade step, or the
/// checks that follow it, find an error that stops the document from loading.
pub fn upgrade(
    document: Document<Any>,
    cx: &LoadContext,
) -> Result<Document<AnimationControllerFile>, Rejected> {
    super::load::upgrade_document(document, cx)
}
