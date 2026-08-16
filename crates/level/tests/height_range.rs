//! Resolves every fixture's Overworld height range through
//! `bedrock_level::height_range` and checks it against `index.json`'s
//! recorded era and observed subchunk range, then does the same for the
//! Nether and End on the fixtures that carry them. `support::fixtures()` is
//! the same harness `fixtures_test.rs` uses.

mod support;

use std::collections::HashMap;

use bedrock_level::height_range::{self, HeightRange};
use bedrock_level::metadata_dictionary::{self, MetadataDictionary};
use bedrock_level::settings::LevelSettings;
use bedrock_shared::world::dimension::Dimension;

/// The nominal (era-default-or-dictionary) Overworld range each fixture
/// should resolve to, built from the version-and-experiment-flag table the
/// module docs describe and cross-checked against every fixture's actual
/// `experiments.caves_and_cliffs` flag and `lastOpenedWithVersion` (dumped
/// from each fixture's real level.dat while this table was written): the
/// three 1.17.x "plain" worlds (`v1_17_20`, `v1_17_40`, `v1_17_40_superflat`)
/// carry the flag unset and resolve to the legacy range; the 1.17.x
/// "caves_and_cliffs" pair carries it set and resolves to the extended
/// range; every 1.18-and-later fixture resolves to the extended range
/// regardless of the flag's own on-disk value, including `v1_18_30` and
/// `v1_19_30`, which carry it unset (1.18 made the extension the default
/// and stopped needing the toggle).
fn expected_overworld_nominal() -> HashMap<&'static str, HeightRange> {
    HashMap::from([
        ("v1_12", height_range::OVERWORLD_BEFORE_CAVES_AND_CLIFFS),
        ("v1_16", height_range::OVERWORLD_BEFORE_CAVES_AND_CLIFFS),
        ("v1_17_20", height_range::OVERWORLD_BEFORE_CAVES_AND_CLIFFS),
        (
            "v1_17_20_caves_and_cliffs",
            height_range::OVERWORLD_AFTER_CAVES_AND_CLIFFS,
        ),
        ("v1_17_40", height_range::OVERWORLD_BEFORE_CAVES_AND_CLIFFS),
        (
            "v1_17_40_caves_and_cliffs",
            height_range::OVERWORLD_AFTER_CAVES_AND_CLIFFS,
        ),
        (
            "v1_17_40_superflat",
            height_range::OVERWORLD_BEFORE_CAVES_AND_CLIFFS,
        ),
        ("v1_18", height_range::OVERWORLD_AFTER_CAVES_AND_CLIFFS),
        ("v1_18_30", height_range::OVERWORLD_AFTER_CAVES_AND_CLIFFS),
        (
            "v1_18_superflat",
            height_range::OVERWORLD_AFTER_CAVES_AND_CLIFFS,
        ),
        (
            "v1_18_updated_superflat",
            height_range::OVERWORLD_AFTER_CAVES_AND_CLIFFS,
        ),
        ("v1_19_30", height_range::OVERWORLD_AFTER_CAVES_AND_CLIFFS),
        ("v1_20_81", height_range::OVERWORLD_AFTER_CAVES_AND_CLIFFS),
    ])
}

/// Fixture/dimension pairs whose *observed* subchunk range (`index.json`'s
/// `subchunk_height_range`, the range of subchunks actually found on disk)
/// reaches outside the resolved nominal range. Real gameplay cannot produce
/// this -- the engine will not generate or let a player place a block
/// outside a dimension's own bounds -- so every occurrence here is
/// specifically noted rather than silently excluded:
///
/// `v1_20_81` is a synthetically generated integration-test world, built to
/// exercise block mapping across every block the target versions know
/// about, not played through normally. Its Overworld carries one subchunk
/// (`y = -80..-64`, one section
/// below the nominal floor) and its Nether carries subchunks up to
/// `y = 256` (double the Nether's nominal ceiling) that a real save could
/// not contain. Both are one fixture's synthetic test data, not evidence
/// against the resolved nominal ranges -- confirmed independently by a
/// direct key scan of the fixture (every subchunk key's resolved absolute
/// index, not just what `index.json` records), and the End dimension in the
/// same world (nominal and observed both `0..256`) shows the exception is
/// confined to the two dimensions actually named below rather than being a
/// general property of this fixture.
fn known_observed_exceeds_nominal(fixture_name: &str, dimension: Dimension) -> bool {
    matches!(
        (fixture_name, dimension),
        ("v1_20_81", Dimension::Overworld) | ("v1_20_81", Dimension::Nether)
    )
}

fn dimension_name_key(dimension: Dimension) -> &'static str {
    match dimension {
        Dimension::Overworld => "overworld",
        Dimension::Nether => "nether",
        Dimension::End => "end",
        Dimension::Undefined | Dimension::Other(_) => {
            unreachable!("test only resolves the three vanilla dimensions")
        }
    }
}

/// Reads `fixture`'s `LevelSettings` and, if present, its
/// `LevelChunkMetaDataDictionary`, mirroring what a future `Level::open`
/// needs to gather before it can call `height_range::resolve`.
fn load_settings_and_dictionary(
    fixture: &support::Fixture,
) -> (tempfile::TempDir, LevelSettings, Option<MetadataDictionary>) {
    let (tmp, db) = fixture.open();

    let level_dat = std::fs::read(fixture.level_dat_path(&tmp))
        .unwrap_or_else(|e| panic!("{}: failed to read level.dat: {e}", fixture.name));
    let settings = LevelSettings::read(level_dat.as_slice())
        .unwrap_or_else(|e| panic!("{}: LevelSettings::read failed: {e}", fixture.name));

    let dictionary = db
        .get(metadata_dictionary::KEY)
        .unwrap_or_else(|e| panic!("{}: dictionary lookup failed: {e}", fixture.name))
        .map(|bytes| {
            MetadataDictionary::decode(bytes.as_ref())
                .unwrap_or_else(|e| panic!("{}: dictionary decode failed: {e}", fixture.name))
        });

    (tmp, settings, dictionary)
}

