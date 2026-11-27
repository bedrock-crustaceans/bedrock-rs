//! Round-trip and malformed-input property tests for `0x2f` `SubChunk` records
//! (`crates/level/src/subchunk.rs`): every subchunk version (1, the
//! non-paletted family 0/2-7, 8, 9), every bit width (including the zero-bit
//! layout) at both unpacking strategies (`Greedy`/`Lazy`), and every palette
//! entry form (`Modern`, `Legacy`; `Numeric` is exercised through the
//! non-paletted layout, which is the only path that produces it).
//!
//! Wire format reproduced here from `subchunk.rs`'s own module docs and its
//! in-crate unit tests (`subchunk_with_palette`, `nonpaletted_bytes`, etc.),
//! since this is an external test crate and cannot reach those private test
//! helpers directly.
//!
//! # Canonicalization carve-out
//!
//! `BitArray::to_disk` always writes the *minimal* valid bit width for the
//! actual palette size, and zeros the unused high bits of a partially-filled
//! final word at widths 3/5/6 (`GreedyArray::pack_into`'s doc comment).
//! Generators here always build the minimal width for the palette size they
//! choose, and always pack indices with zeroed padding, so the identity
//! properties below hold byte-for-byte. A real record with a non-minimal
//! width or non-zero padding is known (and separately tested, in-crate, by
//! `bit_layers.rs`'s `packed_index_padding_bits_are_not_always_zero`) to
//! decode fine but not re-encode byte-identically -- that is documented
//! non-identity, not something these properties assert.

pub mod support;

use std::io::Cursor;

use bedrock_level::subchunk::{SubChunk, SubChunkVersion};
use bedrock_level::{Greedy, Lazy};
use proptest::prelude::*;
use support::bit_packing::{
    VALID_BITS, lower_boundary, minimal_width, pack_indices, upper_boundary,
};

/// A modern palette entry's NBT bytes: `{name, version?, states}` (or
/// `{name, states, version}`, matching `FieldOrder`), `states` an empty
/// compound. Varying `version`/`states_first` alone is enough to exercise
/// both real on-disk field orders (`FieldOrder::of` in `subchunk.rs`).
fn modern_entry_bytes(name: &str, version: Option<i32>, states_first: bool) -> Vec<u8> {
    let mut c = nbtx::Compound::new();
    c.insert("name".into(), nbtx::Value::String(name.into()));
    let version_entry = version.map(nbtx::Value::Int);
    if states_first {
        c.insert(
            "states".into(),
            nbtx::Value::Compound(nbtx::Compound::new()),
        );
        if let Some(v) = version_entry {
            c.insert("version".into(), v);
        }
    } else {
        if let Some(v) = version_entry {
            c.insert("version".into(), v);
        }
        c.insert(
            "states".into(),
            nbtx::Value::Compound(nbtx::Compound::new()),
        );
    }
    nbtx::to_le_bytes(&nbtx::Value::Compound(c)).unwrap()
}

/// A legacy `{name, val}` palette entry's NBT bytes.
fn legacy_entry_bytes(name: &str, val: i16) -> Vec<u8> {
    let c = nbtx::Compound::from_iter([
        ("name".into(), nbtx::Value::String(name.into())),
        ("val".into(), nbtx::Value::Short(val)),
    ]);
    nbtx::to_le_bytes(&nbtx::Value::Compound(c)).unwrap()
}

/// One palette entry to build into a layer: either the modern or the
/// pre-flattening legacy form.
#[derive(Clone, Debug)]
enum EntrySpec {
    Modern {
        name: String,
        version: Option<i32>,
        states_first: bool,
    },
    Legacy {
        name: String,
        val: i16,
    },
}

impl EntrySpec {
    fn bytes(&self) -> Vec<u8> {
        match self {
            EntrySpec::Modern {
                name,
                version,
                states_first,
            } => modern_entry_bytes(name, *version, *states_first),
            EntrySpec::Legacy { name, val } => legacy_entry_bytes(name, *val),
        }
    }
}

fn entry_spec() -> impl Strategy<Value = EntrySpec> {
    let name = "[a-z]{1,8}".prop_map(|s| format!("minecraft:{s}"));
    prop_oneof![
        (
            name.clone(),
            proptest::option::of(any::<i32>()),
            any::<bool>()
        )
            .prop_map(|(name, version, states_first)| EntrySpec::Modern {
                name,
                version,
                states_first,
            }),
        (name, any::<i16>()).prop_map(|(name, val)| EntrySpec::Legacy { name, val }),
    ]
}

