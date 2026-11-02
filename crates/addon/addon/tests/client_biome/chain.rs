//! The chain: contiguity, the range model's selection, floor, ceiling and beta
//! and history without upgrade.

use bedrock_addon::document::Content;
use bedrock_addon::families::client_biome::ClientBiome;
use bedrock_addon::families::client_biome::history::{self, Any, ClientBiomeFamily};
use bedrock_addon::version::{ENGINE_VERSION, FormatVersion, FormatVersionInput};
use bedrock_addon::versioning::{
    EnvelopeCheck, Family, MissingVersion, RangeModel, Selection, VersionChain, check_chain, select,
};

use super::common::{base_game, beta_apis, biome, context, document, json, loads, path};

/// The chain's versions, in order.
const VERSIONS: [(u16, u16, u16); 7] = [
    (1, 21, 40),
    (1, 21, 70),
    (1, 21, 90),
    (1, 21, 100),
    (1, 21, 110),
    (1, 21, 120),
    (1, 21, 130),
];

/// What `version` selects.
fn selects(version: &str) -> Selection {
    select::<ClientBiomeFamily>(&FormatVersionInput::parse(version).expect("a version"))
}

#[test]
fn the_chain_is_contiguous_and_ends_at_1_21_130_with_a_beta_struct() {
    const HAS_BETA: bool = <Any as VersionChain<ClientBiomeFamily>>::HAS_BETA;

    check_chain::<ClientBiomeFamily>().expect("the chain is well formed");
    let versions: Vec<_> = VERSIONS
        .iter()
        .map(|&(a, b, c)| FormatVersion::new(a, b, c))
        .collect();
    assert_eq!(
        <Any as VersionChain<ClientBiomeFamily>>::VERSIONS,
        versions.as_slice()
    );
    let has_beta = HAS_BETA;
    assert!(has_beta);
    assert_eq!(
        <ClientBiomeFamily as Family>::RANGE_MODEL,
        RangeModel::ClosedToEngine
    );
    assert_eq!(
        <ClientBiomeFamily as Family>::FLOOR,
        FormatVersion::new(1, 21, 40)
    );
    assert_eq!(<ClientBiomeFamily as Family>::NAME, "client_biome");
    assert_eq!(
        <ClientBiomeFamily as Family>::PAYLOAD_KEY,
        "minecraft:client_biome"
    );
    assert_eq!(<ClientBiomeFamily as Family>::DIRECTORIES, &["biomes"]);
    assert_eq!(
        <ClientBiomeFamily as Family>::MISSING_VERSION,
        MissingVersion::Reject
    );
    assert_eq!(
        <ClientBiomeFamily as Family>::ENVELOPE,
        EnvelopeCheck::From(FormatVersion::new(1, 21, 130))
    );
    assert_eq!(<ClientBiomeFamily as Family>::SKIP_UPGRADES_KEY, None);
}

#[test]
fn each_version_selects_the_struct_whose_range_holds_it() {
    for (index, &(a, b, c)) in VERSIONS.iter().enumerate() {
        assert_eq!(selects(&format!("{a}.{b}.{c}")), Selection::Struct(index));
        if index > 0 {
            // The version just below a struct's belongs to the struct before it.
            assert_eq!(
                selects(&format!("{a}.{b}.{}", c - 1)),
                Selection::Struct(index - 1)
            );
        }
    }
    // Vanilla's 1.26.0 and 1.26.30 stamps, up to the engine, are 1.21.130 documents.
    assert_eq!(selects("1.26.0"), Selection::Struct(6));
    assert_eq!(selects(&ENGINE_VERSION.to_string()), Selection::Struct(6));
    let above = format!(
        "{}.{}.{}",
        ENGINE_VERSION.major,
        ENGINE_VERSION.minor,
        ENGINE_VERSION.patch + 1
    );
    assert_eq!(selects(&above), Selection::Unsupported);
    assert_eq!(selects("1.21.39"), Selection::BelowFloor);
    assert_eq!(selects("1.21.40-beta.1"), Selection::BelowFloor);
}

