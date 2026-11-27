//! The `feature_rules` family's codes: the game's content-log lines, byte for byte, each a
//! definition beside the rules that raise it. Each id is `feature_rules/<name>`, the name its
//! constant's in lower case.
//!
//! Every one is an error-level line the game logs and goes on from: the rule is kept, so each is a
//! kept error.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

/// The identifier's name is not the file stem; the arguments are the name and the stem.
pub static IDENTIFIER_MISMATCH: CodeDef<2> = CodeDef::new(
    "feature_rules/identifier_mismatch",
    Kind::InvalidValue,
    Class::KeptError,
    "identifier {} does not match filename {}",
)
.game("Feature rule identifier '{}' does not match filename '{}'");

/// The identifier has no `namespace:` before its name; the argument is the identifier.
pub static IDENTIFIER_NAMESPACE: CodeDef<1> = CodeDef::new(
    "feature_rules/identifier_namespace",
    Kind::InvalidValue,
    Class::KeptError,
    "identifier {} has no namespace",
)
.game("Feature rule identifier '{}' does not use the appropriate namespace syntax");

/// The `placement_pass` is none the game defines; the rule is kept in that pass's bucket, which
/// nothing places. The arguments are the identifier and the pass.
pub static UNKNOWN_PASS: CodeDef<2> = CodeDef::new(
    "feature_rules/unknown_pass",
    Kind::InvalidValue,
    Class::KeptError,
    "identifier {} specifies unknown pass {}",
)
.game("Feature rule identifier '{}' specifies unknown pass '{}'.");

/// A constant `iterations` below 0.
pub static ITERATIONS_NEGATIVE: CodeDef<0> = CodeDef::new(
    "feature_rules/iterations_negative",
    Kind::InvalidValue,
    Class::KeptError,
    "iterations below 0",
)
.game("Bad value for iterations - should be >= 0");

/// A constant percent `scatter_chance` outside 0 to 100, exclusive.
pub static SCATTER_CHANCE_RANGE: CodeDef<0> = CodeDef::new(
    "feature_rules/scatter_chance_range",
    Kind::InvalidValue,
    Class::KeptError,
    "scatter_chance outside (0, 100)",
)
.game("Bad value for scatter_chance - should be between 0 and 100 (exclusive)");

/// A fraction `scatter_chance` whose denominator is not above its numerator.
pub static SCATTER_CHANCE_FRACTION: CodeDef<0> = CodeDef::new(
    "feature_rules/scatter_chance_fraction",
    Kind::InvalidValue,
    Class::KeptError,
    "scatter_chance denominator not above numerator",
)
.game("Bad value for scatter_chance - denominator should be greater than the numerator");

/// A `step_size` beside a distribution that is no grid.
pub static STEP_SIZE_NOT_GRID: CodeDef<0> = CodeDef::new(
    "feature_rules/step_size_not_grid",
    Kind::InvalidValue,
    Class::KeptError,
    "step_size without a grid distribution",
)
.game("step_size is only valid when distribution is fixed_grid or jittered_grid");

/// A `grid_offset` beside a distribution that is no grid.
pub static GRID_OFFSET_NOT_GRID: CodeDef<0> = CodeDef::new(
    "feature_rules/grid_offset_not_grid",
    Kind::InvalidValue,
    Class::KeptError,
    "grid_offset without a grid distribution",
)
.game("grid_offset is only valid when distribution is fixed_grid or jittered_grid");

/// An extent whose maximum is not above its minimum.
pub static EXTENT_EMPTY: CodeDef<0> = CodeDef::new(
    "feature_rules/extent_empty",
    Kind::InvalidValue,
    Class::KeptError,
    "extent maximum not above minimum",
)
.game("Bad value for extent - maximum must be greater than minimum");

/// A coordinate that is neither a usable expression nor an object; the argument is the coordinate.
pub static COORDINATE_REQUIRED: CodeDef<1> = CodeDef::new(
    "feature_rules/coordinate_required",
    Kind::InvalidValue,
    Class::KeptError,
    "coordinate {} not provided",
)
.game("{} must be provided, either as a Molang expression string or as a sub-object");

/// Every definition of this module.
pub static ALL: &[&CodeMeta] = &[
    IDENTIFIER_MISMATCH.meta(),
    IDENTIFIER_NAMESPACE.meta(),
    UNKNOWN_PASS.meta(),
    ITERATIONS_NEGATIVE.meta(),
    SCATTER_CHANCE_RANGE.meta(),
    SCATTER_CHANCE_FRACTION.meta(),
    STEP_SIZE_NOT_GRID.meta(),
    GRID_OFFSET_NOT_GRID.meta(),
    EXTENT_EMPTY.meta(),
    COORDINATE_REQUIRED.meta(),
];
