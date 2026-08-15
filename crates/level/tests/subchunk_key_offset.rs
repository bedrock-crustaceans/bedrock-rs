//! Empirical evidence for the subchunk key index-byte offset (see
//! `crates/level/src/key.rs`) against the imported fixture worlds.
//!
//! Two independent checks:
//!
//! - `subchunk_key_matches_v9_record_index_everywhere`: for every fixture,
//!   every version-9 subchunk record's embedded index byte must equal its
//!   key's raw index byte. None of the imported fixtures carry a version-9
//!   record whose chunk version falls inside the offset window (25..=28),
//!   so this pins delta == 0 in every fixture rather than demonstrating a
//!   nonzero delta directly -- see the module doc on why that is expected
//!   fixture coverage, not a gap in the check.
//! - `experimental_caves_and_cliffs_key_range_implies_the_offset`: the one
//!   fixture whose chunk version (25) *is* inside the window,
//!   `v1_17_20_caves_and_cliffs`, carries only version-8 subchunk records,
//!   which embed no index byte to compare against. The offset is instead
//!   inferred from the raw key index range: it is exactly four sections
//!   taller than its non-experimental sibling `v1_17_20`, entirely on the
//!   low end, and every raw index stays non-negative even though the
//!   corresponding un-offset 1.17.40 experimental world
//!   (`v1_17_40_caves_and_cliffs`, chunk version 31, outside the window)
//!   stores that same extra depth as genuinely negative indices. Reading
//!   the 1.17.20 experimental range through the offset conversion lines its
//!   absolute range up exactly with the 1.17.40 one.

mod support;

use std::collections::BTreeMap;
use std::io::Cursor;

use bedrock_level::key::{Key, KeyVariant, subchunk_index_from_key};
use bedrock_shared::world::dimension::Dimension;

/// One decoded `0x2f` record: the key's raw index byte, the record's own
/// version byte, and -- only for version 9 -- the index byte embedded in
/// the record body (the byte immediately after version + layer count).
struct SubChunkRow {
    dimension: Dimension,
    key_index: i8,
    record_index: Option<i8>,
}

fn scan_subchunks(fixture_name: &str, db: &bedrock_level::db::Database) -> Vec<SubChunkRow> {
    let mut rows = Vec::new();
    let mut keys = db.keys().expect("failed to iterate database");

    for kv in &mut keys {
        let key_bytes = kv.key();
        let mut cursor = Cursor::new(key_bytes.as_ref());
        let Ok(key) = Key::deserialize(&mut cursor) else {
            continue;
        };

        if let KeyVariant::SubChunk { index } = key.data {
            let value = kv.value();
            let record_version = value.first().copied().unwrap_or(0);
            // Layout: version(1) | layer_count(1) | index(1, only version 9) | ...
            let record_index = (record_version == 9).then(|| {
                *value.get(2).unwrap_or_else(|| {
                    panic!(
                        "{fixture_name}: version-9 subchunk record at key index {index} is \
                         shorter than its own header (missing the embedded index byte)"
                    )
                }) as i8
            });

            rows.push(SubChunkRow {
                dimension: key.dimension,
                key_index: index,
                record_index,
            });
        }
    }

    rows
}

/// Per-fixture, per-dimension key index range for version-9 subchunk
/// records ([min, max], inclusive), used to compare the raw on-disk range
/// between sibling plain/experimental worlds.
fn overworld_key_index_range(rows: &[SubChunkRow]) -> (i8, i8) {
    let indices: Vec<i8> = rows
        .iter()
        .filter(|r| r.dimension == Dimension::Overworld)
        .map(|r| r.key_index)
        .collect();
    (
        *indices.iter().min().expect("no overworld subchunks found"),
        *indices.iter().max().expect("no overworld subchunks found"),
    )
}

