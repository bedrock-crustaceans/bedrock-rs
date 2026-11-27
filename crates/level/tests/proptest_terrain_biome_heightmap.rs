//! Round-trip and malformed-input property tests for `0x30` `LegacyTerrain`,
//! `0x2b` Biomes3d, the `0x2d` HeightMap/Biomes2d pair
//! (`crates/level/src/legacy_terrain.rs`, `biome.rs`, `height_map.rs`).

mod support;

use std::io::Cursor;

use bedrock_level::biome::{Biomes2d, Biomes3d, COLUMNS};
use bedrock_level::height_map::HeightMap;
use bedrock_level::legacy_terrain::{ESTABLISHED_LEN, LegacyTerrain};
use bedrock_level::{Greedy, Lazy};
use proptest::prelude::*;
use support::bit_packing::{
    VALID_BITS, lower_boundary, minimal_width, pack_indices, upper_boundary,
};

// ---------------------------------------------------------------------
// LegacyTerrain (0x30)
// ---------------------------------------------------------------------

proptest! {
    // Each case allocates and decodes a ~49KB column; a smaller case count
    // keeps this test's runtime proportionate to what it is actually
    // checking (byte-for-byte passthrough, not a combinatorial space).
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// Any bytes of at least `ESTABLISHED_LEN` round-trip byte-identically:
    /// the established region has no invalid encoding (every id/nibble byte
    /// value is legal), and everything past it is preserved verbatim as
    /// `tail`, whatever it contains.
    #[test]
    fn legacy_terrain_round_trips_byte_identical(
        established in proptest::collection::vec(any::<u8>(), ESTABLISHED_LEN),
        tail in proptest::collection::vec(any::<u8>(), 0..64),
    ) {
        let mut bytes = established;
        bytes.extend(&tail);

        let decoded = LegacyTerrain::from_disk(&mut Cursor::new(bytes.as_slice())).unwrap();
        prop_assert_eq!(decoded.tail(), tail.as_slice());

        let mut out = Cursor::new(Vec::new());
        decoded.to_disk(&mut out).unwrap();
        prop_assert_eq!(out.into_inner(), bytes);
    }

    /// A record shorter than the established region must error, never
    /// panic -- `decode_nonpaletted_column` reads a fixed-size id array then
    /// a fixed-size nibble array, so any shortfall is a read past EOF.
    #[test]
    fn legacy_terrain_never_panics_on_short_input(
        bytes in proptest::collection::vec(any::<u8>(), 0..ESTABLISHED_LEN),
    ) {
        let _ = LegacyTerrain::from_disk(&mut Cursor::new(bytes.as_slice()));
    }
}

// ---------------------------------------------------------------------
// Biomes2d (tail half of 0x2d)
// ---------------------------------------------------------------------

proptest! {
    /// Exactly `COLUMNS` bytes round-trip byte-identically -- every byte
    /// value is a legal (if not necessarily meaningful) biome id.
    #[test]
    fn biomes2d_round_trips_byte_identical(ids in proptest::collection::vec(any::<u8>(), COLUMNS)) {
        let mut cursor = Cursor::new(ids.as_slice());
        let decoded = Biomes2d::from_disk(&mut cursor).unwrap();

        let mut out = Cursor::new(Vec::new());
        decoded.to_disk(&mut out).unwrap();
        prop_assert_eq!(out.into_inner(), ids);
    }

    /// Fewer than `COLUMNS` bytes errors (a short read), never panics.
    #[test]
    fn biomes2d_never_panics_on_short_input(bytes in proptest::collection::vec(any::<u8>(), 0..COLUMNS)) {
        let _ = Biomes2d::from_disk(&mut Cursor::new(bytes.as_slice()));
    }
}

// ---------------------------------------------------------------------
// HeightMap (0x2d)
// ---------------------------------------------------------------------

fn heightmap_bytes(heights: &[i16], biome_ids: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(COLUMNS * 2 + COLUMNS);
    for h in heights {
        out.extend_from_slice(&h.to_le_bytes());
    }
    out.extend_from_slice(biome_ids);
    out
}

