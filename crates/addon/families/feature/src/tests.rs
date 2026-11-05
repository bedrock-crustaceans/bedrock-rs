//! Unit tests of the feature module's tables: the payload keys, the chains and their ranges.

use super::load::{Fam, FeatureType, KEYS, TypeFn, with_type};
use super::{
    MultiBlockFeature, OreFeature, ScatterFeature, SingleBlockFeature, SnapToSurfaceFeature,
};
use bedrock_addon_core::json::NullRule;
use bedrock_addon_core::load::DocumentSource;
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput, known};
use bedrock_addon_core::versioning::{
    Family, InputKind, Selection, StructIndex, VersionChain, check_chain, select,
};

/// The payload key a type dispatches to.
struct Key;

impl TypeFn<&'static str> for Key {
    fn call<T: FeatureType>(self) -> &'static str {
        T::KEY
    }
}

#[test]
fn every_registered_key_dispatches_to_its_own_type() {
    // The 25 keys of engine 1.26.20, the three added by engine 1.26.45 and
    // multipart_block_column.
    assert_eq!(KEYS.len(), 29);
    for key in KEYS {
        assert_eq!(with_type(key, Key), Some(*key));
    }
    assert_eq!(with_type("minecraft:nope_feature", Key), None);
}

/// Checks the chain of a type and returns its versions.
struct Chain;

impl TypeFn<Vec<FormatVersion>> for Chain {
    fn call<T: FeatureType>(self) -> Vec<FormatVersion> {
        check_chain::<Fam<T>>().unwrap_or_else(|e| panic!("{}: {e}", T::KEY));
        <T::Chain as VersionChain<Fam<T>>>::VERSIONS.to_vec()
    }
}

/// Whether a type's chain has a beta struct.
struct HasBeta;

impl TypeFn<bool> for HasBeta {
    fn call<T: FeatureType>(self) -> bool {
        <T::Chain as VersionChain<Fam<T>>>::HAS_BETA
    }
}

/// The struct a version selects in the chain of `T`.
fn selects<T: FeatureType>(major: u16, minor: u16, patch: u16) -> Selection {
    select::<Fam<T>>(&FormatVersionInput {
        version: Some(FormatVersion::new(major, minor, patch)),
        beta_literal: false,
        prerelease: false,
    })
}

#[test]
fn every_chain_is_contiguous_and_ends_at_its_latest_struct() {
    let v = FormatVersion::new;
    for key in KEYS {
        let versions = with_type(key, Chain).unwrap();
        let expected = match *key {
            "minecraft:scatter_feature" => vec![v(1, 13, 0), v(1, 21, 10)],
            "minecraft:single_block_feature" => vec![v(1, 13, 0), v(1, 21, 40)],
            "minecraft:snap_to_surface_feature" => vec![v(1, 13, 0), v(1, 26, 50)],
            "minecraft:multi_block_feature" | "minecraft:multipart_block_column_feature" => {
                vec![v(1, 26, 40)]
            }
            _ => vec![v(1, 13, 0)],
        };
        assert_eq!(versions, expected, "{key}");
        assert_eq!(with_type(key, HasBeta), Some(false), "{key}");
    }
}

#[test]
fn versions_select_by_the_closed_to_engine_model() {
    assert_eq!(selects::<ScatterFeature>(1, 13, 0), Selection::Struct(0));
    assert_eq!(selects::<ScatterFeature>(1, 21, 0), Selection::Struct(0));
    assert_eq!(selects::<ScatterFeature>(1, 21, 10), Selection::Struct(1));
    assert_eq!(
        selects::<SingleBlockFeature>(1, 21, 30),
        Selection::Struct(0)
    );
    assert_eq!(
        selects::<SingleBlockFeature>(1, 21, 40),
        Selection::Struct(1)
    );
    assert_eq!(
        selects::<SnapToSurfaceFeature>(1, 26, 40),
        Selection::Struct(0)
    );
    assert_eq!(
        selects::<SnapToSurfaceFeature>(1, 26, 50),
        Selection::Struct(1)
    );
    // The floor and the engine ceiling.
    assert_eq!(selects::<OreFeature>(1, 12, 0), Selection::BelowFloor);
    assert_eq!(selects::<OreFeature>(1, 26, 50), Selection::Struct(0));
    assert_eq!(selects::<OreFeature>(1, 26, 60), Selection::Struct(0));
    assert_eq!(selects::<OreFeature>(1, 26, 61), Selection::Unsupported);
    assert_eq!(
        selects::<MultiBlockFeature>(1, 26, 30),
        Selection::BelowFloor
    );
    assert_eq!(
        selects::<MultiBlockFeature>(1, 26, 40),
        Selection::Struct(0)
    );
}

