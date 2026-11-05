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
mod read;
mod texts;
mod write;

#[cfg(test)]
mod tests;

use crate::command::CURRENT;
use crate::read::{FLOOR, PAYLOAD, ReadFile, SceneOutcome, read};
use crate::texts::{DUPLICATE, formatted};
use crate::write::fast_writer;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Phase, Severity,
};
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, JsonContent, NoBeta, Origin, Placed,
    Registry, RegistryBuilder, Rejected, VersionInfo,
};
use bedrock_addon_core::id::DialogueSceneTag;
use bedrock_addon_core::id::kind;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::json::{EncodeError, Precedence, WriteStyle};
use bedrock_addon_core::load::{CollectionLoader, LoadContext, StackCx, Walk, WalkFile};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::SemVersion;
use indexmap::IndexMap;

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

/// The directory the game walks.
const DIRECTORY: &str = "dialogue";

impl bedrock_addon_core::__private::Sealed for DialogueFile {}
impl bedrock_addon_core::__private::Sealed for DialogueScene {}

impl JsonContent for DialogueFile {}

impl Content for DialogueFile {
    const NAME: &'static str = "dialogue";
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];
    /// The writer writes every member it knows from the value, so a member of the same key in
    /// the table is never written over it.
    const UNKNOWN_PRECEDENCE: Precedence = Precedence::KnownWins;

    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        let origin = origin(PackRef::app_data(), PackSide::Behavior, path, None);
        let file = match read(&stem(path), bytes, cx) {
            Ok(file) => file,
            Err(diagnostics) => {
                return Err(Rejected::located(origin, diagnostics));
            }
        };
        // The unknown members of the root and of the payload are in the file's table; each
        // stored scene's are added below its index among the stored ones.
        let mut diagnostics = file.diagnostics;
        let scenes_at = JsonPath::new().child(PAYLOAD).child("scenes");
        let mut scenes: Vec<DialogueScene> = Vec::new();
        for scene in file.scenes {
            match scene.outcome {
                SceneOutcome::Loaded(loaded, mut found) => {
                    let kept = found.take_unknown();
                    diagnostics.extend(found);
                    // The first scene of a tag is the one stored; a later one is dropped like
                    // any other scene a rule drops. The game logs it at the content log's error
                    // level and goes on with the next scene, so the finding is a kept error.
                    if scenes.iter().any(|kept| kept.scene_tag == loaded.scene_tag) {
                        diagnostics.push_kept_error(duplicate(
                            &origin,
                            loaded.scene_tag.as_str(),
                            &scene.at,
                        ));
                    } else {
                        diagnostics
                            .unknown_mut()
                            .absorb(&scenes_at.index(scenes.len()), kept);
                        scenes.push(loaded);
                    }
                }
                // The file loads without the scene: the game skips a scene its scene reader
                // refuses and goes on with the next one, so what drops a scene is a kept error
                // here. Where the scene is the document (the registry) it is an error that
                // rejects the scene, because a document with an error is not loaded.
                SceneOutcome::Dropped(_, found) => {
                    for finding in found {
                        if finding.severity == Severity::Error {
                            diagnostics.push_kept_error(finding);
                        } else {
                            diagnostics.push(finding);
                        }
                    }
                }
                SceneOutcome::Untagged(found) => diagnostics.extend(found),
            }
        }
        diagnostics.locate(&origin);
        let version = version_info(&file.format_version);
        let value = DialogueFile {
            format_version: file.format_version,
            scenes,
        };
        Ok(Document::new(value, origin, version, diagnostics))
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        self.write_into(out, style)
    }
}

impl Definition for DialogueScene {
    type Kind = kind::DialogueScene;
    type File = DialogueFile;
    type Beta = NoBeta<kind::DialogueScene>;
}

impl Placed for DialogueFile {
    /// `dialogue/<first scene tag>.json`, or `dialogue/dialogue.json` for a file without
    /// scenes.
    fn default_path(&self) -> PackPath {
        let fallback = DialogueSceneTag::parse(DIRECTORY);
        let tag = self
            .scenes
            .first()
            .map_or(&fallback, |scene| &scene.scene_tag);
        bedrock_addon_core::document::default_path(DIRECTORY, tag)
    }
}

