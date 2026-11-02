//! Constructors and derived views of dialogue.

use super::command::CURRENT;
use super::read::FLOOR;
use super::write::fast_writer;
use super::{
    ActionMode, Button, DialogueFile, DialogueScene, DialogueText, NpcAction, StoredCommand,
};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::id::DialogueSceneTag;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::version::SemVersion;

impl DialogueText {
    /// The text as vanilla stores it: a plain string as written, an object as compact JSON
    /// followed by one newline (`Json::FastWriter`), `{"rawtext":[{"text":"Hi"}]}\n`.
    ///
    /// The compact form and the newline are vanilla's. Two details of an object's text are
    /// inferred, not read from the game: its members are written in key order (bytewise), as
    /// jsoncpp's sorted member map would give them, and numbers are spelled as this crate
    /// writes them (integers as integers, reals in the shortest form). An object holding a
    /// number JSON cannot spell gives the empty text.
    #[must_use]
    pub fn stored(&self) -> String {
        match self {
            Self::Plain(text) => text.clone(),
            Self::Raw(object) => fast_writer(object),
        }
    }
}

impl DialogueScene {
    /// The scene's actions as the game holds them (vanilla `npc::ActionContainer`): one per
    /// button, then the open commands, then the close commands.
    pub fn actions(&self) -> Vec<NpcAction> {
        let mut actions: Vec<NpcAction> =
            self.buttons.iter().flatten().map(Button::action).collect();
        // A present list, even an empty one, is one action; open (mode 2) before close
        // (mode 1).
        let lists = [
            (ActionMode::OnOpen, &self.on_open_commands),
            (ActionMode::OnClose, &self.on_close_commands),
        ];
        for (mode, commands) in lists {
            if let Some(commands) = commands {
                actions.push(NpcAction::Command {
                    mode,
                    button_name: DialogueText::Plain(String::new()),
                    commands: commands.clone(),
                });
            }
        }
        actions
    }
}

impl Button {
    /// The button's action: a URL action when it has a non-empty URL, else a command action
    /// (also without commands).
    #[must_use]
    pub fn action(&self) -> NpcAction {
        match &self.url {
            Some(url) if !url.is_empty() => NpcAction::Url {
                button_name: self.name.clone(),
                url: url.clone(),
            },
            _ => NpcAction::Command {
                mode: ActionMode::Button,
                button_name: self.name.clone(),
                commands: self.commands.clone().unwrap_or_default(),
            },
        }
    }

    /// The label the button shows: the name, evaluated as raw text when it is a raw-text
    /// object's JSON, cut to its first 16 characters.
    ///
    /// The cut is vanilla's. What a raw-text object evaluates to is inferred, not read from
    /// the game: each `text` part's string and each `translate` part's key, untranslated,
    /// joined in order; other parts (`score`, `selector`) give nothing. A name that is not
    /// JSON for an object with a `rawtext` array is shown as stored.
    ///
    /// The raw name is kept as written in [`name`](Self::name); only the shown label is cut, by
    /// characters rather than bytes.
    #[must_use]
    pub fn label(&self) -> String {
        evaluated(&self.name.stored())
            .chars()
            .take(LABEL_CODE_POINTS)
            .collect()
    }

    /// Whether [`label`](Self::label) is shorter than the evaluated name.
    pub(crate) fn label_is_cut(&self) -> bool {
        evaluated(&self.name.stored()).chars().count() > LABEL_CODE_POINTS
    }
}

impl NpcAction {
    /// The action's text as the game sends it to clients: each command followed by a newline
    /// (the last one included); the URL of a URL button.
    #[must_use]
    pub fn text(&self) -> String {
        match self {
            Self::Command { commands, .. } => commands
                .iter()
                .flat_map(|command| [command.raw.as_str(), "\n"])
                .collect(),
            Self::Url { url, .. } => url.clone(),
        }
    }
}

/// How many characters of a button's evaluated name its label keeps.
const LABEL_CODE_POINTS: usize = 16;

/// `raw` as the game shows it: the text of the raw-text object it spells, or `raw` itself when
/// it does not spell one.
///
/// What `TextObjectRoot::asString()` gives is inferred: each `text` part's string, each
/// `translate` part's key untranslated, nothing for other parts.
fn evaluated(raw: &str) -> String {
    let parsed = bedrock_addon_core::json::from_str::<Value>(raw).ok();
    let parts = parsed
        .as_ref()
        .and_then(Value::as_object)
        .and_then(|object| object.get("rawtext"))
        .and_then(Value::as_array);
    let Some(parts) = parts else {
        return raw.to_owned();
    };
    parts
        .iter()
        .filter_map(|part| {
            let part = part.as_object()?;
            part.get("text")
                .or_else(|| part.get("translate"))?
                .as_string()
        })
        .map(facet_value::VString::as_str)
        .collect()
}

impl DialogueFile {
    /// A file of these scenes at format version 1.14.0, the lowest one vanilla loads (every
    /// version from there on reads the same shape).
    #[must_use]
    pub fn new(scenes: Vec<DialogueScene>) -> DialogueFile {
        DialogueFile {
            format_version: SemVersion::from(FLOOR),
            scenes,
            dialogue_extra: Extra::default(),
            extra: Extra::default(),
        }
    }
}

impl DialogueScene {
    /// A scene with this tag and text: no name, no buttons, no commands.
    #[must_use]
    pub fn new(scene_tag: DialogueSceneTag, text: DialogueText) -> DialogueScene {
        DialogueScene {
            scene_tag,
            npc_name: None,
            text,
            buttons: None,
            on_open_commands: None,
            on_close_commands: None,
            extra: Extra::default(),
        }
    }
}

impl Button {
    /// A button with this name: no commands, no URL.
    #[must_use]
    pub fn new(name: DialogueText) -> Button {
        Button {
            name,
            commands: None,
            url: None,
            extra: Extra::default(),
        }
    }
}

impl StoredCommand {
    /// A command at the current command version (46), the one a pack without
    /// `min_engine_version` gets. The text is kept as given; a loaded command is trimmed.
    pub fn new(raw: impl Into<String>) -> StoredCommand {
        StoredCommand {
            version: CURRENT,
            raw: raw.into(),
        }
    }
}
