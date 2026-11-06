//! The document kit of a file no `format_version` is read on: its origin, its document, its
//! rejection. The one copy of the `lone`/`origin`/`document`/`reject` kit that
//! the families without a `format_version` share.

use super::{Document, Origin, Rejected, VersionInfo};
use crate::diagnostic::{Diagnostic, Diagnostics};
use crate::load::WalkFile;
use crate::pack_identity::{PackPath, PackRef, PackSide};
use crate::version::FormatVersion;

/// The origin of a file read on its own, in no pack: the app-data handle, resource side.
pub fn lone(path: &PackPath) -> Origin {
    origin(PackRef::app_data(), PackSide::Resource, path)
}

/// The origin of the file `path` of `pack` on `side`.
pub fn origin(pack: PackRef, side: PackSide, path: &PackPath) -> Origin {
    Origin::file(pack, side, path.clone())
}

/// The origin of a walked file: its pack, side and path in the pack (subpack folder included).
pub fn origin_of(file: &WalkFile) -> Origin {
    origin(file.pack.clone(), file.side, &file.path)
}

/// The version information of a file no version is read on, at `version` (0.0.0 for most;
/// the engine version or a family's own for the few that state one).
pub fn version_at(version: FormatVersion) -> VersionInfo {
    VersionInfo::new(None, false, version, version, false)
}

/// The version information of a file no version is read on: 0.0.0.
pub fn version() -> VersionInfo {
    version_at(FormatVersion::new(0, 0, 0))
}

/// The document of `value` read from `origin`, at version 0.0.0, every finding given the
/// file's pack and path.
pub fn document<T>(value: T, origin: Origin, diagnostics: Diagnostics) -> Document<T> {
    document_at(value, origin, diagnostics, FormatVersion::new(0, 0, 0))
}

/// [`document`] at `version`.
pub fn document_at<T>(
    value: T,
    origin: Origin,
    mut diagnostics: Diagnostics,
    version: FormatVersion,
) -> Document<T> {
    locate(&mut diagnostics, &origin);
    Document::new(value, origin, version_at(version), diagnostics)
}

/// The rejection of `origin` with one finding, given the file's pack and path.
///
/// Like [`rejected`], it drops the moves and places the findings held, unlike
/// [`Rejected::located`], which keeps them.
pub fn reject(origin: Origin, finding: impl Into<Box<Diagnostic>>) -> Rejected {
    let mut diagnostics = Diagnostics::default();
    diagnostics.push(*finding.into());
    rejected(origin, diagnostics)
}

/// The rejection of `origin` with its findings, each given the file's pack and path.
///
/// The list is rebuilt from its findings, as the kits it replaces do, so the moves and places
/// it held are dropped; [`Rejected::located`] keeps them.
pub fn rejected(origin: Origin, mut diagnostics: Diagnostics) -> Rejected {
    locate(&mut diagnostics, &origin);
    Rejected::new(origin, diagnostics)
}

