//! Loading: the file entry point (`_buildInheritanceTree` for one file), `Content` /
//! `Definition`, the walk of `models/`, and the combine rule (per identifier the last definition
//! of the walk replaces the earlier one whole; a legacy parent is looked up across the stack).

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::history::v1_8_0::MAX_FORMAT_VERSION as KEYED_MAXIMUM;
use super::history::{Any, VERSIONS};
use super::read::V1_19_30;
use super::schema::{
    DID_NOT_VALIDATE, EXCEEDS_GAME_VERSION, INTERNAL_BLOCKS_FLOOR, JsonKind, NOT_AN_OBJECT, Node,
    PARENT_NOT_DEFINED, PARSE_ERROR, SchemaWalk, VERSION_NOT_VALID, text,
};
use super::validate::{Place, first_bad_identifier, missing_bone_parents, parse_time};
use super::{Geometry, GeometryFile, Inherited};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, JsonPathSegment, Phase, Severity,
    VanillaText,
};
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, NoBeta, Notice, Origin, Placed,
    Registry, RegistryBuilder, Rejected, VersionInfo,
};
use bedrock_addon_core::id::{GeometryId, kind};
use bedrock_addon_core::json::{
    EncodeError, JsonSlice, MAX_JSON_DEPTH, Value, WriteStyle, check_depth,
};
use bedrock_addon_core::json::{parse_at, strip_bom};
use bedrock_addon_core::load::{
    Accept, AppDataPlace, CollectionLoader, ListDepth, LoadContext, StackCx, Walk, WalkFile,
    WalkOrder,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput, SemVersion};
use bedrock_addon_core::versioning::{ParseCx, UpgradeCx};

impl bedrock_addon_core::__private::Sealed for GeometryFile {}
impl bedrock_addon_core::__private::Sealed for Geometry {}

/// The version a file without a readable `format_version` is read as (vanilla's `"1.0"`
/// default).
const DEFAULT_VERSION: FormatVersion = FormatVersion::new(1, 0, 0);
/// The directory of block geometry.
const BLOCKS: &str = "models/blocks/";
/// The directory of entity geometry, where the writer puts a file.
const ENTITY: &str = "models/entity";
/// The file of the old built-in mob geometry.
const MOBS: &str = "models/mobs.json";

fn member(name: &str) -> JsonPath {
    let mut at = JsonPath::new();
    at.push(name);
    at
}

fn did_not_validate(diagnostics: &mut Diagnostics) {
    diagnostics.push(Diagnostic::error(DiagnosticCode::InvalidValue {
        text: text(DID_NOT_VALIDATE, &[]),
    }));
}

/// What the envelope of a file gave: the struct of its version, not upgraded.
struct Envelope {
    any: Any,
    /// `format_version` as the file has it; `None` when absent or unreadable.
    written: Option<FormatVersion>,
    /// The version the file is read as.
    effective: FormatVersion,
    /// The version the file states, when the keyed door reads it by an earlier one.
    stated: FormatVersion,
}

/// What reading a file gave.
enum Outcome<T> {
    /// The file loads.
    Loaded(T),
    /// The file is empty or `null`: skipped, which is no failure.
    Empty,
    /// The file does not load.
    Failed,
}

/// The file's text after the BOM, the depth check and the JSON syntax check.
fn text_of<'a>(bytes: &'a [u8], diagnostics: &mut Diagnostics) -> Outcome<Cow<'a, str>> {
    let mut malformed = |message: String| {
        diagnostics.push(Diagnostic::error(DiagnosticCode::MalformedInput {
            text: text(PARSE_ERROR, &[&message]),
            message,
        }));
        Outcome::Failed
    };
    let bytes = strip_bom(bytes);
    if bytes.iter().all(u8::is_ascii_whitespace) {
        return Outcome::Empty;
    }
    if check_depth(bytes, MAX_JSON_DEPTH).is_err() {
        return malformed("nesting too deep".to_owned());
    }
    let text = match std::str::from_utf8(bytes) {
        Ok(text) => text,
        Err(error) => return malformed(error.to_string()),
    };
    // The game parses the whole text with its JSON reader first: a syntax error anywhere drops
    // the file.
    if let Err(error) = parse_at::<Value>(JsonSlice::new(text)) {
        return malformed(error.error.to_string());
    }
    // Of a member written twice the last one stands, whole and at its place:
    // the walk and the version peek read the text without the earlier copies.
    Outcome::Loaded(bedrock_addon_core::json::last_wins(text))
}

