use std::io::{Cursor, Read};
use std::io::{ErrorKind, Write};
use std::num::NonZeroUsize;

use byteorder::LittleEndian;
use byteorder::{ReadBytesExt, WriteBytesExt};
use smallvec::SmallVec;

use crate::UnpackingMethod;
use crate::bits::{BitArray, IndicesType};
use crate::error::Result;
use crate::greedy::GreedyArray;
use crate::types::BlockPosition;

/// Column count of a 16x16 chunk. Also the heightmap element count in both
/// on-disk layouts, and the biome id count in the 2D layout — the 3D
/// layout's per-fragment biome storage is a full 4096-index array (see
/// [`Biomes3d`]'s doc comment), not this either.
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

    /// The raw packed index at `offset` (see [`crate::subchunk::to_offset`] for the
    /// coordinate convention this shares with the block layer), or `None` if `offset` is
    /// out of range for the underlying array.
    pub fn index_at(&self, offset: usize) -> Option<u16> {
        self.array.get(offset)
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

/// Palette-based per-subchunk biome data (`LevelDB` key tag `0x2b`).
///
/// One [`BiomeEncoding`] fragment per subchunk, ordered bottom to top, each covering that
/// subchunk's full 16x16x16 volume. A [`BiomeEncoding::Palette`] fragment's packed index
/// array is the same 4096-index, `to_offset`-ordered shape the block layer uses (see
/// [`crate::subchunk::to_offset`]), not a smaller 4x4x4 grid replicated across it — a
/// corpus check (comparing neighbouring indices within what would be a 4-block cell)
/// found values that genuinely vary at block granularity in real records, not just at
/// 4-block boundaries, which a true 4x4x4 quantization would rule out.
///
/// **The fragment count is presence-driven, not derived from a dimension's height
/// range.** A corpus survey (`tests/biomes3d_write.rs`) found that real records carry the
/// same fragment count across every dimension of one world, keyed on the world/version,
/// not on the writing record's own dimension: a v1.18-family world's Nether (8 sections)
/// and End (16 sections) records both carry 25 fragments, the same count its Overworld
/// records do (24 sections, plus one); a v1.20.81 world's Nether/End records carry 24,
/// again matching its Overworld. Neither reference implementation consulted while
/// building this derives the fragment count from the dimension's height range either —
/// this is stated as an observed corpus fact, not sourced from either one. **Do not
/// compute a `Biomes3d`'s fragment count from `crate::height_range` or a dimension's
/// bounds** — on a real record that undercounts (for Nether/End) or is off by exactly one
/// (the v1.18-family's 25-vs-24 quirk), [`Biomes3d::new`] would then reject data this
/// format's own writer produced. This is also why [`Biomes3d`] carries no `storage_count`
/// field or parameter anywhere on this type: there is no dimension-derived number that is
/// safe to validate fragment lists against.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Biomes3d {
    heightmap: Box<[u16; COLUMNS]>,
    fragments: SmallVec<[BiomeEncoding; 1]>,
}

/// A [`Biomes3d::new`] fragment list that violates the write-semantics rule the corpus
/// survey in `tests/biomes3d_write.rs` confirmed (see that test's module doc for the
/// numbers): the fragment list may not be empty, and its first entry may never be
/// [`BiomeEncoding::Inherit`] (there is nothing below the bottom subchunk to inherit
/// from — zero of 44,245 real fragments across the corpus put one there). There is no
/// upper-bound rule: see [`Biomes3d`]'s doc comment for why an upper bound is not
/// something this type validates against.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Biomes3dError {
    /// The fragment list has no entries at all.
    #[error("biome fragment list is empty")]
    Empty,
    /// The first fragment is `Inherit`, with no subchunk below it to inherit from.
    #[error("first biome fragment is Inherit, but there is no subchunk below it")]
    FirstFragmentInherits,
}

impl Biomes3d {
    /// This chunk's heightmap.
    #[inline]
    pub fn heightmap(&self) -> &[u16; COLUMNS] {
        &self.heightmap
    }

