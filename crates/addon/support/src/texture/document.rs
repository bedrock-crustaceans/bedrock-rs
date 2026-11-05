//! The documents of the texture files: no `format_version` is read on any path of the
//! unversioned ones, and a finding carries its pack and file. The kit is core's
//! [`document::unversioned`](bedrock_addon_core::document::unversioned); these are its names
//! here.

use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Severity};
use bedrock_addon_core::document::{self, Document, Origin, Rejected};
use bedrock_addon_core::load::WalkFile;
use bedrock_addon_core::pack_identity::PackPath;

/// The origin of a file read on its own. `unversioned::lone`.
pub fn detached(path: &PackPath) -> Origin {
    document::unversioned::lone(path)
}

/// A document of an unversioned file: no `format_version` is read on any path.
/// `unversioned::document`.
pub fn unversioned<T>(value: T, origin: Origin, diagnostics: Diagnostics) -> Document<T> {
    document::unversioned::document(value, origin, diagnostics)
}

/// A rejection of the file `origin` with its findings. `unversioned::rejected`.
pub fn rejected(origin: Origin, diagnostics: Diagnostics) -> Rejected {
    document::unversioned::rejected(origin, diagnostics)
}

/// Gives every finding the document's pack and file.
pub fn locate(diagnostics: &mut Diagnostics, origin: &Origin) {
    let found = std::mem::take(diagnostics);
    for mut finding in found {
        finding.location.pack = Some(origin.pack.clone());
        finding.location.path = Some(origin.path.clone());
        diagnostics.push(finding);
    }
}

/// A finding about the whole file `origin`.
pub fn file_finding(code: DiagnosticCode, severity: Severity, origin: &Origin) -> Diagnostic {
    finding_at(code, severity, origin, JsonPath::new())
}

/// A finding at `path` of the file `origin`.
pub fn finding_at(
    code: DiagnosticCode,
    severity: Severity,
    origin: &Origin,
    path: JsonPath,
) -> Diagnostic {
    Diagnostic::error(code)
        .severity(severity)
        .at(path)
        .in_file(origin)
}

/// The origin of a walked file. `unversioned::origin_of`.
pub fn origin_of(file: &WalkFile) -> Origin {
    document::unversioned::origin_of(file)
}
