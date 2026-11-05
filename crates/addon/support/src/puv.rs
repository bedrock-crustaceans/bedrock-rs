//! The shadowed-definition note the versioned loaders of feature rules and dimensions give
//! alike.

use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Phase};
use bedrock_addon_core::document::Origin;

/// `ShadowedDefinition` (info) at a dropped document.
pub fn shadowed(origin: &Origin) -> Diagnostic {
    Diagnostic::info(DiagnosticCode::ShadowedDefinition { text: None })
        .phase(Phase::Validate)
        .in_file(origin)
}
