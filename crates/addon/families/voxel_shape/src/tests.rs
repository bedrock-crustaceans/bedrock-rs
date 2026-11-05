//! Unit tests of the crate-private parts: the chain and the vanilla texts.

use super::load::VoxelShapeFamily;
use super::{FLOOR, texts};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    EnvelopeCheck, Family, MissingVersion, RangeModel, VersionChain, check_chain,
};

#[test]
fn the_chain_is_one_struct_from_1_21_110_to_the_engine() {
    assert_eq!(check_chain::<VoxelShapeFamily>(), Ok(()));
    assert_eq!(FLOOR, FormatVersion::new(1, 21, 110));
    assert_eq!(
        <<VoxelShapeFamily as Family>::Any as VersionChain<VoxelShapeFamily>>::VERSIONS,
        [FLOOR]
    );
    let has_beta = <<VoxelShapeFamily as Family>::Any as VersionChain<VoxelShapeFamily>>::HAS_BETA;
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
            texts::failed_to_load("shapes/a.json").to_string(),
            "Failed to load voxel shape file shapes/a.json",
        ),
        (
            texts::namespace_required("a:b").to_string(),
            "The `minecraft` namespace is required for Vanilla shape: `a:b`",
        ),
        (
            texts::namespace_not_valid("minecraft:b").to_string(),
            "The `minecraft` namespace is not valid for custom voxel shape: `minecraft:b`",
        ),
        (
            texts::box_invalid(3, "a:b").to_string(),
            "Box at index 3 is invalid for voxel shape `a:b`",
        ),
        (
            texts::box_out_of_bounds(31, "a:b").to_string(),
            "Box at index 31 in voxel shape 'a:b' is outside the allowed bounds of (-14, -14, -14) to (30, 30, 30).",
        ),
        (
            texts::no_overlap("a:b").to_string(),
            "Voxel shape 'a:b' has no boxes that overlap the base block (0-16 pixels on all axes).",
        ),
        (
            texts::extent_too_wide("a:b", "z", 44.0).to_string(),
            "Total extent of voxel shape 'a:b' on axis z is 44 pixels, which exceeds the maximum of 30 pixels (1 block + 14 pixels).",
        ),
        (
            texts::not_in_schema().to_string(),
            "this member was found in the input, but is not present in the Schema",
        ),
        (texts::expected_x_y_z().to_string(), "Expected [x, y, z]."),
        (
            bedrock_addon_core::diagnostic::cereal::expected_object().to_string(),
            "expected an object",
        ),
        (
            texts::too_many_boxes().to_string(),
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
        let text = texts::extent_too_wide("a", "x", extent).to_string();
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
    use bedrock_addon_core::versioning::{ErrorMask, Family, InputKind, StructIndex};
    // The game reads a shape through the document-input reader, and the one slice is registered
    // with the strict error mask.
    let slice = StructIndex::Stable(0);
    assert_eq!(VoxelShapeFamily::ENVELOPE.input, InputKind::Document);
    assert_eq!(
        VoxelShapeFamily::reading(slice, DocumentSource::Json).null_rule(),
        Some(NullRule::Rejected)
    );
    assert_eq!(
        VoxelShapeFamily::reading(slice, DocumentSource::Json).error_mask(),
        ErrorMask::Strict
    );
}
