//! Geometry's own texts: the game's loader lines and the crate's own wording. The node-tree
//! texts every schema walk shares are core's (`json::node_tree`).

/// The crate's own text for the same member below 1.19.30, where vanilla removes it without a
/// message.
pub(crate) const CHILD_REMOVED: &str =
    "child '{}' not valid here (removed; format_version < 1.19.30)";
/// The text for a file whose schema validation failed.
pub(crate) const DID_NOT_VALIDATE: &str = "geometry file didn't validate";
/// The text for a root that is not an object.
pub(crate) const NOT_AN_OBJECT: &str = "geometry file could not be loaded: expected the contents to be inside a JSON object - The file has to start with an \"{\" and end with a \"}\"";
/// The text for a file newer than the game.
pub(crate) const EXCEEDS_GAME_VERSION: &str =
    "Error: geometry file format version: {} exceeds game version: {}. Skipping.";
/// The text for a base-game `models/blocks/` file below 1.19.30.
pub(crate) const INTERNAL_BLOCKS_FLOOR: &str =
    "Internal blocks are required to use geometry version >= 1.19.30";
/// The text for a namespace in a geometry identifier outside the base game.
pub(crate) const NO_NAMESPACE_CUSTOM: &str =
    "Error: namespaces are not supported for custom geometry: identifier: {}";
/// The identifier loop: a namespace in a base-game entity geometry.
pub(crate) const NO_NAMESPACE_INTERNAL: &str =
    "Error: namespaces are not supported for internal entity geometry: identifier: {}";
/// The identifier loop: a base-game block geometry outside `minecraft:`.
pub(crate) const NEEDS_MINECRAFT_NAMESPACE: &str =
    "Error: Internal block geometries required the minecraft namespace: {}";
/// The text for `embedded` below 1.21.110.
pub(crate) const EMBEDDED_TOO_OLD: &str = "\"item_display_transform\" for \"{}\" is invalid - \"embedded\" can only be used in a file versioned 1.21.110 or higher";
/// The text for `shelf` below 1.26.40.
pub(crate) const SHELF_TOO_OLD: &str = "\"item_display_transform\" for \"{}\" is invalid - \"{}\" can only be used in a file version 1.26.40 or higher";
/// The text for a display-transform vector outside its range.
pub(crate) const TRANSFORM_OUT_OF_BOUNDS: &str = "\"item_display_transform\" - {} ({}, {}, {}) is out of bounds. Allowed range: ({}, {}, {}) to ({}, {}, {}).";
/// The text for a face rotation that is not a quarter turn.
pub(crate) const INVALID_UV_ROTATION: &str =
    "Invalid uv rotation value '{}', valid values are 0, 90, 180 and 270";
/// The text for an index-less mesh whose lists differ in length.
pub(crate) const NO_POLYS_SECTION: &str = "No 'polys' section, and the number of positions, normals, and UVs aren't equal ({}, {}, {} resp)";
/// The text for a `tri_list` whose positions are not triangles.
pub(crate) const NOT_TRIANGLES: &str =
    "\"polys\" specifies \"tri_list\" but positions[] isn't a multiple of three?";
/// The text for a `quad_list` whose positions are not quads.
pub(crate) const NOT_QUADS: &str =
    "\"polys\" specifies \"quad_list\" but positions[] isn't a multiple of four?";
/// The model build: a bone that names itself as its parent.
pub(crate) const PARENT_IS_SELF: &str = "Error: parenting model part to itself!";
/// The crate's own text for an unreadable `format_version`.
pub(crate) const VERSION_NOT_VALID: &str =
    "format_version '{}' is not a valid version; treated as 1.0.0";
/// The crate's own text for a legacy parent no pack defines.
pub(crate) const PARENT_NOT_DEFINED: &str =
    "geometry '{}' inherits from '{}', which is not defined";
/// The crate's own text for a bone whose parent is no bone of the geometry.
pub(crate) const BONE_PARENT_MISSING: &str =
    "bone '{}' has parent '{}', which is not a bone of geometry '{}'";
/// The crate's own text for a clamped bound.
pub(crate) const BOUNDS_CLAMPED: &str = "visible_bounds_{} {} clamped to {}";
/// The crate's own text for a box `uv` component with a fraction, which the game's integer
/// read drops.
pub(crate) const UV_TRUNCATED: &str = "box uv {} is read as the whole number {}";
/// The crate's own text for a bone's `name` or `parent` written in another case than the lower
/// case it is read in.
pub(crate) const BONE_LOWER_CASED: &str = "bone {} '{}' is read as '{}'";
/// The crate's own text for a `poly_mesh` index that is no whole number of 32 bits, which the
/// game's unsigned read turns into one.
pub(crate) const INDEX_NOT_WHOLE: &str = "poly_mesh index {} is read as {}";
/// The crate's own text for the geometry-level `cape`, which is lost.
pub(crate) const CAPE_DROPPED: &str = "cape is not carried over from format 1.8 geometry";
