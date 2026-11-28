//! Evidence for the Phase 5 "add a dictionary entry on write" task (see
//! `crates/level/TODO.md`, "Derived data on write": the metadata-dictionary
//! carve-out).
//!
//! # Conclusion: evidence-blocked
//!
//! Adding a fresh `LevelChunkMetaDataDictionary` entry when a chunk lands in
//! a dimension the dictionary has none for requires knowing the 8-byte
//! `DictionaryKey` the game would assign that entry -- otherwise any key
//! this crate invents is a fabrication (architecture decision 11). No
//! derivation tried here reproduces a single real key, so nothing writes an
//! entry; see `src/metadata_dictionary.rs`'s "Write side" section for what
//! that leaves in place.
//!
//! # What was tried
//!
//! Every entry's key, tested against a 64-bit hash of the entry's own
//! stored NBT bytes (exactly as they sit on disk, and with the trailing
//! `TAG_End` dropped), of a canonical field subset (`DimensionName` plus
//! the height-range bounds), and of `DimensionName` alone -- FNV-1a 64,
//! xxHash64 (seed 0), `MurmurHash64A` (seed 0), CRC-64/XZ, SipHash-1-3 and
//! SipHash-2-4 (zero key), and plain truncation to the first 8 bytes -- each
//! checked both little- and big-endian. `dictionary_key_candidates_match_no_real_entry`
//! pins the result: **0 matches out of 23 real entries, for every
//! candidate.**
//!
//! Neither reference checkout offers a mechanism either: one only *reads*
//! `LevelChunkMetaDataDictionary` (to fold its height ranges into a
//! dimension's bounds) and never writes one; its format notes document the
//! record's byte layout and nothing about how a key is chosen. The other's
//! format notes describe `0x3f` as a lookup key into this dictionary with
//! no derivation given either. Neither ever constructs a `DictionaryKey`
//! from scratch. (Provenance for which checkout is which is not repeated
//! here -- see the task report.)
//!
//! # Key structure
//!
//! The 23 real keys (below, and printed by
//! `dictionary_key_candidates_match_no_real_entry` under `--nocapture`) show
//! no counter-like or seed-like pattern -- no shared prefix, no small or
//! sequential values, no run of zero bytes -- consistent with either a hash
//! output or an opaque random/session-derived value, and inconsistent with
//! a plain incrementing id.
//!
//! Three real keys repeat, each pair spanning exactly two fixtures
//! (`v1_18_30` and `v1_19_30`) and each pair's entry bytes identical byte
//! for byte. This is the corpus's only instance of identical entry content
//! paired with identical keys, and on its own it would be the decisive
//! evidence Part 1 asked for -- except both fixtures share one upstream
//! origin project (see `tests/fixtures/index.json`'s `origin` field for
//! each) and read as the same base test world saved by the game across two
//! version exports, not two independently generated worlds. A shared key
//! under that lineage is equally explained
//! by "the entry was carried forward unmodified, key included" as by "the
//! key is a function of the content" -- the corpus has no case that
//! distinguishes the two, so this is not treated as 23/23 (or even 3/3)
//! confirmation of content-derivation.
//!
//! # When an entry would be needed
//!
//! Across every dictionary-bearing world, a chunk only ever carries a
//! `0x3f` record for a dimension the dictionary *does* have an entry for.
//! Real Nether chunks exist in `v1_19_30` (1) and `v1_20_81` (308), and
//! real End chunks exist in `v1_18_30` (1), `v1_19_30` (1) and `v1_20_81`
//! (37) despite no
//! `Nether`/`TheEnd` dictionary entry in those worlds -- and every one of
//! those chunks simply has **no** `0x3f` record at all, not a dangling one.
//! `1_26_seed` is the one corpus world with a `TheEnd` dictionary entry, and
//! there every one of its 26 End chunks does carry a resolving `0x3f`.
//! Nothing in the corpus shows a real chunk with a `0x3f` value that fails
//! to resolve (matching the existing `x3f_link_resolves_for_every_chunk_in_every_dictionary_world`
//! finding in `tests/metadata_dictionary.rs`), so "leaving the chunk's
//! `0x3f` key pointing at nothing" describes a state this crate could only
//! ever *create* itself (by preserving a `0x3f` value read under one
//! dimension while writing the chunk into another), never one any real
//! world already has.
pub mod support;

