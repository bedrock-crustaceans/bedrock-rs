//! Corpus survey and write-semantics tests for `0x2b` (paletted 3D biomes).
//!
//! The survey (`corpus_survey_fragment_counts_vs_subchunk_counts`) establishes, from
//! every real `0x2b` record in every `biomes_3d` fixture, the per-fixture fragment-count
//! histogram and where `Inherit` fragments occur in the list, and pins both as standing
//! corpus facts. That evidence is what `Biomes3d::biome_at`/`from_columns`/`new` in
//! `biome.rs` implement against -- see `Biomes3d`'s own doc comment for the conclusion
//! this survey supports (the fragment count is presence-driven, not derivable from a
//! dimension's height range) and this test's `println!` output (run with `--nocapture`)
//! for the numbers themselves. The corpus here is the imported fixture set only: the 1.26
//! seed world (`tests/level.tar.gz`) was checked and carries zero `0x2b` records, so it
//! contributes nothing to these counts.

pub mod support;

use std::collections::BTreeMap;
use std::io::{Cursor, Write};
use std::num::NonZeroUsize;

use bedrock_level::Greedy;
use bedrock_level::biome::{BiomeEncoding, Biomes2d, Biomes3d, Biomes3dError, COLUMNS};
use bedrock_level::key::{Key, KeyVariant};
use bedrock_level::level::Level;
use bedrock_level::types::BlockPosition;
use smallvec::SmallVec;

