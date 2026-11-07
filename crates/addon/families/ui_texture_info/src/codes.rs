//! The `ui_texture_info` family's codes: the game's content-log lines, byte for byte, each a
//! definition beside the rules that raise it. Each [`was`](CodeDef::was) the legacy code its sites
//! raised, so a pin or a filter on the code's name keeps matching.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

/// A `nineslice_size` with a negative side: a debug assertion in the game; the value is kept.
pub static NEGATIVE_NINESLICE: CodeDef<0> = CodeDef::new(
    "ui_texture_info/negative_nineslice",
    Kind::InvalidValue,
    Class::Lint,
    "negative nineslice_size",
)
.game("NineSlice cannot be negative")
.was("InvalidValue");

/// A `base_size` with a negative side: a debug assertion in the game; the value is kept.
pub static NEGATIVE_BASE_SIZE: CodeDef<0> = CodeDef::new(
    "ui_texture_info/negative_base_size",
    Kind::InvalidValue,
    Class::Lint,
    "negative base_size",
)
.game("UV Base size can't be negative!")
.was("InvalidValue");

/// Every definition of this module.
pub static ALL: &[&CodeMeta] = &[NEGATIVE_NINESLICE.meta(), NEGATIVE_BASE_SIZE.meta()];
