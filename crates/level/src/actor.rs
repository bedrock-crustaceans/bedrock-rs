//! Actor (entity) storage: the `digp`/`actorprefix` scheme 1.18.30+ worlds
//! use, plus the legacy inline `0x32` list older worlds still carry.
//!
//! A chunk's entities can live in either of two shapes, and a full read has
//! to check both:
//!
//! - **Modern (`digp` + `actorprefix`).** [`KeyVariant::ActorDigest`]
//!   (`digp`) is a chunk-scoped record -- an ordinary member of that chunk's
//!   [`ChunkRecords`] group -- whose value is a flat array of 8-byte
//!   [`ActorStorageKey`]s. Each one names an `actorprefix` record: a global
//!   (not chunk-scoped) key of its own, `b"actorprefix"` followed by that
//!   same 8 bytes, holding one uncompressed little-endian NBT compound with
//!   the actor's full state. The storage key is derived from the entity's
//!   `UniqueID` rather than being that value verbatim, and this crate never
//!   computes that derivation -- it only carries the key bytes through.
//! - **Legacy (`0x32`).** [`KeyVariant::Entity`] holds a bare concatenation
//!   of little-endian NBT compounds with no count prefix, read until the
//!   value is exhausted. Also an ordinary `ChunkRecords` member.
//!
//! A subset of `digp`-era worlds additionally carry [`KeyVariant::ActorDigestVersion`]
//! (`0x41`), a one-byte stamp for the digest scheme's own revision -- see
//! [`decode_actor_digest_version`].
//!
//! Neither shape's NBT payload is modeled field by field here -- per
//! architecture decision 8 (vanilla data lives in a separate crate; core
//! stays independent of it), an actor's fields are game content, not format,
//! so both decode to raw [`nbtx::Value`]. The one piece of an actor record
//! this crate *does* have a structural reason to know about is the
//! storage-key linkage the digest exists to express, which is what
//! [`ChunkActors::read`] surfaces.
//!
//! Per architecture decision 13, `digp` is a **recomputed index**: a
//! disagreement between it and the `actorprefix` records it names does not
//! merely go stale like a cache, it leaks or hides entities. This module's
//! read side tolerates an existing disagreement without panicking or
//! aborting the rest of the chunk's actor read (see [`ChunkActors::dangling`]
//! and decision 11's malformed/unmodelled split).
//!
//! ## Write-side maintenance
//!
//! [`ChunkActors::add`]/[`ChunkActors::remove`] are the mutable actor
//! surface: [`crate::level_chunk::Chunk::actors_mut`] exposes them, and
//! [`crate::level_chunk::Chunk::encode_at`] regenerates the `digp` value
//! from the resulting [`ChunkActors::resolved`] set on every encode (decision
//! 13's "always recomputed" policy for this record) -- which is also where a
//! dangling entry left over from before an edit quietly drops out, since a
//! digest recomputed from the actual actor set never re-includes something
//! it could not resolve to a record in the first place. [`ChunkActors::add`]
//! requires the caller to supply the storage key rather than deriving one
//! from the entity's `UniqueID`: as the module docs above state, this crate
//! does not know that derivation, so guessing at it here would be exactly
//! the kind of fabrication decision 11 rules out -- a caller that has (or
//! can compute) the real derivation passes the key it produces; a caller
//! that does not has no honest way to synthesize one.
//!
//! `actorprefix` writes and deletions are a separate concern from `digp`
//! itself, because `actorprefix` keys have no chunk coordinates and so can
//! never be members of a [`ChunkRecords`] group (see `crate::chunk`'s module
//! docs) -- [`Chunk::encode_at`](crate::level_chunk::Chunk::encode_at) cannot
//! carry them. [`Chunk::encode_actor_writes`](crate::level_chunk::Chunk::encode_actor_writes)
//! is the other half: it reports the [`ActorWriteSet`] -- new/changed
//! `actorprefix` records to upsert, plus every storage key the *previous*
//! `digp` (as decoded) named that the *new* one does not, which is exactly
//! the removed-actor keys and any GC-eligible orphans this chunk's own
//! digest used to vouch for. `crate::level::Level::put_chunk` applies it
//! alongside the `ChunkRecords` write automatically; a caller driving
//! `Chunk::encode_at` directly, without going through a `Level`, still owns
//! that second write itself -- see `Chunk::encode_actor_writes`'s own docs.
//! `Level::remove_chunk` covers the deletion side of the same problem for a
//! chunk removed outright: it deletes the `actorprefix` records that
//! chunk's own `digp` named, dangling entries included (deleting an already-
//! absent key is a documented no-op -- see [`ActorWriteSet::apply`]).

