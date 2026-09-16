//! Phase 7's per-version test-fixture matrix: the acceptance test for the
//! "widest possible version range" goal (see `TODO.md`).
//!
//! For every chunk in every imported fixture this asserts a read -> write
//! round trip at *both* layers:
//!
//! - **Raw layer** (decision 3): `Chunk::decode` then `Chunk::encode` must
//!   reproduce every record byte for byte, with the two documented,
//!   corpus-confirmed exceptions treated as an explicit allowlist rather
//!   than a blanket skip -- a record only escapes the byte-identity
//!   requirement if it is a `0x2b`/`0x2f` record *and* re-decoding the
//!   round-tripped bytes still produces the identical typed value. See
//!   `packed_index_padding_bits_are_not_always_zero` in `bit_layers.rs` and
//!   `decodes_and_round_trips_every_record` in `biome3d.rs` for the two
//!   findings this mirrors (packed-index padding bits at widths 3/5/6, and
//!   the storage header's decode-invisible low bit).
//! - **Level layer** (decision 4): decoding the round-tripped bytes back
//!   into a `Chunk` must be structurally identical (terrain, biomes,
//!   source version, and resolved actors) to the `Chunk` the original bytes
//!   decoded to. This is "semantically stable" rather than byte-identical
//!   because the level layer's contract is the typed value, not the bytes
//!   (`level_chunk`'s module docs).
//!
//! `read_encode_write_round_trip_preserves_records_byte_for_byte` in
//! `level_test.rs` already checks this for one chunk per fixture; this file
//! is the exhaustive generalization over every chunk, plus the level-layer
//! comparison that test does not attempt.
//!
//! # Runtime and gating
//!
//! [`HEAVY_FIXTURES`] names the fixtures the default `cargo test` run
//! samples instead of walking exhaustively -- gated by measured decode
//! cost, not by compressed tarball size, since those two do not track each
//! other closely enough to gate on the wrong one: `v1_18`'s tarball is the
//! largest in the corpus by a wide margin and its exhaustive decode cost
//! matches that, but `v1_20_81`'s tarball is unremarkable (the fourth
//! largest) while its exhaustive decode cost is close to `v1_18`'s (both
//! took roughly 40-55s of the exhaustive run measured while writing this
//! test, together over half the exhaustive total). Decoding and re-encoding
//! every chunk (and every record inside it) across all thirteen fixtures
//! exhaustively is gated behind the `BEDROCK_LEVEL_EXHAUSTIVE_MATRIX`
//! environment variable; seeing the measured cost is what justifies the
//! gate (reported below, not asserted here since CI timing is not a
//! portable invariant). The default `cargo test` run still walks every
//! chunk of every *other* fixture exhaustively, and takes a deterministic
//! bounded sample of each heavy fixture (the first [`HEAVY_FIXTURE_SAMPLE`]
//! chunk positions in sorted order per dimension), so the default run still
//! exercises every heavy fixture, just not exhaustively.

mod support;

use std::collections::HashMap;
use std::io::Cursor;
use std::time::Instant;

use bedrock_level::Greedy;
use bedrock_level::biome::Biomes3d;
use bedrock_level::chunk::ChunkRecords;
use bedrock_level::key::KeyVariant;
use bedrock_level::level_chunk::Chunk;
use bedrock_level::subchunk::SubChunk;
use bedrock_level::types::ChunkPosition;
use bedrock_shared::world::dimension::Dimension;

/// Fixtures whose exhaustive record-by-record sweep is expensive enough
/// that the default `cargo test` run samples them instead of walking every
/// chunk. Chosen by measured decode cost (see the module docs), not by
/// compressed tarball size -- `v1_20_81` earns its place here despite a
/// mid-sized tarball because its exhaustive decode cost rivals `v1_18`'s.
const HEAVY_FIXTURES: &[&str] = &["v1_18", "v1_20_81"];

/// How many chunk positions per dimension a heavy fixture contributes to
/// the default (non-exhaustive) run.
const HEAVY_FIXTURE_SAMPLE: usize = 150;

fn exhaustive_enabled() -> bool {
    std::env::var_os("BEDROCK_LEVEL_EXHAUSTIVE_MATRIX").is_some()
}

/// One dimension's chunk positions, in deterministic order -- see
/// [`run_fixture`]'s grouping comment.
type DimensionGroup = (String, Dimension, Vec<(ChunkPosition, ChunkRecords)>);

/// Outcome of comparing one record's original bytes against its
/// round-tripped bytes.
enum RecordOutcome {
    ByteIdentical,
    /// Bytes differ, but the affected record is on the allowlist and
    /// re-decoding both byte strings produces an equal typed value.
    CarveOut,
    Mismatch(String),
}