/// Builds one canonical paletted layer's raw bytes: header byte (bit width
/// or the zero-bit sentinel), the packed index array (unless zero-bit), the
/// palette-length word (unless zero-bit), then each entry's NBT.
///
/// `indices` selects a palette entry (by index into `entries`) for every one
/// of the 4096 positions -- always a single repeated pattern here (position
/// `i % entries.len()`), which is enough to exercise every entry at least
/// once without needing a separate `Vec<u16>` strategy per case.
fn layer_bytes(entries: &[EntrySpec]) -> Vec<u8> {
    let mut out = Vec::new();
    if entries.len() == 1 {
        // Zero-bit layout: header byte 0, then the single entry directly,
        // no index words and no palette-length word.
        out.push(0);
        out.extend(entries[0].bytes());
        return out;
    }

    let bits = minimal_width(entries.len());
    out.push(bits << 1);

    let indices: Vec<u16> = (0..4096u32)
        .map(|i| u16::try_from(i as usize % entries.len()).unwrap())
        .collect();
    out.extend(pack_indices(&indices, bits));

    out.extend_from_slice(&u32::try_from(entries.len()).unwrap().to_le_bytes());
    for entry in entries {
        out.extend(entry.bytes());
    }
    out
}

/// 1 to 4 entries, canonically packed -- covers the zero-bit layout (1
/// entry) and every non-trivial minimal width up through 2 bits
/// (`minimal_width(2..=4) == 2`). Kept small so every `SubChunkVersion`
/// case in the property below stays fast; `bit_width_property` separately
/// sweeps every `VALID_BITS` width at a fixed entry count.
fn small_palette() -> impl Strategy<Value = Vec<EntrySpec>> {
    proptest::collection::vec(entry_spec(), 1..=4)
}

/// Exactly `n` distinct-by-content entries, forcing `minimal_width(n)` to
/// actually govern the header -- used to sweep every `VALID_BITS` value at
/// its own minimal palette size.
fn palette_of_size(n: usize) -> impl Strategy<Value = Vec<EntrySpec>> {
    proptest::collection::vec(entry_spec(), n..=n)
}

/// Builds a full canonical paletted `SubChunk` record: version byte,
/// layer-count byte (skipped for `Legacy`), subchunk index (`Limitless`
/// only), then each layer's bytes.
fn paletted_subchunk_bytes(
    version: SubChunkVersion,
    layers: &[Vec<EntrySpec>],
    index: i8,
) -> Vec<u8> {
    let mut out = vec![version.raw()];
    if version != SubChunkVersion::Legacy {
        out.push(u8::try_from(layers.len()).unwrap());
    }
    if version == SubChunkVersion::Limitless {
        out.push(index.cast_unsigned());
    }
    for layer in layers {
        out.extend(layer_bytes(layer));
    }
    out
}

fn paletted_version() -> impl Strategy<Value = SubChunkVersion> {
    prop_oneof![
        Just(SubChunkVersion::Legacy),
        Just(SubChunkVersion::Limited),
        Just(SubChunkVersion::Limitless),
    ]
}

proptest! {
    /// Every paletted `SubChunkVersion` (1, 8, 9) round-trips byte-
    /// identically for a canonical one-or-two-layer record built from a
    /// small mixed modern/legacy palette. `Legacy` is forced to exactly one
    /// layer (it carries no layer-count byte and the real format never has
    /// more than one); `Limited`/`Limitless` allow up to two (the
    /// block/liquid convention).
    #[test]
    fn paletted_subchunk_round_trips_byte_identical(
        version in paletted_version(),
        layer_count in 1usize..=2,
        layer_seed in proptest::collection::vec(small_palette(), 1..=2),
        index in any::<i8>(),
    ) {
        let layer_count = if version == SubChunkVersion::Legacy { 1 } else { layer_count };
        let layers: Vec<_> = layer_seed.into_iter().take(layer_count.max(1)).collect();
        let layers = if layers.is_empty() { vec![vec![EntrySpec::Legacy { name: "minecraft:air".into(), val: 0 }]] } else { layers };

        let bytes = paletted_subchunk_bytes(version, &layers, index);

        let chunk = SubChunk::from_disk::<Greedy, _>(&mut Cursor::new(bytes.as_slice())).unwrap();
        prop_assert_eq!(chunk.version(), version);

        let mut out = Cursor::new(Vec::new());
        chunk.to_disk::<Greedy, _>(&mut out).unwrap();
        prop_assert_eq!(out.into_inner(), bytes.clone());

        // Lazy unpacking must decode to an equal value (not necessarily
        // re-encoding through the same code path) and re-encode
        // byte-identically too.
        let lazy_chunk = SubChunk::from_disk::<Lazy, _>(&mut Cursor::new(bytes.as_slice())).unwrap();
        let mut lazy_out = Cursor::new(Vec::new());
        lazy_chunk.to_disk::<Lazy, _>(&mut lazy_out).unwrap();
        prop_assert_eq!(lazy_out.into_inner(), bytes);
    }

    /// Every non-paletted version byte (0, 2-7) round-trips byte-identically
    /// for an arbitrary id/nibble column -- there is no invalid encoding
    /// here short of truncation, since every byte value is a legal id or
    /// nibble pair.
    #[test]
    fn nonpaletted_subchunk_round_trips_byte_identical(
        version in prop_oneof![Just(0u8), Just(2u8), Just(3u8), Just(4u8), Just(5u8), Just(6u8), Just(7u8)],
        ids in proptest::collection::vec(any::<u8>(), 4096),
        nibbles in proptest::collection::vec(any::<u8>(), 2048),
    ) {
        let mut bytes = vec![version];
        bytes.extend(ids);
        bytes.extend(nibbles);

        let chunk = SubChunk::from_disk::<Greedy, _>(&mut Cursor::new(bytes.as_slice())).unwrap();
        prop_assert_eq!(chunk.version(), SubChunkVersion::NonPaletted(version));

        let mut out = Cursor::new(Vec::new());
        chunk.to_disk::<Greedy, _>(&mut out).unwrap();
        prop_assert_eq!(out.into_inner(), bytes);
    }

    /// Arbitrary bytes of any length must never panic `SubChunk::from_disk`
    /// under either unpacking strategy -- decision 11: malformed input is an
    /// `Err`, never a panic.
    #[test]
    fn from_disk_never_panics_on_arbitrary_bytes(bytes in proptest::collection::vec(any::<u8>(), 0..600)) {
        let _ = SubChunk::from_disk::<Greedy, _>(&mut Cursor::new(bytes.as_slice()));
        let _ = SubChunk::from_disk::<Lazy, _>(&mut Cursor::new(bytes.as_slice()));
    }

    /// A truncated well-formed paletted record must never panic, however
    /// far into the record the cut lands (mid-header, mid-index-array,
    /// mid-palette-NBT).
    #[test]
    fn from_disk_never_panics_on_truncated_paletted_records(
        version in paletted_version(),
        layer in palette_of_size(3),
        cut in 0usize..200,
    ) {
        let bytes = paletted_subchunk_bytes(version, &[layer], 0);
        let truncated = &bytes[..cut.min(bytes.len())];
        let _ = SubChunk::from_disk::<Greedy, _>(&mut Cursor::new(truncated));
    }

    /// A truncated non-paletted record must never panic either.
    #[test]
    fn from_disk_never_panics_on_truncated_nonpaletted_records(
        cut in 0usize..4097,
    ) {
        let mut bytes = vec![0u8];
        bytes.extend(std::iter::repeat_n(0u8, 4096 + 2048));
        let truncated = &bytes[..cut.min(bytes.len())];
        let _ = SubChunk::from_disk::<Greedy, _>(&mut Cursor::new(truncated));
    }

    /// An invalid subchunk version byte (anything not in `{0..=9}`, per
    /// `SubChunkVersion::try_from`) is rejected rather than panicking or
    /// silently defaulting.
    #[test]
    fn invalid_version_byte_errors_not_panics(byte in 10u8..=255) {
        let bytes = vec![byte];
        prop_assert!(SubChunk::from_disk::<Greedy, _>(&mut Cursor::new(bytes.as_slice())).is_err());
    }
}