use std::collections::HashSet;
use std::io::Cursor;

use crate::chunk::ChunkRecords;
use crate::db::Database;
use crate::error::{Error, Result};
use crate::key::KeyVariant;

/// The ASCII prefix on an `actorprefix` key.
pub const ACTOR_PREFIX: &[u8] = b"actorprefix";

/// Length in bytes of an actor storage key -- the suffix of an `actorprefix`
/// key, and one entry of a `digp` value.
pub const ACTOR_STORAGE_KEY_LEN: usize = 8;

/// Total length in bytes of a well-formed `actorprefix` key:
/// [`ACTOR_PREFIX`] followed by one [`ActorStorageKey`].
pub const ACTOR_PREFIX_KEY_LEN: usize = ACTOR_PREFIX.len() + ACTOR_STORAGE_KEY_LEN;

/// The 8-byte on-disk actor storage key: one entry of a `digp` digest, and
/// the suffix of the `actorprefix` key it names. Derived from the entity's
/// `UniqueID` long rather than being that long verbatim (see the module
/// docs) -- this crate carries the bytes through without computing or
/// interpreting them.
///
/// A fixed-size `[u8; 8]`, not [`bstr::BString`]: the settled NBT-string
/// finding (`tests/nbt_actor_storage_key.rs`) is specifically that these
/// same 8 bytes *also* appear, verbatim, as the payload of an NBT string tag
/// inside the resolved record --
/// `internalComponents.EntityStorageKeyComponent.StorageKey` -- where they
/// are frequently not valid UTF-8. That is a fact about what an NBT string
/// tag is allowed to hold ([`nbtx::Value::String`] is a `BString` for
/// exactly this reason), not about what this type is; here the bytes are a
/// fixed-length binary key, and `[u8; 8]` says so at the type level. Compare
/// against a decoded record's own `StorageKey` field with `.as_slice()`.
pub type ActorStorageKey = [u8; ACTOR_STORAGE_KEY_LEN];

/// Builds the on-disk `actorprefix` key for `storage_key`.
pub fn actorprefix_key_bytes(storage_key: &ActorStorageKey) -> Vec<u8> {
    let mut out = Vec::with_capacity(ACTOR_PREFIX_KEY_LEN);
    out.extend_from_slice(ACTOR_PREFIX);
    out.extend_from_slice(storage_key);
    out
}

/// Parses a raw LevelDB key as an `actorprefix` key, returning its storage
/// key if it has that shape (`ACTOR_PREFIX_KEY_LEN` bytes, `ACTOR_PREFIX`
/// prefix). Returns `None` -- not an error -- for anything else: like
/// `Key::deserialize`'s string-key fallback, a key that is not this shape is
/// simply some other kind of record, not a malformed one.
pub fn parse_actorprefix_key(bytes: &[u8]) -> Option<ActorStorageKey> {
    if bytes.len() != ACTOR_PREFIX_KEY_LEN || &bytes[..ACTOR_PREFIX.len()] != ACTOR_PREFIX {
        return None;
    }
    let mut storage_key = [0u8; ACTOR_STORAGE_KEY_LEN];
    storage_key.copy_from_slice(&bytes[ACTOR_PREFIX.len()..]);
    Some(storage_key)
}

