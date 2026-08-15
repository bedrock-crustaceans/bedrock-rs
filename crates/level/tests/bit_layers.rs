//! Evidence for three Phase 1 tasks (see `crates/level/TODO.md`), gathered and pinned
//! against every fixture world plus the 1.26 seed world (`tests/level.tar.gz`), in one
//! streaming pass over every `0x2f` SubChunk record:
//!
//! - The `0x00` bit-array header (zero bits per index): confirms the on-disk layout is
//!   "header byte, then exactly one NBT compound -- no palette-length word, no index
//!   words" -- and that every real occurrence decodes and re-encodes to an equal
//!   in-memory value through the public API, not just that the layout looks right by
//!   inspection. (`crates/level/src/subchunk.rs`'s `zero_bit_header_round_trips_byte_identical`
//!   additionally pins full byte identity, using a synthetic record: a real palette entry's
//!   NBT field order is not always the order this crate's encoder writes it back in, which
//!   is a separate, pre-existing limitation shared by every palette entry regardless of bit
//!   width, not something specific to the zero-bit layout under test here.)
//! - The `0x7f` inherit sentinel never appears on a block/liquid layer.
//! - Bit widths 3, 5 and 6 (the ones whose packed words don't divide 4096 evenly) occur
//!   abundantly in real data, and a sample of real records at those widths decodes and
//!   re-encodes to an equal value, both greedily and lazily unpacked.
mod support;

use std::collections::BTreeMap;
use std::io::{Cursor, Seek};

use bedrock_level::db::Database;
use bedrock_level::key::{Key, KeyVariant};
use bedrock_level::subchunk::SubChunk;
use bedrock_level::{Greedy, Lazy, UnpackingMethod};
use byteorder::ReadBytesExt;

fn open_1_26() -> (tempfile::TempDir, Database) {
    let tmp = tempfile::tempdir().expect("tmp");
    let tar_gz = std::fs::File::open("tests/level.tar.gz").unwrap();
    let tar = flate2::read::GzDecoder::new(tar_gz);
    tar::Archive::new(tar).unpack(tmp.path()).unwrap();
    let db_path = tmp.path().join("debug/db");
    (tmp, Database::open(db_path.to_str().unwrap()).unwrap())
}

/// Header bytes of every layer in one subchunk record, in order: the value after `>> 1`.
/// A zero-bit header is followed by exactly one NBT compound and no count word (the
/// confirmed layout, see `bit_array_header_evidence_across_all_fixtures`), so the walk
/// skips past it and keeps going -- a subchunk can have a zero-bit block layer followed by
/// a zero-bit liquid layer. An inherit header cannot be skipped (its layout is exactly what
/// staying rejected means there's no need to define), so the walk stops there.
fn layer_headers(value: &[u8]) -> Vec<u8> {
    let mut headers = Vec::new();
    let mut r = Cursor::new(value);
    let Ok(version) = r.read_u8() else {
        return headers;
    };
    let layer_count = if version == 1 {
        1
    } else {
        match r.read_u8() {
            Ok(v) => v,
            Err(_) => return headers,
        }
    };
    if version == 9 && r.read_i8().is_err() {
        return headers;
    }

    for _ in 0..layer_count {
        let Ok(header) = r.read_u8() else {
            return headers;
        };
        let bits = header >> 1;
        headers.push(bits);

        if bits == 0x7f {
            return headers;
        }
        if bits == 0 {
            if nbtx::from_le_bytes::<nbtx::Value>(&mut r).is_err() {
                return headers;
            }
            continue;
        }

        let per_word = 32u32 / bits as u32;
        let word_count = 4096u32.div_ceil(per_word);
        if r.seek_relative(word_count as i64 * 4).is_err() {
            return headers;
        }
        let Ok(len) = r.read_u32::<byteorder::LittleEndian>() else {
            return headers;
        };
        for _ in 0..len {
            if nbtx::from_le_bytes::<nbtx::Value>(&mut r).is_err() {
                return headers;
            }
        }
    }

    headers
}

/// Decodes, re-encodes, and decodes again, checking the two decoded values are equal.
/// Real palette entries are not always written back in the same NBT field order they were
/// read in (a separate, pre-existing limitation -- see the module doc comment), so this
/// checks the decoded *value*, not the re-encoded bytes, against the original record.
fn round_trips_to_an_equal_value<M: UnpackingMethod>(source: &str, value: &[u8], bits: u8) {
    let chunk = SubChunk::from_disk::<M, _>(&mut Cursor::new(value))
        .unwrap_or_else(|e| panic!("{source}: failed to decode a {bits}-bit record: {e}"));

    let mut out = Cursor::new(Vec::new());
    chunk
        .to_disk::<M, _>(&mut out)
        .unwrap_or_else(|e| panic!("{source}: failed to re-encode a {bits}-bit record: {e}"));
    let out = out.into_inner();

    let chunk2 =
        SubChunk::from_disk::<M, _>(&mut Cursor::new(out.as_slice())).unwrap_or_else(|e| {
            panic!("{source}: failed to decode the re-encoded {bits}-bit record: {e}")
        });

    assert_eq!(
        chunk, chunk2,
        "{source}: {bits}-bit record did not round-trip to an equal value"
    );
}