/// Subchunk count spanned by a `HeightRange`: `(max - min) / 16`.
fn subchunk_count(range: bedrock_level::height_range::HeightRange) -> i32 {
    (range.max - range.min) / 16
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one survey pass over the corpus followed by the pinned facts it must reproduce"
)]
fn corpus_survey_fragment_counts_vs_subchunk_counts() {
    let fixtures: Vec<_> = support::fixtures()
        .into_iter()
        .filter(|f| f.capabilities.biomes_3d)
        .collect();
    assert!(
        !fixtures.is_empty(),
        "expected at least one biomes_3d fixture"
    );

    // fixture -> fragment_count -> record count. Dimension-independent: `Biomes3d`'s doc
    // comment explains why the fragment count does not key on the record's own dimension.
    let mut histogram: BTreeMap<String, BTreeMap<usize, usize>> = BTreeMap::new();
    let mut inherit_first: usize = 0;
    let mut inherit_mid: usize = 0;
    let mut inherit_tail: usize = 0;
    let mut tail_omitted: usize = 0;
    let mut total_records: usize = 0;

    for fixture in &fixtures {
        let tmp = tempfile::tempdir().expect("failed to create temp dir");
        let tar_gz_path = format!("tests/fixtures/{}", fixture.tarball);
        let tar_gz = std::fs::File::open(&tar_gz_path).unwrap();
        let tar = flate2::read::GzDecoder::new(tar_gz);
        tar::Archive::new(tar).unpack(tmp.path()).unwrap();

        let level = Level::open(tmp.path().join("world"))
            .unwrap_or_else(|e| panic!("{}: failed to open as a Level: {e}", fixture.name));
        let db = level.raw();
        let mut keys = db.keys().unwrap();

        // The Overworld's own resolved bound, used as the "expected" section count for
        // every record regardless of its own dimension -- see `Biomes3d`'s doc comment:
        // real Nether/End records carry the same fragment count their world's Overworld
        // records do, not their own (smaller) dimension bound.
        let Some(range) = level.bounds(bedrock_shared::world::dimension::Dimension::Overworld)
        else {
            continue;
        };
        let expected_subchunks = subchunk_count(range);

        for kv in &mut keys {
            let mut key_cursor = Cursor::new(kv.key());
            let Ok(key) = Key::deserialize(&mut key_cursor) else {
                continue;
            };
            if key.data != KeyVariant::Biome3d {
                continue;
            }

            let bytes: Vec<u8> = kv.value().into();
            let mut reader = Cursor::new(bytes.as_slice());
            let decoded = Biomes3d::from_disk::<Greedy, _>(&mut reader).unwrap_or_else(|e| {
                panic!(
                    "{}: chunk {:?}: decode failed: {e}",
                    fixture.name, key.chunk
                )
            });

            let n = decoded.fragments().len();
            total_records += 1;
            *histogram
                .entry(fixture.name.clone())
                .or_default()
                .entry(n)
                .or_insert(0) += 1;

            if i32::try_from(n).unwrap() < expected_subchunks {
                tail_omitted += 1;
            }

            for (i, frag) in decoded.fragments().iter().enumerate() {
                if matches!(frag, BiomeEncoding::Inherit) {
                    if i == 0 {
                        inherit_first += 1;
                    } else if i + 1 == n {
                        inherit_tail += 1;
                    } else {
                        inherit_mid += 1;
                    }
                }
            }
        }
    }

    println!("=== 0x2b corpus survey: per-fixture fragment-count histogram ===");
    for (fixture, per) in &histogram {
        println!("{fixture}: {per:?}");
    }
    println!(
        "totals: records={total_records} tail_omitted={tail_omitted} \
         Inherit position: first={inherit_first} mid={inherit_mid} tail={inherit_tail}"
    );

    // Pinned standing corpus facts -- a change to any of these means the fixtures
    // changed, or the decoder changed, and either is worth noticing rather than a
    // silent `> 0` pass.
    assert_eq!(inherit_first, 0, "an Inherit fragment must never be first");
    assert_eq!(
        tail_omitted, 0,
        "no real record in this corpus omits a trailing storage -- the tail fill-up \
         rule biome_at implements is unexercised by real writes, which this pins"
    );
    assert_eq!(total_records, 7795);

    let expected: BTreeMap<&str, BTreeMap<usize, usize>> = BTreeMap::from([
        ("v1_17_20_caves_and_cliffs", BTreeMap::from([(32, 182)])),
        ("v1_17_40_caves_and_cliffs", BTreeMap::from([(32, 138)])),
        ("v1_18", BTreeMap::from([(25, 2820)])),
        ("v1_18_30", BTreeMap::from([(24, 792)])),
        ("v1_18_superflat", BTreeMap::from([(25, 6)])),
        ("v1_18_updated_superflat", BTreeMap::from([(25, 7)])),
        ("v1_19_30", BTreeMap::from([(24, 792)])),
        ("v1_20_81", BTreeMap::from([(24, 3055), (25, 3)])),
    ]);
    let observed: BTreeMap<&str, BTreeMap<usize, usize>> = histogram
        .iter()
        .map(|(name, per)| (name.as_str(), per.clone()))
        .collect();
    assert_eq!(
        observed, expected,
        "per-fixture fragment-count histogram changed from the pinned corpus observation"
    );
}

fn sample_heightmap() -> Box<[u16; COLUMNS]> {
    let mut hm = Box::new([0u16; COLUMNS]);
    for (i, h) in hm.iter_mut().enumerate() {
        *h = u16::try_from(i % 256).unwrap();
    }
    hm
}

fn uniform_columns(id: u8) -> Biomes2d {
    Box::new([id; COLUMNS]).into()
}

fn checkerboard_columns() -> Biomes2d {
    let mut ids = Box::new([0u8; COLUMNS]);
    for (i, id) in ids.iter_mut().enumerate() {
        *id = if i % 2 == 0 { 1 } else { 4 };
    }
    ids.into()
}

fn nz(n: usize) -> NonZeroUsize {
    NonZeroUsize::new(n).unwrap()
}

#[test]
fn new_rejects_empty_fragment_list() {
    let err = Biomes3d::new(sample_heightmap(), SmallVec::new()).unwrap_err();
    assert_eq!(err, Biomes3dError::Empty);
}

