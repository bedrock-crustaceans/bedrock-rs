//! The geometry family's codes: the game's loader lines for geometry files, byte for byte, and
//! the crate's own wording, each a definition beside the rules that raise it. The node-tree lines
//! every schema walk shares are core's ([`codes`](bedrock_addon_core::diagnostic::codes)). Each
//! [`was`](CodeDef::was) the legacy code its sites raised, so a pin or a filter on the code's name
//! keeps matching.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Disposition, Kind};
use bedrock_addon_core::json::node_tree;

// ---- the envelope and its version gates ----

/// The closing line of a file whose schema validation failed.
pub static DID_NOT_VALIDATE: CodeDef<0> = CodeDef::new(
    "geometry/did_not_validate",
    Kind::InvalidValue,
    Class::Error,
    "geometry file did not validate",
)
.game("geometry file didn't validate")
.was("InvalidValue");

/// A root that is not an object. The finding carries the crate's message as its
/// `DiagnosticDetail::Syntax`.
pub static NOT_AN_OBJECT: CodeDef<0> = CodeDef::new(
    "geometry/not_an_object",
    Kind::Syntax,
    Class::Error,
    "the geometry file's root is not an object",
)
.game("geometry file could not be loaded: expected the contents to be inside a JSON object - The file has to start with an \"{\" and end with a \"}\"")
.was("MalformedInput");

/// A file newer than the game; the arguments are the version as written and the engine version.
pub static EXCEEDS_GAME_VERSION: CodeDef<2> = CodeDef::new(
    "geometry/exceeds_game_version",
    Kind::UnsupportedVersion,
    Class::Error,
    "geometry format_version {} above the game's {}",
)
.game("Error: geometry file format version: {} exceeds game version: {}. Skipping.")
.was("UnsupportedVersion");

/// A base-game `models/blocks/` file below 1.19.30.
pub static INTERNAL_BLOCKS_FLOOR: CodeDef<0> = CodeDef::new(
    "geometry/internal_blocks_floor",
    Kind::UnsupportedVersion,
    Class::Error,
    "base-game block geometry below 1.19.30",
)
.game("Internal blocks are required to use geometry version >= 1.19.30")
.was("UnsupportedVersion");

/// The keyed (1.8) layout's version node maximum: 1.11.1 up to 1.12.0 fails the schema (the
/// node tree's `ABOVE_MAX_VERSION`).
pub static ABOVE_MAX_VERSION: CodeDef<2> = CodeDef::new(
    "geometry/above_max_version",
    Kind::UnsupportedVersion,
    Class::Error,
    "format_version {} above the keyed layout's maximum {}",
)
.game(node_tree::ABOVE_MAX_VERSION)
.was("UnsupportedVersion");

/// The crate's own information for an unreadable `format_version`.
pub static VERSION_NOT_VALID: CodeDef<1> = CodeDef::new(
    "geometry/version_not_valid",
    Kind::FormatVersion,
    Class::Lint,
    "format_version '{}' is not a valid version; treated as 1.0.0",
)
.game("format_version '{}' is not a valid version; treated as 1.0.0")
.was("MalformedFormatVersion");

// ---- identifiers ----

/// A namespace in a geometry identifier outside the base game.
pub static NO_NAMESPACE_CUSTOM: CodeDef<1> = CodeDef::new(
    "geometry/no_namespace_custom",
    Kind::InvalidValue,
    Class::Error,
    "namespace in custom geometry identifier {}",
)
.game("Error: namespaces are not supported for custom geometry: identifier: {}")
.was("InvalidValue");

/// The identifier loop: a namespace in a base-game entity geometry.
pub static NO_NAMESPACE_INTERNAL: CodeDef<1> = CodeDef::new(
    "geometry/no_namespace_internal",
    Kind::InvalidValue,
    Class::Error,
    "namespace in base-game entity geometry identifier {}",
)
.game("Error: namespaces are not supported for internal entity geometry: identifier: {}")
.was("InvalidValue");

