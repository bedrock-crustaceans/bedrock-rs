use std::io::{Cursor, Read, Seek, Write};

use bedrock_shared::read::SeekExt;

use crate::biome::{Biomes2d, COLUMNS, column_index};
use crate::error::{Error, Result};

/// Size in bytes of the heightmap half of the payload: 256 little-endian `i16` values.
const HEIGHTS_SIZE: usize = COLUMNS * size_of::<i16>();
/// Size in bytes of the biome half of the payload: one id per column.
const BIOMES_SIZE: usize = COLUMNS;
const PAYLOAD_SIZE: usize = HEIGHTS_SIZE + BIOMES_SIZE;

/// Pre-1.18 per-chunk column data (`LevelDB` key tag `0x2d`).
///
/// Holds a 16x16 column heightmap paired with a single biome id per column
/// ([`Biomes2d`]). Chunks using this record predate 3D biomes (`0x2b`),
/// where every column has one biome for its whole height. Both halves share
/// the same column order: for column `(x, z)` the flat index is
/// `z * 16 + x`, so the 16 `x` values for a given `z` are contiguous.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeightMap {
    heights: Box<[i16; COLUMNS]>,
    biomes: Biomes2d,
}

impl HeightMap {
    /// The column heightmap, in `z * 16 + x` order.
    #[inline]
    pub fn heights(&self) -> &[i16; COLUMNS] {
        &self.heights
    }

    /// The height of column `(x, z)`, or `None` if either coordinate is
    /// outside `0..16`.
    #[inline]
    pub fn height_at(&self, x: usize, z: usize) -> Option<i16> {
        column_index(x, z).map(|i| self.heights[i])
    }

    /// Replaces the whole heightmap in one call -- the write-back counterpart to
    /// [`Self::heights`]' shared borrow, for a caller (a heightmap recompute, architecture
    /// decision 13) that has a fresh `[i16; COLUMNS]` to install rather than editing
    /// columns one at a time.
    #[inline]
    pub fn set_heights(&mut self, heights: [i16; COLUMNS]) {
        *self.heights = heights;
    }

    /// Mutable access to this record's heightmap, for an in-place partial update that
    /// [`Self::set_heights`]' whole-array replacement does not fit.
    #[inline]
    pub fn heights_mut(&mut self) -> &mut [i16; COLUMNS] {
        &mut self.heights
    }

    /// The per-column biome ids, in the same layout as [`Self::heights`].
    #[inline]
    pub fn biomes(&self) -> &Biomes2d {
        &self.biomes
    }

    pub const fn size_hint(&self) -> usize {
        PAYLOAD_SIZE
    }

    /// Writes the 256 heights followed by the 256 biome ids.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to `writer` fails.
    pub fn to_disk<W>(&self, writer: &mut Cursor<W>) -> Result<()>
    where
        Cursor<W>: Write,
    {
        writer.write_all(bytemuck::cast_slice::<i16, u8>(self.heights.as_slice()))?;
        self.biomes.to_disk(writer)?;

        Ok(())
    }

    /// Reads a `0x2d` record: 256 heights followed by 256 biome ids.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Invalid`] if the bytes remaining in `reader` are not
    /// exactly the 768-byte payload, or an error if reading from `reader`
    /// fails.
    pub fn from_disk<R>(reader: &mut Cursor<R>) -> Result<HeightMap>
    where
        Cursor<R>: Read + Seek,
    {
        // The record is a fixed 768-byte pair (heightmap, biomes); nothing legitimately
        // truncates or extends it, so any other length is malformed data.
        let remaining = reader.stream_len_ext()? - reader.stream_position()?;
        if remaining != PAYLOAD_SIZE as u64 {
            return Err(Error::Invalid("0x2d payload size"));
        }

        let mut heights: Box<[i16; COLUMNS]> = Box::new([0; COLUMNS]);
        reader.read_exact(bytemuck::cast_slice_mut::<i16, u8>(heights.as_mut()))?;

        let biomes = Biomes2d::from_disk(reader)?;

        Ok(HeightMap { heights, biomes })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> HeightMap {
        let mut heights = Box::new([0i16; COLUMNS]);
        let mut ids = Box::new([0u8; COLUMNS]);
        for (i, (h, b)) in heights.iter_mut().zip(ids.iter_mut()).enumerate() {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_possible_wrap,
                reason = "i indexes one of the 256 columns"
            )]
            let i16_index = i as i16;
            *h = i16_index * 3 - 128;
            #[expect(clippy::cast_possible_truncation, reason = "reduced modulo 256 first")]
            let id = (i % 256) as u8;
            *b = id;
        }

        HeightMap {
            heights,
            biomes: ids.into(),
        }
    }

    #[test]
    fn round_trips_through_disk() {
        let original = sample();

        let mut buf = Cursor::new(Vec::new());
        original.to_disk(&mut buf).unwrap();
        let bytes = buf.into_inner();

        assert_eq!(bytes.len(), PAYLOAD_SIZE);

        let mut reader = Cursor::new(bytes.as_slice());
        let decoded = HeightMap::from_disk(&mut reader).unwrap();

        assert_eq!(decoded, original);
    }

    #[test]
    fn rejects_truncated_payload() {
        let mut buf = Cursor::new(Vec::new());
        sample().to_disk(&mut buf).unwrap();
        let mut bytes = buf.into_inner();
        bytes.truncate(HEIGHTS_SIZE);

        let mut reader = Cursor::new(bytes.as_slice());
        assert!(HeightMap::from_disk(&mut reader).is_err());
    }

    #[test]
    fn rejects_oversized_payload() {
        let mut buf = Cursor::new(Vec::new());
        sample().to_disk(&mut buf).unwrap();
        let mut bytes = buf.into_inner();
        bytes.push(0);

        let mut reader = Cursor::new(bytes.as_slice());
        assert!(HeightMap::from_disk(&mut reader).is_err());
    }

    #[test]
    fn height_at_matches_z_major_flat_index() {
        let map = sample();

        for z in 0..16usize {
            for x in 0..16usize {
                assert_eq!(map.height_at(x, z), Some(map.heights()[z * 16 + x]));
            }
        }
    }

    #[test]
    fn height_at_rejects_out_of_range_coordinates() {
        let map = sample();

        assert_eq!(map.height_at(16, 0), None);
        assert_eq!(map.height_at(0, 16), None);
        assert_eq!(map.height_at(16, 16), None);
    }
}