#[test]
fn new_rejects_first_fragment_inherit() {
    let fragments: SmallVec<[BiomeEncoding; 1]> = SmallVec::from_buf([BiomeEncoding::Inherit]);
    let err = Biomes3d::new(sample_heightmap(), fragments).unwrap_err();
    assert_eq!(err, Biomes3dError::FirstFragmentInherits);
}

#[test]
fn new_accepts_an_arbitrarily_long_fragment_list() {
    // There is no upper bound: see `Biomes3d`'s doc comment for why a dimension-derived
    // bound would reject real data (a v1.18-family world's Nether/End records carry 25
    // fragments against an 8- or 16-section dimension bound).
    let fragments: SmallVec<[BiomeEncoding; 1]> = SmallVec::from_vec(vec![
        BiomeEncoding::Single(1),
        BiomeEncoding::Inherit,
        BiomeEncoding::Inherit,
        BiomeEncoding::Inherit,
        BiomeEncoding::Inherit,
    ]);
    let biomes = Biomes3d::new(sample_heightmap(), fragments).unwrap();
    assert_eq!(biomes.fragments().len(), 5);
}

#[test]
fn new_accepts_a_single_fragment_list() {
    // Fewer fragments than a caller's presumed context is not a rejection -- the tail
    // fill-up rule biome_at implements is exactly for this case.
    let fragments: SmallVec<[BiomeEncoding; 1]> = SmallVec::from_buf([BiomeEncoding::Single(7)]);
    let biomes = Biomes3d::new(sample_heightmap(), fragments).unwrap();
    assert_eq!(biomes.fragments().len(), 1);
}

#[test]
fn from_columns_uniform_produces_single_then_inherit() {
    let biomes = Biomes3d::from_columns(sample_heightmap(), &uniform_columns(3), nz(5));
    assert_eq!(biomes.fragments().len(), 5);
    assert_eq!(biomes.fragments()[0], BiomeEncoding::Single(3));
    for frag in &biomes.fragments()[1..] {
        assert_eq!(*frag, BiomeEncoding::Inherit);
    }
}

#[test]
fn from_columns_varied_produces_palette_then_inherit() {
    let columns = checkerboard_columns();
    let biomes = Biomes3d::from_columns(sample_heightmap(), &columns, nz(3));
    assert_eq!(biomes.fragments().len(), 3);
    match &biomes.fragments()[0] {
        BiomeEncoding::Palette(array) => {
            assert_eq!(array.palette().len(), 2);
        }
        other => panic!("expected Palette, got {other:?}"),
    }
    assert_eq!(biomes.fragments()[1], BiomeEncoding::Inherit);
    assert_eq!(biomes.fragments()[2], BiomeEncoding::Inherit);
}

#[test]
fn from_columns_round_trips_through_disk() {
    let columns = checkerboard_columns();
    let biomes = Biomes3d::from_columns(sample_heightmap(), &columns, nz(4));

    let mut writer = Cursor::new(Vec::new());
    biomes.to_disk(&mut writer).unwrap();
    let bytes = writer.into_inner();

    let mut reader = Cursor::new(bytes.as_slice());
    let decoded = Biomes3d::from_disk::<Greedy, _>(&mut reader).unwrap();
    assert_eq!(decoded, biomes);
}

#[test]
fn biome_at_resolves_a_varied_bottom_fragment_by_column() {
    let columns = checkerboard_columns();
    let biomes = Biomes3d::from_columns(sample_heightmap(), &columns, nz(3));

    // Column (0, 0) -> flat index 0 -> id 1; column (1, 0) -> flat index 1 -> id 4.
    assert_eq!(biomes.biome_at(0, BlockPosition(0, 0, 0)), Some(1));
    assert_eq!(biomes.biome_at(0, BlockPosition(1, 0, 0)), Some(4));
}

