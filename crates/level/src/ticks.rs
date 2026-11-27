//! `0x33` [`crate::key::KeyVariant::PendingTicks`] and `0x3a`
//! [`crate::key::KeyVariant::RandomTicks`]: a chunk's scheduled block
//! updates, in whichever of the two disjoint tick queues the tag names.
//!
//! The schema below is established from real records only: one uncompressed
//! little-endian NBT compound, read to the end of the value with nothing
//! trailing it, of the shape
//!
//! ```text
//! Compound {
//!     currentTick: Int,
//!     tickList: List<Compound {
//!         blockState: Compound { .. },
//!         time: Long,
//!         x: Int,
//!         y: Int,
//!         z: Int,
//!     }>,
//! }
//! ```
//!
//! Confirmed against all 323 real records in the corpus (141 `0x33`, 182
//! `0x3a`, spanning every fixture from `v1_12` through the 1.26 seed world):
//! every one parses to exactly this shape, with the field order shown above
//! -- `currentTick` before `tickList` at the top level, `blockState` before
//! `time` before `x`/`y`/`z` in every entry -- and zero exceptions to either
//! order. `currentTick` is an `Int` in all 323 records, never the `Long`
//! that the level-wide `currentTick` field in `level.dat` uses; entries
//! agree on `time: Long` and `x`/`y`/`z: Int` with the same zero-exception
//! consistency. No real record has an empty `tickList`, and `blockState`
//! is a `Compound` in every entry but is not further interpreted: it holds
//! either a legacy `{name, val}` pair or a modern `{name, states, version}`
//! triple depending on the block, and per the wider crate's vanilla-data
//! independence a block state is content, not format, so it passes through
//! as opaque NBT for the upgrade path to interpret.
//!
//! [`TickList`]/[`TickEntry`] wrap the decoded [`nbtx::Value`] rather than
//! reconstructing a fixed struct, the same choice `MetadataDictionaryEntry`
//! makes and for the same reason: `Value::Compound` is order-preserving, so
//! re-encoding the wrapped value reproduces the source bytes exactly,
//! including any field this crate's accessors do not name -- decode -> encode
//! is byte-identical by construction rather than by reassembling the fields
//! this module happens to know about, and a future field neither key has
//! shown yet still survives a read-modify-write untouched (architecture
//! decision 7).

use crate::error::{Error, Result};

/// Looks up a top-level field of a `Compound` value by name. Shared by
/// [`TickEntry::field`] and [`TickList::field`], which differ only in the
/// lifetime the borrow comes back tied to (the source [`TickList`]'s own
/// borrow for one, the [`TickEntry`]'s already-borrowed `'a` for the
/// other) -- both fall out of this single signature.
fn field<'v>(value: &'v nbtx::Value, name: &str) -> Option<&'v nbtx::Value> {
    let nbtx::Value::Compound(compound) = value else {
        return None;
    };
    compound.get(name.as_bytes())
}

/// One entry in a [`TickList`]'s `tickList`: one scheduled block update.
///
/// Borrows from the [`TickList`] it came from rather than cloning -- see
/// the module docs for why this wraps the entry's whole NBT compound
/// instead of a fixed struct.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickEntry<'a> {
    value: &'a nbtx::Compound,
}

impl<'a> TickEntry<'a> {
    /// The entry's full NBT compound, verbatim.
    pub fn value(&self) -> &'a nbtx::Compound {
        self.value
    }

    fn field(&self, name: &str) -> Option<&'a nbtx::Value> {
        self.value.get(bstr::BStr::new(name))
    }

    /// The `x` field. `Int` in every real record found; `None` if absent or
    /// a different tag.
    pub fn x(&self) -> Option<i32> {
        match self.field("x")? {
            nbtx::Value::Int(v) => Some(*v),
            _ => None,
        }
    }

    /// The `y` field. `Int` in every real record found; `None` if absent or
    /// a different tag.
    pub fn y(&self) -> Option<i32> {
        match self.field("y")? {
            nbtx::Value::Int(v) => Some(*v),
            _ => None,
        }
    }

    /// The `z` field. `Int` in every real record found; `None` if absent or
    /// a different tag.
    pub fn z(&self) -> Option<i32> {
        match self.field("z")? {
            nbtx::Value::Int(v) => Some(*v),
            _ => None,
        }
    }

    /// The `time` field: the tick this entry is scheduled to run at. `Long`
    /// in every real record found; `None` if absent or a different tag.
    pub fn time(&self) -> Option<i64> {
        match self.field("time")? {
            nbtx::Value::Long(v) => Some(*v),
            _ => None,
        }
    }

    /// The `blockState` field, verbatim and uninterpreted -- see the module
    /// docs for why this crate does not resolve it into a typed block
    /// state here.
    pub fn block_state(&self) -> Option<&'a nbtx::Value> {
        self.field("blockState")
    }
}

