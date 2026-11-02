//! Format 1.14.0 up to 1.16.0: the members of 1.12; and the step to 1.16, which changes
//! nothing (the 1.16 schema adds bone `binding`).
//!
//! The game's step validates the file leniently against the 1.14 schema (an undeclared member
//! is removed) and then sets `format_version` to `"1.16.0"`.

use super::v1_16_0;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};

super::modern_file! {
    FormatVersion::new(1, 14, 0);
    /// A geometry file of format 1.14.0 up to 1.16.0. A cube without `pivot` rotates around its
    /// centre, as in every version.
}

impl Upgrade for GeometryFile {
    type Next = v1_16_0::GeometryFile;

    /// The step to 1.16: no data change.
    fn upgrade(self, _: &mut UpgradeCx<'_>) -> v1_16_0::GeometryFile {
        v1_16_0::GeometryFile {
            format_version: self.format_version,
            debug: self.debug,
            geometries: self.geometries,
        }
    }
}
