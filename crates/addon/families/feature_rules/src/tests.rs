//! Unit tests of the feature-rule family's chain.

use super::FeatureRuleFamily;
use bedrock_addon_core::versioning::check_chain;

#[test]
fn the_chain_is_one_struct_from_the_floor_to_the_engine() {
    assert_eq!(check_chain::<FeatureRuleFamily>(), Ok(()));
}

#[test]
fn the_enum_discriminants_are_vanilla_s_values() {
    use super::{CoordinateEvalOrder, RandomDistribution};
    // The discriminants of the distribution types (the single-valued type is 0 and has no JSON
    // name) and of the coordinate evaluation orders are the numbers the game uses.
    assert_eq!(
        [
            RandomDistribution::Uniform as u8,
            RandomDistribution::Gaussian as u8,
            RandomDistribution::InverseGaussian as u8,
            RandomDistribution::FixedGrid as u8,
            RandomDistribution::JitteredGrid as u8,
            RandomDistribution::Triangle as u8,
        ],
        [1, 2, 3, 4, 5, 6]
    );
    assert_eq!(
        [
            CoordinateEvalOrder::Xyz as u8,
            CoordinateEvalOrder::Xzy as u8,
            CoordinateEvalOrder::Yxz as u8,
            CoordinateEvalOrder::Yzx as u8,
            CoordinateEvalOrder::Zxy as u8,
            CoordinateEvalOrder::Zyx as u8,
        ],
        [0, 1, 2, 3, 4, 5]
    );
}

#[test]
fn the_family_reads_a_minecraft_document_input_under_the_unstrict_mask() {
    use bedrock_addon_core::json::NullRule;
    use bedrock_addon_core::load::DocumentSource;
    use bedrock_addon_core::versioning::{Family, InputKind, StructIndex, UnknownMask};
    // The game reads the rule as JSON input with `null` members rejected, and registers the one
    // slice with the unstrict error mask, so an unknown member only warns.
    assert_eq!(FeatureRuleFamily::ENVELOPE.input, InputKind::Document);
    let slice = StructIndex::Stable(0);
    assert_eq!(
        FeatureRuleFamily::reading(slice, DocumentSource::Json).null_rule(),
        Some(NullRule::Rejected)
    );
    assert_eq!(
        FeatureRuleFamily::reading(slice, DocumentSource::Json).unknown,
        UnknownMask::Warn
    );
}
