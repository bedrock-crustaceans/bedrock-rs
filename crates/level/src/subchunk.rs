use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io::{Cursor, Read, Write};
use std::iter::FusedIterator;
use std::ops::Index;

use byteorder::{ReadBytesExt, WriteBytesExt};
use facet::Facet;
use nbtx::LittleEndian;
use nohash_hasher::BuildNoHashHasher;
use rustc_hash::FxHasher;

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

/// Definition of block in the sub chunk block palette.
#[derive(Debug, Clone, PartialEq, Facet)]
#[cfg_attr(
    not(feature = "deny-unknown-fields"),
    facet(nbtx::allow_unknown_fields)
)]
pub struct BlockDef {
    /// Name of the block.
    pub name: String,
    /// Version of the block: one byte each of major, minor, patch and revision,
    /// most significant first, packed into the `Int` tag Bedrock stores.
    ///
    /// Held as the word rather than the four bytes because there is no hook for
    /// converting a single field on the way in and out, and this type is nested
    /// inside other decoded structs (`ItemStack::block`,
    /// `FlowerPot::plant_block`), so a conversion written here would not be
    /// reached at those sites. [`Self::version_parts`] and [`Self::pack_version`]
    /// give the split form.
    #[facet(default)]
    pub version: Option<i32>,
    /// Block-specific properties.
    #[facet(default)]
    pub states: HashMap<String, nbtx::Value>,
}

impl BlockDef {
    /// The packed [`version`](Self::version) split into
    /// `[major, minor, patch, revision]`.
    #[inline]
    pub fn version_parts(&self) -> Option<[u8; 4]> {
        self.version.map(i32::to_be_bytes)
    }

    /// Packs `[major, minor, patch, revision]` into the word stored in
    /// [`version`](Self::version).
    #[inline]
    pub const fn pack_version(parts: [u8; 4]) -> i32 {
        i32::from_be_bytes(parts)
    }
}

impl Hash for BlockDef {
    /// Hashes this block.
    ///
    /// `states` is a `HashMap`, whose iteration order is not guaranteed to match between
    /// two maps holding the same entries -- hashing it in encounter order would make this
    /// value's hash depend on that unspecified order, breaking the rule that equal values
    /// hash equally. Each entry is hashed on its own with a fresh hasher and the resulting
    /// values combined with a commutative fold (wrapping add), so the result depends only
    /// on the entry set, never on the order iteration happens to visit it in, without
    /// collecting the map into an intermediate `Vec` to sort first.
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.name.as_bytes());

        let combined = self.states.iter().fold(0u64, |acc, (k, v)| {
            let mut entry_hasher = FxHasher::default();
            k.hash(&mut entry_hasher);
            v.hash(&mut entry_hasher);
            acc.wrapping_add(entry_hasher.finish())
        });
        state.write_u64(combined);
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
///
/// A bit size of 0 is a distinct, more compact layout: every index is implicitly 0, there is
/// no index array and no palette-length word, and exactly one NBT compound -- the sole
/// palette entry -- follows the header byte directly.
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
            IndicesType::Empty => BitArray::Empty,
            IndicesType::Inherit => {
                return Err(Error::Invalid(
                    "block layer bit-array header is 0x7f (inherit), which block/liquid layers do not support",
                ));
            }
            IndicesType::Data(array) => array,
        };

        // A zero bits-per-index layer (`BitArray::Empty`) has no palette-length word: the
        // single palette entry it implies follows the header directly.
        let palette = if let BitArray::Empty = array {
            let entry: BlockDef = nbtx::from_le_bytes(reader)?;
            vec![entry]
        } else {
            let len = reader.read_u32::<LittleEndian>()? as usize;
            let mut palette = Vec::with_capacity(len);

            for _ in 0..len {
                // No fallback or `lenient_width` here: every palette entry across
                // the whole real test world (every layer of every subchunk) has
                // `version` written as an `Int`, so there is nothing to reconcile,
                // and this runs once per entry.
                let entry: BlockDef = nbtx::from_le_bytes(reader)?;
                palette.push(entry);
            }

            palette
        };

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

        // A zero bits-per-index layer writes its single palette entry directly after the
        // header, with no palette-length word -- see `BitArray::Empty`.
        if let BitArray::Empty = self.array {
            nbtx::to_le_bytes_in(writer, &self.palette[0])?;
            return Ok(());
        }

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