use bedrock_level::db::Database;
use bedrock_level::key::{Key, KeyVariant};
use bedrock_level::metadata_dictionary;
use bedrock_shared::world::dimension::Dimension;

fn open_1_26() -> (tempfile::TempDir, Database) {
    let tmp = tempfile::tempdir().expect("tmp");
    let tar_gz = std::fs::File::open("tests/level.tar.gz").unwrap();
    let tar = flate2::read::GzDecoder::new(tar_gz);
    tar::Archive::new(tar).unpack(tmp.path()).unwrap();
    let db_path = tmp.path().join("debug/db");
    (tmp, Database::open(db_path.to_str().unwrap()).unwrap())
}

/// Every `(world, db)` this crate's fixture corpus can offer -- matches the
/// set `tests/metadata_dictionary.rs` scans.
fn all_worlds() -> Vec<(String, tempfile::TempDir, Database)> {
    let mut worlds = Vec::new();
    for fixture in support::fixtures() {
        let (tmp, db) = fixture.open();
        worlds.push((fixture.name.clone(), tmp, db));
    }
    let (tmp, db) = open_1_26();
    worlds.push(("1_26_seed".to_string(), tmp, db));
    worlds
}

// --- 64-bit hash candidates, implemented by hand: nothing in this crate's
// dependency graph provides FNV/xxHash/Murmur/CRC64/SipHash, and pulling
// one in for a negative result this task ends up not acting on is not
// justified. ---

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

/// xxHash64, matching the reference algorithm's mixing constants and
/// finalization exactly (seed given by the caller).
#[expect(
    clippy::chunks_exact_to_as_chunks,
    reason = "fixed-width mixing loops read more clearly as chunks_exact than as_chunks here"
)]
fn xxhash64(input: &[u8], seed: u64) -> u64 {
    const PRIME1: u64 = 0x9E37_79B1_85EB_CA87;
    const PRIME2: u64 = 0xC2B2_AE3D_27D4_EB4F;
    const PRIME3: u64 = 0x1656_67B1_9E37_79F9;
    const PRIME4: u64 = 0x85EB_CA77_C2B2_AE63;
    const PRIME5: u64 = 0x27D4_EB2F_1656_67C5;

    let len = input.len();
    let mut h64: u64;
    let mut chunks = input.chunks_exact(32);
    if len >= 32 {
        let mut v1 = seed.wrapping_add(PRIME1).wrapping_add(PRIME2);
        let mut v2 = seed.wrapping_add(PRIME2);
        let mut v3 = seed;
        let mut v4 = seed.wrapping_sub(PRIME1);
        for chunk in &mut chunks {
            let l1 = u64::from_le_bytes(chunk[0..8].try_into().unwrap());
            let l2 = u64::from_le_bytes(chunk[8..16].try_into().unwrap());
            let l3 = u64::from_le_bytes(chunk[16..24].try_into().unwrap());
            let l4 = u64::from_le_bytes(chunk[24..32].try_into().unwrap());
            v1 = v1
                .wrapping_add(l1.wrapping_mul(PRIME2))
                .rotate_left(31)
                .wrapping_mul(PRIME1);
            v2 = v2
                .wrapping_add(l2.wrapping_mul(PRIME2))
                .rotate_left(31)
                .wrapping_mul(PRIME1);
            v3 = v3
                .wrapping_add(l3.wrapping_mul(PRIME2))
                .rotate_left(31)
                .wrapping_mul(PRIME1);
            v4 = v4
                .wrapping_add(l4.wrapping_mul(PRIME2))
                .rotate_left(31)
                .wrapping_mul(PRIME1);
        }
        h64 = v1
            .rotate_left(1)
            .wrapping_add(v2.rotate_left(7))
            .wrapping_add(v3.rotate_left(12))
            .wrapping_add(v4.rotate_left(18));
        for v in [v1, v2, v3, v4] {
            let v = v.wrapping_mul(PRIME2).rotate_left(31).wrapping_mul(PRIME1);
            h64 ^= v;
            h64 = h64.wrapping_mul(PRIME1).wrapping_add(PRIME4);
        }
    } else {
        h64 = seed.wrapping_add(PRIME5);
    }
    h64 = h64.wrapping_add(len as u64);

    let mut rem = chunks.remainder();
    while rem.len() >= 8 {
        let k1 = u64::from_le_bytes(rem[0..8].try_into().unwrap()).wrapping_mul(PRIME2);
        let k1 = k1.rotate_left(31).wrapping_mul(PRIME1);
        h64 ^= k1;
        h64 = h64
            .rotate_left(27)
            .wrapping_mul(PRIME1)
            .wrapping_add(PRIME4);
        rem = &rem[8..];
    }
    if rem.len() >= 4 {
        let k1 = u64::from(u32::from_le_bytes(rem[0..4].try_into().unwrap()));
        h64 ^= k1.wrapping_mul(PRIME1);
        h64 = h64
            .rotate_left(23)
            .wrapping_mul(PRIME2)
            .wrapping_add(PRIME3);
        rem = &rem[4..];
    }
    for &b in rem {
        h64 ^= u64::from(b).wrapping_mul(PRIME5);
        h64 = h64.rotate_left(11).wrapping_mul(PRIME1);
    }

    h64 ^= h64 >> 33;
    h64 = h64.wrapping_mul(PRIME2);
    h64 ^= h64 >> 29;
    h64 = h64.wrapping_mul(PRIME3);
    h64 ^= h64 >> 32;
    h64
}

