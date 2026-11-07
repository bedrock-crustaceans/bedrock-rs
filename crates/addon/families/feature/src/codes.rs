//! The feature family's codes: the client's content-log lines of the feature loader stage, the
//! schema's node lines the core does not hold, and the setters' messages, byte for byte. Each
//! [`was`](CodeDef::was) the legacy code its sites raised, so a pin or a filter on the code's
//! name keeps matching.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

// ---- the loader stage ----

/// A file that is no valid JSON; the argument is its path. The finding carries the reader's
/// message as its `DiagnosticDetail::Syntax`.
pub static INVALID_JSON: CodeDef<1> = CodeDef::new(
    "feature/invalid_json",
    Kind::Syntax,
    Class::Error,
    "the feature file {} is not valid JSON",
)
.game("[Feature Registry] Passed in contents could not be evaluated as valid JSON in the following file: {}")
.was("MalformedInput");

/// A file without a description identifier.
pub static MISSING_IDENTIFIER: CodeDef<1> = CodeDef::new(
    "feature/missing_identifier",
    Kind::MissingRequired,
    Class::Error,
    "the feature file {} has no description identifier",
)
.game("[Feature Registry] Missing description identifier for JSON in the following file: {}")
.was("MissingRequiredField");

/// An empty description identifier.
pub static EMPTY_IDENTIFIER: CodeDef<1> = CodeDef::new(
    "feature/empty_identifier",
    Kind::InvalidValue,
    Class::Error,
    "the feature file {} has an empty description identifier",
)
.game("[Feature Registry] Empty description identifier for JSON in the following file: {}")
.was("InvalidValue");

/// An identifier that is not `namespace:id`.
pub static NOT_NAMESPACED: CodeDef<1> = CodeDef::new(
    "feature/not_namespaced",
    Kind::Identifier,
    Class::Error,
    "the feature identifier {} is not namespace:id",
)
.game("The feature identifier '{}' should be in the format namespace:id")
.was("NamespaceRule");

/// The built-in definitions' namespace rule (the game's spelling).
pub static NOT_MINECRAFT: CodeDef<1> = CodeDef::new(
    "feature/not_minecraft",
    Kind::Identifier,
    Class::Error,
    "the built-in feature identifier {} is not in the minecraft namespace",
)
.game("The feature identifer '{}' should be in the 'minecraft' namespace")
.was("NamespaceRule");

/// An internal feature type used outside the game's packs.
pub static INTERNAL_TYPE: CodeDef<1> = CodeDef::new(
    "feature/internal_type",
    Kind::Toggle,
    Class::Error,
    "the internal feature type {} outside the game's packs",
)
.game("The feature uses the feature type '{}', which is considered internal and thus not compatible with this engine version of the game.")
.was("MissingToggles");

/// A second feature of one identifier in a pack (engine 1.26.40 and later).
pub static DUPLICATE: CodeDef<1> = CodeDef::new(
    "feature/duplicate",
    Kind::Duplicate,
    Class::Error,
    "the feature {} already exists",
)
.game("Feature with identifier '{}' already exists")
.was("DuplicateIdentifier");

// ---- the schema's nodes ----

/// An `IntRange` whose lower end is outside the node's range.
pub static RANGE_MIN_OUT_OF_RANGE: CodeDef<3> = CodeDef::new(
    "feature/range_min_out_of_range",
    Kind::InvalidValue,
    Class::Error,
    "range_min {} is outside [{}, {}]",
)
.game("'range_min' with value '{}' outside valid range [{}, {}]")
.was("InvalidValue");

/// An `IntRange` whose upper end is outside the node's range.
pub static RANGE_MAX_OUT_OF_RANGE: CodeDef<3> = CodeDef::new(
    "feature/range_max_out_of_range",
    Kind::InvalidValue,
    Class::Error,
    "range_max {} is outside [{}, {}]",
)
.game("'range_max' with value '{}' outside valid range [{}, {}]")
.was("InvalidValue");

// ---- the setters of named members (an unknown name) ----

/// A `snap_to_surface_feature`'s unknown `surface`.
pub static SNAP_SURFACE: CodeDef<0> = CodeDef::new(
    "feature/snap_surface",
    Kind::InvalidValue,
    Class::Error,
    "bad surface of a snap_to_surface_feature",
)
.game("Bad value for surface - should be 'ceiling', 'floor', 'random_horizontal', or `wall`")
.was("InvalidValue");

