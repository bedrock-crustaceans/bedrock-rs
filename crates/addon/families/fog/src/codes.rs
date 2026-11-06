//! The fog family's codes: the game's content-log lines for fog files, byte for byte, each a
//! definition beside the rules that raise it. All of them go to the Rendering log area unless
//! noted. Each [`was`](CodeDef::was) the legacy code its sites raised, so a pin or a filter on
//! the code's name keeps matching.

use bedrock_addon_core::diagnostic::{Class, CodeDef, Kind};

// ---- the gates of `registerFogFromJson` ----

/// `Json::Reader::parse` failed; the argument is the reader's messages. The finding carries
/// them again as its `DiagnosticDetail::Syntax`.
pub static PARSE_ERROR: CodeDef<1> = CodeDef::new(
    "fog/parse_error",
    Kind::Syntax,
    Class::Error,
    "parse error: {}",
)
.game("parse error:\n{}")
.was("MalformedInput");

/// The file's content was empty.
pub static EMPTY_RESOURCE: CodeDef<0> = CodeDef::new(
    "fog/empty_resource",
    Kind::Syntax,
    Class::Error,
    "empty fog file",
)
.game("The raw Json in was empty after getting resources.")
.was("EmptyDocument");

/// The text parsed to an empty value.
pub static EMPTY_AFTER_PARSE: CodeDef<0> = CodeDef::new(
    "fog/empty_after_parse",
    Kind::Syntax,
    Class::Error,
    "the fog file parses to an empty value",
)
.game("The raw Json was empty after parse.")
.was("EmptyDocument");

/// `parseFogDefinitionIdentifier` failed.
pub static NO_IDENTIFIER: CodeDef<0> = CodeDef::new(
    "fog/no_identifier",
    Kind::MissingRequired,
    Class::Error,
    "no fog identifier",
)
.game("Could not parse Fog Settings Identifier")
.was("MissingIdentifier");

/// `format_version` absent or not a version (a Warning in the game; the file is skipped).
pub static INVALID_VERSION: CodeDef<0> = CodeDef::new(
    "fog/invalid_version",
    Kind::FormatVersion,
    Class::Error,
    "no fog format_version",
)
.game("Invalid 'format_version' in fog settings JSON")
.was("MissingFormatVersion");

/// `format_version` below 1.16.100 (a Warning in the game; the file is skipped).
pub static UNSUPPORTED_VERSION: CodeDef<0> = CodeDef::new(
    "fog/unsupported_version",
    Kind::UnsupportedVersion,
    Class::Error,
    "fog format_version below 1.16.100",
)
.game("Unsupported 'format_version' value in fog settings JSON")
.was("UnsupportedVersion");

/// Either parser refused the document: the closing line.
pub static FAILED_TO_VALIDATE: CodeDef<0> = CodeDef::new(
    "fog/failed_to_validate",
    Kind::Closing,
    Class::Error,
    "fog document failed schema validation",
)
.game("Failed to validate fog schema")
.was("DocumentRejected");

/// The `minecraft` namespace in a pack that is not the base game's.
pub static MINECRAFT_NAMESPACE: CodeDef<0> = CodeDef::new(
    "fog/minecraft_namespace",
    Kind::Identifier,
    Class::Error,
    "the minecraft namespace in a fog identifier",
)
.game("Invalid fog identifier namespace. \"minecraft\" namespace cannot be used.")
.was("NamespaceRule");

// ---- the strict schema (below 1.26.20) ----

/// A colour string that is not `#rrggbb`.
pub static COLOR_FORMAT: CodeDef<0> = CodeDef::new(
    "fog/color_format",
    Kind::InvalidValue,
    Class::Error,
    "fog colour is not #rrggbb",
)
.game("Color Value was not in the correct format as Alpha is ignored for these colors. Please use a hex value with 6 characters. EX. #FFFFFF")
.was("InvalidValue");

/// A colour array with fewer than three elements (the parse pass's text, log area Json).
pub static COLOR_SHAPE: CodeDef<0> = CodeDef::new(
    "fog/color_shape",
    Kind::InvalidValue,
    Class::Error,
    "fog colour array with fewer than three elements",
)
.game("Expected [ r, g, b ] or \"#RRGGBB\".")
.was("InvalidValue");

