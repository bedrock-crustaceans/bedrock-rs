//! Format 1.12.0 up to 1.14.0: `minecraft:geometry[]` with a `description`; and the step to
//! 1.14, which writes the centre of every cube without a `pivot` as its pivot.
//!
//! The game's step validates the file leniently against the 1.12 schema (an undeclared member
//! is removed), sets `format_version` to `"1.14.0"`, and gives each cube without a `pivot` the
//! pivot `origin + size * 0.5`, where a missing component counts as 0.

use super::v1_14_0;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};

super::modern_file! {
    FormatVersion::new(1, 12, 0);
    /// A geometry file of format 1.12.0 up to 1.14.0.
}

impl Upgrade for GeometryFile {
    type Next = v1_14_0::GeometryFile;

    /// The step to 1.14: every cube without `pivot` gets its centre, rotated or not. The parser
    /// of every version uses the same centre for a missing pivot, so nothing changes on screen.
    fn upgrade(mut self, _: &mut UpgradeCx<'_>) -> v1_14_0::GeometryFile {
        let cubes = self
            .geometries
            .iter_mut()
            .flat_map(|geometry| geometry.bones.iter_mut().flatten())
            .flat_map(|bone| bone.cubes.iter_mut().flatten());
        for cube in cubes {
            if cube.pivot.is_none() {
                cube.pivot = Some(cube.centre());
            }
        }
        v1_14_0::GeometryFile {
            format_version: self.format_version,
            debug: self.debug,
            geometries: self.geometries,
        }
    }
}
