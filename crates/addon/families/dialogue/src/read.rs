//! The reader: one dialogue file to its scenes, as the game's own dialogue reader reads it (file,
//! then scene, then button).
//!
//! The file's envelope is the game's sequence of checks over the parsed tree, each with its own
//! text (the version, the payload, `scenes`). Each scene is then a document of its own, read by
//! the derived read of [`DialogueScene`] and [`Button`] with the file's stem as the read's
//! subject; the hand code below the envelope is the fields' `read_with` functions and the
//! structs' guards and rule. Every finding is built here without pack and file; the loader
//! locates them.

use super::codes;
use super::command::command_version;
use super::{Button, DialogueScene, DialogueText, StoredCommand};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Code, Diagnostic, DiagnosticDetail, Diagnostics, JsonPath, Severity,
};
use bedrock_addon_core::document::Unknown;
use bedrock_addon_core::json::kit::{Elements, Failures, Members, Node, Profile, Read, UnknownAt};
use bedrock_addon_core::json::{Prepared, Value};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::schema::{
    ElementFailure, FromJson, UnknownRule, read_document, read_each, read_list,
};
use bedrock_addon_core::version::{FormatVersion, SemVersion};
use bedrock_addon_core::versioning::{NullHandling, Reading};
use facet_value::VObject;

/// The payload member.
pub(crate) const PAYLOAD: &str = "minecraft:npc_dialogue";

/// The lowest `format_version` the game loads, 1.14.0.
pub(crate) const FLOOR: FormatVersion = FormatVersion::new(1, 14, 0);

/// The most valid buttons a scene may have.
const MAX_NUM_BUTTONS: usize = 6;

/// How the game reads a dialogue file: jsoncpp's const `operator[]` and `isNull`, so a member
/// written `null` is absent, without a word; a member is of its own JSON type or it is not (no
/// scalar is converted).
const READING: Reading = Reading::NULL_ABSENT
    .with_null(NullHandling::AbsentSilently)
    .strict();

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

/// Reads one file: the file, or every finding when the whole file is skipped. `stem` (the
/// file's name without folders and extension) is the `%s` of every vanilla text; `cx` gives the
/// command version.
pub(crate) fn read(stem: &str, bytes: &[u8], cx: &LoadContext) -> Result<ReadFile, Diagnostics> {
    let reader = Reader { stem, cx };
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
    for (key, value) in read.unknown(&members, known, UnknownRule::Keep) {
        unknown.put(keep_at, key, value.clone());
    }
}

/// An error at `at`.
fn error(read: &mut Read<'_>, code: Code, at: JsonPath) {
    read.push(Diagnostic::error(code).at(at));
}

/// The state of reading one file.
struct Reader<'s> {
    /// The file's stem.
    stem: &'s str,
    /// The load the file is read in: the pack's command version.
    cx: &'s LoadContext,
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
            error(read, codes::NO_PAYLOAD.with([self.stem]), payload_at);
            return None;
        };
        let scenes_at = payload_at.child("scenes");
        // One text for both an absent and a non-array `scenes`; a payload that is not an object
        // reads `scenes` as null.
        let Some(scenes) = member(read, &payload, "scenes") else {
            error(
                read,
                codes::NO_SCENES_MISSING_REQUIRED_FIELD.with([self.stem]),
                scenes_at,
            );
            return None;
        };
        let Some(scenes) = elements(read, &scenes) else {
            error(
                read,
                codes::NO_SCENES_INVALID_TYPE.with([self.stem]),
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
            Some(text) => {
                bedrock_addon_core::json::from_str_jsonc(&bedrock_addon_core::json::last_wins(text))
                    .map_err(|error| error.kind.to_string())
            }
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
                codes::NO_FORMAT_VERSION_MISSING_FORMAT_VERSION.with([self.stem]),
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
                codes::NO_FORMAT_VERSION_MALFORMED_FORMAT_VERSION.with([self.stem]),
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
            error(read, codes::OUT_OF_RANGE.with([self.stem, &floor]), at);
            return None;
        }
        Some(version)
    }

    /// One scene element, read as a document of its own: the derived read of
    /// [`DialogueScene`], the stem the subject of every text. A value that is no object is read
    /// as one without members (jsoncpp reads every member of it as `null`).
    fn scene(&self, scene: &Node<'_>) -> SceneOutcome {
        let empty = empty_object();
        let scene = jsoncpp_object(scene, &empty);
        let mut diagnostics = Diagnostics::default();
        let mut read = kit(&mut diagnostics)
            .profile(SCENE)
            .subject(self.stem)
            .load(self.cx);
        let read_scene = read_document::<DialogueScene>(&scene, &mut read);
        drop(read);
        match read_scene {
            Some(read_scene) => {
                // The unknown members of the scene and of its buttons, at paths from the scene.
                diagnostics
                    .unknown_mut()
                    .rebase(scene.path(), &JsonPath::new());
                SceneOutcome::Loaded(read_scene, diagnostics)
            }
            // A dropped scene keeps no unknown member (the read keeps none of a value it fails).
            None if diagnostics.iter().any(Diagnostic::rejects) => {
                let tag = dropped_tag(&scene, &diagnostics);
                SceneOutcome::Dropped(tag, diagnostics)
            }
            // A scene whose read failed without an error is one of an empty tag (`tagged`).
            None => SceneOutcome::Untagged(diagnostics),
        }
    }
}

