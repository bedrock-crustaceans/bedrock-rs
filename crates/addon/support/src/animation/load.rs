//! What loading an animation file and an animation controller file share: the document of a
//! parsed file, the per-definition split of a walked file, and the combine rule (per identifier,
//! the higher pack's definition replaces the lower one whole).

use super::read::Parsed;
use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics};
use bedrock_addon_core::document::{
    CombineRule, Contribution, Definition, Document, Notice, Origin, Registry, RegistryBuilder,
    Rejected, VersionInfo,
};
use bedrock_addon_core::id::{AnimationRef, kind};
use bedrock_addon_core::load::{LoadContext, StackCx, WalkFile};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide, PackType};
use bedrock_addon_core::version::{FormatVersion, SemVersion};

/// The side a context's pack is on (a resource pack is the client).
#[must_use]
pub fn side_of(cx: &LoadContext) -> PackSide {
    if cx.pack_type == PackType::Resource {
        PackSide::Resource
    } else {
        PackSide::Behavior
    }
}

/// Gives every finding the file's pack and path.
#[must_use]
pub fn locate(diagnostics: Diagnostics, origin: &Origin) -> Diagnostics {
    let mut out = Diagnostics::default();
    for mut finding in diagnostics {
        finding.location.pack = Some(origin.pack.clone());
        finding.location.path = Some(origin.path.clone());
        out.push(finding);
    }
    out
}

/// The version record of a file whose `format_version` was written as `written`, parsed as
/// `parsed_as`, and whether it was upgraded to get there.
#[must_use]
pub fn version_info(written: &SemVersion, parsed_as: FormatVersion, upgraded: bool) -> VersionInfo {
    let written = written.format_version();
    VersionInfo::new(Some(written), false, written, parsed_as, upgraded)
}

/// One file's result as a document of the whole file.
///
/// # Errors
///
/// `Rejected` (with the file's located diagnostics) when the parse produced no value or the file's
/// `format_version` was not read.
pub fn whole<T>(
    parsed: Parsed<T>,
    origin: Origin,
    parsed_as: impl FnOnce(&SemVersion) -> (FormatVersion, bool),
) -> Result<Document<T>, Rejected> {
    let diagnostics = locate(parsed.diagnostics, &origin);
    match (parsed.value, parsed.written) {
        (Some(value), Some(written)) => {
            let (as_, upgraded) = parsed_as(&written);
            Ok(Document::new(
                value,
                origin,
                version_info(&written, as_, upgraded),
                diagnostics,
            ))
        }
        _ => Err(Rejected::new(origin, diagnostics)),
    }
}

/// `<directory>/<first key without its prefix>.json`.
#[must_use]
pub fn file_path(directory: &str, prefix: &str, first: Option<&String>) -> PackPath {
    let name = first.map_or("unnamed", |key| {
        key.get(..prefix.len())
            .filter(|head| head.eq_ignore_ascii_case(prefix))
            .map_or(key.as_str(), |_| &key[prefix.len()..])
    });
    PackPath::new(&format!("{directory}/{name}.json"))
        .unwrap_or_else(|_| PackPath::from_static("animations/unnamed.json"))
}

// ---- the collections --------------------------------------------------------------------------

/// What one walked file gives a registry: its definitions, or its rejection, and the file's
/// findings that belong to no one definition.
#[derive(Debug)]
pub struct FileContribution<T: Definition> {
    contributions: Vec<Contribution<T>>,
    findings: Vec<Diagnostic>,
    /// The file's pack.
    pack: Option<PackRef>,
    /// The file came from the pack's active subpack.
    subpack: bool,
}

impl<T: Definition> FileContribution<T> {
    /// The same contribution with the pack and the subpack-ness of `file` set (a file whose path
    /// differs from its key lies below a subpack).
    #[must_use]
    pub fn with_file(mut self, file: &WalkFile) -> Self {
        self.pack = Some(file.pack.clone());
        self.subpack = file.path != file.key;
        self
    }
}

/// Splits a loaded file into one document per definition: each definition's findings (those
/// below `<member>/<key>`) go with it, the rest to the registry.
pub fn split<T: Definition<Kind = kind::Animation>, F>(
    result: Result<Document<F>, Rejected>,
    member: &str,
    definitions: impl FnOnce(F) -> Vec<(String, T)>,
) -> FileContribution<T> {
    let document = match result {
        Ok(document) => document,
        Err(rejected) => {
            return FileContribution {
                contributions: vec![Contribution::Rejected(None, rejected)],
                findings: Vec::new(),
                pack: None,
                subpack: false,
            };
        }
    };
    let origin = document.origin().clone();
    let version = document.version().clone();
    let mut rest: Vec<Diagnostic> = document.diagnostics().iter().cloned().collect();
    let mut contributions = Vec::new();
    for (key, value) in definitions(document.into_value()) {
        let prefix = bedrock_addon_core::diagnostic::JsonPath::of_keys([member, key.as_str()]);
        let (mine, others): (Vec<_>, Vec<_>) = rest
            .into_iter()
            .partition(|d| d.location.json_path.starts_with(&prefix));
        rest = others;
        let mut diagnostics = Diagnostics::default();
        for d in mine {
            diagnostics.push(d);
        }
        let mut origin = origin.clone();
        origin.key = Some(key.clone().into_boxed_str());
        let id = AnimationRef::parse(key.to_ascii_lowercase());
        contributions.push(Contribution::Stable(
            id,
            Document::new(value, origin, version.clone(), diagnostics),
        ));
    }
    FileContribution {
        contributions,
        findings: rest,
        pack: None,
        subpack: false,
    }
}

/// The origin of a walked file.
pub fn walk_origin(file: &WalkFile) -> Origin {
    Origin::new(
        file.pack.clone(),
        file.side,
        file.path.clone(),
        None,
        Vec::new(),
    )
}

/// A `.json` file: vanilla compares the four bytes of the extension as written.
#[must_use]
pub fn is_json(path: &PackPath) -> bool {
    path.as_str()
        .rsplit_once('.')
        .is_some_and(|(_, extension)| extension == "json")
}

/// Combine the files' contributions into one registry: per identifier the higher pack's definition
/// replaces the lower one whole; `shadowed` is the finding for a replaced one.
#[must_use]
pub fn combine<T: Definition>(
    mut parsed: Vec<FileContribution<T>>,
    stack: &StackCx<'_>,
    shadowed: Notice,
) -> Registry<T> {
    // Within a pack, the game visits the active subpack's files before the base pack's, so with
    // per-identifier last-wins a base-pack definition replaces a subpack one.
    let mut group = 0_usize;
    let mut previous: Option<PackRef> = None;
    let mut keyed: Vec<(usize, bool, FileContribution<T>)> = Vec::with_capacity(parsed.len());
    for file in parsed.drain(..) {
        if file.pack != previous {
            group += 1;
            previous.clone_from(&file.pack);
        }
        keyed.push((group, !file.subpack, file));
    }
    keyed.sort_by_key(|(group, base, _)| (*group, *base));
    let parsed = keyed.into_iter().map(|(_, _, file)| file);
    let mut builder = RegistryBuilder::new(
        CombineRule::TopWins {
            shadowed: Some(shadowed),
        },
        CombineRule::top_wins(),
        stack.options,
    );
    for file in parsed {
        for finding in file.findings {
            builder.push_diagnostic(finding);
        }
        for contribution in file.contributions {
            builder.push(contribution);
        }
    }
    builder.finish()
}
