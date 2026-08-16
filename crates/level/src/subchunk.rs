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
    ///
    /// Order-preserving (an [`indexmap::IndexMap`], not a [`HashMap`]): a
    /// real `states` compound routinely carries more than one key (up to six
    /// in the fixture corpus), and the game writes them in a fixed order
    /// per block, not alphabetically or by insertion into some other
    /// structure. `nbtx`'s struct/map codec both reads and writes a map
    /// field in the same order -- entries are appended to the map as they
    /// are read off the wire, and written back out in the map's iteration
    /// order -- so this alone reproduces `states`' on-disk key order with no
    /// extra bookkeeping. [`HashMap`]'s iteration order is unspecified and
    /// would scramble it.
    #[facet(default)]
    pub states: indexmap::IndexMap<String, nbtx::Value>,
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

/// A pre-flattening palette entry: `name` plus an opaque `val` short, and no `states`
/// compound at all. Its own private decode target -- distinct from [`BlockDef`] rather
/// than a variant of it, because [`BlockDef`]'s field set is a public type other code in
/// this crate (`ItemStack::block`, `FlowerPot::plant_block`) already relies on, and this
/// format has nothing in common with it beyond `name`.
///
/// Confirmed against every one of `v1_12`'s 6,246 real legacy entries (the corpus's one
/// pre-flattening fixture): the compound is always exactly these two fields, `name` before
/// `val`, uniformly -- no entry carries `version` or `states`, none reverse the field
/// order, and `val` is always in `[0, 15]` (a nibble range, consistent with a pre-
/// flattening block's data value). So unlike [`BlockDef`]'s `version`/`states` ordering,
/// there is nothing here for [`FieldOrder`] to track: a fixed struct field order already
/// matches every real record.
#[derive(Debug, Clone, Facet)]
#[cfg_attr(
    not(feature = "deny-unknown-fields"),
    facet(nbtx::allow_unknown_fields)
)]
struct LegacyBlockDef {
    name: String,
    val: i16,
}

/// One entry in a subchunk layer's palette: either the modern, flattened form
/// (`name` plus optional `version` and `states`) or the pre-flattening
/// `{name, val}` form a legacy world's palette stores instead.
///
/// A `Vec<PaletteEntry>` rather than folding `val` into [`BlockDef`] as an extra optional
/// field: `BlockDef` is a public type other decoded structs embed directly
/// (`ItemStack::block`, `FlowerPot::plant_block`), and those never carry a legacy `val` --
/// they are always modern flattened states. Adding a field there would widen every one of
/// those call sites for a case that cannot occur in them. Keeping the two forms as
/// variants of one palette-entry type instead means a layer's palette is one first-class
/// list covering both eras, with no second parallel palette or optional-legacy-list
/// bolted on beside it.
///
/// `val` is carried opaquely -- an `i16`, not interpreted as a block/data-value pair or
/// resolved against any table. Deciding what a given `(name, val)` pair actually means is
/// the upgrade provider's job (Phase 3), not this type's.
#[derive(Debug, Clone, PartialEq)]
pub enum PaletteEntry {
    /// The modern, flattened form.
    Modern(BlockDef),
    /// The pre-flattening form: a block name plus its opaque data value.
    Legacy {
        /// Name of the block.
        name: String,
        /// The on-disk data value. Opaque -- see this type's doc comment.
        val: i16,
    },
}

impl PaletteEntry {
    /// This entry's block name, regardless of which form it is.
    pub fn name(&self) -> &str {
        match self {
            Self::Modern(block) => &block.name,
            Self::Legacy { name, .. } => name,
        }
    }

    /// Borrows the modern form, or `None` if this is a legacy `{name, val}` entry.
    pub fn as_block(&self) -> Option<&BlockDef> {
        match self {
            Self::Modern(block) => Some(block),
            Self::Legacy { .. } => None,
        }
    }

