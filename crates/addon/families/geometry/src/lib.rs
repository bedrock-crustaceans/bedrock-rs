//! Geometry: `Geometry`, `GeometryFile`, `Bone`, `Cube`, `Locator`.
//!
//! Model geometry of a resource pack: `models/mobs.json`, `models/entity/**/*.json` and
//! `models/blocks/**/*.json`. Vanilla reads these with a loader of its own: the file is validated once against the schema of its own
//! `format_version`, upgraded in place to the newest shape, and each geometry is registered by
//! its identifier.
//!
//! The reader stays a schema walk on the family's walker (`schema_walk`), because vanilla validates
//! each file against the JSON schema of its `format_version` rather than reading a typed struct.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod history;

mod inherit;
mod molang;
mod read;
mod schema_walk;
mod types;
mod validate;
mod write;

#[cfg(test)]
mod tests;

use std::collections::HashMap;

use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, JsonPathSegment};
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, NoBeta, Notice, Origin, Placed,
    Registry, RegistryBuilder, Rejected, VersionInfo,
};
use bedrock_addon_core::id::{GeometryId, kind};
use bedrock_addon_core::json::{EncodeError, WriteStyle};
use bedrock_addon_core::load::{Accept, CollectionLoader, LoadContext, StackCx, Walk, WalkFile};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use history::GeometryHistory;
use inherit::inherit;
use read::{
    BLOCKS, ENTITY, MOBS, Outcome, detached_origin, finish, load, place_of, read_written,
    version_info,
};

pub use read::{load_file, load_keyed_file};
pub use types::{
    Bone, BoneLegacy, Cube, CubeUv, Description, FaceUv, Faces, FitToFrame, Geometry, GeometryFile,
    GeometryLegacy, Inherited, ItemDisplayTransform, ItemDisplayTransforms,
    ItemDisplayTransformsIter, Locator, PolyMesh, Polys, TextureMesh,
};

impl bedrock_addon_core::__private::Sealed for GeometryFile {}
impl bedrock_addon_core::__private::Sealed for Geometry {}

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
    ///; [`history::upgrade`](history::upgrade) finishes it.
    ///
    /// # Errors
    ///
    /// Returns [`Rejected`] when the file cannot be read into a value at all, such as bytes that
    /// are not JSON or a document the family refuses; it carries every finding about the file.
    pub fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> Result<Document<GeometryHistory>, Rejected> {
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
    document: Document<GeometryHistory>,
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
#[doc(hidden)]
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
const GEOMETRY_WALK: Walk = Walk::json_tree_in(&["models"])
    .accept(Accept::Fn(is_geometry_file))
    // No subpack-shadowed copies are read.
    .read_error(UNREAD);

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
///
/// By hand rather than `load::merged`: the gates read the path without its subpack folder, a
/// file holds several geometries with findings of their own, and a legacy parent is resolved
/// across the stack before the registry is built.
#[doc(hidden)]
pub const LOADER: CollectionLoader<Registry<Geometry>, FileContribution> = CollectionLoader {
    walk: GEOMETRY_WALK,
    parse: parse_walked,
    combine,
    // The combine asks the stack about no other collection and names no extra files.
    needs: &[],
    then: None,
};
