//! Loading dialogue: `Content` / `Definition`, the walk of `dialogue/`, and the combine rule
//! (the first scene stored under a tag wins, lowest-priority pack first).

use super::read::{FLOOR, ReadFile, SceneOutcome, read};
use super::texts::{DUPLICATE, formatted};
use super::{DialogueFile, DialogueScene};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity,
};
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, NoBeta, Origin, Placed, Registry,
    RegistryBuilder, Rejected, VersionInfo,
};
use bedrock_addon_core::id::{DialogueSceneTag, kind};
use bedrock_addon_core::json::{EncodeError, WriteStyle};
use bedrock_addon_core::load::{
    Accept, AppDataPlace, CollectionLoader, ListDepth, LoadContext, StackCx, Walk, WalkFile,
    WalkOrder,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::SemVersion;
use indexmap::IndexMap;

/// The directory the game walks.
const DIRECTORY: &str = "dialogue";

impl bedrock_addon_core::__private::Sealed for DialogueFile {}
impl bedrock_addon_core::__private::Sealed for DialogueScene {}

impl Content for DialogueFile {
    const NAME: &'static str = "dialogue";
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        let origin = origin(PackRef::app_data(), PackSide::Behavior, path, None);
        let file = match read(&stem(path), bytes, cx) {
            Ok(file) => file,
            Err(mut diagnostics) => {
                locate(&mut diagnostics, &origin);
                return Err(Rejected::new(origin, diagnostics));
            }
        };
        let mut diagnostics = file.diagnostics;
        let mut scenes: Vec<DialogueScene> = Vec::new();
        for scene in file.scenes {
            match scene.outcome {
                SceneOutcome::Loaded(loaded, found) => {
                    diagnostics.extend(found);
                    // The first scene of a tag is the one stored; a later one is dropped like
                    // any other scene a rule drops. The game logs it at the content log's error
                    // level and goes on with the next scene, so the finding is a kept error.
                    if scenes.iter().any(|kept| kept.scene_tag == loaded.scene_tag) {
                        diagnostics.push_kept_error(duplicate(
                            &origin,
                            loaded.scene_tag.as_str(),
                            &scene.at,
                            Severity::Error,
                        ));
                    } else {
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
        locate(&mut diagnostics, &origin);
        let version = version_info(&file.format_version);
        let value = DialogueFile {
            format_version: file.format_version,
            scenes,
            dialogue_extra: file.dialogue_extra,
            extra: file.extra,
        };
        Ok(Document::new(value, origin, version, diagnostics))
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        self.write_into(out, style)
    }

    fn to_json(&self) -> Result<String, EncodeError> {
        let mut out = Vec::new();
        self.write_into(&mut out, WriteStyle::default())?;
        Ok(String::from_utf8(out).unwrap_or_default())
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

/// Vanilla's "already exists" text for the scene `tag` at `at` of `file`.
fn duplicate(file: &Origin, tag: &str, at: &JsonPath, severity: Severity) -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::DuplicateIdentifier {
            text: formatted(DUPLICATE, &[&stem(&file.path), tag]),
        },
        severity,
        Phase::Validate,
        Location::new(
            Some(file.pack.clone()),
            Some(file.path.clone()),
            at.clone(),
            None,
        ),
    )
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

/// Gives every finding the document's pack and file.
fn locate(diagnostics: &mut Diagnostics, origin: &Origin) {
    let found = std::mem::take(diagnostics);
    for mut finding in found {
        finding.location.pack = Some(origin.pack.clone());
        finding.location.path = Some(origin.path.clone());
        diagnostics.push(finding);
    }
}

// ---- the collection ----------------------------------------------------------------------

/// A dialogue file: extension exactly `json`, compared case-sensitively.
fn is_json(path: &PackPath) -> bool {
    path.segments()
        .last()
        .and_then(|name| name.rsplit_once('.'))
        .is_some_and(|(_, extension)| extension == "json")
}

/// `dialogue/**/*.json`, packs bottom-up (the game walks the stack from its last element, the
/// lowest priority, to the first).
const WALK: Walk = Walk {
    directories: &[DIRECTORY],
    depth: ListDepth::Recursive,
    accept: Accept::Fn(is_json),
    order: WalkOrder::BottomUp,
    directory_major: false,
    app_data: &[],
    app_data_place: AppDataPlace::After,
    in_pack: None,    // every matching file of every pack is read
    shadowed: None,   // a file an active subpack shadows is not read
    read_error: None, // a file that cannot be read gets the stack driver's generic text
};

/// One file's part of the registry.
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
            Err(mut diagnostics) => {
                locate(&mut diagnostics, &file_origin);
                builder.push(Contribution::Rejected(
                    None,
                    Rejected::new(file_origin, diagnostics),
                ));
                continue;
            }
        };
        let mut file_diagnostics = file.diagnostics;
        locate(&mut file_diagnostics, &file_origin);
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
            locate(&mut diagnostics, &origin);
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
                    diagnostics.push(duplicate(file, tag.as_str(), at, Severity::Error));
                    builder.push(Contribution::Rejected(
                        Some(tag),
                        Rejected::new(origin, diagnostics),
                    ));
                }
                // A higher-priority pack's scene: the game's error, and the registry lists it
                // as shadowed (the crate's own `ShadowedDefinition`, raised by the combine
                // rule).
                Some(_) => {
                    let error = duplicate(file, tag.as_str(), at, Severity::Error);
                    let document = Document::new(scene, origin, version.clone(), diagnostics);
                    builder.push(Contribution::Stable(tag, document));
                    builder.push_diagnostic(error);
                }
            }
        }
        SceneOutcome::Dropped(tag, mut diagnostics) => {
            let origin = origin(file.pack.clone(), file.side, &file.path, tag.as_deref());
            locate(&mut diagnostics, &origin);
            builder.push(Contribution::Rejected(
                tag.map(DialogueSceneTag::parse),
                Rejected::new(origin, diagnostics),
            ));
        }
        SceneOutcome::Untagged(mut diagnostics) => {
            locate(&mut diagnostics, file);
            for finding in diagnostics {
                builder.push_diagnostic(finding);
            }
        }
    }
}

/// The loader of `bp.dialogue()`: `dialogue/**/*.json` of every behaviour pack, the
/// lowest-priority pack's scene winning.
pub const fn load_registry() -> CollectionLoader<Registry<DialogueScene>, Parsed> {
    CollectionLoader {
        walk: WALK,
        parse: parse_walk_file,
        combine,
        needs: &[], // combining reads no other collection
        then: None, // every file is named by the walk itself; there is no second stage
    }
}