/// Gives every finding the file's pack and path, the way the kits it replaces do: taken out
/// and pushed back, so what the push records for each finding is recorded anew.
// TODO(phase 1 step 4): this loop becomes the existing `Diagnostics::locate` (and `reject`/
// `rejected` become `Rejected::located`). That switch is behaviour-bearing, per family: the
// repush here drops the moves and places, `locate` keeps them in place, so each family moves
// over in its own change with a test of what it now keeps.
// The unknown members the list holds are not findings: they are taken first and put back.
fn locate(diagnostics: &mut Diagnostics, origin: &Origin) {
    let mut found = std::mem::take(diagnostics);
    let unknown = found.take_unknown();
    for mut finding in found {
        finding.location.pack = Some(origin.pack.clone());
        finding.location.path = Some(origin.path.clone());
        diagnostics.push(finding);
    }
    if !unknown.is_empty() {
        *diagnostics.unknown_mut() = unknown;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::{DiagnosticCode, JsonPath, Location, Phase, Severity};
    use crate::json::Value;

    fn path(text: &str) -> PackPath {
        PackPath::new(text).unwrap()
    }

    fn finding() -> Diagnostic {
        Diagnostic::new(
            DiagnosticCode::MalformedInput {
                message: "not JSON".to_owned(),
                text: None,
            },
            Severity::Error,
            Phase::Parse,
            Location::default(),
        )
    }

    /// The `locate` of the kits this replaces, as `support/fixed_path.rs` spells it.
    fn kit_locate(diagnostics: &mut Diagnostics, origin: &Origin) {
        let found = std::mem::take(diagnostics);
        for mut finding in found {
            finding.location.pack = Some(origin.pack.clone());
            finding.location.path = Some(origin.path.clone());
            diagnostics.push(finding);
        }
    }

    #[test]
    fn a_lone_file_is_in_no_pack_on_the_resource_side() {
        let origin = lone(&path("font/glyph_E0.json"));
        assert_eq!(origin.pack, PackRef::app_data());
        assert_eq!(origin.side, PackSide::Resource);
        assert_eq!(origin.path, path("font/glyph_E0.json"));
    }

    #[test]
    fn a_walked_file_keeps_its_path_with_the_subpack_folder() {
        let file = WalkFile {
            pack: PackRef::app_data(),
            side: PackSide::Behavior,
            path: path("subpacks/a/texts/en_US.lang"),
            key: path("texts/en_US.lang"),
            bytes: bytes::Bytes::new(),
        };
        let origin = origin_of(&file);
        assert_eq!(origin.path, file.path);
        assert_eq!(origin.side, PackSide::Behavior);
    }

    #[test]
    fn a_document_is_version_zero_with_its_findings_located() {
        let origin = lone(&path("credits/credits.json"));
        let mut diagnostics = Diagnostics::default();
        diagnostics.push(finding());
        let mut expected = diagnostics.clone();
        kit_locate(&mut expected, &origin);
        let document = document(7, origin.clone(), diagnostics);
        assert_eq!(*document, 7);
        assert_eq!(document.version(), &version());
        assert_eq!(document.version().effective, FormatVersion::new(0, 0, 0));
        assert_eq!(document.version().written, None);
        assert!(!document.version().upgraded);
        assert_eq!(document.diagnostics(), &expected);
        let finding = document.diagnostics().iter().next().unwrap();
        assert_eq!(finding.location.path, Some(origin.path.clone()));
        assert_eq!(finding.location.pack, Some(origin.pack.clone()));
    }

    #[test]
    fn a_document_keeps_the_unknown_members_of_its_findings() {
        let mut diagnostics = Diagnostics::default();
        diagnostics.push(finding());
        diagnostics
            .unknown_mut()
            .put(&JsonPath::of_keys(["a"]), "zz", Value::from(1i64));
        let document = document((), lone(&path("a.json")), diagnostics);
        assert_eq!(
            document.unknown().get(&JsonPath::of_keys(["a"]), "zz"),
            Some(&Value::from(1i64))
        );
        assert_eq!(document.diagnostics().len(), 1);
    }

    #[test]
    fn document_at_states_its_version() {
        let version = FormatVersion::new(1, 13, 0);
        let document = document_at((), lone(&path("a.json")), Diagnostics::default(), version);
        assert_eq!(document.version().effective, version);
        assert_eq!(document.version().parsed_as, version);
        assert_eq!(document.version().written, None);
    }

    #[test]
    fn a_rejection_takes_a_finding_boxed_or_not() {
        let origin = lone(&path("ui/_ui_defs.json"));
        let mut expected = Diagnostics::default();
        expected.push(finding());
        kit_locate(&mut expected, &origin);
        let plain = reject(origin.clone(), finding());
        let boxed = reject(origin.clone(), Box::new(finding()));
        assert_eq!(plain.diagnostics, expected);
        assert_eq!(boxed.diagnostics, expected);
        assert_eq!(plain.origin, origin);
        let many = rejected(origin, {
            let mut d = Diagnostics::default();
            d.push(finding());
            d
        });
        assert_eq!(many.diagnostics, expected);
    }
}
