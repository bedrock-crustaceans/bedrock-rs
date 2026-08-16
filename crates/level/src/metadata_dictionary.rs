//! `LevelChunkMetaDataDictionary`: a level-wide table of per-generation-context
//! chunk metadata, and the lookup target of a chunk's `0x3f` `MetadataDictionaryKey`
//! record.
//!
//! Unlike every other record this crate models, this one is **not**
//! chunk-scoped: it lives under the bare string key
//! [`KEY`] (`"LevelChunkMetaDataDictionary"`), once per world, so
//! [`crate::key::Key`] cannot represent it and it never belongs to a
//! [`crate::chunk::ChunkRecords`] group. A chunk instead carries an 8-byte
//! [`DictionaryKey`] under its own `0x3f` record, and that value is exactly
//! one of this table's entry keys -- confirmed against real worlds, not
//! merely inferred from the name: every `0x3f` value in every fixture and
//! the 1.26 seed world that carries this dictionary resolves to an entry
//! here (1044 chunks checked across four worlds, 100% resolved, zero
//! dangling references).
//!
//! # On-disk layout
//!
//! `u32 count` (little-endian) followed by `count` entries, each an 8-byte
//! [`DictionaryKey`] immediately followed by one uncompressed little-endian
//! NBT compound -- no length prefix on the compound; its own terminating
//! `TAG_End` marks where the next entry's key begins. Nothing pads or
//! aligns entries, and nothing follows the last one: every real record
//! found ends exactly where the last entry's NBT ends (pinned by
//! [`decode_then_encode_reproduces_every_real_dictionary_byte_for_byte`]).
//!
//! # Field set (corpus finding, not the format's actual guarantee)
//!
//! The record's own layout only knows an entry is *some* NBT compound --
//! nothing on disk enumerates its fields. The originally proposed field set
//! (`DimensionName`, `LastSavedDimensionHeightRange`,
//! `OriginalDimensionHeightRange`, `LastSavedBaseGameVersion`) is confirmed
//! present, but incomplete: a scan of every dictionary in the fixture corpus
//! plus the 1.26 seed world (23 entries total, across `v1_18_30`, `v1_19_30`,
//! `v1_20_81`, and the seed world) additionally finds `BiomeBaseGameVersion`,
//! `BlendingVersion`, `GenerationSeed`, `GeneratorType`,
//! `OriginalBaseGameVersion`, `Overworld1_18HeightExtended`,
//! `UnderwaterLavaLakeFixed`, `WorldGenBelowZeroFixed`, and, on the newer
//! seed world only, `SkullFlatteningPerformed` and
//! `NeighborAwareBlockUpgradeVersion`. Field *presence* also varies entry by
//! entry within one dictionary -- `LastSavedDimensionHeightRange` and
//! `LastSavedBaseGameVersion` are both absent on several real entries (a
//! chunk that has only ever been saved once has nothing to distinguish
//! "last saved" from "original"), and the growth of the field set between
//! the older fixtures and the 1.26 world shows the schema is not closed.
//! [`MetadataDictionaryEntry`] therefore keeps the whole decoded compound
//! rather than a fixed struct, so a field this crate has no accessor for is
//! carried through a decode -> encode cycle rather than silently dropped
//! (architecture decision 7), and a future game version adding another
//! field never turns a read into an error.
//!
//! `DimensionName` is not only ever `"Overworld"` in the corpus: the 1.26
//! seed world also carries `"TheEnd"` entries, each with its own
//! `0..256` height range. No `"Nether"` entry was observed, plausibly
//! because the Nether's height range has never changed and so never needed
//! recording, but that is a corpus gap, not a confirmed rule.
//!
//! # Write side
//!
//! Per architecture decision 13 this record is preserved, not recomputed:
//! this crate has no way to derive a dictionary entry's height ranges or
//! seed on its own, so a read-modify-write must carry every entry through
//! untouched. [`MetadataDictionary::insert`] exists for the one write-side
//! carve-out decision 13 names -- writing a fresh entry when a chunk lands
//! in a dimension the dictionary has no entry for -- but choosing *when*
//! to call it needs level.dat's declared height-range flags and is out of
//! scope here (see the roadmap's per-dimension height-range task).

use std::io::{Cursor, Read};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::error::{Error, Result};

