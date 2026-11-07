//! The reader: one dialogue file to its scenes, walking the parsed JSON tree the way the game's
//! own dialogue reader does (file, then scene, then button).
//!
//! Every finding is built here without pack and file; the loader locates them.

use super::codes;
use super::command::command_version;
use super::{Button, DialogueScene, DialogueText, StoredCommand};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticDetail, Diagnostics, JsonPath, Severity,
};
use bedrock_addon_core::document::Unknown;
use bedrock_addon_core::id::DialogueSceneTag;
use bedrock_addon_core::json::kit::{Disposition, Elements, Members, Node, Read};
use bedrock_addon_core::json::{Prepared, Value};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::version::{FormatVersion, SemVersion};
use bedrock_addon_core::versioning::Reading;

/// The payload member.
pub(crate) const PAYLOAD: &str = "minecraft:npc_dialogue";

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

/// How the game reads a dialogue file: jsoncpp's const `operator[]` and `isNull`, so a member
/// written `null` is absent.
const READING: Reading = Reading::NULL_ABSENT;

/// A file vanilla reads: its version, its scenes and the file-level findings.
pub(crate) struct ReadFile {
    /// `format_version` as read.
    pub(crate) format_version: SemVersion,
    /// Every scene element, in file order.
    pub(crate) scenes: Vec<ReadScene>,
    /// File-level warnings and infos, with the unknown members of the root and of
    /// `minecraft:npc_dialogue` in their table.
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
    /// It loaded, with its warnings and infos; the unknown members of the scene and of its
    /// buttons are in their table, at paths from the scene.
    Loaded(DialogueScene, Diagnostics),
    /// A rule dropped it: the tag when it had a string one, and every finding (the dropping one
    /// at error level).
    Dropped(Option<String>, Diagnostics),
    /// It parsed with an empty tag, which vanilla drops silently; its warnings.
    Untagged(Diagnostics),
}

/// What the read of one scene came to, before its findings are attached.
enum Scene {
    /// It loaded.
    Loaded(DialogueScene),
    /// The rule `code` at the path dropped it; the tag when it had a string one.
    Dropped(Option<String>, DiagnosticCode, JsonPath),
    /// It parsed with an empty tag.
    Untagged,
}

/// Reads one file: the file, or every finding when the whole file is skipped. `stem` (the
/// file's name without folders and extension) is the `%s` of every vanilla text; `cx` gives the
/// command version.
pub(crate) fn read(stem: &str, bytes: &[u8], cx: &LoadContext) -> Result<ReadFile, Diagnostics> {
    let reader = Reader {
        stem,
        command_version: command_version(cx.min_engine_version.as_ref()),
    };
    // A number no double holds is read as the largest finite one, with a warning, so the
    // document can be written back.
    let finite = bedrock_addon_core::document::FiniteText::of(bytes);
    let mut diagnostics = Diagnostics::default();
    let mut unknown = Unknown::new();
    let file = reader.file(finite.bytes(), &mut kit(&mut diagnostics), &mut unknown);
    finite.settle(&mut diagnostics, None);
    match file {
        Some((format_version, scenes)) => {
            *diagnostics.unknown_mut() = unknown;
            Ok(ReadFile {
                format_version,
                scenes,
                diagnostics,
            })
        }
        None => Err(diagnostics),
    }
}

/// A read telling into `findings`: an unknown member is an information (the game ignores it,
/// so the crate's own finding says so).
fn kit(findings: &mut Diagnostics) -> Read<'_> {
    Read::new(READING, findings).unknown_severity(Severity::Info)
}

/// The members of `node` when it is an object; nothing is told when it is not (jsoncpp reads
/// every member of a value that is no object as `null`).
fn members<'v>(read: &mut Read<'_>, node: &Node<'v>) -> Option<Members<'v>> {
    if node.value().is_object() {
        read.object(node)
    } else {
        None
    }
}

/// The member `key` of `node`, as jsoncpp's const `operator[]` and `isNull` see it: `None` when
/// `node` is not an object, the member is absent, or it is `null`.
fn member<'v>(read: &mut Read<'_>, node: &Node<'v>, key: &str) -> Option<Node<'v>> {
    let members = members(read, node)?;
    read.opt(&members, key)
}

/// The elements of `node` when it is an array; nothing is told when it is not.
fn elements<'v>(read: &mut Read<'_>, node: &Node<'v>) -> Option<Elements<'v>> {
    if node.value().is_array() {
        read.array(node)
    } else {
        None
    }
}

