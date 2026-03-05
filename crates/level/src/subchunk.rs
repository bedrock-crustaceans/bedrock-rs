use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io::{Read, Write};
use std::iter::FusedIterator;
use std::ops::Index;

use byteorder::{ReadBytesExt, WriteBytesExt};
use nbtx::LittleEndian;
use serde::{Deserialize, Serialize};
use vek::Vec3;

use crate::error::{Error, Result};
use crate::unpacker::{ArrayType, PackedArray, PackedArrayIter, PackedResult};

/// Version of the subchunk.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SubChunkVersion {
    /// Legacy sub chunks are from before the Aquatic update.
    /// These sub chunks only contain a single layer.
    Legacy = 1,
    /// Limited sub chunks are from before the Caves and Cliffs update.
    Limited = 8,
    /// Limitless are post Caves and Cliffs. The only difference between `Limitless` and `Limited` is the fact that limitless
    /// contains a sub chunk index.
    Limitless = 9,
}

impl TryFrom<u8> for SubChunkVersion {
    type Error = Error;

    fn try_from(v: u8) -> Result<Self> {
        Ok(match v {
            1 => Self::Legacy,
            8 => Self::Limited,
            9 => Self::Limitless,
            _ => return Err(Error::Invalid("sub chunk version")),
        })
    }
}

/// Serialisation and deserialisation for block versions.
mod block_version {
    use serde::{Deserialize, Deserializer, Serializer};

    /// Deserializes a block version.
    pub fn deserialize<'de, D>(de: D) -> Result<Option<[u8; 4]>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let word = Option::<i32>::deserialize(de)?;
        Ok(word.map(i32::to_be_bytes))
    }

    /// Serializes a block version.
    #[allow(clippy::trivially_copy_pass_by_ref)] // Serde requirement.
    pub fn serialize<S>(v: &Option<[u8; 4]>, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(b) = v {
            ser.serialize_i32(i32::from_be_bytes(*b))
        } else {
            ser.serialize_none()
        }
    }
}

/// Definition of block in the sub chunk block palette.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename = "")]
pub struct BlockDef {
    /// Name of the block.
    pub name: String,
    /// Version of the block.
    #[serde(default)]
    #[serde(with = "block_version")]
    pub version: Option<[u8; 4]>,
    /// Block-specific properties.
    pub states: HashMap<String, nbtx::Value>,
}

impl BlockDef {
    /// Hashes this block. This bypasses the `Hash` trait because floats do not implement `Eq`.
    pub fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();

        hasher.write(self.name.as_bytes());
        for (k, v) in &self.states {
            hasher.write(k.as_bytes());
            v.hash(&mut hasher);
        }

        hasher.finish()
    }
}

/// Array-agnostic iterators.
enum LayerIterInner<'l> {
    Lazy(PackedArrayIter<'l>),
    Greedy(std::slice::Iter<'l, u16>),
}

/// An iterator over the blocks in a layer.
pub struct LayerIter<'l> {
    inner: LayerIterInner<'l>,
    palette: &'l [BlockDef],
}

impl<'l> Iterator for LayerIter<'l> {
    type Item = &'l BlockDef;

    fn next(&mut self) -> Option<&'l BlockDef> {
        let index = match &mut self.inner {
            LayerIterInner::Greedy(iter) => iter.next().copied(),
            LayerIterInner::Lazy(iter) => iter.next(),
        }? as usize;

        Some(&self.palette[index])
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<'l> FusedIterator for LayerIter<'l> {}

impl<'l> ExactSizeIterator for LayerIter<'l> {
    fn len(&self) -> usize {
        match &self.inner {
            LayerIterInner::Greedy(iter) => iter.len(),
            LayerIterInner::Lazy(iter) => iter.len(),
        }
    }
}

impl<'l> From<&'l Layer> for LayerIter<'l> {
    fn from(layer: &'l Layer) -> LayerIter<'l> {
        let inner = match &layer.indices {
            ArrayType::Greedy(greedy) => LayerIterInner::Greedy(greedy.iter()),
            ArrayType::Lazy(array) => LayerIterInner::Lazy(array.iter()),
        };

        LayerIter {
            palette: &layer.palette,
            inner,
        }
    }
}

/// A layer in a sub chunk.
///
/// Unlike [`LazyLayer`] this layer immediately unpacks the entire chunk allowing for much faster iteration at
/// a much higher memory cost.
///
/// Sub chunks can have multiple layers.
/// The first layer contains plain old block data,
/// while the second layer (if it exists) generally contains water logging data.
///
/// The layer is prefixed with a byte indicating the size in bits of the block indices.
/// This is followed by `4096 / (32 / bits)` 32-bit integers containing the actual indices.
/// In case the size is 3, 5 or 6, there is one more integer appended to the end to fit all data.
///
/// Immediately following the indices, the palette starts.
/// This is prefixed with a 32-bit little endian integer specifying the size of the palette.
/// The rest of the palette then consists of `n` concatenated NBT compounds.
#[doc(alias = "storage record")]
#[derive(Debug, Clone, PartialEq)]
pub struct Layer {
    /// List of indices into the palette.
    ///
    /// Coordinates can be converted to an offset into the array using [`to_offset`].
    indices: ArrayType,
    /// List of all different block types in this sub chunk layer.
    palette: Vec<BlockDef>,
}

impl Layer {
    /// Returns the block at the specified position in the layer.
    /// 
    /// This function returns `None` if `pos` was outside of the subchunks [0, 16)^3 range. 
    pub fn get<K>(&self, pos: K) -> Option<&BlockDef>
    where
        K: Into<Vec3<u8>>,
    {
        let pos = pos.into();

        let offset = to_offset(pos);
        let idx = self.indices.get(offset)?;

        self.palette.get(idx as usize)
    }

