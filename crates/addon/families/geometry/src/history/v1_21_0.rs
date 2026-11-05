//! Format 1.21.0 up to 1.26.40: `item_display_transforms` and face `uv_rotation`; and the step
//! to 1.26.40, which copies the `fixed` transform to `shelf`.
//!
//! The game's step looks at each geometry whose `item_display_transforms` is an object: a
//! `shelf` member logs the "can only be used in a file version 1.26.40 or higher" text and
//! fails the file; otherwise an object `fixed` is copied to `shelf` as it is.

use super::super::schema::{SHELF_TOO_OLD, text};
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, JsonPath, Phase};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{Upgrade, UpgradeCx, Versioned};

super::modern_file! {
    FormatVersion::new(1, 21, 0);
    /// A geometry file of format 1.21.0 up to 1.26.40.
}

impl Upgrade for GeometryFile {
    type Next = super::super::GeometryFile;

    /// The step to 1.26.40, on an engine of 1.26.40 or later (an older game has neither the
    /// step nor the transform): the first geometry with a `shelf` transform fails the file;
    /// before it, a `fixed` transform is copied to `shelf`, out-of-range values included.
    fn upgrade(mut self, cx: &mut UpgradeCx<'_>) -> super::super::GeometryFile {
        let step = <super::super::GeometryFile as Versioned>::VERSION;
        if cx.load.engine_version >= step {
            for (index, geometry) in self.geometries.iter_mut().enumerate() {
                let Some(transforms) = &mut geometry.item_display_transforms else {
                    continue;
                };
                if transforms.shelf.is_some() {
                    let at = JsonPath::new()
                        .child("minecraft:geometry")
                        .index(index)
                        .child("item_display_transforms")
                        .child("shelf");
                    cx.diagnostics.push(
                        Diagnostic::error(DiagnosticCode::InvalidValue {
                            text: text(SHELF_TOO_OLD, &[&geometry.description.identifier, "shelf"]),
                        })
                        .at(at)
                        .phase(Phase::Upgrade),
                    );
                    break;
                }
                transforms.shelf.clone_from(&transforms.fixed);
            }
        }
        super::super::GeometryFile {
            format_version: self.format_version,
            debug: self.debug,
            geometries: self.geometries,
        }
    }
}