/// Compares one key's original bytes against its round-tripped bytes,
/// applying the two known carve-outs exactly -- never a blanket "skip this
/// key variant" rule. A record only qualifies for [`RecordOutcome::CarveOut`]
/// if the bytes differ *and* both sides still decode to the same typed
/// value; a record whose bytes differ and which fails to decode, or decodes
/// to a different value, is a [`RecordOutcome::Mismatch`] regardless of
/// which key variant it is.
fn compare_record(variant: KeyVariant, original: &[u8], round_tripped: &[u8]) -> RecordOutcome {
    if original == round_tripped {
        return RecordOutcome::ByteIdentical;
    }

    match variant {
        KeyVariant::Biome3d => {
            let a = Biomes3d::from_disk::<Greedy, _>(&mut Cursor::new(original));
            let b = Biomes3d::from_disk::<Greedy, _>(&mut Cursor::new(round_tripped));
            match (a, b) {
                (Ok(a), Ok(b)) if a == b => RecordOutcome::CarveOut,
                _ => RecordOutcome::Mismatch(
                    "Biome3d bytes differ and do not decode to an equal value".to_string(),
                ),
            }
        }
        KeyVariant::SubChunk { .. } => {
            let a = SubChunk::from_disk::<Greedy, _>(&mut Cursor::new(original));
            let b = SubChunk::from_disk::<Greedy, _>(&mut Cursor::new(round_tripped));
            match (a, b) {
                (Ok(a), Ok(b)) if a == b => RecordOutcome::CarveOut,
                _ => RecordOutcome::Mismatch(
                    "SubChunk bytes differ and do not decode to an equal value".to_string(),
                ),
            }
        }
        _ => RecordOutcome::Mismatch(format!(
            "{} bytes originally, {} after round trip, first differing byte at {:?}",
            original.len(),
            round_tripped.len(),
            original
                .iter()
                .zip(round_tripped.iter())
                .position(|(a, b)| a != b)
        )),
    }
}

/// Structural equality of the level-layer's decoded `Chunk` -- every field
/// the type exposes that is meaningful to compare across a round trip.
/// `unmodelled_records` is deliberately excluded: it is exactly the raw
/// byte group already checked at the raw layer, so comparing it again here
/// would just be re-running that check under a different name.
fn chunks_semantically_equal(a: &Chunk, b: &Chunk) -> Vec<String> {
    let mut problems = Vec::new();
    if a.position() != b.position() {
        problems.push(format!(
            "position differs: {:?} vs {:?}",
            a.position(),
            b.position()
        ));
    }
    if a.dimension() != b.dimension() {
        problems.push(format!(
            "dimension differs: {:?} vs {:?}",
            a.dimension(),
            b.dimension()
        ));
    }
    if a.chunk_version() != b.chunk_version() {
        problems.push(format!(
            "chunk_version differs: {} vs {}",
            a.chunk_version(),
            b.chunk_version()
        ));
    }
    if a.terrain() != b.terrain() {
        problems.push("terrain differs".to_string());
    }
    if a.biomes() != b.biomes() {
        problems.push("biomes differ".to_string());
    }
    if a.actors() != b.actors() {
        problems.push("resolved actors differ".to_string());
    }
    problems
}

#[derive(Default)]
struct FixtureSummary {
    chunks_checked: usize,
    records_checked: usize,
    raw_byte_identical: usize,
    /// `Biome3d`/`SubChunk` records whose bytes differ only in a padding
    /// bit the typed value never carries -- see [`compare_record`]. The
    /// only carve-out this matrix still allows: `Chunk::encode_at` used to
    /// also synthesize a `0x2c` stamp for a versionless, contentless record
    /// group, but that was a real bug (fixed on `level_chunk.rs`), so an
    /// unearned version key appearing after a round trip is now an ordinary
    /// mismatch, tracked separately in [`version_stamp_regressions`](Self::
    /// version_stamp_regressions) below for a clearer failure message than
    /// the generic mismatch list gives.
    raw_carve_out_padding: usize,
    /// A `0x2c` key present after the round trip where the original record
    /// group had neither `0x2c` nor `0x76` -- pinned at exactly zero. See
    /// [`raw_carve_out_padding`](Self::raw_carve_out_padding)'s doc comment.
    version_stamp_regressions: usize,
    raw_mismatches: Vec<String>,
    level_semantically_stable: usize,
    level_mismatches: Vec<String>,
    decode_errors: Vec<String>,
}

