//! Unit tests of the dimension family's chain.

use crate::history::DimensionFamily;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::versioning::check_chain;

#[test]
fn the_chain_is_v1_18_0_then_v1_26_50_to_the_engine() {
    assert_eq!(check_chain::<DimensionFamily>(), Ok(()));
}

#[test]
fn generator_type_converts_to_its_id() {
    use crate::GeneratorType;
    for generator in [
        GeneratorType::Legacy,
        GeneratorType::Overworld,
        GeneratorType::Flat,
        GeneratorType::Nether,
        GeneratorType::TheEnd,
        GeneratorType::Void,
        GeneratorType::Undefined,
    ] {
        assert_eq!(u8::from(generator), generator.id(), "{generator:?}");
    }
    assert_eq!(u8::from(GeneratorType::Undefined), 6);
}

/// The shadowed-definition note, built by hand the long way (`Diagnostic::new`, `Origin::new`):
/// the helper is written over core's builder, and must give exactly this.
#[test]
fn the_shadowed_definition_note() {
    use bedrock_addon_core::diagnostic::{Code, Diagnostic, JsonPath, Location, Phase, Severity};
    use bedrock_addon_core::document::Origin;
    use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
    let pack = PackRef::app_data();
    let path = PackPath::from_static("font/glyph_E0.json");
    let origin = Origin::new(
        pack.clone(),
        PackSide::Behavior,
        path.clone(),
        None,
        Vec::new(),
    );
    let found = crate::shadowed(&origin);
    let long = Diagnostic::new(
        Code::from(&core_codes::SHADOWED_DEFINITION),
        Severity::Info,
        Phase::Validate,
        Location::new(Some(pack), Some(path), JsonPath::new(), None),
    );
    assert_eq!(found, long);
    assert_eq!(found.kept, long.kept, "kept");
    assert_eq!(found.detail, long.detail, "detail");
}
