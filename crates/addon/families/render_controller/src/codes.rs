//! The `render_controller` family's codes: the game's content-log lines, byte for byte, each a definition
//! beside the rules that raise it. Each [`was`](CodeDef::was) the legacy code its sites raised,
//! so a pin or a filter on the code's name keeps matching.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

/// `error parsing %s:\n%s`: the file's path and the JSON reader's messages.
pub static ERROR_PARSING: CodeDef<2> = CodeDef::new(
    "render_controller/error_parsing",
    Kind::Syntax,
    Class::Error,
    "malformed input",
)
.game("error parsing {}:\n{}")
.was("MalformedInput");

/// The text for a controller without a `textures` member.
pub static MUST_CONTAIN_TEXTURES: CodeDef<0> = CodeDef::new(
    "render_controller/must_contain_textures",
    Kind::MissingRequired,
    Class::Error,
    "missing required field",
)
.game("Render controller must contain textures.")
.was("MissingRequiredField");

/// `MissingFormatVersion`: the line the core's cereal reader words (`REQUIRED_CHILD_TEXT`).
pub static REQUIRED_CHILD: CodeDef<1> = CodeDef::new(
    "render_controller/required_child",
    Kind::FormatVersion,
    Class::Error,
    "no format_version",
)
.game(bedrock_addon_core::diagnostic::cereal::REQUIRED_CHILD_TEXT)
.was("MissingFormatVersion");

/// `UnsupportedVersion`: the line the core's cereal reader words (`UNKNOWN_CHILD_OPTION_TEXT`).
pub static UNKNOWN_OPTION_TYPE: CodeDef<1> = CodeDef::new(
    "render_controller/unknown_option_type",
    Kind::UnsupportedVersion,
    Class::Error,
    "unsupported format_version",
)
.game(bedrock_addon_core::diagnostic::cereal::UNKNOWN_CHILD_OPTION_TEXT)
.was("UnsupportedVersion");

/// `UnsupportedVersion`: the game's line `Version {} greater than max allowed version ({})`.
pub static ABOVE_MAX_VERSION: CodeDef<2> = CodeDef::new(
    "render_controller/above_max_version",
    Kind::UnsupportedVersion,
    Class::Error,
    "unsupported format_version",
)
.game("Version {} greater than max allowed version ({})")
.was("UnsupportedVersion");

/// Every definition of this module, for the id registry.
pub static ALL: &[&CodeMeta] = &[
    ERROR_PARSING.meta(),
    MUST_CONTAIN_TEXTURES.meta(),
    REQUIRED_CHILD.meta(),
    UNKNOWN_OPTION_TYPE.meta(),
    ABOVE_MAX_VERSION.meta(),
];
