//! Vanilla's content-log texts of the voxel-shape loader,
//! byte for byte, each built with the arguments it takes.

use bedrock_addon_core::diagnostic::VanillaText;

/// A member the schema does not bind.
pub(crate) fn not_in_schema() -> VanillaText {
    VanillaText::Fixed(bedrock_addon_core::diagnostic::cereal::MEMBER_NOT_IN_SCHEMA_TEXT)
}

/// The help text of a corner that is neither `[x, y, z]` nor an object that loads.
pub(crate) fn expected_x_y_z() -> VanillaText {
    VanillaText::Fixed("Expected [x, y, z].")
}

/// More than 32 boxes, as the game writes the line (path prefix included).
pub(crate) fn too_many_boxes() -> VanillaText {
    VanillaText::Fixed(" -> shape -> boxes: too many input elements, expected no more than 32")
}

/// A base-game pack names a shape outside the `minecraft` namespace.
pub(crate) fn namespace_required(identifier: &str) -> VanillaText {
    VanillaText::Formatted {
        template: "The `minecraft` namespace is required for Vanilla shape: `{}`",
        args: vec![identifier.to_owned()],
    }
}

/// A pack that is not the base game's names a shape in the `minecraft` namespace.
pub(crate) fn namespace_not_valid(identifier: &str) -> VanillaText {
    VanillaText::Formatted {
        template: "The `minecraft` namespace is not valid for custom voxel shape: `{}`",
        args: vec![identifier.to_owned()],
    }
}

/// Box `index` (zero-based) has a `min` that is not below its `max`.
pub(crate) fn box_invalid(index: usize, identifier: &str) -> VanillaText {
    VanillaText::Formatted {
        template: "Box at index {} is invalid for voxel shape `{}`",
        args: vec![index.to_string(), identifier.to_owned()],
    }
}

/// Box `index` leaves the bounds (vanilla formats the two corners with `%g`; they are
/// constants).
pub(crate) fn box_out_of_bounds(index: usize, identifier: &str) -> VanillaText {
    VanillaText::Formatted {
        template: "Box at index {} in voxel shape '{}' is outside the allowed bounds of (-14, -14, -14) to (30, 30, 30).",
        args: vec![index.to_string(), identifier.to_owned()],
    }
}

/// No box overlaps the block itself.
pub(crate) fn no_overlap(identifier: &str) -> VanillaText {
    VanillaText::Formatted {
        template: "Voxel shape '{}' has no boxes that overlap the base block (0-16 pixels on all axes).",
        args: vec![identifier.to_owned()],
    }
}

/// The shape spans `extent` pixels on `axis` (`x`, `y` or `z`), more than 30.
pub(crate) fn extent_too_wide(identifier: &str, axis: &str, extent: f32) -> VanillaText {
    VanillaText::Formatted {
        template: "Total extent of voxel shape '{}' on axis {} is {} pixels, which exceeds the maximum of {} pixels (1 block + 14 pixels).",
        args: vec![
            identifier.to_owned(),
            axis.to_owned(),
            general(extent),
            "30".to_owned(),
        ],
    }
}

/// `x` as vanilla's `{:g}` prints it: six significant digits without trailing zeros. Only for
/// the extents the width check reports, which lie between 30 and 44 (every box passed the
/// bounds check), so the form is
/// always two digits, a point and up to four more.
fn general(x: f32) -> String {
    let fixed = format!("{x:.4}");
    fixed.trim_end_matches('0').trim_end_matches('.').to_owned()
}

/// The line after a file the schema loader rejected; the argument is the file's path in its
/// pack.
pub(crate) fn failed_to_load(path: &str) -> VanillaText {
    VanillaText::Formatted {
        template: "Failed to load voxel shape file {}",
        args: vec![path.to_owned()],
    }
}