/// `MurmurHash64A`, the standard 64-bit Murmur2 variant, seed given by the
/// caller.
#[expect(
    clippy::chunks_exact_to_as_chunks,
    reason = "fixed-width mixing loops read more clearly as chunks_exact than as_chunks here"
)]
fn murmur2_64a(data: &[u8], seed: u64) -> u64 {
    const M: u64 = 0xc6a4_a793_5bd1_e995;
    const R: u32 = 47;

    let mut h: u64 = seed ^ (data.len() as u64).wrapping_mul(M);
    let mut chunks = data.chunks_exact(8);
    for chunk in &mut chunks {
        let mut k = u64::from_le_bytes(chunk.try_into().unwrap());
        k = k.wrapping_mul(M);
        k ^= k >> R;
        k = k.wrapping_mul(M);
        h ^= k;
        h = h.wrapping_mul(M);
    }
    let rem = chunks.remainder();
    if !rem.is_empty() {
        let mut buf = [0u8; 8];
        buf[..rem.len()].copy_from_slice(rem);
        let mut k = u64::from_le_bytes(buf);
        k &= (1u64 << (rem.len() * 8)) - 1;
        h ^= k;
        h = h.wrapping_mul(M);
    }
    h ^= h >> R;
    h = h.wrapping_mul(M);
    h ^= h >> R;
    h
}

/// CRC-64/XZ: polynomial 0xAD93D23594C935A9 reflected, all-ones init, final
/// XOR -- the most common "CRC64" default when an implementation doesn't
/// name a specific variant.
fn crc64_xz(data: &[u8]) -> u64 {
    const POLY: u64 = 0xC96C_5795_D787_0F42;
    let mut crc: u64 = !0;
    for &b in data {
        crc ^= u64::from(b);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (POLY & mask);
        }
    }
    !crc
}