/// Builds a canonical `Limited` (single-layer) subchunk record whose sole
/// layer's palette is exactly `n` distinct legacy entries, and checks it
/// round-trips byte-identically at `expected_bits`.
fn assert_width_round_trips(n: usize, expected_bits: u8) {
    let entries: Vec<EntrySpec> = (0..n)
        .map(|i| EntrySpec::Legacy {
            name: format!("minecraft:e{i}"),
            val: i16::try_from(i % 16).unwrap(),
        })
        .collect();
    assert_eq!(minimal_width(entries.len()), expected_bits, "n={n}");

    let bytes = paletted_subchunk_bytes(SubChunkVersion::Limited, &[entries], 0);
    let chunk = SubChunk::from_disk::<Greedy, _>(&mut Cursor::new(bytes.as_slice())).unwrap();

    let mut out = Cursor::new(Vec::new());
    chunk.to_disk::<Greedy, _>(&mut out).unwrap();
    assert_eq!(out.into_inner(), bytes, "n={n} bits={expected_bits}");
}

/// Every valid bit width (1, 2, 3, 4, 5, 6, 8, 16) round-trips byte-
/// identically at its own minimal palette size, canonically packed --
/// specifically exercises 3/5/6, the widths whose packed words don't divide
/// 4096 evenly and therefore carry a partially-filled final word. Checked at
/// both the width's lower boundary (`prev_capacity + 1`, the smallest
/// palette size that actually needs it) and, for every width of 3 bits or
/// more, its upper boundary (`2^bits` entries exactly) -- widths 1 and 2
/// skip the upper check since their capacities (2, 4) are already covered
/// densely by `paletted_subchunk_round_trips_byte_identical`'s small mixed
/// palettes elsewhere in this file.
///
/// Written as a plain exhaustive loop rather than a `proptest!` property:
/// the domain is exactly the 8 `VALID_BITS` values, so sampling it
/// randomly buys nothing a full enumeration doesn't already give for less
/// (and much more predictable) cost -- width 16's upper boundary alone
/// builds a 65,536-entry palette, which a proptest case count would either
/// hit rarely (weak coverage) or repeatedly (wasted time).
#[test]
fn every_valid_bit_width_round_trips_byte_identical_at_both_boundaries() {
    for (bits_index, &bits) in VALID_BITS.iter().enumerate() {
        assert_width_round_trips(lower_boundary(bits_index), bits);
        if bits >= 3 {
            assert_width_round_trips(upper_boundary(bits_index), bits);
        }
    }
}