    /// Write-back access to this chunk's heightmap -- see [`HeightMap::set_heights`] for
    /// why a setter exists alongside the shared borrow rather than only a `_mut` accessor.
    #[inline]
    pub fn set_heights(&mut self, heights: [u16; COLUMNS]) {
        *self.heightmap = heights;
    }

    /// Mutable access to this chunk's heightmap, for an in-place partial update (a caller
    /// recomputing only some columns, say) that [`Self::set_heights`]' whole-array
    /// replacement does not fit. Named to match [`HeightMap::heights_mut`].
    #[inline]
    pub fn heights_mut(&mut self) -> &mut [u16; COLUMNS] {
        &mut self.heightmap
    }

    /// The per-subchunk biome fragments, ordered bottom to top.
    #[inline]
    pub fn fragments(&self) -> &[BiomeEncoding] {
        &self.fragments
    }

    /// Builds a validated [`Biomes3d`] from a heightmap and an explicit fragment list.
    ///
    /// Validates only the two rules [`Biomes3dError`] documents: the list must not be
    /// empty, and its first entry must not be [`BiomeEncoding::Inherit`]. There is
    /// deliberately no upper bound on `fragments.len()` -- see [`Biomes3d`]'s doc comment
    /// for why a dimension-derived bound would reject real data.
    ///
    /// # Errors
    ///
    /// Returns [`Biomes3dError::Empty`] if `fragments` is empty, or
    /// [`Biomes3dError::FirstFragmentInherits`] if its first entry is
    /// [`BiomeEncoding::Inherit`].
    pub fn new(
        heightmap: Box<[u16; COLUMNS]>,
        fragments: SmallVec<[BiomeEncoding; 1]>,
    ) -> std::result::Result<Biomes3d, Biomes3dError> {
        if fragments.is_empty() {
            return Err(Biomes3dError::Empty);
        }
        if matches!(fragments[0], BiomeEncoding::Inherit) {
            return Err(Biomes3dError::FirstFragmentInherits);
        }

        Ok(Biomes3d {
            heightmap,
            fragments,
        })
    }

    /// Promotes a pre-3D-biome column record ([`Biomes2d`], paired with its own
    /// [`HeightMap`]-half heightmap) to the 3D layout, emitting `fragment_count`
    /// fragments: a bottom fragment carrying the real data (a [`BiomeEncoding::Single`]
    /// if every column shares one id, a [`BiomeEncoding::Palette`] otherwise, replicating
    /// each column's id across all 16 of that subchunk's Y layers), and
    /// [`BiomeEncoding::Inherit`] for every fragment above it.
    ///
    /// This whole-column shape -- one real fragment, everything above it `Inherit` --
    /// is a valid encoding this format supports (it round-trips through
    /// [`Self::to_disk`]/[`Self::from_disk`] like any other record) and is the natural
    /// encoding for data with no vertical variation, which is exactly what a promoted 2D
    /// column record is. It is not, however, a shape the corpus actually contains: of the
    /// 44,245 real `0x2b` fragments surveyed, not one uniform-biome column (`Single`
    /// bottom, `Inherit` above, all the way to the top) occurs, and only 6 of 7,795 real
    /// records are `Palette`-bottom-then-all-`Inherit`. `Inherit` itself is pervasive in
    /// the corpus (41,412 mid-list, 2,833 trailing) as a *technique* real writes use
    /// constantly to avoid re-storing an unchanged value going up -- it is just essentially
    /// never applied to a *whole* column this way, because real terrain's biome varies
    /// somewhere above ground even when a naive promotion from 2D data would not.
    ///
    /// `fragment_count` cannot be zero (a biome record with no storage at all is not a
    /// shape this format has) -- enforced by taking [`NonZeroUsize`] rather than returning
    /// a [`Biomes3dError`], so this and [`Self::new`] agree on what "no fragments" means
    /// without this infallible constructor needing a fallible signature.
    pub fn from_columns(
        heightmap: Box<[u16; COLUMNS]>,
        biomes: &Biomes2d,
        fragment_count: NonZeroUsize,
    ) -> Biomes3d {
        let ids = biomes.ids();
        let first_id = u32::from(ids[0]);
        let uniform = ids.iter().all(|&id| u32::from(id) == first_id);

        let bottom = if uniform {
            BiomeEncoding::Single(first_id)
        } else {
            BiomeEncoding::Palette(column_biome_array(ids))
        };

        let fragment_count = fragment_count.get();
        let mut fragments = SmallVec::with_capacity(fragment_count);
        fragments.push(bottom);
        for _ in 1..fragment_count {
            fragments.push(BiomeEncoding::Inherit);
        }

        Biomes3d {
            heightmap,
            fragments,
        }
    }

