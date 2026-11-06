//! The finding builders of the fixed-path files (the menu texts, the fonts and the UI images'
//! side files): a finding about a file as a whole, and the codes their readers raise.

use super::{Diagnostic, DiagnosticCode, Phase, Severity, VanillaText};
use crate::pack_identity::{PackPath, PackRef};

/// A finding about the file `path` of `pack` as a whole.
pub fn file_finding(
    code: DiagnosticCode,
    severity: Severity,
    pack: &PackRef,
    path: &PackPath,
) -> Diagnostic {
    let mut finding = Diagnostic::error(code)
        .severity(severity)
        .phase(Phase::Validate);
    finding.location.pack = Some(pack.clone());
    finding.location.path = Some(path.clone());
    finding
}

/// `InvalidValue` without a vanilla text (a finding of the crate's own).
pub fn invalid() -> DiagnosticCode {
    DiagnosticCode::InvalidValue { text: None }
}

/// `InvalidValue` carrying vanilla's text.
pub fn invalid_text(text: &'static str) -> DiagnosticCode {
    DiagnosticCode::InvalidValue {
        text: Some(VanillaText::Fixed(text)),
    }
}

/// A file that is not JSON: `MalformedInput` with the reader's message and, where vanilla
/// says something, its text.
pub fn malformed(message: String, text: Option<&'static str>) -> DiagnosticCode {
    DiagnosticCode::MalformedInput {
        message,
        text: text.map(VanillaText::Fixed),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::{JsonPath, Location};

    /// A pin of `file_finding`, built by hand the long way (`Diagnostic::new`): the helper is
    /// written over the builder, and must give exactly this.
    #[test]
    fn the_fixed_path_file_findings() {
        let pack = PackRef::app_data();
        let path = PackPath::from_static("font/glyph_E0.json");
        let code = || DiagnosticCode::InvalidValue { text: None };
        for severity in [Severity::Info, Severity::Warning, Severity::Error] {
            let found = file_finding(code(), severity, &pack, &path);
            let long = Diagnostic::new(
                code(),
                severity,
                Phase::Validate,
                Location::new(
                    Some(pack.clone()),
                    Some(path.clone()),
                    JsonPath::new(),
                    None,
                ),
            );
            assert_eq!(found, long);
            assert_eq!(found.kept, long.kept, "kept");
            assert_eq!(found.detail, long.detail, "detail");
        }
    }
}
