use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io::{Cursor, Read, Write};
use std::iter::FusedIterator;
use std::ops::Index;

use byteorder::{ReadBytesExt, WriteBytesExt};
use nbtx::LittleEndian;
use nohash_hasher::BuildNoHashHasher;
use rustc_hash::FxHasher;
use serde::{Deserialize, Serialize};

use crate::bits::{BitArray, BitArrayIter, IndicesType};
use crate::error::{Error, Result};
use crate::types::BlockPosition;
use crate::{Greedy, Lazy, UnpackingMethod};

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

impl Hash for BlockDef {
    /// Hashes this block.
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.name.as_bytes());
        for (k, v) in &self.states {
            state.write(k.as_bytes());
            v.hash(state);
        }
    }
}

/// Iterates over all blocks in a layer.
pub struct LayerIter<'l> {
    /// An iterator over the indices
    array_iter: BitArrayIter<'l>,
    /// The palette.
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
    /// Used to check which blocks are already in the palette. Block definitions are hashed manually and compared with their hash in this set.
    /// This is because `BlockDef` does not implement `Eq` and we also prevent cloning the entire block definition on each insertion.
    hashes: HashMap<u64, u16, BuildNoHashHasher<u64>>,
}

impl Layer {
    const HASH_SEED: usize = 0;

    /// Whether this layer uses the lazy unpacking strategy.
    #[inline]
    pub fn is_lazy(&self) -> bool {
        self.array.is_lazy()
    }

    /// Retrieves the block at `position`.
    pub fn get<K: Into<BlockPosition>>(&self, position: K) -> Option<&BlockDef> {
        let pos = position.into();
        let offset = to_offset(pos);
        let index = self.array.get(offset)?;
        Some(&self.palette[index as usize])
    }

    /// Sets the block at `position` to `block`.
    ///
    ///
    pub fn set<K: Into<BlockPosition>>(&mut self, position: K, block: BlockDef) {
        // Check whether the block is in the palette
        let hash = Self::hash_def(&block);
        let palette_index = *self.hashes.entry(hash).or_insert_with(|| {
            // Block does not exist in palette, push it.
            self.palette.push(block);
            self.palette.len() as u16 - 1
        });

        let index = to_offset(position.into());
        self.array.set(index, palette_index);
    }

    /// Computes the hash of the block.
    pub(crate) fn hash_def(block: &BlockDef) -> u64 {
        let mut state = FxHasher::with_seed(Self::HASH_SEED);
        block.hash(&mut state);
        state.finish()
    }

    /// Determines the index in the palette of the block.
    pub fn palette_index(&self, block: &BlockDef) -> Option<u16> {
        let hash = Self::hash_def(block);
        self.hashes.get(&hash).copied()
    }

    /// Whether the palette contains the given block.
    pub fn contains(&self, block: &BlockDef) -> bool {
        let hash = Self::hash_def(block);
        self.hashes.contains_key(&hash)
    }

    /// Returns the palette used for this chunk
    #[inline]
    pub fn palette(&self) -> &[BlockDef] {
        &self.palette
    }

    /// Returns the indices
    #[inline]
    pub fn indices(&self) -> &BitArray {
        &self.array
    }

    /// Deserializes a single layer from the given buffer.
    fn from_disk<M: UnpackingMethod, R>(reader: &mut Cursor<R>) -> Result<Self>
    where
        Cursor<R>: Read,
    {
        let array = match BitArray::from_disk::<M, _>(reader)? {
            IndicesType::Empty => {
                return Err(Error::Invalid(
                    "found empty bit array while deserializing chunk, expected data",
                ));
            }
            IndicesType::Inherit => {
                return Err(Error::Invalid(
                    "chunks do not support inheriting bit arrays",
                ));
            }
            IndicesType::Data(array) => array,
        };

        let len = reader.read_u32::<LittleEndian>()? as usize;
        let mut palette = Vec::with_capacity(len);

        for _ in 0..len {
            let entry = nbtx::from_le_bytes(reader)?;
            palette.push(entry);
        }

        let mut hashes =
            HashMap::with_capacity_and_hasher(palette.len(), BuildNoHashHasher::default());
        hashes.extend(palette.iter().enumerate().map(|(i, block)| {
            let hash = Self::hash_def(block);
            (hash, i as u16)
        }));

        Ok(Self {
            array,
            hashes,
            palette,
        })
    }

