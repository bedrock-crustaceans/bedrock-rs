//! `ChunkRecords` against real fixture worlds: the group a chunk builds
//! matches an independent full scan exactly, and a group written into a
//! fresh database reads back byte-identical to what went in.

mod support;

use std::collections::HashMap;
use std::io::Cursor;

use bedrock_level::chunk::ChunkRecords;
use bedrock_level::db::Database;
use bedrock_level::key::{Key, KeyVariant};
use bedrock_level::types::ChunkPosition;
use bedrock_shared::world::dimension::Dimension;

use support::Fixture;

/// How many chunk positions to sample per dimension per fixture. Kept small
/// enough that the suite stays fast but large enough that a membership bug
/// affecting only some record kinds (subchunks at only some indices, say)
/// has more than one chance to surface.
const CHUNKS_PER_DIMENSION: usize = 6;

/// One fixture per era, chosen the same way the rest of the suite does:
/// filter on the harness's capability flags rather than naming a world
/// directly, so the selection tracks whatever the fixture set actually
/// contains.
fn pick_by_capability(all: &[Fixture], pred: impl Fn(&Fixture) -> bool) -> &Fixture {
    let mut matches: Vec<&Fixture> = all.iter().filter(|f| pred(f)).collect();
    matches.sort_by(|a, b| a.name.cmp(&b.name));
    matches
        .into_iter()
        .next()
        .expect("no fixture matched the capability filter")
}

/// Scans every key in `db` directly and groups the chunk-scoped ones by
/// `(chunk, dimension)`, deliberately not going through
/// `bedrock_level::chunk`'s own scan -- this is the independent reference
/// `ChunkRecords` is checked against, not a second call into the code under
/// test.
fn independent_scan(
    db: &Database,
) -> HashMap<(ChunkPosition, Dimension), HashMap<KeyVariant, Vec<u8>>> {
    let mut out: HashMap<(ChunkPosition, Dimension), HashMap<KeyVariant, Vec<u8>>> = HashMap::new();
    let mut keys = db.keys().expect("failed to iterate database");

    for kv in &mut keys {
        let key_bytes = kv.key();
        let mut cursor = Cursor::new(key_bytes.as_ref());
        let Ok(key) = Key::deserialize(&mut cursor) else {
            continue;
        };
        if !key.data.is_chunk_scoped() {
            continue;
        }
        out.entry((key.chunk, key.dimension))
            .or_default()
            .insert(key.data, kv.value().into());
    }

    out
}

/// Picks up to `CHUNKS_PER_DIMENSION` chunk positions from each dimension
/// present in `reference`, sorted for determinism.
fn sample_positions(
    reference: &HashMap<(ChunkPosition, Dimension), HashMap<KeyVariant, Vec<u8>>>,
) -> Vec<(ChunkPosition, Dimension)> {
    let mut by_dimension: HashMap<Dimension, Vec<ChunkPosition>> = HashMap::new();
    for &(pos, dim) in reference.keys() {
        by_dimension.entry(dim).or_default().push(pos);
    }

    let mut out = Vec::new();
    for (dim, mut positions) in by_dimension {
        positions.sort_by_key(|p| (p.0, p.1));
        positions.truncate(CHUNKS_PER_DIMENSION);
        out.extend(positions.into_iter().map(|p| (p, dim)));
    }
    out.sort_by_key(|(p, dim)| (p.0, p.1, i32::from(*dim)));
    out
}

/// Opens a fresh, empty database in its own temp directory.
fn open_fresh_db() -> (tempfile::TempDir, Database) {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let db_path = tmp.path().join("db");
    let db = Database::open(db_path.to_str().unwrap()).expect("failed to create fresh database");
    (tmp, db)
}