/// Decodes a `digp` value: a flat array of [`ActorStorageKey`]s with no
/// count or length prefix of its own, `len / 8` entries in on-disk order.
///
/// Per decision 11, a length that is not a multiple of 8 is malformed (not
/// merely unmodelled) and is an error rather than a truncated or padded
/// result.
pub fn decode_actor_digest(bytes: &[u8]) -> Result<Vec<ActorStorageKey>> {
    if !bytes.len().is_multiple_of(ACTOR_STORAGE_KEY_LEN) {
        return Err(Error::Invalid(
            "digp value length is not a multiple of 8 bytes",
        ));
    }
    Ok(bytes
        .chunks_exact(ACTOR_STORAGE_KEY_LEN)
        .map(|chunk| chunk.try_into().expect("chunk is exactly 8 bytes"))
        .collect())
}

/// Encodes a `digp` value: the inverse of [`decode_actor_digest`], and its
/// exact byte-for-byte round trip partner (each entry is copied verbatim, so
/// decode -> encode reproduces the input exactly regardless of content).
pub fn encode_actor_digest(storage_keys: &[ActorStorageKey]) -> Vec<u8> {
    let mut out = Vec::with_capacity(storage_keys.len() * ACTOR_STORAGE_KEY_LEN);
    for key in storage_keys {
        out.extend_from_slice(key);
    }
    out
}

/// Decodes an `actorprefix` value: one uncompressed little-endian NBT
/// compound, decoded to raw [`nbtx::Value`] rather than a typed struct --
/// per architecture decision 8, an actor's fields are game content this
/// crate stays independent of, not format this crate models.
pub fn decode_actor_record(bytes: &[u8]) -> Result<nbtx::Value> {
    let mut cursor = Cursor::new(bytes);
    Ok(nbtx::from_le_bytes(&mut cursor)?)
}

/// Encodes an `actorprefix` value: the inverse of [`decode_actor_record`].
/// Round-trips byte-identically for every real record this crate's test
/// suite has checked (`tests/nbt_actor_storage_key.rs`,
/// `tests/actor.rs`) because `Value` preserves NBT compound field order and
/// raw `BString` string payloads rather than normalizing either.
pub fn encode_actor_record(value: &nbtx::Value) -> Result<Vec<u8>> {
    Ok(nbtx::to_le_bytes(value)?)
}

/// Decodes a legacy inline `0x32` ([`KeyVariant::Entity`]) value: a bare
/// concatenation of little-endian NBT compounds with no count prefix, read
/// until the buffer is exhausted.
pub fn decode_legacy_entities(bytes: &[u8]) -> Result<Vec<nbtx::Value>> {
    let mut cursor = Cursor::new(bytes);
    let len = bytes.len() as u64;
    let mut out = Vec::new();
    while cursor.position() < len {
        let value: nbtx::Value = nbtx::from_le_bytes(&mut cursor)?;
        out.push(value);
    }
    Ok(out)
}

/// Encodes a legacy inline `0x32` value: the inverse of
/// [`decode_legacy_entities`], concatenating each compound's encoding with
/// no separator or count prefix, in the same order.
pub fn encode_legacy_entities(values: &[nbtx::Value]) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    for value in values {
        nbtx::to_le_bytes_in(&mut out, value)?;
    }
    Ok(out)
}

/// Decodes a `0x41` [`KeyVariant::ActorDigestVersion`] value: a single byte
/// stamping the revision of the `digp`/`actorprefix` digest scheme a
/// chunk's actor storage was written under. Every real record in the
/// corpus (597 across three fixtures) is exactly one byte holding `0`, the
/// only digest-scheme revision this format has shipped so far -- this
/// decode does not assume that value, only the one-byte length, so a future
/// revision stamped differently still round-trips.
///
/// Per decision 11, a value that is not exactly one byte is malformed
/// rather than unmodelled, since "the version marker" is definitionally a
/// single byte and nothing about a different length is a recognizable
/// variant of it.
pub fn decode_actor_digest_version(bytes: &[u8]) -> Result<u8> {
    match bytes {
        [version] => Ok(*version),
        _ => Err(Error::Invalid(
            "actorDigestVersion value is not exactly one byte",
        )),
    }
}