proptest! {
    /// The fixed 768-byte (`heights` + `biomes`) payload round-trips
    /// byte-identically for any content -- every `i16` height and every `u8`
    /// biome id is a legal encoding.
    #[test]
    fn heightmap_round_trips_byte_identical(
        heights in proptest::collection::vec(any::<i16>(), COLUMNS),
        biome_ids in proptest::collection::vec(any::<u8>(), COLUMNS),
    ) {
        let bytes = heightmap_bytes(&heights, &biome_ids);
        let decoded = HeightMap::from_disk(&mut Cursor::new(bytes.as_slice())).unwrap();

        let mut out = Cursor::new(Vec::new());
        decoded.to_disk(&mut out).unwrap();
        prop_assert_eq!(out.into_inner(), bytes);
    }

    /// Any length other than the exact 768-byte payload size is malformed
    /// (`HeightMap::from_disk` checks `stream_len_ext` against
    /// `PAYLOAD_SIZE` up front), never a panic, and never a truncated or
    /// padded decode.
    #[test]
    fn heightmap_never_panics_and_rejects_wrong_length(
        bytes in proptest::collection::vec(any::<u8>(), 0..(COLUMNS * 2 + COLUMNS) * 2),
    ) {
        let result = HeightMap::from_disk(&mut Cursor::new(bytes.as_slice()));
        if bytes.len() != COLUMNS * 2 + COLUMNS {
            prop_assert!(result.is_err());
        }
    }
}

// ---------------------------------------------------------------------
// Biomes3d (0x2b)
// ---------------------------------------------------------------------

const INHERIT_FLAG: u8 = 0x7f;
const EMPTY_FLAG: u8 = 0x00;
const BIOME_HEADER_LOW_BIT: u8 = 1;

#[derive(Clone, Debug)]
enum FragmentSpec {
    Inherit,
    Single(u32),
    /// A palette of `values.len()` entries, canonically packed at the
    /// minimal width for that size (mirrors `proptest_subchunk.rs`'s
    /// `minimal_width`/`pack_indices` -- biome storage reuses the same
    /// `BitArray` packer, just with the header's low bit forced to 1
    /// instead of 0).
    Palette(Vec<u32>),
}

impl FragmentSpec {
    fn write(&self, out: &mut Vec<u8>) {
        match self {
            FragmentSpec::Inherit => out.push((INHERIT_FLAG << 1) | BIOME_HEADER_LOW_BIT),
            FragmentSpec::Single(v) => {
                out.push((EMPTY_FLAG << 1) | BIOME_HEADER_LOW_BIT);
                out.extend_from_slice(&v.to_le_bytes());
            }
            FragmentSpec::Palette(values) => {
                let bits = minimal_width(values.len());
                let mut header = vec![bits << 1];
                header[0] |= BIOME_HEADER_LOW_BIT;
                out.extend(header);

                let indices: Vec<u16> = (0..4096u32)
                    .map(|i| (i as usize % values.len()) as u16)
                    .collect();
                out.extend(pack_indices(&indices, bits));

                out.extend_from_slice(&(values.len() as u32).to_le_bytes());
                for v in values {
                    out.extend_from_slice(&v.to_le_bytes());
                }
            }
        }
    }
}

fn fragment_spec() -> impl Strategy<Value = FragmentSpec> {
    prop_oneof![
        Just(FragmentSpec::Inherit),
        any::<u32>().prop_map(FragmentSpec::Single),
        proptest::collection::vec(any::<u32>(), 1..=4).prop_map(FragmentSpec::Palette),
    ]
}

fn biomes3d_bytes(heightmap: &[u16], fragments: &[FragmentSpec]) -> Vec<u8> {
    let mut out = Vec::with_capacity(COLUMNS * 2);
    for h in heightmap {
        out.extend_from_slice(&h.to_le_bytes());
    }
    for fragment in fragments {
        fragment.write(&mut out);
    }
    out
}