#[test]
fn chunk_records_match_scan_and_round_trip_across_eras() {
    let all = support::fixtures();

    let old = pick_by_capability(&all, |f| {
        f.capabilities.biomes_2d && !f.capabilities.biomes_3d
    });
    let mid = pick_by_capability(&all, |f| {
        f.capabilities.biomes_3d && !f.capabilities.negative_y
    });
    let new = pick_by_capability(&all, |f| {
        f.capabilities.biomes_3d && f.capabilities.negative_y && f.dimensions.len() > 1
    });

    let mut fixtures_tested = 0usize;
    let mut chunks_tested_total = 0usize;
    let mut records_tested_total = 0usize;

    for fixture in [old, mid, new] {
        let (_tmp, db) = fixture.open();

        let reference = independent_scan(&db);
        assert!(
            !reference.is_empty(),
            "{}: independent scan found no chunk-scoped keys at all",
            fixture.name
        );

        let sample = sample_positions(&reference);
        assert!(
            !sample.is_empty(),
            "{}: sampled zero chunk positions",
            fixture.name
        );

        // Build every sampled chunk's group in one pass, the way a caller
        // touching more than one chunk should.
        let groups = ChunkRecords::read_many(&db, sample.iter().copied())
            .unwrap_or_else(|e| panic!("{}: read_many failed: {e}", fixture.name));
        assert_eq!(
            groups.len(),
            sample.len(),
            "{}: read_many returned a different chunk count than was sampled -- a real \
             chunk position is missing from the result",
            fixture.name
        );

        let (_tmp2, fresh_db) = open_fresh_db();

        // `ChunkRecords::read` (the single-chunk convenience) is built on
        // the same scan as `read_many`, so it is checked once per fixture
        // against whichever chunk was sampled first rather than once per
        // sampled chunk -- each call is its own full database scan (see its
        // doc comment), and repeating that per chunk here would multiply
        // this test's runtime by the sample size for no additional
        // coverage over what the `read_many`-driven loop below already
        // proves for every other sampled chunk.
        let (first_pos, first_dim) = sample[0];
        let single = ChunkRecords::read(&db, first_pos, first_dim)
            .unwrap_or_else(|e| panic!("{}: read failed: {e}", fixture.name))
            .unwrap_or_else(|| {
                panic!(
                    "{}: {first_pos:?}/{first_dim:?} present in the scan but read() returned None",
                    fixture.name
                )
            });
        assert_eq!(single.position(), first_pos);
        assert_eq!(single.dimension(), first_dim);
        assert_eq!(single.len(), groups[&(first_pos, first_dim)].len());

        for &(pos, dim) in &sample {
            let group = groups.get(&(pos, dim)).unwrap_or_else(|| {
                panic!("{}: {pos:?}/{dim:?} missing from read_many", fixture.name)
            });
            let reference_group = &reference[&(pos, dim)];

            assert!(
                !group.is_empty(),
                "{}: {pos:?}/{dim:?} was returned as a group but holds zero records",
                fixture.name
            );

            // Exact key-set equality and byte identity against the
            // independent scan: every key the scan attributes to this
            // chunk must be in the group, with the same bytes, and the
            // group must contain nothing extra. `KeyVariant`'s discriminant
            // collapses every `SubChunk` index to the same tag byte, so the
            // set is compared through `HashSet` (which uses the variant's
            // full `Eq`/`Hash`, index included) rather than a sorted `Vec`.
            let group_variants: std::collections::HashSet<KeyVariant> =
                group.iter().map(|(key, _)| key.data).collect();
            let reference_variants: std::collections::HashSet<KeyVariant> =
                reference_group.keys().copied().collect();
            assert_eq!(
                group_variants, reference_variants,
                "{}: {pos:?}/{dim:?} key set disagrees with the independent scan",
                fixture.name
            );

            for (variant, expected_bytes) in reference_group {
                let actual = group.get(*variant).unwrap_or_else(|| {
                    panic!(
                        "{}: {pos:?}/{dim:?} is missing {variant:?}, present in the independent scan",
                        fixture.name
                    )
                });
                assert_eq!(
                    actual,
                    expected_bytes.as_slice(),
                    "{}: {pos:?}/{dim:?} {variant:?} bytes disagree with the independent scan",
                    fixture.name
                );
            }

            records_tested_total += group.len();
            chunks_tested_total += 1;

            group
                .write(&fresh_db)
                .unwrap_or_else(|e| panic!("{}: write failed: {e}", fixture.name));
        }

        // Round-trip: every sampled group was just written into `fresh_db`.
        // Scanning it independently must reproduce every record, byte for
        // byte, with no extra keys and none missing -- architecture decision
        // 3's raw-layer guarantee, exercised end to end.
        let round_tripped = independent_scan(&fresh_db);
        assert_eq!(
            round_tripped.len(),
            sample.len(),
            "{}: fresh database holds a different chunk count than was written",
            fixture.name
        );
        for &(pos, dim) in &sample {
            let expected = &reference[&(pos, dim)];
            let actual = round_tripped.get(&(pos, dim)).unwrap_or_else(|| {
                panic!(
                    "{}: {pos:?}/{dim:?} missing entirely after the round trip",
                    fixture.name
                )
            });
            assert_eq!(
                actual, expected,
                "{}: {pos:?}/{dim:?} did not round-trip byte-identically",
                fixture.name
            );
        }

        fixtures_tested += 1;
    }

    // Guard against a vacuous pass: real fixtures, real chunks, real
    // records must actually have been exercised.
    assert_eq!(fixtures_tested, 3);
    assert!(chunks_tested_total > 0);
    assert!(records_tested_total > 0);
    println!(
        "chunk_records_match_scan_and_round_trip_across_eras: {fixtures_tested} fixtures, \
         {chunks_tested_total} chunks, {records_tested_total} records"
    );
}

