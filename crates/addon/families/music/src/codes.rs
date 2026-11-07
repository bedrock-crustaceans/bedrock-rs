//! The `music` family's codes: the game's content-log lines, byte for byte, each a definition
//! beside the rules that raise it. Each [`was`](CodeDef::was) the legacy code its sites raised, so
//! a pin or a filter on the code's name keeps matching.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

/// A definition under the empty name (a warning in the game; the entry is kept as written).
pub static NO_NAME: CodeDef<0> = CodeDef::new(
    "music/no_name",
    Kind::InvalidValue,
    Class::Warning,
    "music definition with no name",
)
.game("Cannot define a music definition with no name.")
.was("InvalidValue");

/// Every definition of this module.
pub static ALL: &[&CodeMeta] = &[NO_NAME.meta()];
