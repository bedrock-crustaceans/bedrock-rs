//! The one line a test prints when the game's files it needs are not there. Such a test returns
//! early and passes, which is no pass of what it checks: the line says so in one spelling,
//! `SKIPPED (not a pass): <what>: <reason>`, so a run's skips can be found and counted.

use std::fmt::Display;

/// The environment variable naming the folder that holds the game's files.
pub const CORPUS: &str = "BEDROCK_ADDON_CORPUS";

/// The environment variable naming an install of the game at 1.26.60.
pub const VANILLA_1_26_60: &str = "BEDROCK_ADDON_VANILLA_1_26_60";

/// The start of every skip line.
pub const SKIPPED: &str = "SKIPPED (not a pass)";

/// The skip line for `what` (the check, row or entry that did not run) and `reason` (why).
pub fn message(what: impl Display, reason: impl Display) -> String {
    format!("{SKIPPED}: {what}: {reason}")
}

/// Prints the skip line for `what` and `reason` to stderr.
pub fn skipped(what: impl Display, reason: impl Display) {
    eprintln!("{}", message(what, reason));
}

/// The reason of a skip whose environment variable `variable` is not set.
pub fn not_set(variable: &str) -> String {
    format!("{variable} is not set")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_skip_line_says_it_is_not_a_pass_and_names_the_variable() {
        let line = message("the vanilla gate", not_set(VANILLA_1_26_60));
        assert!(line.starts_with("SKIPPED (not a pass): "), "{line}");
        assert_eq!(
            line,
            "SKIPPED (not a pass): the vanilla gate: BEDROCK_ADDON_VANILLA_1_26_60 is not set"
        );
    }
}
