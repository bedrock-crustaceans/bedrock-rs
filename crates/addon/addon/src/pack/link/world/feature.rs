//! The references a feature names: the features a feature names; what it names in other
//! modules (blocks, structures) is in the pair files.

use super::super::{At, Findings, LinkCx, Target, kinds, unresolved};
use crate::diagnostic::Severity;
use crate::families::feature::{self, Reference};

/// The rules of the features.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    feature_references(cx, out);
}

/// What each feature names, through this file and the pair files. A member that names a
/// feature no pack defines (the app's own definitions count, and a file read later than the
/// one that names it) is an `UnresolvedReference` information at the member.
fn feature_references(cx: &LinkCx<'_>, out: &mut Findings) {
    for (_, document) in cx.behavior().features() {
        let references = feature::references(document.value());
        for (path, reference) in &references {
            if let Reference::Feature(id) = reference
                && let Target::Missing(hint) = cx.feature(id)
            {
                // Feature references are lazy: the game logs nothing at load, so the crate
                // reports an information.
                let at = At::new(document.origin(), path.clone(), Severity::Info);
                out.push(unresolved(at, kinds::FEATURE, id.as_str(), hint));
            }
        }
        #[cfg(feature = "block")]
        super::feature_block::feature_blocks(cx, out, document, &references);
        #[cfg(feature = "structure")]
        super::feature_structure::feature_template(cx, out, document);
    }
}