/// A version and the test of the struct it selects.
type Case = (&'static str, fn(&Any) -> bool);

/// `parse_as_written` gives the struct the version selects, which writes the document back in
/// its own version; `history::upgrade` then gives what `parse` gives.
#[test]
fn history_parses_without_upgrading_and_upgrades_on_request() {
    let cases: [Case; 7] = [
        ("1.21.40", |any| matches!(any, Any::V1_21_40(_))),
        ("1.21.70", |any| matches!(any, Any::V1_21_70(_))),
        ("1.21.90", |any| matches!(any, Any::V1_21_90(_))),
        ("1.21.100", |any| matches!(any, Any::V1_21_100(_))),
        ("1.21.110", |any| matches!(any, Any::V1_21_110(_))),
        ("1.21.120", |any| matches!(any, Any::V1_21_120(_))),
        ("1.21.130", |any| matches!(any, Any::V1_21_130(_))),
    ];
    let cx = context();
    for (version, is_variant) in cases {
        let text = biome(
            version,
            "minecraft:swampland",
            r##""minecraft:water_appearance": {"surface_color": "#617B64"},
               "minecraft:ambient_sounds": {"loop": "l"}"##,
        );
        let as_written = ClientBiome::parse_as_written(&path(), text.as_bytes(), &cx)
            .unwrap_or_else(|r| panic!("{version}: {r}"));
        assert!(is_variant(as_written.value()), "{version}");
        assert_eq!(
            as_written.value().parsed_version(),
            FormatVersion::new(1, 21, version[5..].parse().unwrap())
        );
        // Not upgraded: the document as it was written.
        assert_eq!(
            json(&as_written.value().to_json().unwrap()),
            json(&text),
            "{version}"
        );
        let upgraded = history::upgrade(as_written, &cx).unwrap();
        let parsed = loads(&text, &cx);
        assert_eq!(upgraded.value(), parsed.value(), "{version}");
        assert_eq!(upgraded.version(), parsed.version(), "{version}");
        assert_eq!(upgraded.diagnostics(), parsed.diagnostics(), "{version}");
    }
}

/// A document below 1.21.130 keeps its `format_version` as written; an unknown member of its
/// root or payload is an error of its strict legacy loader (0xfffc) and refuses it.
#[test]
fn a_legacy_struct_keeps_its_root() {
    let text = r#"{"format_version": "1.21.45", "minecraft:client_biome": {"description": {"identifier": "plains"}, "components": {}}}"#;
    let as_written = ClientBiome::parse_as_written(&path(), text.as_bytes(), &context()).unwrap();
    let Any::V1_21_40(biome) = as_written.value() else {
        panic!("a 1.21.40 struct");
    };
    assert_eq!(biome.format_version, "1.21.45");
    assert_eq!(biome.description.identifier.as_str(), "plains");
    assert_eq!(json(&as_written.value().to_json().unwrap()), json(text));
    let unknown = r#"{"format_version": "1.21.45", "minecraft:client_biome": {"description": {"identifier": "plains"}, "components": {}, "tag": 1}, "note": [1]}"#;
    assert!(ClientBiome::parse_as_written(&path(), unknown.as_bytes(), &context()).is_err());
}

#[test]
fn a_beta_document_is_the_beta_struct_and_is_not_upgraded() {
    let text = document(
        "beta",
        r#"{"description": {"identifier": "myns:glade"}, "components": {}, "environment_attributes": {}}"#,
    );
    for cx in [beta_apis(), base_game()] {
        let as_written = ClientBiome::parse_as_written(&path(), text.as_bytes(), &cx).unwrap();
        assert!(matches!(as_written.value(), Any::Beta(_)));
        assert_eq!(json(&as_written.value().to_json().unwrap()), json(&text));
        // The stable document has no parser for it.
        assert!(history::upgrade(as_written, &cx).is_err());
    }
    assert!(ClientBiome::parse_as_written(&path(), text.as_bytes(), &context()).is_err());
    assert!(ClientBiome::parse(&path(), text.as_bytes(), &beta_apis()).is_err());
}

/// The game's document input: the version is the first
/// textual `format_version`, a number read as its text: `1.26` is 1.26.0, read by the 1.21.130
/// struct.
#[test]
fn the_version_is_read_as_minecraft_document_input_reads_it() {
    let text = biome("X", "myns:glade", "").replace(r#""X""#, "1.26");
    let document = loads(&text, &context());
    assert_eq!(
        document.version().written,
        Some(FormatVersion::new(1, 26, 0))
    );
    assert_eq!(document.version().parsed_as, FormatVersion::new(1, 21, 130));
}
