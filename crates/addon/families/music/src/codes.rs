//! The `music` family's codes: the game's content-log lines, byte for byte, each a definition
//! beside the rules that raise it. Each id is `music/<name>`, the name its constant's in lower
//! case.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

/// A definition under the empty name (a warning in the game; the entry is kept as written).
pub static NO_NAME: CodeDef<0> = CodeDef::new(
    "music/no_name",
    Kind::InvalidValue,
    Class::Warning,
    "music definition with no name",
)
.game("Cannot define a music definition with no name.");

/// Every definition of this module.
pub static ALL: &[&CodeMeta] = &[NO_NAME.meta()];
