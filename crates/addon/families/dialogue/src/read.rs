//! The reader: one dialogue file to its scenes, walking the parsed JSON tree the way the game's
//! own dialogue reader does (file, then scene, then button).
//!
//! Every finding is built here without pack and file; the loader locates them.

use super::command::command_version;
use super::texts::{
    COMMAND_TYPE, COMMANDS_AND_URL, MAX_BUTTONS, NO_BUTTON_NAME, NO_FORMAT_VERSION, NO_PAYLOAD,
    NO_SCENE_TAG, NO_SCENES, NO_TEXT, NPC_NAME, OUT_OF_RANGE, READER, ROOT_NOT_OBJECT,
    SCENE_COMMAND_TYPE, URL_TYPE, formatted,
};
use super::{Button, DialogueScene, DialogueText, StoredCommand};
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, JsonPath};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::id::DialogueSceneTag;
use bedrock_addon_core::json::{MAX_JSON_DEPTH, Value, check_depth, strip_bom};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::version::{FormatVersion, SemVersion};

/// The payload member.
const PAYLOAD: &str = "minecraft:npc_dialogue";

/// The lowest `format_version` the game loads, 1.14.0.
pub(crate) const FLOOR: FormatVersion = FormatVersion::new(1, 14, 0);

/// The most valid buttons a scene may have.
const MAX_NUM_BUTTONS: usize = 6;

/// The members the game reads from a scene.
const SCENE_MEMBERS: &[&str] = &[
    "npc_name",
    "scene_tag",
    "text",
    "buttons",
    "on_open_commands",
    "on_close_commands",
];

/// A file vanilla reads: its version, its scenes and the file-level findings.
pub(crate) struct ReadFile {
    /// `format_version` as read.
    pub(crate) format_version: SemVersion,
    /// Every scene element, in file order.
    pub(crate) scenes: Vec<ReadScene>,
    /// Unknown members of `minecraft:npc_dialogue`.
    pub(crate) dialogue_extra: Extra,
    /// Unknown members of the root.
    pub(crate) extra: Extra,
    /// File-level warnings and infos.
    pub(crate) diagnostics: Diagnostics,
}

/// One element of `scenes`.
pub(crate) struct ReadScene {
    /// `minecraft:npc_dialogue/scenes/<index>`.
    pub(crate) at: JsonPath,
    /// What became of it.
    pub(crate) outcome: SceneOutcome,
}

/// What became of one scene.
pub(crate) enum SceneOutcome {
    /// It loaded, with its warnings and infos.
    Loaded(DialogueScene, Diagnostics),
    /// A rule dropped it: the tag when it had a string one, and every finding (the dropping one
    /// at error level).
    Dropped(Option<String>, Diagnostics),
    /// It parsed with an empty tag, which vanilla drops silently; its warnings.
    Untagged(Diagnostics),
}

/// Reads one file: the file, or every finding when the whole file is skipped. `stem` (the
/// file's name without folders and extension) is the `%s` of every vanilla text; `cx` gives the
/// command version.
pub(crate) fn read(stem: &str, bytes: &[u8], cx: &LoadContext) -> Result<ReadFile, Diagnostics> {
    let mut reader = Reader {
        stem,
        command_version: command_version(cx.min_engine_version.as_ref()),
        diagnostics: Diagnostics::default(),
    };
    // A number no double holds is read as the largest finite one, with a warning, so the
    // document can be written back.
    let finite = bedrock_addon_core::document::FiniteText::of(bytes);
    let mut file = reader.file(finite.bytes());
    match &mut file {
        Some(file) => finite.settle(&mut file.diagnostics, None),
        None => finite.settle(&mut reader.diagnostics, None),
    }
    file.ok_or(reader.diagnostics)
}

/// The member `key` of `value`, as jsoncpp's const `operator[]` and `isNull` see it: `None` when
/// `value` is not an object, the member is absent, or it is `null`.
fn member<'v>(value: &'v Value, key: &str) -> Option<&'v Value> {
    value
        .as_object()?
        .get(key)
        .filter(|member| !member.is_null())
}