    /// Serializes a single layer into the given buffer.
    fn to_disk<W>(&self, writer: &mut Cursor<W>) -> Result<()>
    where
        Cursor<W>: Write,
    {
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
    I: Into<BlockPosition>,
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
pub const fn to_offset(position: BlockPosition) -> usize {
    16 * 16 * position.0 as usize + 16 * position.2 as usize + position.1 as usize
}

/// Converts an offset back to coordinates.
///
/// This offset should be in the range [0, 4096).
#[inline]
pub const fn from_offset(offset: usize) -> BlockPosition {
    let x = (offset >> 8) as u8 & 0xf;
    let y = offset as u8 & 0xf;
    let z = (offset >> 4) as u8 & 0xf;

    BlockPosition(x, y, z)
}

/// A Minecraft sub chunk.
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
    #[inline]
    pub fn version(&self) -> SubChunkVersion {
        self.version
    }

    /// Vertical index of this subchunk
    #[inline]
    pub fn index(&self) -> i8 {
        self.index
    }

    /// Gets the `n`-th layer of this chunk. Usually chunks will only have one layer or two layers.
    ///
    /// Returns `None` if the layer does not exist.
    #[inline]
    pub fn get_layer(&self, index: usize) -> Option<&Layer> {
        self.layers.get(index)
    }

    /// Gets the `n`-th layer of this chunk mutably. Usually chunks only have one layer or two layers.
    ///
    /// Returns `None` if the layer does not exist.
    #[inline]
    pub fn get_layer_mut(&mut self, index: usize) -> Option<&mut Layer> {
        self.layers.get_mut(index)
    }

    /// Deserialize a full sub chunk from the given buffer.
    ///
    /// The generic `M` is the unpacking method to use. See [`from_disk_lazy`] and [`from_disk_greedy`]
    /// for more information.
    ///
    /// [`from_disk_lazy`]: Self::from_disk_lazy
    /// [`from_disk_greedy`]: Self::from_disk_greedy
    pub fn from_disk<M: UnpackingMethod, R>(reader: &mut Cursor<R>) -> Result<Self>
    where
        Cursor<R>: Read,
    {
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
            let layer = Layer::from_disk::<M, _>(reader)?;
            layers.push(layer);
        }

        Ok(Self {
            version,
            index,
            layers,
        })
    }

    /// Lazily unpacks the subchunk. This means the the internal indices array is only unpacked when you actually access those blocks.
    ///
    /// This makes deserialising slightly faster but iteration a lot slower. This method is unable to make use of SIMD while [`from_disk_greedy`]
    /// uses a SIMD-accelerated deserializer. Additionally using this method means that editing the subchunk might cause the bit array to be repacked
    /// to accomodate the larger palette size.
    #[inline]
    pub fn from_disk_lazy<R>(reader: &mut Cursor<R>) -> Result<Self>
    where
        Cursor<R>: Read,
    {
        Self::from_disk::<Lazy, _>(reader)
    }

    /// Greedily unpacks the subchunk. This means that the entire index array is unpacked immediately. This allows the deserialisation process to be
    /// accelerated with SIMD which makes it about 2.5x faster. Additionally this means the chunk does not have to be re-encoded whenever the bit size changes.
    /// It however uses more memory.
    #[inline]
    pub fn from_disk_greedy<R>(reader: &mut Cursor<R>) -> Result<Self>
    where
        Cursor<R>: Read,
    {
        Self::from_disk::<Greedy, _>(reader)
    }

    /// Serialises the sub chunk into the given writer.
    pub fn to_disk<M: UnpackingMethod, W>(&self, writer: &mut Cursor<W>) -> Result<()>
    where
        Cursor<W>: Write,
    {
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
