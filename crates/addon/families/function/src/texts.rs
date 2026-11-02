//! Vanilla's content-log texts of the function loader, as the loader quotes them. `{}`
//! is where `VanillaText::Formatted` puts each argument.

use bedrock_addon_core::diagnostic::VanillaText;

// Texts logged while a function file is loaded.
/// The line before a function's line errors; the argument is the function's name.
pub(super) const FAILED_HEADER: &str = "Function {} failed to load correctly with error(s):";

/// The unknown-command text: the line number and the first word.
macro_rules! unknown_command {
    ($hint:literal) => {
        concat!("Error on line {}: unknown or invalid command '{}'", $hint)
    };
}

/// The first word of a line is no command.
pub(super) const UNKNOWN_COMMAND: &str = unknown_command!("");
/// The same, for a first word that starts with `//`.
pub(super) const UNKNOWN_COMMAND_COMMENT: &str =
    unknown_command!(" (If you intended to make a comment use '#' instead of '//')");
/// The same, for a first word of two bytes or more that starts with `/`.
pub(super) const UNKNOWN_COMMAND_SLASH: &str =
    unknown_command!(" (Commands in functions may not begin with '/')");

// Texts logged while `tick.json` is loaded.
/// A tick list that is not JSON: the file name (always `tick.json`) and the reader's errors.
pub(super) const TICK_NOT_JSON: &str = "File {} failed to load with error(s) {}";
/// A tick entry that names no loaded function; the argument is the name.
pub(super) const TICK_NOT_FOUND: &str = "Function listed in tick.json with name {} not found.";
/// A tick list whose `values` is missing, `null` or no array; the argument is `values`.
pub(super) const TICK_NO_VALUES: &str = "Field '{}' not found in tick.json or was found but is not an array, couldn't load tick functions.";

/// A vanilla text with arguments: `template` with each `{}` replaced by the next argument.
/// Every formatted text of the module is built here, so a test can check the arity.
#[allow(
    clippy::unnecessary_wraps,
    reason = "every call site fills an `Option<VanillaText>` field"
)]
pub(super) fn formatted<const N: usize>(
    template: &'static str,
    args: [String; N],
) -> Option<VanillaText> {
    Some(VanillaText::Formatted {
        template,
        args: args.into(),
    })
}

/// Every template and the number of arguments its call site gives it, for the arity test.
#[cfg(test)]
pub(super) const FORMATTED: &[(&str, usize)] = &[
    (FAILED_HEADER, 1),
    (UNKNOWN_COMMAND, 2),
    (UNKNOWN_COMMAND_COMMENT, 2),
    (UNKNOWN_COMMAND_SLASH, 2),
    (TICK_NOT_JSON, 2),
    (TICK_NOT_FOUND, 1),
    (TICK_NO_VALUES, 1),
];