/// How a scene is read: its members in the game's order (the order [`DialogueScene`] declares),
/// the first rule it breaks dropping it and ending its checks, the members the game never reads
/// told after the others.
const SCENE: Profile = Profile::DECLARED_ORDER
    .with_failures(Failures::First)
    .with_unknown(UnknownAt::Last);

/// An object without members, for a value jsoncpp reads as one.
fn empty_object() -> Value {
    Value::from(VObject::new())
}

/// `node` as jsoncpp's const `operator[]` sees it: itself when it is an object, else `empty`
/// at its path (every member of a value that is no object reads as `null`).
fn jsoncpp_object<'v>(node: &Node<'v>, empty: &'v Value) -> Node<'v> {
    if node.value().is_object() {
        node.clone()
    } else {
        Node::new(empty, node.path().clone())
    }
}

/// The tag a dropped scene is rejected under: its `scene_tag` when that is a string and the read
/// came past it (the rule that dropped the scene is about neither the name nor the tag, the two
/// members the game reads first).
fn dropped_tag(scene: &Node<'_>, diagnostics: &Diagnostics) -> Option<String> {
    let dropping = diagnostics.iter().find(|finding| finding.rejects())?;
    let at = dropping.location.json_path.last_key();
    if matches!(at, Some("npc_name" | "scene_tag")) {
        return None;
    }
    let tag = scene.value().as_object()?.get("scene_tag")?.as_string()?;
    Some(tag.as_str().to_owned())
}

/// The member `key` of the object `node` when it is written and not `null`.
fn written<'v>(node: &Node<'v>, key: &str) -> Option<&'v Value> {
    node.value()
        .as_object()?
        .get(key)
        .filter(|value| !value.is_null())
}

/// The subject of the read, the file's stem every text names.
fn stem<'d>(read: &Read<'d>) -> &'d str {
    read.subject_text().unwrap_or_default()
}

/// `schema::guard` of a scene: a scene that parsed with an empty tag is dropped without a word.
pub(crate) fn tagged(scene: &Node<'_>, _: &mut Read<'_>) -> bool {
    written(scene, "scene_tag")
        .and_then(Value::as_string)
        .is_none_or(|tag| !tag.is_empty())
}

/// `schema::guard` of a button, before its commands: a button with both commands and a URL
/// (both present and not `null`, checked before either type is looked at) is dropped.
pub(crate) fn one_action(button: &Node<'_>, read: &mut Read<'_>) -> bool {
    let both = written(button, "commands").is_some() && written(button, "url").is_some();
    if both {
        read.refuse(button, codes::COMMANDS_AND_URL.with([stem(read)]));
    }
    !both
}

/// `schema::guard` of a button, after its members: a button that parsed with an empty name is
/// dropped without a word.
pub(crate) fn named(button: &Node<'_>, _: &mut Read<'_>) -> bool {
    written(button, "name")
        .and_then(Value::as_string)
        .is_none_or(|name| !name.is_empty())
}

