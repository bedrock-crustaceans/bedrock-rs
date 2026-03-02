use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io::Cursor;
use std::iter::FusedIterator;
use std::ops::{Index, IndexMut};

use byteorder::{ReadBytesExt, WriteBytesExt};
use nbtx::LittleEndian;
use serde::{Deserialize, Serialize};
use vek::Vec3;

use crate::error::{Error, Result};
use crate::packed::{self, PackedResult};

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
#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename = "")]
pub struct BlockDef {
    /// Name of the block.
    pub name: String,
    /// Version of the block.
    #[serde(with = "block_version")]
    pub version: Option<[u8; 4]>,
    /// Block-specific properties.
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

/// A layer in a sub chunk.
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
#[derive(Debug, PartialEq)]
pub struct ChunkLayer {
    /// List of indices into the palette.
    ///
    /// Coordinates can be converted to an offset into the array using [`to_offset`].
    pub indices: Box<[u16; 4096]>,
    /// List of all different block types in this sub chunk layer.
    pub palette: Vec<BlockDef>,
}

impl ChunkLayer {
    /// Creates an iterator over the blocks in this layer.
    ///
    /// This iterates over every indices
    pub fn iter(&self) -> BlockIter<'_> {
        BlockIter::from(self)
    }

    /// Gets a reference to a block inside the subchunk.
    pub fn get<V>(&self, pos: V) -> Option<&BlockDef>
    where
        V: Into<Vec3<u8>>,
    {
        let pos = pos.into();

        if pos.x > 16 || pos.y > 16 || pos.z > 16 {
            return None;
        }

        let offset = to_offset(pos);
        assert!(offset < 4096, "Array offset out of range");

        let index = self.indices[offset] as usize;
        Some(&self.palette[index])
    }

    // FIXME: Using this method will modify every block with the same index
    // instead of only the block at the specified position.
    // pub fn get_mut(&mut self, pos: Vector<u8, 3>) -> Option<&mut PaletteEntry> {
    //     if pos.x > 16 || pos.y > 16 || pos.z > 16 {
    //         return None;
    //     }

    //     let offset = to_offset(pos);
    //     debug_assert!(offset < 4096);

    //     let index = self.indices[offset] as usize;
    //     Some(&mut self.palette[index])
    // }

    /// Returns a reference to the block palette.
    pub fn palette(&self) -> &[BlockDef] {
        &self.palette
    }

    /// Returns a mutable reference to the block palette.
    pub fn palette_mut(&mut self) -> &mut [BlockDef] {
        &mut self.palette
    }

    /// Returns a reference to the block indices.
    pub const fn indices(&self) -> &[u16; 4096] {
        &self.indices
    }

    /// Returns a mutable reference to the block indices.
    pub fn indices_mut(&mut self) -> &mut [u16; 4096] {
        &mut self.indices
    }

    /// Takes ownership of the layer and returns the indices.
    pub fn take_indices(self) -> Box<[u16; 4096]> {
        self.indices
    }

    /// Creates an empty subchunk layer.
    pub fn empty() -> Self {
        Self {
            indices: Box::new([0; 4096]),
            palette: vec![],
        }
    }

    /// Whether this subchunk layer is empty.
    pub fn is_empty(&self) -> bool {
        self.palette.is_empty()
    }

    /// Deserializes a single layer from the given buffer.
    fn deserialize_disk(mut reader: &mut Cursor<&[u8]>) -> Result<Self> {
        let indices = match packed::deserialize_array(&mut reader)? {
            PackedResult::Data(data) => data,
            PackedResult::Empty => return Err(Error::Invalid("chunk layer packed array cannot be empty")),
            PackedResult::Inherit => return Err(Error::Invalid("chunk layers do not support inheritance")),
        };

        let len = reader.read_u32::<LittleEndian>()? as usize;
        let mut palette = Vec::with_capacity(len);

        for _ in 0..len {
            let entry  = nbtx::from_le_bytes(&mut reader)?;
            palette.push(entry);
        }

        Ok(Self { indices, palette })
    }

    /// Serializes a single layer into the given buffer.
    fn serialize_disk(&self, writer: &mut Vec<u8>) -> Result<()> {
        packed::serialize_array(writer, &self.indices, self.palette.len(), false)?;

        writer.write_u32::<LittleEndian>(self.palette.len() as u32)?;
        for entry in &self.palette {
            nbtx::to_le_bytes_in(writer, entry)?;
        }

        Ok(())
    }
}

impl<'a> IntoIterator for &'a ChunkLayer {
    type IntoIter = BlockIter<'a>;
    type Item = &'a BlockDef;

    fn into_iter(self) -> Self::IntoIter {
        BlockIter::from(self)
    }
}

