//! The `spawn_groups` family's codes: the game's content-log lines, byte for byte, each a
//! definition beside the rules that raise it. Each id is `spawn_groups/<name>`, the name its
//! constant's in lower case.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

/// The closing line of a spawn-group file that failed.
pub static FAILED_GROUPS: CodeDef<0> = CodeDef::new(
    "spawn_groups/failed_groups",
    Kind::Closing,
    Class::Error,
    "spawn group file failed to load",
)
.game("Failed to load spawn groups");

/// Every definition of this module.
pub static ALL: &[&CodeMeta] = &[FAILED_GROUPS.meta()];
