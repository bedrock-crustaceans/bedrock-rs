use std::io::Read;

use crate::{error::Result, unpacker::ArrayType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BiomeArray {
    indices: ArrayType,
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
    fragments: Vec<BiomeEncoding>
}

impl Biomes {
    pub fn heightmap(&self) -> &[u16; 4096] {
        &self.heightmap
    }

    pub fn deserialize_from_disk<R: Read>(mut reader: R) -> Result<Biomes> {
        let mut heightmap: Box<[u16; 4096]> = Box::new([0; 4096]);

        let heightmap_bytes = bytemuck::cast_slice_mut::<u16, u8>(heightmap.as_mut());
        reader.read_exact(heightmap_bytes)?;

        let mut fragments = Vec::new();
        loop {
            todo!()
        }

        todo!()
    }
}