//! Vanilla's content-log texts of the camera documents (vanilla's spelling kept).

use bedrock_addon_core::diagnostic::VanillaText;

// Preset file texts: logged while a preset file is read.
/// A member the preset schema does not bind, at any format version.
pub(crate) const UNRECOGNIZED_FIELD: &str = "Unrecognized field on camera preset!";
/// The closing line of a preset file that failed to load.
pub(crate) const FAILED_PRESETS: &str = "Failed to load camera presets";
/// `identifier` or `inherit_from` fails `^(\w+:\w+)?$`.
pub(crate) const IDENTIFIER: &str = "Identifier requires a namespace and name separated by a ':' and each must have at least one alphanumeric or underscore character.";
/// `horizontal_rotation_limit`: x + y above 360.
pub(crate) const VECTOR_TOTAL: &str =
    "Vector elements added together cannot be greater than the defined constraint!";
/// `vertical_rotation_limit`: x equals y.
pub(crate) const VECTOR_EQUAL: &str = "Vector's elements cannot equal each other!";
/// `vertical_rotation_limit`: x not below y.
pub(crate) const VECTOR_ORDER: &str = "Vector's x must be less than y!";

// Texts logged when the presets of every pack are checked and an invalid one is discarded.
/// A preset named like a vanilla camera sets `radius`.
pub(crate) const ILLEGAL_OVERRIDE: &str =
    "Illegal override: Preset '{}' cannot change vanilla camera settings. It will be discarded.";
/// `inherit_from` names a vanilla camera outside `ALLOWED_MC_INHERIT`.
pub(crate) const ILLEGAL_INHERITANCE: &str =
    "Illegal inheritance: Preset '{}' cannot inherit from '{}'. It will be discarded.";
/// `inherit_from` names no loaded preset.
pub(crate) const UNKNOWN_PARENT: &str =
    "Preset '{}' inherits from unknown preset '{}' and will be discarded.";
/// The `inherit_from` chain comes back to a preset it passed.
pub(crate) const CIRCULAR: &str =
    "Circular reference: Preset '{}' inherits from '{}' more than once and will be discarded.";

// The next three texts are logged by the game's client when it sets a preset up; the crate
// reports them at load, as warnings.
/// A preset that is no vanilla camera and inherits from nothing.
pub(crate) const MISSING_INHERIT: &str =
    "Could not set up Camera Preset with name '{}'. Are you missing 'inherit_from'?";
/// `yaw_limits_min` not below `yaw_limits_max`.
pub(crate) const YAW_ORDER: &str =
    "Yaw limit min must be smaller than the yaw limit's max for preset {}!";
/// A preset named like a vanilla camera sets `starting_rot`.
pub(crate) const VANILLA_STARTING_ROT: &str =
    "Preset {} cannot change the starting rotation of a vanilla camera";

// Texts logged while a camera list file is read.
/// A list without presets.
pub(crate) const LIST_EMPTY: &str =
    "Camera List '{}' must contain at least one Camera Preset identifier, it will not be loaded.";
/// A list naming a preset twice.
pub(crate) const LIST_DUPLICATE: &str =
    "Camera List '{}' references Camera Preset '{}' more than once, it will not be loaded.";
/// A list naming a preset that is not loaded.
pub(crate) const LIST_UNKNOWN: &str =
    "Camera List '{}' references unknown Camera Preset '{}', it will not be loaded.";
/// A list file that is not JSON: the file and the reader's message.
pub(crate) const LIST_NOT_JSON: &str = "Camera List file {} is not valid JSON: {}";
/// The closing line of a list file that failed to load.
pub(crate) const LIST_INVALID: &str = "Invalid Camera List file";

// Texts logged while a spline file is read.
/// A spline `name` that fails `^\w+:\w+$`.
pub(crate) const SPLINE_NAME: &str = "\"name\" must be in \"namespace:name\" format.";
/// A negative `total_time`.
pub(crate) const SPLINE_TOTAL_TIME: &str = "Total time cannot be a negative value.";
/// A `spline_type` other than the two names.
pub(crate) const SPLINE_TYPE: &str = "\"spline_type\" must be either \"catmullrom\" or \"linear\".";
/// A `progress` outside 0 to 1.
pub(crate) const SPLINE_PROGRESS: &str = "The progress values must be between 0.0 and 1.0.";
/// A negative progress key frame `time`.
pub(crate) const SPLINE_PROGRESS_TIME: &str = "Time cannot be a negative value.";
/// A negative rotation key frame `time` (vanilla's text has no final period).
pub(crate) const SPLINE_ROTATION_TIME: &str = "Time cannot be a negative value";
/// The closing line of a spline file that failed to load.
pub(crate) const SPLINES_FAILED: &str = "Custom camera splines not loaded correctly.";

// Texts logged while a camera entity document is read.
/// A camera entity `description.identifier` that fails `^\w+:\w+$`.
pub(crate) const ENTITY_IDENTIFIER: &str =
    "\"identifier\" must contain a namespace and a name, separated by a colon";
/// `minecraft:target.horizontal_rotation_limit`: a component outside 0 to 360.
pub(crate) const TARGET_HORIZONTAL_RANGE: &str =
    "x and y components of the horizontal rotation must be in the [0, 360] range.";
/// `minecraft:target.horizontal_rotation_limit`: x + y above 360.
pub(crate) const TARGET_HORIZONTAL_SUM: &str =
    "The sum of the [x, y] horizontal rotation limits must not exceed 360.0 degrees.";

/// Every template with arguments and its number of arguments, for the arity test.
#[cfg(test)]
pub(crate) const FORMATTED: &[(&str, usize)] = &[
    (ILLEGAL_OVERRIDE, 1),
    (ILLEGAL_INHERITANCE, 2),
    (UNKNOWN_PARENT, 2),
    (CIRCULAR, 2),
    (MISSING_INHERIT, 1),
    (YAW_ORDER, 1),
    (VANILLA_STARTING_ROT, 1),
    (LIST_EMPTY, 1),
    (LIST_DUPLICATE, 2),
    (LIST_UNKNOWN, 2),
    (LIST_NOT_JSON, 2),
];

/// `template` with `args` in its `{}` placeholders.
#[allow(
    clippy::unnecessary_wraps,
    reason = "every caller fills a code's optional text"
)]
pub(crate) fn formatted(template: &'static str, args: &[&str]) -> Option<VanillaText> {
    Some(VanillaText::Formatted {
        template,
        args: args.iter().map(|arg| (*arg).to_owned()).collect(),
    })
}
