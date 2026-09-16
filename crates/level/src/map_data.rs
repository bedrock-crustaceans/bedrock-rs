//! `map_<id>`: one saved map's data -- the item-frame/hand map render a
//! player has captured.
//!
//! The key is the ASCII prefix `map_` followed by the map id in decimal
//! (not hex), and the value is one uncompressed little-endian NBT compound.
//! Neither half of the key/value pair is chunk-scoped: a map's id is
//! assigned when the map item is created and has no relation to a chunk
//! position, so this is a flat, database-wide key family like
//! [`crate::metadata_dictionary`], not a [`crate::key::Key`].
//!
//! # Field order varies
//!
//! The corpus (65 real records across `v1_12`, `v1_18_30`, and `v1_19_30`
//! -- `v1_16` through `v1_20_81` otherwise carry none) shows the same
//! phenomenon Phase 1 found in a palette entry's field order: the field
//! *set* is uniform (all 65 records carry exactly the same thirteen
//! fields), but the field *order* is not, and not even consistently per
//! world -- `v1_18_30` and `v1_19_30` each mix two different orderings
//! between records of the same world (10 of 25 records in each write
//! `colors, decorations, dimension, fullyExplored, height, mapId,
//! mapLocked, parentMapId, scale, unlimitedTracking, width, xCenter,
//! zCenter`; the other 15 write `mapId, parentMapId, decorations,
//! fullyExplored, scale, dimension, height, width, unlimitedTracking,
//! xCenter, zCenter, mapLocked, colors`). No fixed struct order can
//! reproduce both, so [`MapRecord`] holds the decoded
//! [`nbtx::Value`] and replays its on-disk order on encode, the same
//! approach [`crate::metadata_dictionary`] uses for the same reason.
//!
//! # Field set
//!
//! Every real record carries: `mapId`/`parentMapId` (`Long`), `scale`
//! (`Byte`), `dimension`, `width`/`height` (`Short`), `xCenter`/`zCenter`
//! (`Int`), `unlimitedTracking`/`mapLocked`/`fullyExplored` (`Byte`
//! booleans), `decorations` (a `List` of compounds, each
//! `{data: {rot, type, x, y}, key: {blockX, blockY, blockZ, type}}` --
//! not modeled field by field here, since nothing in this crate has a
//! structural reason to interpret a map marker), and `colors`, a
//! `ByteArray` of `width * height * 4` RGBA bytes (65536 for a full-size
//! 128x128 map, confirmed on every real record found). `fullyExplored` is
//! not part of the originally proposed field list but is present on every
//! real record.
//!
//! `dimension` is a `Byte` on every real record found (all three worlds
//! carrying this key, `v1_12` through `v1_19_30`); no `Int`-width instance
//! exists in the corpus. [`MapRecord::dimension`] still reads either width
//! -- accepting the wider encoding costs nothing since the value is held
//! as [`nbtx::Value`] regardless, and the field is documented as
//! version-dependent by both `id` naming (`mapId` truncated to `i32` would
//! collide across worlds; the field is a full `Long`) and by the general
//! pattern of small on-disk fields widening across format revisions
//! elsewhere in this format -- but this widening itself is unverified
//! against a real record.
//!
//! # An empty list loses its declared element type through `nbtx::Value`
//!
//! Most real records carry an empty `decorations` list (only 4 of the 65
//! do not), and every one of them writes its element type as `Compound`
//! even though there are no elements to derive that type from. Re-deriving
//! bytes generically from a decoded [`nbtx::Value`] cannot reproduce this:
//! [`nbtx::Value::List`] has no field to hold a declared element type
//! independent of its elements, so an empty list re-encodes with element
//! type `TAG_End` -- correct by the same convention vanilla Minecraft uses
//! for an empty list, just not what this real field does. [`encode`]
//! works around this by returning a decoded record's original bytes
//! verbatim rather than re-deriving them; see its doc comment.

use std::io::Cursor;

use bedrock_shared::world::dimension::Dimension;

use crate::db::Database;
use crate::error::{Error, Result};