    /// Resolves the biome id at `position` (each axis `0..16`, block-granular within the
    /// subchunk) inside the subchunk at `subchunk_index` (`0`-based, counting up from this
    /// record's bottom-most stored subchunk -- the same order [`Self::fragments`] iterates
    /// in).
    ///
    /// `subchunk_index` is supplied by the caller rather than carried on `Biomes3d`
    /// itself: this type is the on-disk format only, with no notion of a dimension or a
    /// minimum Y (that lives in `bedrock_shared::world::dimension` and
    /// `crate::height_range`, both outside this module's concern), and per [`Biomes3d`]'s
    /// doc comment there is no dimension-derived count this method could validate against
    /// even if it wanted to -- resolving which absolute subchunk maps to which
    /// `subchunk_index` is a level-layer job for a later wave.
    ///
    /// There is no upper bound on `subchunk_index`: any index at or past
    /// [`Self::fragments`]' length (a storage omitted from the tail, or simply a query
    /// higher than this record happens to stock) resolves against the topmost *stored*
    /// fragment, per this format's fill-up rule -- unexercised by real writes (the corpus
    /// survey found `tail_omitted == 0`, pinned in `tests/biomes3d_write.rs`), but a
    /// decode-time affordance this method still honors.
    ///
    /// Returns `None` when any coordinate in `position` is out of range, the fragment
    /// list is empty, a walk down an `Inherit` chain reaches fragment 0 and finds it is
    /// itself `Inherit` (malformed data [`Self::new`]/[`Self::from_columns`] cannot
    /// produce, but a value decoded from disk is not re-validated this way -- decode never
    /// rejects malformed data that only a builder can prevent), or (for a
    /// [`BiomeEncoding::Palette`] fragment) the resolved palette index is out of range for
    /// that fragment's own palette. The last two cases are malformed data, not a normal
    /// miss, but this method has no error channel to report through and `None` is the
    /// honest "could not resolve" answer either way.
    pub fn biome_at(
        &self,
        subchunk_index: usize,
        position: impl Into<BlockPosition>,
    ) -> Option<u32> {
        let BlockPosition(x, y, z) = position.into();
        if x >= 16 || y >= 16 || z >= 16 {
            return None;
        }
        let offset = 256 * x as usize + 16 * z as usize + y as usize;

        let last_stored = self.fragments.len().checked_sub(1)?;
        let mut index = subchunk_index.min(last_stored);
        loop {
            match self.fragments.get(index)? {
                BiomeEncoding::Inherit => index = index.checked_sub(1)?,
                BiomeEncoding::Single(id) => return Some(*id),
                BiomeEncoding::Palette(array) => {
                    let palette_index = array.index_at(offset)? as usize;
                    return array.palette().get(palette_index).copied();
                }
            }
        }
    }

    // This roughly estimates the size of the output buffer. Chunks with many paletted fragments
    // will be larger than this estimate while chunks with many inherited biome fragments will be smaller.
    pub fn size_hint(&self) -> usize {
        const HEIGHTMAP_SIZE: usize = COLUMNS * 2;
        HEIGHTMAP_SIZE + self.fragments.len() * std::mem::size_of::<BiomeEncoding>()
    }

