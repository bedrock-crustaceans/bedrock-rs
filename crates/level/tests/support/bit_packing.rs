//! Shared bit-packing helpers for property tests that build canonical
//! paletted storage bytes by hand -- the `0x2f` SubChunk block/liquid layer
//! packer (`crates/level/src/subchunk.rs`/`bits.rs`/`greedy.rs`) and the
//! `0x2b` Biomes3d packer (`crates/level/src/biome.rs`) both reuse the same
//! `GreedyArray::pack_into` packing rule, just wrapped in a different header
//! convention (block/liquid layers leave the header's low bit 0; biome
//! fragments force it to 1 -- see `biome.rs`'s own `BIOME_HEADER_LOW_BIT`
//! doc comment). This module gives both callers one copy of "how to pack
//! indices at a given bit width" and "which width a given palette size
//! needs", reproduced here (not `pub` in the library) from `bits.rs`'s and
//! `greedy.rs`'s doc comments, since these are external integration-test
//! crates and cannot reach the private helpers directly.

/// Every valid bits-per-index width `BitArray::VALID_BITS` accepts.
pub const VALID_BITS: [u8; 8] = [1, 2, 3, 4, 5, 6, 8, 16];

/// The minimal valid bit width that can address `palette_size` distinct
/// entries -- the same rule `BitArray::to_disk` applies when choosing which
/// width to write.
pub fn minimal_width(palette_size: usize) -> u8 {
    VALID_BITS
        .into_iter()
        .find(|&b| 2usize.pow(u32::from(b)) >= palette_size)
        .unwrap_or(16)
}

/// Packs `indices` (values in `0..palette_size`) at `bits` per index, the
/// same way `GreedyArray::pack_into` does: `32 / bits` indices per
/// little-endian `u32` word, high bits of a partial final word left zero
/// (the canonicalization carve-out for widths 3/5/6, whose packed words
/// don't divide 4096 evenly).
pub fn pack_indices(indices: &[u16], bits: u8) -> Vec<u8> {
    let per_word = 32u32 / u32::from(bits);
    let word_count = 4096u32.div_ceil(per_word) as usize;
    let mut words = vec![0u32; word_count];

    let mut offset = 0usize;
    for word in &mut words {
        let mut w = 0u32;
        for slot in 0..per_word {
            if offset == indices.len() {
                break;
            }
            w |= u32::from(indices[offset]) << (slot * u32::from(bits));
            offset += 1;
        }
        *word = w;
    }

    let mut out = Vec::with_capacity(word_count * 4);
    for word in words {
        out.extend_from_slice(&word.to_le_bytes());
    }
    out
}

/// The palette size that first requires `VALID_BITS[bits_index]` (one more
/// than the previous width's capacity, `0` for the first width) -- the
/// lower boundary a width-sweep property checks.
pub fn lower_boundary(bits_index: usize) -> usize {
    if bits_index == 0 {
        1
    } else {
        2usize.pow(u32::from(VALID_BITS[bits_index - 1])) + 1
    }
}

/// The palette size `VALID_BITS[bits_index]` can address at most (`2^bits`)
/// -- the upper boundary a width-sweep property checks.
pub fn upper_boundary(bits_index: usize) -> usize {
    2usize.pow(u32::from(VALID_BITS[bits_index]))
}