/// A `growing_plant_feature`'s unknown `growth_direction`.
pub static GROWTH_DIRECTION: CodeDef<0> = CodeDef::new(
    "feature/growth_direction",
    Kind::InvalidValue,
    Class::Error,
    "bad growth_direction",
)
.game("Bad value for growth_direction - should be 'UP' or 'DOWN'")
.was("InvalidValue");

/// A `partially_exposed_blob_feature`'s unknown `exposed_face`.
pub static EXPOSED_FACE: CodeDef<0> = CodeDef::new(
    "feature/exposed_face",
    Kind::InvalidValue,
    Class::Error,
    "bad exposed_face",
)
.game("Bad value for exposed_face - should be 'up' or 'down'")
.was("InvalidValue");

/// A `vegetation_patch_feature`'s unknown `surface`.
pub static VEGETATION_SURFACE: CodeDef<0> = CodeDef::new(
    "feature/vegetation_surface",
    Kind::InvalidValue,
    Class::Error,
    "bad surface of a vegetation_patch_feature",
)
.game("Bad value for surface - should be 'ceiling' or 'floor'")
.was("InvalidValue");

// ---- the setters' checks (kept errors: the client logs and keeps the value) ----

/// A negative constant `iterations`; 1 is used.
pub static ITERATIONS_NEGATIVE: CodeDef<0> = CodeDef::new(
    "feature/iterations_negative",
    Kind::InvalidValue,
    Class::KeptError,
    "negative scatter iterations",
)
.game("Bad value for iterations - should be >= 0")
.was("ScatterParamValue");

/// A constant percent `scatter_chance` outside (0, 100]; 100 is used.
pub static SCATTER_CHANCE_PERCENT: CodeDef<0> = CodeDef::new(
    "feature/scatter_chance_percent",
    Kind::InvalidValue,
    Class::KeptError,
    "scatter_chance outside (0, 100]",
)
.game("Bad value for scatter_chance - should be between 0 and 100 (exclusive)")
.was("ScatterParamValue");

/// A fraction `scatter_chance` whose denominator is not above its numerator; 1 is used.
pub static SCATTER_CHANCE_FRACTION: CodeDef<0> = CodeDef::new(
    "feature/scatter_chance_fraction",
    Kind::InvalidValue,
    Class::KeptError,
    "scatter_chance denominator not above its numerator",
)
.game("Bad value for scatter_chance - denominator should be greater than the numerator")
.was("ScatterParamValue");

/// An absent or empty coordinate; the argument is its key.
pub static MUST_BE_PROVIDED: CodeDef<1> = CodeDef::new(
    "feature/must_be_provided",
    Kind::InvalidValue,
    Class::KeptError,
    "the coordinate {} is absent or empty",
)
.game("{} must be provided, either as a Molang expression string or as a sub-object")
.was("ScatterParamValue");

/// A `step_size` beside a distribution that is no grid.
pub static STEP_SIZE_NOT_GRID: CodeDef<0> = CodeDef::new(
    "feature/step_size_not_grid",
    Kind::InvalidValue,
    Class::KeptError,
    "step_size outside a grid distribution",
)
.game("step_size is only valid when distribution is fixed_grid or jittered_grid")
.was("ScatterParamValue");

/// A `grid_offset` beside a distribution that is no grid.
pub static GRID_OFFSET_NOT_GRID: CodeDef<0> = CodeDef::new(
    "feature/grid_offset_not_grid",
    Kind::InvalidValue,
    Class::KeptError,
    "grid_offset outside a grid distribution",
)
.game("grid_offset is only valid when distribution is fixed_grid or jittered_grid")
.was("ScatterParamValue");

/// A constant `extent` whose maximum is not above its minimum.
pub static EXTENT_ORDER: CodeDef<0> = CodeDef::new(
    "feature/extent_order",
    Kind::InvalidValue,
    Class::KeptError,
    "an extent whose maximum is not above its minimum",
)
.game("Bad value for extent - maximum must be greater than minimum")
.was("ScatterParamValue");

/// The trunk setter's text for a negative `height_modifier` larger than the trunk height.
pub static INVERTED_TRUNK: CodeDef<0> = CodeDef::new(
    "feature/inverted_trunk",
    Kind::InvalidValue,
    Class::KeptError,
    "a negative height_modifier larger than the trunk height",
)
.game("A negative 'height_modifier' cannot have greater magnitude than 'trunk_height:range_min' (trunk would invert!)")
.was("InvalidValue");

