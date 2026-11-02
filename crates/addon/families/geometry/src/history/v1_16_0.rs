//! Format 1.16.0 up to 1.19.30: bone `binding`; and the step to 1.19.30, which changes nothing
//! (the 1.19.30 schema lets `description.identifier` carry a namespace and is validated
//! strictly).
//!
//! The game's step validates the file leniently against the 1.16 schema (an undeclared member
//! is removed) and then sets `format_version` to `"1.19.30"`.

use super::v1_19_30;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};

super::modern_file! {
    FormatVersion::new(1, 16, 0);
    /// A geometry file of format 1.16.0 up to 1.19.30: a bone may have a `binding`.
}

impl Upgrade for GeometryFile {
    type Next = v1_19_30::GeometryFile;

    /// The step to 1.19.30: no data change.
    fn upgrade(self, _: &mut UpgradeCx<'_>) -> v1_19_30::GeometryFile {
        v1_19_30::GeometryFile {
            format_version: self.format_version,
            debug: self.debug,
            geometries: self.geometries,
        }
    }
}
