//! `PosTrackDB-0x<id>` and `PositionTrackDB-LastId`: lodestone position
//! tracking.
//!
//! Placing a compass on a lodestone assigns the lodestone a sequential
//! tracking id (starting at 1) and records its position under
//! `PosTrackDB-0x` followed by that id as 8 lower-case hex digits --
//! e.g. `PosTrackDB-0x00000001`. `PositionTrackDB-LastId` is a single
//! counter record holding the most recently assigned id, letting a reader
//! enumerate every entry (`1..=last_id`) without a database-wide scan --
//! the same role `digp` plays for actors, except this index is written by
//! the game itself, not recomputed by this crate. Neither key is
//! chunk-scoped: both are flat, database-wide keys like [`crate::portals`],
//! not a [`crate::key::Key`].
//!
//! # Corpus
//!
//! Present in exactly one of the 14 worlds scanned (the 1.26 seed world,
//! one `PosTrackDB-0x00000001` entry plus the `LastId` counter; the 13
//! fixtures carry neither key). Both real records match the originally
//! proposed field set and order exactly --
//! entry: `{dim: Int, id: String, pos: List<Int> (3), status: Byte,
//! version: Byte}`; `LastId`: `{id: String, version: Byte}` -- and a
//! reference implementation that both reads and writes this exact shape
//! (same field names, same order, same `"0x" + 8 lowercase hex digits`
//! id-string convention) agrees. [`Entry`]/[`LastId`] are therefore typed
//! structs, matching [`crate::portals`]'s reasoning: one real, uniform
//! sample plus an independently-implemented reference is enough evidence
//! for a fixed field order, unlike [`crate::map_data`]'s demonstrated
//! per-record variation.
//!
//! `id` is carried as a `String` on both types even though it is
//! reconstructible from the numeric id ([`Entry::key`]/[`key`] format it
//! the same way): it is a real, independently-writable field -- nothing in
//! the format guarantees a game-written value always agrees with its own
//! key suffix, only that the one real record found does -- so round
//! tripping it verbatim, rather than resynthesizing it from the numeric
//! id, is what byte-identical re-encoding requires.
//!
//! Unknown fields on either type are tolerated on decode (matching this
//! crate's `player.rs` convention for typed NBT records) rather than
//! rejected, per architecture decision 11 -- "degrade on unmodelled" --
//! but are not preserved through a decode then encode cycle, since
//! nothing in the corpus has ever shown one.

use std::io::Cursor;

use facet::Facet;

use crate::db::Database;
use crate::error::{Error, Result};

/// The ASCII prefix on a `PosTrackDB-0x<id>` key, including the `0x` --
/// the id suffix is exactly 8 lower-case hex digits with no `0x` of its
/// own.
pub const KEY_PREFIX: &[u8] = b"PosTrackDB-0x";

/// The `PositionTrackDB-LastId` counter key.
pub const LAST_ID_KEY: &[u8] = b"PositionTrackDB-LastId";

/// Builds the on-disk key for tracking id `id`.
pub fn key(id: u32) -> Vec<u8> {
    format!("PosTrackDB-0x{id:08x}").into_bytes()
}

/// Extracts the tracking id from a `PosTrackDB-0x<id>` key, if
/// `key_bytes` is shaped like one: [`KEY_PREFIX`] followed by exactly 8
/// lower-case hex digits and nothing else.
pub fn id_from_key(key_bytes: &[u8]) -> Option<u32> {
    let suffix = key_bytes.strip_prefix(KEY_PREFIX)?;
    if suffix.len() != 8 || !suffix.iter().all(|&b| is_ascii_lowercase_hexdigit(b)) {
        return None;
    }
    let suffix = std::str::from_utf8(suffix).ok()?;
    u32::from_str_radix(suffix, 16).ok()
}

/// Formats a tracking id the way the format's own `id` field carries it:
/// `"0x"` followed by 8 lower-case hex digits.
pub fn id_string(id: u32) -> String {
    format!("0x{id:08x}")
}

fn is_ascii_lowercase_hexdigit(b: u8) -> bool {
    b.is_ascii_digit() || (b'a'..=b'f').contains(&b)
}

