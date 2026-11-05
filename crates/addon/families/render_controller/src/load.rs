//! Loading render controllers: `Content` / `Definition` / `Placed`, the walk of
//! `render_controllers/` and the combine rule (per name the highest pack's controller wins; a
//! controller that failed leaves the lower definition in place).

use indexmap::IndexMap;

use super::read::{self, Parsed, group_by_controller};
use super::{RenderController, RenderControllerFile};
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Phase, Severity};
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, JsonContent, NoBeta, Notice, Origin,
    Placed, Registry, RegistryBuilder, Rejected, VersionInfo, default_path,
};
use bedrock_addon_core::id::{RenderControllerId, kind};
use bedrock_addon_core::json::{EncodeError, WriteStyle};
use bedrock_addon_core::load::{CollectionLoader, LoadContext, StackCx, Walk, WalkFile};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::{FormatVersion, SemVersion};

impl bedrock_addon_core::__private::Sealed for RenderControllerFile {}
impl bedrock_addon_core::__private::Sealed for RenderController {}

/// The one struct's version: every accepted file is parsed as it.
const PARSED_AS: FormatVersion = FormatVersion::new(1, 8, 0);

fn version_info(written: Option<&SemVersion>) -> VersionInfo {
    let written = written.map(SemVersion::format_version);
    VersionInfo::new(
        written,
        false,
        written.unwrap_or(PARSED_AS),
        PARSED_AS,
        false,
    )
}

impl Content for RenderControllerFile {
    const NAME: &'static str = "render controller file";
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// The file with the controllers that loaded. A controller the game drops on its own (no
    /// `textures`, Molang in `geometry`, `textures` or `arrays` that does not compile, or a
    /// `materials` value written as a number) is kept apart in
    /// [`dropped`](RenderControllerFile::dropped) and its errors stay in the document's
    /// findings: the file itself loaded.
    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        let origin = Origin::file(PackRef::app_data(), PackSide::Resource, path.clone());
        let Parsed {
            file,
            dropped,
            written,
            mut diagnostics,
        } = read::parse_file(path, bytes, cx);
        for (_, findings) in dropped {
            diagnostics.extend(findings);
        }
        let diagnostics = diagnostics.located(&origin);
        match file {
            Some(file) => Ok(Document::new(
                file,
                origin,
                version_info(written.as_ref()),
                diagnostics,
            )),
            None => Err(Rejected::new(origin, diagnostics)),
        }
    }

    /// The file as JSON. A capital the model holds (text the game's case folding did not
    /// reach) is written as a `\u00XX` escape wherever the folding would lower it, so the game
    /// and this crate read the written file as the value that was written.
    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        let mut text = Vec::new();
        bedrock_addon_core::json::write_json(&self.to_value()?, &mut text, style)?;
        out.extend(super::fold::protect(&text));
        Ok(())
    }
}

impl JsonContent for RenderControllerFile {}

impl Definition for RenderController {
    type Kind = kind::RenderController;
    type File = RenderControllerFile;
    type Beta = NoBeta<kind::RenderController>;
}

/// The directory of the family.
const DIRECTORY: &str = "render_controllers";

/// The file name of a file without controllers.
const EMPTY_FILE: &str = "render_controllers/unnamed.json";

impl Placed for RenderControllerFile {
    /// `render_controllers/<first key>.json` (`render_controllers/controller.render.cow.json`):
    /// the first loaded controller's key, or the first dropped one's in a file none of whose
    /// controllers loaded; a file without controllers is `render_controllers/unnamed.json`.
    fn default_path(&self) -> PackPath {
        let mut keys = self.render_controllers.keys().chain(self.dropped.keys());
        match keys.next() {
            Some(first) => default_path(
                "render_controllers",
                &RenderControllerId::parse(first.as_str()),
            ),
            None => PackPath::from_static(EMPTY_FILE),
        }
    }
}

// ---- the collection ----------------------------------------------------------------------------

/// What one walked file gives the registry: its controllers, loaded or dropped, or its
/// rejection, and the file's findings that belong to no one controller.
#[derive(Debug)]
pub struct FileContribution {
    contributions: Vec<Contribution<RenderController>>,
    findings: Vec<Diagnostic>,
}

/// Every `.json` file below `render_controllers/`, packs bottom-up: the game walks its stack
/// from the last pack to the first and replaces a name's controller by the one it loads last.
/// A file that is no `.json` file is no file of the walk: `combine` reports it from the
/// pack's file list, its bytes unread. Sub-folders are read in every pack (the game reads them
/// recursively except in packs shipped inside the app).
const WALK: Walk = Walk::json_tree_in(&[DIRECTORY]);

