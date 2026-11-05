//! Write-back: a dialogue file as JSON text, and the compact text vanilla stores for an object.

use facet_value::{VArray, VObject};

use super::{Button, DialogueFile, DialogueScene, DialogueText, StoredCommand};
use bedrock_addon_core::json::{EncodeError, Indent, Value, WriteStyle, write_json};
use bedrock_addon_core::version::SemVersion;

impl DialogueFile {
    /// Writes the file in `style`: the known members in schema order, and no member the author
    /// did not write.
    pub(crate) fn write_into(
        &self,
        out: &mut Vec<u8>,
        style: WriteStyle,
    ) -> Result<(), EncodeError> {
        write_json(&self.root(), out, style)
    }

    /// The file's known members.
    fn root(&self) -> Value {
        let mut scenes = VArray::new();
        for scene in &self.scenes {
            scenes.push(scene_json(scene));
        }
        let mut payload = VObject::new();
        payload.insert("scenes", scenes);
        // Not core's `write_envelope`: the version is written as the file spelled it (a
        // `SemVersion`, not a `FormatVersion`), and dialogue is no family chain.
        let mut root = VObject::new();
        root.insert("format_version", version_text(&self.format_version));
        root.insert("minecraft:npc_dialogue", payload);
        Value::from(root)
    }
}

/// The version as vanilla reads it back: the beta sentinel as the literal `beta`, any other
/// version in full.
fn version_text(version: &SemVersion) -> String {
    if SemVersion::parse("beta").as_ref() == Some(version) {
        "beta".to_owned()
    } else {
        version.to_string()
    }
}

/// One scene.
fn scene_json(scene: &DialogueScene) -> Value {
    let mut object = VObject::new();
    object.insert("scene_tag", scene.scene_tag.as_str());
    if let Some(name) = &scene.npc_name {
        object.insert("npc_name", text_json(name));
    }
    object.insert("text", text_json(&scene.text));
    if let Some(buttons) = &scene.buttons {
        let mut list = VArray::new();
        for button in buttons {
            list.push(button_json(button));
        }
        object.insert("buttons", list);
    }
    if let Some(commands) = &scene.on_open_commands {
        object.insert("on_open_commands", commands_json(commands));
    }
    if let Some(commands) = &scene.on_close_commands {
        object.insert("on_close_commands", commands_json(commands));
    }
    Value::from(object)
}

/// One button.
fn button_json(button: &Button) -> Value {
    let mut object = VObject::new();
    object.insert("name", text_json(&button.name));
    if let Some(commands) = &button.commands {
        object.insert("commands", commands_json(commands));
    }
    if let Some(url) = &button.url {
        object.insert("url", url.as_str());
    }
    Value::from(object)
}

/// A text: its string, or its object as written.
fn text_json(text: &DialogueText) -> Value {
    match text {
        DialogueText::Plain(text) => Value::from(text.as_str()),
        DialogueText::Raw(object) => object.clone(),
    }
}

/// A command list: the command texts (the command version is the pack's, never written).
fn commands_json(commands: &[StoredCommand]) -> VArray {
    let mut list = VArray::new();
    for command in commands {
        list.push(Value::from(command.raw.as_str()));
    }
    list
}

/// `Json::FastWriter::write` of `value`: compact JSON and one newline. Object members come in
/// key order, as `Json::Value`'s sorted member map would hold them (inferred, not observed:
/// the trailing newline is the game's, the member order and the number spelling follow
/// jsoncpp's conventions). A value JSON cannot spell (a non-finite number) gives the empty
/// text.
pub(crate) fn fast_writer(value: &Value) -> String {
    let mut out = Vec::new();
    let style = WriteStyle {
        indent: Indent::Compact,
        trailing_newline: true,
    };
    if write_json(&by_key(value), &mut out, style).is_err() {
        return String::new();
    }
    String::from_utf8(out).unwrap_or_default()
}

/// `value` with every object's members sorted by key (bytewise, as `strcmp`).
fn by_key(value: &Value) -> Value {
    if let Some(object) = value.as_object() {
        let mut members: Vec<(&str, &Value)> = object
            .iter()
            .map(|(key, member)| (key.as_str(), member))
            .collect();
        members.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
        let mut sorted = VObject::new();
        for (key, member) in members {
            sorted.insert(key, by_key(member));
        }
        return Value::from(sorted);
    }
    if let Some(array) = value.as_array() {
        let mut elements = VArray::new();
        for element in array {
            elements.push(by_key(element));
        }
        return Value::from(elements);
    }
    value.clone()
}