/// One `PosTrackDB-0x<id>` entry: a lodestone's tracked position.
///
/// `pos` is `Vec<nbtx::Value>` rather than `Vec<i32>`: this codec maps a
/// plain `Vec<i32>` field to the NBT `IntArray` tag, but the real record
/// stores `pos` as a `List` of `Int` -- a different tag with the same
/// element type. Holding the raw per-element `Value`s reproduces the
/// on-disk `List` tag on re-encode; [`Self::pos`] is the typed accessor
/// over it.
#[derive(Facet, Debug, Clone, PartialEq, Eq)]
#[facet(nbtx::allow_unknown_fields)]
pub struct Entry {
    /// The dimension id the lodestone was placed in.
    pub dim: i32,
    /// This entry's own tracking id, as `"0x" + 8 lower-case hex digits`
    /// -- see the module docs for why this is not resynthesized from the
    /// key.
    pub id: String,
    /// The lodestone's block position, as raw `Int` list elements -- see
    /// the struct docs for why this isn't `Vec<i32>`. Every real record
    /// found has exactly 3 elements; use [`Self::pos`] for the typed
    /// `[x, y, z]` view.
    pos: Vec<nbtx::Value>,
    /// `0` for actively tracked, `1` for a pending update -- Bedrock has
    /// no broader tracking-state concept than this one bit.
    pub status: i8,
    pub version: i8,
}

impl Entry {
    /// Builds an entry from typed fields, encoding `pos` as the `List` of
    /// `Int` the real format uses (see the struct docs).
    pub fn new(dim: i32, id: String, pos: [i32; 3], status: i8, version: i8) -> Self {
        Self {
            dim,
            id,
            pos: pos.into_iter().map(nbtx::Value::Int).collect(),
            status,
            version,
        }
    }

    /// This entry's on-disk key, parsing [`Self::id`] the same way
    /// [`id_from_key`] would need to for a round trip -- `None` if `id`
    /// is not the `"0x" + 8 lower-case hex digits` shape every real
    /// record uses.
    pub fn key(&self) -> Option<Vec<u8>> {
        let hex = self.id.strip_prefix("0x")?;
        let id = u32::from_str_radix(hex, 16).ok()?;
        Some(key(id))
    }

    /// The lodestone's block position as `[x, y, z]`, `None` if `pos`
    /// does not have exactly 3 `Int` elements (every real record found
    /// does).
    pub fn pos(&self) -> Option<[i32; 3]> {
        let [x, y, z] = <&[nbtx::Value; 3]>::try_from(self.pos.as_slice()).ok()?;
        match (x, y, z) {
            (nbtx::Value::Int(x), nbtx::Value::Int(y), nbtx::Value::Int(z)) => Some([*x, *y, *z]),
            _ => None,
        }
    }
}

/// The `PositionTrackDB-LastId` counter record.
#[derive(Facet, Debug, Clone, PartialEq, Eq)]
#[facet(nbtx::allow_unknown_fields)]
pub struct LastId {
    /// The most recently assigned tracking id, as `"0x" + 8 lower-case hex
    /// digits`.
    pub id: String,
    pub version: i8,
}

impl LastId {
    /// [`Self::id`] parsed back to a numeric tracking id, `None` if it is
    /// not the `"0x" + 8 lower-case hex digits` shape every real record
    /// uses.
    pub fn last_id(&self) -> Option<u32> {
        u32::from_str_radix(self.id.strip_prefix("0x")?, 16).ok()
    }
}

/// Decodes a `PosTrackDB-0x<id>` value.
///
/// # Errors
///
/// Returns an error if `bytes` is not a valid NBT [`Entry`], or
/// [`Error::Invalid`] if bytes follow the compound or its `pos` list is not
/// exactly three `Int`s.
pub fn decode_entry(bytes: &[u8]) -> Result<Entry> {
    let mut cursor = Cursor::new(bytes);
    let entry: Entry = nbtx::from_le_bytes(&mut cursor)?;
    if cursor.position() != bytes.len() as u64 {
        return Err(Error::Invalid(
            "position track entry has trailing bytes past its NBT compound",
        ));
    }
    if entry.pos().is_none() {
        return Err(Error::Invalid(
            "position track entry's pos list is not exactly 3 Int elements",
        ));
    }
    Ok(entry)
}

/// Encodes a `PosTrackDB-0x<id>` value. The inverse of [`decode_entry`].
///
/// # Errors
///
/// Returns an error if `entry` cannot be encoded as NBT.
pub fn encode_entry(entry: &Entry) -> Result<Vec<u8>> {
    Ok(nbtx::to_le_bytes(entry)?)
}

/// Decodes a `PositionTrackDB-LastId` value.
///
/// # Errors
///
/// Returns an error if `bytes` is not a valid NBT [`LastId`], or
/// [`Error::Invalid`] if bytes follow the compound.
pub fn decode_last_id(bytes: &[u8]) -> Result<LastId> {
    let mut cursor = Cursor::new(bytes);
    let last_id: LastId = nbtx::from_le_bytes(&mut cursor)?;
    if cursor.position() != bytes.len() as u64 {
        return Err(Error::Invalid(
            "position track last-id record has trailing bytes past its NBT compound",
        ));
    }
    Ok(last_id)
}