proptest! {
    /// A heightmap plus 0 to 4 fragments (`Inherit`, `Single`, or a
    /// canonically-packed `Palette`) round-trips byte-identically, greedily
    /// unpacked.
    #[test]
    fn biomes3d_round_trips_byte_identical(
        heightmap in proptest::collection::vec(any::<u16>(), COLUMNS),
        fragments in proptest::collection::vec(fragment_spec(), 0..4),
    ) {
        let bytes = biomes3d_bytes(&heightmap, &fragments);
        let decoded = Biomes3d::from_disk::<Greedy, _>(&mut Cursor::new(bytes.as_slice())).unwrap();
        prop_assert_eq!(decoded.heightmap().as_slice(), heightmap.as_slice());
        prop_assert_eq!(decoded.fragments().len(), fragments.len());

        let mut out = Cursor::new(Vec::new());
        decoded.to_disk(&mut out).unwrap();
        prop_assert_eq!(out.into_inner(), bytes.clone());

        // Lazy unpacking decodes and re-encodes the same bytes too.
        let lazy_decoded = Biomes3d::from_disk::<Lazy, _>(&mut Cursor::new(bytes.as_slice())).unwrap();
        let mut lazy_out = Cursor::new(Vec::new());
        lazy_decoded.to_disk(&mut lazy_out).unwrap();
        prop_assert_eq!(lazy_out.into_inner(), bytes);
    }

    /// Arbitrary bytes never panic `Biomes3d::from_disk` -- a truncated
    /// heightmap or a corrupt fragment header must surface as `Err`
    /// (`BitArray::from_disk` returning `UnexpectedEof` ends the fragment
    /// loop cleanly; anything else propagates as an error) rather than a
    /// panic.
    #[test]
    fn biomes3d_never_panics_on_arbitrary_bytes(bytes in proptest::collection::vec(any::<u8>(), 0..600)) {
        let _ = Biomes3d::from_disk::<Greedy, _>(&mut Cursor::new(bytes.as_slice()));
        let _ = Biomes3d::from_disk::<Lazy, _>(&mut Cursor::new(bytes.as_slice()));
    }

    /// A truncated well-formed record must never panic either, whatever the
    /// cut point.
    #[test]
    fn biomes3d_never_panics_on_truncated_well_formed_records(
        heightmap in proptest::collection::vec(any::<u16>(), COLUMNS),
        fragments in proptest::collection::vec(fragment_spec(), 1..3),
        cut in 0usize..600,
    ) {
        let bytes = biomes3d_bytes(&heightmap, &fragments);
        let truncated = &bytes[..cut.min(bytes.len())];
        let _ = Biomes3d::from_disk::<Greedy, _>(&mut Cursor::new(truncated));
    }
}

/// Builds a single-fragment `Biomes3d` record whose palette is exactly `n`
/// distinct biome ids, and checks it round-trips byte-identically at
/// `expected_bits` -- the `Biomes3d` counterpart of
/// `proptest_subchunk.rs`'s `assert_width_round_trips`. `fragment_spec`'s
/// own generator caps a `Palette` at 4 entries (kept small so the
/// all-fragment-kinds property above stays fast), which never exercises a
/// width above 2 bits; this walks every `VALID_BITS` width directly instead,
/// at both its lower and (width >= 3) upper boundary, with the real header
/// low bit convention (`BIOME_HEADER_LOW_BIT`) `fragment_spec` also uses.
fn assert_biomes3d_width_round_trips(n: usize, expected_bits: u8) {
    let values: Vec<u32> = (0..n as u32).collect();
    assert_eq!(minimal_width(values.len()), expected_bits, "n={n}");

    let heightmap = vec![0u16; COLUMNS];
    let bytes = biomes3d_bytes(&heightmap, &[FragmentSpec::Palette(values)]);

    let decoded = Biomes3d::from_disk::<Greedy, _>(&mut Cursor::new(bytes.as_slice())).unwrap();
    let mut out = Cursor::new(Vec::new());
    decoded.to_disk(&mut out).unwrap();
    assert_eq!(out.into_inner(), bytes, "n={n} bits={expected_bits}");
}

/// Every valid bit width round-trips byte-identically for a `Biomes3d`
/// palette fragment, at both its lower boundary and (for widths of 3 bits
/// or more) its upper boundary -- see `proptest_subchunk.rs`'s identically-
/// shaped `every_valid_bit_width_round_trips_byte_identical_at_both_boundaries`
/// for why this is a plain exhaustive loop rather than a `proptest!`
/// property (a finite 8/14-case domain, one of which builds a 65,536-entry
/// palette).
#[test]
fn every_valid_bit_width_round_trips_byte_identical_at_both_boundaries() {
    for bits_index in 0..VALID_BITS.len() {
        let bits = VALID_BITS[bits_index];
        assert_biomes3d_width_round_trips(lower_boundary(bits_index), bits);
        if bits >= 3 {
            assert_biomes3d_width_round_trips(upper_boundary(bits_index), bits);
        }
    }
}