/// For every fixture, every version-9 subchunk record's embedded index byte
/// must equal the raw index byte on its own key. This is the decisive check
/// the offset window was defined against: a nonzero delta would mean either
/// the window is wrong or the conversion direction is wrong. Every imported
/// fixture's chunk version falls outside 25..=28, so every delta pins to 0 --
/// see the module doc.
#[test]
fn subchunk_key_matches_v9_record_index_everywhere() {
    let mut any_v9_checked = false;

    for fixture in support::fixtures() {
        let (_tmp, db) = fixture.open();
        let rows = scan_subchunks(&fixture.name, &db);

        let mut mismatches: BTreeMap<i32, u32> = BTreeMap::new();
        let mut v9_count = 0u32;

        for row in &rows {
            let Some(record_index) = row.record_index else {
                continue;
            };
            v9_count += 1;
            let delta = row.key_index as i32 - record_index as i32;
            if delta != 0 {
                *mismatches.entry(delta).or_default() += 1;
            }
        }

        assert!(
            mismatches.is_empty(),
            "{}: key index disagreed with the version-9 record's embedded index \
             (delta -> count): {mismatches:?}",
            fixture.name,
        );

        if v9_count > 0 {
            any_v9_checked = true;
        }
    }

    assert!(
        any_v9_checked,
        "expected at least one fixture with version-9 subchunk records"
    );
}

/// `v1_17_20_caves_and_cliffs` is chunk version 25, the only imported
/// fixture inside the offset window, but its subchunks are version 8 (no
/// embedded index to check directly). The offset is instead confirmed by
/// comparing its raw key range against two other fixtures that need no
/// conversion to interpret: its own non-experimental sibling `v1_17_20`
/// (same era, no offset -- chunk version 22 is outside the window), and the
/// later, un-offset `v1_17_40_caves_and_cliffs` (chunk version 31, past the
/// window, storing real negative indices directly).
#[test]
fn experimental_caves_and_cliffs_key_range_implies_the_offset() {
    let all = support::fixtures();
    let find = |name: &str| {
        all.iter()
            .find(|f| f.name == name)
            .unwrap_or_else(|| panic!("fixture {name} not found"))
    };

    let plain_1_17_20 = find("v1_17_20");
    let experimental_1_17_20 = find("v1_17_20_caves_and_cliffs");
    let experimental_1_17_40 = find("v1_17_40_caves_and_cliffs");

    assert_eq!(
        plain_1_17_20.chunk_versions,
        vec![22],
        "test assumes v1_17_20 is chunk version 22"
    );
    assert_eq!(
        experimental_1_17_20.chunk_versions,
        vec![25],
        "test assumes v1_17_20_caves_and_cliffs is chunk version 25, the \
         start of the offset window"
    );
    assert_eq!(
        experimental_1_17_40.chunk_versions,
        vec![31],
        "test assumes v1_17_40_caves_and_cliffs is chunk version 31, past \
         the offset window"
    );

    let (_tmp_plain, db_plain) = plain_1_17_20.open();
    let (_tmp_exp_20, db_exp_20) = experimental_1_17_20.open();
    let (_tmp_exp_40, db_exp_40) = experimental_1_17_40.open();

    let plain_range = overworld_key_index_range(&scan_subchunks(&plain_1_17_20.name, &db_plain));
    let exp_20_range =
        overworld_key_index_range(&scan_subchunks(&experimental_1_17_20.name, &db_exp_20));
    let exp_40_range =
        overworld_key_index_range(&scan_subchunks(&experimental_1_17_40.name, &db_exp_40));

    // Raw, unconverted: the 1.17.20 experimental world's on-disk indices
    // never go negative, and cover exactly 4 more sections than its plain
    // sibling, all appended below (min stays 0, max grows by 4).
    assert_eq!(plain_range, (0, 6));
    assert_eq!(exp_20_range, (0, 10));

    // Converting the 1.17.20 experimental range through the offset
    // (chunk version 25, Overworld) reproduces the 1.17.40 experimental
    // world's genuinely negative range exactly -- the same physical depth,
    // stored two different ways on either side of the window.
    let converted_range = (
        subchunk_index_from_key(exp_20_range.0, 25, Dimension::Overworld),
        subchunk_index_from_key(exp_20_range.1, 25, Dimension::Overworld),
    );
    assert_eq!(converted_range, exp_40_range);
    assert_eq!(exp_40_range, (-4, 6));
}
