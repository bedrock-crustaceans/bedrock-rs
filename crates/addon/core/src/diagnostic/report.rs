//! [`PackReport`]: the manifest and container findings of one pack.

use std::fmt;

use super::Diagnostics;
use crate::load::Place;
use crate::pack_identity::{PackIdVersion, PackOrigin, PackType};
use crate::version::BaseGameVersion;

/// Manifest and container findings of one pack: the field set of the game's pack report.
///
/// Built by the pack loader; `has_errors` is what decides, in vanilla, whether a manifest was
/// valid.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct PackReport {
    /// Errors and warnings about the manifest and the container (vanilla keeps two lists; the
    /// severity tells them apart).
    pub diagnostics: Diagnostics,
    /// The manifest was upgraded from an older format.
    pub was_upgraded: bool,
    /// An upgrade was attempted.
    pub attempted_upgrade: bool,
    /// Where the pack was found.
    pub location: Place,
    /// The base-game version the pack requires, when it declares one.
    pub required_base_game_version: Option<BaseGameVersion>,
    /// The header name as written.
    pub original_name: String,
    /// The header version as written.
    pub original_version: String,
    /// `(uuid, version)` of the header; `None` when the manifest yields no identity.
    pub identity: Option<PackIdVersion>,
    /// What the pack holds.
    pub pack_type: PackType,
    /// Where the pack comes from.
    pub pack_origin: PackOrigin,
}

impl PackReport {
    /// An empty report for a pack found at `location`.
    // hidden: the report is the output of `bedrock_addon`'s loaders and is built only there
    // (`load/pack.rs`, `pack/mod.rs`, `pack/writer/mod.rs`); it needs `pub` because that is
    // another crate. The struct is `#[non_exhaustive]` and will gain fields.
    #[doc(hidden)]
    pub fn new(location: Place, pack_type: PackType, pack_origin: PackOrigin) -> Self {
        Self {
            diagnostics: Diagnostics::default(),
            was_upgraded: false,
            attempted_upgrade: false,
            location,
            required_base_game_version: None,
            original_name: String::new(),
            original_version: String::new(),
            identity: None,
            pack_type,
            pack_origin,
        }
    }

    /// The findings about the manifest and the container.
    pub fn diagnostics(&self) -> &Diagnostics {
        &self.diagnostics
    }

    /// At least one rejecting error-level finding (a kept error does not count), as the
    /// game's pack report counts errors.
    pub fn has_errors(&self) -> bool {
        self.diagnostics.has_errors()
    }

    /// For error messages: the first error, else the most severe finding, else "no findings".
    pub fn summary(&self) -> impl fmt::Display + '_ {
        self.diagnostics.summary()
    }
}