/// Tells the members of the object `node` vanilla never reads (every one but `known`), each
/// an information, and keeps each in `unknown` at `keep_at` (the object's path in the
/// document) for write-back. A value that is not an object has none.
fn unknown(
    read: &mut Read<'_>,
    node: &Node<'_>,
    known: &[&str],
    unknown: &mut Unknown,
    keep_at: &JsonPath,
) {
    let Some(members) = members(read, node) else {
        return;
    };
    for (key, value) in read.unknown(&members, known, Disposition::Keep) {
        unknown.put(keep_at, key, value.clone());
    }
}

/// An error at `at`.
fn error(read: &mut Read<'_>, code: DiagnosticCode, at: JsonPath) {
    read.push(Diagnostic::error(code).at(at));
}

/// The state of reading one file.
struct Reader<'s> {
    /// The file's stem.
    stem: &'s str,
    /// The pack's command version.
    command_version: u8,
}

impl Reader<'_> {
    /// The file's version and scenes, or `None` when it is skipped (the findings say why). The
    /// unknown members of the root and of the payload go in `kept`.
    fn file(
        &self,
        bytes: &[u8],
        read: &mut Read<'_>,
        kept: &mut Unknown,
    ) -> Option<(SemVersion, Vec<ReadScene>)> {
        let root = self.json(bytes, read)?;
        let root = Node::new(&root, JsonPath::new());
        let format_version = self.version(&root, read)?;
        let payload_at = JsonPath::new().child(PAYLOAD);
        // Only a null check: the payload's type is not looked at.
        let Some(payload) = member(read, &root, PAYLOAD) else {
            error(
                read,
                DiagnosticCode::from(codes::NO_PAYLOAD.with([self.stem])),
                payload_at,
            );
            return None;
        };
        let scenes_at = payload_at.child("scenes");
        // One text for both an absent and a non-array `scenes`; a payload that is not an object
        // reads `scenes` as null.
        let Some(scenes) = member(read, &payload, "scenes") else {
            error(
                read,
                DiagnosticCode::from(codes::NO_SCENES_MISSING_REQUIRED_FIELD.with([self.stem])),
                scenes_at,
            );
            return None;
        };
        let Some(scenes) = elements(read, &scenes) else {
            error(
                read,
                DiagnosticCode::from(codes::NO_SCENES_INVALID_TYPE.with([self.stem])),
                scenes_at,
            );
            return None;
        };
        let scenes = scenes
            .iter()
            .map(|scene| {
                let at = scene.path().clone();
                let outcome = self.scene(&scene);
                ReadScene { at, outcome }
            })
            .collect();
        unknown(
            read,
            &root,
            &["format_version", PAYLOAD],
            kept,
            &JsonPath::new(),
        );
        unknown(read, &payload, &["scenes"], kept, &payload_at);
        Some((format_version, scenes))
    }

    /// The parsed root, comments allowed (`Json::Reader` with `allowComments`).
    fn json(&self, bytes: &[u8], read: &mut Read<'_>) -> Option<Value> {
        // A leading byte-order mark is blanked, not cut, so every offset stays an offset into the
        // file as written.
        let prepared = Prepared::new(bytes).ok();
        let parsed = match prepared.as_ref().map(Prepared::as_str) {
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
                read.push(
                    Diagnostic::of(codes::READER.with([self.stem, &message]))
                        .detail(DiagnosticDetail::Syntax(message))
                        .at(JsonPath::new()),
                );
                return None;
            }
        };
        if root.as_object().is_none() {
            read.push(
                Diagnostic::of(codes::ROOT_NOT_OBJECT.with([self.stem]))
                    .detail(DiagnosticDetail::Syntax(
                        "the root is not an object".to_owned(),
                    ))
                    .at(JsonPath::new()),
            );
            return None;
        }
        Some(root)
    }

    /// `format_version`, at 1.14.0 or above, with no maximum (`NPC_DIALOGUE_V1_14`).
    fn version(&self, root: &Node<'_>, read: &mut Read<'_>) -> Option<SemVersion> {
        let at = JsonPath::new().child("format_version");
        let Some(written) = member(read, root, "format_version") else {
            error(
                read,
                DiagnosticCode::from(
                    codes::NO_FORMAT_VERSION_MISSING_FORMAT_VERSION.with([self.stem]),
                ),
                at,
            );
            return None;
        };
        let written = written.value();
        // A version is a string (no wildcards), or an array of exactly three elements
        // convertible to unsigned; anything else is unreadable and gets the same text as absent.
        let version = match written.as_string() {
            Some(text) => version_string(text.as_str()),
            None => written
                .as_array()
                // Core's reader of the array form, shared with the manifest.
                .and_then(|elements| SemVersion::from_json_array(elements.as_slice())),
        };
        let Some(version) = version else {
            error(
                read,
                DiagnosticCode::from(
                    codes::NO_FORMAT_VERSION_MALFORMED_FORMAT_VERSION.with([self.stem]),
                ),
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
            read.push(
                Diagnostic::of(&core_codes::INVALID_VALUE)
                    .severity(Severity::Info)
                    .at(at.clone()),
            );
        }
        // A plain `<` against the floor: 1.14.0 itself loads.
        if version < SemVersion::from(FLOOR) {
            let floor = FLOOR.to_string();
            error(
                read,
                DiagnosticCode::from(codes::OUT_OF_RANGE.with([self.stem, &floor])),
                at,
            );
            return None;
        }
        Some(version)
    }

    /// One scene element, with its own findings.
    fn scene(&self, scene: &Node<'_>) -> SceneOutcome {
        let mut diagnostics = Diagnostics::default();
        let mut kept = Unknown::new();
        let read = self.scene_members(scene, &mut kit(&mut diagnostics), &mut kept);
        match read {
            Scene::Loaded(scene) => {
                *diagnostics.unknown_mut() = kept;
                SceneOutcome::Loaded(scene, diagnostics)
            }
            // A dropped scene keeps no unknown member.
            Scene::Dropped(tag, code, at) => {
                diagnostics.push(Diagnostic::error(code).at(at));
                SceneOutcome::Dropped(tag, diagnostics)
            }
            Scene::Untagged => SceneOutcome::Untagged(diagnostics),
        }
    }

    /// The members of one scene element; the unknown members of the scene and of its buttons
    /// go in `kept`, at paths from the scene.
    fn scene_members(&self, scene: &Node<'_>, read: &mut Read<'_>, kept: &mut Unknown) -> Scene {
        let at = scene.path();
        // The game's order: `npc_name`, `scene_tag`, `text`, `buttons` and their count, the open
        // and the close commands; the first failing rule drops the scene and ends the checks.
        // A null or absent name is the empty name; a string or an object is a name; anything else
        // drops the scene.
        let npc_name = match member(read, scene, "npc_name") {
            None => None,
            Some(name) => {
                let Some(name) = text(name.value()) else {
                    let code = DiagnosticCode::from(codes::NPC_NAME.with([self.stem]));
                    return Scene::Dropped(None, code, at.child("npc_name"));
                };
                Some(name)
            }
        };
        // Absent, null or any other type than a string: one text.
        let Some(tag) = member(read, scene, "scene_tag") else {
            let code =
                DiagnosticCode::from(codes::NO_SCENE_TAG_MISSING_REQUIRED_FIELD.with([self.stem]));
            return Scene::Dropped(None, code, at.child("scene_tag"));
        };
        let Some(tag) = tag.value().as_string() else {
            let code = DiagnosticCode::from(codes::NO_SCENE_TAG_INVALID_TYPE.with([self.stem]));
            return Scene::Dropped(None, code, at.child("scene_tag"));
        };
        let tag = tag.as_str().to_owned();
        // An object is kept as its JSON text and anything else is read as a string with the empty
        // default: a bool reads as its word, a number or an array as the empty text.
        let text = match member(read, scene, "text") {
            None => {
                let code = DiagnosticCode::from(codes::NO_TEXT.with([self.stem]));
                return Scene::Dropped(Some(tag), code, at.child("text"));
            }
            Some(node) => text(node.value()).unwrap_or_else(|| {
                // The game reads these without a content-log line, so the crate reports the
                // coercion as an information.
                read.push(
                    Diagnostic::of(&core_codes::INVALID_TYPE)
                        .severity(Severity::Info)
                        .at(at.child("text")),
                );
                DialogueText::Plain(
                    match node.value().as_bool() {
                        Some(true) => "true",
                        Some(false) => "false",
                        None => "",
                    }
                    .to_owned(),
                )
            }),
        };
        // Anything but an array is ignored.
        let buttons_at = at.child("buttons");
        let buttons: Option<Vec<Button>> = match member(read, scene, "buttons") {
            Some(buttons) => elements(read, &buttons).map(|buttons| {
                let mut read_buttons = Vec::new();
                for button in buttons.iter() {
                    // Kept at the index the button is written at: dropped ones do not count.
                    let keep_at = JsonPath::new().child("buttons").index(read_buttons.len());
                    read_buttons.extend(self.button(&button, read, kept, &keep_at));
                }
                read_buttons
            }),
            None => None,
        };
        // The count is taken after every button has been read, so dropped buttons do not count.
        if buttons
            .as_ref()
            .is_some_and(|kept| kept.len() > MAX_NUM_BUTTONS)
        {
            let code = DiagnosticCode::from(
                codes::MAX_BUTTONS.with([self.stem, &MAX_NUM_BUTTONS.to_string()]),
            );
            return Scene::Dropped(Some(tag), code, buttons_at);
        }
        let on_open_commands = self.scene_commands(scene, "on_open_commands", read);
        let on_close_commands = self.scene_commands(scene, "on_close_commands", read);
        // A scene that parsed with an empty tag is dropped without a message.
        if tag.is_empty() {
            return Scene::Untagged;
        }
        unknown(read, scene, SCENE_MEMBERS, kept, &JsonPath::new());
        Scene::Loaded(DialogueScene {
            scene_tag: DialogueSceneTag::parse(tag),
            npc_name,
            text,
            buttons,
            on_open_commands,
            on_close_commands,
        })
    }

    /// One button element; `None` when it is dropped: with a kept error by a rule, silently for
    /// an empty name. The game logs each of these (a missing name, both commands and a URL, a
    /// command that is no string, a URL that is no string) at the content log's error level and
    /// skips the button while the scene goes on, so each finding is a kept error. Its unknown
    /// members go in `kept` at `keep_at`.
    fn button(
        &self,
        button: &Node<'_>,
        read: &mut Read<'_>,
        kept: &mut Unknown,
        keep_at: &JsonPath,
    ) -> Option<Button> {
        let at = button.path();
        let warn = |read: &mut Read<'_>, code: DiagnosticCode, at: JsonPath| {
            read.push(Diagnostic::error(code).at(at).into_kept());
        };
        // An absent or null name and one that is neither a string nor an object share one text.
        let Some(name) = member(read, button, "name") else {
            warn(
                read,
                DiagnosticCode::from(
                    codes::NO_BUTTON_NAME_MISSING_REQUIRED_FIELD.with([self.stem]),
                ),
                at.child("name"),
            );
            return None;
        };
        let Some(name) = text(name.value()) else {
            warn(
                read,
                DiagnosticCode::from(codes::NO_BUTTON_NAME_INVALID_TYPE.with([self.stem])),
                at.child("name"),
            );
            return None;
        };
        // Both present and non-null, checked before either type is looked at.
        let commands = member(read, button, "commands");
        let url = member(read, button, "url");
        if commands.is_some() && url.is_some() {
            warn(
                read,
                DiagnosticCode::from(codes::COMMANDS_AND_URL.with([self.stem])),
                at.clone(),
            );
            return None;
        }
        // A non-array is ignored; a non-string element drops the button.
        let commands = match commands.and_then(|commands| elements(read, &commands)) {
            None => None,
            Some(list) => {
                let mut stored = Vec::new();
                for command in list.iter() {
                    let Some(raw) = command.value().as_string() else {
                        warn(
                            read,
                            DiagnosticCode::from(codes::COMMAND_TYPE.with([self.stem])),
                            command.path().clone(),
                        );
                        return None;
                    };
                    stored.extend(self.command(raw.as_str()));
                }
                Some(stored)
            }
        };
        // An absent or null `url` is none; any other type than a string drops the button.
        let url = match url {
            None => None,
            Some(url) => {
                let Some(url) = url.value().as_string() else {
                    warn(
                        read,
                        DiagnosticCode::from(codes::URL_TYPE.with([self.stem])),
                        at.child("url"),
                    );
                    return None;
                };
                Some(url.as_str().to_owned())
            }
        };
        // A button that parsed with an empty name is dropped without a message.
        if matches!(&name, DialogueText::Plain(name) if name.is_empty()) {
            return None;
        }
        unknown(read, button, &["name", "commands", "url"], kept, keep_at);
        let button = Button {
            name,
            commands,
            url,
        };
        if button.label_is_cut() {
            // The label is cut, not the name; the game says nothing, so the crate reports an
            // information.
            read.push(Diagnostic::of(&core_codes::CLAMPED_VALUE).at(at.child("name")));
        }
        Some(button)
    }

    /// The scene's command list `key` (`on_open_commands`, `on_close_commands`): `None` when
    /// it is absent or not an array; an element that is not a string is skipped with a kept
    /// error and the rest are kept.
    fn scene_commands(
        &self,
        scene: &Node<'_>,
        key: &'static str,
        read: &mut Read<'_>,
    ) -> Option<Vec<StoredCommand>> {
        let list = member(read, scene, key)?;
        let list = elements(read, &list)?;
        // The game's text names the list in the singular: `on_open_command`, `on_close_command`.
        let name = key.strip_suffix('s').unwrap_or(key);
        let mut stored = Vec::new();
        for command in list.iter() {
            match command.value().as_string() {
                Some(raw) => stored.extend(self.command(raw.as_str())),
                // The game logs it at the content log's error level and skips the command, so
                // the finding is a kept error.
                None => read.push(
                    Diagnostic::of(codes::SCENE_COMMAND_TYPE.with([self.stem, name]))
                        .at(command.path().clone()),
                ),
            }
        }
        Some(stored)
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