/// A `fog_color` array element that is not integral (log area Json); the component is 255 and
/// the file loads: a kept error.
pub static INTEGRAL: CodeDef<0> = CodeDef::new(
    "fog/integral",
    Kind::InvalidValue,
    Class::KeptError,
    "fog colour element is not integral",
)
.game("Expected Integral Number.")
.was("InvalidValue");

/// A `fog_color` array element `parseValue<float>` cannot read (log area Json); the schema's
/// validation fails.
pub static EXPECTED_NUMBER: CodeDef<0> = CodeDef::new(
    "fog/expected_number",
    Kind::WrongType,
    Class::Error,
    "fog colour element is not a number",
)
.game("Expected Number.")
.was("InvalidType");

// ---- the cereal parser's validation lambda (1.26.20) ----

/// The document's `format_version` text is no version.
pub static BAD_FORMAT_VERSION: CodeDef<0> = CodeDef::new(
    "fog/bad_format_version",
    Kind::InvalidValue,
    Class::Error,
    "fog format_version is no version",
)
.game("Invalid format version.")
.was("InvalidValue");

/// `fog_start` is above `fog_end`.
pub static END_BELOW_START: CodeDef<0> = CodeDef::new(
    "fog/end_below_start",
    Kind::InvalidValue,
    Class::Error,
    "fog_end below fog_start",
)
.game("Value for 'fog_end' was less than 'fog_start', 'fog_start' has to be less than or equal to 'fog_end'.")
.was("InvalidValue");

/// A `render` distance starts above 1 (a warning in the game).
pub static RENDER_START_ABOVE: CodeDef<0> = CodeDef::new(
    "fog/render_start_above",
    Kind::InvalidValue,
    Class::Warning,
    "render fog_start above 1",
)
.game("Value for 'fog_start' should not be greater than 1 when using 'render' distance type.")
.was("InvalidValue");

/// A `render` distance starts below -1 (a warning in the game).
pub static RENDER_START_BELOW: CodeDef<0> = CodeDef::new(
    "fog/render_start_below",
    Kind::InvalidValue,
    Class::Warning,
    "render fog_start below -1",
)
.game("Value for 'fog_start' should not be less than -1 when using 'render' distance type.")
.was("InvalidValue");

/// A `render` distance ends above 1 (a warning in the game).
pub static RENDER_END_ABOVE: CodeDef<0> = CodeDef::new(
    "fog/render_end_above",
    Kind::InvalidValue,
    Class::Warning,
    "render fog_end above 1",
)
.game("Value for 'fog_end' should not be greater than 1 when using 'render' distance type.")
.was("InvalidValue");

/// `mid_seconds` is not below `max_seconds`, and they are not both 0.
pub static MID_BELOW_MAX: CodeDef<0> = CodeDef::new(
    "fog/mid_below_max",
    Kind::InvalidValue,
    Class::Error,
    "mid_seconds not below max_seconds",
)
.game("Value for 'mid_seconds' should be less than 'max_seconds' unless they are both set to 0.")
.was("InvalidValue");

/// A height is written beside `uniform: true`.
pub static UNIFORM_HEIGHTS: CodeDef<0> = CodeDef::new(
    "fog/uniform_heights",
    Kind::InvalidValue,
    Class::Error,
    "density heights beside uniform: true",
)
.game("Values for 'max_density_height' and 'zero_density_height' are not used when 'uniform' equals true.")
.was("InvalidValue");

/// An entry that is not uniform has no `max_density_height`.
pub static MAX_HEIGHT_REQUIRED: CodeDef<0> = CodeDef::new(
    "fog/max_height_required",
    Kind::InvalidValue,
    Class::Error,
    "no max_density_height",
)
.game("Value for 'max_density_height' is required unless 'uniform' equals true.")
.was("InvalidValue");

/// An entry that is not uniform has no `zero_density_height`.
pub static ZERO_HEIGHT_REQUIRED: CodeDef<0> = CodeDef::new(
    "fog/zero_height_required",
    Kind::InvalidValue,
    Class::Error,
    "no zero_density_height",
)
.game("Value for 'zero_density_height' is required unless 'uniform' equals true.")
.was("InvalidValue");

/// `zero_density_height` is below `max_density_height`.
pub static ZERO_BELOW_MAX: CodeDef<0> = CodeDef::new(
    "fog/zero_below_max",
    Kind::InvalidValue,
    Class::Error,
    "zero_density_height below max_density_height",
)
.game("Value for 'zero_density_height' has to be greater than or equal to 'max_density_height'.")
.was("InvalidValue");