/// Encodes a `0x41` [`KeyVariant::ActorDigestVersion`] value: the inverse of
/// [`decode_actor_digest_version`].
pub fn encode_actor_digest_version(version: u8) -> Vec<u8> {
    vec![version]
}

/// One actor reached through a chunk's `digp` digest: the storage key the
/// digest named, paired with the `actorprefix` record found under it.
#[derive(Debug, Clone, PartialEq)]
pub struct ActorRecord {
    pub storage_key: ActorStorageKey,
    pub value: nbtx::Value,
}

/// One chunk's full entity set, merging every storage scheme the format has
/// used. See the module docs for what each field's source key holds on
/// disk.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ChunkActors {
    /// Actors resolved from this chunk's `digp` digest against real
    /// `actorprefix` records, in digest order.
    pub resolved: Vec<ActorRecord>,
    /// Storage keys the digest names for which no `actorprefix` record
    /// exists on disk -- the index and its referent already disagree,
    /// independently of anything this crate did. Per decision 11 this is
    /// reported rather than silently dropped, and does not fail the rest of
    /// the chunk's actor read; per decision 13, a write path that
    /// regenerates the digest (Phase 5) is what actually fixes it.
    pub dangling: Vec<ActorStorageKey>,
    /// Entities from this chunk's legacy inline `0x32` record, if it has
    /// one, in on-disk order.
    pub legacy: Vec<nbtx::Value>,
    /// Storage keys [`Self::add`] has placed into [`Self::resolved`] since
    /// this value was read (or built fresh) -- these are exactly the
    /// entries [`Self::write_plan`] must write a fresh `actorprefix` record
    /// for, because they are the only ones whose NBT this crate actually
    /// changed. An entry that arrived through [`ChunkActors::read`] and was
    /// never touched keeps its on-disk `actorprefix` record untouched too,
    /// which is what makes an unmodified chunk's encode a no-op on this
    /// side. Not `pub`: a caller mutates this set only through
    /// [`Self::add`]/[`Self::remove`], never directly.
    added: HashSet<ActorStorageKey>,
}

impl ChunkActors {
    /// Reads one chunk's full actor set: resolves its `digp` digest (if any)
    /// against `db`'s `actorprefix` records, and decodes its legacy inline
    /// `0x32` list (if any).
    ///
    /// `records` supplies the `digp` and `0x32` bytes already read as part of
    /// this chunk's [`ChunkRecords`] group (both are ordinary chunk-scoped
    /// members of it); `db` is consulted only to resolve the `actorprefix`
    /// records the digest names, since those are not chunk-scoped and never
    /// appear in a `ChunkRecords` group.
    ///
    /// A `digp` entry naming a missing `actorprefix` record is *not* an
    /// error: it is reported through [`Self::dangling`] instead, per
    /// decision 11 (see the module docs). A structurally malformed `digp`
    /// value (wrong length) or an `actorprefix`/`0x32` value that fails to
    /// decode as NBT is an error, since those are malformed records rather
    /// than a pre-existing index/referent disagreement -- decision 11's
    /// other half.
    pub fn read(records: &ChunkRecords, db: &Database) -> Result<Self> {
        let mut out = Self::default();

        if let Some(digest_bytes) = records.get(KeyVariant::ActorDigest) {
            for storage_key in decode_actor_digest(digest_bytes)? {
                let key_bytes = actorprefix_key_bytes(&storage_key);
                match db.get(&key_bytes)? {
                    Some(buf) => {
                        let value = decode_actor_record(&buf)?;
                        out.resolved.push(ActorRecord { storage_key, value });
                    }
                    None => out.dangling.push(storage_key),
                }
            }
        }

        if let Some(legacy_bytes) = records.get(KeyVariant::Entity) {
            out.legacy = decode_legacy_entities(legacy_bytes)?;
        }

        Ok(out)
    }

