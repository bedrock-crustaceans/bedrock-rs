//! The pack settings' codes: the game's content-log lines for its settings files, byte for byte.
//! Each [`was`](CodeDef::was) the legacy code its site raised, so a pin or a filter on the
//! code's name keeps matching.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

/// The closing error of a settings file that failed to load; the argument is the file's path.
pub static LOAD_FAILED: CodeDef<1> = CodeDef::new(
    "pack_settings/load_failed",
    Kind::Closing,
    Class::Error,
    "the pack settings file {} failed to load",
)
.game("Pack Settings failed to load [{}]")
.was("DocumentRejected");

/// Every definition of this table, for the id registry.
pub static ALL: &[&CodeMeta] = &[LOAD_FAILED.meta()];
