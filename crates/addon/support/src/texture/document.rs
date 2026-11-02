//! The documents of the texture files: no `format_version` is read on any path of the
//! unversioned ones, and a finding carries its pack and file.

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity,
};
use bedrock_addon_core::document::{Document, Origin, Rejected, VersionInfo};
use bedrock_addon_core::load::WalkFile;
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::FormatVersion;

/// The origin of a file read on its own.
#[must_use]
pub fn detached(path: &PackPath) -> Origin {
    Origin::new(
        PackRef::app_data(),
        PackSide::Resource,
        path.clone(),
        None,
        Vec::new(),
    )
}

/// A document of an unversioned file: no `format_version` is read on any path.
pub fn unversioned<T>(value: T, origin: Origin, mut diagnostics: Diagnostics) -> Document<T> {
    locate(&mut diagnostics, &origin);
    let none = FormatVersion::new(0, 0, 0);
    let version = VersionInfo::new(None, false, none, none, false);
    Document::new(value, origin, version, diagnostics)
}

/// A rejection of the file `origin` with its findings.
#[must_use]
pub fn rejected(origin: Origin, mut diagnostics: Diagnostics) -> Rejected {
    locate(&mut diagnostics, &origin);
    Rejected::new(origin, diagnostics)
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
#[must_use]
pub fn file_finding(code: DiagnosticCode, severity: Severity, origin: &Origin) -> Diagnostic {
    finding_at(code, severity, origin, JsonPath::new())
}

/// A finding at `path` of the file `origin`.
#[must_use]
pub fn finding_at(
    code: DiagnosticCode,
    severity: Severity,
    origin: &Origin,
    path: JsonPath,
) -> Diagnostic {
    let location = Location::new(
        Some(origin.pack.clone()),
        Some(origin.path.clone()),
        path,
        None,
    );
    Diagnostic::new(code, severity, Phase::Parse, location)
}

/// The origin of a walked file.
pub fn origin_of(file: &WalkFile) -> Origin {
    Origin::new(
        file.pack.clone(),
        file.side,
        file.path.clone(),
        None,
        Vec::new(),
    )
}