/// The members of the object `value` vanilla never reads (every one but `known`), each
/// reported as an information at `at` (the game ignores them, so the crate's own finding says
/// so) and kept for write-back. A value that is not an object has none.
fn unknown(value: &Value, known: &[&str], at: &JsonPath, diagnostics: &mut Diagnostics) -> Extra {
    let mut extra = Extra::default();
    for (key, member) in value.as_object().into_iter().flatten() {
        if known.contains(&key.as_str()) {
            continue;
        }
        diagnostics.push(
            Diagnostic::info(DiagnosticCode::UnrecognizedField { text: None })
                .at(child(at, key.as_str())),
        );
        extra.insert(key.as_str().to_owned(), member.clone());
    }
    extra
}

/// `base` followed by `key`.
fn child(base: &JsonPath, key: &str) -> JsonPath {
    let mut at = base.clone();
    at.push(key);
    at
}

/// `base` followed by `index`.
fn element(base: &JsonPath, index: usize) -> JsonPath {
    let mut at = base.clone();
    at.push_index(index);
    at
}

/// The state of reading one file.
struct Reader<'s> {
    /// The file's stem.
    stem: &'s str,
    /// The pack's command version.
    command_version: u8,
    /// File-level findings.
    diagnostics: Diagnostics,
}

impl Reader<'_> {
    /// The file, or `None` when it is skipped (the findings say why).
    fn file(&mut self, bytes: &[u8]) -> Option<ReadFile> {
        let root = self.json(bytes)?;
        let format_version = self.version(&root)?;
        let payload_at = child(&JsonPath::new(), PAYLOAD);
        // Only a null check: the payload's type is not looked at.
        let Some(payload) = member(&root, PAYLOAD) else {
            self.error(
                DiagnosticCode::MissingPayload {
                    text: formatted(NO_PAYLOAD, &[self.stem]),
                },
                payload_at,
            );
            return None;
        };
        let scenes_at = child(&payload_at, "scenes");
        // One text for both an absent and a non-array `scenes`; a payload that is not an object
        // reads `scenes` as null.
        let text = formatted(NO_SCENES, &[self.stem]);
        let Some(scenes) = member(payload, "scenes") else {
            self.error(DiagnosticCode::MissingRequiredField { text }, scenes_at);
            return None;
        };
        let Some(scenes) = scenes.as_array() else {
            self.error(DiagnosticCode::InvalidType { text }, scenes_at);
            return None;
        };
        let scenes = scenes
            .iter()
            .enumerate()
            .map(|(index, scene)| {
                let at = element(&scenes_at, index);
                let outcome = self.scene(scene, &at);
                ReadScene { at, outcome }
            })
            .collect();
        let extra = unknown(
            &root,
            &["format_version", PAYLOAD],
            &JsonPath::new(),
            &mut self.diagnostics,
        );
        let dialogue_extra = unknown(payload, &["scenes"], &payload_at, &mut self.diagnostics);
        Some(ReadFile {
            format_version,
            scenes,
            dialogue_extra,
            extra,
            diagnostics: std::mem::take(&mut self.diagnostics),
        })
    }

    /// The parsed root, comments allowed (`Json::Reader` with `allowComments`).
    fn json(&mut self, bytes: &[u8]) -> Option<Value> {
        // A leading byte-order mark is blanked, not cut, so every offset stays an offset into the
        // file as written.
        let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
        let bytes = strip_bom(bytes);
        let text = check_depth(bytes, MAX_JSON_DEPTH)
            .ok()
            .and_then(|()| std::str::from_utf8(bytes).ok());
        let parsed = match text {
            // Of a member written twice the last stands.
            Some(text) => bedrock_addon_core::json::from_str_jsonc::<Value>(
                &bedrock_addon_core::json::last_wins(text),
            )
            .map_err(|error| error.to_string()),
            None => Err("not UTF-8 JSON within the nesting limit".to_owned()),
        };
        let root = match parsed {
            Ok(root) => root,
            Err(message) => {
                // The file is skipped with the reader's message.
                let text = formatted(READER, &[self.stem, &message]);
                self.error(
                    DiagnosticCode::MalformedInput { message, text },
                    JsonPath::new(),
                );
                return None;
            }
        };
        if root.as_object().is_none() {
            self.error(
                DiagnosticCode::MalformedInput {
                    message: "the root is not an object".to_owned(),
                    text: formatted(ROOT_NOT_OBJECT, &[self.stem]),
                },
                JsonPath::new(),
            );
            return None;
        }
        Some(root)
    }

    /// `format_version`, at 1.14.0 or above, with no maximum (`NPC_DIALOGUE_V1_14`).
    fn version(&mut self, root: &Value) -> Option<SemVersion> {
        let at = child(&JsonPath::new(), "format_version");
        let Some(written) = member(root, "format_version") else {
            self.error(
                DiagnosticCode::MissingFormatVersion {
                    text: formatted(NO_FORMAT_VERSION, &[self.stem]),
                },
                at,
            );
            return None;
        };
        // A version is a string (no wildcards), or an array of exactly three elements
        // convertible to unsigned; anything else is unreadable and gets the same text as absent.
        let read = match written.as_string() {
            Some(text) => version_string(text.as_str()),
            None => written
                .as_array()
                // Core's reader of the array form, shared with the manifest.
                .and_then(|elements| SemVersion::from_json_array(elements.as_slice())),
        };
        let Some(version) = read else {
            self.error(
                DiagnosticCode::MalformedFormatVersion {
                    text: formatted(NO_FORMAT_VERSION, &[self.stem]),
                },
                at,
            );
            return None;
        };
        if written
            .as_string()
            .is_some_and(|text| text.as_str() == "beta")
        {
            // The literal `beta` is read as the sentinel 9999.9999.9999-beta. The game writes no
            // content-log line for it, so the crate reports it as an information.
            self.diagnostics
                .push(Diagnostic::info(DiagnosticCode::InvalidValue { text: None }).at(at.clone()));
        }
        // A plain `<` against the floor: 1.14.0 itself loads.
        if version < SemVersion::from(FLOOR) {
            let floor = FLOOR.to_string();
            self.error(
                DiagnosticCode::UnsupportedVersion {
                    text: formatted(OUT_OF_RANGE, &[self.stem, &floor]),
                },
                at,
            );
            return None;
        }
        Some(version)
    }

    /// A file-level error.
    fn error(&mut self, code: DiagnosticCode, at: JsonPath) {
        self.diagnostics.push(Diagnostic::error(code).at(at));
    }

    /// One scene element.
    fn scene(&self, scene: &Value, at: &JsonPath) -> SceneOutcome {
        // The game's order: `npc_name`, `scene_tag`, `text`, `buttons` and their count, the open
        // and the close commands; the first failing rule drops the scene and ends the checks.
        let mut diagnostics = Diagnostics::default();
        // A null or absent name is the empty name; a string or an object is a name; anything else
        // drops the scene.
        let npc_name = match member(scene, "npc_name") {
            None => None,
            Some(name) => {
                let Some(name) = text(name) else {
                    let code = DiagnosticCode::InvalidType {
                        text: formatted(NPC_NAME, &[self.stem]),
                    };
                    return dropped(None, diagnostics, code, child(at, "npc_name"));
                };
                Some(name)
            }
        };
        // Absent, null or any other type than a string: one text.
        let Some(tag) = member(scene, "scene_tag") else {
            let code = DiagnosticCode::MissingRequiredField {
                text: formatted(NO_SCENE_TAG, &[self.stem]),
            };
            return dropped(None, diagnostics, code, child(at, "scene_tag"));
        };
        let Some(tag) = tag.as_string() else {
            let code = DiagnosticCode::InvalidType {
                text: formatted(NO_SCENE_TAG, &[self.stem]),
            };
            return dropped(None, diagnostics, code, child(at, "scene_tag"));
        };
        let tag = tag.as_str().to_owned();
        // An object is kept as its JSON text and anything else is read as a string with the empty
        // default: a bool reads as its word, a number or an array as the empty text.
        let text = match member(scene, "text") {
            None => {
                let code = DiagnosticCode::MissingRequiredField {
                    text: formatted(NO_TEXT, &[self.stem]),
                };
                return dropped(Some(tag), diagnostics, code, child(at, "text"));
            }
            Some(value) => text(value).unwrap_or_else(|| {
                // The game reads these without a content-log line, so the crate reports the
                // coercion as an information.
                diagnostics.push(
                    Diagnostic::info(DiagnosticCode::InvalidType { text: None })
                        .at(child(at, "text")),
                );
                DialogueText::Plain(
                    match value.as_bool() {
                        Some(true) => "true",
                        Some(false) => "false",
                        None => "",
                    }
                    .to_owned(),
                )
            }),
        };
        // Anything but an array is ignored.
        let buttons_at = child(at, "buttons");
        let buttons: Option<Vec<Button>> =
            member(scene, "buttons")
                .and_then(Value::as_array)
                .map(|buttons| {
                    buttons
                        .iter()
                        .enumerate()
                        .filter_map(|(index, button)| {
                            self.button(button, &element(&buttons_at, index), &mut diagnostics)
                        })
                        .collect()
                });
        // The count is taken after every button has been read, so dropped buttons do not count.
        if buttons
            .as_ref()
            .is_some_and(|kept| kept.len() > MAX_NUM_BUTTONS)
        {
            let code = DiagnosticCode::InvalidValue {
                text: formatted(MAX_BUTTONS, &[self.stem, &MAX_NUM_BUTTONS.to_string()]),
            };
            return dropped(Some(tag), diagnostics, code, buttons_at);
        }
        let on_open_commands = self.scene_commands(scene, "on_open_commands", at, &mut diagnostics);
        let on_close_commands =
            self.scene_commands(scene, "on_close_commands", at, &mut diagnostics);
        // A scene that parsed with an empty tag is dropped without a message.
        if tag.is_empty() {
            return SceneOutcome::Untagged(diagnostics);
        }
        let extra = unknown(scene, SCENE_MEMBERS, at, &mut diagnostics);
        SceneOutcome::Loaded(
            DialogueScene {
                scene_tag: DialogueSceneTag::parse(tag),
                npc_name,
                text,
                buttons,
                on_open_commands,
                on_close_commands,
                extra,
            },
            diagnostics,
        )
    }

    /// One button element; `None` when it is dropped: with a kept error by a rule, silently for
    /// an empty name. The game logs each of these (a missing name, both commands and a URL, a
    /// command that is no string, a URL that is no string) at the content log's error level and
    /// skips the button while the scene goes on, so each finding is a kept error.
    fn button(
        &self,
        button: &Value,
        at: &JsonPath,
        diagnostics: &mut Diagnostics,
    ) -> Option<Button> {
        let mut warn = |code: DiagnosticCode, at: JsonPath| {
            diagnostics.push_kept_error(Diagnostic::error(code).at(at));
        };
        // An absent or null name and one that is neither a string nor an object share one text.
        let Some(name) = member(button, "name") else {
            let text = formatted(NO_BUTTON_NAME, &[self.stem]);
            warn(
                DiagnosticCode::MissingRequiredField { text },
                child(at, "name"),
            );
            return None;
        };
        let Some(name) = text(name) else {
            let text = formatted(NO_BUTTON_NAME, &[self.stem]);
            warn(DiagnosticCode::InvalidType { text }, child(at, "name"));
            return None;
        };
        // Both present and non-null, checked before either type is looked at.
        let commands = member(button, "commands");
        let url = member(button, "url");
        if commands.is_some() && url.is_some() {
            let text = formatted(COMMANDS_AND_URL, &[self.stem]);
            warn(DiagnosticCode::InvalidValue { text }, at.clone());
            return None;
        }
        // A non-array is ignored; a non-string element drops the button.
        let commands = match commands.and_then(Value::as_array) {
            None => None,
            Some(list) => {
                let commands_at = child(at, "commands");
                let mut kept = Vec::new();
                for (index, command) in list.iter().enumerate() {
                    let Some(raw) = command.as_string() else {
                        let text = formatted(COMMAND_TYPE, &[self.stem]);
                        warn(
                            DiagnosticCode::InvalidType { text },
                            element(&commands_at, index),
                        );
                        return None;
                    };
                    kept.extend(self.command(raw.as_str()));
                }
                Some(kept)
            }
        };
        // An absent or null `url` is none; any other type than a string drops the button.
        let url = match url {
            None => None,
            Some(url) => {
                let Some(url) = url.as_string() else {
                    let text = formatted(URL_TYPE, &[self.stem]);
                    warn(DiagnosticCode::InvalidType { text }, child(at, "url"));
                    return None;
                };
                Some(url.as_str().to_owned())
            }
        };
        // A button that parsed with an empty name is dropped without a message.
        if matches!(&name, DialogueText::Plain(name) if name.is_empty()) {
            return None;
        }
        let button = Button {
            name,
            commands,
            url,
            extra: unknown(button, &["name", "commands", "url"], at, diagnostics),
        };
        if button.label_is_cut() {
            // The label is cut, not the name; the game says nothing, so the crate reports an
            // information.
            diagnostics.push(
                Diagnostic::info(DiagnosticCode::ClampedValue { text: None }).at(child(at, "name")),
            );
        }
        Some(button)
    }

    /// The scene's command list `key` (`on_open_commands`, `on_close_commands`): `None` when
    /// it is absent or not an array; an element that is not a string is skipped with a kept
    /// error and the rest are kept.
    fn scene_commands(
        &self,
        scene: &Value,
        key: &'static str,
        at: &JsonPath,
        diagnostics: &mut Diagnostics,
    ) -> Option<Vec<StoredCommand>> {
        let list = member(scene, key)?.as_array()?;
        let list_at = child(at, key);
        // The game's text names the list in the singular: `on_open_command`, `on_close_command`.
        let name = key.strip_suffix('s').unwrap_or(key);
        let mut kept = Vec::new();
        for (index, command) in list.iter().enumerate() {
            match command.as_string() {
                Some(raw) => kept.extend(self.command(raw.as_str())),
                // The game logs it at the content log's error level and skips the command, so
                // the finding is a kept error.
                None => diagnostics.push_kept_error(
                    Diagnostic::error(DiagnosticCode::InvalidType {
                        text: formatted(SCENE_COMMAND_TYPE, &[self.stem, name]),
                    })
                    .at(element(&list_at, index)),
                ),
            }
        }
        Some(kept)
    }

    /// One command as the game stores it: trimmed, with the pack's command version; `None` when
    /// nothing is left. A leading `/` stays.
    fn command(&self, raw: &str) -> Option<StoredCommand> {
        let raw = raw.trim_ascii();
        (!raw.is_empty()).then(|| StoredCommand {
            version: self.command_version,
            raw: raw.to_owned(),
        })
    }
}

