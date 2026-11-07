//! The `atomic_client_entity` family's codes: the game's content-log lines, byte for byte, each a
//! definition beside the rules that raise it. Each [`was`](CodeDef::was) the legacy code its sites
//! raised, so a pin or a filter on the code's name keeps matching.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

/// The identifier matches `^(?:.)+:(?:.)+$`: a namespace is required.
pub static NAMESPACE_TEXT: CodeDef<0> = CodeDef::new(
    "atomic_client_entity/namespace_text",
    Kind::InvalidValue,
    Class::Error,
    "identifier has no namespace",
)
.game("string must be prefixed with a namespace (eg. namespace:value)")
.was("InvalidValue");

/// A number member whose minimum is 0.0, inclusive.
pub static NOT_NEGATIVE_TEXT: CodeDef<0> = CodeDef::new(
    "atomic_client_entity/not_negative_text",
    Kind::InvalidValue,
    Class::Error,
    "number below 0",
)
.game("the input must be a number >= 0")
.was("InvalidValue");

/// Every definition of this module.
pub static ALL: &[&CodeMeta] = &[NAMESPACE_TEXT.meta(), NOT_NEGATIVE_TEXT.meta()];