/// The member's version: 1.0.0 with an information when it is absent (vanilla is silent: it
/// writes `format_version "1.1"` into the file and validates it against the schema older
/// than 1.8) or a warning when unreadable.
fn read_version(
    root: Node<'_>,
    diagnostics: &mut Diagnostics,
) -> (Option<FormatVersion>, FormatVersion, String) {
    let walk = SchemaWalk::new(true, &JsonPath::new());
    let node = walk
        .members(root)
        .unwrap_or_default()
        .into_iter()
        .find_map(|(key, node)| (key == "format_version").then_some(node));
    let Some(node) = node else {
        // The crate's own finding: the game has no text for a missing version here.
        diagnostics.push(Diagnostic::info(DiagnosticCode::MissingFormatVersion {
            text: None,
        }));
        return (None, DEFAULT_VERSION, "1.0".to_owned());
    };
    let as_written = if node.kind() == JsonKind::String {
        parse_at::<String>(JsonSlice::new(node.text())).unwrap_or_default()
    } else {
        node.text().to_owned()
    };
    let version = (node.kind() == JsonKind::String)
        .then(|| SemVersion::parse(&as_written))
        .flatten()
        // `"beta"` and a wildcard are no version the game's version parser reads.
        .filter(|v| as_written != "beta" && !v.is_any())
        .map(|v| v.format_version());
    if let Some(version) = version {
        return (Some(version), version, as_written);
    }
    // The crate's own information: the game has no text for this.
    diagnostics.push(
        Diagnostic::info(DiagnosticCode::MalformedFormatVersion {
            text: text(VERSION_NOT_VALID, &[&as_written]),
        })
        .at(member("format_version")),
    );
    (None, DEFAULT_VERSION, as_written)
}

/// The envelope and the schema of a file: its version, the gates on it, and the parse by the
/// struct of its era.
fn read_written(
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
    keyed: bool,
    diagnostics: &mut Diagnostics,
) -> Outcome<Envelope> {
    // A leading byte-order mark is blanked, not cut, so every span is an offset into the file
    // as written.
    let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
    let source = match text_of(bytes, diagnostics) {
        Outcome::Loaded(source) => source,
        Outcome::Empty => return Outcome::Empty,
        Outcome::Failed => return Outcome::Failed,
    };
    let slice = JsonSlice::new(&source);
    let Some(root) = Node::of(slice) else {
        return Outcome::Failed;
    };
    match root.kind() {
        JsonKind::Null => return Outcome::Empty,
        JsonKind::Object => {}
        _ => {
            diagnostics.push(Diagnostic::error(DiagnosticCode::MalformedInput {
                message: "the root is not an object".to_owned(),
                text: text_fixed(NOT_AN_OBJECT),
            }));
            return Outcome::Failed;
        }
    }
    let (written, effective, as_written) = read_version(root, diagnostics);
    if effective > cx.engine_version {
        diagnostics.push(
            Diagnostic::error(DiagnosticCode::UnsupportedVersion {
                text: text(
                    EXCEEDS_GAME_VERSION,
                    &[&as_written, &cx.engine_version.to_string()],
                ),
            })
            .at(member("format_version")),
        );
        return Outcome::Failed;
    }
    if place_of(path, cx).requires_minecraft_namespace() && effective < V1_19_30 {
        diagnostics.push(
            Diagnostic::error(DiagnosticCode::UnsupportedVersion {
                text: text(INTERNAL_BLOCKS_FLOOR, &[]),
            })
            .at(member("format_version")),
        );
        return Outcome::Failed;
    }
    // The keyed door: a file that states a version the keyed layout ended before is read by
    // the last keyed struct's rules; the gates above held for the version it states.
    let stated = effective;
    let effective = if keyed {
        effective.min(KEYED_MAXIMUM)
    } else {
        effective
    };
    let parse = ParseCx {
        load: cx,
        written: FormatVersionInput {
            version: Some(effective),
            beta_literal: false,
            prerelease: false,
        },
        original_version: effective,
        at: JsonPath::new(),
    };
    let Some(any) = Any::parse(slice, &parse, diagnostics) else {
        did_not_validate(diagnostics);
        return Outcome::Failed;
    };
    Outcome::Loaded(Envelope {
        any,
        written,
        effective,
        stated,
    })
}

/// A vanilla text without arguments.
#[allow(
    clippy::unnecessary_wraps,
    reason = "every code field is an `Option<VanillaText>`"
)]
fn text_fixed(template: &'static str) -> Option<VanillaText> {
    Some(VanillaText::Fixed(template))
}

