//! The `attachable` family's codes: the game's content-log lines, byte for byte, each a definition
//! beside the rules that raise it. Each id is `attachable/<name>`, the name its constant's in lower
//! case.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

/// An `item` that is neither a string nor an object; the argument is the allowed types.
pub static UNKNOWN_ITEM_TYPE: CodeDef<1> = CodeDef::new(
    "attachable/unknown_item_type",
    Kind::InvalidValue,
    Class::Error,
    "item is neither a string nor an object ({})",
)
.game("unknown child schema option type.  Allowed types: {}");

/// Every definition of this module.
pub static ALL: &[&CodeMeta] = &[UNKNOWN_ITEM_TYPE.meta()];