/// For every fixture: the resolved Overworld range matches the expected
/// era/dictionary value, and it contains the range of subchunks the corpus
/// scan actually found on disk (`index.json`'s `subchunk_height_range`,
/// which is the range of *saved* subchunks and so can be, and usually is,
/// narrower than the dimension's nominal bounds) -- except the two
/// documented cases in [`known_observed_exceeds_nominal`].
#[test]
fn every_fixture_overworld_range_matches_expected_and_contains_observed() {
    let expected = expected_overworld_nominal();

    for fixture in support::fixtures() {
        let (_tmp, settings, dictionary) = load_settings_and_dictionary(&fixture);

        let resolved = height_range::resolve(Dimension::Overworld, &settings, dictionary.as_ref())
            .unwrap_or_else(|| panic!("{}: Overworld always has a defined range", fixture.name));

        let expected_range = *expected
            .get(fixture.name.as_str())
            .unwrap_or_else(|| panic!("{}: no expected nominal range recorded", fixture.name));
        assert_eq!(
            resolved, expected_range,
            "{}: resolved Overworld range does not match the expected era value",
            fixture.name
        );

        check_contains_observed(&fixture, Dimension::Overworld, resolved);
    }
}

/// The same check as above, for the Nether and the End, on the fixtures
/// that carry those dimensions (`v1_18` and `v1_20_81`). Both have fixed,
/// version-independent nominal ranges, so there is no era table to build --
/// only the containment check applies.
#[test]
fn multi_dimension_fixtures_nether_and_end_ranges_contain_observed() {
    for fixture in support::fixtures() {
        if fixture.dimensions.len() < 2 {
            continue;
        }
        let (_tmp, settings, dictionary) = load_settings_and_dictionary(&fixture);

        for dimension in [Dimension::Nether, Dimension::End] {
            if !fixture
                .dimensions
                .iter()
                .any(|d| d == dimension_name_key(dimension))
            {
                continue;
            }

            let resolved = height_range::resolve(dimension, &settings, dictionary.as_ref())
                .unwrap_or_else(|| {
                    panic!("{}: {dimension:?} always has a defined range", fixture.name)
                });
            let expected = match dimension {
                Dimension::Nether => height_range::NETHER,
                Dimension::End => height_range::END,
                _ => unreachable!(),
            };
            assert_eq!(
                resolved, expected,
                "{}: resolved {dimension:?} range does not match the fixed nominal value",
                fixture.name
            );

            check_contains_observed(&fixture, dimension, resolved);
        }
    }
}

fn check_contains_observed(
    fixture: &support::Fixture,
    dimension: Dimension,
    resolved: HeightRange,
) {
    let Some(range) = fixture
        .subchunk_height_range
        .get(dimension_name_key(dimension))
    else {
        return;
    };
    let observed = HeightRange::new(range[0], range[1]);

    if known_observed_exceeds_nominal(&fixture.name, dimension) {
        assert!(
            !resolved.contains_range(observed),
            "{}: {dimension:?} observed range no longer exceeds nominal -- \
             update known_observed_exceeds_nominal",
            fixture.name
        );
    } else {
        assert!(
            resolved.contains_range(observed),
            "{}: resolved {dimension:?} range {resolved:?} does not contain the observed \
             subchunk range {observed:?}",
            fixture.name
        );
    }
}

/// Real dictionary-precedence check: `v1_18_30`, `v1_19_30`, and `v1_20_81`
/// are the three fixtures that carry a `LevelChunkMetaDataDictionary` (see
/// `metadata_dictionary.rs`'s module docs), and every Overworld entry in
/// each one records `-64..320` -- so resolving through the real dictionary
/// must produce the same value the era default would have produced anyway.
/// This proves the dictionary path reads real on-disk dictionaries
/// correctly; `height_range.rs`'s own unit tests separately prove the
/// dictionary path is actually consulted ahead of the default, using
/// synthetic data engineered to disagree with it (a check no real fixture
/// can pin, since none disagrees).
#[test]
fn real_dictionary_bearing_fixtures_resolve_through_the_dictionary() {
    for name in ["v1_18_30", "v1_19_30", "v1_20_81"] {
        let fixture = support::fixtures()
            .into_iter()
            .find(|f| f.name == name)
            .unwrap_or_else(|| panic!("fixture {name} not found in the index"));

        let (_tmp, _settings, dictionary) = load_settings_and_dictionary(&fixture);
        let dictionary = dictionary
            .unwrap_or_else(|| panic!("{name}: expected a LevelChunkMetaDataDictionary record"));

        assert!(
            dictionary.len() > 1,
            "{name}: expected multiple dictionary entries (the multiple-entry policy is real \
             here, even though every entry agrees on the range)"
        );

        let via_dictionary =
            height_range::dictionary_height_range(Dimension::Overworld, &dictionary)
                .unwrap_or_else(|| panic!("{name}: expected an Overworld dictionary entry"));
        assert_eq!(
            via_dictionary,
            height_range::OVERWORLD_AFTER_CAVES_AND_CLIFFS
        );
    }
}
