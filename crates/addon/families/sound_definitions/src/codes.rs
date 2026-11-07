//! The `sound_definitions` family's codes: the game's content-log lines, byte for byte, each a
//! definition beside the rules that raise it. Each id is `sound_definitions/<name>`, the name its
//! constant's in lower case.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

/// Vanilla's text for a `sound_definitions` member that is no object.
pub static NOT_A_LIST: CodeDef<0> = CodeDef::new(
    "sound_definitions/not_a_list",
    Kind::InvalidValue,
    Class::Error,
    "invalid value",
)
.game("sound definition field isn't a list of sound definitions!");

/// Vanilla's text (sic, no space) for a definition that is no object.
pub static MALFORMED_SOUND: CodeDef<1> = CodeDef::new(
    "sound_definitions/malformed_sound",
    Kind::InvalidValue,
    Class::KeptError,
    "invalid value",
)
.game("sound definitions has sound '{}'that is malformed");

/// Vanilla's text for a merged root that is no object.
pub static CONTENTS_MALFORMED: CodeDef<0> = CodeDef::new(
    "sound_definitions/contents_malformed",
    Kind::InvalidValue,
    Class::KeptError,
    "invalid value",
)
.game("sound_definitions.json contents is malformed");

/// Vanilla's text for a merged `sound_definitions` member that is no object.
pub static SECTION_MALFORMED: CodeDef<0> = CodeDef::new(
    "sound_definitions/section_malformed",
    Kind::InvalidValue,
    Class::KeptError,
    "invalid value",
)
.game("sound_definitions section is malformed");

/// Vanilla's text for a `music.*` definition of another category.
pub static NOT_MUSIC: CodeDef<1> = CodeDef::new(
    "sound_definitions/not_music",
    Kind::InvalidValue,
    Class::Warning,
    "invalid value",
)
.game("The sound category of '{}' is not set as music in sound_definitions.json");

/// Every definition of this module, for the id registry.
pub static ALL: &[&CodeMeta] = &[
    NOT_A_LIST.meta(),
    MALFORMED_SOUND.meta(),
    CONTENTS_MALFORMED.meta(),
    SECTION_MALFORMED.meta(),
    NOT_MUSIC.meta(),
];
