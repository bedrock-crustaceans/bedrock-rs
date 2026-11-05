//! What loading an animation file and an animation controller file share: the document of a
//! parsed file and the combine both loaders run over the files' copies (each file split into
//! its definitions; per identifier, the higher pack's definition replaces the lower one whole).

use super::read::Parsed;
use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics};
use bedrock_addon_core::document::{
    CombineRule, Contribution, Definition, Document, Notice, Origin, Registry, RegistryBuilder,
    Rejected, VersionInfo,
};
use bedrock_addon_core::id::{AnimationRef, kind};
use bedrock_addon_core::load::{FileCopy, LoadContext, StackCx};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide, PackType};
use bedrock_addon_core::version::{FormatVersion, SemVersion};

/// The side a context's pack is on (a resource pack is the client).
pub fn side_of(cx: &LoadContext) -> PackSide {
    if cx.pack_type == PackType::Resource {
        PackSide::Resource
    } else {
        PackSide::Behavior
    }
}

/// Gives every finding the file's pack and path, pushed one by one: the places the findings
/// held are not carried over.
fn locate(diagnostics: Diagnostics, origin: &Origin) -> Diagnostics {
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

/// Combine the files' copies into one registry: each file read whole is split into one
/// document per definition (`definitions` gives them, `member` is the object they sit under),
/// and per identifier the higher pack's definition replaces the lower one whole; `shadowed` is
/// the finding for a replaced one.
///
/// Each definition's findings (those below `<member>/<key>`) go with it, the rest of the
/// file's to the registry; a rejected file is one rejection with no identifier.
pub fn combine<T: Definition<Kind = kind::Animation>, F>(
    parsed: Vec<FileCopy<F>>,
    stack: &StackCx<'_>,
    member: &str,
    definitions: fn(F) -> Vec<(String, T)>,
    shadowed: Notice,
) -> Registry<T> {
    // Within a pack, the game visits the active subpack's files before the base pack's, so with
    // per-identifier last-wins a base-pack definition replaces a subpack one. A file whose path
    // differs from its key lies below a subpack.
    let mut group = 0_usize;
    let mut previous: Option<PackRef> = None;
    let mut keyed: Vec<(usize, bool, Result<Document<F>, Rejected>)> =
        Vec::with_capacity(parsed.len());
    for (key, file) in parsed {
        let origin = match &file {
            Ok(document) => document.origin(),
            Err(rejected) => &rejected.origin,
        };
        if previous.as_ref() != Some(&origin.pack) {
            group += 1;
            previous = Some(origin.pack.clone());
        }
        let base = origin.path == key;
        keyed.push((group, base, file));
    }
    keyed.sort_by_key(|(group, base, _)| (*group, *base));
    let mut builder = RegistryBuilder::new(
        CombineRule::TopWins {
            shadowed: Some(shadowed),
        },
        CombineRule::top_wins(),
        stack.options,
    );
    for (_, _, file) in keyed {
        split(file, member, definitions, &mut builder);
    }
    builder.finish()
}

/// Splits a loaded file into one document per definition, pushed in file order after the
/// file's findings that belong to no one definition.
fn split<T: Definition<Kind = kind::Animation>, F>(
    result: Result<Document<F>, Rejected>,
    member: &str,
    definitions: fn(F) -> Vec<(String, T)>,
    builder: &mut RegistryBuilder<T>,
) {
    let document = match result {
        Ok(document) => document,
        Err(rejected) => {
            builder.push(Contribution::Rejected(None, rejected));
            return;
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
    for finding in rest {
        builder.push_diagnostic(finding);
    }
    for contribution in contributions {
        builder.push(contribution);
    }
}
