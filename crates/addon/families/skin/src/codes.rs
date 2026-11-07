//! The `skin` family's codes: the game's content-log lines, byte for byte, each a definition beside
//! the rules that raise it. Each [`was`](CodeDef::was) the legacy code its sites raised, so a pin
//! or a filter on the code's name keeps matching.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

/// A pack without `skins.json`.
pub static NOT_FOUND: CodeDef<0> = CodeDef::new(
    "skin/not_found",
    Kind::MissingRequired,
    Class::Error,
    "no skins.json",
)
.game("Failed to load skinpack, skins.json not found")
.was("MissingRequiredField");

/// A `skins.json` that is not JSON; the reader's message rides as the finding's syntax detail.
pub static NOT_JSON: CodeDef<0> = CodeDef::new(
    "skin/not_json",
    Kind::Syntax,
    Class::Error,
    "skins.json is not JSON",
)
.game("Failed to load skinpack, failed to parse manifest")
.was("MalformedInput");

/// A `skins.json` whose root is no object; the literal `/n` and the two spaces are the game's. The
/// arguments are the pack's name, author and UUID.
pub static NOT_OBJECT: CodeDef<3> = CodeDef::new(
    "skin/not_object",
    Kind::WrongType,
    Class::Error,
    "skins.json root is no object ({}, {}, {})",
)
.game("Skin manifest failed to load.  Pack Name: {} /nAuthor: {}, /nUUID: {}")
.was("InvalidType");

/// The start of the content-log line for a skin whose files are missing; the game logs it at the
/// error level and loads the next skin: the skin is dropped while the pack is kept. The argument is
/// the list of what is missing.
pub static INVALID_SKIN: CodeDef<1> = CodeDef::new(
    "skin/invalid_skin",
    Kind::InvalidValue,
    Class::KeptError,
    "skin files missing:{}",
)
.game("Trying to add an invalid classic skin.{}")
.was("InvalidValue");

/// An entry of `skins` that is no object; the game logs it at the error level and goes on to the
/// next entry.
pub static ENTRY_NOT_OBJECT: CodeDef<0> = CodeDef::new(
    "skin/entry_not_object",
    Kind::WrongType,
    Class::KeptError,
    "skin entry is no object",
)
.game("Invalid entry in skins.json - Every entry under 'skins' must be a json object - it has to start with an \"{\" and end with a \"}\"")
.was("InvalidType");

/// Every definition of this module.
pub static ALL: &[&CodeMeta] = &[
    NOT_FOUND.meta(),
    NOT_JSON.meta(),
    NOT_OBJECT.meta(),
    INVALID_SKIN.meta(),
    ENTRY_NOT_OBJECT.meta(),
];