fn place_of(path: &PackPath, cx: &LoadContext) -> Place {
    Place {
        base_game: cx.is_base_game_pack,
        blocks: path.as_str().starts_with(BLOCKS),
    }
}

/// A file after its upgrade and its checks.
struct Finished {
    /// The file with the geometries that are registered.
    file: GeometryFile,
    /// The identifiers of the geometries the identifier loop stopped at and after.
    tail: Vec<String>,
}

/// The steps from the file's struct to the newest one, then the identifier loop and the
/// parse-time checks. `None` when a step or the `binding` compile fails the file.
fn finish(
    any: Any,
    effective: FormatVersion,
    place: Place,
    cx: &LoadContext,
    diagnostics: &mut Diagnostics,
) -> Option<Finished> {
    let before = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count();
    let mut file = any.upgrade_to_end(&mut UpgradeCx {
        load: cx,
        original_version: effective,
        identifier: "",
        actor_type: ActorType::Mob,
        skip_section_a: false,
        diagnostics,
    });
    let errors = |d: &Diagnostics| d.iter().filter(|d| d.severity == Severity::Error).count();
    if errors(diagnostics) != before {
        return None;
    }
    let written = FormatVersionInput {
        version: Some(effective),
        beta_literal: false,
        prerelease: false,
    };
    // The schema compiles `binding` while it validates: before the identifier loop.
    if !super::molang::run(&mut file, written, cx, diagnostics) {
        did_not_validate(diagnostics);
        return None;
    }
    let tail = match first_bad_identifier(&file, effective, place, diagnostics) {
        Some(index) => file
            .geometries
            .split_off(index)
            .into_iter()
            .map(|g| g.description.identifier)
            .collect(),
        None => Vec::new(),
    };
    parse_time(&mut file, effective, diagnostics);
    Some(Finished { file, tail })
}

/// One file read to the end.
struct Loaded {
    finished: Option<Finished>,
    version: VersionInfo,
    diagnostics: Diagnostics,
    /// The file was empty or `null`.
    empty: bool,
}

fn version_info(written: Option<FormatVersion>, effective: FormatVersion) -> VersionInfo {
    let parsed_as = VERSIONS[Any::index_of(effective)];
    VersionInfo::new(
        written,
        false,
        effective,
        parsed_as,
        parsed_as != VERSIONS[VERSIONS.len() - 1],
    )
}

