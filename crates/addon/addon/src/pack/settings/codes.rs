//! The pack settings' codes: the game's content-log lines for its settings files, byte for byte.
//! Each id is `pack_settings/<name>`, the name its constant's in lower case.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

/// The closing error of a settings file that failed to load; the argument is the file's path.
pub static LOAD_FAILED: CodeDef<1> = CodeDef::new(
    "pack_settings/load_failed",
    Kind::Closing,
    Class::Error,
    "the pack settings file {} failed to load",
)
.game("Pack Settings failed to load [{}]");

/// Every definition of this table, for the id registry.
pub static ALL: &[&CodeMeta] = &[LOAD_FAILED.meta()];