impl<I> Index<I> for ChunkLayer
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
        assert!(
            position.x <= 16 && position.y <= 16 && position.z <= 16,
            "Block position out of sub chunk bounds"
        );

        let offset = to_offset(position);
        let index = self.indices[offset] as usize;
        &self.palette[index]
    }
}

impl<I> IndexMut<I> for ChunkLayer
where
    I: Into<Vec3<u8>>,
{
    /// # Panics
    /// 
    /// This function panics if the given position is out of range.
    /// In other words, it requires that `x <= 16`, `y <= 16` and `z <= 16`.
    fn index_mut(&mut self, position: I) -> &mut BlockDef {
        let position = position.into();
        assert!(
            position.x <= 16 && position.y <= 16 && position.z <= 16,
            "Block position out of sub chunk bounds"
        );

        let offset = to_offset(position);
        let index = self.indices[offset] as usize;
        &mut self.palette[index]
    }
}

impl Default for ChunkLayer {
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

/// A Minecraft sub chunk.
///
/// Every world contains
#[derive(Debug, PartialEq)]
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
    pub layers: Vec<ChunkLayer>,
}

impl SubChunk {
    /// Creates a subchunk filled with air.
    pub fn empty(index: i8) -> Self {
        Self {
            index,
            layers: vec![ChunkLayer::empty()],
            version: SubChunkVersion::Limitless,
        }
    }

    /// Whether this subchunk is empty.
    pub fn is_empty(&self) -> bool {
        self.layers.is_empty() || self.layers[0].is_empty()
    }

    /// Version of this subchunk.
    /// See [`SubChunkVersion`] for more information.
    pub fn version(&self) -> SubChunkVersion {
        self.version
    }

    /// Vertical index of this subchunk
    pub fn index(&self) -> i8 {
        self.index
    }

    /// The layers (storage records) contained in this subchunk.
    pub fn layers(&self) -> &[ChunkLayer] {
        &self.layers
    }

    /// Get an immutable reference to the layer at the specified index.
    pub fn layer(&self, index: usize) -> Option<&ChunkLayer> {
        self.layers.get(index)
    }

    /// Get a mutable reference to the layer at the specified index.
    pub fn layer_mut(&mut self, index: usize) -> Option<&mut ChunkLayer> {
        self.layers.get_mut(index)
    }

    /// Takes ownership of the subchunk and returns an owned list of its layers.
    #[inline]
    pub fn take_layers(self) -> Vec<ChunkLayer> {
        self.layers
    }

    /// Deserialize a full sub chunk from the given buffer.
    pub fn deserialize_disk(reader: &mut Cursor<&[u8]>) -> Result<Self> {
        let version = SubChunkVersion::try_from(reader.read_u8()?)?;
        let layer_count = match version {
            SubChunkVersion::Legacy => 1,
            _ => reader.read_u8()?,
        };

        let index = if version == SubChunkVersion::Limitless { 
            reader.read_i8()?
        } else { 0 };

        // let mut layers = SmallVec::with_capacity(layer_count as usize);
        let mut layers = Vec::with_capacity(layer_count as usize);
        for _ in 0..layer_count {
            layers.push(ChunkLayer::deserialize_disk(reader)?);
        }

        Ok(Self { version, index, layers })
    }

    /// Serialises the sub chunk into the given writer.
    pub fn serialize_disk(&self, writer: &mut Vec<u8>) -> Result<()> {
        writer.write_u8(self.version as u8)?;
        writer.write_u8(self.layers.len() as u8)?;

        if self.version == SubChunkVersion::Limitless {
            writer.write_i8(self.index)?;
        }

        for layer in &self.layers {
            layer.serialize_disk(writer)?;
        }

        Ok(())
    }
}

impl Index<usize> for SubChunk {
    type Output = ChunkLayer;

    fn index(&self, index: usize) -> &Self::Output {
        &self.layers[index]
    }
}

impl IndexMut<usize> for SubChunk {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.layers[index]
    }
}

/// Iterator over blocks in a layer.
pub struct BlockIter<'l> {
    /// Indices in the sub chunk.
    /// While iterating, this is slowly consumed by `std::slice::split_at`.
    indices: std::slice::Iter<'l, u16>,
    /// All possible block states in the current chunk.
    palette: &'l [BlockDef],
}

impl<'l> From<&'l ChunkLayer> for BlockIter<'l> {
    fn from(layer: &'l ChunkLayer) -> Self {
        Self {
            indices: layer.indices.iter(),
            palette: &layer.palette,
        }
    }
}

impl<'a> Iterator for BlockIter<'a> {
    type Item = &'a BlockDef;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.indices.next()?;
        self.palette.get(*index as usize)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.len()))
    }
}

impl FusedIterator for BlockIter<'_> {}

impl ExactSizeIterator for BlockIter<'_> {
    fn len(&self) -> usize {
        self.indices.len()
    }
}