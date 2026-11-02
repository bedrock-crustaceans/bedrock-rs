//! The documents of the unversioned text files: no `format_version` is read on any path.

use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};

use bedrock_addon_core::diagnostic::Diagnostic;
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::document::Rejected;
use bedrock_addon_core::document::{Document, Origin, VersionInfo};
use bedrock_addon_core::version::FormatVersion;

/// The origin of a file of `pack`.
pub fn origin(pack: PackRef, side: PackSide, path: &PackPath) -> Origin {
    Origin::new(pack, side, path.clone(), None, Vec::new())
}

/// A document of an unversioned family: no `format_version` is read on any path.
pub fn document<T>(value: T, origin: Origin, mut diagnostics: Diagnostics) -> Document<T> {
    locate(&mut diagnostics, &origin);
    let none = FormatVersion::new(0, 0, 0);
    let version = VersionInfo::new(None, false, none, none, false);
    Document::new(value, origin, version, diagnostics)
}

/// A rejection with one finding.
pub fn reject(origin: Origin, finding: Box<Diagnostic>) -> Rejected {
    let mut diagnostics = Diagnostics::default();
    diagnostics.push(*finding);
    locate(&mut diagnostics, &origin);
    Rejected::new(origin, diagnostics)
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