/// The ASCII prefix on a `map_<id>` key.
pub const KEY_PREFIX: &[u8] = b"map_";

/// Builds the on-disk key for map `id`.
pub fn key(id: i64) -> Vec<u8> {
    format!("map_{id}").into_bytes()
}

/// Extracts the map id from a `map_<id>` key, if `key_bytes` is shaped like
/// one: the [`KEY_PREFIX`] followed by a valid decimal `i64` (a leading
/// `-` accepted, matching the negative ids every real record's key uses).
/// Returns `None` for anything else, including a prefix match whose suffix
/// is not a clean decimal (leading zeros, whitespace, or overflow), so a
/// caller cannot mistake a malformed suffix for id `0`.
pub fn id_from_key(key_bytes: &[u8]) -> Option<i64> {
    let suffix = key_bytes.strip_prefix(KEY_PREFIX)?;
    let suffix = std::str::from_utf8(suffix).ok()?;
    // `i64::from_str` itself accepts a leading zero ("00" parses as 0), so
    // the "no leading zeros" half of the doc comment above needs its own
    // check: reject a '0' immediately followed by another digit, on either
    // side of an optional leading '-'.
    let digits = suffix.strip_prefix('-').unwrap_or(suffix);
    if digits.len() > 1 && digits.starts_with('0') {
        return None;
    }
    suffix.parse().ok()
}

/// An NBT value paired with the exact bytes it was decoded from, if any.
///
/// Shared by [`MapRecord`] and [`crate::raw_keys::RawCompound`], which are
/// both, underneath their own typed accessors (or lack thereof), a decoded
/// [`nbtx::Value`] that needs to re-emit its original on-disk bytes when
/// unmodified rather than re-deriving them: [`nbtx::Value::List`] cannot
/// always reproduce a real record's on-disk element-type byte for an empty
/// list through a generic re-derivation (a real `map_<id>` record's
/// `decorations` field, empty on most real records, is written with element
/// type `Compound` even though it has no elements to derive that type
/// from -- `Value::List` has nowhere to store a declared element type
/// independent of its, here absent, elements, so `nbtx` falls back to
/// `TAG_End` for an empty list on re-derivation, correct by the same
/// convention vanilla Minecraft itself uses for an empty list, but not what
/// this one real field does). Re-emitting the original bytes sidesteps the
/// question entirely for a record that was not modified; a record built
/// fresh (no original bytes to fall back to) or mutated after decode always
/// re-derives from `value` and is subject to that same limitation.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RetainedNbt {
    value: nbtx::Value,
    original: Option<Vec<u8>>,
}

impl RetainedNbt {
    /// Wraps an already-decoded NBT value with no original on-disk bytes to
    /// fall back to -- see the type docs for the consequence of that on a
    /// value with an empty list field.
    pub(crate) fn new(value: nbtx::Value) -> Self {
        Self {
            value,
            original: None,
        }
    }

    /// Wraps a decoded NBT value together with the exact bytes it was
    /// decoded from.
    pub(crate) fn decoded(value: nbtx::Value, original: Vec<u8>) -> Self {
        Self {
            value,
            original: Some(original),
        }
    }

    pub(crate) fn value(&self) -> &nbtx::Value {
        &self.value
    }

    /// Mutable access to the wrapped value. Clears the preserved original
    /// bytes -- see the type docs -- so the next [`Self::encode`]
    /// re-derives from the edited value instead of returning stale bytes.
    pub(crate) fn value_mut(&mut self) -> &mut nbtx::Value {
        self.original = None;
        &mut self.value
    }

    pub(crate) fn into_value(self) -> nbtx::Value {
        self.value
    }

    /// Encodes the wrapped value: returns the preserved original bytes
    /// verbatim if there are any, else re-derives from `value` -- see the
    /// type docs for why these two paths can disagree on an empty list
    /// field.
    pub(crate) fn encode(&self) -> Result<Vec<u8>> {
        if let Some(original) = &self.original {
            return Ok(original.clone());
        }
        Ok(nbtx::to_le_bytes(&self.value)?)
    }
}

