//! The dialogue family's codes: the game's content-log lines, byte for byte, each a definition
//! beside the rules that raise it. Each id is `dialogue/<name>`, the name its constant's in lower
//! case.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

/// One text with vanilla's prefix `JSON: %s.json has an error: \n `; the first argument of
/// every definition is the file's stem.
macro_rules! text {
    ($rest:literal) => {
        concat!("JSON: {}.json has an error: \n ", $rest)
    };
}

/// A syntax error: the prefix and the reader's message. Vanilla's second argument is the JSON
/// reader's formatted error messages; the crate's own is facet-json's message.
pub static READER: CodeDef<2> = CodeDef::new(
    "dialogue/reader",
    Kind::Syntax,
    Class::Error,
    "malformed input",
)
.game(text!("{}"));

/// The root is not an object.
pub static ROOT_NOT_OBJECT: CodeDef<1> = CodeDef::new(
    "dialogue/root_not_object",
    Kind::Syntax,
    Class::Error,
    "malformed input",
)
.game(text!("Root value is not a object"));

/// `format_version` absent or unreadable (`SemVersion::fromJson` gives `MatchType::None`).
pub static NO_FORMAT_VERSION_MISSING_FORMAT_VERSION: CodeDef<1> = CodeDef::new(
    "dialogue/no_format_version_missing_format_version",
    Kind::FormatVersion,
    Class::Error,
    "no format_version",
)
.game(text!("Couldn't find 'format_version' property."));

/// `format_version` absent or unreadable (`SemVersion::fromJson` gives `MatchType::None`).
pub static NO_FORMAT_VERSION_MALFORMED_FORMAT_VERSION: CodeDef<1> = CodeDef::new(
    "dialogue/no_format_version_malformed_format_version",
    Kind::FormatVersion,
    Class::Error,
    "Malformed format version",
)
.game(text!("Couldn't find 'format_version' property."));

/// `format_version` below 1.14.0. Vanilla's format has two more `%s` without arguments; the
/// crate writes the text up to the version.
pub static OUT_OF_RANGE: CodeDef<2> = CodeDef::new(
    "dialogue/out_of_range",
    Kind::UnsupportedVersion,
    Class::Error,
    "unsupported format_version",
)
.game(text!(
    "'format_version' was out of range of expected values. Must be greater than {}"
));

/// `minecraft:npc_dialogue` is absent or `null` (its type is not checked).
pub static NO_PAYLOAD: CodeDef<1> = CodeDef::new(
    "dialogue/no_payload",
    Kind::Envelope,
    Class::Error,
    "the document has no payload member",
)
.game(text!("Couldn't find 'minecraft:npc_dialogue' object."));

/// `scenes` is absent, `null` or not an array.
pub static NO_SCENES_MISSING_REQUIRED_FIELD: CodeDef<1> = CodeDef::new(
    "dialogue/no_scenes_missing_required_field",
    Kind::MissingRequired,
    Class::Error,
    "missing required field",
)
.game(text!("Couldn't find 'scenes' array."));

/// `scenes` is absent, `null` or not an array.
pub static NO_SCENES_INVALID_TYPE: CodeDef<1> = CodeDef::new(
    "dialogue/no_scenes_invalid_type",
    Kind::WrongType,
    Class::Error,
    "wrong JSON type",
)
.game(text!("Couldn't find 'scenes' array."));

/// The tag is already stored (format `%s%s%s`: the text, the tag, the text).
pub static DUPLICATE: CodeDef<2> = CodeDef::new(
    "dialogue/duplicate",
    Kind::Duplicate,
    Class::Error,
    "duplicate identifier",
)
.game(text!(
    "A scene with a scene name of '{}' already exists. Skipping scene..."
));

/// `scene_tag` is absent or not a string.
pub static NO_SCENE_TAG_MISSING_REQUIRED_FIELD: CodeDef<1> = CodeDef::new(
    "dialogue/no_scene_tag_missing_required_field",
    Kind::MissingRequired,
    Class::Error,
    "missing required field",
)
.game(text!(
    "Each scene needs a 'scene_tag' to reference, was not found. Skipping scene..."
));

/// `scene_tag` is absent or not a string.
pub static NO_SCENE_TAG_INVALID_TYPE: CodeDef<1> = CodeDef::new(
    "dialogue/no_scene_tag_invalid_type",
    Kind::WrongType,
    Class::Error,
    "wrong JSON type",
)
.game(text!(
    "Each scene needs a 'scene_tag' to reference, was not found. Skipping scene..."
));

