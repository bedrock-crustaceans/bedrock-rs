use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::Read;
use std::iter::FusedIterator;
use std::ops::Index;

use byteorder::{ReadBytesExt, WriteBytesExt};
use nbtx::LittleEndian;
use serde::{Deserialize, Serialize};
use vek::Vec3;

use crate::PackingMethod;
use crate::bits::{BitArray, BitArrayIter};
use crate::error::{Error, Result};

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
    #[serde(default)]
    pub states: HashMap<String, nbtx::Value>,
}

impl BlockDef {
    /// Hashes this block.
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

/// Iterates over all blocks in a layer.
pub struct LayerIter<'l> {
    array_iter: BitArrayIter<'l>,
    palette: &'l [BlockDef],
}

impl<'l> Iterator for LayerIter<'l> {
    type Item = &'l BlockDef;

    fn next(&mut self) -> Option<&'l BlockDef> {
        let index = self.array_iter.next()?;
        Some(&self.palette[index as usize])
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl FusedIterator for LayerIter<'_> {}

impl ExactSizeIterator for LayerIter<'_> {
    fn len(&self) -> usize {
        self.array_iter.len()
    }
}

impl<'l> From<&'l Layer> for LayerIter<'l> {
    fn from(layer: &'l Layer) -> LayerIter<'l> {
        let array_iter = layer.array.iter();

        LayerIter {
            palette: &layer.palette,
            array_iter,
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
    array: BitArray,
    /// List of all different block types in this sub chunk layer.
    palette: Vec<BlockDef>,
}

impl Layer {
    pub fn get<K: Into<Vec3<u8>>>(&self, block: K) -> Option<&BlockDef> {
        let pos = block.into();
        let offset = to_offset(pos);
        let index = self.array.get(offset)?;
        Some(&self.palette[index as usize])
    }

    pub fn set<K: Into<Vec3<u8>>>(&self, _block: K, _value: BlockDef) {
        todo!()
    }

    pub fn palette(&self) -> &[BlockDef] {
        &self.palette
    }

    #[inline]
    pub fn indices(&self) -> &BitArray {
        &self.array
    }

    /// Deserializes a single layer from the given buffer.
    fn from_disk<M: PackingMethod, R: Read>(mut reader: R) -> Result<Self> {
        let array = BitArray::from_disk::<M, _>(&mut reader)?;

        let len = reader.read_u32::<LittleEndian>()? as usize;
        let mut palette = Vec::with_capacity(len);

        for _ in 0..len {
            let entry = nbtx::from_le_bytes(&mut reader)?;
            palette.push(entry);
        }

        Ok(Self { array, palette })
    }

    /// Serializes a single layer into the given buffer.
    fn to_disk(&self, writer: &mut Vec<u8>) -> Result<()> {
        let plen = self.palette.len();

        self.array.to_disk(writer, plen)?;

        writer.write_u32::<LittleEndian>(plen as u32)?;
        for entry in &self.palette {
            nbtx::to_le_bytes_in(writer, entry)?;
        }

        Ok(())
    }

    /// Creates an iterator over the blocks in this layer.
    ///
    /// This iterates over every indices
    pub fn iter(&self) -> LayerIter<'_> {
        LayerIter::from(self)
    }

    /// Whether this subchunk layer is empty.
    pub fn is_empty(&self) -> bool {
        self.palette.is_empty()
    }
}

impl<'a> IntoIterator for &'a Layer {
    type Item = &'a BlockDef;
    type IntoIter = LayerIter<'a>;

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
    fn index(&self, position: I) -> &BlockDef {
        let position = position.into();
        let offset = to_offset(position);
        let index = self.array.get(offset).expect("layer index out of bounds");

        &self.palette[index as usize]
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

/// A Minecraft sub chunk.
///
/// Every world contains
#[derive(Debug, Clone, PartialEq)]
pub struct SubChunk {
    /// Version of the sub chunk.
    ///
    /// See [`SubChunkVersion`] for more info.
    version: SubChunkVersion,
    /// Index of the sub chunk.
    ///
    /// This specifies the vertical position of the sub chunk.
    /// It is only used if `version` is set to [`Limitless`](SubChunkVersion::Limitless)
    /// and set to 0 otherwise.
    index: i8,
    /// Layers the sub chunk consists of.
    ///
    /// See [`SubLayer`] for more info.
    layers: Vec<Layer>,
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

    /// Gets the `n`-th layer of this chunk. Usually chunks will only have one layer or two layers.
    pub fn layer(&self, index: usize) -> &Layer {
        &self.layers[index]
    }

    /// Deserialize a full sub chunk from the given buffer.
    pub fn from_disk<M: PackingMethod, R: Read>(mut reader: R) -> Result<Self> {
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
            let layer = Layer::from_disk::<M, _>(&mut reader)?;
            layers.push(layer);
        }

        Ok(Self {
            version,
            index,
            layers,
        })
    }

    /// Serialises the sub chunk into the given writer.
    pub fn to_disk<M: PackingMethod>(&self, writer: &mut Vec<u8>) -> Result<()> {
        writer.write_u8(self.version as u8)?;
        writer.write_u8(self.layers.len() as u8)?;

        if self.version == SubChunkVersion::Limitless {
            writer.write_i8(self.index)?;
        }

        for layer in &self.layers {
            layer.to_disk(writer)?;
        }

        Ok(())
    }
}