/// `schema::after_read` of a button: a label the game cuts. The label is cut, not the name; the
/// game says nothing, so the crate reports an information.
pub(crate) fn label_rule(button: &mut Button, node: &Node<'_>, read: &mut Read<'_>) {
    if button.label_is_cut() {
        let mut finding = Diagnostic::of(&core_codes::CLAMPED_VALUE).at(node.path().child("name"));
        finding.location.span = read
            .object(node)
            .and_then(|members| members.get("name"))
            .and_then(|name| name.span());
        read.push(finding);
    }
}

/// The scene's `text`: a string or an object as written; anything else is read as a string with
/// the empty default, a bool as its word, a number or an array as the empty text.
#[allow(clippy::unnecessary_wraps, reason = "the `read_with` signature")]
pub(crate) fn scene_text(node: &Node<'_>, read: &mut Read<'_>) -> Option<DialogueText> {
    if let Some(text) = text(node.value()) {
        return Some(text);
    }
    // The game reads these without a content-log line, so the crate reports the coercion as an
    // information.
    let mut finding = Diagnostic::of(&core_codes::INVALID_TYPE)
        .severity(Severity::Info)
        .at(node.path().clone());
    finding.location.span = node.span();
    read.push(finding);
    let word = match node.value().as_bool() {
        Some(true) => "true",
        Some(false) => "false",
        None => "",
    };
    Some(DialogueText::Plain(word.to_owned()))
}

/// The scene's buttons: each read by itself, the first rule one breaks skipping it (the scene
/// goes on); more than six that stay drop the scene. A button that is no object is read as one
/// without members. Each is kept at the index it has among the buttons kept.
pub(crate) fn buttons(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<Button>> {
    let empty = empty_object();
    let buttons = read_list(node, read, ElementFailure::Unit, |button, read| {
        Button::from_json(&jsoncpp_object(button, &empty), read)
    })?;
    // The count is taken after every button has been read, so dropped buttons do not count.
    if buttons.len() > MAX_NUM_BUTTONS {
        let most = MAX_NUM_BUTTONS.to_string();
        read.refuse(node, codes::MAX_BUTTONS.with([stem(read), &most]));
        return None;
    }
    Some(buttons)
}

/// A button's `commands`: a command that is no string drops the button.
pub(crate) fn button_commands(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<StoredCommand>> {
    let commands = read_each(node, read, |command, read| {
        if command.value().as_string().is_none() {
            read.refuse(command, codes::COMMAND_TYPE.with([stem(read)]));
            return None;
        }
        StoredCommand::from_json(command, read)
    })?;
    Some(kept(commands))
}

/// The scene's `on_open_commands` or `on_close_commands`: a command that is no string is
/// skipped with a kept error (the game logs it at the content log's error level), the rest kept.
pub(crate) fn scene_commands(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<StoredCommand>> {
    // The game's text names the list in the singular: `on_open_command`, `on_close_command`.
    let key = node.path().last_key().unwrap_or_default();
    let name = key.strip_suffix('s').unwrap_or(key);
    let commands = read_each(node, read, |command, read| {
        if command.value().as_string().is_none() {
            let mut finding = Diagnostic::of(codes::SCENE_COMMAND_TYPE.with([stem(read), name]))
                .at(command.path().clone());
            finding.location.span = command.span();
            read.push(finding);
            return Some(None);
        }
        StoredCommand::from_json(command, read).map(Some)
    })?;
    Some(kept(commands.into_iter().flatten().collect()))
}

/// The commands the game keeps: those with text left once trimmed.
fn kept(mut commands: Vec<StoredCommand>) -> Vec<StoredCommand> {
    commands.retain(|command| !command.raw.is_empty());
    commands
}

/// One command as the game stores it: the string trimmed (a leading `/` stays), with the pack's
/// command version; another value is a wrong type.
impl FromJson for StoredCommand {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let raw = read.string(node)?.trim_ascii();
        let pack = read
            .load_context()
            .and_then(|cx| cx.min_engine_version.as_ref());
        Some(Self {
            version: command_version(pack),
            raw: raw.to_owned(),
        })
    }
}

/// A string or any object; another value is a wrong type (worded by the member's code).
impl FromJson for DialogueText {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        if read.stopped() {
            return None;
        }
        let text = text(node.value());
        if text.is_none() {
            read.wrong_type(node);
        }
        text
    }
}

bedrock_addon_core::impl_schema_leaf!(DialogueText, StoredCommand);
bedrock_addon_core::impl_no_molang!(DialogueText, StoredCommand);

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
