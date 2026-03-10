use std::io::Read;

use byteorder::ReadBytesExt;
use byteorder::LittleEndian;
use smallvec::SmallVec;

use crate::PackingMethod;
use crate::{error::Result};
use crate::bits::{BitArray, IndicesType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BiomeArray {
    array: BitArray,
    palette: Vec<u32>
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
    Palette(BiomeArray)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Biomes {
    heightmap: Box<[u16; 4096]>,
    fragments: SmallVec<[BiomeEncoding; 1]>
}

impl Biomes {
    pub fn heightmap(&self) -> &[u16; 4096] {
        &self.heightmap
    }

    pub fn from_disk<M: PackingMethod, R: Read>(mut reader: R) -> Result<Biomes> {
        let mut heightmap: Box<[u16; 4096]> = Box::new([0; 4096]);

        let heightmap_bytes = bytemuck::cast_slice_mut::<u16, u8>(heightmap.as_mut());
        reader.read_exact(heightmap_bytes)?;

        let mut fragments = SmallVec::new();
        loop {
            let indices = BitArray::from_disk_typed::<M, _>(&mut reader)?;
            match indices {
                IndicesType::Data(array) => {
                    let len = reader.read_u32::<LittleEndian>()?;
                    let mut palette = Vec::with_capacity(len as usize);
                    for _ in 0..len {
                        palette.push(reader.read_u32::<LittleEndian>()?);
                    }

                    fragments.push(BiomeEncoding::Palette(BiomeArray {
                        array, palette
                    }));
                },
                IndicesType::Empty => {
                    let single = reader.read_u32::<LittleEndian>()?;
                    fragments.push(BiomeEncoding::Single(single))
                },
                IndicesType::Inherit => {
                    fragments.push(BiomeEncoding::Inherit)
                }
            }
        }

        Ok(Biomes {
            heightmap, fragments
        })
    }
}