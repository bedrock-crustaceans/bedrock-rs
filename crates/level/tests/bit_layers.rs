//! Evidence for several Phase 1 tasks (see `crates/level/TODO.md`), gathered and pinned
//! against every fixture world plus the 1.26 seed world (`tests/level.tar.gz`), in
//! streaming passes over every `0x2f` SubChunk record:
//!
//! - The `0x00` bit-array header (zero bits per index): confirms the on-disk layout is
//!   "header byte, then exactly one NBT compound -- no palette-length word, no index
//!   words" -- and that every real occurrence decodes and re-encodes to an equal
//!   in-memory value through the public API, not just that the layout looks right by
//!   inspection. (`crates/level/src/subchunk.rs`'s `zero_bit_header_round_trips_byte_identical`
//!   additionally pins full byte identity, using a synthetic record.)
//! - The `0x7f` inherit sentinel never appears on a block/liquid layer.
//! - Bit widths 3, 5 and 6 (the ones whose packed words don't divide 4096 evenly) occur
//!   abundantly in real data, and a sample of real records at those widths decodes and
//!   re-encodes to an equal value, both greedily and lazily unpacked.
//! - Every palette entry's NBT field order round-trips byte-identically -- top-level
//!   (`name`/`version`/`states` for modern entries, `name`/`val` for the pre-flattening
//!   `PaletteEntry::Legacy` form) and `states`' own internal key order alike
//!   (`palette_entries_round_trip_byte_identical_across_all_fixtures`), with one
//!   exactly-counted exclusion that is not a palette bug: a pre-existing packed-index-word
//!   limitation (see `packed_index_padding_bits_are_not_always_zero`).
//! - `BitArray::to_disk` deriving the header's bit width from the palette size, rather than
//!   retaining a read width, needs no change: every real layer's header already is that
//!   minimal width (same test).
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
/// Used for the zero-bit and padded-width samples below, which only need to confirm the
/// bit-array layout and packing are right; `palette_entries_round_trip_byte_identical_across_all_fixtures`
/// is the exhaustive, byte-identity version of this same idea, covering every record.
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

/// A palette entry's minimal valid bit width for a palette of length `palette_size` --
/// the same rule `BitArray::to_disk` derives the header from. Restated here (rather than
/// calling the crate's private helper) so this test is checking the crate's behaviour
/// against an independent computation, not against itself.
fn minimal_bits(palette_size: usize) -> u8 {
    const VALID_BITS: [u8; 8] = [1, 2, 3, 4, 5, 6, 8, 16];
    VALID_BITS
        .into_iter()
        .find(|&b| 2usize.pow(b as u32) >= palette_size)
        .unwrap_or(16)
}

/// Where a re-encoded record's first differing byte falls, relative to the layer that
/// contains it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum MismatchRegion {
    /// Inside the packed index words -- never inside a decoded index itself (every real
    /// case found is explained by [`packed_index_padding_bits_are_not_always_zero`]), but
    /// the encoder cannot reproduce padding bits it never read.
    PackedIndices,
    /// Inside the palette -- would mean the field-order fix this test exists to pin has a
    /// real gap.
    Palette,
}

/// Per-record findings for the full-corpus palette byte-identity pass.
#[derive(Default)]
struct PaletteFindings {
    records: u64,
    byte_identical: u64,
    /// Records whose `SubChunk::from_disk` failed. Pinned empty below: every record in the
    /// corpus, including `v1_12`'s pre-flattening `{name, val}` palettes (Phase 2's "Legacy
    /// `{name, val}` palette entries" task), now decodes.
    decode_failed_by_source: BTreeMap<String, u64>,
    /// Records that decoded but did not re-encode byte-identically, keyed by
    /// `(bit width, which region the first differing byte falls in)`. See
    /// [`packed_index_padding_bits_are_not_always_zero`] for what the `PackedIndices` bucket
    /// is; a nonzero `Palette` count would mean the field-order fix has a real gap.
    not_byte_identical: BTreeMap<(u8, MismatchRegion), u64>,
    /// Real layers whose header bit width is not the minimal width for its palette size,
    /// by (source, header bits, minimal bits, palette len) -- expected to stay empty.
    non_minimal_width: Vec<(String, u8, u8, usize)>,
}

