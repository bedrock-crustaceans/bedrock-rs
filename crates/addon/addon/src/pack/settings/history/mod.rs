//! The saved-settings document as written in older format versions, and the step that upgrades
//! it to [`super::SavedSettings`].
//!
//! The chain is `v1_21_100 → v1_26_40`: [`v1_21_100::SavedSettings`] parses `[1.21.100,
//! 1.26.40)` and [`super::SavedSettings`] exactly 1.26.40 (the newest struct closes the last
//! range at its own version). The step is a plain copy: 1.26.40 only lets a value also be an
//! array of strings. Below 1.21.100 a document is not loaded; there is no beta struct.

pub mod v1_21_100;

/// The pack-settings family of the version-chain machinery (hidden: only the parser names it).
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct PackSettingsFamily;

bedrock_addon_core::version_chain! {
    family = PackSettingsFamily;
    /// A saved-settings document parsed by the struct its `format_version` selects, not
    /// upgraded.
    #[derive(Clone, Debug, PartialEq)]
    pub enum Any {
        /// Format 1.21.100 up to 1.26.40.
        V1_21_100(v1_21_100::SavedSettings),
        /// Format 1.26.40: the newest document.
        V1_26_40(super::SavedSettings),
    }
    molang = none;
}
