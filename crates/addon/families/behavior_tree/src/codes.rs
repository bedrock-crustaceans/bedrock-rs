//! The `behavior_tree` family's codes: the game's content-log lines, byte for byte, each a definition
//! beside the rules that raise it. Each [`was`](CodeDef::was) the legacy code its sites raised,
//! so a pin or a filter on the code's name keeps matching.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

/// Vanilla's text for a file that is not JSON, with the reader's message.
pub static PARSE_ERROR: CodeDef<1> = CodeDef::new(
    "behavior_tree/parse_error",
    Kind::Syntax,
    Class::Error,
    "malformed input",
)
.game("parse error:\n{}")
.was("MalformedInput");

/// Vanilla's text for an element of a block position that is not integral.
pub static EXPECTED_INTEGRAL: CodeDef<0> = CodeDef::new(
    "behavior_tree/expected_integral",
    Kind::WrongType,
    Class::KeptError,
    "wrong JSON type",
)
.game("Expected Integral Number.")
.was("InvalidType");

/// Vanilla's text for an element of a position that is not a number.
pub static EXPECTED_NUMBER: CodeDef<0> = CodeDef::new(
    "behavior_tree/expected_number",
    Kind::WrongType,
    Class::KeptError,
    "wrong JSON type",
)
.game("Expected Number.")
.was("InvalidType");

/// Every definition of this module, for the id registry.
pub static ALL: &[&CodeMeta] = &[
    PARSE_ERROR.meta(),
    EXPECTED_INTEGRAL.meta(),
    EXPECTED_NUMBER.meta(),
];
