//! The command version a pack's commands run with, which the pack's `min_engine_version` maps to.
//! The mapping itself is core's `MinEngineVersion::command_version`, shared with every other
//! family that stores commands.

use bedrock_addon_core::version::{CURRENT_COMMAND_VERSION, MinEngineVersion};

/// The current command version: what a pack without a valid `min_engine_version` gets.
pub(crate) const CURRENT: u8 = CURRENT_COMMAND_VERSION;

/// The command version of a pack with this `min_engine_version` (`None` when the pack has none
/// or an invalid one).
pub(crate) fn command_version(min_engine_version: Option<&MinEngineVersion>) -> u8 {
    min_engine_version.map_or(CURRENT, MinEngineVersion::command_version)
}