/// The cherry trunk setter's text for a start offset of one value.
pub static CHERRY_START_OFFSET: CodeDef<0> = CodeDef::new(
    "feature/cherry_start_offset",
    Kind::InvalidValue,
    Class::KeptError,
    "branch_start_offset_from_top holds one value",
)
.game("branch_start_offset_from_top range must contain at least 2 values because branches must start at different heights.")
.was("InvalidValue");

/// The simple canopy setter's text for `canopy_offset.min > max`.
pub static CANOPY_OFFSET: CodeDef<0> = CodeDef::new(
    "feature/canopy_offset",
    Kind::InvalidValue,
    Class::KeptError,
    "canopy_offset min above max",
)
.game("'min' must be <= 'max'")
.was("InvalidValue");

/// "The max values of the Bounding Box must be greater than the min Values."
pub static BOUNDING_BOX: CodeDef<0> = CodeDef::new(
    "feature/bounding_box",
    Kind::InvalidValue,
    Class::KeptError,
    "a bounding box maximum not above its minimum",
)
.game("The max values of the Bounding Box must be greater than the min Values.")
.was("InvalidValue");

/// "Ragged Factor values must be [0,1] and min must be less than max."
pub static RAGGED_RANGE: CodeDef<0> = CodeDef::new(
    "feature/ragged_range",
    Kind::InvalidValue,
    Class::KeptError,
    "beard raggedness outside [0, 1] or min not below max",
)
.game("Ragged Factor values must be [0,1] and min must be less than max.")
.was("InvalidValue");

/// "Ragged Factor min must be leess than max" (sic).
pub static RAGGED_ORDER: CodeDef<0> = CodeDef::new(
    "feature/ragged_order",
    Kind::InvalidValue,
    Class::KeptError,
    "beard raggedness min not below max",
)
.game("Ragged Factor min must be leess than max")
.was("InvalidValue");

/// A `rect_layout`'s `ratio_of_empty_space` of 1 or above.
pub static EMPTY_SPACE: CodeDef<0> = CodeDef::new(
    "feature/empty_space",
    Kind::InvalidValue,
    Class::KeptError,
    "ratio_of_empty_space of 1 or above",
)
.game("100% or above empty space is nonsensical for this feature.")
.was("InvalidValue");

/// A `height_difference_filter_feature` without any bound; the feature stays registered.
pub static NO_HEIGHT_DIFFERENCES: CodeDef<0> = CodeDef::new(
    "feature/no_height_differences",
    Kind::MissingRequired,
    Class::KeptError,
    "a height_difference_filter_feature without a bound",
)
.game("height_difference_filter_feature: No height differences specified")
.was("MissingRequiredField");

/// A carver's float range array of another length than two; the range then fails its
/// validation, which fails the schema.
pub static FLOAT_RANGE_ARRAY: CodeDef<0> = CodeDef::new(
    "feature/float_range_array",
    Kind::InvalidValue,
    Class::Error,
    "a float range array without 2 values",
)
.game("Float range array was not parsed. Float range arrays should have 2 values.")
.was("InvalidValue");

/// Every definition of this table, for the id registry.
pub static ALL: &[&CodeMeta] = &[
    INVALID_JSON.meta(),
    MISSING_IDENTIFIER.meta(),
    EMPTY_IDENTIFIER.meta(),
    NOT_NAMESPACED.meta(),
    NOT_MINECRAFT.meta(),
    INTERNAL_TYPE.meta(),
    DUPLICATE.meta(),
    RANGE_MIN_OUT_OF_RANGE.meta(),
    RANGE_MAX_OUT_OF_RANGE.meta(),
    SNAP_SURFACE.meta(),
    GROWTH_DIRECTION.meta(),
    EXPOSED_FACE.meta(),
    VEGETATION_SURFACE.meta(),
    ITERATIONS_NEGATIVE.meta(),
    SCATTER_CHANCE_PERCENT.meta(),
    SCATTER_CHANCE_FRACTION.meta(),
    MUST_BE_PROVIDED.meta(),
    STEP_SIZE_NOT_GRID.meta(),
    GRID_OFFSET_NOT_GRID.meta(),
    EXTENT_ORDER.meta(),
    INVERTED_TRUNK.meta(),
    CHERRY_START_OFFSET.meta(),
    CANOPY_OFFSET.meta(),
    BOUNDING_BOX.meta(),
    RAGGED_RANGE.meta(),
    RAGGED_ORDER.meta(),
    EMPTY_SPACE.meta(),
    NO_HEIGHT_DIFFERENCES.meta(),
    FLOAT_RANGE_ARRAY.meta(),
];
