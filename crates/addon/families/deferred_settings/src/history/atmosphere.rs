//! Atmosphere settings as written at format 1.21.40: the members of the newest struct, with
//! `description` optional. The step to 1.26.20 changes no data.

use crate::AtmosphereSettings;

/// Format 1.21.40.
pub mod v1_21_40 {
    /// Atmosphere settings at format 1.21.40: the members of the newest struct, `description`
    /// optional.
    #[derive(Clone, Debug, Default, PartialEq)]
    pub struct AtmosphereSettings(pub crate::AtmosphereSettings);
}

chain! {
    latest = AtmosphereSettings;
    /// An atmosphere-settings file parsed by the struct its `format_version` selects, not
    /// upgraded.
    enum AtmosphereHistory {
        /// Format 1.21.40.
        V1_21_40(v1_21_40::AtmosphereSettings),
        /// Format 1.26.20: the newest atmosphere settings.
        V1_26_20(AtmosphereSettings),
    }
}