/// `npc_name` is a number, a bool or an array.
pub static NPC_NAME: CodeDef<1> = CodeDef::new(
    "dialogue/npc_name",
    Kind::WrongType,
    Class::Error,
    "wrong JSON type",
)
.game(text!(
    "The 'name' of an NPC must be a string or rawtext object. Skipping scene..."
));

/// `text` is absent or `null`.
pub static NO_TEXT: CodeDef<1> = CodeDef::new(
    "dialogue/no_text",
    Kind::MissingRequired,
    Class::Error,
    "missing required field",
)
.game(text!(
    "Each scene needs a 'text' value to display to the reader, was not found. Skipping scene..."
));

/// More valid buttons than the maximum of six (format `%s%i%s`, the number in the middle).
pub static MAX_BUTTONS: CodeDef<2> = CodeDef::new(
    "dialogue/max_buttons",
    Kind::InvalidValue,
    Class::Error,
    "invalid value",
)
.game(text!(
    "Only a maximum number of {} buttons are currently supported. Skipping scene..."
));

/// A button's `name` is absent or neither a string nor an object.
pub static NO_BUTTON_NAME_MISSING_REQUIRED_FIELD: CodeDef<1> = CodeDef::new(
    "dialogue/no_button_name_missing_required_field",
    Kind::MissingRequired,
    Class::KeptError,
    "missing required field",
)
.game(text!(
    "Each button needs a 'name' field, was not found. Skipping button..."
));

/// A button's `name` is absent or neither a string nor an object.
pub static NO_BUTTON_NAME_INVALID_TYPE: CodeDef<1> = CodeDef::new(
    "dialogue/no_button_name_invalid_type",
    Kind::WrongType,
    Class::KeptError,
    "wrong JSON type",
)
.game(text!(
    "Each button needs a 'name' field, was not found. Skipping button..."
));

/// A button has both `commands` and `url`.
pub static COMMANDS_AND_URL: CodeDef<1> = CodeDef::new(
    "dialogue/commands_and_url",
    Kind::InvalidValue,
    Class::KeptError,
    "invalid value",
)
.game(text!(
    "Each button supports either a 'commands' array or a URL to give it functionality, not both. Skipping button..."
));

/// An element of a button's `commands` is not a string.
pub static COMMAND_TYPE: CodeDef<1> = CodeDef::new(
    "dialogue/command_type",
    Kind::WrongType,
    Class::KeptError,
    "wrong JSON type",
)
.game(text!(
    "Each command needs to be a string type. Skipping button..."
));

/// A button's `url` is not a string.
pub static URL_TYPE: CodeDef<1> = CodeDef::new(
    "dialogue/url_type",
    Kind::WrongType,
    Class::KeptError,
    "wrong JSON type",
)
.game(text!(
    "The URL needs to be a string type. Skipping button..."
));

/// An element of `on_open_commands` / `on_close_commands` is not a string; the argument is
/// `on_open_command` or `on_close_command`. Vanilla's format has three `%s` and two arguments,
/// so it prints the key before "Each command" and nothing at the end; the crate puts it where
/// the sentence wants it.
pub static SCENE_COMMAND_TYPE: CodeDef<2> = CodeDef::new(
    "dialogue/scene_command_type",
    Kind::WrongType,
    Class::KeptError,
    "wrong JSON type",
)
.game(text!(
    "Each command needs to be a string type. Skipping this {} command..."
));

/// Every definition of this module, for the id registry.
pub static ALL: &[&CodeMeta] = &[
    READER.meta(),
    ROOT_NOT_OBJECT.meta(),
    NO_FORMAT_VERSION_MISSING_FORMAT_VERSION.meta(),
    NO_FORMAT_VERSION_MALFORMED_FORMAT_VERSION.meta(),
    OUT_OF_RANGE.meta(),
    NO_PAYLOAD.meta(),
    NO_SCENES_MISSING_REQUIRED_FIELD.meta(),
    NO_SCENES_INVALID_TYPE.meta(),
    DUPLICATE.meta(),
    NO_SCENE_TAG_MISSING_REQUIRED_FIELD.meta(),
    NO_SCENE_TAG_INVALID_TYPE.meta(),
    NPC_NAME.meta(),
    NO_TEXT.meta(),
    MAX_BUTTONS.meta(),
    NO_BUTTON_NAME_MISSING_REQUIRED_FIELD.meta(),
    NO_BUTTON_NAME_INVALID_TYPE.meta(),
    COMMANDS_AND_URL.meta(),
    COMMAND_TYPE.meta(),
    URL_TYPE.meta(),
    SCENE_COMMAND_TYPE.meta(),
];