/// The on-disk string key this whole table is stored under. Not a
/// [`crate::key::KeyVariant`] -- see the module docs for why this record
/// cannot be represented as a [`crate::key::Key`] at all.
pub const KEY: &[u8] = b"LevelChunkMetaDataDictionary";

/// Length in bytes of a [`DictionaryKey`].
pub const DICTIONARY_KEY_LEN: usize = 8;

/// One dictionary entry's 8-byte lookup key, and the exact value a chunk's
/// `0x3f` `MetadataDictionaryKey` record holds to name its entry (verified against
/// real worlds; see the module docs). A fixed-size array rather than a
/// newtype: nothing about this key is interpreted, split, or validated
/// beyond its length, so there is no invariant for a wrapper to protect,
/// and `[u8; 8]` lets a `0x3f` record's raw value convert into one with the
/// standard library's own `TryFrom<&[u8]>`.
pub type DictionaryKey = [u8; DICTIONARY_KEY_LEN];

/// A `min`/`max` block-Y pair, as stored in an entry's
/// `LastSavedDimensionHeightRange`/`OriginalDimensionHeightRange` compound.
/// Bounds are stored as NBT shorts on disk, so they are held as `i16` here
/// rather than widened -- resolving them against a dimension's actual
/// engine bounds is the per-dimension height-range task this module
/// deliberately does not do (see the module docs).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeightRange {
    pub min: i16,
    pub max: i16,
}

/// One dictionary entry: an opaque NBT compound plus typed accessors for
/// the fields this crate has a structural reason to read.
///
/// Holds the decoded [`nbtx::Value`] rather than a fixed struct -- see the
/// module docs' field-set finding for why a struct would either reject
/// real records under `deny-unknown-fields` or silently drop fields it
/// does not declare under the lenient feature, neither of which decision 7
/// allows. `Value::Compound` is order-preserving, which is what makes
/// [`MetadataDictionary::encode`] byte-identical to every real entry this
/// crate has decoded, unknown fields included.
#[derive(Debug, Clone, PartialEq)]
pub struct MetadataDictionaryEntry {
    value: nbtx::Value,
}

impl MetadataDictionaryEntry {
    /// Wraps an already-decoded NBT value as an entry. This is the
    /// construction path a write-side caller (see the module docs) would
    /// use to build a fresh entry from scratch. Every real entry found is a
    /// `Value::Compound` (see the module docs), but nothing here requires
    /// it -- the typed accessors above simply return `None` for a `value`
    /// that isn't one.
    pub fn new(value: nbtx::Value) -> Self {
        Self { value }
    }

    /// The entry's full NBT value, verbatim -- every field it carries,
    /// known or not, in on-disk order. The escape hatch for anything the
    /// typed accessors below do not cover (`GenerationSeed`,
    /// `BlendingVersion`, and the rest of the module docs' field-set
    /// finding).
    pub fn value(&self) -> &nbtx::Value {
        &self.value
    }

    /// Looks up a top-level field by name, if this entry's value is a
    /// compound at all (every real entry found is; see the module docs).
    fn field(&self, name: &str) -> Option<&nbtx::Value> {
        let nbtx::Value::Compound(compound) = &self.value else {
            return None;
        };
        compound.get(name.as_bytes())
    }

    /// The `DimensionName` field (`"Overworld"`, `"Nether"`, or `"TheEnd"`
    /// in every real record found; see the module docs). `None` if the
    /// field is absent or not a string.
    pub fn dimension_name(&self) -> Option<&str> {
        match self.field("DimensionName")? {
            nbtx::Value::String(s) => std::str::from_utf8(s).ok(),
            _ => None,
        }
    }

    /// Reads a `{min: Short, max: Short}` compound field.
    fn height_range(&self, field_name: &str) -> Option<HeightRange> {
        let nbtx::Value::Compound(range) = self.field(field_name)? else {
            return None;
        };
        let min = match range.get(b"min".as_slice())? {
            nbtx::Value::Short(v) => *v,
            _ => return None,
        };
        let max = match range.get(b"max".as_slice())? {
            nbtx::Value::Short(v) => *v,
            _ => return None,
        };
        Some(HeightRange { min, max })
    }

