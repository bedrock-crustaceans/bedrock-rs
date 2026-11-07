//! The `animation_controller` family's codes: the game's content-log lines, byte for byte, each a
//! definition beside the rules that raise it. Each id is `animation_controller/<name>`, the name
//! its constant's in lower case.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

/// `MissingFormatVersion`: the game's line `Error: ill-formatted/missing "format_version" field. Skipping.`.
pub static ILL_FORMATTED_MISSING_FORMAT_VERSION: CodeDef<0> = CodeDef::new(
    "animation_controller/ill_formatted_missing_format_version",
    Kind::FormatVersion,
    Class::Error,
    "no format_version",
)
.game("Error: ill-formatted/missing \"format_version\" field. Skipping.");

/// `UnsupportedVersion`: the game's line `Error: ill-formatted/missing "format_version" field. Skipping.`.
pub static ILL_FORMATTED_UNSUPPORTED_VERSION: CodeDef<0> = CodeDef::new(
    "animation_controller/ill_formatted_unsupported_version",
    Kind::UnsupportedVersion,
    Class::Error,
    "unsupported format_version",
)
.game("Error: ill-formatted/missing \"format_version\" field. Skipping.");

/// The "exceeds game version" text (two spaces before "Skipping", as the game writes it).
pub static EXCEEDS_GAME: CodeDef<2> = CodeDef::new(
    "animation_controller/exceeds_game",
    Kind::UnsupportedVersion,
    Class::Error,
    "unsupported format_version",
)
.game("Error: animation controller file format version: {} exceeds game version: {}.  Skipping.");

/// `UnsupportedVersion`: the game's line `Unknown/unsupported version value specified in "format_version" field: {}`.
pub static UNKNOWN_FORMAT_VERSION: CodeDef<1> = CodeDef::new(
    "animation_controller/unknown_format_version",
    Kind::UnsupportedVersion,
    Class::Error,
    "unsupported format_version",
)
.game("Unknown/unsupported version value specified in \"format_version\" field: {}");

/// `UnsupportedVersion`: the line the core's cereal reader words (`UNKNOWN_CHILD_OPTION_TEXT`).
pub static UNKNOWN_OPTION_TYPE: CodeDef<1> = CodeDef::new(
    "animation_controller/unknown_option_type",
    Kind::UnsupportedVersion,
    Class::Error,
    "unsupported format_version",
)
.game(bedrock_addon_core::diagnostic::cereal::UNKNOWN_CHILD_OPTION_TEXT);

/// `Error: state %s not found - defaulting to using state %d`.
pub static STATE_NOT_FOUND: CodeDef<2> = CodeDef::new(
    "animation_controller/state_not_found",
    Kind::UnresolvedReference,
    Class::Lint,
    "unresolved reference",
)
.game("Error: state {} not found - defaulting to using state {}");

/// Every definition of this module, for the id registry.
pub static ALL: &[&CodeMeta] = &[
    ILL_FORMATTED_MISSING_FORMAT_VERSION.meta(),
    ILL_FORMATTED_UNSUPPORTED_VERSION.meta(),
    EXCEEDS_GAME.meta(),
    UNKNOWN_FORMAT_VERSION.meta(),
    UNKNOWN_OPTION_TYPE.meta(),
    STATE_NOT_FOUND.meta(),
];