/// Vanilla's "already exists" text for the scene `tag` at `at` of `file`: an error.
fn duplicate(file: &Origin, tag: &str, at: &JsonPath) -> Diagnostic {
    Diagnostic::error(DiagnosticCode::DuplicateIdentifier {
        text: formatted(DUPLICATE, &[&stem(&file.path), tag]),
    })
    .at(at.clone())
    .phase(Phase::Validate)
    .in_file(file)
}

/// The version record of a file of `version`: written and effective as read, parsed by the
/// one shape.
fn version_info(version: &SemVersion) -> VersionInfo {
    let written = version.format_version();
    VersionInfo::new(Some(written), false, written, FLOOR, false)
}

/// The stem of a file: its last segment without the extension.
pub(crate) fn stem(path: &PackPath) -> String {
    let name = path.segments().last().unwrap_or_default();
    match name.rsplit_once('.') {
        Some((stem, _)) => stem.to_owned(),
        None => name.to_owned(),
    }
}

/// The origin of a file of `pack`, or of the scene `key` in it.
fn origin(pack: PackRef, side: PackSide, path: &PackPath, key: Option<&str>) -> Origin {
    Origin::new(pack, side, path.clone(), key.map(Into::into), Vec::new())
}

// ---- the collection ----------------------------------------------------------------------

/// `dialogue/**/*.json`, packs bottom-up (the game walks the stack from its last element, the
/// lowest priority, to the first).
const WALK: Walk = Walk::json_tree_in(&[DIRECTORY]);

/// One file's part of the registry.
#[doc(hidden)]
pub struct Parsed {
    /// The file's origin.
    origin: Origin,
    /// What reading it gave: the file or the findings that skip it; `None` for a file
    /// the game's walk does not reach.
    read: Option<Result<ReadFile, Diagnostics>>,
}

/// Reads one walked file.
fn parse_walk_file(file: &WalkFile, cx: &LoadContext) -> Parsed {
    // The game's directory walk recurses into sub-folders unless the pack lives in the app
    // package, so the packs the game ships are walked flat and every other pack with its
    // sub-folders. `is_base_game_pack` is the closest fact the load context carries for "shipped
    // inside the app".
    let nested = file.key.segments().count() > 2;
    let reached = !(cx.is_base_game_pack && nested);
    Parsed {
        origin: origin(file.pack.clone(), file.side, &file.path, None),
        read: reached.then(|| read(&stem(&file.path), &file.bytes, cx)),
    }
}

/// The registry: every scene in walk order, the first one stored under a tag winning
/// (the game keeps its scenes in one map keyed by tag, filled lowest-priority pack first).
fn combine(parsed: Vec<Parsed>, stack: &StackCx<'_>) -> Registry<DialogueScene> {
    let mut builder = RegistryBuilder::new(
        CombineRule::bottom_wins(),
        CombineRule::bottom_wins(),
        stack.options,
    );
    // Which pack stored each tag.
    let mut stored: IndexMap<DialogueSceneTag, PackRef> = IndexMap::new();
    for Parsed {
        origin: file_origin,
        read,
    } in parsed
    {
        let Some(read) = read else {
            continue;
        };
        let file = match read {
            Ok(file) => file,
            Err(diagnostics) => {
                builder.push(Contribution::Rejected(
                    None,
                    Rejected::located(file_origin, diagnostics),
                ));
                continue;
            }
        };
        let mut file_diagnostics = file.diagnostics;
        // The file is no document here: the unknown members of its root and payload go with it.
        drop(file_diagnostics.take_unknown());
        file_diagnostics.locate(&file_origin);
        for finding in file_diagnostics {
            builder.push_diagnostic(finding);
        }
        let version = version_info(&file.format_version);
        for scene in file.scenes {
            push_scene(
                &mut builder,
                &mut stored,
                &file_origin,
                &version,
                scene.outcome,
                &scene.at,
            );
        }
    }
    builder.finish()
}