/// Runs the matrix against one fixture's whole database (every dimension,
/// every chunk position present), sampling heavy fixtures down to
/// [`HEAVY_FIXTURE_SAMPLE`] positions per dimension unless
/// [`exhaustive_enabled`].
fn run_fixture(fixture: &support::Fixture) -> FixtureSummary {
    let (_tmp, db) = fixture.open();
    let mut summary = FixtureSummary::default();

    // One linear scan for the whole database, decoded once and reused for
    // both the raw-layer and level-layer comparisons below -- see the
    // module docs on `ChunkRecords::read_all` for why looping `ChunkRecords::
    // read` per chunk would instead re-scan the whole database per chunk.
    let all = ChunkRecords::read_all(&db)
        .unwrap_or_else(|e| panic!("{}: ChunkRecords::read_all failed: {e}", fixture.name));

    let heavy = HEAVY_FIXTURES.contains(&fixture.name.as_str());
    let sampled = heavy && !exhaustive_enabled();

    // Deterministic order: `Dimension` has no `Ord` (its `Other(i32)` id is
    // the only orderable part, and there is no reason to sort on it alone),
    // so group by dimension keyed on its `Debug` form -- stable across runs,
    // unlike `HashMap` iteration order -- and sort each dimension's chunk
    // positions with `ChunkPosition`'s own `Ord`. This is what makes a
    // sampled run exercise the same positions every time.
    let mut by_dimension: HashMap<String, (Dimension, Vec<(ChunkPosition, ChunkRecords)>)> =
        HashMap::new();
    for ((position, dimension), records) in all {
        by_dimension
            .entry(format!("{dimension:?}"))
            .or_insert_with(|| (dimension, Vec::new()))
            .1
            .push((position, records));
    }
    let mut dimension_groups: Vec<DimensionGroup> = by_dimension
        .into_iter()
        .map(|(label, (dimension, mut positions))| {
            positions.sort_by_key(|(position, _)| *position);
            (label, dimension, positions)
        })
        .collect();
    dimension_groups.sort_by(|a, b| a.0.cmp(&b.0));

    for (_, dimension, positions) in dimension_groups {
        let take = if sampled {
            HEAVY_FIXTURE_SAMPLE
        } else {
            usize::MAX
        };
        for (position, original) in positions.into_iter().take(take) {
            let original_records: Vec<(KeyVariant, Vec<u8>)> = original
                .iter()
                .map(|(key, bytes)| (key.data, bytes.to_vec()))
                .collect();

            let chunk = match Chunk::decode(original.clone(), &db) {
                Ok(chunk) => chunk,
                Err(e) => {
                    summary.decode_errors.push(format!(
                        "{}: {position:?}/{dimension:?}: Chunk::decode failed: {e}",
                        fixture.name
                    ));
                    continue;
                }
            };

            let round_tripped = match chunk.encode() {
                Ok(records) => records,
                Err(e) => {
                    summary.decode_errors.push(format!(
                        "{}: {position:?}/{dimension:?}: Chunk::encode failed: {e}",
                        fixture.name
                    ));
                    continue;
                }
            };

            let round_tripped_map: HashMap<KeyVariant, Vec<u8>> = round_tripped
                .iter()
                .map(|(key, bytes)| (key.data, bytes.to_vec()))
                .collect();
            let original_map: HashMap<KeyVariant, Vec<u8>> =
                original_records.iter().cloned().collect();

            for (variant, bytes) in &original_records {
                summary.records_checked += 1;
                match round_tripped_map.get(variant) {
                    None => summary.raw_mismatches.push(format!(
                        "{}: {position:?}/{dimension:?} {variant:?}: present originally, missing after round trip",
                        fixture.name
                    )),
                    Some(rt) => match compare_record(*variant, bytes, rt) {
                        RecordOutcome::ByteIdentical => summary.raw_byte_identical += 1,
                        RecordOutcome::CarveOut => summary.raw_carve_out_padding += 1,
                        RecordOutcome::Mismatch(reason) => summary.raw_mismatches.push(format!(
                            "{}: {position:?}/{dimension:?} {variant:?}: {reason}",
                            fixture.name
                        )),
                    },
                }
            }
            // `Chunk::encode_at` used to synthesize a `0x2c` stamp for
            // *any* record group missing both version keys, including
            // versionless, contentless generation-progress stubs -- fixed
            // on `level_chunk.rs` to synthesize one only when this chunk is
            // about to write real terrain or biome content. A group with no
            // terrain/biomes and no version key must therefore stay exactly
            // that way through a round trip; any newly-appearing key here is
            // an unconditional mismatch, and a resurfacing `0x2c` key
            // specifically is counted separately so a regression is
            // unmistakable rather than lost in the generic mismatch list.
            let had_no_version_key = !original_map.contains_key(&KeyVariant::ChunkVersion)
                && !original_map.contains_key(&KeyVariant::LegacyVersion);
            for variant in round_tripped_map.keys() {
                if original_map.contains_key(variant) {
                    continue;
                }
                if had_no_version_key && *variant == KeyVariant::ChunkVersion {
                    summary.version_stamp_regressions += 1;
                }
                summary.raw_mismatches.push(format!(
                    "{}: {position:?}/{dimension:?} {variant:?}: absent originally, present after round trip",
                    fixture.name
                ));
            }

            // Level-layer semantic stability: decode the round-tripped bytes
            // again and compare the two typed `Chunk`s. A fresh `ChunkRecords`
            // is built from `round_tripped_map` (rather than reusing
            // `round_tripped` directly) purely because `Chunk::decode` consumes
            // its argument and `round_tripped` was already iterated above --
            // the bytes are identical either way.
            let mut round_tripped_records = ChunkRecords::new(position, dimension);
            for (variant, bytes) in round_tripped.iter() {
                round_tripped_records.insert(
                    bedrock_level::key::Key {
                        chunk: position,
                        dimension,
                        data: variant.data,
                    },
                    bytes.to_vec(),
                );
            }
            match Chunk::decode(round_tripped_records, &db) {
                Ok(re_decoded) => {
                    let problems = chunks_semantically_equal(&chunk, &re_decoded);
                    if problems.is_empty() {
                        summary.level_semantically_stable += 1;
                    } else {
                        summary.level_mismatches.push(format!(
                            "{}: {position:?}/{dimension:?}: {}",
                            fixture.name,
                            problems.join("; ")
                        ));
                    }
                }
                Err(e) => summary.decode_errors.push(format!(
                    "{}: {position:?}/{dimension:?}: re-decoding the round-tripped bytes failed: {e}",
                    fixture.name
                )),
            }

            summary.chunks_checked += 1;
        }
    }

    summary
}