    /// The legacy data value, or `None` if this is a modern entry.
    ///
    /// See this type's doc comment: opaque, not interpreted here.
    pub fn legacy_val(&self) -> Option<i16> {
        match self {
            Self::Modern(_) => None,
            Self::Legacy { val, .. } => Some(*val),
        }
    }
}

impl Hash for BlockDef {
    /// Hashes this block.
    ///
    /// `states` is order-preserving (see its doc comment) precisely so a decoded entry
    /// re-encodes with the key order it was read in, but two maps holding the same entries
    /// in different orders must still be the same block for every purpose *other than*
    /// re-encoding -- palette deduplication (`Layer::set`) has to recognize them as one
    /// entry regardless of which order either was built in. Hashing in iteration order
    /// would make this value's hash depend on that order, breaking the rule that equal
    /// values hash equally. Each entry is hashed on its own with a fresh hasher and the
    /// resulting values combined with a commutative fold (wrapping add), so the result
    /// depends only on the entry set, never on the order iteration happens to visit it in,
    /// without collecting the map into an intermediate `Vec` to sort first.
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

impl Hash for PaletteEntry {
    /// Delegates to [`BlockDef`]'s `Hash` for the modern form. For the legacy form, hashes
    /// `name` together with `val`: `val` is a legacy entry's content identity, the legacy
    /// analogue of `states` (`minecraft:wool` at different `val`s is a different colour, the
    /// same way two `states` compounds with different entries are different blocks) -- unlike
    /// `BlockDef`'s own `Hash`, which excludes `version` because `version` is a provenance
    /// stamp, not part of what block a `BlockDef` names.
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Modern(block) => block.hash(state),
            Self::Legacy { name, val } => {
                state.write(name.as_bytes());
                state.write_i16(*val);
            }
        }
    }
}

/// The order a palette entry's `version` and `states` keys were written in
/// the on-disk NBT compound, as a top-level sibling of `name`.
///
/// `#[derive(Facet)]`-driven struct decode/encode both use the struct's fixed
/// field declaration order (`name, version, states`), because `nbtx` has no
/// mechanism to remember or replay the order a struct's keys arrived in --
/// unlike a map field (see [`BlockDef::states`]), a struct's shape fixes its
/// field order at compile time. The game does not follow one order
/// consistently: a corpus scan of every `0x2f` record across all fourteen
/// imported fixtures plus the 1.26 seed world (585,804 modern
/// name/version/states entries) found eleven of the fourteen fixtures write
/// every record `name, states, version`; `v1_18_30`, `v1_19_30` and
/// `v1_20_81` write both orders, freely mixed between different palette
/// records within the same world. So no single struct field order matches
/// every record, and reordering `BlockDef`'s fields would only have fixed
/// the eleven uniform fixtures. `name` leads in every real record observed,
/// and among the 43,733 real records carrying more than one modern entry,
/// every entry in a given record agreed on the order of the other two --
/// order is a per-record property, never mixed within one record's own
/// palette. Recording this one bit per entry is what makes a byte-identical
/// re-encode possible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum FieldOrder {
    /// `name, states, version` -- the order the majority of real records use
    /// (480,372 of 585,804 modern entries surveyed), and what a freshly
    /// built entry not read from disk is written as.
    #[default]
    StatesThenVersion,
    /// `name, version, states` -- `BlockDef`'s own struct field order.
    VersionThenStates,
}

impl FieldOrder {
    /// Reads the order of a decoded entry's `version` and `states` keys from
    /// its dynamic form, before it is converted into a [`BlockDef`] (which
    /// does not retain it on its own).
    fn of(value: &nbtx::Value) -> Self {
        let nbtx::Value::Compound(compound) = value else {
            return Self::default();
        };
        // `name` leads every real record's compound (corpus-checked, see this type's doc
        // comment); this only tracks `version`/`states`' relative order on that assumption,
        // so a violation here would silently mispredict the order rather than error --
        // surfaced in a debug build rather than left invisible.
        debug_assert!(
            matches!(compound.get_index_of(b"name".as_slice()), None | Some(0)),
            "a palette entry's `name` key is not first in the compound"
        );
        let version_pos = compound.get_index_of(b"version".as_slice());
        let states_pos = compound.get_index_of(b"states".as_slice());
        match (version_pos, states_pos) {
            (Some(v), Some(s)) if v < s => Self::VersionThenStates,
            _ => Self::default(),
        }
    }
}