/// `take`/`insert` model the "the crate interpreted this key" overlay: a
/// caller pulls a record out for interpretation, and putting a replacement
/// back leaves every other record untouched. This does not depend on real
/// game data -- it is exercising `ChunkRecords`' own contract -- but it does
/// so against a group actually built from a fixture, not a hand-built one.
#[test]
fn take_and_insert_leave_the_rest_of_the_group_untouched() {
    let all = support::fixtures();
    let fixture = pick_by_capability(&all, |f| f.capabilities.biomes_3d);
    let (_tmp, db) = fixture.open();

    let reference = independent_scan(&db);
    let &(pos, dim) = reference
        .keys()
        .find(|(_, dim)| *dim == Dimension::Overworld)
        .expect("no overworld chunk found");

    let mut group = ChunkRecords::read(&db, pos, dim)
        .unwrap()
        .expect("chunk present in the scan must be readable");
    let original_len = group.len();
    assert!(original_len > 0);

    // Every variant actually present can be taken out and put back with
    // different bytes, and nothing else in the group moves.
    let variants: Vec<KeyVariant> = group.iter().map(|(key, _)| key.data).collect();
    for variant in variants {
        let before: std::collections::HashSet<KeyVariant> =
            group.iter().map(|(key, _)| key.data).collect();

        let taken = group.take(variant).expect("variant was just enumerated");
        assert!(!group.contains(variant));
        assert_eq!(group.len(), original_len - 1);

        let mut replacement = taken.clone();
        replacement.push(0xAB);
        let key = Key {
            chunk: pos,
            dimension: dim,
            data: variant,
        };
        let previous = group.insert(key.clone(), replacement.clone());
        assert_eq!(previous, None);
        assert_eq!(group.len(), original_len);
        assert_eq!(group.get(variant), Some(replacement.as_slice()));

        let after: std::collections::HashSet<KeyVariant> =
            group.iter().map(|(key, _)| key.data).collect();
        assert_eq!(
            before, after,
            "the record set changed shape for {variant:?}"
        );

        // Restore the original bytes so the next iteration's `before`
        // snapshot reflects the untouched group again.
        group.insert(key, taken);
    }
}

#[test]
#[should_panic(expected = "different chunk position")]
fn insert_with_a_mismatched_chunk_position_panics() {
    let mut group = ChunkRecords::new(ChunkPosition(0, 0), Dimension::Overworld);
    let key = Key {
        chunk: ChunkPosition(1, 1),
        dimension: Dimension::Overworld,
        data: KeyVariant::FinalizedState,
    };
    group.insert(key, vec![0]);
}

#[test]
#[should_panic(expected = "different dimension")]
fn insert_with_a_mismatched_dimension_panics() {
    let mut group = ChunkRecords::new(ChunkPosition(0, 0), Dimension::Overworld);
    let key = Key {
        chunk: ChunkPosition(0, 0),
        dimension: Dimension::Nether,
        data: KeyVariant::FinalizedState,
    };
    group.insert(key, vec![0]);
}

