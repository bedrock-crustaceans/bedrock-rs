//! Format 1.19.30 up to 1.21.0: an identifier may carry a namespace and any undeclared member
//! fails the file; and the step to 1.21, which changes nothing (the 1.21 schema adds
//! `item_display_transforms` and face `uv_rotation`).
//!
//! The game's step validates the file strictly against the 1.19.30 schema (an undeclared member
//! is an error, not silently removed) and then sets `format_version` to `"1.21.0"`.

use super::v1_21_0;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};

super::modern_file! {
    FormatVersion::new(1, 19, 30);
    /// A geometry file of format 1.19.30 up to 1.21.0, the first strictly validated one.
}

impl Upgrade for GeometryFile {
    type Next = v1_21_0::GeometryFile;

    /// The step to 1.21: no data change.
    fn upgrade(self, _: &mut UpgradeCx<'_>) -> v1_21_0::GeometryFile {
        v1_21_0::GeometryFile {
            format_version: self.format_version,
            debug: self.debug,
            geometries: self.geometries,
        }
    }
}