/// Whether a decoded palette entry compound is the pre-flattening `{name, val}` form
/// rather than the modern `name`/`version`/`states` one.
///
/// `val` present and `states` absent is what distinguishes the two: every real legacy
/// entry carries `val` and never `states` (and vice versa for modern entries -- see
/// [`LegacyBlockDef`]'s doc comment), so checking both rather than just one guards against
/// a hypothetical record that carries `val` alongside a full modern shape, which would
/// otherwise be misrouted into the legacy decode path and lose `states`/`version`.
fn is_legacy_entry(value: &nbtx::Value) -> bool {
    let nbtx::Value::Compound(compound) = value else {
        return false;
    };
    compound.contains_key(b"val".as_slice()) && !compound.contains_key(b"states".as_slice())
}

/// Decodes one palette entry, keeping the order a modern entry's `version`/`states` keys
/// were written in (see [`FieldOrder`]). A legacy `{name, val}` entry has no such order to
/// keep (see [`LegacyBlockDef`]'s doc comment), so the returned [`FieldOrder`] is
/// meaningless for it -- [`write_palette_entry`] never consults it in that case.
///
/// Goes through [`nbtx::Value`] rather than decoding straight into
/// [`BlockDef`]: a `Value::Compound` is order-preserving (an `IndexMap`
/// under the hood), so the key order survives long enough to inspect, and
/// [`nbtx::from_value`] then does the same name-matching, default-filling,
/// unknown-field handling the direct byte decode would have -- reused rather
/// than duplicated.
fn read_palette_entry<R: Read>(reader: &mut R) -> Result<(PaletteEntry, FieldOrder)> {
    // No fallback or `lenient_width` here: every modern palette entry across the whole
    // real test world (every layer of every subchunk) has `version` written as an `Int`,
    // so there is nothing to reconcile, and this runs once per entry.
    let value: nbtx::Value = nbtx::from_le_bytes(reader)?;
    if is_legacy_entry(&value) {
        let legacy: LegacyBlockDef = nbtx::from_value(value)?;
        return Ok((
            PaletteEntry::Legacy {
                name: legacy.name,
                val: legacy.val,
            },
            FieldOrder::default(),
        ));
    }
    let order = FieldOrder::of(&value);
    let entry = nbtx::from_value(value)?;
    Ok((PaletteEntry::Modern(entry), order))
}