/// Iterates over a [`TickList`]'s entries, in on-disk order. See
/// [`TickList::entries`].
pub struct TickEntries<'a> {
    inner: std::slice::Iter<'a, nbtx::Compound>,
}

impl<'a> Iterator for TickEntries<'a> {
    type Item = TickEntry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|value| TickEntry { value })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ExactSizeIterator for TickEntries<'_> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl DoubleEndedIterator for TickEntries<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|value| TickEntry { value })
    }
}

/// A decoded `0x33` PendingTicks or `0x3a` RandomTicks value -- the two
/// tags share this exact schema (see the module docs), so one type serves
/// both; which tag a given value came from is tracked by the [`crate::key::Key`]
/// it was read under, not by this type.
#[derive(Debug, Clone, PartialEq)]
pub struct TickList {
    value: nbtx::Value,
}

impl TickList {
    /// Wraps an already-decoded NBT value. Every real record found is a
    /// `Compound` with the shape the module docs describe, but nothing
    /// here requires it -- the typed accessors below simply return `None`
    /// for a `value` that isn't shaped that way.
    pub fn new(value: nbtx::Value) -> Self {
        Self { value }
    }

    /// The whole value, verbatim.
    pub fn value(&self) -> &nbtx::Value {
        &self.value
    }

    fn field(&self, name: &str) -> Option<&nbtx::Value> {
        field(&self.value, name)
    }

    /// The `currentTick` field: the tick this record was last written at.
    /// `Int` in every real record found (never the `Long` the level-wide
    /// field of the same name in `level.dat` uses); `None` if absent or a
    /// different tag.
    pub fn current_tick(&self) -> Option<i32> {
        match self.field("currentTick")? {
            nbtx::Value::Int(v) => Some(*v),
            _ => None,
        }
    }

    /// Iterates over the `tickList` entries, in on-disk order. Empty if the
    /// field is absent or not a list -- no real record found has either
    /// case, but this degrades rather than panics per architecture
    /// decision 11 (unmodelled, not malformed).
    pub fn entries(&self) -> TickEntries<'_> {
        match self.field("tickList") {
            // Every real `tickList` is a list of compounds; any other element type has no
            // entries to model.
            Some(nbtx::Value::List(nbtx::ValueList::Compound(list))) => {
                TickEntries { inner: list.iter() }
            }
            _ => TickEntries { inner: [].iter() },
        }
    }
}

/// Decodes a `0x33`/`0x3a` value: one uncompressed little-endian NBT
/// document occupying the whole buffer. Per architecture decision 11,
/// trailing bytes after the document are malformed rather than ignored --
/// no real record found has any, and a value with trailing bytes should not
/// have them silently discarded.
pub fn decode_tick_list(bytes: &[u8]) -> Result<TickList> {
    let mut cursor = std::io::Cursor::new(bytes);
    let value: nbtx::Value = nbtx::from_le_bytes(&mut cursor)?;
    if cursor.position() as usize != bytes.len() {
        return Err(Error::Invalid(
            "tick list value has trailing bytes past its NBT document",
        ));
    }
    Ok(TickList { value })
}