    /// Sets the block at the specified position in the layer.
    /// 
    /// If the subchunk uses the lazy method, this method might repack the array with more bits
    /// depending on the amount of unique blocks in the chunk.
    pub fn set<K>(&mut self, block: K, value: BlockDef)
    where
        K: Into<Vec3<u8>>,
    {
        let pos = block.into();
        let offset = to_offset(pos);
        
        // Check whether definition exists in palette.
        let idx = self.palette
            .iter()
            .enumerate()
            .find_map(|(i, d)| value.eq(d).then_some(i));

        if idx.is_none() {
            self.palette.push(value);
        }

        let idx = idx.unwrap_or_else(|| self.palette.len() - 1);
        self.indices.set(offset, idx as u16);
    }

    /// Returns the block palette of this layer.
    pub fn palette(&self) -> &[BlockDef] {
        &self.palette
    }

    /// Deserializes a single layer from the given buffer.
    pub(crate) fn deserialize_from_disk<M: PackingMethod, R: Read>(mut reader: R) -> Result<Self> {
        let indices = if M::IS_GREEDY {
            let indices = match Layer::unpack_array(&mut reader)? {
                PackedResult::Data(data) => data,
                PackedResult::Empty => {
                    return Err(Error::Invalid("chunk layer packed array cannot be empty"))
                }
                PackedResult::Inherit => {
                    return Err(Error::Invalid("chunk layers do not support inheritance"))
                }
            };

            ArrayType::Greedy(indices)
        } else {
            let bits = reader.read_u8()? >> 1;
            let indices = match bits {
                0 => return Err(Error::Invalid("chunk layer packed array cannot be empty")),
                0x7f => return Err(Error::Invalid("chunk layers do not support inheritance")),
                bits => {
                    let blocks_per_word = 32 / bits;
                    let word_count = 4096usize.div_ceil(blocks_per_word as usize);

                    let mut words = Vec::with_capacity(word_count);
                    for _ in 0..word_count {
                        words.push(reader.read_u32::<LittleEndian>()?);
                    }

                    PackedArray::new(bits as u32, words)
                }
            };

            ArrayType::Lazy(indices)
        };

        let len = reader.read_u32::<LittleEndian>()? as usize;
        let mut palette = Vec::with_capacity(len);

        for _ in 0..len {
            let entry = nbtx::from_le_bytes(&mut reader)?;
            palette.push(entry);
        }

        Ok(Self { indices, palette })
    }

    // /// Deserializes a single layer from the given buffer.
    // fn deserialize_from_disk<R: Read>(mut reader: R) -> Result<Self> {
    //     let bits = reader.read_u8()?;
    //     let indices = match bits {
    //         0 => return Err(Error::Invalid("chunk layer packed array cannot be empty")),
    //         0x7f => return Err(Error::Invalid("chunk layers do not support inheritance")),
    //         bits => {
    //             let blocks_per_word = 32 / bits;
    //             let word_count = 4096 / blocks_per_word as usize;

    //             let words = vec![0u32; word_count];
    //             PackedArray::new(bits as u32, words)
    //         }
    //     };

    //     let len = reader.read_u32::<LittleEndian>()? as usize;
    //     let mut palette = Vec::with_capacity(len);

    //     for _ in 0..len {
    //         let entry = nbtx::from_le_bytes(&mut reader)?;
    //         palette.push(entry);
    //     }

    //     Ok(Self { indices, palette })
    // }

    // /// Serializes a single layer into the given buffer.
    // fn serialize_to_disk<W: Write>(&self, mut writer: W) -> Result<()> {
    //     writer.write_u8((self.indices.bits << 1) as u8)?;

    //     let cast = bytemuck::cast_slice::<u32, u8>(&self.indices.words);
    //     writer.write_all(cast)?;

    //     writer.write_u32::<LittleEndian>(self.palette.len() as u32)?;
    //     for entry in &self.palette {
    //         nbtx::to_le_bytes_in(&mut writer, entry)?;
    //     }

    //     Ok(())
    // }

    /// Serializes a single layer into the given buffer.
    fn serialize_to_disk<W: Write>(&self, mut writer: W) -> Result<()> {
        match &self.indices {
            ArrayType::Greedy(array) => {
                Self::pack_array(&mut writer, array, self.palette.len() - 1, false)?
            }
            ArrayType::Lazy(array) => {
                writer.write_u8((array.bits() << 1) as u8)?;

                let cast = bytemuck::cast_slice::<u32, u8>(array.words());
                writer.write_all(cast)?;
            }
        };

        writer.write_u32::<LittleEndian>(self.palette.len() as u32)?;
        for entry in &self.palette {
            nbtx::to_le_bytes_in(&mut writer, entry)?;
        }

        Ok(())
    }