/// Walks one `0x2f` record's layers, in the same framing `layer_headers` uses, calling
/// `on_layer` with each data layer's bit width, the byte offset its packed index words end
/// at (where the palette-length word starts), and its palette length.
fn for_each_data_layer<F: FnMut(u8, usize, usize)>(value: &[u8], mut on_layer: F) -> Option<()> {
    let mut r = Cursor::new(value);
    let version = r.read_u8().ok()?;
    let layer_count = if version == 1 { 1 } else { r.read_u8().ok()? };
    if version == 9 {
        r.read_i8().ok()?;
    }
    for _ in 0..layer_count {
        let header = r.read_u8().ok()?;
        let bits = header >> 1;
        if bits == 0x7f {
            return Some(());
        }
        if bits == 0 {
            nbtx::from_le_bytes::<nbtx::Value>(&mut r).ok()?;
            continue;
        }
        let per_word = 32u32 / bits as u32;
        let word_count = 4096u32.div_ceil(per_word);
        let index_end = r.position() as usize + (word_count as usize * 4);
        r.seek_relative(word_count as i64 * 4).ok()?;
        let len = r.read_u32::<byteorder::LittleEndian>().ok()?;
        for _ in 0..len {
            nbtx::from_le_bytes::<nbtx::Value>(&mut r).ok()?;
        }
        on_layer(bits, index_end, len as usize);
    }
    Some(())
}

fn scan_palette_fidelity(source: &str, db: &Database, f: &mut PaletteFindings) {
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

        // The last data layer's `index_end` is what matters below: a two-layer record's
        // first differing byte, if any, is compared against whichever layer it actually
        // falls in, but every real mismatch found while establishing this test landed in
        // the first layer, so tracking just the most recent `index_end` is enough to
        // classify them; a genuinely later-layer mismatch would undercount into `Palette`
        // rather than go unnoticed, since `index_end` here would be smaller than the true
        // boundary for that layer.
        let mut layer_index_ends = Vec::new();
        for_each_data_layer(&value, |bits, index_end, palette_len| {
            layer_index_ends.push(index_end);
            let minimal = minimal_bits(palette_len);
            if minimal != bits {
                f.non_minimal_width
                    .push((source.to_string(), bits, minimal, palette_len));
            }
        });

        match SubChunk::from_disk::<Greedy, _>(&mut Cursor::new(value.as_ref())) {
            Ok(chunk) => {
                let mut out = Cursor::new(Vec::new());
                chunk
                    .to_disk::<Greedy, _>(&mut out)
                    .unwrap_or_else(|e| panic!("{source}: failed to re-encode a record: {e}"));
                let out = out.into_inner();
                if out == value.as_ref() {
                    f.byte_identical += 1;
                } else {
                    let n = value.len().min(out.len());
                    let first_diff = (0..n).find(|&i| value[i] != out[i]).unwrap_or_else(|| {
                        panic!("{source}: differing lengths but no differing byte")
                    });
                    let region = if layer_index_ends.iter().any(|&end| first_diff < end) {
                        MismatchRegion::PackedIndices
                    } else {
                        MismatchRegion::Palette
                    };
                    // The header this record's *first* layer carries is enough to bucket
                    // it: every mismatch found while establishing this test was a
                    // single-layer 3-bit record.
                    let bits = layer_headers(&value).first().copied().unwrap_or(0);
                    *f.not_byte_identical.entry((bits, region)).or_insert(0) += 1;
                }
            }
            Err(_) => {
                *f.decode_failed_by_source
                    .entry(source.to_string())
                    .or_insert(0) += 1;
            }
        }
    }
}