/// Encodes one palette entry in the given field order (a modern entry's `version`/`states`
/// order -- ignored for a legacy entry, which has a fixed two-field shape).
///
/// For the modern form, [`nbtx::to_value`] converts `entry` to a `Value::Compound` in
/// `BlockDef`'s struct order (`name, version, states` -- `version` omitted if `None`).
/// That is already [`FieldOrder::VersionThenStates`]; for the other order,
/// moving `version` to the end of the compound (a no-op if it was already
/// absent) leaves `states` before it, matching what [`FieldOrder::of`]
/// detects on the way back in.
fn write_palette_entry<W: Write>(
    writer: &mut W,
    entry: &PaletteEntry,
    order: FieldOrder,
) -> Result<()> {
    match entry {
        PaletteEntry::Modern(block) => {
            let value = nbtx::to_value(block)?;
            let nbtx::Value::Compound(mut compound) = value else {
                unreachable!("a struct always converts to a compound")
            };
            if order == FieldOrder::StatesThenVersion
                && let Some(version) = compound.shift_remove(b"version".as_slice())
            {
                compound.insert("version".into(), version);
            }
            nbtx::to_le_bytes_in(writer, &nbtx::Value::Compound(compound))?;
        }
        PaletteEntry::Legacy { name, val } => {
            // Built directly as a `Value::Compound` rather than through `LegacyBlockDef` and
            // `nbtx::to_value` (as the modern arm above does): `LegacyBlockDef::name` is an
            // owned `String`, so populating one from this `&str` borrow to hand it to
            // `to_value` would clone `name` once into the temporary struct and then again
            // when the Facet serializer turns that into a `Value::String`. Writing the
            // compound by hand still copies `name`'s bytes exactly once, into the `Value`
            // tree, the same single allocation the modern arm's `to_value(block)` pays
            // internally for its own `String` fields.
            let value = nbtx::Value::Compound(nbtx::Compound::from_iter([
                ("name".into(), nbtx::Value::String(name.as_str().into())),
                ("val".into(), nbtx::Value::Short(*val)),
            ]));
            nbtx::to_le_bytes_in(writer, &value)?;
        }
    }
    Ok(())
}

/// Iterates over all palette entries in a layer, one per block position. Each yielded entry
/// resolves to the palette, so it may be a legacy `{name, val}` entry ([`PaletteEntry::Legacy`])
/// rather than a resolved modern block -- see [`PaletteEntry`].
pub struct LayerIter<'l> {
    /// An iterator over the indices
    array_iter: BitArrayIter<'l>,
    /// The palette.
    palette: &'l [PaletteEntry],
}

impl<'l> Iterator for LayerIter<'l> {
    type Item = &'l PaletteEntry;

    fn next(&mut self) -> Option<&'l PaletteEntry> {
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
#[derive(Debug, Clone)]
pub struct Layer {
    /// List of indices into the palette.
    ///
    /// Coordinates can be converted to an offset into the array using [`to_offset`].
    array: BitArray,
    /// List of all different block types in this sub chunk layer -- both eras a real
    /// palette can hold, see [`PaletteEntry`].
    palette: Vec<PaletteEntry>,
    /// Used to check which blocks are already in the palette. Block definitions are hashed manually and compared with their hash in this set.
    /// This is because `PaletteEntry` does not implement `Eq` and we also prevent cloning the entire block definition on each insertion.
    hashes: HashMap<u64, u16, BuildNoHashHasher<u64>>,
    /// The field order each *modern* `palette` entry re-encodes with, kept in lock step
    /// with `palette` by index -- both are only ever pushed to, together, in `set` and
    /// `from_disk`, never independently. See [`FieldOrder`]. Meaningless (and never read)
    /// at an index whose `palette` entry is [`PaletteEntry::Legacy`] -- a legacy entry has
    /// a fixed field order, nothing for this to track.
    entry_order: Vec<FieldOrder>,
}

impl PartialEq for Layer {
    /// Compares `array` and `palette` only. `hashes` is a cache fully determined by
    /// `palette` (same hash function, so it never disagrees with a `palette` comparison
    /// on its own), and `entry_order` is on-disk serialization metadata, not part of a
    /// block's identity -- two layers holding the same blocks are equal regardless of
    /// which order either happened to read or write them in.
    fn eq(&self, other: &Self) -> bool {
        self.array == other.array && self.palette == other.palette
    }
}

impl Layer {
    const HASH_SEED: usize = 0;

    /// Whether this layer uses the lazy unpacking strategy.
    #[inline]
    pub fn is_lazy(&self) -> bool {
        self.array.is_lazy()
    }

    /// Retrieves the palette entry at `position`. The entry may be a legacy `{name, val}`
    /// entry ([`PaletteEntry::Legacy`]) rather than a resolved modern block, if this layer
    /// was decoded from a pre-flattening world -- see [`PaletteEntry`].
    pub fn get<K: Into<BlockPosition>>(&self, position: K) -> Option<&PaletteEntry> {
        let pos = position.into();
        let offset = to_offset(pos);
        let index = self.array.get(offset)?;
        Some(&self.palette[index as usize])
    }