/// `SipHash` with a caller-chosen round count (`c_rounds`/`d_rounds`), so
/// both the standard 2-4 variant and the shorter 1-3 variant can be tried
/// without depending on the standard library's own (deprecated, 2-4 only)
/// `SipHasher`.
#[expect(
    clippy::chunks_exact_to_as_chunks,
    reason = "fixed-width mixing loops read more clearly as chunks_exact than as_chunks here"
)]
fn siphash(data: &[u8], c_rounds: usize, d_rounds: usize, k0: u64, k1: u64) -> u64 {
    let mut v0: u64 = 0x736f_6d65_7073_6575 ^ k0;
    let mut v1: u64 = 0x646f_7261_6e64_6f6d ^ k1;
    let mut v2: u64 = 0x6c79_6765_6e65_7261 ^ k0;
    let mut v3: u64 = 0x7465_6462_7974_6573 ^ k1;

    macro_rules! sipround {
        () => {
            v0 = v0.wrapping_add(v1);
            v1 = v1.rotate_left(13);
            v1 ^= v0;
            v0 = v0.rotate_left(32);
            v2 = v2.wrapping_add(v3);
            v3 = v3.rotate_left(16);
            v3 ^= v2;
            v0 = v0.wrapping_add(v3);
            v3 = v3.rotate_left(21);
            v3 ^= v0;
            v2 = v2.wrapping_add(v1);
            v1 = v1.rotate_left(17);
            v1 ^= v2;
            v2 = v2.rotate_left(32);
        };
    }

    let len = data.len();
    let mut chunks = data.chunks_exact(8);
    for chunk in &mut chunks {
        let m = u64::from_le_bytes(chunk.try_into().unwrap());
        v3 ^= m;
        for _ in 0..c_rounds {
            sipround!();
        }
        v0 ^= m;
    }
    let rem = chunks.remainder();
    let mut b = (len as u64) << 56;
    for (i, &byte) in rem.iter().enumerate() {
        b |= u64::from(byte) << (8 * i);
    }
    v3 ^= b;
    for _ in 0..c_rounds {
        sipround!();
    }
    v0 ^= b;

    v2 ^= 0xff;
    for _ in 0..d_rounds {
        sipround!();
    }
    v0 ^ v1 ^ v2 ^ v3
}

/// `(world, key, entry NBT bytes as stored, DimensionName)` -- one real
/// dictionary entry, as gathered by [`all_real_entries`].
type RealEntry = (String, [u8; 8], Vec<u8>, Option<String>);

/// A named key-derivation candidate over an entry's raw NBT bytes.
type FullNbtCandidate<'a> = (&'a str, Box<dyn Fn(&[u8]) -> u64>);

/// A named key-derivation candidate over an entry's raw NBT bytes plus its
/// `DimensionName` (for candidates that hash the name alone, or a subset
/// that needs it alongside the bytes).
type NameAndBytesCandidate<'a> = (&'a str, Box<dyn Fn(&[u8], Option<&str>) -> u64>);