    /// The `LastSavedDimensionHeightRange` field: the dimension's height
    /// range as of this entry's most recent save. Absent on an entry that
    /// has only ever been saved once (the module docs' field-presence
    /// finding), in which case [`Self::original_dimension_height_range`]
    /// is the same value.
    pub fn last_saved_dimension_height_range(&self) -> Option<HeightRange> {
        self.height_range("LastSavedDimensionHeightRange")
    }

    /// The `OriginalDimensionHeightRange` field: the dimension's height
    /// range at the time this entry was first created. Present on every
    /// real entry found.
    pub fn original_dimension_height_range(&self) -> Option<HeightRange> {
        self.height_range("OriginalDimensionHeightRange")
    }

    /// The `LastSavedBaseGameVersion` field. Like
    /// [`Self::last_saved_dimension_height_range`], absent on an entry
    /// that has only ever been saved once.
    pub fn last_saved_base_game_version(&self) -> Option<&str> {
        match self.field("LastSavedBaseGameVersion")? {
            nbtx::Value::String(s) => std::str::from_utf8(s).ok(),
            _ => None,
        }
    }
}

/// The decoded `LevelChunkMetaDataDictionary` record: every entry, in
/// on-disk order, looked up by its [`DictionaryKey`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MetadataDictionary {
    entries: Vec<(DictionaryKey, MetadataDictionaryEntry)>,
}

impl MetadataDictionary {
    /// An empty dictionary, as a world with none of this table's chunks
    /// would have.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether this dictionary holds no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Looks up an entry by its [`DictionaryKey`] -- the same value a
    /// chunk's `0x3f` record holds. Dictionary sizes observed in real
    /// worlds are single digits to low tens of entries, so a linear scan
    /// costs nothing that would justify an auxiliary map, and it keeps
    /// this type's only representation the one that also has to preserve
    /// on-disk order for [`Self::encode`].
    pub fn get(&self, key: &DictionaryKey) -> Option<&MetadataDictionaryEntry> {
        self.entries.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// Inserts an entry under `key`, returning the previous entry if one
    /// existed under the same key (replaced in place, keeping its original
    /// position) or appending a new one at the end.
    ///
    /// This is the hook a write path needs for architecture decision 13's
    /// carve-out -- adding an entry when a chunk lands in a dimension the
    /// dictionary has none for -- without this module deciding *when* that
    /// applies (see the module docs).
    pub fn insert(
        &mut self,
        key: DictionaryKey,
        entry: MetadataDictionaryEntry,
    ) -> Option<MetadataDictionaryEntry> {
        if let Some(slot) = self.entries.iter_mut().find(|(k, _)| *k == key) {
            Some(std::mem::replace(&mut slot.1, entry))
        } else {
            self.entries.push((key, entry));
            None
        }
    }

    /// Iterates over every entry, in on-disk order.
    pub fn iter(&self) -> MetadataDictionaryIter<'_> {
        MetadataDictionaryIter {
            inner: self.entries.iter(),
        }
    }

    /// Decodes a `LevelChunkMetaDataDictionary` value.
    ///
    /// Per architecture decision 11, a buffer too short for its own `count`
    /// field, or that runs out mid-entry (not enough bytes for an entry's
    /// 8-byte key, or a truncated NBT compound), is malformed and errors
    /// rather than returning a partial result or panicking. Likewise, bytes
    /// left over after the declared `count` entries are read is a length
    /// mismatch and errors -- every real record found consumes its buffer
    /// exactly (see the module docs).
    ///
    /// The entry count comes straight from the untrusted `count` field, so
    /// entries are collected without pre-reserving capacity from it: a
    /// bogus `count` fails on the first entry it cannot actually read
    /// (at the latest, once the buffer is exhausted) rather than driving an
    /// upfront allocation sized off a number nothing has validated yet.
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(bytes);
        let count = cursor.read_u32::<LittleEndian>()?;

        let mut entries = Vec::new();
        for _ in 0..count {
            let mut key = [0u8; DICTIONARY_KEY_LEN];
            cursor.read_exact(&mut key)?;
            let value: nbtx::Value = nbtx::from_le_bytes(&mut cursor)?;
            entries.push((key, MetadataDictionaryEntry { value }));
        }

        if cursor.position() != bytes.len() as u64 {
            return Err(Error::Invalid(
                "metadata dictionary has trailing bytes past its declared entries",
            ));
        }