    /// Writes this record in the `0x2b` on-disk layout.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to `writer` fails.
    pub fn to_disk<W>(&self, writer: &mut Cursor<W>) -> Result<()>
    where
        Cursor<W>: Write,
    {
        const EMPTY_FLAG: u8 = 0x00;
        const INHERIT_FLAG: u8 = 0x7f;
        // A biome storage header's low bit -- discarded by `BitArray::from_disk`'s `>> 1`
        // on the way in for every header, block/liquid and biome alike, so it carries no
        // information this crate decodes -- is nonetheless always set to 1 in real biome
        // records: a corpus scan of every `0x2b` fragment header across the fixtures found
        // it set on all 13,171 Single, 6,985 Data, and 10,869 Inherit fragments observed,
        // with zero exceptions. Block/liquid layer headers are the opposite case (always
        // 0 -- see `bits.rs`'s own corpus-checked width assertions), so this is a real
        // per-storage-kind convention, not noise, and it means a biome header always ends
        // in an odd byte where the equivalent block/liquid header ends in an even one.
        const BIOME_HEADER_LOW_BIT: u8 = 1;

        writer.write_all(bytemuck::cast_slice::<u16, u8>(self.heightmap.as_slice()))?;

        for fragment in &self.fragments {
            match fragment {
                BiomeEncoding::Inherit => {
                    writer.write_u8((INHERIT_FLAG << 1) | BIOME_HEADER_LOW_BIT)?;
                }
                BiomeEncoding::Single(v) => {
                    writer.write_u8((EMPTY_FLAG << 1) | BIOME_HEADER_LOW_BIT)?;
                    writer.write_u32::<LittleEndian>(*v)?;
                }
                BiomeEncoding::Palette(v) => {
                    // `BitArray::to_disk` writes a correct header/word layout for the
                    // block/liquid low-bit convention (0); reuse it for the packed-word
                    // encoding, which does not depend on the low bit at all, and patch
                    // the one bit that differs for this storage kind afterward rather
                    // than duplicating the width-selection and packing logic here.
                    let mut packed = Cursor::new(Vec::new());
                    v.array.to_disk::<Vec<u8>>(&mut packed, v.palette.len())?;
                    let mut packed = packed.into_inner();
                    packed[0] |= BIOME_HEADER_LOW_BIT;
                    writer.write_all(&packed)?;

                    #[expect(
                        clippy::cast_possible_truncation,
                        reason = "a biome palette holds at most 4096 entries"
                    )]
                    let palette_len = v.palette.len() as u32;
                    writer.write_u32::<LittleEndian>(palette_len)?;

                    for entry in &v.palette {
                        writer.write_u32::<LittleEndian>(*entry)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Reads a record in the `0x2b` on-disk layout. The fragment list ends where `reader`
    /// runs out while reading a fragment's header or packed indices.
    ///
    /// # Errors
    ///
    /// Returns an error if `reader` ends inside the heightmap, a palette or a single biome
    /// id, if a fragment header names a bit size that is not valid, or if reading from
    /// `reader` fails for a reason other than running out of data.
    pub fn from_disk<M: UnpackingMethod, R>(reader: &mut Cursor<R>) -> Result<Biomes3d>
    where
        Cursor<R>: Read,
    {
        let mut heightmap: Box<[u16; COLUMNS]> = Box::new([0; COLUMNS]);

        let heightmap_bytes = bytemuck::cast_slice_mut::<u16, u8>(heightmap.as_mut());
        reader.read_exact(heightmap_bytes)?;

        let mut fragments = SmallVec::new();
        loop {
            // The record ends where a storage would start; running out of bytes inside one
            // is an error, not the end of the record.
            let start = reader.position();
            match reader.read_u8() {
                Ok(_) => reader.set_position(start),
                Err(err) if err.kind() == ErrorKind::UnexpectedEof => break,
                Err(err) => return Err(err.into()),
            }
            let indices = BitArray::from_disk::<M, _>(reader)?;

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
                    fragments.push(BiomeEncoding::Single(single));
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

/// Builds a [`BiomeArray`] for [`Biomes3d::from_columns`]'s varied-column case: one
/// packed index per block position (the same 4096-entry, `to_offset`-ordered shape a
/// real `Palette` fragment uses, per [`Biomes3d`]'s own doc comment), replicating each
/// column's id across all 16 Y layers of the subchunk -- the promoted data has no
/// vertical variation, only the horizontal variation the source [`Biomes2d`] already had.
fn column_biome_array(ids: &[u8; COLUMNS]) -> BiomeArray {
    let mut palette: Vec<u32> = Vec::new();
    let mut indices: Box<[u16; 4096]> = Box::new([0; 4096]);

    for x in 0..16usize {
        for z in 0..16usize {
            let id = u32::from(ids[z * 16 + x]);
            let palette_index = palette.iter().position(|&v| v == id).unwrap_or_else(|| {
                palette.push(id);
                palette.len() - 1
            });
            #[expect(
                clippy::cast_possible_truncation,
                reason = "the palette holds at most one entry per column, so at most 256"
            )]
            let palette_index = palette_index as u16;

            for y in 0..16usize {
                indices[256 * x + 16 * z + y] = palette_index;
            }
        }
    }

    BiomeArray {
        array: BitArray::Greedy(GreedyArray::from(indices)),
        palette,
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

    /// Writes the per-column biome ids.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to `writer` fails.
    pub fn to_disk<W>(&self, writer: &mut Cursor<W>) -> Result<()>
    where
        Cursor<W>: Write,
    {
        writer.write_all(self.ids.as_slice())?;

        Ok(())
    }

    /// Reads the per-column biome ids.
    ///
    /// # Errors
    ///
    /// Returns an error if `reader` holds fewer than 256 bytes.
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

    /// Writes the biome data in its own on-disk layout.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to `writer` fails.
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
    ///
    /// # Errors
    ///
    /// Returns an error under the same conditions as [`Biomes3d::from_disk`].
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
        for (id, value) in ids.iter_mut().zip((0..=u8::MAX).cycle()) {
            *id = value;
        }
        Biomes2d { ids }
    }

    fn sample_3d() -> Biomes3d {
        Biomes3d {
            heightmap: Box::new([0; COLUMNS]),
            fragments: SmallVec::from_buf([BiomeEncoding::Single(1)]),
        }
    }

    /// A `0x2b` record with a single-biome storage, then a one-bit paletted storage.
    fn two_storage_biomes_3d() -> Vec<u8> {
        let mut out = vec![0; COLUMNS * 2];
        out.extend([0x01, 1, 0, 0, 0]);
        out.push(1 << 1 | 1);
        out.extend([0; 128 * 4]);
        out.extend(1u32.to_le_bytes());
        out.extend(7u32.to_le_bytes());
        out
    }

    /// The record ends where a storage would start. A value cut inside a storage -- after
    /// its header, inside its words, inside its palette -- is an error, not a shorter
    /// record that would be written back without the cut storage.
    #[test]
    fn biomes_3d_cut_inside_a_storage_is_an_error() {
        let bytes = two_storage_biomes_3d();
        let whole =
            Biomes3d::from_disk::<crate::Greedy, _>(&mut Cursor::new(bytes.as_slice())).unwrap();
        assert_eq!(whole.fragments().len(), 2);

        let second = COLUMNS * 2 + 5;
        for cut in [second + 1, second + 1 + 100, bytes.len() - 2] {
            let result = Biomes3d::from_disk::<crate::Greedy, _>(&mut Cursor::new(&bytes[..cut]));
            assert!(result.is_err(), "cut at {cut} read as {result:?}");
        }

        let shorter =
            Biomes3d::from_disk::<crate::Greedy, _>(&mut Cursor::new(&bytes[..second])).unwrap();
        assert_eq!(shorter.fragments(), &whole.fragments()[..1]);
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