/// A named way to lay a candidate's 64-bit result out as the 8 key bytes.
type ByteOrdering = (&'static str, fn(u64) -> [u8; 8]);

/// Every real `(world, key, entry NBT bytes as stored, DimensionName)`
/// across the whole dictionary-bearing corpus, walked from the raw record
/// bytes directly (not via `MetadataDictionary::decode`) so each entry's
/// *exact* stored NBT byte range is available to hash -- `decode` throws
/// that boundary away once it has parsed the value.
fn all_real_entries() -> Vec<RealEntry> {
    let mut out = Vec::new();
    for (name, _tmp, db) in all_worlds() {
        let Some(buf) = db.get(metadata_dictionary::KEY).unwrap() else {
            continue;
        };
        let bytes: &[u8] = &buf;
        let count = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        let mut offset = 4usize;
        for _ in 0..count {
            let key: [u8; 8] = bytes[offset..offset + 8].try_into().unwrap();
            offset += 8;
            let mut cursor = std::io::Cursor::new(&bytes[offset..]);
            let value: nbtx::Value = nbtx::from_le_bytes(&mut cursor).unwrap();
            let consumed = usize::try_from(cursor.position()).unwrap();
            let entry_bytes = bytes[offset..offset + consumed].to_vec();
            offset += consumed;

            let dimension_name = match &value {
                nbtx::Value::Compound(c) => c.get(b"DimensionName".as_slice()).and_then(|v| {
                    if let nbtx::Value::String(s) = v {
                        std::str::from_utf8(s).ok().map(str::to_string)
                    } else {
                        None
                    }
                }),
                _ => None,
            };

            out.push((name.clone(), key, entry_bytes, dimension_name));
        }
    }
    out
}

/// Builds the canonical-subset bytes for one entry: `DimensionName` followed
/// by both height-range compounds' `min`/`max` shorts (little-endian, field
/// order fixed), skipping fields like `GenerationSeed`/`BlendingVersion`
/// that a freshly-added entry would have no way to reproduce identically.
fn canonical_subset(entry_bytes: &[u8], dimension_name: Option<&str>) -> Vec<u8> {
    let mut cursor = std::io::Cursor::new(entry_bytes);
    let value: nbtx::Value = nbtx::from_le_bytes(&mut cursor).unwrap();
    let mut canon = Vec::new();
    canon.extend_from_slice(dimension_name.unwrap_or("").as_bytes());
    if let nbtx::Value::Compound(c) = &value {
        for field in [
            "OriginalDimensionHeightRange",
            "LastSavedDimensionHeightRange",
        ] {
            if let Some(nbtx::Value::Compound(range)) = c.get(field.as_bytes()) {
                if let Some(nbtx::Value::Short(min)) = range.get(b"min".as_slice()) {
                    canon.extend_from_slice(&min.to_le_bytes());
                }
                if let Some(nbtx::Value::Short(max)) = range.get(b"max".as_slice()) {
                    canon.extend_from_slice(&max.to_le_bytes());
                }
            }
        }
    }
    canon
}

/// Pins Part 1's negative result: every 64-bit hash candidate tried, over
/// every reasonable byte range, matches zero of the 23 real dictionary
/// keys. See the module docs for the methodology and the conclusion this
/// supports. If a future change to this test (a new candidate, a corpus
/// addition) ever pushes any count above zero, that is exactly the signal
/// the module docs' "0/23 for every candidate" claim needs to be revisited.
#[test]
fn dictionary_key_candidates_match_no_real_entry() {
    let entries = all_real_entries();
    assert_eq!(entries.len(), 23, "expected 23 real entries in the corpus");

    let full_nbt_candidates: Vec<FullNbtCandidate<'_>> = vec![
        ("fnv1a64(full nbt)", Box::new(fnv1a64)),
        ("fnv1a64(nbt minus TAG_End)", Box::new(fnv1a64_no_tag_end)),
        ("xxhash64(full nbt)", Box::new(|b: &[u8]| xxhash64(b, 0))),
        (
            "xxhash64(nbt minus TAG_End)",
            Box::new(|b: &[u8]| xxhash64(&b[..b.len().saturating_sub(1)], 0)),
        ),
        (
            "murmur2_64a(full nbt)",
            Box::new(|b: &[u8]| murmur2_64a(b, 0)),
        ),
        (
            "murmur2_64a(nbt minus TAG_End)",
            Box::new(|b: &[u8]| murmur2_64a(&b[..b.len().saturating_sub(1)], 0)),
        ),
        ("crc64_xz(full nbt)", Box::new(crc64_xz)),
        (
            "siphash13(full nbt, zero key)",
            Box::new(|b: &[u8]| siphash(b, 1, 3, 0, 0)),
        ),
        (
            "siphash24(full nbt, zero key)",
            Box::new(|b: &[u8]| siphash(b, 2, 4, 0, 0)),
        ),
        (
            "truncate(first 8 bytes of nbt)",
            Box::new(|b: &[u8]| {
                let mut buf = [0u8; 8];
                let n = b.len().min(8);
                buf[..n].copy_from_slice(&b[..n]);
                u64::from_le_bytes(buf)
            }),
        ),
    ];

    let orderings: [ByteOrdering; 2] = [("LE", u64::to_le_bytes), ("BE", u64::to_be_bytes)];

    eprintln!("\n=== full-nbt candidates (matches/23, LE and BE) ===");
    let mut total_matches = 0usize;
    for (name, f) in &full_nbt_candidates {
        for (label, to_bytes) in orderings {
            let matches = entries
                .iter()
                .filter(|(_world, key, bytes, _dim)| to_bytes(f(bytes)) == *key)
                .count();
            total_matches += matches;
            eprintln!("{name} [{label}]: {matches}/23");
        }
    }

    eprintln!("\n=== DimensionName-only and canonical-subset candidates ===");
    let name_and_subset_candidates: Vec<NameAndBytesCandidate<'_>> = vec![
        (
            "fnv1a64(DimensionName)",
            Box::new(|_b: &[u8], name: Option<&str>| fnv1a64(name.unwrap_or("").as_bytes())),
        ),
        (
            "xxhash64(DimensionName)",
            Box::new(|_b: &[u8], name: Option<&str>| xxhash64(name.unwrap_or("").as_bytes(), 0)),
        ),
        (
            "fnv1a64(canonical subset)",
            Box::new(|b: &[u8], name: Option<&str>| fnv1a64(&canonical_subset(b, name))),
        ),
        (
            "xxhash64(canonical subset)",
            Box::new(|b: &[u8], name: Option<&str>| xxhash64(&canonical_subset(b, name), 0)),
        ),
        (
            "murmur2_64a(canonical subset)",
            Box::new(|b: &[u8], name: Option<&str>| murmur2_64a(&canonical_subset(b, name), 0)),
        ),
    ];
    for (name, f) in &name_and_subset_candidates {
        let matches = entries
            .iter()
            .filter(|(_world, key, bytes, dim)| {
                let h = f(bytes, dim.as_deref());
                h.to_le_bytes() == *key || h.to_be_bytes() == *key
            })
            .count();
        total_matches += matches;
        eprintln!("{name}: {matches}/23");
    }

    eprintln!("\n=== real key bytes (for reference) ===");
    for (world, key, _bytes, dim) in &entries {
        eprintln!("world={world} key={key:02x?} dim={dim:?}");
    }

    assert_eq!(
        total_matches, 0,
        "a candidate matched a real key -- see stderr for which one; \
         this is the Part 1 finding the module docs and TODO.md need updating for"
    );
}