    /// Creates an iterator over the blocks in this layer.
    ///
    /// This iterates over every indices
    pub fn iter(&self) -> LayerIter<'_> {
        LayerIter::from(self)
    }

    /// Creates an empty subchunk layer.
    pub fn empty() -> Self {
        todo!()
        // Self {
        //     indices: Box::new([0; 4096]),
        //     palette: vec![],
        // }
    }

    /// Whether this subchunk layer is empty.
    pub fn is_empty(&self) -> bool {
        self.palette.is_empty()
    }
}

impl<'a> IntoIterator for &'a Layer {
    type IntoIter = LayerIter<'a>;
    type Item = &'a BlockDef;

    fn into_iter(self) -> Self::IntoIter {
        LayerIter::from(self)
    }
}

impl<I> Index<I> for Layer
where
    I: Into<Vec3<u8>>,
{
    type Output = BlockDef;

    /// # Panics
    ///
    /// This function panics if the given position is out of range.
    /// In other words, it requires that `x <= 16`, `y <= 16` and `z <= 16`.
    fn index(&self, pos: I) -> &BlockDef {
        let pos = pos.into();
        self.get(pos).unwrap()
    }
}

impl Default for Layer {
    fn default() -> Self {
        Self::empty()
    }
}

/// Converts coordinates to offsets into the block palette indices.
///
/// These coordinates should be in the range [0, 16) for each component.
#[inline]
pub const fn to_offset(position: Vec3<u8>) -> usize {
    16 * 16 * position.x as usize + 16 * position.z as usize + position.y as usize
}

/// Converts an offset back to coordinates.
///
/// This offset should be in the range [0, 4096).
#[inline]
pub const fn from_offset(offset: usize) -> Vec3<u8> {
    let x = (offset >> 8) as u8 & 0xf;
    let y = offset as u8 & 0xf;
    let z = (offset >> 4) as u8 & 0xf;

    Vec3::new(x, y, z)
}

mod private {
    pub trait Sealed {}
}

pub trait PackingMethod: private::Sealed {
    const IS_GREEDY: bool;
}

pub enum Greedy {}

impl private::Sealed for Greedy {}

impl PackingMethod for Greedy {
    const IS_GREEDY: bool = true;
}

pub enum Lazy {}

impl private::Sealed for Lazy {}

impl PackingMethod for Lazy {
    const IS_GREEDY: bool = false;
}

/// A Minecraft sub chunk.
///
/// Every world contains
#[derive(Debug, Clone, PartialEq)]
pub struct SubChunk {
    /// Version of the sub chunk.
    ///
    /// See [`SubChunkVersion`] for more info.
    pub version: SubChunkVersion,
    /// Index of the sub chunk.
    ///
    /// This specifies the vertical position of the sub chunk.
    /// It is only used if `version` is set to [`Limitless`](SubChunkVersion::Limitless)
    /// and set to 0 otherwise.
    pub index: i8,
    /// Layers the sub chunk consists of.
    ///
    /// See [`SubLayer`] for more info.
    pub layers: Vec<Layer>,
}

impl SubChunk {
    /// Version of this subchunk.
    /// See [`SubChunkVersion`] for more information.
    pub fn version(&self) -> SubChunkVersion {
        self.version
    }

    /// Vertical index of this subchunk
    pub fn index(&self) -> i8 {
        self.index
    }

    pub fn layer(&self, index: usize) -> &Layer {
        &self.layers[index]
    }

    /// Deserialize a full sub chunk from the given buffer.
    pub fn deserialize_from_disk<M: PackingMethod, R: Read>(mut reader: R) -> Result<Self> {
        let version = SubChunkVersion::try_from(reader.read_u8()?)?;
        let layer_count = match version {
            SubChunkVersion::Legacy => 1,
            _ => reader.read_u8()?,
        };

        let index = if version == SubChunkVersion::Limitless {
            reader.read_i8()?
        } else {
            0
        };

        // let mut layers = SmallVec::with_capacity(layer_count as usize);
        let mut layers = Vec::with_capacity(layer_count as usize);
        for _ in 0..layer_count {
            let layer = Layer::deserialize_from_disk::<M, _>(&mut reader)?;
            layers.push(layer);
        }

        Ok(Self {
            version,
            index,
            layers,
        })
    }

    /// Serialises the sub chunk into the given writer.
    pub fn serialize_to_disk<M: PackingMethod, W: Write>(&self, mut writer: W) -> Result<()> {
        writer.write_u8(self.version as u8)?;
        writer.write_u8(self.layers.len() as u8)?;

        if self.version == SubChunkVersion::Limitless {
            writer.write_i8(self.index)?;
        }

        for layer in &self.layers {
            layer.serialize_to_disk(&mut writer)?;
        }

        Ok(())
    }
}