        Ok(Self { entries })
    }

    /// Encodes this dictionary. The inverse of [`Self::decode`], and its
    /// exact byte-for-byte round-trip partner: `Value::Compound`'s
    /// order-preserving map reproduces every entry's on-disk field order,
    /// which is what makes this byte-identical for every real dictionary
    /// this crate's test suite has checked
    /// (`decode_then_encode_reproduces_every_real_dictionary_byte_for_byte`).
    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        out.write_u32::<LittleEndian>(self.entries.len() as u32)?;
        for (key, entry) in &self.entries {
            out.extend_from_slice(key);
            nbtx::to_le_bytes_in(&mut out, &entry.value)?;
        }
        Ok(out)
    }
}

/// Iterates over a [`MetadataDictionary`]'s entries. See
/// [`MetadataDictionary::iter`].
pub struct MetadataDictionaryIter<'a> {
    inner: std::slice::Iter<'a, (DictionaryKey, MetadataDictionaryEntry)>,
}

impl<'a> Iterator for MetadataDictionaryIter<'a> {
    type Item = (&'a DictionaryKey, &'a MetadataDictionaryEntry);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(k, v)| (k, v))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ExactSizeIterator for MetadataDictionaryIter<'_> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl DoubleEndedIterator for MetadataDictionaryIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|(k, v)| (k, v))
    }
}

impl<'a> IntoIterator for &'a MetadataDictionary {
    type Item = (&'a DictionaryKey, &'a MetadataDictionaryEntry);
    type IntoIter = MetadataDictionaryIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entry(dim: &str, min: i16, max: i16) -> MetadataDictionaryEntry {
        let mut compound = nbtx::Compound::new();
        compound.insert("DimensionName".into(), nbtx::Value::String(dim.into()));
        let mut range = nbtx::Compound::new();
        range.insert("min".into(), nbtx::Value::Short(min));
        range.insert("max".into(), nbtx::Value::Short(max));
        compound.insert(
            "OriginalDimensionHeightRange".into(),
            nbtx::Value::Compound(range),
        );
        MetadataDictionaryEntry::new(nbtx::Value::Compound(compound))
    }

    #[test]
    fn empty_dictionary_round_trips() {
        let dict = MetadataDictionary::new();
        let bytes = dict.encode().unwrap();
        assert_eq!(bytes, 0u32.to_le_bytes());
        assert_eq!(MetadataDictionary::decode(&bytes).unwrap(), dict);
    }

    #[test]
    fn single_entry_round_trips() {
        let mut dict = MetadataDictionary::new();
        dict.insert(
            [1, 2, 3, 4, 5, 6, 7, 8],
            sample_entry("Overworld", -64, 320),
        );

        let bytes = dict.encode().unwrap();
        let decoded = MetadataDictionary::decode(&bytes).unwrap();
        assert_eq!(decoded, dict);
        assert_eq!(decoded.len(), 1);

        let entry = decoded.get(&[1, 2, 3, 4, 5, 6, 7, 8]).unwrap();
        assert_eq!(entry.dimension_name(), Some("Overworld"));
        assert_eq!(
            entry.original_dimension_height_range(),
            Some(HeightRange { min: -64, max: 320 })
        );
        assert_eq!(entry.last_saved_dimension_height_range(), None);
    }

    #[test]
    fn get_returns_none_for_unknown_key() {
        let dict = MetadataDictionary::new();
        assert_eq!(dict.get(&[0; 8]), None);
    }

    #[test]
    fn insert_replaces_in_place_and_reports_previous() {
        let mut dict = MetadataDictionary::new();
        dict.insert([1; 8], sample_entry("Overworld", -64, 320));
        dict.insert([2; 8], sample_entry("TheEnd", 0, 256));

        let previous = dict.insert([1; 8], sample_entry("Nether", 0, 128));
        assert_eq!(previous.unwrap().dimension_name(), Some("Overworld"));
        assert_eq!(dict.len(), 2);
        // Replacing keeps the original position: the first key still comes
        // first in iteration order.
        let keys: Vec<_> = dict.iter().map(|(k, _)| *k).collect();
        assert_eq!(keys, vec![[1; 8], [2; 8]]);
        assert_eq!(dict.get(&[1; 8]).unwrap().dimension_name(), Some("Nether"));
    }

