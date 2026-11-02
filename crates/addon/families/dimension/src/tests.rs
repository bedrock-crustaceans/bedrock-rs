//! Unit tests of the dimension family's chain.

use crate::history::DimensionFamily;
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
