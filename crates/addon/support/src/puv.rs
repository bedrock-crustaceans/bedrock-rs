//! Findings the versioned loaders of feature rules and dimensions give alike: the below-floor text
//! and the shadowed-definition note.

use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, Phase, VanillaText};
use bedrock_addon_core::document::Origin;
use bedrock_addon_core::version::FormatVersion;

/// The versioned JSON loaders of feature rules and dimensions have no floor text: below the
/// first slice the version selects no parser, as above the engine. Gives core's below-floor
/// finding the game's "no parser available" text.
pub fn puv_below_floor(diagnostics: &mut Diagnostics, written: Option<FormatVersion>) {
    let found = std::mem::take(diagnostics);
    for mut finding in found {
        if let DiagnosticCode::UnsupportedVersion {
            text: text @ Some(VanillaText::UseFormatVersion { .. }),
        } = &mut finding.code
        {
            *text = Some(VanillaText::NoParserForVersion {
                version: written.map(|v| v.to_string()).unwrap_or_default(),
            });
        }
        diagnostics.push(finding);
    }
}

/// `ShadowedDefinition` (info) at a dropped document.
pub fn shadowed(origin: &Origin) -> Diagnostic {
    Diagnostic::info(DiagnosticCode::ShadowedDefinition { text: None })
        .phase(Phase::Validate)
        .in_file(origin)
}
