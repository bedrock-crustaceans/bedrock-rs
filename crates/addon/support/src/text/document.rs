//! The documents of the unversioned text files: no `format_version` is read on any path. The
//! kit is core's [`unversioned`]; these are its names here.

use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};

use bedrock_addon_core::diagnostic::Diagnostic;
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::document::{Document, Origin, Rejected, unversioned};

/// The origin of a file of `pack`. [`unversioned::origin`].
pub fn origin(pack: PackRef, side: PackSide, path: &PackPath) -> Origin {
    unversioned::origin(pack, side, path)
}

/// A document of an unversioned family: no `format_version` is read on any path.
/// [`unversioned::document`].
pub fn document<T>(value: T, origin: Origin, diagnostics: Diagnostics) -> Document<T> {
    unversioned::document(value, origin, diagnostics)
}

/// A rejection with one finding. [`unversioned::reject`].
pub fn reject(origin: Origin, finding: Box<Diagnostic>) -> Rejected {
    unversioned::reject(origin, finding)
}
