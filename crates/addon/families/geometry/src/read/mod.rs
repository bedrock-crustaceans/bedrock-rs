//! Reading one geometry file: the text, the envelope and its version gates, the parse by the
//! struct of the file's era ([`era`]), the upgrade to the newest shape and the identifier and
//! parse-time checks.

mod era;

pub(crate) use era::{
    Era, LocatorForm, V1_12_0, V1_19_30, read_file, read_locators, read_poly_mesh,
    read_render_group_id, read_texture_mesh,
};

use std::borrow::Cow;

use crate::GeometryFile;
use crate::history::v1_8_0::MAX_FORMAT_VERSION as KEYED_MAXIMUM;
use crate::history::{GeometryHistory, VERSIONS};
use crate::molang;
use crate::texts::{
    DID_NOT_VALIDATE, EXCEEDS_GAME_VERSION, INTERNAL_BLOCKS_FLOOR, NOT_AN_OBJECT, VERSION_NOT_VALID,
};
use crate::validate::{Place, first_bad_identifier, parse_time};
use crate::walk::rules;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Severity, VanillaText,
};
use bedrock_addon_core::document::{Document, Origin, Rejected, VersionInfo};
use bedrock_addon_core::json::node_tree::{Kind, Node, PARSE_ERROR, Walk, text};
use bedrock_addon_core::json::{
    JsonSlice, MAX_JSON_DEPTH, Value, check_depth, parse_at, strip_bom,
};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput, SemVersion};
use bedrock_addon_core::versioning::{ParseCx, UpgradeCx};

/// The version a file without a readable `format_version` is read as (vanilla's `"1.0"`
/// default).
pub(crate) const DEFAULT_VERSION: FormatVersion = FormatVersion::new(1, 0, 0);
/// The directory of block geometry.
pub(crate) const BLOCKS: &str = "models/blocks/";
/// The directory of entity geometry, where the writer puts a file.
pub(crate) const ENTITY: &str = "models/entity";
/// The file of the old built-in mob geometry.
pub(crate) const MOBS: &str = "models/mobs.json";

fn member(name: &str) -> JsonPath {
    JsonPath::new().child(name)
}

fn did_not_validate(diagnostics: &mut Diagnostics) {
    diagnostics.push(Diagnostic::error(DiagnosticCode::InvalidValue {
        text: text(DID_NOT_VALIDATE, &[]),
    }));
}

/// What the envelope of a file gave: the struct of its version, not upgraded.
pub(crate) struct Envelope {
    pub(crate) any: GeometryHistory,
    /// `format_version` as the file has it; `None` when absent or unreadable.
    pub(crate) written: Option<FormatVersion>,
    /// The version the file is read as.
    pub(crate) effective: FormatVersion,
    /// The version the file states, when the keyed door reads it by an earlier one.
    pub(crate) stated: FormatVersion,
}

/// What reading a file gave.
pub(crate) enum Outcome<T> {
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
    // Stripped, not `Prepared`: blanking would shift every span by three, so that switch is
    // a behaviour-bearing change for later.
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
    let walk = Walk::new(rules(true), &JsonPath::new());
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
    let as_written = if node.kind() == Kind::String {
        parse_at::<String>(JsonSlice::new(node.text())).unwrap_or_default()
    } else {
        node.text().to_owned()
    };
    let version = (node.kind() == Kind::String)
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
pub(crate) fn read_written(
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
        Kind::Null => return Outcome::Empty,
        Kind::Object => {}
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
    let Some(any) = GeometryHistory::parse(slice, &parse, diagnostics) else {
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

pub(crate) fn place_of(path: &PackPath, cx: &LoadContext) -> Place {
    Place {
        base_game: cx.is_base_game_pack,
        blocks: path.as_str().starts_with(BLOCKS),
    }
}

/// A file after its upgrade and its checks.
pub(crate) struct Finished {
    /// The file with the geometries that are registered.
    pub(crate) file: GeometryFile,
    /// The identifiers of the geometries the identifier loop stopped at and after.
    pub(crate) tail: Vec<String>,
}

/// The steps from the file's struct to the newest one, then the identifier loop and the
/// parse-time checks. `None` when a step or the `binding` compile fails the file.
pub(crate) fn finish(
    any: GeometryHistory,
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
    if !molang::run(&mut file, written, cx, diagnostics) {
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
pub(crate) struct Loaded {
    pub(crate) finished: Option<Finished>,
    pub(crate) version: VersionInfo,
    pub(crate) diagnostics: Diagnostics,
    /// The file was empty or `null`.
    pub(crate) empty: bool,
}

pub(crate) fn version_info(
    written: Option<FormatVersion>,
    effective: FormatVersion,
) -> VersionInfo {
    let parsed_as = VERSIONS[GeometryHistory::index_of(effective)];
    VersionInfo::new(
        written,
        false,
        effective,
        parsed_as,
        parsed_as != VERSIONS[VERSIONS.len() - 1],
    )
}

pub(crate) fn load(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Loaded {
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

pub(crate) fn detached_origin(path: &PackPath) -> Origin {
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