/// One decoded `map_<id>` record: an opaque NBT compound plus typed
/// accessors for the fields listed in the module docs.
///
/// Holds the decoded [`nbtx::Value`] rather than a fixed struct -- see the
/// module docs' field-order finding for why a fixed field order cannot
/// reproduce every real record byte-identically. `Value::Compound` is
/// order-preserving, which is what makes a re-derived encoding reproduce a
/// decoded record's field layout exactly -- with one further wrinkle
/// [`RetainedNbt`]'s doc comment covers.
#[derive(Debug, Clone, PartialEq)]
pub struct MapRecord {
    inner: RetainedNbt,
}

impl MapRecord {
    /// Wraps an already-decoded NBT value as a record with no original
    /// on-disk bytes to fall back to -- see [`RetainedNbt`]'s doc comment
    /// for the consequence of that on a value with an empty list field.
    pub fn new(value: nbtx::Value) -> Self {
        Self {
            inner: RetainedNbt::new(value),
        }
    }

    /// The record's full NBT value, verbatim -- every field it carries, in
    /// on-disk order. The escape hatch for `decorations`' internal shape
    /// and anything else the typed accessors below do not cover.
    pub fn value(&self) -> &nbtx::Value {
        self.inner.value()
    }

    fn field(&self, name: &str) -> Option<&nbtx::Value> {
        let nbtx::Value::Compound(compound) = self.inner.value() else {
            return None;
        };
        compound.get(name.as_bytes())
    }

    fn long(&self, name: &str) -> Option<i64> {
        match self.field(name)? {
            nbtx::Value::Long(v) => Some(*v),
            _ => None,
        }
    }

    fn byte(&self, name: &str) -> Option<i8> {
        match self.field(name)? {
            nbtx::Value::Byte(v) => Some(*v),
            _ => None,
        }
    }

    fn short(&self, name: &str) -> Option<i16> {
        match self.field(name)? {
            nbtx::Value::Short(v) => Some(*v),
            _ => None,
        }
    }

    fn int(&self, name: &str) -> Option<i32> {
        match self.field(name)? {
            nbtx::Value::Int(v) => Some(*v),
            _ => None,
        }
    }

    fn bool_byte(&self, name: &str) -> Option<bool> {
        self.byte(name).map(|v| v != 0)
    }

    /// The `mapId` field: this map's own id, matching its key's suffix on
    /// every real record found.
    pub fn map_id(&self) -> Option<i64> {
        self.long("mapId")
    }

    /// The `parentMapId` field: the id of the map this one was copied or
    /// expanded from, or the same value as [`Self::map_id`] on a map with
    /// no parent.
    pub fn parent_map_id(&self) -> Option<i64> {
        self.long("parentMapId")
    }

    /// The `scale` field: the map's zoom level (0 = most zoomed in).
    pub fn scale(&self) -> Option<i8> {
        self.byte("scale")
    }

    /// The `dimension` field, widened to an `i32` and resolved to a
    /// [`Dimension`] the same way a chunk key's dimension field is --
    /// see the module docs for the `Byte`/`Int` width note.
    pub fn dimension(&self) -> Option<Dimension> {
        match self.field("dimension")? {
            nbtx::Value::Byte(v) => Some(Dimension::from(i32::from(*v))),
            nbtx::Value::Int(v) => Some(Dimension::from(*v)),
            _ => None,
        }
    }

    /// The `width` field, in pixels/columns.
    pub fn width(&self) -> Option<i16> {
        self.short("width")
    }

    /// The `height` field, in pixels/rows.
    pub fn height(&self) -> Option<i16> {
        self.short("height")
    }

    /// The `xCenter` field: the world X coordinate the map is centered on.
    pub fn x_center(&self) -> Option<i32> {
        self.int("xCenter")
    }

    /// The `zCenter` field: the world Z coordinate the map is centered on.
    pub fn z_center(&self) -> Option<i32> {
        self.int("zCenter")
    }

    /// The `unlimitedTracking` field.
    pub fn unlimited_tracking(&self) -> Option<bool> {
        self.bool_byte("unlimitedTracking")
    }