    /// Sets the block at `position` to `block`.
    ///
    /// Always inserts (or reuses) a modern palette entry -- there is no public way to
    /// construct a [`PaletteEntry::Legacy`], since that form only ever arrives by decoding
    /// an old world (see [`PaletteEntry`]'s doc comment).
    pub fn set<K: Into<BlockPosition>>(&mut self, position: K, block: BlockDef) {
        // Check whether the block is in the palette
        let hash = Self::hash_def(&block);
        let palette_index = *self.hashes.entry(hash).or_insert_with(|| {
            // Block does not exist in palette, push it. A newly built entry was not read
            // off disk, so it gets the default (and dominant) field order.
            self.palette.push(PaletteEntry::Modern(block));
            self.entry_order.push(FieldOrder::default());
            self.palette.len() as u16 - 1
        });

        let index = to_offset(position.into());
        self.array.set(index, palette_index);
    }

    /// Computes the hash of a modern block, for looking it up against this layer's
    /// palette (which may also hold legacy entries -- [`Self::hash_entry`] covers both).
    pub(crate) fn hash_def(block: &BlockDef) -> u64 {
        let mut state = FxHasher::with_seed(Self::HASH_SEED);
        block.hash(&mut state);
        state.finish()
    }

    /// Computes the hash of a palette entry, modern or legacy, the way [`Self::from_disk`]
    /// populates `hashes` for both -- see [`PaletteEntry`]'s `Hash` impl for what each form
    /// hashes over.
    fn hash_entry(entry: &PaletteEntry) -> u64 {
        let mut state = FxHasher::with_seed(Self::HASH_SEED);
        entry.hash(&mut state);
        state.finish()
    }

    /// Determines the index in the palette of the (modern) block.
    pub fn palette_index(&self, block: &BlockDef) -> Option<u16> {
        let hash = Self::hash_def(block);
        self.hashes.get(&hash).copied()
    }

    /// Whether the palette contains the given (modern) block.
    pub fn contains(&self, block: &BlockDef) -> bool {
        let hash = Self::hash_def(block);
        self.hashes.contains_key(&hash)
    }

