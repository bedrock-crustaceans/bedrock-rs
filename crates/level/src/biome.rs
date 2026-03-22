use std::io::Read;
use std::io::{ErrorKind, Write};

use byteorder::LittleEndian;
use byteorder::{ReadBytesExt, WriteBytesExt};
use smallvec::SmallVec;

use crate::PackingMethod;
use crate::bits::{BitArray, IndicesType};
use crate::error::Error;
use crate::error::Result;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Biomes {
    heightmap: Box<[u16; 256]>,
    fragments: SmallVec<[BiomeEncoding; 1]>,
}

impl Biomes {
    pub fn heightmap(&self) -> &[u16; 256] {
        &self.heightmap
    }

    // This roughly estimates the size of the output buffer. Chunks with many paletted fragments
    // will be larger than this estimate while chunks with many inherited biome fragments will be smaller.
    pub fn size_hint(&self) -> usize {
        const HEIGHTMAP_SIZE: usize = 256 * 2;
        HEIGHTMAP_SIZE + self.fragments.len() * std::mem::size_of::<BiomeEncoding>()
    }

    pub fn to_disk(&self, writer: &mut Vec<u8>) -> Result<()> {
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

    pub fn from_disk<M: PackingMethod, R: Read>(mut reader: R) -> Result<Biomes> {
        let mut heightmap: Box<[u16; 256]> = Box::new([0; 256]);

        let heightmap_bytes = bytemuck::cast_slice_mut::<u16, u8>(heightmap.as_mut());
        reader.read_exact(heightmap_bytes)?;

        let mut fragments = SmallVec::new();
        loop {
            let indices = match BitArray::from_disk_typed::<M, _>(&mut reader) {
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

        Ok(Biomes {
            heightmap,
            fragments,
        })
    }
}