/// Encodes a `0x33`/`0x3a` value: the inverse of [`decode_tick_list`]. The
/// wrapped [`nbtx::Value`] is order-preserving (see the module docs), so
/// this reproduces a decoded real record byte-identically.
pub fn encode_tick_list(value: &TickList) -> Result<Vec<u8>> {
    Ok(nbtx::to_le_bytes(&value.value)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entry(x: i32, y: i32, z: i32, time: i64) -> nbtx::Value {
        let mut compound = nbtx::Compound::new();
        let mut block_state = nbtx::Compound::new();
        block_state.insert("name".into(), nbtx::Value::String("minecraft:fire".into()));
        block_state.insert("val".into(), nbtx::Value::Short(0));
        compound.insert("blockState".into(), nbtx::Value::Compound(block_state));
        compound.insert("time".into(), nbtx::Value::Long(time));
        compound.insert("x".into(), nbtx::Value::Int(x));
        compound.insert("y".into(), nbtx::Value::Int(y));
        compound.insert("z".into(), nbtx::Value::Int(z));
        nbtx::Value::Compound(compound)
    }

    fn sample_tick_list() -> nbtx::Value {
        let mut compound = nbtx::Compound::new();
        compound.insert("currentTick".into(), nbtx::Value::Int(30));
        compound.insert(
            "tickList".into(),
            nbtx::Value::List(
                nbtx::ValueList::try_from(vec![
                    sample_entry(15, 4, 44, 30),
                    sample_entry(14, 4, 43, 31),
                ])
                .unwrap(),
            ),
        );
        nbtx::Value::Compound(compound)
    }

    #[test]
    fn decodes_a_synthetic_well_formed_record() {
        let bytes = nbtx::to_le_bytes(&sample_tick_list()).unwrap();
        let list = decode_tick_list(&bytes).unwrap();
        assert_eq!(list.current_tick(), Some(30));

        let entries: Vec<_> = list.entries().collect();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].x(), Some(15));
        assert_eq!(entries[0].y(), Some(4));
        assert_eq!(entries[0].z(), Some(44));
        assert_eq!(entries[0].time(), Some(30));
        assert_eq!(
            entries[0].block_state(),
            Some(&{
                let mut c = nbtx::Compound::new();
                c.insert("name".into(), nbtx::Value::String("minecraft:fire".into()));
                c.insert("val".into(), nbtx::Value::Short(0));
                nbtx::Value::Compound(c)
            })
        );
        assert_eq!(entries[1].x(), Some(14));
    }

    #[test]
    fn round_trips_a_synthetic_well_formed_record() {
        let bytes = nbtx::to_le_bytes(&sample_tick_list()).unwrap();
        let list = decode_tick_list(&bytes).unwrap();
        let re_encoded = encode_tick_list(&list).unwrap();
        assert_eq!(re_encoded, bytes);
    }

    #[test]
    fn round_trips_an_empty_tick_list() {
        let mut compound = nbtx::Compound::new();
        compound.insert("currentTick".into(), nbtx::Value::Int(0));
        compound.insert("tickList".into(), nbtx::Value::List(nbtx::ValueList::End));
        let bytes = nbtx::to_le_bytes(&nbtx::Value::Compound(compound)).unwrap();

        let list = decode_tick_list(&bytes).unwrap();
        assert_eq!(list.current_tick(), Some(0));
        assert_eq!(list.entries().count(), 0);
        assert_eq!(encode_tick_list(&list).unwrap(), bytes);
    }

    #[test]
    fn unknown_field_survives_decode_then_encode() {
        // A field this crate has no accessor for must not be dropped on
        // re-encode (architecture decision 7).
        let mut compound = nbtx::Compound::new();
        compound.insert("currentTick".into(), nbtx::Value::Int(5));
        compound.insert("tickList".into(), nbtx::Value::List(nbtx::ValueList::End));
        compound.insert("someFutureField".into(), nbtx::Value::Long(99));
        let bytes = nbtx::to_le_bytes(&nbtx::Value::Compound(compound)).unwrap();

        let list = decode_tick_list(&bytes).unwrap();
        let re_encoded = encode_tick_list(&list).unwrap();
        assert_eq!(re_encoded, bytes);
    }

    #[test]
    fn accessors_return_none_on_missing_or_wrong_tag_fields() {
        let list = TickList::new(nbtx::Value::Compound(nbtx::Compound::new()));
        assert_eq!(list.current_tick(), None);
        assert_eq!(list.entries().count(), 0);

        let entry = TickEntry {
            value: &nbtx::Compound::new(),
        };
        assert_eq!(entry.x(), None);
        assert_eq!(entry.y(), None);
        assert_eq!(entry.z(), None);
        assert_eq!(entry.time(), None);
        assert_eq!(entry.block_state(), None);
    }

    #[test]
    fn decode_rejects_trailing_bytes() {
        let mut bytes = nbtx::to_le_bytes(&sample_tick_list()).unwrap();
        bytes.push(0xff);
        assert!(decode_tick_list(&bytes).is_err());
    }

    #[test]
    fn decode_rejects_non_nbt_bytes() {
        assert!(decode_tick_list(&[0xff, 0xff, 0xff]).is_err());
    }
}
