//! The `voxel_shape` family's codes: the game's content-log lines, byte for byte, each a definition
//! beside the rules that raise it. Each [`was`](CodeDef::was) the legacy code its sites raised, so
//! a pin or a filter on the code's name keeps matching.
//!
//! Each carries the arguments it takes.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

/// The line after a file the schema loader rejected; the argument is the file's path in its pack.
pub static FAILED_TO_LOAD: CodeDef<1> = CodeDef::new(
    "voxel_shape/failed_to_load",
    Kind::Closing,
    Class::Error,
    "voxel shape file {} failed to load",
)
.game("Failed to load voxel shape file {}")
.was("DocumentRejected");

/// A base-game pack names a shape outside the `minecraft` namespace; the argument is the
/// identifier.
pub static NAMESPACE_REQUIRED: CodeDef<1> = CodeDef::new(
    "voxel_shape/namespace_required",
    Kind::InvalidValue,
    Class::Error,
    "base-game shape {} outside the minecraft namespace",
)
.game("The `minecraft` namespace is required for Vanilla shape: `{}`")
.was("InvalidValue");

/// A pack that is not the base game's names a shape in the `minecraft` namespace; the argument is
/// the identifier.
pub static NAMESPACE_NOT_VALID: CodeDef<1> = CodeDef::new(
    "voxel_shape/namespace_not_valid",
    Kind::InvalidValue,
    Class::Error,
    "custom shape {} in the minecraft namespace",
)
.game("The `minecraft` namespace is not valid for custom voxel shape: `{}`")
.was("InvalidValue");

/// Box `index` (zero-based) has a `min` that is not below its `max`; the arguments are the index
/// and the identifier.
pub static BOX_INVALID: CodeDef<2> = CodeDef::new(
    "voxel_shape/box_invalid",
    Kind::InvalidValue,
    Class::Error,
    "box {} of shape {} has min not below max",
)
.game("Box at index {} is invalid for voxel shape `{}`")
.was("InvalidValue");

/// Box `index` leaves the bounds (the game formats the two corners with `%g`; they are constants);
/// the arguments are the index and the identifier.
pub static BOX_OUT_OF_BOUNDS: CodeDef<2> = CodeDef::new(
    "voxel_shape/box_out_of_bounds",
    Kind::InvalidValue,
    Class::Error,
    "box {} of shape {} leaves the bounds",
)
.game("Box at index {} in voxel shape '{}' is outside the allowed bounds of (-14, -14, -14) to (30, 30, 30).")
.was("InvalidValue");

/// No box overlaps the block itself; the argument is the identifier.
pub static NO_OVERLAP: CodeDef<1> = CodeDef::new(
    "voxel_shape/no_overlap",
    Kind::InvalidValue,
    Class::Error,
    "shape {} has no box over the block",
)
.game("Voxel shape '{}' has no boxes that overlap the base block (0-16 pixels on all axes).")
.was("InvalidValue");

/// The shape spans more than 30 pixels on an axis; the arguments are the identifier, the axis, the
/// extent as the game prints `%g` and the limit.
pub static EXTENT_TOO_WIDE: CodeDef<4> = CodeDef::new(
    "voxel_shape/extent_too_wide",
    Kind::InvalidValue,
    Class::Error,
    "shape {} on axis {} spans {} pixels, above {}",
)
.game("Total extent of voxel shape '{}' on axis {} is {} pixels, which exceeds the maximum of {} pixels (1 block + 14 pixels).")
.was("InvalidValue");

/// More than 32 boxes, as the game writes the line (path prefix included).
pub static TOO_MANY_BOXES: CodeDef<0> = CodeDef::new(
    "voxel_shape/too_many_boxes",
    Kind::InvalidValue,
    Class::Error,
    "more than 32 boxes",
)
.game(" -> shape -> boxes: too many input elements, expected no more than 32")
.was("InvalidValue");

/// The help text of a corner that is neither `[x, y, z]` nor an object that loads.
pub static EXPECTED_X_Y_Z: CodeDef<0> = CodeDef::new(
    "voxel_shape/expected_x_y_z",
    Kind::WrongType,
    Class::Error,
    "corner is no [x, y, z]",
)
.game("Expected [x, y, z].")
.was("InvalidType");

/// Every definition of this module.
pub static ALL: &[&CodeMeta] = &[
    FAILED_TO_LOAD.meta(),
    NAMESPACE_REQUIRED.meta(),
    NAMESPACE_NOT_VALID.meta(),
    BOX_INVALID.meta(),
    BOX_OUT_OF_BOUNDS.meta(),
    NO_OVERLAP.meta(),
    EXTENT_TOO_WIDE.meta(),
    TOO_MANY_BOXES.meta(),
    EXPECTED_X_Y_Z.meta(),
];
