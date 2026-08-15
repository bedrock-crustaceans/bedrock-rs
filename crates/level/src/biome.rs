use std::io::{Cursor, Read};
use std::io::{ErrorKind, Write};

use byteorder::LittleEndian;
use byteorder::{ReadBytesExt, WriteBytesExt};
use smallvec::SmallVec;

use crate::UnpackingMethod;
use crate::bits::{BitArray, IndicesType};
use crate::error::Error;
use crate::error::Result;

/// Column count of a 16x16 chunk. Also the heightmap element count in both
/// on-disk layouts, and the biome id count in the 2D layout only — the 3D
/// layout's per-fragment biome grid is 4x4x4 (64), not this.
pub const COLUMNS: usize = 256;

/// Flat index of column `(x, z)` in a 16x16 chunk, or `None` if either
/// coordinate is outside `0..16`. Columns are stored in row-major order by
/// Z: the flat index is `z * 16 + x`, so the 16 `x` values for a given `z`
/// are contiguous.
#[inline]
pub const fn column_index(x: usize, z: usize) -> Option<usize> {
    if x < 16 && z < 16 {
        Some(z * 16 + x)
    } else {
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BiomeArray {
    array: BitArray,
    palette: Vec<u32>,
}

impl BiomeArray {
    pub fn palette(&self) -> &[u32] {
        &self.palette
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BiomeEncoding {
    /// Inherits all data from the previous subchunk
    Inherit,
    /// Chunk is a single biome
    Single(u32),
    /// Chunk contains multiple biomes
    Palette(BiomeArray),
}

/// Palette-based per-subchunk biome data (LevelDB key tag `0x2b`).
///
/// One [`BiomeEncoding`] fragment per subchunk, ordered bottom to top, each
/// covering that subchunk's 16x16x16 volume as a 4x4x4 grid of biome ids.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Biomes3d {
    heightmap: Box<[u16; COLUMNS]>,
    fragments: SmallVec<[BiomeEncoding; 1]>,
}

impl Biomes3d {
    /// This chunk's heightmap.
    #[inline]
    pub fn heightmap(&self) -> &[u16; COLUMNS] {
        &self.heightmap
    }

    /// The per-subchunk biome fragments, ordered bottom to top.
    #[inline]
    pub fn fragments(&self) -> &[BiomeEncoding] {
        &self.fragments
    }

    // This roughly estimates the size of the output buffer. Chunks with many paletted fragments
    // will be larger than this estimate while chunks with many inherited biome fragments will be smaller.
    pub fn size_hint(&self) -> usize {
        const HEIGHTMAP_SIZE: usize = COLUMNS * 2;
        HEIGHTMAP_SIZE + self.fragments.len() * std::mem::size_of::<BiomeEncoding>()
    }

    pub fn to_disk<W>(&self, writer: &mut Cursor<W>) -> Result<()>
    where
        Cursor<W>: Write,
    {
        const EMPTY_FLAG: u8 = 0x00;
        const INHERIT_FLAG: u8 = 0x7f;

        writer.write_all(bytemuck::cast_slice::<u16, u8>(self.heightmap.as_slice()))?;

        for fragment in &self.fragments {
            match fragment {
                BiomeEncoding::Inherit => writer.write_u8(INHERIT_FLAG << 1)?,
                BiomeEncoding::Single(v) => {
                    writer.write_u8(EMPTY_FLAG << 1)?;
                    writer.write_u32::<LittleEndian>(*v)?;
                }
                BiomeEncoding::Palette(v) => {
                    v.array.to_disk(writer, v.palette.len())?;
                    writer.write_u32::<LittleEndian>(v.palette.len() as u32)?;

                    for entry in &v.palette {
                        writer.write_u32::<LittleEndian>(*entry)?;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn from_disk<M: UnpackingMethod, R>(reader: &mut Cursor<R>) -> Result<Biomes3d>
    where
        Cursor<R>: Read,
    {
        let mut heightmap: Box<[u16; COLUMNS]> = Box::new([0; COLUMNS]);

        let heightmap_bytes = bytemuck::cast_slice_mut::<u16, u8>(heightmap.as_mut());
        reader.read_exact(heightmap_bytes)?;

        let mut fragments = SmallVec::new();
        loop {
            let indices = match BitArray::from_disk::<M, _>(reader) {
                Ok(indices) => indices,
                Err(err) => {
                    if let Error::IoError(io) = &err
                        && io.kind() == ErrorKind::UnexpectedEof
                    {
                        // We found the end of the fragment array, stop the loop
                        break;
                    }

                    // Something actually went wrong...
                    return Err(err);
                }
            };

            match indices {
                IndicesType::Data(array) => {
                    let len = reader.read_u32::<LittleEndian>()?;
                    let mut palette = Vec::with_capacity(len as usize);
                    for _ in 0..len {
                        palette.push(reader.read_u32::<LittleEndian>()?);
                    }

                    fragments.push(BiomeEncoding::Palette(BiomeArray { array, palette }));
                }
                IndicesType::Empty => {
                    let single = reader.read_u32::<LittleEndian>()?;
                    fragments.push(BiomeEncoding::Single(single))
                }
                IndicesType::Inherit => fragments.push(BiomeEncoding::Inherit),
            }
        }

        Ok(Biomes3d {
            heightmap,
            fragments,
        })
    }
}

/// Per-column biome ids predating per-subchunk biome data: the tail of the
/// `0x2d` payload, one byte per column, applying to that column's whole
/// height. Columns are stored in row-major order by Z — for column `(x, z)`
/// the flat index is `z * 16 + x`, so the 16 `x` values for a given `z` are
/// contiguous.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Biomes2d {
    ids: Box<[u8; COLUMNS]>,
}

impl Biomes2d {
    /// The per-column biome ids, in `z * 16 + x` order.
    #[inline]
    pub fn ids(&self) -> &[u8; COLUMNS] {
        &self.ids
    }

    /// The biome id of column `(x, z)`, or `None` if either coordinate is
    /// outside `0..16`.
    #[inline]
    pub fn id_at(&self, x: usize, z: usize) -> Option<u8> {
        column_index(x, z).map(|i| self.ids[i])
    }

    pub const fn size_hint(&self) -> usize {
        COLUMNS
    }

    pub fn to_disk<W>(&self, writer: &mut Cursor<W>) -> Result<()>
    where
        Cursor<W>: Write,
    {
        writer.write_all(self.ids.as_slice())?;

        Ok(())
    }

    pub fn from_disk<R>(reader: &mut Cursor<R>) -> Result<Biomes2d>
    where
        Cursor<R>: Read,
    {
        let mut ids: Box<[u8; COLUMNS]> = Box::new([0; COLUMNS]);
        reader.read_exact(ids.as_mut())?;

        Ok(Biomes2d { ids })
    }
}

impl From<Box<[u8; COLUMNS]>> for Biomes2d {
    fn from(ids: Box<[u8; COLUMNS]>) -> Self {
        Biomes2d { ids }
    }
}

/// A chunk's biome data, in whichever on-disk layout produced it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Biomes {
    /// One biome id per column (`0x2d`), predating per-subchunk biomes.
    Columns(Biomes2d),
    /// Palette-based biome ids per subchunk (`0x2b`).
    Volumes(Biomes3d),
}

impl Biomes {
    pub fn size_hint(&self) -> usize {
        match self {
            Biomes::Columns(v) => v.size_hint(),
            Biomes::Volumes(v) => v.size_hint(),
        }
    }

    pub fn to_disk<W>(&self, writer: &mut Cursor<W>) -> Result<()>
    where
        Cursor<W>: Write,
    {
        match self {
            Biomes::Columns(v) => v.to_disk(writer),
            Biomes::Volumes(v) => v.to_disk(writer),
        }
    }

    /// Decodes the palette-based `0x2b` layout. Column biomes (`0x2d`) are
    /// read through [`Biomes2d::from_disk`], since that record is paired
    /// with a heightmap of a different layout entirely.
    pub fn from_disk<M: UnpackingMethod, R>(reader: &mut Cursor<R>) -> Result<Biomes>
    where
        Cursor<R>: Read,
    {
        Ok(Biomes3d::from_disk::<M, _>(reader)?.into())
    }
}

impl From<Biomes2d> for Biomes {
    fn from(v: Biomes2d) -> Self {
        Biomes::Columns(v)
    }
}

impl From<Biomes3d> for Biomes {
    fn from(v: Biomes3d) -> Self {
        Biomes::Volumes(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_2d() -> Biomes2d {
        let mut ids = Box::new([0u8; COLUMNS]);
        for (i, id) in ids.iter_mut().enumerate() {
            *id = (i % 256) as u8;
        }
        Biomes2d { ids }
    }

    fn sample_3d() -> Biomes3d {
        Biomes3d {
            heightmap: Box::new([0; COLUMNS]),
            fragments: SmallVec::from_buf([BiomeEncoding::Single(1)]),
        }
    }

    #[test]
    fn biomes_2d_round_trips_through_disk() {
        let original = sample_2d();

        let mut buf = Cursor::new(Vec::new());
        original.to_disk(&mut buf).unwrap();
        let bytes = buf.into_inner();

        assert_eq!(bytes.len(), COLUMNS);

        let mut reader = Cursor::new(bytes.as_slice());
        let decoded = Biomes2d::from_disk(&mut reader).unwrap();

        assert_eq!(decoded, original);
    }

    #[test]
    fn biomes_2d_id_at_matches_z_major_flat_index() {
        let biomes = sample_2d();

        for z in 0..16usize {
            for x in 0..16usize {
                assert_eq!(biomes.id_at(x, z), Some(biomes.ids()[z * 16 + x]));
            }
        }
    }

    #[test]
    fn biomes_2d_id_at_rejects_out_of_range_coordinates() {
        let biomes = sample_2d();

        assert_eq!(biomes.id_at(16, 0), None);
        assert_eq!(biomes.id_at(0, 16), None);
        assert_eq!(biomes.id_at(16, 16), None);
    }

    #[test]
    fn biomes_wraps_both_layouts_and_compares_across_variants() {
        let columns: Biomes = sample_2d().into();
        assert!(matches!(columns, Biomes::Columns(_)));

        let other_columns: Biomes = sample_2d().into();
        assert_eq!(columns, other_columns);

        let volumes: Biomes = sample_3d().into();
        assert!(matches!(volumes, Biomes::Volumes(_)));

        // The two layouts never compare equal to each other.
        assert_ne!(columns, volumes);
    }
}