#[cfg(test)]
mod palette_tests {
    use super::*;
    use crate::Greedy;

    /// Builds a one-layer legacy subchunk whose 4096 indices are all zero and
    /// whose palette is the single entry `palette_entry`.
    fn subchunk_with_palette(palette_entry: &nbtx::Value) -> Vec<u8> {
        let mut out = vec![SubChunkVersion::Legacy as u8];
        // One bit per index, so 4096 indices pack into 128 words, all zero.
        out.push(1 << 1);
        out.extend(std::iter::repeat_n(0u8, 128 * 4));
        out.extend_from_slice(&1u32.to_le_bytes());
        out.extend(nbtx::to_le_bytes(palette_entry).unwrap());
        out
    }

    fn entry_with_version(version: nbtx::Value) -> nbtx::Value {
        nbtx::Value::Compound(nbtx::Compound::from_iter([
            ("name".into(), nbtx::Value::String("minecraft:stone".into())),
            ("version".into(), version),
            (
                "states".into(),
                nbtx::Value::Compound(nbtx::Compound::new()),
            ),
        ]))
    }

    /// The tag the game actually writes, read on the fast path.
    #[test]
    fn palette_entry_with_int_version() {
        let bytes = subchunk_with_palette(&entry_with_version(nbtx::Value::Int(17_959_425)));
        let chunk = SubChunk::from_disk::<Greedy, _>(&mut Cursor::new(bytes.as_slice())).unwrap();
        let block = &chunk.get_layer(0).unwrap().palette()[0];
        assert_eq!(block.version, Some(17_959_425));
    }

    /// A narrower tag than `version`'s declared `Int` width is rejected: no
    /// real palette entry has ever been observed writing it any other way, so
    /// there is no `lenient_width` accepting one.
    #[test]
    fn palette_entry_with_narrow_version_tag_still_fails() {
        let bytes = subchunk_with_palette(&entry_with_version(nbtx::Value::Short(1)));
        assert!(SubChunk::from_disk::<Greedy, _>(&mut Cursor::new(bytes.as_slice())).is_err());
    }

    /// A genuinely broken entry still fails, rather than the retry hiding it.
    #[test]
    fn palette_entry_with_wrong_name_type_still_fails() {
        let entry = nbtx::Value::Compound(nbtx::Compound::from_iter([(
            "name".into(),
            nbtx::Value::Int(3),
        )]));
        let bytes = subchunk_with_palette(&entry);
        assert!(SubChunk::from_disk::<Greedy, _>(&mut Cursor::new(bytes.as_slice())).is_err());
    }

    /// Builds a one-layer legacy subchunk with a zero bits-per-index header: no index
    /// words, no palette-length word, just the single entry directly after the header.
    fn subchunk_with_zero_bit_layer(palette_entry: &nbtx::Value) -> Vec<u8> {
        let mut out = vec![SubChunkVersion::Legacy as u8];
        out.push(0); // header byte: 0 bits per index, `>> 1` gives 0x00 (Empty).
        out.extend(nbtx::to_le_bytes(palette_entry).unwrap());
        out
    }

    /// Confirmed against 23 real zero-bit block/liquid layers across the fixture worlds
    /// (`tests/bit_layers.rs`): the header byte is followed directly by one NBT compound,
    /// with no palette-length word and no index words at all.
    #[test]
    fn zero_bit_header_decodes_a_single_entry_palette_and_every_index_is_zero() {
        let entry = entry_with_version(nbtx::Value::Int(17_959_425));
        let bytes = subchunk_with_zero_bit_layer(&entry);
        let chunk = SubChunk::from_disk::<Greedy, _>(&mut Cursor::new(bytes.as_slice())).unwrap();
        let layer = chunk.get_layer(0).unwrap();

        assert_eq!(layer.palette().len(), 1);
        assert_eq!(layer.palette()[0].version, Some(17_959_425));
        for offset in [0usize, 1, 2048, 4095] {
            assert_eq!(layer.array.get(offset), Some(0));
        }
    }