    /// Returns the palette used for this chunk
    #[inline]
    pub fn palette(&self) -> &[PaletteEntry] {
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
        let (palette, entry_order): (Vec<PaletteEntry>, Vec<FieldOrder>) =
            if let BitArray::Empty = array {
                let (entry, order) = read_palette_entry(reader)?;
                (vec![entry], vec![order])
            } else {
                let len = reader.read_u32::<LittleEndian>()? as usize;
                let mut palette = Vec::with_capacity(len);
                let mut entry_order = Vec::with_capacity(len);

                for _ in 0..len {
                    let (entry, order) = read_palette_entry(reader)?;
                    palette.push(entry);
                    entry_order.push(order);
                }

                (palette, entry_order)
            };

        let mut hashes =
            HashMap::with_capacity_and_hasher(palette.len(), BuildNoHashHasher::default());
        hashes.extend(palette.iter().enumerate().map(|(i, entry)| {
            let hash = Self::hash_entry(entry);
            (hash, i as u16)
        }));

        Ok(Self {
            array,
            hashes,
            palette,
            entry_order,
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
            write_palette_entry(writer, &self.palette[0], self.entry_order[0])?;
            return Ok(());
        }

        writer.write_u32::<LittleEndian>(plen as u32)?;
        for (entry, &order) in self.palette.iter().zip(&self.entry_order) {
            write_palette_entry(writer, entry, order)?;
        }

        Ok(())
    }

    /// Creates an iterator over the palette entries in this layer, one per block position,
    /// resolved through the palette -- see [`LayerIter`] for what a yielded entry can be.
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
    type Item = &'a PaletteEntry;
    type IntoIter = LayerIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        LayerIter::from(self)
    }
}

impl<I> Index<I> for Layer
where
    I: Into<BlockPosition>,
{
    type Output = PaletteEntry;

    /// Indexes into this layer's resolved palette entries -- the entry returned may be a
    /// legacy `{name, val}` entry ([`PaletteEntry::Legacy`]) rather than a resolved modern
    /// block, if this layer was decoded from a pre-flattening world -- see [`PaletteEntry`].
    ///
    /// # Panics
    ///
    /// This function panics if the given position is out of range.
    /// In other words, it requires that `x <= 16`, `y <= 16` and `z <= 16`.
    fn index(&self, position: I) -> &PaletteEntry {
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
        let entry = &chunk.get_layer(0).unwrap().palette()[0];
        assert_eq!(entry.as_block().unwrap().version, Some(17_959_425));
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

    /// A palette entry compound with `name`, `version` and `states` in the given order --
    /// unlike [`entry_with_version`], which always writes `BlockDef`'s own struct order.
    fn entry_with_order(
        order: FieldOrder,
        version: nbtx::Value,
        states: nbtx::Compound,
    ) -> nbtx::Value {
        let mut c = nbtx::Compound::new();
        c.insert("name".into(), nbtx::Value::String("minecraft:stone".into()));
        match order {
            FieldOrder::StatesThenVersion => {
                c.insert("states".into(), nbtx::Value::Compound(states));
                c.insert("version".into(), version);
            }
            FieldOrder::VersionThenStates => {
                c.insert("version".into(), version);
                c.insert("states".into(), nbtx::Value::Compound(states));
            }
        }
        nbtx::Value::Compound(c)
    }

    /// Both real on-disk orders of the top-level `version`/`states` keys round-trip
    /// byte-identically -- not just the order that happens to match `BlockDef`'s own
    /// struct field order.
    #[test]
    fn palette_entry_round_trips_byte_identical_regardless_of_field_order() {
        for order in [FieldOrder::StatesThenVersion, FieldOrder::VersionThenStates] {
            let entry =
                entry_with_order(order, nbtx::Value::Int(17_959_425), nbtx::Compound::new());
            let bytes = subchunk_with_palette(&entry);

            let chunk =
                SubChunk::from_disk::<Greedy, _>(&mut Cursor::new(bytes.as_slice())).unwrap();
            let mut out = Cursor::new(Vec::new());
            chunk.to_disk::<Greedy, _>(&mut out).unwrap();

            assert_eq!(
                out.into_inner(),
                bytes,
                "{order:?} did not round-trip byte-identically"
            );
        }
    }

    /// A `states` compound with more than one key keeps the exact key order it was read
    /// in, in both directions -- `states` being order-preserving (see its doc comment) is
    /// what makes this possible, not anything specific to the top-level field order.
    #[test]
    fn states_inner_key_order_round_trips_byte_identical() {
        for keys in [
            ["persistent_bit", "update_bit"],
            ["update_bit", "persistent_bit"],
        ] {
            let mut states = nbtx::Compound::new();
            for k in keys {
                states.insert(k.into(), nbtx::Value::Byte(1));
            }
            let entry =
                entry_with_order(FieldOrder::StatesThenVersion, nbtx::Value::Int(1), states);
            let bytes = subchunk_with_palette(&entry);

            let chunk =
                SubChunk::from_disk::<Greedy, _>(&mut Cursor::new(bytes.as_slice())).unwrap();
            let mut out = Cursor::new(Vec::new());
            chunk.to_disk::<Greedy, _>(&mut out).unwrap();

            assert_eq!(
                out.into_inner(),
                bytes,
                "{keys:?} states order did not round-trip byte-identically"
            );
        }
    }

    /// An entry never read from disk -- built directly through [`Layer::set`] -- writes
    /// in the dominant real-world order, not `BlockDef`'s own struct order.
    #[test]
    fn freshly_set_entry_gets_the_default_field_order() {
        let mut layer = Layer {
            array: BitArray::Empty,
            palette: Vec::new(),
            hashes: HashMap::with_hasher(BuildNoHashHasher::default()),
            entry_order: Vec::new(),
        };
        layer.set(
            (0u8, 0u8, 0u8),
            BlockDef {
                name: "minecraft:test".into(),
                version: Some(1),
                states: indexmap::IndexMap::new(),
            },
        );
        assert_eq!(layer.entry_order, vec![FieldOrder::StatesThenVersion]);
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
        assert_eq!(
            layer.palette()[0].as_block().unwrap().version,
            Some(17_959_425)
        );
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
    /// order, must be equal and hash equal -- the `Hash` impl must not depend on
    /// `states`' insertion order, even though that order is preserved for re-encoding.
    #[test]
    fn identical_states_hash_equal_regardless_of_insertion_order() {
        use std::collections::hash_map::DefaultHasher;

        let keys = ["a", "b", "c", "d", "e", "f", "g"];

        let mut forward = indexmap::IndexMap::new();
        for (i, k) in keys.iter().enumerate() {
            forward.insert(k.to_string(), nbtx::Value::Int(i as i32));
        }
        let mut backward = indexmap::IndexMap::new();
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

    /// A pre-flattening `{name, val}` compound, in the order every real record uses (see
    /// [`LegacyBlockDef`]'s doc comment).
    fn legacy_entry(name: &str, val: i16) -> nbtx::Value {
        nbtx::Value::Compound(nbtx::Compound::from_iter([
            ("name".into(), nbtx::Value::String(name.into())),
            ("val".into(), nbtx::Value::Short(val)),
        ]))
    }

    /// The shape confirmed against real `v1_12` records (`tests/bit_layers.rs`): a legacy
    /// entry decodes into `PaletteEntry::Legacy` with its name and opaque `val` intact, no
    /// `states`/`version` synthesized for it.
    #[test]
    fn legacy_palette_entry_decodes_name_and_val() {
        let bytes = subchunk_with_palette(&legacy_entry("minecraft:wool", 4));
        let chunk = SubChunk::from_disk::<Greedy, _>(&mut Cursor::new(bytes.as_slice())).unwrap();
        let entry = &chunk.get_layer(0).unwrap().palette()[0];

        assert_eq!(entry.name(), "minecraft:wool");
        assert_eq!(entry.legacy_val(), Some(4));
        assert!(entry.as_block().is_none());
    }

    /// A legacy entry decodes and re-encodes to the exact original bytes -- no `states` or
    /// `version` key gets added, and the field order stays `name, val`.
    #[test]
    fn legacy_palette_entry_round_trips_byte_identical() {
        let bytes = subchunk_with_palette(&legacy_entry("minecraft:wood", 3));
        let chunk = SubChunk::from_disk::<Greedy, _>(&mut Cursor::new(bytes.as_slice())).unwrap();

        let mut out = Cursor::new(Vec::new());
        chunk.to_disk::<Greedy, _>(&mut out).unwrap();
        assert_eq!(out.into_inner(), bytes);
    }

    /// A legacy entry through the zero-bits-per-index layout (header `0x00`) -- the same
    /// compact layer layout `zero_bit_header_round_trips_byte_identical` confirms for a
    /// modern entry, but with a legacy palette entry following the header.
    #[test]
    fn legacy_palette_entry_round_trips_through_a_zero_bit_layer() {
        let bytes = subchunk_with_zero_bit_layer(&legacy_entry("minecraft:sapling", 0));
        let chunk = SubChunk::from_disk::<Greedy, _>(&mut Cursor::new(bytes.as_slice())).unwrap();
        let entry = &chunk.get_layer(0).unwrap().palette()[0];
        assert_eq!(entry.legacy_val(), Some(0));

        let mut out = Cursor::new(Vec::new());
        chunk.to_disk::<Greedy, _>(&mut out).unwrap();
        assert_eq!(out.into_inner(), bytes);
    }

    /// `val`'s type is checked like every other field -- a narrower or wider tag than the
    /// `Short` every real record uses is rejected outright rather than coerced.
    #[test]
    fn legacy_palette_entry_with_wrong_val_type_still_fails() {
        let entry = nbtx::Value::Compound(nbtx::Compound::from_iter([
            ("name".into(), nbtx::Value::String("minecraft:wool".into())),
            ("val".into(), nbtx::Value::Int(4)),
        ]));
        let bytes = subchunk_with_palette(&entry);
        assert!(SubChunk::from_disk::<Greedy, _>(&mut Cursor::new(bytes.as_slice())).is_err());
    }

    /// Builds a one-layer legacy subchunk whose palette holds two entries (indices packed
    /// at 1 bit each) and whose 4096 indices are all zero, i.e. every block resolves to
    /// `entries[0]`. Used to build a palette mixing a modern and a legacy entry -- the
    /// crate's decode/encode path must handle that even though no fixture in the corpus
    /// happens to contain one (checked: `v1_12`'s 6,246 legacy entries never sit beside a
    /// modern one in the same record).
    fn subchunk_with_two_entry_palette(entries: [&nbtx::Value; 2]) -> Vec<u8> {
        let mut out = vec![SubChunkVersion::Legacy as u8];
        out.push(1 << 1); // 1 bit per index.
        out.extend(std::iter::repeat_n(0u8, 128 * 4));
        out.extend_from_slice(&2u32.to_le_bytes());
        for entry in entries {
            out.extend(nbtx::to_le_bytes(entry).unwrap());
        }
        out
    }

    /// A palette holding one modern and one legacy entry, in the same layer, decodes both
    /// correctly and re-encodes byte-identically -- the two forms are first-class members
    /// of one palette, not mutually exclusive per layer.
    #[test]
    fn mixed_modern_and_legacy_palette_round_trips_byte_identical() {
        let modern = entry_with_version(nbtx::Value::Int(17_959_425));
        let legacy = legacy_entry("minecraft:wool", 4);
        let bytes = subchunk_with_two_entry_palette([&modern, &legacy]);

        let chunk = SubChunk::from_disk::<Greedy, _>(&mut Cursor::new(bytes.as_slice())).unwrap();
        let palette = chunk.get_layer(0).unwrap().palette();
        assert_eq!(palette.len(), 2);
        assert!(palette[0].as_block().is_some());
        assert_eq!(palette[1].name(), "minecraft:wool");
        assert_eq!(palette[1].legacy_val(), Some(4));

        let mut out = Cursor::new(Vec::new());
        chunk.to_disk::<Greedy, _>(&mut out).unwrap();
        assert_eq!(out.into_inner(), bytes);
    }

    /// Two legacy entries with the same name but different `val` are distinct palette
    /// entries -- `val` is part of a legacy entry's identity (pre-flattening,
    /// `minecraft:wool` at different `val`s is a different colour), so the palette must not
    /// collapse them the way it would if only `name` were hashed.
    #[test]
    fn legacy_entries_with_different_val_are_distinct_palette_entries() {
        let bytes = subchunk_with_two_entry_palette([
            &legacy_entry("minecraft:wool", 0),
            &legacy_entry("minecraft:wool", 4),
        ]);
        let chunk = SubChunk::from_disk::<Greedy, _>(&mut Cursor::new(bytes.as_slice())).unwrap();
        let palette = chunk.get_layer(0).unwrap().palette();

        assert_eq!(palette.len(), 2);
        assert_eq!(palette[0].legacy_val(), Some(0));
        assert_eq!(palette[1].legacy_val(), Some(4));
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