    /// The `mapLocked` field.
    pub fn map_locked(&self) -> Option<bool> {
        self.bool_byte("mapLocked")
    }

    /// The `fullyExplored` field -- present on every real record found but
    /// absent from the originally proposed field list (see the module
    /// docs).
    pub fn fully_explored(&self) -> Option<bool> {
        self.bool_byte("fullyExplored")
    }

    /// The `colors` field: `width * height * 4` RGBA bytes, `None` if the
    /// field is absent or not a byte array.
    pub fn colors(&self) -> Option<&[u8]> {
        match self.field("colors")? {
            nbtx::Value::ByteArray(bytes) => Some(bytes.as_slice()),
            _ => None,
        }
    }

    /// The `decorations` field, verbatim -- see the module docs for why
    /// its per-entry shape (`{data: {...}, key: {...}}`) is not modeled
    /// further.
    pub fn decorations(&self) -> Option<&[nbtx::Value]> {
        match self.field("decorations")? {
            nbtx::Value::List(entries) => Some(entries.as_slice()),
            _ => None,
        }
    }
}

/// Decodes a `map_<id>` value: one uncompressed little-endian NBT compound
/// with no trailing bytes.
pub fn decode(bytes: &[u8]) -> Result<MapRecord> {
    let mut cursor = Cursor::new(bytes);
    let value: nbtx::Value = nbtx::from_le_bytes(&mut cursor)?;
    if cursor.position() != bytes.len() as u64 {
        return Err(Error::Invalid(
            "map record has trailing bytes past its NBT compound",
        ));
    }
    Ok(MapRecord {
        inner: RetainedNbt::decoded(value, bytes.to_vec()),
    })
}

/// Encodes a `map_<id>` value: the inverse of [`decode`]. See
/// [`RetainedNbt`]'s doc comment for how a decoded-but-unmodified record's
/// original bytes are preferred over a re-derivation, and why that matters
/// for this record's `decorations` field specifically.
pub fn encode(record: &MapRecord) -> Result<Vec<u8>> {
    record.inner.encode()
}

