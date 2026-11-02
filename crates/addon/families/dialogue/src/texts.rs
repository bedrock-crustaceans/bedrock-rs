//! Vanilla's content-log texts of the dialogue loader,
//! each with the prefix `JSON: %s.json has an error: \n ` vanilla puts before it. `{}` is where
//! `VanillaText::Formatted` puts each argument; the first is always the file's stem.

use bedrock_addon_core::diagnostic::VanillaText;

/// One text with vanilla's prefix.
macro_rules! text {
    ($rest:literal) => {
        concat!("JSON: {}.json has an error: \n ", $rest)
    };
}

/// A syntax error: the prefix and the reader's message. Vanilla's second argument is the JSON
/// reader's formatted error messages; the crate's own is facet-json's message.
pub(crate) const READER: &str = text!("{}");
/// The root is not an object.
pub(crate) const ROOT_NOT_OBJECT: &str = text!("Root value is not a object");
/// `format_version` absent or unreadable (`SemVersion::fromJson` gives `MatchType::None`).
pub(crate) const NO_FORMAT_VERSION: &str = text!("Couldn't find 'format_version' property.");
/// `format_version` below 1.14.0. Vanilla's format has two more `%s` without arguments; the
/// crate writes the text up to the version.
pub(crate) const OUT_OF_RANGE: &str =
    text!("'format_version' was out of range of expected values. Must be greater than {}");
/// `minecraft:npc_dialogue` is absent or `null` (its type is not checked).
pub(crate) const NO_PAYLOAD: &str = text!("Couldn't find 'minecraft:npc_dialogue' object.");
/// `scenes` is absent, `null` or not an array.
pub(crate) const NO_SCENES: &str = text!("Couldn't find 'scenes' array.");
/// The tag is already stored (format `%s%s%s`: the text, the tag, the text).
pub(crate) const DUPLICATE: &str =
    text!("A scene with a scene name of '{}' already exists. Skipping scene...");

/// `scene_tag` is absent or not a string.
pub(crate) const NO_SCENE_TAG: &str =
    text!("Each scene needs a 'scene_tag' to reference, was not found. Skipping scene...");
/// `npc_name` is a number, a bool or an array.
pub(crate) const NPC_NAME: &str =
    text!("The 'name' of an NPC must be a string or rawtext object. Skipping scene...");
/// `text` is absent or `null`.
pub(crate) const NO_TEXT: &str = text!(
    "Each scene needs a 'text' value to display to the reader, was not found. Skipping scene..."
);
/// More valid buttons than the maximum of six (format `%s%i%s`, the number in the middle).
pub(crate) const MAX_BUTTONS: &str =
    text!("Only a maximum number of {} buttons are currently supported. Skipping scene...");

/// A button's `name` is absent or neither a string nor an object.
pub(crate) const NO_BUTTON_NAME: &str =
    text!("Each button needs a 'name' field, was not found. Skipping button...");
/// A button has both `commands` and `url`.
pub(crate) const COMMANDS_AND_URL: &str = text!(
    "Each button supports either a 'commands' array or a URL to give it functionality, not both. Skipping button..."
);
/// An element of a button's `commands` is not a string.
pub(crate) const COMMAND_TYPE: &str =
    text!("Each command needs to be a string type. Skipping button...");
/// A button's `url` is not a string.
pub(crate) const URL_TYPE: &str = text!("The URL needs to be a string type. Skipping button...");

/// An element of `on_open_commands` / `on_close_commands` is not a string; the argument is
/// `on_open_command` or `on_close_command`. Vanilla's format has three `%s` and two arguments,
/// so it prints the key before "Each command" and nothing at the end; the crate puts it where
/// the sentence wants it.
pub(crate) const SCENE_COMMAND_TYPE: &str =
    text!("Each command needs to be a string type. Skipping this {} command...");

/// A text with its arguments, as the `text` member of a diagnostic code takes it.
#[expect(
    clippy::unnecessary_wraps,
    reason = "every `text` member is an `Option<VanillaText>`"
)]
pub(crate) fn formatted(template: &'static str, args: &[&str]) -> Option<VanillaText> {
    Some(VanillaText::Formatted {
        template,
        args: args.iter().map(|arg| (*arg).to_owned()).collect(),
    })
}
