//! Unit tests of the crate-private parts: the chain and the vanilla texts.

use super::VoxelShapeFamily;
use super::validate::extent_too_wide;
use super::{FLOOR, codes};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    EnvelopeCheck, Family, MissingVersion, RangeModel, VersionChain, check_chain,
};

#[test]
fn the_chain_is_one_struct_from_1_21_110_to_the_engine() {
    assert_eq!(check_chain::<VoxelShapeFamily>(), Ok(()));
    assert_eq!(FLOOR, FormatVersion::new(1, 21, 110));
    assert_eq!(
        <<VoxelShapeFamily as Family>::History as VersionChain<VoxelShapeFamily>>::VERSIONS,
        [FLOOR]
    );
    let has_beta =
        <<VoxelShapeFamily as Family>::History as VersionChain<VoxelShapeFamily>>::HAS_BETA;
    assert!(!has_beta);
    assert_eq!(VoxelShapeFamily::FLOOR, FLOOR);
    assert_eq!(VoxelShapeFamily::RANGE_MODEL, RangeModel::ClosedToEngine);
    assert_eq!(VoxelShapeFamily::PAYLOAD_KEY, "minecraft:voxel_shape");
    assert_eq!(super::LOADER.walk.directories, ["shapes"]);
    assert!(matches!(
        VoxelShapeFamily::ENVELOPE.missing,
        MissingVersion::Reject { .. }
    ));
    assert_eq!(VoxelShapeFamily::ENVELOPE.check, EnvelopeCheck::From(FLOOR));
}

#[test]
fn every_text_is_vanilla_s_with_its_arguments_filled_in() {
    // One argument per placeholder: a missing one would leave a gap, a spare one is dropped,
    // and either shows here.
    let texts = [
        (
            codes::FAILED_TO_LOAD.with(["shapes/a.json"]).to_string(),
            "Failed to load voxel shape file shapes/a.json",
        ),
        (
            codes::NAMESPACE_REQUIRED.with(["a:b"]).to_string(),
            "The `minecraft` namespace is required for Vanilla shape: `a:b`",
        ),
        (
            codes::NAMESPACE_NOT_VALID.with(["minecraft:b"]).to_string(),
            "The `minecraft` namespace is not valid for custom voxel shape: `minecraft:b`",
        ),
        (
            codes::BOX_INVALID.with(["3", "a:b"]).to_string(),
            "Box at index 3 is invalid for voxel shape `a:b`",
        ),
        (
            codes::BOX_OUT_OF_BOUNDS.with(["31", "a:b"]).to_string(),
            "Box at index 31 in voxel shape 'a:b' is outside the allowed bounds of (-14, -14, -14) to (30, 30, 30).",
        ),
        (
            codes::NO_OVERLAP.with(["a:b"]).to_string(),
            "Voxel shape 'a:b' has no boxes that overlap the base block (0-16 pixels on all axes).",
        ),
        (
            extent_too_wide("a:b", "z", 44.0).to_string(),
            "Total extent of voxel shape 'a:b' on axis z is 44 pixels, which exceeds the maximum of 30 pixels (1 block + 14 pixels).",
        ),
        (
            bedrock_addon_core::diagnostic::cereal::member_not_in_schema().to_string(),
            "this member was found in the input, but is not present in the Schema",
        ),
        (
            codes::EXPECTED_X_Y_Z.template().to_owned(),
            "Expected [x, y, z].",
        ),
        (
            bedrock_addon_core::diagnostic::cereal::expected_object().to_string(),
            "expected an object",
        ),
        (
            codes::TOO_MANY_BOXES.template().to_owned(),
            " -> shape -> boxes: too many input elements, expected no more than 32",
        ),
    ];
    for (rendered, vanilla) in texts {
        assert_eq!(rendered, vanilla);
    }
}

#[test]
fn an_extent_is_printed_with_six_significant_digits() {
    let printed = |extent: f32| {
        let text = extent_too_wide("a", "x", extent).to_string();
        let (_, rest) = text.split_once(" is ").unwrap();
        rest.split_once(" pixels").unwrap().0.to_owned()
    };
    assert_eq!(printed(32.0), "32");
    assert_eq!(printed(40.0), "40");
    assert_eq!(printed(30.5), "30.5");
    assert_eq!(printed(30.25), "30.25");
    assert_eq!(printed(30.000_1), "30.0001");
    assert_eq!(printed(30.000_04), "30");
    assert_eq!(printed(43.999_96), "44");
    assert_eq!(printed(31.415_927), "31.4159");
}

#[test]
fn the_family_reads_a_minecraft_document_input_under_the_strict_mask() {
    use bedrock_addon_core::json::NullRule;
    use bedrock_addon_core::load::DocumentSource;
    use bedrock_addon_core::versioning::{Family, InputKind, StructIndex, UnknownMask};
    // The game reads a shape through the document-input reader, and the one slice is registered
    // with the strict error mask.
    let slice = StructIndex::Stable(0);
    assert_eq!(VoxelShapeFamily::ENVELOPE.input, InputKind::Document);
    assert_eq!(
        VoxelShapeFamily::reading(slice, DocumentSource::Json).null_rule(),
        Some(NullRule::Rejected)
    );
    assert_eq!(
        VoxelShapeFamily::reading(slice, DocumentSource::Json).unknown,
        UnknownMask::Error
    );
}