    /// Decoding a zero-bit layer and writing it back out reproduces the original bytes
    /// exactly -- no index words or palette-length word get synthesized on the way out.
    #[test]
    fn zero_bit_header_round_trips_byte_identical() {
        let entry = entry_with_version(nbtx::Value::Int(17_959_425));
        let bytes = subchunk_with_zero_bit_layer(&entry);
        let chunk = SubChunk::from_disk::<Greedy, _>(&mut Cursor::new(bytes.as_slice())).unwrap();

        let mut out = Cursor::new(Vec::new());
        chunk.to_disk::<Greedy, _>(&mut out).unwrap();
        assert_eq!(out.into_inner(), bytes);
    }

    /// A block/liquid layer never carries the biome-only inherit sentinel (header `0x7f`
    /// after `>> 1`) -- confirmed by scanning every subchunk layer of all 14 fixture worlds
    /// in `tests/bit_layers.rs` (zero occurrences). Rejecting it is still checked here, and
    /// the message names the header value that triggered it.
    #[test]
    fn inherit_header_is_rejected_and_names_the_header_value() {
        let mut out = vec![SubChunkVersion::Legacy as u8];
        out.push(0x7f << 1); // header byte: `>> 1` gives the 0x7f inherit sentinel.
        let err = SubChunk::from_disk::<Greedy, _>(&mut Cursor::new(out.as_slice())).unwrap_err();
        assert!(
            err.to_string().contains("0x7f"),
            "error message should name the header value: {err}"
        );
    }

    /// A legacy (version 1) subchunk carries no layer-count byte on disk -- `from_disk`
    /// never reads one for this version, so `to_disk` must not write one either, or the
    /// record grows an extra byte on every round trip.
    #[test]
    fn legacy_subchunk_round_trip_writes_no_layer_count_byte() {
        let entry = entry_with_version(nbtx::Value::Int(17_959_425));
        let bytes = subchunk_with_palette(&entry);
        let chunk = SubChunk::from_disk::<Greedy, _>(&mut Cursor::new(bytes.as_slice())).unwrap();

        let mut out = Cursor::new(Vec::new());
        chunk.to_disk::<Greedy, _>(&mut out).unwrap();
        let out = out.into_inner();

        // Byte 1 is the packed-array header in a legacy record (no layer-count byte
        // between it and the version byte); if a layer-count byte had been written, byte 1
        // would be `1` (one layer) instead.
        assert_eq!(out.len(), bytes.len());
        assert_eq!(
            out[1],
            1 << 1,
            "byte 1 should be the header, not a layer count"
        );
        assert_eq!(out, bytes);
    }

    /// Two `BlockDef`s whose `states` maps hold the same entries, inserted in opposite
    /// order, must be equal and hash equal -- the `Hash` impl must not depend on the
    /// `HashMap`'s unspecified iteration order.
    #[test]
    fn identical_states_hash_equal_regardless_of_insertion_order() {
        use std::collections::hash_map::DefaultHasher;

        let keys = ["a", "b", "c", "d", "e", "f", "g"];

        let mut forward = HashMap::new();
        for (i, k) in keys.iter().enumerate() {
            forward.insert(k.to_string(), nbtx::Value::Int(i as i32));
        }
        let mut backward = HashMap::new();
        for (i, k) in keys.iter().enumerate().rev() {
            backward.insert(k.to_string(), nbtx::Value::Int(i as i32));
        }

        let a = BlockDef {
            name: "minecraft:test".into(),
            version: Some(1),
            states: forward,
        };
        let b = BlockDef {
            name: "minecraft:test".into(),
            version: Some(1),
            states: backward,
        };

        assert_eq!(a, b);

        let mut ha = DefaultHasher::new();
        a.hash(&mut ha);
        let mut hb = DefaultHasher::new();
        b.hash(&mut hb);
        assert_eq!(ha.finish(), hb.finish());
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

        // A legacy subchunk always has exactly one layer and carries no layer-count byte
        // -- `from_disk` never reads one for this version either.
        if self.version != SubChunkVersion::Legacy {
            writer.write_u8(self.layers.len() as u8)?;
        }

        if self.version == SubChunkVersion::Limitless {
            writer.write_i8(self.index)?;
        }

        for layer in &self.layers {
            layer.to_disk(writer)?;
        }

        Ok(())
    }
}