    /// Whether this chunk has no actors in either storage scheme.
    pub fn is_empty(&self) -> bool {
        self.resolved.is_empty() && self.dangling.is_empty() && self.legacy.is_empty()
    }

    /// Total actor count: resolved modern records plus legacy inline
    /// records. Deliberately excludes [`Self::dangling`] -- those are named
    /// by the digest but have no record to count as an actor.
    pub fn len(&self) -> usize {
        self.resolved.len() + self.legacy.len()
    }

    /// Adds a new actor under `storage_key`, or replaces the value of one
    /// already present under it -- an add-or-modify, not add-only, since
    /// this crate has no separate "edit an existing actor's NBT" entry
    /// point and remove-then-add through the same key is the only way to
    /// change one. `storage_key` is never derived here: see the module
    /// docs' write-side section for why a caller-supplied key is the only
    /// honest contract this crate can offer, whether the entity is
    /// genuinely new or being re-added under a key recovered some other
    /// way. Also clears `storage_key` out of [`Self::dangling`] if it was
    /// there -- a caller that supplies a record for a previously-dangling
    /// reference has just resolved it.
    ///
    /// The record only actually reaches disk once this chunk is encoded and
    /// the resulting [`ActorWriteSet`] (see [`Self::write_plan`]) is applied
    /// to a database.
    pub fn add(&mut self, storage_key: ActorStorageKey, value: nbtx::Value) {
        match self
            .resolved
            .iter_mut()
            .find(|record| record.storage_key == storage_key)
        {
            Some(existing) => existing.value = value,
            None => self.resolved.push(ActorRecord { storage_key, value }),
        }
        self.dangling.retain(|key| *key != storage_key);
        self.added.insert(storage_key);
    }

    /// Removes and returns the resolved actor stored under `storage_key`,
    /// if this chunk has one. `None` if `storage_key` names no resolved
    /// actor -- including if it only ever named a [`Self::dangling`] entry,
    /// which is not an actor this crate can "remove" (there was never a
    /// record backing it); a dangling entry drops out on its own the next
    /// time the digest is recomputed, precisely because recomputing never
    /// re-includes something with no record.
    ///
    /// Like [`Self::add`], the removal only reaches disk once this chunk is
    /// encoded and the resulting [`ActorWriteSet`] is applied.
    pub fn remove(&mut self, storage_key: &ActorStorageKey) -> Option<ActorRecord> {
        let index = self
            .resolved
            .iter()
            .position(|record| &record.storage_key == storage_key)?;
        let removed = self.resolved.remove(index);
        self.added.remove(storage_key);
        Some(removed)
    }

    /// The storage keys of every resolved actor, in [`Self::resolved`]'s own
    /// order -- exactly the entries a recomputed `digp` digest must name.
    /// Order matters here only insofar as it makes an unmodified chunk's
    /// recomputed digest byte-identical to the one it was decoded from:
    /// [`ChunkActors::read`] appends to `resolved` in on-disk digest order
    /// and skips dangling entries entirely, so replaying `resolved`'s own
    /// order (unless a caller has since added or removed something)
    /// reproduces the original digest exactly.
    pub fn digest_keys(&self) -> Vec<ActorStorageKey> {
        self.resolved
            .iter()
            .map(|record| record.storage_key)
            .collect()
    }

