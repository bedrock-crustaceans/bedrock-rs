//! The five chains: contiguity, struct selection by the
//! `ClosedToEngine` model, the floors, the engine ceiling and the missing beta struct.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::families::jigsaw::JigsawStructure;
use bedrock_addon::families::jigsaw::history::{
    MetadataFamily, PoolFamily, ProcessorListFamily, StructureFamily, StructureSetFamily,
};
use bedrock_addon::load::{LoadContext, LoadOptions};
use bedrock_addon::version::{ENGINE_VERSION, FormatVersion, FormatVersionInput};
use bedrock_addon::versioning::{Family, RangeModel, Selection, check_chain, select};

use super::common::{document, rejected};

/// A version input.
fn version(text: &str) -> FormatVersionInput {
    FormatVersionInput::parse(text).expect("a version")
}

/// The version one minor above the engine's.
fn above_engine() -> String {
    format!("{}.{}.0", ENGINE_VERSION.major, ENGINE_VERSION.minor + 10)
}

#[test]
fn every_chain_is_contiguous() {
    check_chain::<StructureFamily>().unwrap();
    check_chain::<PoolFamily>().unwrap();
    check_chain::<StructureSetFamily>().unwrap();
    check_chain::<ProcessorListFamily>().unwrap();
    check_chain::<MetadataFamily>().unwrap();
}

/// The four JSON families: one struct from 1.21.20 to the engine, nothing below.
fn json_family_selection<F: Family>() {
    assert_eq!(F::RANGE_MODEL, RangeModel::ClosedToEngine);
    assert_eq!(F::FLOOR, FormatVersion::new(1, 21, 20));
    assert_eq!(select::<F>(&version("1.21.20")), Selection::Struct(0));
    assert_eq!(select::<F>(&version("1.26.20")), Selection::Struct(0));
    assert_eq!(
        select::<F>(&version(&ENGINE_VERSION.to_string())),
        Selection::Struct(0)
    );
    assert_eq!(select::<F>(&version("1.21.19")), Selection::BelowFloor);
    assert_eq!(
        select::<F>(&version("1.21.20-beta.1")),
        Selection::BelowFloor
    );
    assert_eq!(
        select::<F>(&version(&above_engine())),
        Selection::Unsupported
    );
}

#[test]
fn json_families_select_their_one_struct_from_1_21_20_to_the_engine() {
    json_family_selection::<StructureFamily>();
    json_family_selection::<PoolFamily>();
    json_family_selection::<StructureSetFamily>();
    json_family_selection::<ProcessorListFamily>();
}

#[test]
fn metadata_selects_v1_21_50_then_v1_21_80() {
    assert_eq!(
        MetadataFamily::FLOOR,
        FormatVersion::new(1, 21, 50),
        "SCHEMA_VERSION_1"
    );
    assert_eq!(
        select::<MetadataFamily>(&version("1.21.49")),
        Selection::BelowFloor
    );
    assert_eq!(
        select::<MetadataFamily>(&version("1.21.50")),
        Selection::Struct(0)
    );
    assert_eq!(
        select::<MetadataFamily>(&version("1.21.79")),
        Selection::Struct(0)
    );
    assert_eq!(
        select::<MetadataFamily>(&version("1.21.80")),
        Selection::Struct(1)
    );
    assert_eq!(
        select::<MetadataFamily>(&version(&ENGINE_VERSION.to_string())),
        Selection::Struct(1)
    );
    assert_eq!(
        select::<MetadataFamily>(&version(&above_engine())),
        Selection::Unsupported
    );
}

#[test]
fn a_beta_document_has_no_struct() {
    let payload = r#"{"description": {"identifier": "demo:a"}, "step": "lakes", "start_pool": "demo:p", "max_depth": 1, "start_height": {"type": "constant", "value": {"absolute": 0}}}"#;
    let d = rejected::<JigsawStructure>(
        "worldgen/structures/a.json",
        &document("beta", "minecraft:jigsaw", payload),
    );
    assert!(d.iter().any(|f| f.severity == Severity::Error), "{d:?}");
    // With the Beta APIs toggle on, vanilla still has no beta slice: no parser.
    let options = LoadOptions {
        beta_apis: true,
        ..LoadOptions::default()
    };
    let cx = LoadContext::for_pack(
        &options,
        uuid::Uuid::from_u128(7),
        bedrock_addon::pack::PackType::Behavior,
        None,
        false,
    );
    let outcome = <JigsawStructure as bedrock_addon::document::Content>::parse(
        &bedrock_addon::storage::PackPath::new("worldgen/structures/a.json").unwrap(),
        document("beta", "minecraft:jigsaw", payload).as_bytes(),
        &cx,
    );
    let d = outcome.expect_err("no beta struct").diagnostics;
    assert!(
        d.iter()
            .any(|f| f.code.id() == "core/no_parser_for_version"),
        "{d:?}"
    );
}