/// The identifier loop: a base-game block geometry outside `minecraft:`.
pub static NEEDS_MINECRAFT_NAMESPACE: CodeDef<1> = CodeDef::new(
    "geometry/needs_minecraft_namespace",
    Kind::InvalidValue,
    Class::Error,
    "base-game block geometry {} outside the minecraft namespace",
)
.game("Error: Internal block geometries required the minecraft namespace: {}")
.was("InvalidValue");

// ---- item display transforms ----

/// `embedded` below 1.21.110; the game ignores the transform and loads the geometry.
pub static EMBEDDED_TOO_OLD: CodeDef<1> = CodeDef::new(
    "geometry/embedded_too_old",
    Kind::InvalidValue,
    Class::KeptError,
    "embedded display transform of {} below 1.21.110",
)
.game("\"item_display_transform\" for \"{}\" is invalid - \"embedded\" can only be used in a file versioned 1.21.110 or higher")
.was("InvalidValue");

/// `shelf` below 1.26.40; the arguments are the geometry's identifier and the transform's name.
pub static SHELF_TOO_OLD: CodeDef<2> = CodeDef::new(
    "geometry/shelf_too_old",
    Kind::InvalidValue,
    Class::Error,
    "{} display transform {} below 1.26.40",
)
.game("\"item_display_transform\" for \"{}\" is invalid - \"{}\" can only be used in a file version 1.26.40 or higher")
.was("InvalidValue");

/// A display-transform vector outside its range; the game does not apply it and loads the
/// geometry.
pub static TRANSFORM_OUT_OF_BOUNDS: CodeDef<10> = CodeDef::new(
    "geometry/transform_out_of_bounds",
    Kind::InvalidValue,
    Class::KeptError,
    "display transform {} ({}, {}, {}) outside ({}, {}, {}) to ({}, {}, {})",
)
.game("\"item_display_transform\" - {} ({}, {}, {}) is out of bounds. Allowed range: ({}, {}, {}) to ({}, {}, {}).")
.was("InvalidValue");

// ---- bones, cubes and meshes ----

/// A face rotation that is not a quarter turn; the game uses 0 and loads the geometry.
pub static INVALID_UV_ROTATION: CodeDef<1> = CodeDef::new(
    "geometry/invalid_uv_rotation",
    Kind::InvalidValue,
    Class::KeptError,
    "uv rotation {} is not a quarter turn",
)
.game("Invalid uv rotation value '{}', valid values are 0, 90, 180 and 270")
.was("InvalidValue");

/// An index-less mesh whose lists differ in length.
pub static NO_POLYS_SECTION: CodeDef<3> = CodeDef::new(
    "geometry/no_polys_section",
    Kind::InvalidValue,
    Class::KeptError,
    "poly_mesh lists of unequal lengths ({}, {}, {})",
)
.game("No 'polys' section, and the number of positions, normals, and UVs aren't equal ({}, {}, {} resp)")
.was("InvalidValue");

/// A `tri_list` whose positions are not triangles.
pub static NOT_TRIANGLES: CodeDef<0> = CodeDef::new(
    "geometry/not_triangles",
    Kind::InvalidValue,
    Class::KeptError,
    "tri_list positions are not triangles",
)
.game("\"polys\" specifies \"tri_list\" but positions[] isn't a multiple of three?")
.was("InvalidValue");

/// A `quad_list` whose positions are not quads.
pub static NOT_QUADS: CodeDef<0> = CodeDef::new(
    "geometry/not_quads",
    Kind::InvalidValue,
    Class::KeptError,
    "quad_list positions are not quads",
)
.game("\"polys\" specifies \"quad_list\" but positions[] isn't a multiple of four?")
.was("InvalidValue");

/// The model build: a bone that names itself as its parent (a developer assertion of the
/// game's, told as an information of the crate's own).
pub static PARENT_IS_SELF: CodeDef<0> = CodeDef::new(
    "geometry/parent_is_self",
    Kind::Cycle,
    Class::Lint,
    "bone is its own parent",
)
.game("Error: parenting model part to itself!")
.was("ReferenceCycle");