/// The exhaustive generalization of `level_test.rs`'s single-chunk round
/// trip: every chunk in every fixture (subject to the heavy-fixture sample
/// described in the module docs) round-trips byte-identically at the raw
/// layer -- modulo the one remaining named carve-out -- and
/// semantically-stably at the level layer. Also pins that the version-stamp
/// carve-out this matrix used to need is gone: [`version_stamp_regressions`]
/// stays at exactly zero.
///
/// [`version_stamp_regressions`]: FixtureSummary::version_stamp_regressions
#[test]
fn every_chunk_round_trips_at_both_layers() {
    let fixtures = support::fixtures();
    assert!(!fixtures.is_empty());

    let started = Instant::now();
    let mut total_chunks = 0usize;
    let mut total_records = 0usize;
    let mut total_carve_out_padding = 0usize;
    let mut total_version_stamp_regressions = 0usize;
    let mut all_problems: Vec<String> = Vec::new();

    println!(
        "version-matrix: exhaustive={} (set BEDROCK_LEVEL_EXHAUSTIVE_MATRIX=1 to cover {:?} fully)",
        exhaustive_enabled(),
        HEAVY_FIXTURES
    );

    for fixture in &fixtures {
        let fixture_started = Instant::now();
        let summary = run_fixture(fixture);
        let elapsed = fixture_started.elapsed();

        println!(
            "  {}: {} chunks, {} records ({} byte-identical, {} padding-bit carve-out, \
             {} mismatch), {} semantically stable at the level layer, {} decode errors, \
             {} version-stamp regressions -- {:.2}s",
            fixture.name,
            summary.chunks_checked,
            summary.records_checked,
            summary.raw_byte_identical,
            summary.raw_carve_out_padding,
            summary.raw_mismatches.len(),
            summary.level_semantically_stable,
            summary.decode_errors.len(),
            summary.version_stamp_regressions,
            elapsed.as_secs_f64(),
        );

        total_chunks += summary.chunks_checked;
        total_records += summary.records_checked;
        total_carve_out_padding += summary.raw_carve_out_padding;
        total_version_stamp_regressions += summary.version_stamp_regressions;
        all_problems.extend(summary.raw_mismatches);
        all_problems.extend(summary.level_mismatches);
        all_problems.extend(summary.decode_errors);
    }

    println!(
        "version-matrix: {total_chunks} chunks, {total_records} records, \
         {total_carve_out_padding} padding-bit carve-outs, total runtime {:.2}s",
        started.elapsed().as_secs_f64()
    );

    assert!(total_chunks > 0, "matrix ran against zero chunks");
    assert_eq!(
        total_version_stamp_regressions, 0,
        "the fixed `Chunk::encode_at` should never synthesize a version stamp for a group that \
         had neither `0x2c` nor `0x76` and no versioned content -- the carve-out this matrix \
         used to need for that is gone, and any nonzero count here is a real regression"
    );
    assert!(
        all_problems.is_empty(),
        "{} problem(s) found across the matrix:\n{}",
        all_problems.len(),
        all_problems.join("\n")
    );
}
