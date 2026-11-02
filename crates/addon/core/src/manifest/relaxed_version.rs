//! `RelaxedSemVersion`: a version that may be written with fewer than three parts.

use crate::version::SemVersion;

use super::read::relaxed_version_text;

/// A version read the relaxed way: `"2.0"` and `"1"` are versions whose missing parts are
/// zero, so `"2.0"` is 2.0.0. The text it was read from is kept, so the version is written
/// back as it was spelled.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelaxedSemVersion {
    /// The version, missing parts zero.
    pub version: SemVersion,
    /// The text the version was read from (`"2.0"`), written back while it still reads as
    /// `version`. `None` for a version read from a `[major, minor, patch]` array or built in
    /// code: it is written in full.
    pub text: Option<String>,
}

impl RelaxedSemVersion {
    /// `version` with no text of its own: written in full.
    pub fn new(version: SemVersion) -> RelaxedSemVersion {
        Self {
            version,
            text: None,
        }
    }

    /// The version `text` spells, keeping the text; `None` when it is not a relaxed version.
    pub(super) fn from_text(text: &str) -> Option<RelaxedSemVersion> {
        relaxed_version_text(text).map(|version| Self {
            version,
            text: Some(text.to_owned()),
        })
    }

    /// The text to write: the text read, while it still spells `version`.
    pub(super) fn spelling(&self) -> Option<&str> {
        self.text
            .as_deref()
            .filter(|text| relaxed_version_text(text).as_ref() == Some(&self.version))
    }
}

impl From<SemVersion> for RelaxedSemVersion {
    fn from(version: SemVersion) -> Self {
        Self::new(version)
    }
}