/// Every map id present in `db`, found by a full-database key scan --
/// nothing in the format indexes map ids the way `PositionTrackDB-LastId`
/// indexes lodestone tracking entries (see
/// [`crate::position_track`]), so there is no cheaper way to enumerate
/// them. Order matches the database's own iteration order, not numeric or
/// insertion order.
pub fn ids(db: &Database) -> Result<Vec<i64>> {
    let mut out = Vec::new();
    let mut keys = db.keys()?;
    for kv in &mut keys {
        if let Some(id) = id_from_key(&kv.key()) {
            out.push(id);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_record() -> nbtx::Value {
        let mut compound = nbtx::Compound::new();
        compound.insert("mapId".into(), nbtx::Value::Long(-64424509182));
        compound.insert("parentMapId".into(), nbtx::Value::Long(-64424509182));
        compound.insert("scale".into(), nbtx::Value::Byte(3));
        compound.insert("dimension".into(), nbtx::Value::Byte(0));
        compound.insert("width".into(), nbtx::Value::Short(128));
        compound.insert("height".into(), nbtx::Value::Short(128));
        compound.insert("xCenter".into(), nbtx::Value::Int(448));
        compound.insert("zCenter".into(), nbtx::Value::Int(448));
        compound.insert("unlimitedTracking".into(), nbtx::Value::Byte(0));
        compound.insert("mapLocked".into(), nbtx::Value::Byte(0));
        compound.insert("fullyExplored".into(), nbtx::Value::Byte(1));
        compound.insert("decorations".into(), nbtx::Value::List(vec![]));
        compound.insert(
            "colors".into(),
            nbtx::Value::ByteArray(vec![0u8; 128 * 128 * 4]),
        );
        nbtx::Value::Compound(compound)
    }

    #[test]
    fn key_and_id_from_key_round_trip_negative_ids() {
        let k = key(-64424509182);
        assert_eq!(k, b"map_-64424509182".to_vec());
        assert_eq!(id_from_key(&k), Some(-64424509182));
    }

    #[test]
    fn key_and_id_from_key_round_trip_positive_ids() {
        let k = key(42);
        assert_eq!(k, b"map_42".to_vec());
        assert_eq!(id_from_key(&k), Some(42));
    }

    #[test]
    fn id_from_key_rejects_non_map_keys() {
        assert_eq!(id_from_key(b"portals"), None);
        assert_eq!(id_from_key(b"mapmap_1"), None);
        assert_eq!(id_from_key(b"map_"), None);
        assert_eq!(id_from_key(b"map_abc"), None);
        assert_eq!(id_from_key(b"map_1x"), None);
    }

    /// `"map_00"` used to parse as id `0` -- `i64::from_str` accepts a
    /// leading zero on its own -- exactly the confusion the doc comment on
    /// [`id_from_key`] says can't happen.
    #[test]
    fn id_from_key_rejects_a_leading_zero_followed_by_another_digit() {
        assert_eq!(id_from_key(b"map_00"), None);
        assert_eq!(id_from_key(b"map_01"), None);
        assert_eq!(id_from_key(b"map_-00"), None);
        assert_eq!(id_from_key(b"map_-01"), None);
        // A bare "0" (or "-0") is not "a leading zero followed by another
        // digit" -- id 0 itself must still parse.
        assert_eq!(id_from_key(b"map_0"), Some(0));
        assert_eq!(id_from_key(b"map_-0"), Some(0));
    }

    #[test]
    fn accessors_read_every_field() {
        let record = MapRecord::new(sample_record());
        assert_eq!(record.map_id(), Some(-64424509182));
        assert_eq!(record.parent_map_id(), Some(-64424509182));
        assert_eq!(record.scale(), Some(3));
        assert_eq!(record.dimension(), Some(Dimension::Overworld));
        assert_eq!(record.width(), Some(128));
        assert_eq!(record.height(), Some(128));
        assert_eq!(record.x_center(), Some(448));
        assert_eq!(record.z_center(), Some(448));
        assert_eq!(record.unlimited_tracking(), Some(false));
        assert_eq!(record.map_locked(), Some(false));
        assert_eq!(record.fully_explored(), Some(true));
        assert_eq!(record.decorations(), Some(&[][..]));
        assert_eq!(record.colors().map(|c| c.len()), Some(128 * 128 * 4));
    }

    #[test]
    fn dimension_accepts_int_width_too() {
        let mut compound = nbtx::Compound::new();
        compound.insert("dimension".into(), nbtx::Value::Int(1));
        let record = MapRecord::new(nbtx::Value::Compound(compound));
        assert_eq!(record.dimension(), Some(Dimension::Nether));
    }

    #[test]
    fn decode_then_encode_round_trips() {
        let bytes = nbtx::to_le_bytes(&sample_record()).unwrap();
        let record = decode(&bytes).unwrap();
        let re_encoded = encode(&record).unwrap();
        assert_eq!(re_encoded, bytes);
    }

    /// From the corpus: `v1_18_30`'s `map_-17179868860`, the
    /// `colors, decorations, dimension, ...` field ordering. Only the
    /// first 64 bytes of the 65732-byte value are asserted -- the rest is
    /// 65536 bytes of mostly-zero color data -- but decode-then-encode
    /// below covers the whole record.
    #[test]
    fn decode_then_encode_reproduces_a_real_record_with_the_alphabetical_field_order() {
        let mut compound = nbtx::Compound::new();
        compound.insert(
            "colors".into(),
            nbtx::Value::ByteArray(vec![0u8; 128 * 128 * 4]),
        );
        compound.insert("decorations".into(), nbtx::Value::List(vec![]));
        compound.insert("dimension".into(), nbtx::Value::Byte(0));
        compound.insert("fullyExplored".into(), nbtx::Value::Byte(0));
        compound.insert("height".into(), nbtx::Value::Short(128));
        compound.insert("mapId".into(), nbtx::Value::Long(-17179868861));
        compound.insert("mapLocked".into(), nbtx::Value::Byte(0));
        compound.insert("parentMapId".into(), nbtx::Value::Long(-17179868860));
        compound.insert("scale".into(), nbtx::Value::Byte(3));
        compound.insert("unlimitedTracking".into(), nbtx::Value::Byte(0));
        compound.insert("width".into(), nbtx::Value::Short(128));
        compound.insert("xCenter".into(), nbtx::Value::Int(448));
        compound.insert("zCenter".into(), nbtx::Value::Int(448));
        let value = nbtx::Value::Compound(compound);

        let bytes = nbtx::to_le_bytes(&value).unwrap();
        let record = decode(&bytes).unwrap();
        assert_eq!(encode(&record).unwrap(), bytes);
    }

    /// Regression pin for the finding in the module docs' "An empty list
    /// loses its declared element type" section: a real record's empty
    /// `decorations` list is written with element type `Compound` (`0x0a`),
    /// not the `TAG_End` (`0x00`) a generic re-derivation from `Value`
    /// would produce. Hand-built with the real element-type byte (rather
    /// than going through `nbtx::to_le_bytes` on a `Value::List(vec![])`,
    /// which would silently launder the exact byte this test exists to
    /// pin) so a regression that stops preferring `original` in [`encode`]
    /// is caught here even though the *content* still round-trips fine.
    #[test]
    fn decode_then_encode_preserves_an_empty_lists_real_element_type() {
        let mut bytes = Vec::new();
        bytes.push(0x0a); // root Compound tag
        bytes.extend_from_slice(&0u16.to_le_bytes()); // root name length 0
        bytes.push(0x09); // "decorations" -> List tag
        bytes.extend_from_slice(&11u16.to_le_bytes());
        bytes.extend_from_slice(b"decorations");
        bytes.push(0x0a); // element type: Compound, not the TAG_End a
        // generic re-derivation from `Value::List(vec![])` would produce
        bytes.extend_from_slice(&0u32.to_le_bytes()); // 0 elements
        bytes.push(0x00); // root Compound end

        let record = decode(&bytes).unwrap();
        assert_eq!(record.decorations(), Some(&[][..]));
        assert_eq!(encode(&record).unwrap(), bytes);
    }

    /// The documented limitation on the other side of that same finding: a
    /// record built fresh through [`MapRecord::new`] (no original bytes to
    /// fall back to) re-derives from `value` and so *cannot* reproduce a
    /// `Compound`-typed empty list -- it always encodes as `TAG_End`. This
    /// pins the limitation itself, not a bug: [`decode`]-sourced records
    /// (the only ones ever read off real disk) are unaffected, per the
    /// test directly above.
    #[test]
    fn a_freshly_built_records_empty_list_loses_its_intended_element_type() {
        let mut compound = nbtx::Compound::new();
        compound.insert("decorations".into(), nbtx::Value::List(vec![]));
        let record = MapRecord::new(nbtx::Value::Compound(compound));

        let bytes = encode(&record).unwrap();
        // Byte 12 is the element-type byte in this specific hand-counted
        // layout (root Compound tag + 2-byte name length, then the
        // "decorations" List tag + 2-byte name length + 11 name bytes).
        let element_type_offset = 1 + 2 + 1 + 2 + "decorations".len();
        assert_eq!(bytes[element_type_offset], 0x00); // TAG_End, not 0x0a
    }

    #[test]
    fn decode_rejects_trailing_bytes() {
        let mut bytes = nbtx::to_le_bytes(&sample_record()).unwrap();
        bytes.push(0xff);
        assert!(decode(&bytes).is_err());
    }

    #[test]
    fn accessors_return_none_on_missing_or_wrong_typed_fields() {
        let record = MapRecord::new(nbtx::Value::Compound(nbtx::Compound::new()));
        assert_eq!(record.map_id(), None);
        assert_eq!(record.dimension(), None);
        assert_eq!(record.colors(), None);
        assert_eq!(record.decorations(), None);

        let wrong_type = MapRecord::new(nbtx::Value::Byte(0));
        assert_eq!(wrong_type.map_id(), None);
    }
}
