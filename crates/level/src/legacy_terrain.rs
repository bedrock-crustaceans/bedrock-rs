//! `0x30` LegacyTerrain: the pre-subchunk terrain record, one blob per chunk
//! covering the whole 16x16x128 column rather than one `0x2f` record per
//! vertical slice. Written by chunk versions 0 through 2, before terrain
//! moved to per-subchunk records at version 3.

use std::io::{Cursor, Read, Write};

use crate::error::Result;
use crate::subchunk::{Layer, decode_nonpaletted_column, encode_nonpaletted_column};

/// The whole column's height in blocks.
pub const COLUMN_HEIGHT: usize = 128;

/// The number of vertical 16-block slices a column splits into.
pub const SLICE_COUNT: usize = COLUMN_HEIGHT / 16;

/// The length, in bytes, of the portion of a `0x30` record this type actually decodes:
/// `16*16*128` raw block ids followed by `16*16*128/2` metadata nibbles.
pub const ESTABLISHED_LEN: usize = 16 * 16 * COLUMN_HEIGHT + 16 * 16 * COLUMN_HEIGHT / 2;

/// A decoded `0x30` LegacyTerrain record.
///
/// Only [`ESTABLISHED_LEN`] bytes are actually decoded: 32,768 raw block ids followed by
/// 16,384 metadata nibbles, in the same X-major/then-Z/then-Y-fastest order and even-low/
/// odd-high nibble packing as a non-paletted `0x2f` subchunk (see
/// [`crate::subchunk::SubChunkVersion::NonPaletted`]), split into eight 16-block vertical
/// [`Layer`]s. Everything after that -- block-light and sky-light nibble arrays, the
/// heightmap, and the biome bytes are known to follow, in some order, but neither reference
/// consulted for this decoder reads past the established bytes, so their exact sizes and
/// order are not established here. That trailing region is kept as opaque
/// [`tail`](Self::tail) bytes rather than parsed, and [`to_disk`](Self::to_disk) writes it
/// back out verbatim -- a decode-then-encode round trip is byte-identical, including the
/// tail, without this type knowing what the tail means.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTerrain {
    /// The eight 16-block vertical slices, lowest first.
    slices: [Layer; SLICE_COUNT],
    /// Everything after the established 49,152 bytes, preserved verbatim and unparsed.
    tail: Vec<u8>,
}

impl LegacyTerrain {
    /// The eight vertical 16-block slices, lowest first.
    pub fn slices(&self) -> &[Layer; SLICE_COUNT] {
        &self.slices
    }

    /// The unparsed bytes following the established region -- see this type's doc comment.
    pub fn tail(&self) -> &[u8] {
        &self.tail
    }

    /// Decodes a `0x30` record's value. `reader` is read to the end: everything past the
    /// established 49,152 bytes becomes [`tail`](Self::tail), whatever its length.
    pub fn from_disk<R>(reader: &mut Cursor<R>) -> Result<Self>
    where
        Cursor<R>: Read,
    {
        let slices = decode_nonpaletted_column(reader, COLUMN_HEIGHT)?
            .try_into()
            .unwrap_or_else(|_| {
                unreachable!("decode_nonpaletted_column(_, 128) always returns 8 layers")
            });

        let mut tail = Vec::new();
        reader.read_to_end(&mut tail)?;

        Ok(Self { slices, tail })
    }

    /// Encodes this record's value, reproducing the original bytes exactly for a value that
    /// came from [`from_disk`](Self::from_disk).
    pub fn to_disk<W>(&self, writer: &mut Cursor<W>) -> Result<()>
    where
        Cursor<W>: Write,
    {
        encode_nonpaletted_column(writer, &self.slices)?;
        writer.write_all(&self.tail)?;
        Ok(())
    }
}
