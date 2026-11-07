//! The `function` family's codes: the game's content-log lines, byte for byte, each a definition
//! beside the rules that raise it. Each id is `function/<name>`, the name its constant's in lower
//! case.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Disposition, Kind};

/// The line before a function's line errors, when the load is strict; the argument is the
/// function's name.
pub static FAILED_HEADER: CodeDef<1> = CodeDef::new(
    "function/failed_header",
    Kind::Closing,
    Class::Error,
    "function {} failed to load",
)
.game("Function {} failed to load correctly with error(s):");

/// The same line when the load is not strict: the function is kept; the argument is the function's
/// name.
pub static FAILED_HEADER_DROPPED: CodeDef<1> = CodeDef::new(
    "function/failed_header_dropped",
    Kind::UnknownMember(Disposition::Dropped),
    Class::Error,
    "function {} failed to load",
)
.game("Function {} failed to load correctly with error(s):");

/// The first word of a line is no command. The arguments are the line number and the first word.
pub static UNKNOWN_COMMAND: CodeDef<2> = CodeDef::new(
    "function/unknown_command",
    Kind::UnresolvedReference,
    Class::Error,
    "unknown command on line {}: {}",
)
.game("Error on line {}: unknown or invalid command '{}'");

/// The same, for a first word that starts with `//`. The arguments are the line number and the
/// first word.
pub static UNKNOWN_COMMAND_COMMENT: CodeDef<2> = CodeDef::new(
    "function/unknown_command_comment",
    Kind::UnresolvedReference,
    Class::Error,
    "unknown command on line {}: {}",
)
.game("Error on line {}: unknown or invalid command '{}' (If you intended to make a comment use '#' instead of '//')");

/// The same, for a first word of two bytes or more that starts with `/`. The arguments are the line
/// number and the first word.
pub static UNKNOWN_COMMAND_SLASH: CodeDef<2> = CodeDef::new(
    "function/unknown_command_slash",
    Kind::UnresolvedReference,
    Class::Error,
    "unknown command on line {}: {}",
)
.game("Error on line {}: unknown or invalid command '{}' (Commands in functions may not begin with '/')");

/// A tick list that is not JSON; the arguments are the file name (always `tick.json`) and the
/// reader's errors, which ride again as the finding's syntax detail.
pub static TICK_NOT_JSON: CodeDef<2> = CodeDef::new(
    "function/tick_not_json",
    Kind::Syntax,
    Class::Error,
    "{} is not JSON: {}",
)
.game("File {} failed to load with error(s) {}");

/// A tick entry that names no loaded function; the argument is the name.
pub static TICK_NOT_FOUND: CodeDef<1> = CodeDef::new(
    "function/tick_not_found",
    Kind::UnresolvedReference,
    Class::Warning,
    "tick function {} not found",
)
.game("Function listed in tick.json with name {} not found.");

/// A tick list whose `values` is missing, `null` or a root that is no object; the argument is
/// `values`.
pub static TICK_NO_VALUES: CodeDef<1> = CodeDef::new(
    "function/tick_no_values",
    Kind::MissingRequired,
    Class::Error,
    "tick.json has no {}",
)
.game("Field '{}' not found in tick.json or was found but is not an array, couldn't load tick functions.");

/// A tick list whose `values` is no array; the argument is `values`.
pub static TICK_VALUES_NOT_ARRAY: CodeDef<1> = CodeDef::new(
    "function/tick_values_not_array",
    Kind::WrongType,
    Class::Error,
    "tick.json {} is no array",
)
.game("Field '{}' not found in tick.json or was found but is not an array, couldn't load tick functions.");

/// Every definition of this module.
pub static ALL: &[&CodeMeta] = &[
    FAILED_HEADER.meta(),
    FAILED_HEADER_DROPPED.meta(),
    UNKNOWN_COMMAND.meta(),
    UNKNOWN_COMMAND_COMMENT.meta(),
    UNKNOWN_COMMAND_SLASH.meta(),
    TICK_NOT_JSON.meta(),
    TICK_NOT_FOUND.meta(),
    TICK_NO_VALUES.meta(),
    TICK_VALUES_NOT_ARRAY.meta(),
];