/// The duplicate findings of one pack holding two definitions of `demo:vein`, on an engine of
/// `engine`.
fn same_pack_duplicates(
    engine: FormatVersion,
) -> Vec<(
    bedrock_addon_core::diagnostic::DiagnosticCode,
    bedrock_addon_core::diagnostic::Severity,
)> {
    use bedrock_addon_core::document::Contribution;
    use bedrock_addon_core::load::{LoadContext, LoadOptions, StackCx, StackPack};
    use bedrock_addon_core::pack_identity::{
        PackIdVersion, PackOrigin, PackPath, PackRef, PackSide, PackType,
    };
    use bedrock_addon_core::version::SemVersion;

    let id = PackIdVersion::new(
        uuid::Uuid::from_u128(1),
        SemVersion::new(1, 0, 0),
        PackType::Behavior,
    );
    let pack = PackRef::new(0, id, "pack");
    let mut cx = LoadContext::detached();
    cx.engine_version = engine;
    let file = |path: &'static str, count: u8| {
        let json = format!(
            r#"{{"format_version": "1.13.0", "minecraft:ore_feature": {{"description": {{"identifier": "demo:vein"}}, "count": {count}}}}}"#
        );
        match super::load::parse_file(
            pack.clone(),
            PackSide::Behavior,
            &PackPath::from_static(path),
            json.as_bytes(),
            &cx,
            super::load::FileSource::Pack,
        ) {
            (Some(id), Ok(document)) => Contribution::Stable(id, document),
            _ => panic!("the file loads"),
        }
    };
    let options = LoadOptions::default();
    let packs = [StackPack::new(pack.clone(), PackOrigin::World, cx.clone())];
    let stack = StackCx::new(&options, &packs);
    let registry = (super::load::LOADER.combine)(
        vec![file("features/a.json", 2), file("features/b.json", 3)],
        &stack,
    );
    registry
        .diagnostics()
        .map(|f| (f.code.clone(), f.severity))
        .collect()
}

#[test]
fn the_duplicate_text_arrives_with_engine_1_26_40() {
    use bedrock_addon_core::diagnostic::{DiagnosticCode, Severity};
    // Silent before 1.26.40 (the crate's own information, without text); from 1.26.40 the
    // game's level-3 text, and the second document is refused: an error.
    let before = same_pack_duplicates(known::V1_26_30);
    assert!(
        matches!(
            &before[..],
            [(
                DiagnosticCode::DuplicateIdentifier { text: None },
                Severity::Info
            )]
        ),
        "{before:?}"
    );
    let from = same_pack_duplicates(known::V1_26_40);
    assert!(
        matches!(
            &from[..],
            [(
                DiagnosticCode::DuplicateIdentifier { text: Some(_) },
                Severity::Error
            )]
        ),
        "{from:?}"
    );
}

#[test]
fn a_feature_is_small_enough_to_be_moved_through_the_load() {
    // An unoptimised build keeps a copy of a `Feature` in each frame that moves one, some
    // fifty in the combine step alone; held inline a tree feature made it 17 296 bytes.
    let size = size_of::<super::Feature>();
    assert!(size <= 8 * 1024, "a Feature is {size} bytes");
}

/// The input kind and null rule a type's family declares.
struct Input;

impl TypeFn<(InputKind, Option<NullRule>)> for Input {
    fn call<T: FeatureType>(self) -> (InputKind, Option<NullRule>) {
        let rule =
            <Fam<T> as Family>::reading(StructIndex::Stable(0), DocumentSource::Json).null_rule();
        (<Fam<T> as Family>::ENVELOPE.input, rule)
    }
}

#[test]
fn every_feature_type_is_read_from_a_jsoncpp_input() {
    // The client reads a feature through a jsoncpp input: rule N1, a member written as `null`
    // is absent.
    for key in KEYS {
        assert_eq!(
            with_type(key, Input),
            Some((InputKind::JsonCpp, Some(NullRule::Absent))),
            "{key}"
        );
    }
}

/// Every finding of `found` is in the app-data pack's file `path`.
fn assert_located(found: &bedrock_addon_core::diagnostic::Diagnostics, path: &'static str) {
    use bedrock_addon_core::pack_identity::{PackPath, PackRef};
    assert!(!found.is_empty(), "no finding to place");
    for finding in found {
        assert_eq!(
            finding.location.pack,
            Some(PackRef::app_data()),
            "{finding:?}"
        );
        assert_eq!(
            finding.location.path,
            Some(PackPath::from_static(path)),
            "{finding:?}"
        );
    }
}

#[test]
fn findings_carry_the_file_and_pack() {
    use super::Feature;
    use bedrock_addon_core::document::Content;
    use bedrock_addon_core::load::LoadContext;
    use bedrock_addon_core::pack_identity::PackPath;
    const PATH: &str = "features/x.json";
    let cx = LoadContext::detached();
    let path = PackPath::from_static(PATH);
    let ore = |member: &str| {
        format!(
            r#"{{"format_version": "1.13.0", "minecraft:ore_feature": {{"description": {{"identifier": "demo:vein"}}, "count": 2{member}}}}}"#
        )
    };
    // A loaded file with a finding (a member written as null).
    let document = Feature::parse(
        &path,
        ore(r#", "discard_chance_on_air_exposure": null"#).as_bytes(),
        &cx,
    )
    .expect("the file loads");
    assert_located(document.diagnostics(), PATH);
    // Refused by its schema (an unknown member), and a file that is no JSON.
    for bytes in [ore(r#", "x": 1"#).into_bytes(), b"{".to_vec()] {
        let rejected = Feature::parse(&path, &bytes, &cx)
            .map(|_| ())
            .expect_err("the file is refused");
        assert_located(&rejected.diagnostics, PATH);
    }
}
