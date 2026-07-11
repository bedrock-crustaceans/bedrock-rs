use std::io::{Cursor, Read, Seek, Write};

use bedrock_shared::read::SeekExt;

use crate::error::{Error, Result};

/// Column count of a 16x16 chunk.
const COLUMNS: usize = 256;
/// Size in bytes of the heightmap half of the payload: 256 little-endian `i16` values.
const HEIGHTS_SIZE: usize = COLUMNS * size_of::<i16>();
/// Size in bytes of the biome half of the payload: one id per column.
const BIOMES_SIZE: usize = COLUMNS;
const PAYLOAD_SIZE: usize = HEIGHTS_SIZE + BIOMES_SIZE;

/// Pre-1.18 per-chunk column data (LevelDB key tag `0x2d`).
///
/// Holds a 16x16 column heightmap paired with a single biome id per column, in the
/// same x/z order as the heightmap. Chunks using this record predate 3D biomes
/// (`0x2b`), where every column has one biome for its whole height.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeightMap {
    heights: Box<[i16; COLUMNS]>,
    biomes: Box<[u8; COLUMNS]>,
}

impl HeightMap {
    /// The column heightmap, in x/z order.
    #[inline]
    pub fn heights(&self) -> &[i16; COLUMNS] {
        &self.heights
    }

    /// The per-column biome ids, in the same x/z order as [`Self::heights`].
    #[inline]
    pub fn biomes(&self) -> &[u8; COLUMNS] {
        &self.biomes
    }

    pub const fn size_hint(&self) -> usize {
        PAYLOAD_SIZE
    }

    pub fn to_disk<W>(&self, writer: &mut Cursor<W>) -> Result<()>
    where
        Cursor<W>: Write,
    {
        writer.write_all(bytemuck::cast_slice::<i16, u8>(self.heights.as_slice()))?;
        writer.write_all(self.biomes.as_slice())?;

        Ok(())
    }

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

        let mut biomes: Box<[u8; COLUMNS]> = Box::new([0; COLUMNS]);
        reader.read_exact(biomes.as_mut())?;

        Ok(HeightMap { heights, biomes })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> HeightMap {
        let mut heights = Box::new([0i16; COLUMNS]);
        let mut biomes = Box::new([0u8; COLUMNS]);
        for (i, (h, b)) in heights.iter_mut().zip(biomes.iter_mut()).enumerate() {
            *h = (i as i16) * 3 - 128;
            *b = (i % 256) as u8;
        }

        HeightMap { heights, biomes }
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
}
