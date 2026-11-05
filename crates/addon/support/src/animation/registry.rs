//! The combine the animation and animation controller loaders run over the files' copies:
//! each file split into its definitions; per identifier, the higher pack's definition replaces
//! the lower one whole.

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics};
use bedrock_addon_core::document::{
    CombineRule, Contribution, Definition, Document, Notice, Registry, RegistryBuilder, Rejected,
};
use bedrock_addon_core::id::{AnimationRef, kind};
use bedrock_addon_core::load::{FileCopy, StackCx};
use bedrock_addon_core::pack_identity::PackRef;

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