#[test]
fn biome_at_resolves_inherit_by_walking_down_to_the_stored_fragment() {
    let biomes = Biomes3d::from_columns(sample_heightmap(), &uniform_columns(9), nz(4));
    // Fragments 1..4 are all Inherit; every position in them should resolve to the
    // bottom fragment's Single(9).
    for subchunk_index in 0..4 {
        assert_eq!(
            biomes.biome_at(subchunk_index, BlockPosition(5, 5, 5)),
            Some(9)
        );
    }
}

#[test]
fn biome_at_fills_upward_past_a_shorter_fragment_list() {
    let fragments: SmallVec<[BiomeEncoding; 1]> = SmallVec::from_buf([BiomeEncoding::Single(42)]);
    let biomes = Biomes3d::new(sample_heightmap(), fragments).unwrap();

    // Only one fragment is actually stored; any subchunk_index at or past the fragment
    // list's length resolves against it, per the tail fill-up rule -- there is no upper
    // bound to reject against.
    assert_eq!(biomes.biome_at(0, BlockPosition(0, 0, 0)), Some(42));
    assert_eq!(biomes.biome_at(23, BlockPosition(0, 0, 0)), Some(42));
    assert_eq!(biomes.biome_at(1000, BlockPosition(0, 0, 0)), Some(42));
}

#[test]
fn biome_at_rejects_out_of_range_coordinates() {
    let biomes = Biomes3d::from_columns(sample_heightmap(), &uniform_columns(1), nz(4));
    assert_eq!(biomes.biome_at(0, BlockPosition(16, 0, 0)), None);
    assert_eq!(biomes.biome_at(0, BlockPosition(0, 16, 0)), None);
    assert_eq!(biomes.biome_at(0, BlockPosition(0, 0, 16)), None);
}

#[test]
fn biome_at_malformed_first_fragment_inherit_resolves_none_only_when_the_walk_reaches_it() {
    // `Biomes3d::new`/`from_columns` cannot produce this layout (first fragment
    // `Inherit`), but `Biomes3d::from_disk` does not re-validate, so it is reachable from
    // a hand-crafted or corrupt record -- built here directly as on-disk bytes
    // (`encode_malformed_first_fragment_inherit`) and decoded back, bypassing `new`.
    // Only a walk that actually reaches fragment 0 sees `None`; a query resolved entirely
    // by a non-Inherit fragment higher up still succeeds.
    let biomes = encode_malformed_first_fragment_inherit();

    // Fragment 1 (Single(5)) resolves fine.
    assert_eq!(biomes.biome_at(1, BlockPosition(0, 0, 0)), Some(5));
    // Fragment 0 is Inherit with nothing below it: the walk cannot proceed and this
    // resolves None, but only because this specific query reaches fragment 0 -- not
    // because the whole record is unresolvable.
    assert_eq!(biomes.biome_at(0, BlockPosition(0, 0, 0)), None);
}

/// Hand-encodes a `0x2b`-shaped byte record whose first fragment is `Inherit` (a layout
/// `Biomes3d::new` rejects) followed by a `Single(5)` fragment, then decodes it through
/// `Biomes3d::from_disk`, which does not run `new`'s validation. Isolated here so the
/// malformed-first-fragment test above does not need the raw byte layout inline.
fn encode_malformed_first_fragment_inherit() -> Biomes3d {
    use byteorder::{LittleEndian, WriteBytesExt};

    let mut writer = Cursor::new(Vec::new());
    writer.write_all(&[0u8; COLUMNS * 2]).unwrap();
    // Inherit fragment: header byte only (0x7f << 1 | low bit, per `Biomes3d::to_disk`'s
    // doc comment on that low bit).
    writer.write_u8((0x7fu8 << 1) | 1).unwrap();
    // Single(5) fragment: header byte (empty flag 0x00 << 1 | low bit) then the id.
    writer.write_u8(1).unwrap();
    writer.write_u32::<LittleEndian>(5).unwrap();

    let bytes = writer.into_inner();
    let mut reader = Cursor::new(bytes.as_slice());
    Biomes3d::from_disk::<Greedy, _>(&mut reader).unwrap()
}