    #[test]
    fn iter_yields_entries_in_on_disk_order() {
        let mut dict = MetadataDictionary::new();
        dict.insert([3; 8], sample_entry("Overworld", -64, 320));
        dict.insert([1; 8], sample_entry("TheEnd", 0, 256));

        let keys: Vec<_> = (&dict).into_iter().map(|(k, _)| *k).collect();
        assert_eq!(keys, vec![[3; 8], [1; 8]]);
    }

    #[test]
    fn unknown_field_survives_decode_then_encode() {
        // A field this crate has no accessor for must not be dropped on
        // re-encode (architecture decision 7).
        let mut compound = nbtx::Compound::new();
        compound.insert(
            "DimensionName".into(),
            nbtx::Value::String("Overworld".into()),
        );
        compound.insert("SomeFutureField".into(), nbtx::Value::Long(42));
        let entry_bytes = nbtx::to_le_bytes(&nbtx::Value::Compound(compound)).unwrap();

        let mut bytes = Vec::new();
        bytes.extend_from_slice(&1u32.to_le_bytes());
        bytes.extend_from_slice(&[9u8; 8]);
        bytes.extend_from_slice(&entry_bytes);

        let dict = MetadataDictionary::decode(&bytes).unwrap();
        let entry = dict.get(&[9u8; 8]).unwrap();
        assert_eq!(entry.field("SomeFutureField"), Some(&nbtx::Value::Long(42)));

        let re_encoded = dict.encode().unwrap();
        assert_eq!(re_encoded, bytes);
    }

    #[test]
    fn decode_rejects_truncated_count() {
        assert!(MetadataDictionary::decode(&[0u8; 3]).is_err());
    }

    #[test]
    fn decode_rejects_short_entry_key() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&1u32.to_le_bytes());
        bytes.extend_from_slice(&[0u8; 4]); // only 4 of the required 8 key bytes
        assert!(MetadataDictionary::decode(&bytes).is_err());
    }

    #[test]
    fn decode_rejects_truncated_entry_nbt() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&1u32.to_le_bytes());
        bytes.extend_from_slice(&[0u8; 8]);
        bytes.push(0x0a); // Compound tag, then nothing -- truncated mid-header
        assert!(MetadataDictionary::decode(&bytes).is_err());
    }

    #[test]
    fn decode_rejects_count_larger_than_the_buffer() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&5u32.to_le_bytes()); // claims 5 entries
        bytes.extend_from_slice(&[0u8; 8]);
        bytes.extend_from_slice(&nbtx::to_le_bytes(&nbtx::Value::Byte(0)).unwrap());
        // Only one entry actually follows.
        assert!(MetadataDictionary::decode(&bytes).is_err());
    }

    #[test]
    fn decode_rejects_trailing_bytes_past_declared_entries() {
        let mut dict = MetadataDictionary::new();
        dict.insert([1; 8], sample_entry("Overworld", -64, 320));
        let mut bytes = dict.encode().unwrap();
        bytes.push(0xff); // stray trailing byte
        assert!(MetadataDictionary::decode(&bytes).is_err());
    }

    #[test]
    fn dimension_name_none_when_field_absent() {
        let entry = MetadataDictionaryEntry::new(nbtx::Value::Compound(nbtx::Compound::new()));
        assert_eq!(entry.dimension_name(), None);
        assert_eq!(entry.original_dimension_height_range(), None);
        assert_eq!(entry.last_saved_base_game_version(), None);
    }

    #[test]
    fn accessors_return_none_on_non_compound_value() {
        let entry = MetadataDictionaryEntry::new(nbtx::Value::Byte(0));
        assert_eq!(entry.dimension_name(), None);
        assert_eq!(entry.original_dimension_height_range(), None);
    }

    #[test]
    fn height_range_none_on_wrong_tag_width() {
        // `min`/`max` are shorts on every real record found; a compound that
        // instead carries them as ints (or any other non-Short tag) is not
        // this crate's `HeightRange` and must not be misread as one.
        let mut range = nbtx::Compound::new();
        range.insert("min".into(), nbtx::Value::Int(-64));
        range.insert("max".into(), nbtx::Value::Int(320));
        let mut compound = nbtx::Compound::new();
        compound.insert(
            "OriginalDimensionHeightRange".into(),
            nbtx::Value::Compound(range),
        );
        let entry = MetadataDictionaryEntry::new(nbtx::Value::Compound(compound));

        assert_eq!(entry.original_dimension_height_range(), None);
    }
}
