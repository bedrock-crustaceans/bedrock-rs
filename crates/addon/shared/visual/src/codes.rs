//! The `visual` family's codes: the game's content-log lines, byte for byte, each a definition
//! beside the rules that raise it. Each [`was`](CodeDef::was) the legacy code its sites raised, so
//! a pin or a filter on the code's name keeps matching.
//!
//! What client entities, attachables and atomic client entities share.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Disposition, Kind};

/// A document without the payload: the game has only a debug assertion here, so the line is the
/// crate's own text of it.
pub static NO_ENTITY_DEFINITION: CodeDef<0> = CodeDef::new(
    "visual/no_entity_definition",
    Kind::MissingRequired,
    Class::Error,
    "no entity definition",
)
.game("No entity definition to parse")
.was("MissingRequiredField");

/// An identifier that is empty names nothing.
pub static MISSING_IDENTIFIER: CodeDef<0> = CodeDef::new(
    "visual/missing_identifier",
    Kind::InvalidValue,
    Class::Error,
    "identifier is empty",
)
.game("Missing or invalid identifier")
.was("InvalidValue");

/// A `held_item_scale` number at or below 0: the game logs it and loads the file. The arguments are
/// the file (the placeholder `rules::FILE` until the load knows the path) and the value.
pub static HELD_ITEM_SCALE_INVALID: CodeDef<2> = CodeDef::new(
    "visual/held_item_scale_invalid",
    Kind::InvalidValue,
    Class::KeptError,
    "held_item_scale of {} is not above zero ({})",
)
.game("In file '{}', parameter \"held_item_scale\" has invalid value: must be greater than zero, was '{}'.")
.was("InvalidValue");

/// A named script the loader cannot parse follows the parser's lines and is not stored; the game
/// loads the file. The argument is the member name.
pub static FAILED_TO_PARSE: CodeDef<1> = CodeDef::new(
    "visual/failed_to_parse",
    Kind::UnknownMember(Disposition::Dropped),
    Class::KeptError,
    "script {} dropped",
)
.game("Failed to parse '{}'!")
.was("DroppedField");

/// The loader's message for a `format_version` that does not read, or one below 1.0.0; the argument
/// is the file's path.
pub static UNKNOWN_VERSION: CodeDef<1> = CodeDef::new(
    "visual/unknown_version",
    Kind::UnsupportedVersion,
    Class::Error,
    "unknown format_version in {}",
)
.game("Unknown/unsupported version value specified in format_version field in {}")
.was("UnsupportedVersion");

/// A `format_version` above the engine's: the loader skips the file. The arguments are the version
/// as written and the engine version.
pub static ACTOR_ABOVE_ENGINE: CodeDef<2> = CodeDef::new(
    "visual/actor_above_engine",
    Kind::UnsupportedVersion,
    Class::Error,
    "format_version {} exceeds the engine's {}",
)
.game("Error: actor resource file has format version: {} exceeds game version: {}. Skipping.")
.was("UnsupportedVersion");

/// A root that is null, no object or empty; the finding carries the same words as its syntax
/// detail.
pub static NOTHING_TO_PARSE: CodeDef<0> = CodeDef::new(
    "visual/nothing_to_parse",
    Kind::Syntax,
    Class::Error,
    "nothing to parse",
)
.game("nothing to parse in")
.was("MalformedInput");

/// Every definition of this module.
pub static ALL: &[&CodeMeta] = &[
    NO_ENTITY_DEFINITION.meta(),
    MISSING_IDENTIFIER.meta(),
    HELD_ITEM_SCALE_INVALID.meta(),
    FAILED_TO_PARSE.meta(),
    UNKNOWN_VERSION.meta(),
    ACTOR_ABOVE_ENGINE.meta(),
    NOTHING_TO_PARSE.meta(),
];
