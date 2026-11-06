//! The finding builders of the fixed-path files (the menu texts, the fonts and the UI images'
//! side files): a finding about a file as a whole, and the codes their readers raise.

use super::{Code, Diagnostic, DiagnosticCode, Phase, Severity, VanillaText, codes};
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
    DiagnosticCode::from(&codes::INVALID_VALUE)
}

/// `InvalidValue` carrying vanilla's text.
pub fn invalid_text(text: &'static str) -> DiagnosticCode {
    codes::INVALID_VALUE_WORDED.with([text]).into()
}

/// The code of a syntax finding whose line is the reader's message: [`codes::SYNTAX`] with the
/// message, or [`codes::MALFORMED_INPUT`] ("malformed input") when the message is empty, as
/// the legacy `MalformedInput` without a text renders them.
pub fn syntax(message: impl Into<String>) -> Code {
    let message = message.into();
    if message.is_empty() {
        Code::from(&codes::MALFORMED_INPUT)
    } else {
        codes::SYNTAX.with([message])
    }
}

/// A file that is not JSON: `MalformedInput` with the reader's message and, where vanilla
/// says something, its text.
pub fn malformed(message: String, text: Option<&'static str>) -> DiagnosticCode {
    match text {
        // The family's sentence with the reader's message beside it: the legacy code holds
        // both, until the family's own definition carries the message as its finding's detail.
        Some(text) => DiagnosticCode::MalformedInput {
            message,
            text: Some(VanillaText::Fixed(text)),
        },
        None => syntax(message).into(),
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
        let code = || DiagnosticCode::from(&codes::INVALID_VALUE);
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