/// The crate's own information for a bone whose parent is no bone of the geometry.
pub static BONE_PARENT_MISSING: CodeDef<3> = CodeDef::new(
    "geometry/bone_parent_missing",
    Kind::UnresolvedReference,
    Class::Lint,
    "bone '{}' has parent '{}', which is not a bone of geometry '{}'",
)
.game("bone '{}' has parent '{}', which is not a bone of geometry '{}'")
.was("UnresolvedReference");

/// The crate's own information for a bone's `name` or `parent` written in another case than the
/// lower case it is read in.
pub static BONE_LOWER_CASED: CodeDef<3> = CodeDef::new(
    "geometry/bone_lower_cased",
    Kind::InvalidValue,
    Class::Lint,
    "bone {} '{}' is read as '{}'",
)
.game("bone {} '{}' is read as '{}'")
.was("InvalidValue");

/// The crate's own information for a clamped bound.
pub static BOUNDS_CLAMPED: CodeDef<3> = CodeDef::new(
    "geometry/bounds_clamped",
    Kind::Clamped,
    Class::Lint,
    "visible_bounds_{} {} clamped to {}",
)
.game("visible_bounds_{} {} clamped to {}")
.was("ClampedValue");

/// The crate's own information for a box `uv` component with a fraction, which the game's
/// integer read drops.
pub static UV_TRUNCATED: CodeDef<2> = CodeDef::new(
    "geometry/uv_truncated",
    Kind::InvalidValue,
    Class::Lint,
    "box uv {} is read as the whole number {}",
)
.game("box uv {} is read as the whole number {}")
.was("InvalidValue");

/// The crate's own information for a `poly_mesh` index that is no whole number of 32 bits,
/// which the game's unsigned read turns into one.
pub static INDEX_NOT_WHOLE: CodeDef<2> = CodeDef::new(
    "geometry/index_not_whole",
    Kind::InvalidValue,
    Class::Lint,
    "poly_mesh index {} is read as {}",
)
.game("poly_mesh index {} is read as {}")
.was("InvalidValue");

// ---- inheritance and the upgrade ----

/// The crate's own text for a legacy parent no pack defines.
pub static PARENT_NOT_DEFINED: CodeDef<2> = CodeDef::new(
    "geometry/parent_not_defined",
    Kind::UnresolvedReference,
    Class::Error,
    "geometry '{}' inherits from '{}', which is not defined",
)
.game("geometry '{}' inherits from '{}', which is not defined")
.was("UnresolvedReference");

/// The crate's own information for the geometry-level `cape`, which is lost.
pub static CAPE_DROPPED: CodeDef<0> = CodeDef::new(
    "geometry/cape_dropped",
    Kind::UnknownMember(Disposition::Dropped),
    Class::Lint,
    "cape is not carried over from format 1.8 geometry",
)
.game("cape is not carried over from format 1.8 geometry")
.was("DroppedField");

/// Every definition of the family, for the registry of ids.
pub static ALL: &[&CodeMeta] = &[
    DID_NOT_VALIDATE.meta(),
    NOT_AN_OBJECT.meta(),
    EXCEEDS_GAME_VERSION.meta(),
    INTERNAL_BLOCKS_FLOOR.meta(),
    ABOVE_MAX_VERSION.meta(),
    VERSION_NOT_VALID.meta(),
    NO_NAMESPACE_CUSTOM.meta(),
    NO_NAMESPACE_INTERNAL.meta(),
    NEEDS_MINECRAFT_NAMESPACE.meta(),
    EMBEDDED_TOO_OLD.meta(),
    SHELF_TOO_OLD.meta(),
    TRANSFORM_OUT_OF_BOUNDS.meta(),
    INVALID_UV_ROTATION.meta(),
    NO_POLYS_SECTION.meta(),
    NOT_TRIANGLES.meta(),
    NOT_QUADS.meta(),
    PARENT_IS_SELF.meta(),
    BONE_PARENT_MISSING.meta(),
    BONE_LOWER_CASED.meta(),
    BOUNDS_CLAMPED.meta(),
    UV_TRUNCATED.meta(),
    INDEX_NOT_WHOLE.meta(),
    PARENT_NOT_DEFINED.meta(),
    CAPE_DROPPED.meta(),
];
