//! The `camera` family's codes: the game's content-log lines, byte for byte, each a definition
//! beside the rules that raise it. Each id is `camera/<name>`, the name its constant's in lower
//! case.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Disposition, Kind};

/// A member the preset schema does not bind, at any format version.
pub static UNRECOGNIZED_FIELD: CodeDef<0> = CodeDef::new(
    "camera/unrecognized_field",
    Kind::UnknownMember(Disposition::Kept),
    Class::Unknown,
    "unrecognized field",
)
.game("Unrecognized field on camera preset!");

/// The closing line of a preset file that failed to load.
pub static FAILED_PRESETS: CodeDef<0> = CodeDef::new(
    "camera/failed_presets",
    Kind::Closing,
    Class::Error,
    "document failed schema validation",
)
.game("Failed to load camera presets");

/// `identifier` or `inherit_from` fails `^(\w+:\w+)?$`.
pub static IDENTIFIER: CodeDef<0> = CodeDef::new(
    "camera/identifier",
    Kind::InvalidValue,
    Class::Error,
    "invalid value",
)
.game("Identifier requires a namespace and name separated by a ':' and each must have at least one alphanumeric or underscore character.");

/// `horizontal_rotation_limit`: x + y above 360.
pub static VECTOR_TOTAL: CodeDef<0> = CodeDef::new(
    "camera/vector_total",
    Kind::InvalidValue,
    Class::Error,
    "invalid value",
)
.game("Vector elements added together cannot be greater than the defined constraint!");

/// `vertical_rotation_limit`: x equals y.
pub static VECTOR_EQUAL: CodeDef<0> = CodeDef::new(
    "camera/vector_equal",
    Kind::InvalidValue,
    Class::Error,
    "invalid value",
)
.game("Vector's elements cannot equal each other!");

/// `vertical_rotation_limit`: x not below y.
pub static VECTOR_ORDER: CodeDef<0> = CodeDef::new(
    "camera/vector_order",
    Kind::InvalidValue,
    Class::Error,
    "invalid value",
)
.game("Vector's x must be less than y!");

/// A preset named like a vanilla camera sets `radius`.
pub static ILLEGAL_OVERRIDE: CodeDef<1> = CodeDef::new(
    "camera/illegal_override",
    Kind::InvalidValue,
    Class::Error,
    "invalid value",
)
.game("Illegal override: Preset '{}' cannot change vanilla camera settings. It will be discarded.");

/// `inherit_from` names a vanilla camera outside `ALLOWED_MC_INHERIT`.
pub static ILLEGAL_INHERITANCE: CodeDef<2> = CodeDef::new(
    "camera/illegal_inheritance",
    Kind::InvalidReference,
    Class::Error,
    "reference resolves to something this use cannot accept",
)
.game("Illegal inheritance: Preset '{}' cannot inherit from '{}'. It will be discarded.");

/// `inherit_from` names no loaded preset.
pub static UNKNOWN_PARENT: CodeDef<2> = CodeDef::new(
    "camera/unknown_parent",
    Kind::UnresolvedReference,
    Class::Error,
    "unresolved reference",
)
.game("Preset '{}' inherits from unknown preset '{}' and will be discarded.");

/// The `inherit_from` chain comes back to a preset it passed.
pub static CIRCULAR: CodeDef<2> = CodeDef::new(
    "camera/circular",
    Kind::Cycle,
    Class::Error,
    "references form a cycle",
)
.game("Circular reference: Preset '{}' inherits from '{}' more than once and will be discarded.");

/// A preset that is no vanilla camera and inherits from nothing.
pub static MISSING_INHERIT: CodeDef<1> = CodeDef::new(
    "camera/missing_inherit",
    Kind::MissingRequired,
    Class::KeptError,
    "missing required field",
)
.game("Could not set up Camera Preset with name '{}'. Are you missing 'inherit_from'?");

/// `yaw_limits_min` not below `yaw_limits_max`.
pub static YAW_ORDER: CodeDef<1> = CodeDef::new(
    "camera/yaw_order",
    Kind::InvalidValue,
    Class::KeptError,
    "invalid value",
)
.game("Yaw limit min must be smaller than the yaw limit's max for preset {}!");

/// A preset named like a vanilla camera sets `starting_rot`.
pub static VANILLA_STARTING_ROT: CodeDef<1> = CodeDef::new(
    "camera/vanilla_starting_rot",
    Kind::InvalidValue,
    Class::Lint,
    "invalid value",
)
.game("Preset {} cannot change the starting rotation of a vanilla camera");

/// A list without presets.
pub static LIST_EMPTY: CodeDef<1> = CodeDef::new(
    "camera/list_empty",
    Kind::InvalidValue,
    Class::Error,
    "invalid value",
)
.game(
    "Camera List '{}' must contain at least one Camera Preset identifier, it will not be loaded.",
);

/// A list naming a preset twice.
pub static LIST_DUPLICATE: CodeDef<2> = CodeDef::new(
    "camera/list_duplicate",
    Kind::Duplicate,
    Class::Error,
    "duplicate identifier",
)
.game("Camera List '{}' references Camera Preset '{}' more than once, it will not be loaded.");

/// A list naming a preset that is not loaded.
pub static LIST_UNKNOWN: CodeDef<2> = CodeDef::new(
    "camera/list_unknown",
    Kind::UnresolvedReference,
    Class::Error,
    "unresolved reference",
)
.game("Camera List '{}' references unknown Camera Preset '{}', it will not be loaded.");

