//! NPC dialogue: `DialogueScene`, `DialogueFile`, `Button`.
//!
//! NPC dialogue: `dialogue/**/*.json` of a behaviour pack (`minecraft:npc_dialogue`), a list of
//! scenes keyed by `scene_tag`. One unversioned shape covers every accepted `format_version`
//! (1.14.0 or above, no maximum): the game reads the files with a hand-written walk over the
//! parsed JSON tree, member by member, rather than through a typed schema. Its rules (which
//! member is optional, which type is tolerated, what drops a scene) are not those of a derived
//! deserialiser, so this module walks a parsed `json::Value` the same way instead of
//! deserialising typed structs.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod command;
mod load;
mod new;
mod read;
mod texts;
mod write;

#[cfg(test)]
mod tests;

use bedrock_addon_core::document::Extra;
use bedrock_addon_core::id::DialogueSceneTag;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::version::SemVersion;

#[doc(hidden)]
pub use load::load_registry;

/// One dialogue file (`dialogue/**/*.json`): the scenes it defines.
///
/// JSON: `{"format_version": "1.14.0", "minecraft:npc_dialogue": {"scenes": [...]}}`. The stack
/// keeps every scene by its tag in `bp.dialogue()`; a file is the unit a pack stores and writes.
#[derive(Clone, Debug, PartialEq)]
#[doc(alias = "minecraft:npc_dialogue")]
pub struct DialogueFile {
    /// The file's version as read (JSON `format_version`): a version string, partial ones
    /// allowed (`"1.14"` is 1.14.0), or three integers. Vanilla loads 1.14.0 and anything above
    /// it; nothing in the shape depends on it.
    pub format_version: SemVersion,
    /// The scenes that loaded, in file order (JSON `minecraft:npc_dialogue.scenes`).
    pub scenes: Vec<DialogueScene>,
    /// Unknown members of `minecraft:npc_dialogue`, which vanilla never reads.
    pub dialogue_extra: Extra,
    /// Unknown members of the root, which vanilla never reads.
    pub extra: Extra,
}

/// One scene: the text an NPC shows, its buttons and the commands
/// it runs when the dialogue opens and closes.
///
/// A scene is what `/dialogue open` and `/dialogue change` name by its tag. Vanilla keeps the
/// first scene stored under a tag: across packs the lowest-priority pack's.
#[derive(Clone, Debug, PartialEq)]
pub struct DialogueScene {
    /// The scene's tag, case-sensitive and unique over the behaviour stack (JSON `scene_tag`).
    /// A scene with an empty tag is dropped.
    pub scene_tag: DialogueSceneTag,
    /// The NPC's name while the scene shows (JSON `npc_name`); `None` when absent, which vanilla
    /// reads as the empty name (the NPC's own name is not used).
    pub npc_name: Option<DialogueText>,
    /// The dialogue text (JSON `text`, required).
    pub text: DialogueText,
    /// The buttons, at most six (JSON `buttons`); `None` when absent or not an array, which
    /// vanilla ignores.
    pub buttons: Option<Vec<Button>>,
    /// The commands run when the scene opens (JSON `on_open_commands`); `None` when absent or not
    /// an array. A present list, even an empty one, is one action.
    pub on_open_commands: Option<Vec<StoredCommand>>,
    /// The commands run when the scene closes (JSON `on_close_commands`); as
    /// [`on_open_commands`](Self::on_open_commands).
    pub on_close_commands: Option<Vec<StoredCommand>>,
    /// Unknown members, which vanilla never reads.
    pub extra: Extra,
}

/// One button of a scene: a label and either commands or a URL.
#[derive(Clone, Debug, PartialEq)]
pub struct Button {
    /// The button's name as written (JSON `name`, required, non-empty). The label the button
    /// shows is [`label`](Self::label).
    pub name: DialogueText,
    /// The commands the button runs (JSON `commands`); `None` when absent or not an array. Never
    /// together with a [`url`](Self::url).
    pub commands: Option<Vec<StoredCommand>>,
    /// The URL the button opens (JSON `url`); `None` when absent. A non-empty URL makes the
    /// button a URL button; an empty one a command button without commands.
    pub url: Option<String>,
    /// Unknown members, which vanilla never reads.
    pub extra: Extra,
}

/// A text of a scene or a button: a plain string, or any JSON object (the intended form is a
/// raw-text object, `{"rawtext": [{"text": "Hi"}]}`), which vanilla keeps as its JSON text.
#[derive(Clone, Debug, PartialEq)]
pub enum DialogueText {
    /// A string, kept as written.
    Plain(String),
    /// An object, kept as written; vanilla stores it as compact JSON text
    /// ([`stored`](DialogueText::stored)).
    Raw(Value),
}

/// One command of a button or a scene: the trimmed text and the command version it runs with.
///
/// Commands are not checked when a pack loads: vanilla compiles each one when the action runs,
/// with the command version of the pack that defined the scene.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StoredCommand {
    /// The command version: the one the pack's `min_engine_version` maps to (46 for a pack
    /// without one).
    pub version: u8,
    /// The command as written, without surrounding whitespace; a leading `/` is kept.
    pub raw: String,
}

/// When an action runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ActionMode {
    /// When its button is pressed.
    Button = 0,
    /// When the dialogue closes.
    OnClose = 1,
    /// When the dialogue opens.
    OnOpen = 2,
}

/// One action of a scene as the game holds it, either a command action or a URL action: the
/// buttons first, then the open, then the close commands.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum NpcAction {
    /// Commands, run by a button or when the dialogue opens or closes.
    Command {
        /// When it runs.
        mode: ActionMode,
        /// The button's name; the empty plain text for an open or close action.
        button_name: DialogueText,
        /// The commands, in order.
        commands: Vec<StoredCommand>,
    },
    /// A button that opens a URL.
    Url {
        /// The button's name.
        button_name: DialogueText,
        /// The URL.
        url: String,
    },
}
