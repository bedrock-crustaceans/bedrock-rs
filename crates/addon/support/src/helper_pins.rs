//! Golden values of the support crate's finding helpers, each built by hand the long way
//! (`Diagnostic::new`, `Origin::new`): the helpers are written over core's builder, and must
//! give exactly these.

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, JsonPath, Location, Phase, Severity,
};
use bedrock_addon_core::document::Origin;
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};

#[test]
fn the_shadowed_definition_note() {
    let pack = PackRef::app_data();
    let path = PackPath::from_static("font/glyph_E0.json");
    let origin = Origin::new(
        pack.clone(),
        PackSide::Behavior,
        path.clone(),
        None,
        Vec::new(),
    );
    let found = crate::puv::shadowed(&origin);
    let long = Diagnostic::new(
        DiagnosticCode::ShadowedDefinition { text: None },
        Severity::Info,
        Phase::Validate,
        Location::new(Some(pack), Some(path), JsonPath::new(), None),
    );
    assert_eq!(found, long);
    assert_eq!(found.kept, long.kept, "kept");
    assert_eq!(found.detail, long.detail, "detail");
}