fn parse_walk_file(file: &WalkFile, cx: &LoadContext) -> FileContribution {
    let origin = Origin::file(file.pack.clone(), file.side, file.path.clone());
    let Parsed {
        file: parsed,
        dropped,
        written,
        diagnostics,
    } = read::parse_file(&file.path, &file.bytes, cx);
    let Some(parsed) = parsed else {
        return FileContribution {
            contributions: vec![Contribution::Rejected(
                None,
                Rejected::located(origin.clone(), diagnostics),
            )],
            findings: Vec::new(),
        };
    };
    let version = version_info(written.as_ref());
    let keyed = |key: &str| {
        let mut origin = origin.clone();
        origin.key = Some(key.into());
        origin
    };
    let mut contributions = Vec::new();
    // One pass over the findings, however many controllers the file holds.
    let (mut by_key, mut rest) = group_by_controller(diagnostics, |key| {
        parsed.render_controllers.contains_key(key)
    });
    for (key, controller) in parsed.render_controllers {
        let mine = by_key.swap_remove(&key).unwrap_or_default();
        let origin = keyed(&key);
        contributions.push(Contribution::Stable(
            RenderControllerId::parse(key),
            Document::new(
                controller,
                origin.clone(),
                version.clone(),
                mine.located(&origin),
            ),
        ));
    }
    for (key, findings) in dropped {
        let origin = keyed(&key);
        contributions.push(Contribution::Rejected(
            Some(RenderControllerId::parse(key)),
            Rejected::located(origin.clone(), findings),
        ));
    }
    // Nothing is left grouped: every key the grouping accepted was a controller's.
    for (_, findings) in by_key {
        rest.extend(findings);
    }
    FileContribution {
        contributions,
        findings: rest.located(&origin).into_iter().collect(),
    }
}

/// The registry: per name the highest pack's controller; each controller it replaces is noted with
/// `RenderControllerShadowed` (the crate's own note; the game replaces silently), and a controller
/// that failed above one that loaded with `RenderControllerFallback`.
fn combine(parsed: Vec<FileContribution>, stack: &StackCx<'_>) -> Registry<RenderController> {
    let mut builder = RegistryBuilder::new(
        CombineRule::TopWinsFallback {
            fallback: Some(Notice {
                code: DiagnosticCode::RenderControllerFallback,
                // The game keeps the lower one silently, so this is the crate's own information.
                severity: Severity::Info,
            }),
        },
        CombineRule::top_wins(),
        stack.options,
    );
    let shadowed = |origin: &Origin| {
        Diagnostic::info(DiagnosticCode::RenderControllerShadowed)
            .phase(Phase::Validate)
            .in_file(origin)
    };
    // Inside one pack the controller loaded last replaces the earlier ones of its name (the
    // walk gives a pack's files in path order): how many definitions each pack has of each
    // name, so that every one but the last is noted and left out.
    let mut remaining: IndexMap<(Option<usize>, RenderControllerId), usize> = IndexMap::new();
    for contribution in parsed.iter().flat_map(|file| &file.contributions) {
        if let Contribution::Stable(id, document) = contribution {
            *remaining
                .entry((document.origin().pack.index(), id.clone()))
                .or_insert(0) += 1;
        }
    }
    // A file of the directory that is no `.json` file, from the packs' file lists: the game
    // skips it without a word, so this is the crate's own information.
    for (pack, ignored) in
        bedrock_addon_core::load::unread_files(stack, DIRECTORY, PackPath::is_json)
    {
        let file = Origin::file(pack.pack.clone(), PackSide::Resource, ignored.path.clone());
        builder.push_diagnostic(
            Diagnostic::info(DiagnosticCode::IgnoredFile { text: None }).in_file(&file),
        );
    }
    // The effective definition of each name so far, to note the one a higher pack replaces.
    let mut effective: IndexMap<RenderControllerId, Origin> = IndexMap::new();
    for file in parsed {
        for finding in file.findings {
            builder.push_diagnostic(finding);
        }
        for contribution in file.contributions {
            if let Contribution::Stable(id, document) = &contribution {
                let incoming = document.origin().clone();
                let left = remaining
                    .get_mut(&(incoming.pack.index(), id.clone()))
                    .map_or(0, |left| {
                        *left -= 1;
                        *left
                    });
                if left > 0 {
                    // A later controller of this pack replaces this one.
                    for finding in document.diagnostics().iter().cloned() {
                        builder.push_diagnostic(finding);
                    }
                    builder.push_diagnostic(shadowed(&incoming));
                    continue;
                }
                if let Some(lower) = effective.insert(id.clone(), incoming) {
                    builder.push_diagnostic(shadowed(&lower));
                }
            }
            builder.push(contribution);
        }
    }
    builder.finish()
}

/// The loader of `rp.render_controllers()`: every `.json` file below `render_controllers/` of
/// every resource pack, the highest pack's controller winning per name.
pub const fn load_registry() -> CollectionLoader<Registry<RenderController>, FileContribution> {
    CollectionLoader {
        walk: WALK,
        parse: parse_walk_file,
        combine,
        needs: &[],
        then: None,
    }
}