/// A list file that is not JSON: the file and the reader's message.
pub static LIST_NOT_JSON: CodeDef<2> = CodeDef::new(
    "camera/list_not_json",
    Kind::Syntax,
    Class::Error,
    "malformed input",
)
.game("Camera List file {} is not valid JSON: {}");

/// The closing line of a list file that failed to load.
pub static LIST_INVALID: CodeDef<0> = CodeDef::new(
    "camera/list_invalid",
    Kind::Closing,
    Class::Error,
    "document failed schema validation",
)
.game("Invalid Camera List file");

/// A spline `name` that fails `^\w+:\w+$`.
pub static SPLINE_NAME: CodeDef<0> = CodeDef::new(
    "camera/spline_name",
    Kind::InvalidValue,
    Class::Error,
    "invalid value",
)
.game("\"name\" must be in \"namespace:name\" format.");

/// A negative `total_time`.
pub static SPLINE_TOTAL_TIME: CodeDef<0> = CodeDef::new(
    "camera/spline_total_time",
    Kind::InvalidValue,
    Class::Error,
    "invalid value",
)
.game("Total time cannot be a negative value.");

/// A `spline_type` other than the two names.
pub static SPLINE_TYPE: CodeDef<0> = CodeDef::new(
    "camera/spline_type",
    Kind::InvalidValue,
    Class::Error,
    "invalid value",
)
.game("\"spline_type\" must be either \"catmullrom\" or \"linear\".");

/// A `progress` outside 0 to 1.
pub static SPLINE_PROGRESS: CodeDef<0> = CodeDef::new(
    "camera/spline_progress",
    Kind::InvalidValue,
    Class::Error,
    "invalid value",
)
.game("The progress values must be between 0.0 and 1.0.");

/// A negative progress key frame `time`.
pub static SPLINE_PROGRESS_TIME: CodeDef<0> = CodeDef::new(
    "camera/spline_progress_time",
    Kind::InvalidValue,
    Class::Error,
    "invalid value",
)
.game("Time cannot be a negative value.");

/// A negative rotation key frame `time` (vanilla's text has no final period).
pub static SPLINE_ROTATION_TIME: CodeDef<0> = CodeDef::new(
    "camera/spline_rotation_time",
    Kind::InvalidValue,
    Class::Error,
    "invalid value",
)
.game("Time cannot be a negative value");

/// The closing line of a spline file that failed to load.
pub static SPLINES_FAILED: CodeDef<0> = CodeDef::new(
    "camera/splines_failed",
    Kind::Closing,
    Class::Error,
    "document failed schema validation",
)
.game("Custom camera splines not loaded correctly.");

/// A camera entity `description.identifier` that fails `^\w+:\w+$`.
pub static ENTITY_IDENTIFIER: CodeDef<0> = CodeDef::new(
    "camera/entity_identifier",
    Kind::InvalidValue,
    Class::Error,
    "invalid value",
)
.game("\"identifier\" must contain a namespace and a name, separated by a colon");

/// `minecraft:target.horizontal_rotation_limit`: a component outside 0 to 360.
pub static TARGET_HORIZONTAL_RANGE: CodeDef<0> = CodeDef::new(
    "camera/target_horizontal_range",
    Kind::InvalidValue,
    Class::Error,
    "invalid value",
)
.game("x and y components of the horizontal rotation must be in the [0, 360] range.");

/// `minecraft:target.horizontal_rotation_limit`: x + y above 360.
pub static TARGET_HORIZONTAL_SUM: CodeDef<0> = CodeDef::new(
    "camera/target_horizontal_sum",
    Kind::InvalidValue,
    Class::Error,
    "invalid value",
)
.game("The sum of the [x, y] horizontal rotation limits must not exceed 360.0 degrees.");

/// Every definition of this module, for the id registry.
pub static ALL: &[&CodeMeta] = &[
    UNRECOGNIZED_FIELD.meta(),
    FAILED_PRESETS.meta(),
    IDENTIFIER.meta(),
    VECTOR_TOTAL.meta(),
    VECTOR_EQUAL.meta(),
    VECTOR_ORDER.meta(),
    ILLEGAL_OVERRIDE.meta(),
    ILLEGAL_INHERITANCE.meta(),
    UNKNOWN_PARENT.meta(),
    CIRCULAR.meta(),
    MISSING_INHERIT.meta(),
    YAW_ORDER.meta(),
    VANILLA_STARTING_ROT.meta(),
    LIST_EMPTY.meta(),
    LIST_DUPLICATE.meta(),
    LIST_UNKNOWN.meta(),
    LIST_NOT_JSON.meta(),
    LIST_INVALID.meta(),
    SPLINE_NAME.meta(),
    SPLINE_TOTAL_TIME.meta(),
    SPLINE_TYPE.meta(),
    SPLINE_PROGRESS.meta(),
    SPLINE_PROGRESS_TIME.meta(),
    SPLINE_ROTATION_TIME.meta(),
    SPLINES_FAILED.meta(),
    ENTITY_IDENTIFIER.meta(),
    TARGET_HORIZONTAL_RANGE.meta(),
    TARGET_HORIZONTAL_SUM.meta(),
];