#[derive(Default)]
struct Findings {
    records: u64,
    zero_bit_by_source: BTreeMap<String, u64>,
    zero_bit_round_tripped: u64,
    inherit_total: u64,
    bit_width_histogram: BTreeMap<u8, u64>,
    /// Per padded width (3, 5, 6), how many real records have been round-tripped so far.
    /// Capped so the test does not decode+re-encode every one of the thousands of real
    /// occurrences -- a handful per width is enough to confirm the implementation, not
    /// just the layout, against real data.
    padded_width_round_tripped: BTreeMap<u8, u64>,
}

const PADDED_WIDTH_SAMPLE_CAP: u64 = 3;

fn scan(source: &str, db: &Database, f: &mut Findings) {
    let mut keys = db.keys().unwrap();
    for kv in &mut keys {
        let mut key_buf = Cursor::new(kv.key());
        let Ok(key) = Key::deserialize(&mut key_buf) else {
            continue;
        };
        let KeyVariant::SubChunk { .. } = key.data else {
            continue;
        };

        f.records += 1;
        let value = kv.value();
        let headers = layer_headers(&value);

        for &bits in &headers {
            match bits {
                0 => {
                    *f.zero_bit_by_source.entry(source.to_string()).or_insert(0) += 1;
                    round_trips_to_an_equal_value::<Greedy>(source, &value, 0);
                    f.zero_bit_round_tripped += 1;
                }
                0x7f => f.inherit_total += 1,
                b @ (3 | 5 | 6) => {
                    *f.bit_width_histogram.entry(b).or_insert(0) += 1;
                    let count = f.padded_width_round_tripped.entry(b).or_insert(0);
                    if *count < PADDED_WIDTH_SAMPLE_CAP {
                        round_trips_to_an_equal_value::<Greedy>(source, &value, b);
                        round_trips_to_an_equal_value::<Lazy>(source, &value, b);
                        *count += 1;
                    }
                }
                b => {
                    *f.bit_width_histogram.entry(b).or_insert(0) += 1;
                }
            }
        }
    }
}

/// Settles, against real data, the two empirical questions the TODO needed answered
/// before `0x00` could be decoded and `0x7f` could stay rejected -- and confirms the
/// bit-width word-packing math (padded widths 3, 5, 6) against a sample of real records,
/// not just synthetic ones. See `greedy::tests` and `lazy::tests` for the exhaustive
/// per-width synthetic coverage.
#[test]
fn bit_array_header_evidence_across_all_fixtures() {
    let mut f = Findings::default();

    let (tmp, db) = open_1_26();
    scan("1_26_seed", &db, &mut f);
    drop(tmp);

    for fixture in support::fixtures() {
        let (tmp, db) = fixture.open();
        scan(&fixture.name, &db, &mut f);
        drop(tmp);
    }

    let zero_bit_total: u64 = f.zero_bit_by_source.values().sum();
    println!("subchunk records scanned: {}", f.records);
    println!("zero-bit layers by source: {:?}", f.zero_bit_by_source);
    println!("zero-bit layers total: {zero_bit_total}");
    println!("inherit-header layers total: {}", f.inherit_total);
    println!("bit-width histogram: {:?}", f.bit_width_histogram);

    assert_eq!(
        f.zero_bit_by_source.get("1_26_seed").copied().unwrap_or(0),
        11,
        "1.26 seed world zero-bit layer count"
    );
    assert_eq!(
        f.zero_bit_by_source.get("v1_18").copied().unwrap_or(0),
        11,
        "v1_18 fixture zero-bit layer count"
    );
    assert_eq!(
        f.zero_bit_by_source.get("v1_19_30").copied().unwrap_or(0),
        1,
        "v1_19_30 fixture zero-bit layer count"
    );
    assert_eq!(
        zero_bit_total, 23,
        "total zero-bit layers across all sources"
    );
    assert_eq!(
        f.zero_bit_round_tripped, 23,
        "all zero-bit layers decode and re-encode to an equal value"
    );
    assert_eq!(
        f.inherit_total, 0,
        "block/liquid layers must never carry the 0x7f inherit header"
    );

    for bits in [3u8, 5, 6] {
        assert!(
            f.bit_width_histogram.get(&bits).copied().unwrap_or(0) > 0,
            "expected at least one real layer at {bits} bits"
        );
        assert_eq!(
            f.padded_width_round_tripped
                .get(&bits)
                .copied()
                .unwrap_or(0),
            PADDED_WIDTH_SAMPLE_CAP,
            "expected to round-trip a sample of real {bits}-bit records"
        );
    }
}