    /// Computes the `actorprefix` maintenance this chunk's actor edits
    /// require, given `original_digest` -- the storage keys the `digp`
    /// digest named as read, dangling entries included (their absence from
    /// [`Self::digest_keys`] means they are naturally covered by the
    /// deletion side below).
    ///
    /// Per decision 13 and the Phase 5 GC task: [`ActorWriteSet::upserts`]
    /// covers exactly the keys [`Self::add`] touched -- untouched resolved
    /// actors are excluded, since their on-disk record is already correct;
    /// [`ActorWriteSet::deletions`] covers exactly the keys
    /// `original_digest` named that [`Self::digest_keys`] no longer does --
    /// removed actors, and any stale orphan this chunk's own shrinking
    /// digest used to vouch for. It deliberately does **not** attempt to
    /// GC `actorprefix` keys the wider database holds that no `digp`
    /// anywhere ever named going into this call -- decision 13's index
    /// policy is scoped to what *this* index used to assert, not to the
    /// database's `actorprefix` keyspace as a whole, which no single
    /// chunk's encode has enough information to reason about.
    pub fn write_plan(&self, original_digest: &[ActorStorageKey]) -> Result<ActorWriteSet> {
        let new_keys: HashSet<ActorStorageKey> = self
            .resolved
            .iter()
            .map(|record| record.storage_key)
            .collect();
        let deletions: Vec<ActorStorageKey> = original_digest
            .iter()
            .copied()
            .filter(|key| !new_keys.contains(key))
            .collect();

        let mut upserts = Vec::with_capacity(self.added.len());
        for record in &self.resolved {
            if self.added.contains(&record.storage_key) {
                upserts.push((record.storage_key, encode_actor_record(&record.value)?));
            }
        }

        Ok(ActorWriteSet { upserts, deletions })
    }
}

/// The `actorprefix` half of writing an edited chunk: which storage keys
/// need a fresh record written, and which need their record deleted. See
/// [`ChunkActors::write_plan`] for how this is computed and
/// [`crate::level_chunk::Chunk::encode_actor_writes`] for how a caller gets
/// one.
///
/// Kept separate from [`ChunkRecords`] deliberately: `actorprefix` keys
/// carry no chunk coordinates, so unlike `digp` they can never be members of
/// a chunk-scoped record group (see `crate::chunk`'s module docs) and this
/// crate has no chunk-scoped container to put them in. Applying this value
/// is therefore a second, explicit database write a caller issues alongside
/// (not instead of) writing the chunk's own `ChunkRecords` group.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ActorWriteSet {
    /// `(storage key, encoded actorprefix value)` pairs to insert or
    /// overwrite.
    pub upserts: Vec<(ActorStorageKey, Vec<u8>)>,
    /// Storage keys whose `actorprefix` record should be deleted.
    pub deletions: Vec<ActorStorageKey>,
}

impl ActorWriteSet {
    /// Whether this plan has nothing to do -- the common case for a chunk
    /// whose actors were never touched.
    pub fn is_empty(&self) -> bool {
        self.upserts.is_empty() && self.deletions.is_empty()
    }

    /// Applies every upsert and deletion in this plan to `db`. Deleting a
    /// key that turns out to already be absent (a dangling reference that
    /// was never backed by a record to begin with) is not an error -- the
    /// end state is the same either way. The two lists are always disjoint
    /// by construction (see [`ChunkActors::write_plan`]: every upsert key is
    /// in the new digest, and no deletion key is), so the order the two
    /// loops below run in can never matter.
    pub fn apply(&self, db: &Database) -> Result<()> {
        for (storage_key, bytes) in &self.upserts {
            db.insert(actorprefix_key_bytes(storage_key), bytes.as_slice())?;
        }
        for storage_key in &self.deletions {
            db.remove(actorprefix_key_bytes(storage_key))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actorprefix_key_round_trips_through_parse() {
        let storage_key: ActorStorageKey = [1, 2, 3, 4, 5, 6, 7, 8];
        let bytes = actorprefix_key_bytes(&storage_key);
        assert_eq!(bytes.len(), ACTOR_PREFIX_KEY_LEN);
        assert_eq!(parse_actorprefix_key(&bytes), Some(storage_key));
    }

    #[test]
    fn parse_actorprefix_key_rejects_wrong_prefix() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"notactorpfx"); // 11 bytes, same length as ACTOR_PREFIX, wrong text
        assert_eq!(bytes.len(), ACTOR_PREFIX.len());
        bytes.extend_from_slice(&[0u8; ACTOR_STORAGE_KEY_LEN]);
        assert_eq!(bytes.len(), ACTOR_PREFIX_KEY_LEN);
        assert_eq!(parse_actorprefix_key(&bytes), None);
    }

