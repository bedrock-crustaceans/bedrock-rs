//! The `lang` family's codes: the game's content-log lines, byte for byte, each a definition beside
//! the rules that raise it. Each id is `lang/<name>`, the name its constant's in lower case.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

/// `MalformedInput`: the game's line `Line: {} - Invalid lang file format. New line character was found while parsing key: '{}'.`.
pub static NEW_LINE_IN_KEY: CodeDef<2> = CodeDef::new(
    "lang/new_line_in_key",
    Kind::Syntax,
    Class::Warning,
    "malformed input",
)
.game("Line: {} - Invalid lang file format. New line character was found while parsing key: '{}'.");

/// `MalformedInput`: the game's line `Line: {} - Invalid lang file format. Null character was found while parsing key: '{}'.`.
pub static NULL_IN_KEY: CodeDef<2> = CodeDef::new(
    "lang/null_in_key",
    Kind::Syntax,
    Class::Warning,
    "malformed input",
)
.game("Line: {} - Invalid lang file format. Null character was found while parsing key: '{}'.");

/// `InvalidType`: the game's line `Languange name entries should be an array with 2 elements, a code and a name`.
pub static NAME_ENTRY_SHAPE_INVALID_TYPE: CodeDef<0> = CodeDef::new(
    "lang/name_entry_shape_invalid_type",
    Kind::WrongType,
    Class::Lint,
    "wrong JSON type",
)
.game("Languange name entries should be an array with 2 elements, a code and a name");

/// `InvalidValue`: the game's line `Languange name entries should be an array with 2 elements, a code and a name`.
pub static NAME_ENTRY_SHAPE_INVALID_VALUE: CodeDef<0> = CodeDef::new(
    "lang/name_entry_shape_invalid_value",
    Kind::InvalidValue,
    Class::Lint,
    "invalid value",
)
.game("Languange name entries should be an array with 2 elements, a code and a name");

/// Every definition of this module, for the id registry.
pub static ALL: &[&CodeMeta] = &[
    NEW_LINE_IN_KEY.meta(),
    NULL_IN_KEY.meta(),
    NAME_ENTRY_SHAPE_INVALID_TYPE.meta(),
    NAME_ENTRY_SHAPE_INVALID_VALUE.meta(),
];