/// Adds one scene of a file.
fn push_scene(
    builder: &mut RegistryBuilder<DialogueScene>,
    stored: &mut IndexMap<DialogueSceneTag, PackRef>,
    file: &Origin,
    version: &VersionInfo,
    outcome: SceneOutcome,
    at: &JsonPath,
) {
    match outcome {
        SceneOutcome::Loaded(scene, mut diagnostics) => {
            let origin = origin(
                file.pack.clone(),
                file.side,
                &file.path,
                Some(scene.scene_tag.as_str()),
            );
            diagnostics.locate(&origin);
            let tag = scene.scene_tag.clone();
            // The game rejects a tag that is already stored with a message; with the stack
            // walked lowest priority first, the lower pack's scene stays.
            match stored.get(&tag) {
                None => {
                    stored.insert(tag.clone(), file.pack.clone());
                    let document = Document::new(scene, origin, version.clone(), diagnostics);
                    builder.push(Contribution::Stable(tag, document));
                }
                // A second scene of the pack that stored the tag: not loaded.
                Some(first) if *first == file.pack => {
                    drop(diagnostics.take_unknown());
                    diagnostics.push(duplicate(file, tag.as_str(), at));
                    builder.push(Contribution::Rejected(
                        Some(tag),
                        Rejected::new(origin, diagnostics),
                    ));
                }
                // A higher-priority pack's scene: the game's error, and the registry lists it
                // as shadowed (the crate's own `ShadowedDefinition`, raised by the combine
                // rule).
                Some(_) => {
                    let error = duplicate(file, tag.as_str(), at);
                    let document = Document::new(scene, origin, version.clone(), diagnostics);
                    builder.push(Contribution::Stable(tag, document));
                    builder.push_diagnostic(error);
                }
            }
        }
        SceneOutcome::Dropped(tag, diagnostics) => {
            let origin = origin(file.pack.clone(), file.side, &file.path, tag.as_deref());
            builder.push(Contribution::Rejected(
                tag.map(DialogueSceneTag::parse),
                Rejected::located(origin, diagnostics),
            ));
        }
        SceneOutcome::Untagged(mut diagnostics) => {
            diagnostics.locate(file);
            for finding in diagnostics {
                builder.push_diagnostic(finding);
            }
        }
    }
}

/// The loader of `bp.dialogue()`: `dialogue/**/*.json` of every behaviour pack, the
/// lowest-priority pack's scene winning. The family's own: a file holds several scenes (the
/// entries keep a hand `Definition`, `File = DialogueFile`), which `load::registry` does not
/// take.
#[doc(hidden)]
pub const LOADER: CollectionLoader<Registry<DialogueScene>, Parsed> = CollectionLoader {
    walk: WALK,
    parse: parse_walk_file,
    combine,
    needs: &[], // combining reads no other collection
    then: None, // every file is named by the walk itself; there is no second stage
};

impl DialogueText {
    /// The text as vanilla stores it: a plain string as written, an object as compact JSON
    /// followed by one newline (`Json::FastWriter`), `{"rawtext":[{"text":"Hi"}]}\n`.
    ///
    /// The compact form and the newline are vanilla's. Two details of an object's text are
    /// inferred, not read from the game: its members are written in key order (bytewise), as
    /// jsoncpp's sorted member map would give them, and numbers are spelled as this crate
    /// writes them (integers as integers, reals in the shortest form). An object holding a
    /// number JSON cannot spell gives the empty text.
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
    pub fn new(scenes: Vec<DialogueScene>) -> DialogueFile {
        DialogueFile {
            format_version: SemVersion::from(FLOOR),
            scenes,
        }
    }
}

impl DialogueScene {
    /// A scene with this tag and text: no name, no buttons, no commands.
    pub fn new(scene_tag: DialogueSceneTag, text: DialogueText) -> DialogueScene {
        DialogueScene {
            scene_tag,
            npc_name: None,
            text,
            buttons: None,
            on_open_commands: None,
            on_close_commands: None,
        }
    }
}

impl Button {
    /// A button with this name: no commands, no URL.
    pub fn new(name: DialogueText) -> Button {
        Button {
            name,
            commands: None,
            url: None,
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