/// Encodes a `PositionTrackDB-LastId` value. The inverse of
/// [`decode_last_id`].
///
/// # Errors
///
/// Returns an error if `last_id` cannot be encoded as NBT.
pub fn encode_last_id(last_id: &LastId) -> Result<Vec<u8>> {
    Ok(nbtx::to_le_bytes(last_id)?)
}

/// Reads the `PositionTrackDB-LastId` counter from `db`, if present.
///
/// # Errors
///
/// Returns an error if the database read fails or the stored value does not
/// [`decode_last_id`].
pub fn read_last_id(db: &Database) -> Result<Option<LastId>> {
    match db.get(LAST_ID_KEY)? {
        Some(bytes) => Ok(Some(decode_last_id(&bytes)?)),
        None => Ok(None),
    }
}

/// Reads tracking entry `id` from `db`, if present.
///
/// # Errors
///
/// Returns an error if the database read fails or the stored value does not
/// [`decode_entry`].
pub fn read_entry(db: &Database, id: u32) -> Result<Option<Entry>> {
    match db.get(key(id))? {
        Some(bytes) => Ok(Some(decode_entry(&bytes)?)),
        None => Ok(None),
    }
}

/// Reads every tracking entry from `db`, using [`LastId`] to enumerate
/// `1..=last_id` rather than a database-wide scan (see the module docs).
/// Returns an empty vector if [`LAST_ID_KEY`] is absent. An id in range
/// that has no entry at all (the counter advanced but the entry itself was
/// never written, or was later deleted) is skipped rather than surfaced --
/// the counter is this crate's only enumeration hint, not a guarantee
/// every id in its range resolves.
///
/// An id that *does* have a value yields that value's `Result` rather
/// than a bare `Entry`: per architecture decision 11, bulk iteration
/// yields a per-item `Result` so one malformed entry does not discard
/// every entry already collected, or abort the rest of the range. A
/// database-level error (as opposed to a malformed record) still
/// propagates immediately through the outer `Result`, matching
/// [`crate::db::Database::keys`]'s own iterator.
///
/// # Errors
///
/// Returns an error if a database read fails or the [`LAST_ID_KEY`] value
/// does not [`decode_last_id`]. A malformed entry is reported in its own
/// item instead.
pub fn read_all(db: &Database) -> Result<Vec<(u32, Result<Entry>)>> {
    let Some(last_id) = read_last_id(db)?.and_then(|l| l.last_id()) else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for id in 1..=last_id {
        if let Some(bytes) = db.get(key(id))? {
            out.push((id, decode_entry(&bytes)));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_and_id_from_key_round_trip() {
        let k = key(1);
        assert_eq!(k, b"PosTrackDB-0x00000001".to_vec());
        assert_eq!(id_from_key(&k), Some(1));

        let k = key(0xdead_beef);
        assert_eq!(id_from_key(&k), Some(0xdead_beef));
    }

    #[test]
    fn id_from_key_rejects_wrong_shapes() {
        assert_eq!(id_from_key(b"PosTrackDB-0x0000001"), None); // 7 digits
        assert_eq!(id_from_key(b"PosTrackDB-0x000000001"), None); // 9 digits
        assert_eq!(id_from_key(b"PosTrackDB-0xDEADBEEF"), None); // upper-case
        assert_eq!(id_from_key(b"PositionTrackDB-LastId"), None);
    }

    #[test]
    fn id_string_matches_the_format_the_key_uses() {
        assert_eq!(id_string(1), "0x00000001");
    }

    /// From the corpus: the 1.26 seed world's only `PosTrackDB-0x00000001`
    /// entry, 75 bytes.
    #[test]
    fn decode_then_encode_reproduces_the_real_entry() {
        let bytes: &[u8] = &[
            0x0a, 0x00, 0x00, 0x03, 0x03, 0x00, 0x64, 0x69, 0x6d, 0x00, 0x00, 0x00, 0x00, 0x08,
            0x02, 0x00, 0x69, 0x64, 0x0a, 0x00, 0x30, 0x78, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
            0x30, 0x31, 0x09, 0x03, 0x00, 0x70, 0x6f, 0x73, 0x03, 0x03, 0x00, 0x00, 0x00, 0xf7,
            0x00, 0x00, 0x00, 0x46, 0x00, 0x00, 0x00, 0x05, 0x01, 0x00, 0x00, 0x01, 0x06, 0x00,
            0x73, 0x74, 0x61, 0x74, 0x75, 0x73, 0x00, 0x01, 0x07, 0x00, 0x76, 0x65, 0x72, 0x73,
            0x69, 0x6f, 0x6e, 0x01, 0x00,
        ];

        let entry = decode_entry(bytes).unwrap();
        assert_eq!(
            entry,
            Entry::new(0, "0x00000001".to_string(), [247, 70, 261], 0, 1)
        );
        assert_eq!(entry.pos(), Some([247, 70, 261]));
        assert_eq!(entry.key().unwrap(), key(1));
        assert_eq!(&encode_entry(&entry).unwrap(), bytes);
    }

    /// From the corpus: the 1.26 seed world's `PositionTrackDB-LastId`, 32
    /// bytes.
    #[test]
    fn decode_then_encode_reproduces_the_real_last_id() {
        let bytes: &[u8] = &[
            0x0a, 0x00, 0x00, 0x08, 0x02, 0x00, 0x69, 0x64, 0x0a, 0x00, 0x30, 0x78, 0x30, 0x30,
            0x30, 0x30, 0x30, 0x30, 0x30, 0x31, 0x01, 0x07, 0x00, 0x76, 0x65, 0x72, 0x73, 0x69,
            0x6f, 0x6e, 0x01, 0x00,
        ];

        let last_id = decode_last_id(bytes).unwrap();
        assert_eq!(
            last_id,
            LastId {
                id: "0x00000001".to_string(),
                version: 1,
            }
        );
        assert_eq!(last_id.last_id(), Some(1));
        assert_eq!(&encode_last_id(&last_id).unwrap(), bytes);
    }

    #[test]
    fn decode_entry_rejects_a_pos_list_with_the_wrong_length() {
        let mut entry = Entry::new(0, "0x00000001".to_string(), [1, 2, 3], 0, 1);
        entry.pos.pop();
        let bytes = nbtx::to_le_bytes(&entry).unwrap();
        assert!(decode_entry(&bytes).is_err());
    }

    #[test]
    fn decode_rejects_malformed_bytes() {
        assert!(decode_entry(&[0xff]).is_err());
        assert!(decode_last_id(&[]).is_err());
    }

    #[test]
    fn decode_entry_rejects_trailing_bytes() {
        let entry = Entry::new(0, "0x00000001".to_string(), [1, 2, 3], 0, 1);
        let mut bytes = nbtx::to_le_bytes(&entry).unwrap();
        bytes.push(0xff);
        assert!(decode_entry(&bytes).is_err());
    }

    #[test]
    fn decode_last_id_rejects_trailing_bytes() {
        let last_id = LastId {
            id: "0x00000001".to_string(),
            version: 1,
        };
        let mut bytes = nbtx::to_le_bytes(&last_id).unwrap();
        bytes.push(0xff);
        assert!(decode_last_id(&bytes).is_err());
    }

    #[test]
    fn entry_key_is_none_for_a_mismatched_id_string() {
        let entry = Entry::new(0, "not-hex".to_string(), [0, 0, 0], 0, 1);
        assert_eq!(entry.key(), None);
    }

    /// Per architecture decision 11, a malformed entry in the middle of the
    /// `1..=last_id` range must not discard the entries already collected
    /// or abort the rest of the range -- it surfaces as that one item's
    /// `Err`, and every other id still resolves.
    #[test]
    fn read_all_yields_a_per_item_result_and_does_not_discard_other_entries() {
        let dir = tempfile::tempdir().unwrap();
        let db = Database::open(dir.path().join("db").to_str().unwrap()).unwrap();

        let entry_1 = Entry::new(0, id_string(1), [1, 2, 3], 0, 1);
        let entry_3 = Entry::new(0, id_string(3), [4, 5, 6], 0, 1);

        db.insert(key(1), encode_entry(&entry_1).unwrap()).unwrap();
        db.insert(key(2), b"not valid nbt").unwrap();
        db.insert(key(3), encode_entry(&entry_3).unwrap()).unwrap();
        db.insert(
            LAST_ID_KEY,
            encode_last_id(&LastId {
                id: id_string(3),
                version: 1,
            })
            .unwrap(),
        )
        .unwrap();

        let results = read_all(&db).unwrap();
        assert_eq!(results.len(), 3);

        assert_eq!(results[0].0, 1);
        assert_eq!(results[0].1.as_ref().unwrap(), &entry_1);

        assert_eq!(results[1].0, 2);
        assert!(results[1].1.is_err());

        assert_eq!(results[2].0, 3);
        assert_eq!(results[2].1.as_ref().unwrap(), &entry_3);
    }
}