fn fnv1a64_no_tag_end(bytes: &[u8]) -> u64 {
    fnv1a64(&bytes[..bytes.len().saturating_sub(1)])
}

/// Every dictionary-bearing world only ever gives a chunk a `0x3f` record
/// for a dimension the dictionary actually names -- a chunk in a dimension
/// with no entry simply carries no `0x3f` at all, never a dangling one. See
/// the module docs' "When an entry would be needed" section.
#[test]
fn chunks_in_a_dimension_without_a_dictionary_entry_carry_no_0x3f_record() {
    use std::io::Cursor;

    // (world, dimensions the dictionary names) -> every other dimension
    // with real chunks in that world must show zero 0x3f records.
    let dictionary_named_dimensions = std::collections::HashMap::from([
        ("v1_18_30", vec![Dimension::Overworld]),
        ("v1_19_30", vec![Dimension::Overworld]),
        ("v1_20_81", vec![Dimension::Overworld]),
        ("1_26_seed", vec![Dimension::Overworld, Dimension::End]),
    ]);

    let mut checked_worlds = 0;
    for (name, _tmp, db) in all_worlds() {
        let Some(named) = dictionary_named_dimensions.get(name.as_str()) else {
            continue;
        };

        let mut chunks_without_entry_that_have_0x3f: Vec<(Dimension, i32, i32)> = Vec::new();
        let mut saw_a_chunk_outside_named_dimensions = false;
        let mut keys = db.keys().unwrap();
        for kv in &mut keys {
            let key_bytes = kv.key();
            let mut cursor = Cursor::new(key_bytes.as_ref());
            let Ok(key) = Key::deserialize(&mut cursor) else {
                continue;
            };
            if named.contains(&key.dimension) {
                continue;
            }
            saw_a_chunk_outside_named_dimensions = true;
            if key.data == KeyVariant::MetadataDictionaryKey {
                chunks_without_entry_that_have_0x3f.push((key.dimension, key.chunk.0, key.chunk.1));
            }
        }

        assert!(
            chunks_without_entry_that_have_0x3f.is_empty(),
            "{name}: found 0x3f records for a dimension the dictionary has no entry for: {chunks_without_entry_that_have_0x3f:?}"
        );
        if saw_a_chunk_outside_named_dimensions {
            checked_worlds += 1;
        }
    }
    // Three of the four worlds (v1_18_30, v1_19_30, v1_20_81) have a real
    // Nether and/or End chunk despite no matching dictionary entry --
    // that's the corpus fact this test is built on. 1_26_seed has no
    // Nether chunks at all, so it never exercises this path even though it
    // is still scanned above (for the dangling-0x3f assertion).
    assert_eq!(
        checked_worlds, 3,
        "expected exactly three worlds to have a chunk outside its dictionary's named dimensions"
    );
}