    #[test]
    fn parse_actorprefix_key_rejects_wrong_length() {
        assert_eq!(parse_actorprefix_key(ACTOR_PREFIX), None);
        let mut too_long = actorprefix_key_bytes(&[0u8; ACTOR_STORAGE_KEY_LEN]);
        too_long.push(0xAB);
        assert_eq!(parse_actorprefix_key(&too_long), None);
    }

    #[test]
    fn decode_actor_digest_splits_into_eight_byte_entries() {
        let bytes: Vec<u8> = (0u8..24).collect();
        let keys = decode_actor_digest(&bytes).unwrap();
        assert_eq!(keys.len(), 3);
        assert_eq!(keys[0], [0, 1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(keys[1], [8, 9, 10, 11, 12, 13, 14, 15]);
        assert_eq!(keys[2], [16, 17, 18, 19, 20, 21, 22, 23]);
    }

    #[test]
    fn decode_actor_digest_rejects_length_not_a_multiple_of_eight() {
        let bytes = vec![0u8; 9];
        assert!(decode_actor_digest(&bytes).is_err());
    }

    #[test]
    fn decode_actor_digest_accepts_empty() {
        assert_eq!(
            decode_actor_digest(&[]).unwrap(),
            Vec::<ActorStorageKey>::new()
        );
    }

    #[test]
    fn actor_digest_round_trips() {
        let keys: Vec<ActorStorageKey> = vec![
            [1, 2, 3, 4, 5, 6, 7, 8],
            [9, 10, 11, 12, 13, 14, 15, 16],
            [0xff; 8],
        ];
        let bytes = encode_actor_digest(&keys);
        assert_eq!(decode_actor_digest(&bytes).unwrap(), keys);
    }

    #[test]
    fn chunk_actors_len_excludes_dangling() {
        let actors = ChunkActors {
            resolved: vec![ActorRecord {
                storage_key: [0; 8],
                value: nbtx::Value::Byte(0),
            }],
            dangling: vec![[1; 8], [2; 8]],
            legacy: vec![nbtx::Value::Byte(1)],
            ..Default::default()
        };
        assert_eq!(actors.len(), 2);
        assert!(!actors.is_empty());
    }

    #[test]
    fn chunk_actors_default_is_empty() {
        assert!(ChunkActors::default().is_empty());
        assert_eq!(ChunkActors::default().len(), 0);
    }

    #[test]
    fn decode_actor_digest_version_accepts_the_observed_value() {
        assert_eq!(decode_actor_digest_version(&[0x00]).unwrap(), 0);
    }

    #[test]
    fn decode_actor_digest_version_accepts_any_single_byte() {
        // Only the length is asserted -- an unobserved but well-formed
        // revision byte must not be rejected as malformed.
        assert_eq!(decode_actor_digest_version(&[0x01]).unwrap(), 1);
        assert_eq!(decode_actor_digest_version(&[0xff]).unwrap(), 255);
    }

    #[test]
    fn decode_actor_digest_version_rejects_wrong_length() {
        assert!(decode_actor_digest_version(&[]).is_err());
        assert!(decode_actor_digest_version(&[0x00, 0x00]).is_err());
    }

    #[test]
    fn actor_digest_version_round_trips() {
        for version in [0u8, 1, 255] {
            let bytes = encode_actor_digest_version(version);
            assert_eq!(decode_actor_digest_version(&bytes).unwrap(), version);
        }
    }

    #[test]
    fn add_appends_a_new_actor_and_marks_it_added() {
        let mut actors = ChunkActors::default();
        actors.add([1; 8], nbtx::Value::Byte(9));

        assert_eq!(actors.resolved.len(), 1);
        assert_eq!(actors.resolved[0].storage_key, [1; 8]);
        assert_eq!(actors.digest_keys(), vec![[1; 8]]);

        let plan = actors.write_plan(&[]).unwrap();
        assert_eq!(
            plan.upserts,
            vec![([1; 8], encode_actor_record(&nbtx::Value::Byte(9)).unwrap())]
        );
        assert!(plan.deletions.is_empty());
    }

    #[test]
    fn add_under_an_existing_key_replaces_its_value_in_place() {
        let mut actors = ChunkActors::default();
        actors.resolved.push(ActorRecord {
            storage_key: [1; 8],
            value: nbtx::Value::Byte(1),
        });

        actors.add([1; 8], nbtx::Value::Byte(2));

        assert_eq!(actors.resolved.len(), 1);
        assert_eq!(actors.resolved[0].value, nbtx::Value::Byte(2));
    }

    #[test]
    fn add_clears_the_same_key_out_of_dangling() {
        let mut actors = ChunkActors {
            dangling: vec![[1; 8]],
            ..Default::default()
        };

        actors.add([1; 8], nbtx::Value::Byte(0));

        assert!(actors.dangling.is_empty());
        assert_eq!(actors.resolved.len(), 1);
    }

    #[test]
    fn remove_takes_a_resolved_actor_out_and_unmarks_it_added() {
        let mut actors = ChunkActors::default();
        actors.add([1; 8], nbtx::Value::Byte(9));

        let removed = actors.remove(&[1; 8]);

        assert_eq!(
            removed,
            Some(ActorRecord {
                storage_key: [1; 8],
                value: nbtx::Value::Byte(9),
            })
        );
        assert!(actors.resolved.is_empty());
        // Re-adding must write a fresh record again -- removing an
        // actor that was `added` this session must not leave it
        // permanently excluded from `write_plan`'s upsert list.
        actors.add([1; 8], nbtx::Value::Byte(9));
        let plan = actors.write_plan(&[]).unwrap();
        assert_eq!(plan.upserts.len(), 1);
    }

    #[test]
    fn remove_of_an_absent_key_is_none() {
        let mut actors = ChunkActors::default();
        assert_eq!(actors.remove(&[0xAB; 8]), None);
    }

    #[test]
    fn write_plan_deletes_keys_the_old_digest_named_that_the_new_one_does_not() {
        let mut actors = ChunkActors::default();
        actors.resolved.push(ActorRecord {
            storage_key: [1; 8],
            value: nbtx::Value::Byte(1),
        });
        // [2; 8] was read from disk as part of this chunk's digest but is
        // not carried into `resolved` at all -- as if it were a dangling
        // entry that never had a backing record.
        let original_digest = vec![[1; 8], [2; 8]];

        actors.remove(&[1; 8]);

        let plan = actors.write_plan(&original_digest).unwrap();
        // Both [1; 8] (explicitly removed) and [2; 8] (never resolved, so
        // implicitly dropped by the recompute) are gone from the new
        // digest, so both are GC'd.
        let mut deletions = plan.deletions.clone();
        deletions.sort();
        assert_eq!(deletions, vec![[1; 8], [2; 8]]);
        assert!(plan.upserts.is_empty());
    }

    #[test]
    fn write_plan_leaves_unmodified_resolved_actors_out_of_the_upsert_list() {
        let mut actors = ChunkActors::default();
        actors.resolved.push(ActorRecord {
            storage_key: [1; 8],
            value: nbtx::Value::Byte(1),
        });
        let original_digest = vec![[1; 8]];

        let plan = actors.write_plan(&original_digest).unwrap();
        assert!(plan.upserts.is_empty());
        assert!(plan.deletions.is_empty());
    }

    #[test]
    fn actor_write_set_is_empty_reports_correctly() {
        assert!(ActorWriteSet::default().is_empty());
        let non_empty = ActorWriteSet {
            upserts: vec![([1; 8], vec![0])],
            deletions: vec![],
        };
        assert!(!non_empty.is_empty());
    }
}