/// A version string as the game reads it without wildcards: `"beta"` is
/// 9999.9999.9999-beta; `"*"` is unreadable; otherwise each number is `0` or up to five digits
/// without a leading zero, missing ones 0, and a pre-release or build suffix may only follow
/// the third number.
///
/// The grammar nests its groups: `-<pre>` and `+<build>` sit inside the group of the patch
/// number, so a partial version such as `1.15-beta` or `2+x` does not match.
fn version_string(text: &str) -> Option<SemVersion> {
    if text != "beta" {
        let core_end = text.find(['-', '+']).unwrap_or(text.len());
        let core = &text[..core_end];
        let canonical = |part: &str| part == "0" || (!part.starts_with('0') && part.len() <= 5);
        if !core.split('.').all(canonical) {
            return None;
        }
        let suffixed = core_end < text.len();
        if suffixed && core.split('.').count() != 3 {
            return None;
        }
    }
    SemVersion::parse(text).filter(|version| !version.is_any())
}

/// A string or an object as a dialogue text.
fn text(value: &Value) -> Option<DialogueText> {
    if let Some(text) = value.as_string() {
        return Some(DialogueText::Plain(text.as_str().to_owned()));
    }
    value.as_object().map(|_| DialogueText::Raw(value.clone()))
}

/// A scene dropped by the rule `code` at `at`, after the findings it already has.
fn dropped(
    tag: Option<String>,
    mut diagnostics: Diagnostics,
    code: DiagnosticCode,
    at: JsonPath,
) -> SceneOutcome {
    diagnostics.push(Diagnostic::error(code).at(at));
    SceneOutcome::Dropped(tag, diagnostics)
}
