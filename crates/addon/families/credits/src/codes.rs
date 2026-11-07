//! The `credits` family's codes: the game's content-log lines, byte for byte, each a definition
//! beside the rules that raise it. Each [`was`](CodeDef::was) the legacy code its sites raised, so
//! a pin or a filter on the code's name keeps matching.
//!
//! The game's asserts on its own file are developer asserts and write no content-log line, so
//! the crate reports them (but the two that refuse the file) as informations with the game's texts.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

/// The assert for an empty file; the file is refused, and the finding carries the reader's message
/// as its syntax detail.
pub static EMPTY_FILE: CodeDef<0> = CodeDef::new(
    "credits/empty_file",
    Kind::Syntax,
    Class::Error,
    "credits file is empty",
)
.game("File is not supposed to be empty")
.was("MalformedInput");

/// The assert for a file that is not JSON; the file is refused, and the finding carries the
/// reader's message as its syntax detail.
pub static NOT_JSON: CodeDef<0> = CodeDef::new(
    "credits/not_json",
    Kind::Syntax,
    Class::Error,
    "credits file is not JSON",
)
.game("Failed to load json file")
.was("MalformedInput");

/// The root is no array: the file has no section.
pub static ROOT_NOT_ARRAY: CodeDef<0> = CodeDef::new(
    "credits/root_not_array",
    Kind::WrongType,
    Class::Lint,
    "credits root is no array",
)
.game("The root must be an array")
.was("InvalidType");

/// A section that is no object with a member.
pub static SECTION_EMPTY: CodeDef<0> = CodeDef::new(
    "credits/section_empty",
    Kind::WrongType,
    Class::Lint,
    "empty credits section",
)
.game("The section data should not be empty")
.was("InvalidType");

/// A section's `disciplines` is present and no array.
pub static DISCIPLINES_NOT_ARRAY: CodeDef<0> = CodeDef::new(
    "credits/disciplines_not_array",
    Kind::WrongType,
    Class::Lint,
    "`disciplines` is no array",
)
.game("\"disciplines\" must be an array")
.was("InvalidType");

/// A discipline that is no object with a member.
pub static DISCIPLINE_EMPTY: CodeDef<0> = CodeDef::new(
    "credits/discipline_empty",
    Kind::WrongType,
    Class::Lint,
    "empty credits discipline",
)
.game("The discipline data should not be empty")
.was("InvalidType");

/// A discipline's `titles` is present and no array.
pub static TITLES_NOT_ARRAY: CodeDef<0> = CodeDef::new(
    "credits/titles_not_array",
    Kind::WrongType,
    Class::Lint,
    "`titles` is no array",
)
.game("\"titles\" must be an array")
.was("InvalidType");

/// A title that is no object with a member.
pub static TITLE_EMPTY: CodeDef<0> = CodeDef::new(
    "credits/title_empty",
    Kind::WrongType,
    Class::Lint,
    "empty credits title",
)
.game("The title object should not be empty")
.was("InvalidType");

/// A title's `names` is present and no array.
pub static NAMES_NOT_ARRAY: CodeDef<0> = CodeDef::new(
    "credits/names_not_array",
    Kind::WrongType,
    Class::Lint,
    "`names` is no array",
)
.game("\"names\" must be an array")
.was("InvalidType");

/// Every definition of this module.
pub static ALL: &[&CodeMeta] = &[
    EMPTY_FILE.meta(),
    NOT_JSON.meta(),
    ROOT_NOT_ARRAY.meta(),
    SECTION_EMPTY.meta(),
    DISCIPLINES_NOT_ARRAY.meta(),
    DISCIPLINE_EMPTY.meta(),
    TITLES_NOT_ARRAY.meta(),
    TITLE_EMPTY.meta(),
    NAMES_NOT_ARRAY.meta(),
];