/// `KeyVariant::LocalPlayer` parses (it is a real, recognized key) but its
/// `Key` carries a fixed placeholder position rather than real coordinates,
/// so it must never surface as a member of any chunk's group. Regressing
/// `is_chunk_scoped` to a catch-all `true` would make every scan-driven
/// test above still pass (none of them ever asks about `LocalPlayer`
/// specifically), so this checks it directly against `read_all` -- which
/// this is also the suite's only exercise of.
#[test]
fn local_player_key_is_excluded_from_every_group() {
    let all = support::fixtures();
    let fixture = pick_by_capability(&all, |f| f.capabilities.biomes_3d);
    let (_tmp, db) = fixture.open();

    // Guard against a vacuous pass: the fixture must actually contain a
    // `~local_player` key, or excluding it proves nothing.
    let mut keys = db.keys().expect("failed to iterate database");
    let has_local_player = (&mut keys).any(|kv| {
        let key_bytes = kv.key();
        let mut cursor = Cursor::new(key_bytes.as_ref());
        matches!(Key::deserialize(&mut cursor), Ok(key) if key.data == KeyVariant::LocalPlayer)
    });
    assert!(
        has_local_player,
        "{}: fixture has no ~local_player key, so this test cannot prove anything",
        fixture.name
    );

    let groups = ChunkRecords::read_all(&db)
        .unwrap_or_else(|e| panic!("{}: read_all failed: {e}", fixture.name));
    assert!(!groups.is_empty());

    for group in groups.values() {
        assert!(!group.contains(KeyVariant::LocalPlayer));
        // Exercises `IntoIterator for &ChunkRecords` directly, not just
        // `ChunkRecords::iter`.
        for (key, _) in group {
            assert_ne!(
                key.data,
                KeyVariant::LocalPlayer,
                "{}: {:?}/{:?} carries a LocalPlayer record",
                fixture.name,
                key.chunk,
                key.dimension
            );
        }
    }
}

/// A record that is `take`n and never `insert`ed back must actually be
/// deleted from the database on `write`, not merely omitted -- otherwise a
/// read -> take -> write cycle against the same database leaves the stale
/// bytes on disk under a key nothing reachable from the group points at
/// anymore. Exercised against a real (extracted, mutable) copy of a fixture
/// database rather than a fresh one, so the deletion is checked against
/// genuine on-disk records, and every *other* record -- in this chunk and
/// every other chunk in the database -- is checked to be byte-identical
/// to what it was before the write.
#[test]
fn take_without_reinsert_deletes_the_stale_record_on_write() {
    let all = support::fixtures();
    let fixture = pick_by_capability(&all, |f| f.capabilities.biomes_3d);
    let (_tmp, db) = fixture.open();

    let before = independent_scan(&db);
    let &(pos, dim) = before
        .keys()
        .find(|(_, dim)| *dim == Dimension::Overworld)
        .expect("no overworld chunk found");

    let mut group = ChunkRecords::read(&db, pos, dim)
        .unwrap_or_else(|e| panic!("{}: read failed: {e}", fixture.name))
        .expect("chunk present in the scan must be readable");
    let dropped_variant = group
        .iter()
        .map(|(key, _)| key.data)
        .next()
        .expect("group has at least one record");
    let dropped_bytes = group
        .take(dropped_variant)
        .expect("just-enumerated variant must be present");
    assert_eq!(dropped_bytes, before[&(pos, dim)][&dropped_variant]);

    group.write(&db).unwrap_or_else(|e| {
        panic!(
            "{}: write back into the source database failed: {e}",
            fixture.name
        )
    });

    let after = independent_scan(&db);

    // Every chunk's group, including this one minus exactly the dropped
    // record, is byte-identical to what the database held before the write.
    let mut expected = before.clone();
    expected
        .get_mut(&(pos, dim))
        .unwrap()
        .remove(&dropped_variant);
    assert_eq!(
        after, expected,
        "{}: database state after take+write differs from expected (dropped {dropped_variant:?} \
         from {pos:?}/{dim:?})",
        fixture.name
    );
}