fn load(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Loaded {
    load_as(path, bytes, cx, false)
}

/// [`load`], through the file door or the keyed one.
fn load_as(path: &PackPath, bytes: &[u8], cx: &LoadContext, keyed: bool) -> Loaded {
    let mut diagnostics = Diagnostics::default();
    let unread = version_info(None, DEFAULT_VERSION);
    match read_written(path, bytes, cx, keyed, &mut diagnostics) {
        Outcome::Empty => {
            // Vanilla skips the file without a message.
            diagnostics.push(Diagnostic::info(DiagnosticCode::EmptyDocument {
                text: None,
            }));
            Loaded {
                finished: None,
                version: unread,
                diagnostics,
                empty: true,
            }
        }
        Outcome::Failed => Loaded {
            finished: None,
            version: unread,
            diagnostics,
            empty: false,
        },
        Outcome::Loaded(written) => {
            let mut finished = finish(
                written.any,
                written.effective,
                place_of(path, cx),
                cx,
                &mut diagnostics,
            );
            let mut version = version_info(written.written, written.effective);
            if written.stated != written.effective {
                // The keyed door: the document is of the version its file states.
                if let Some(finished) = &mut finished {
                    finished.file.format_version = written.stated;
                }
                version = VersionInfo::new(
                    written.written,
                    false,
                    written.stated,
                    version.parsed_as,
                    true,
                );
            }
            Loaded {
                finished,
                version,
                diagnostics,
                empty: false,
            }
        }
    }
}

fn detached_origin(path: &PackPath) -> Origin {
    Origin::file(PackRef::app_data(), PackSide::Resource, path.clone())
}

/// Loads one geometry file as vanilla's `GeometryGroup` does: the version gates, the schema of
/// the file's own era (lenient below 1.19.30, strict from it), the upgrade to the newest shape,
/// the identifier rules and the parse-time checks.
///
/// `path` is the file's path in its pack: a file below `models/blocks/` of a base-game pack
/// must be of 1.19.30 or later and name its geometries `minecraft:geometry.<name>`. The result
/// is the whole file or nothing: an empty file, and a file in which one geometry breaks an
/// identifier rule, are rejected here (a pack stack keeps the geometries before the offending
/// one, as vanilla does).
///
/// # Errors
///
/// Returns [`Rejected`] when the file is empty, is not valid JSON, or breaks an identifier rule or
/// a parse-time check; it carries every finding about the file.
pub fn load_file(
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> Result<Document<GeometryFile>, Rejected> {
    let origin = detached_origin(path);
    let loaded = load(path, bytes, cx);
    let diagnostics = loaded.diagnostics.located(&origin);
    match loaded.finished {
        Some(finished) if !diagnostics.has_errors() => Ok(Document::new(
            finished.file,
            origin,
            loaded.version,
            diagnostics,
        )),
        _ => Err(Rejected::new(origin, diagnostics)),
    }
}

/// Loads a file whose models are keyed at its root (`{ "format_version": …,
/// "geometry.<name>": { … } }`) by the keyed layout's rules, whatever version it states: the
/// models of a persona piece, which say 1.12.0 and are never read through the game's file
/// schema (the client takes the model out of the file by its name). A file below 1.12.0 reads
/// as [`load_file`] reads it; from 1.12.0 on it is read by the last keyed struct (1.8.0) and
/// keeps the version it states (`GeometryFile::format_version`, `Document::version`). The
/// engine's ceiling and every other rule of [`load_file`] hold, the warning for a bone whose
/// parent is no bone of its model among them: a caller that merges the model into another's
/// bones (a piece into the character's) reads that finding as a note. A `minecraft:geometry`
/// member is no model of this layout.
///
/// # Errors
///
/// The findings of the file, when it does not load.
pub fn load_keyed_file(
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> Result<Document<GeometryFile>, Rejected> {
    let origin = detached_origin(path);
    let loaded = load_as(path, bytes, cx, true);
    let diagnostics = loaded.diagnostics.located(&origin);
    match loaded.finished {
        Some(finished) if !diagnostics.has_errors() => Ok(Document::new(
            finished.file,
            origin,
            loaded.version,
            diagnostics,
        )),
        _ => Err(Rejected::new(origin, diagnostics)),
    }
}

impl Content for GeometryFile {
    const NAME: &'static str = "geometry file";
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        load_file(path, bytes, cx)
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        bedrock_addon_core::json::write_json(&self.to_value()?, out, style)
    }
}

impl bedrock_addon_core::document::JsonContent for GeometryFile {}

impl Definition for Geometry {
    type Kind = kind::Geometry;
    type File = GeometryFile;
    type Beta = NoBeta<kind::Geometry>;
}

impl Placed for GeometryFile {
    /// `models/entity/geometry.<name>.json`, after the file's first geometry;
    /// `models/entity/geometry.json` for a file without one. Block geometry belongs in
    /// `models/blocks/`, which the content does not say: the writer is given that path.
    fn default_path(&self) -> PackPath {
        match self.geometries.first() {
            Some(first) => bedrock_addon_core::document::default_path(
                ENTITY,
                &GeometryId::parse(first.description.identifier.as_str()),
            ),
            None => PackPath::from_static("models/entity/geometry.json"),
        }
    }
}

impl GeometryFile {
    /// Parses one geometry file without upgrading it: the struct its `format_version` selects
    ///; [`history::upgrade`](super::history::upgrade) finishes it.
    ///
    /// # Errors
    ///
    /// Returns [`Rejected`] when the file cannot be read into a value at all, such as bytes that
    /// are not JSON or a document the family refuses; it carries every finding about the file.
    pub fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> Result<Document<Any>, Rejected> {
        let origin = detached_origin(path);
        let mut diagnostics = Diagnostics::default();
        let outcome = read_written(path, bytes, cx, false, &mut diagnostics);
        if matches!(outcome, Outcome::Empty) {
            diagnostics.push(Diagnostic::info(DiagnosticCode::EmptyDocument {
                text: None,
            }));
        }
        let diagnostics = diagnostics.located(&origin);
        match outcome {
            Outcome::Loaded(written) => {
                let parsed_as = written.any.parsed_version();
                let version =
                    VersionInfo::new(written.written, false, written.effective, parsed_as, false);
                Ok(Document::new(written.any, origin, version, diagnostics))
            }
            _ => Err(Rejected::new(origin, diagnostics)),
        }
    }
}

/// `history::upgrade`: the steps and the checks on a document `parse_as_written` gave.
pub(crate) fn upgrade_document(
    document: Document<Any>,
    cx: &LoadContext,
) -> Result<Document<GeometryFile>, Rejected> {
    let origin = document.origin().clone();
    let effective = document.version().effective;
    let version = version_info(document.version().written, effective);
    let mut diagnostics = Diagnostics::default();
    let finished = finish(
        document.into_value(),
        effective,
        place_of(&origin.path, cx),
        cx,
        &mut diagnostics,
    );
    let diagnostics = diagnostics.located(&origin);
    match finished {
        Some(finished) if !diagnostics.has_errors() => {
            Ok(Document::new(finished.file, origin, version, diagnostics))
        }
        _ => Err(Rejected::new(origin, diagnostics)),
    }
}

// ---- the collection ----------------------------------------------------------------------------

/// What one walked file gives the registry.
#[derive(Debug)]
pub struct FileContribution {
    contributions: Vec<Contribution<Geometry>>,
    /// The file's findings that belong to no one geometry.
    findings: Vec<Diagnostic>,
    pack: PackRef,
    /// 0 `models/mobs.json`, 1 `models/entity/`, 2 `models/blocks/`: vanilla's order in a pack.
    rank: u8,
    path: PackPath,
}

/// The index of the geometry a finding belongs to: one at `minecraft:geometry/<i>/…`, or one a
/// step or the legacy schema raised below a `geometry.<name>` key.
fn owner(finding: &Diagnostic, keys: &HashMap<String, usize>) -> Option<usize> {
    let mut segments = finding.location.json_path.segments();
    match (segments.next()?, segments.next()) {
        (JsonPathSegment::Key("minecraft:geometry"), Some(JsonPathSegment::Index(index))) => {
            Some(index)
        }
        (JsonPathSegment::Key(key), _) => keys.get(key).copied(),
        _ => None,
    }
}

fn parse_walked(file: &WalkFile, cx: &LoadContext) -> FileContribution {
    let origin = Origin::file(file.pack.clone(), file.side, file.path.clone());
    let key = file.key.as_str();
    let rank = if key == MOBS {
        0
    } else if key.starts_with(BLOCKS) {
        2
    } else {
        1
    };
    let mut out = FileContribution {
        contributions: Vec::new(),
        findings: Vec::new(),
        pack: file.pack.clone(),
        rank,
        path: file.key.clone(),
    };
    // The gates read the directory from the path without a subpack folder.
    let loaded = load(&file.key, &file.bytes, cx);
    let diagnostics = loaded.diagnostics.located(&origin);
    let Some(finished) = loaded.finished else {
        if loaded.empty {
            out.findings.extend(diagnostics);
        } else {
            out.contributions.push(Contribution::Rejected(
                None,
                Rejected::new(origin, diagnostics),
            ));
        }
        return out;
    };
    // A legacy key is `identifier[:parent]`.
    let keys: HashMap<String, usize> = finished
        .file
        .geometries
        .iter()
        .enumerate()
        .map(|(index, g)| {
            let key = match &g.legacy.parent {
                Some(parent) => format!("{}:{parent}", g.description.identifier),
                None => g.description.identifier.clone(),
            };
            (key, index)
        })
        .collect();
    let kept = finished.file.geometries.len();
    let mut per_geometry: Vec<Diagnostics> = (0..kept).map(|_| Diagnostics::default()).collect();
    let mut tail = Diagnostics::default();
    for finding in diagnostics {
        match owner(&finding, &keys) {
            Some(index) if index < kept => per_geometry[index].push(finding),
            Some(_) => tail.push(finding),
            None => out.findings.push(finding),
        }
    }
    for (geometry, diagnostics) in finished.file.geometries.into_iter().zip(per_geometry) {
        let mut origin = origin.clone();
        origin.key = Some(geometry.description.identifier.clone().into_boxed_str());
        let id = GeometryId::parse(geometry.description.identifier.as_str());
        out.contributions.push(Contribution::Stable(
            id,
            Document::new(geometry, origin, loaded.version.clone(), diagnostics),
        ));
    }
    // The identifier loop stopped: the geometry that broke its rule carries the findings, the
    // ones after it are dropped with it.
    for (index, identifier) in finished.tail.into_iter().enumerate() {
        let mut origin = origin.clone();
        origin.key = Some(identifier.into_boxed_str());
        let diagnostics = if index == 0 {
            std::mem::take(&mut tail)
        } else {
            Diagnostics::default()
        };
        out.contributions.push(Contribution::Rejected(
            None,
            Rejected::new(origin, diagnostics),
        ));
    }
    out
}

/// A `.json` file of the geometry loader: `models/mobs.json`, or a file below `models/entity/`
/// or `models/blocks/` whose extension is exactly `.json`.
fn is_geometry_file(path: &PackPath) -> bool {
    let path = path.as_str();
    if path == MOBS {
        return true;
    }
    (path.starts_with("models/entity/") || path.starts_with(BLOCKS))
        && path
            .rsplit_once('.')
            .is_some_and(|(_, extension)| extension == "json")
}

/// The loader's text for a file whose bytes could not be read; `{}` is the file's path in its
/// pack. The game has a second text for a source pack that is no longer live; it has no case
/// here, because a pack stays live during a load.
const UNREAD: &str = "Cannot load geometry file {}: pack has no access strategy";

/// `models/**`, packs bottom-up. Vanilla's directory strategy reads sub-folders for every pack
/// but those shipped inside the app.
const GEOMETRY_WALK: Walk = Walk {
    directories: &["models"],
    depth: ListDepth::Recursive,
    accept: Accept::Fn(is_geometry_file),
    order: WalkOrder::BottomUp,
    directory_major: false,
    app_data: &[],
    app_data_place: AppDataPlace::After,
    in_pack: None,
    // No subpack-shadowed copies are read.
    shadowed: None,
    read_error: Some(UNREAD),
};

/// What a geometry gives the geometries that inherit from it.
#[derive(Clone)]
enum Parent {
    /// Its own bones and its own inheritance.
    Built(Arc<Inherited>),
    /// It is not built: a parent up its chain is missing, or the chain returns to itself.
    NotBuilt,
}

/// Where the effective definition of an identifier is: file and contribution index.
type Slot = (usize, usize);

fn stable(files: &[FileContribution], slot: Slot) -> Option<&Document<Geometry>> {
    match &files[slot.0].contributions[slot.1] {
        Contribution::Stable(_, document) => Some(document),
        _ => None,
    }
}

/// How the walk up from a geometry ended.
enum End {
    /// At a geometry settled by an earlier walk (not on this walk).
    Settled(Parent),
    /// At a geometry without a parent (the top of the walk).
    Root,
    /// At a geometry whose parent no pack defines (the top of the walk).
    Missing,
    /// At a geometry already on the walk.
    Cycle,
}

/// Resolves every legacy parent against the definitions that win, without recursion: a child
/// gets its parent's bones; a child whose parent no pack defines, or whose chain of parents
/// returns to itself, is not built.
fn inherit(files: &mut [FileContribution]) {
    let mut winners: HashMap<GeometryId, Slot> = HashMap::new();
    for (f, file) in files.iter().enumerate() {
        for (c, contribution) in file.contributions.iter().enumerate() {
            if let Contribution::Stable(id, _) = contribution {
                winners.insert(id.clone(), (f, c));
            }
        }
    }
    let parent_of = |slot: Slot| -> Option<GeometryId> {
        stable(files, slot)?.value().parent().map(GeometryId::parse)
    };
    let mut children: Vec<Slot> = winners
        .values()
        .copied()
        .filter(|slot| parent_of(*slot).is_some())
        .collect();
    children.sort_unstable();
    let mut settled: HashMap<Slot, Parent> = HashMap::new();
    let mut inherited: Vec<Heir> = Vec::new();
    let mut failed: Vec<(Slot, DiagnosticCode)> = Vec::new();
    for start in children {
        let mut walk: Vec<Slot> = Vec::new();
        let mut on_walk: HashSet<Slot> = HashSet::new();
        let mut at = start;
        let end = loop {
            if let Some(parent) = settled.get(&at) {
                break End::Settled(parent.clone());
            }
            if !on_walk.insert(at) {
                break End::Cycle;
            }
            walk.push(at);
            let Some(parent) = parent_of(at) else {
                break End::Root;
            };
            match winners.get(&parent) {
                Some(next) => at = *next,
                None => break End::Missing,
            }
        };
        // What the geometry on top of the walk inherits; `Err` when it is not built.
        let cycle = matches!(end, End::Cycle);
        // `at` is the settled geometry the walk stopped at.
        let mut above: Result<Option<(Slot, Arc<Inherited>)>, ()> = match end {
            End::Settled(Parent::Built(node)) => Ok(Some((at, node))),
            End::Root => Ok(None),
            End::Settled(Parent::NotBuilt) | End::Missing | End::Cycle => Err(()),
        };
        for slot in walk.into_iter().rev() {
            let Some(geometry) = stable(files, slot).map(Document::value) else {
                continue;
            };
            if let Ok(parent) = above {
                if let Some((parent, node)) = &parent {
                    inherited.push(Heir {
                        slot,
                        parent: *parent,
                        node: node.clone(),
                    });
                }
                let node = Arc::new(Inherited::new(
                    geometry.description.identifier.clone(),
                    geometry.bones().to_vec(),
                    parent.map(|(_, node)| node),
                ));
                settled.insert(slot, Parent::Built(node.clone()));
                above = Ok(Some((slot, node)));
            } else {
                let code = if cycle {
                    DiagnosticCode::ReferenceCycle { text: None }
                } else {
                    DiagnosticCode::UnresolvedReference {
                        text: text(
                            PARENT_NOT_DEFINED,
                            &[
                                &geometry.description.identifier,
                                geometry.parent().unwrap_or_default(),
                            ],
                        ),
                    }
                };
                failed.push((slot, code));
                settled.insert(slot, Parent::NotBuilt);
            }
        }
    }
    let missing = unknown_bone_parents(files, &inherited);
    apply(files, inherited, missing, failed);
}

/// A geometry that inherits: where it is, where its parent is, and what it inherits.
struct Heir {
    slot: Slot,
    parent: Slot,
    node: Arc<Inherited>,
}

/// The missing-parent check for every geometry that inherits: per geometry, the indices of its bones whose parent
/// is neither one of its own bones nor a bone of a geometry up its chain.
///
/// One walk of the inheritance forest from its roots, with the bone names of the geometries
/// between the root and the geometry in hand counted in one map: each bone of each geometry is
/// looked at once, whatever the length of a chain. No recursion.
fn unknown_bone_parents(files: &[FileContribution], heirs: &[Heir]) -> HashMap<Slot, Vec<usize>> {
    let mut children: HashMap<Slot, Vec<Slot>> = HashMap::new();
    let mut inherits: HashSet<Slot> = HashSet::new();
    for heir in heirs {
        children.entry(heir.parent).or_default().push(heir.slot);
        inherits.insert(heir.slot);
    }
    let mut roots: Vec<Slot> = children
        .keys()
        .copied()
        .filter(|slot| !inherits.contains(slot))
        .collect();
    roots.sort_unstable();
    let bones = |slot: Slot| stable(files, slot).map_or(&[][..], |d| d.value().bones());
    let mut missing: HashMap<Slot, Vec<usize>> = HashMap::new();
    // How many geometries above the one in hand have a bone of each name.
    let mut above: HashMap<&str, usize> = HashMap::new();
    // `true`: the geometry's subtree is done and its names leave the map.
    let mut pending: Vec<(Slot, bool)> = roots.into_iter().map(|slot| (slot, false)).collect();
    while let Some((slot, done)) = pending.pop() {
        let own = bones(slot);
        if done {
            for bone in own {
                if let Some(count) = above.get_mut(bone.name.as_str()) {
                    *count = count.saturating_sub(1);
                }
            }
            continue;
        }
        if inherits.contains(&slot) {
            let names: HashSet<&str> = own.iter().map(|bone| bone.name.as_str()).collect();
            let unknown: Vec<usize> = own
                .iter()
                .enumerate()
                .filter(|(_, bone)| {
                    bone.parent.as_deref().is_some_and(|parent| {
                        !names.contains(parent) && above.get(parent).is_none_or(|n| *n == 0)
                    })
                })
                .map(|(index, _)| index)
                .collect();
            if !unknown.is_empty() {
                missing.insert(slot, unknown);
            }
        }
        for bone in own {
            *above.entry(bone.name.as_str()).or_default() += 1;
        }
        pending.push((slot, true));
        if let Some(below) = children.get(&slot) {
            pending.extend(below.iter().map(|child| (*child, false)));
        }
    }
    missing
}

/// Gives each child the chain it inherits and its bone-parent findings, and turns each
/// geometry that is not built into a rejected contribution with its finding.
fn apply(
    files: &mut [FileContribution],
    inherited: Vec<Heir>,
    mut missing: HashMap<Slot, Vec<usize>>,
    failed: Vec<(Slot, DiagnosticCode)>,
) {
    for Heir { slot, node, .. } in inherited {
        let unknown = missing.remove(&slot).unwrap_or_default();
        replace(files, slot, |id, document| {
            let origin = document.origin().clone();
            let version = document.version().clone();
            let mut diagnostics = document.diagnostics().clone();
            let mut geometry = document.into_value();
            geometry.inherited = Some(node);
            let mut found = Diagnostics::default();
            // A file's registered geometries are its first contributions, in file order.
            missing_bone_parents(&geometry, slot.1, &unknown, &mut found);
            diagnostics.extend(found.located(&origin));
            Contribution::Stable(id, Document::new(geometry, origin, version, diagnostics))
        });
    }
    let mut not_built: HashSet<GeometryId> = HashSet::new();
    for (slot, code) in failed {
        replace(files, slot, |id, document| {
            not_built.insert(id.clone());
            let origin = document.origin().clone();
            let mut diagnostics = document.diagnostics().clone();
            diagnostics.push(
                Diagnostic::error(code)
                    .phase(Phase::Validate)
                    .in_file(&origin),
            );
            Contribution::Rejected(Some(id), Rejected::new(origin, diagnostics))
        });
    }
    shadow_for_good(files, &not_built);
}

/// The last loaded definition of an identifier is its only candidate. The earlier
/// definitions of an identifier whose last one is not built leave the walk here, each with the
/// note a replaced definition gets, so that the registry does not make one of them effective
/// in the rejected one's place.
fn shadow_for_good(files: &mut [FileContribution], not_built: &HashSet<GeometryId>) {
    if not_built.is_empty() {
        return;
    }
    for file in files {
        let mut kept = Vec::with_capacity(file.contributions.len());
        for contribution in std::mem::take(&mut file.contributions) {
            match contribution {
                Contribution::Stable(id, document) if not_built.contains(&id) => {
                    let origin = document.origin();
                    file.findings.push(
                        Diagnostic::info(Notice::shadowed().code)
                            .phase(Phase::Validate)
                            .in_file(origin),
                    );
                }
                other => kept.push(other),
            }
        }
        file.contributions = kept;
    }
}

/// Replaces the stable contribution at `slot` by what `with` makes of it.
fn replace(
    files: &mut [FileContribution],
    slot: Slot,
    with: impl FnOnce(GeometryId, Document<Geometry>) -> Contribution<Geometry>,
) {
    let contributions = &mut files[slot.0].contributions;
    let placeholder = Contribution::Rejected(
        None,
        Rejected::new(
            detached_origin(&PackPath::from_static(MOBS)),
            Diagnostics::default(),
        ),
    );
    match std::mem::replace(&mut contributions[slot.1], placeholder) {
        Contribution::Stable(id, document) => contributions[slot.1] = with(id, document),
        other => contributions[slot.1] = other,
    }
}

fn combine(mut parsed: Vec<FileContribution>, stack: &StackCx<'_>) -> Registry<Geometry> {
    // Inside a pack vanilla reads `models/mobs.json`, then `models/entity/`, then
    // `models/blocks/`; the crate reads each directory in path order (vanilla uses enumeration
    // order).
    let mut group = 0_usize;
    let mut previous: Option<PackRef> = None;
    let mut keyed: Vec<(usize, FileContribution)> = Vec::with_capacity(parsed.len());
    for file in parsed.drain(..) {
        if previous.as_ref() != Some(&file.pack) {
            group += 1;
            previous = Some(file.pack.clone());
        }
        keyed.push((group, file));
    }
    keyed.sort_by(|a, b| {
        (a.0, a.1.rank, a.1.path.as_str()).cmp(&(b.0, b.1.rank, b.1.path.as_str()))
    });
    let mut files: Vec<FileContribution> = keyed.into_iter().map(|(_, file)| file).collect();
    inherit(&mut files);
    // The walk runs from the lowest pack to the highest and the last definition replaces the
    // earlier one, without a message in vanilla.
    let mut builder = RegistryBuilder::new(
        CombineRule::LastLoadedWins {
            replaced: Some(Notice::shadowed()),
        },
        CombineRule::top_wins(),
        stack.options,
    );
    for file in files {
        for finding in file.findings {
            builder.push_diagnostic(finding);
        }
        for contribution in file.contributions {
            builder.push(contribution);
        }
    }
    builder.finish()
}

/// The loader of `geometries()`: `models/mobs.json`, `models/entity/**/*.json` and
/// `models/blocks/**/*.json`, combined per identifier, the last definition of the walk winning.
pub const fn load_geometries() -> CollectionLoader<Registry<Geometry>, FileContribution> {
    CollectionLoader {
        walk: GEOMETRY_WALK,
        parse: parse_walked,
        combine,
        // The combine asks the stack about no other collection and names no extra files.
        needs: &[],
        then: None,
    }
}
