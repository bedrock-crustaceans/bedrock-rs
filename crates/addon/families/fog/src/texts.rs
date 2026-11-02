//! The game's content-log texts for fog files, byte for byte. All of them go to the Rendering log
//! area unless noted.

/// `Json::Reader::parse` failed; the argument is the reader's messages.
pub(crate) const PARSE_ERROR: &str = "parse error:\n{}";
/// The file's content was empty.
pub(crate) const EMPTY_RESOURCE: &str = "The raw Json in was empty after getting resources.";
/// The text parsed to an empty value.
pub(crate) const EMPTY_AFTER_PARSE: &str = "The raw Json was empty after parse.";
/// `parseFogDefinitionIdentifier` failed.
pub(crate) const NO_IDENTIFIER: &str = "Could not parse Fog Settings Identifier";
/// `format_version` absent or not a version (a Warning in the game).
pub(crate) const INVALID_VERSION: &str = "Invalid 'format_version' in fog settings JSON";
/// `format_version` below 1.16.100 (a Warning in the game).
pub(crate) const UNSUPPORTED_VERSION: &str =
    "Unsupported 'format_version' value in fog settings JSON";
/// Either parser refused the document.
pub(crate) const FAILED_TO_VALIDATE: &str = "Failed to validate fog schema";
/// The `minecraft` namespace in a pack that is not the base game's.
pub(crate) const MINECRAFT_NAMESPACE: &str =
    "Invalid fog identifier namespace. \"minecraft\" namespace cannot be used.";

// ---- the strict schema (below 1.26.20) ----

/// A colour string that is not `#rrggbb`.
pub(crate) const COLOR_FORMAT: &str = "Color Value was not in the correct format as Alpha is ignored for these colors. Please use a hex value with 6 characters. EX. #FFFFFF";
/// A colour array with fewer than three elements (the parse pass's text, log area Json).
pub(crate) const COLOR_SHAPE: &str = "Expected [ r, g, b ] or \"#RRGGBB\".";
/// A `fog_color` array element that is not integral (log area Json); the component is 255.
pub(crate) const INTEGRAL: &str = "Expected Integral Number.";
/// A `fog_color` array element `parseValue<float>` cannot read (log area Json); the schema's
/// validation fails.
pub(crate) const EXPECTED_NUMBER: &str = "Expected Number.";
/// The two `render_distance_type` patterns as the string node prints them.
pub(crate) const DISTANCE_PATTERNS: &str = "fixed or render";

// ---- the cereal parser's validation lambda (1.26.20) ----

/// The document's `format_version` text is no version.
pub(crate) const BAD_FORMAT_VERSION: &str = "Invalid format version.";
/// `fog_start` is above `fog_end`.
pub(crate) const END_BELOW_START: &str = "Value for 'fog_end' was less than 'fog_start', 'fog_start' has to be less than or equal to 'fog_end'.";
/// A `render` distance starts above 1 (a warning in the game).
pub(crate) const RENDER_START_ABOVE: &str =
    "Value for 'fog_start' should not be greater than 1 when using 'render' distance type.";
/// A `render` distance starts below -1 (a warning in the game).
pub(crate) const RENDER_START_BELOW: &str =
    "Value for 'fog_start' should not be less than -1 when using 'render' distance type.";
/// A `render` distance ends above 1 (a warning in the game).
pub(crate) const RENDER_END_ABOVE: &str =
    "Value for 'fog_end' should not be greater than 1 when using 'render' distance type.";
/// `mid_seconds` is not below `max_seconds`, and they are not both 0.
pub(crate) const MID_BELOW_MAX: &str =
    "Value for 'mid_seconds' should be less than 'max_seconds' unless they are both set to 0.";
/// A height is written beside `uniform: true`.
pub(crate) const UNIFORM_HEIGHTS: &str = "Values for 'max_density_height' and 'zero_density_height' are not used when 'uniform' equals true.";
/// An entry that is not uniform has no `max_density_height`.
pub(crate) const MAX_HEIGHT_REQUIRED: &str =
    "Value for 'max_density_height' is required unless 'uniform' equals true.";
/// An entry that is not uniform has no `zero_density_height`.
pub(crate) const ZERO_HEIGHT_REQUIRED: &str =
    "Value for 'zero_density_height' is required unless 'uniform' equals true.";
/// `zero_density_height` is below `max_density_height`.
pub(crate) const ZERO_BELOW_MAX: &str =
    "Value for 'zero_density_height' has to be greater than or equal to 'max_density_height'.";