/// The corpus-wide payoff of the palette-entry field-order fix (`subchunk.rs`'s
/// `FieldOrder`/order-preserving `states`) plus the legacy `{name, val}` palette entry
/// (`subchunk.rs`'s `PaletteEntry::Legacy`): every `0x2f` record in the corpus -- every
/// "modern" `name`/`version`/`states` entry and every pre-flattening `name`/`val` entry, at
/// every bit width, not a sample -- decodes, and re-encodes with its palette bytes matching
/// the original exactly, pinned by asserting the `MismatchRegion::Palette` bucket of
/// `not_byte_identical` stays empty. One thing this test found is *not* a palette bug and
/// is excluded by name rather than folded into a blanket byte-identity assertion:
/// [`packed_index_padding_bits_are_not_always_zero`] is an unrelated, pre-existing
/// limitation in the packed *index* words, nothing to do with the palette or either fix.
///
/// Also settles the bit-width-retention question from the same TODO entry: every real
/// layer's header already carries the minimal width for its palette size, so
/// `BitArray::to_disk` deriving rather than retaining the width needs no change --
/// pinned as a standing corpus assertion.
#[test]
fn palette_entries_round_trip_byte_identical_across_all_fixtures() {
    let mut f = PaletteFindings::default();

    let (tmp, db) = open_1_26();
    scan_palette_fidelity("1_26_seed", &db, &mut f);
    drop(tmp);

    for fixture in support::fixtures() {
        let (tmp, db) = fixture.open();
        scan_palette_fidelity(&fixture.name, &db, &mut f);
        drop(tmp);
    }

    let decode_failed_total: u64 = f.decode_failed_by_source.values().sum();
    let not_byte_identical_total: u64 = f.not_byte_identical.values().sum();
    println!("subchunk records scanned: {}", f.records);
    println!("byte-identical round trips: {}", f.byte_identical);
    println!(
        "decode failures (legacy palette format) by source: {:?}",
        f.decode_failed_by_source
    );
    println!("decode failures total: {decode_failed_total}");
    println!(
        "not-byte-identical by (bits, region): {:?}",
        f.not_byte_identical
    );

    assert!(
        f.non_minimal_width.is_empty(),
        "real layers whose header width is not minimal for their palette size: {:?}",
        f.non_minimal_width
    );

    // Every mismatch this test finds must fall in the packed index words, never the
    // palette -- a `Palette`-region mismatch would mean the field-order fix has a real gap.
    for (&(bits, region), &count) in &f.not_byte_identical {
        assert_eq!(
            region,
            MismatchRegion::PackedIndices,
            "{bits}-bit record(s) ({count}) did not round-trip byte-identically inside the \
             palette -- the field-order fix has a gap"
        );
    }
    // Pinned exactly rather than just asserted `PackedIndices`: a change to this number
    // (in either direction) is worth noticing. See
    // `packed_index_padding_bits_are_not_always_zero` for what these are.
    assert_eq!(
        f.not_byte_identical,
        BTreeMap::from([((3u8, MismatchRegion::PackedIndices), 20u64)]),
        "packed-index-padding mismatch set changed shape"
    );

    // Every record in the corpus decodes, `v1_12`'s 1,118 pre-flattening `{name, val}`
    // records (the only fixture carrying that format) included: `PaletteEntry::Legacy`
    // models it, so there is no longer a decode-failure source at all.
    assert_eq!(
        f.decode_failed_by_source,
        BTreeMap::new(),
        "every record in the corpus should decode -- the legacy-palette format is modeled now"
    );
    assert_eq!(
        f.byte_identical,
        f.records - decode_failed_total - not_byte_identical_total,
        "every record whose palette this crate models, and whose packed index words carry \
         no interior padding-bit garbage, round-trips byte-identically"
    );
}

/// A bit width of 3, 5 or 6 does not divide the 32-bit word evenly, so *every* packed
/// word -- not just a layer's final, partially-filled one (see
/// `greedy::tests::trailing_slot_garbage_is_ignored_on_decode`) -- carries a few leftover
/// high bits above its `32 / bits` whole indices (2 bits, for width 3: `10 * 3 = 30`).
/// `GreedyArray::unpack`/`pack_into` never read or write those bits -- they play no part in
/// any decoded index -- so if the game itself ever leaves them non-zero, a decode-then-
/// re-encode cycle is a lossy-on-paper, lossless-in-practice normalization: every decoded
/// block position is still exactly what was on disk, but the specific bytes differ.
///
/// It does happen: of the 149,646 real subchunk records this crate can decode at all
/// (`palette_entries_round_trip_byte_identical_across_all_fixtures`), exactly 20 -- all
/// 3-bit, spread across `v1_16`, `v1_17_20_caves_and_cliffs`, `v1_17_40` and `v1_18` --
/// carry non-zero padding bits inside an interior word (never the palette, and never
/// widths 5 or 6, at least in this corpus). This is a distinct, pre-existing limitation in
/// the packed-index codec (`greedy.rs`/`lazy.rs`), unrelated to the palette NBT field-order
/// fix this file otherwise pins byte identity for, and is out of scope here -- recorded as
/// a fact about the format rather than fixed.
#[test]
fn packed_index_padding_bits_are_not_always_zero() {
    // See the doc comment: this test exists to be found from
    // `palette_entries_round_trip_byte_identical_across_all_fixtures`'s pinned count, and
    // to keep the fact discoverable by name rather than only in a comment.
}
