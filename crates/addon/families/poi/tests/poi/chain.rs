//! The two chains:
//! contiguity, struct selection by the `ClosedToEngine` model, the floor and the engine ceiling.

use bedrock_addon_core::version::{ENGINE_VERSION, FormatVersion, FormatVersionInput};
use bedrock_addon_core::versioning::{
    EnvelopeCheck, Family, MissingVersion, RangeModel, Selection, VersionChain, Versioned,
    check_chain, select,
};
use bedrock_addon_family_poi::history::{
    PoiTagFamily, PoiTagHistory, PoiTypeFamily, PoiTypeHistory,
};
use bedrock_addon_family_poi::{PoiTag, PoiType};

/// A version input.
fn version(text: &str) -> FormatVersionInput {
    FormatVersionInput::parse(text).expect("a version")
}

/// One struct from 1.26.40 to the engine, nothing below, nothing above.
fn one_struct_from_1_26_40_to_the_engine<F: Family>() {
    assert_eq!(F::RANGE_MODEL, RangeModel::ClosedToEngine);
    assert_eq!(F::FLOOR, FormatVersion::new(1, 26, 40));
    assert_eq!(
        <F::History as VersionChain<F>>::VERSIONS,
        [FormatVersion::new(1, 26, 40)]
    );
    assert!(
        !<F::History as VersionChain<F>>::HAS_BETA,
        "Latest = Beta in vanilla: no beta struct"
    );
    assert_eq!(select::<F>(&version("1.26.40")), Selection::Struct(0));
    assert_eq!(select::<F>(&version("1.26.45")), Selection::Struct(0));
    assert_eq!(
        select::<F>(&version(&ENGINE_VERSION.to_string())),
        Selection::Struct(0)
    );
    assert_eq!(select::<F>(&version("1.26.39")), Selection::BelowFloor);
    assert_eq!(select::<F>(&version("1.21.130")), Selection::BelowFloor);
    assert_eq!(
        select::<F>(&version("1.26.40-beta.1")),
        Selection::BelowFloor
    );
    let above = format!(
        "{}.{}.{}",
        ENGINE_VERSION.major,
        ENGINE_VERSION.minor,
        ENGINE_VERSION.patch + 1
    );
    assert_eq!(select::<F>(&version(&above)), Selection::Unsupported);
}

#[test]
fn both_chains_are_contiguous() {
    check_chain::<PoiTypeFamily>().unwrap();
    check_chain::<PoiTagFamily>().unwrap();
}

#[test]
fn both_families_select_their_one_struct_from_1_26_40_to_the_engine() {
    one_struct_from_1_26_40_to_the_engine::<PoiTypeFamily>();
    one_struct_from_1_26_40_to_the_engine::<PoiTagFamily>();
    assert_eq!(
        <PoiType as Versioned>::VERSION,
        FormatVersion::new(1, 26, 40)
    );
    assert_eq!(
        <PoiTag as Versioned>::VERSION,
        FormatVersion::new(1, 26, 40)
    );
}

#[test]
fn the_payload_keys_and_the_envelope_are_the_modern_documents() {
    assert_eq!(PoiTypeFamily::PAYLOAD_KEY, "minecraft:poi_block");
    assert_eq!(PoiTagFamily::PAYLOAD_KEY, "minecraft:poi_tag");
    for (envelope, missing) in [
        (
            PoiTypeFamily::ENVELOPE.check,
            PoiTypeFamily::ENVELOPE.missing,
        ),
        (PoiTagFamily::ENVELOPE.check, PoiTagFamily::ENVELOPE.missing),
    ] {
        assert_eq!(envelope, EnvelopeCheck::Always, "exactly two root members");
        assert!(
            matches!(missing, MissingVersion::Reject { .. }),
            "no version, no load"
        );
    }
    assert!(PoiTypeFamily::components().0.is_empty());
    assert!(PoiTagFamily::components().0.is_empty());
}

#[test]
fn types_are_read_from_poi_blocks_and_tags_from_poi_tags() {
    // `PoiTypeLoader::loadResources`, `PoiTagLoader::loadResources`.
    assert_eq!(
        bedrock_addon_family_poi::LOADER.walk.directories,
        ["poi/blocks", "poi/tags"]
    );
}

#[test]
fn the_chain_enums_hold_the_newest_struct() {
    use bedrock_addon_core::id::{PoiTagId, PoiTypeId};
    use bedrock_addon_core::versioning::Single;
    let forge = PoiType::new(PoiTypeId::from_static("test:forge"), Vec::new());
    let chain: PoiTypeHistory = Single(forge.clone());
    let Single(inner) = chain;
    assert_eq!(inner, forge);
    let civic = PoiTag::new(PoiTagId::from_static("test:civic"));
    let chain: PoiTagHistory = Single(civic.clone());
    let Single(inner) = chain;
    assert_eq!(inner, civic);
}
