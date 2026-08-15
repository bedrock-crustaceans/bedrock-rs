//! The raw layer's chunk record group: every on-disk key that belongs to one
//! chunk position, grouped without interpreting any of the value bytes.
//!
//! A chunk is not one LevelDB record, it is a family of them sharing an
//! `(x, z, dimension)` prefix -- the version stamp, each subchunk, the
//! heightmap/biomes, the finalized-state byte, blending data, checksums, and
//! whatever else the format has stapled to a chunk position over its
//! history. [`ChunkRecords`] holds that whole family as `Key` plus raw
//! bytes, which is what makes a read-modify-write through it lossless: a
//! caller can pull out the handful of keys it actually understands, leave
//! everything else exactly as read, and write the result back unchanged in
//! every byte it did not touch.
//!
//! **Entities are not part of this group.** On 1.18.30+ worlds an entity
//! lives under `digp`/`actorprefix` keys, a different wire shape (an ASCII
//! prefix plus coordinates, no tag byte) that this layer does not parse yet
//! -- that is the not-yet-implemented Phase 2 actor-storage task's job, and
//! entities routing through `digp`/`actorprefix` are also a *recomputed
//! index* rather than a preserve-verbatim record, so they do not belong in
//! this generic byte-preservation container even once that parsing exists.
//! A world can carry thousands of these per chunk (one imported fixture has
//! 793 `digp` and 3748 `actorprefix` keys) and none of them are read,
//! written, or otherwise touched by [`ChunkRecords`]: a caller copying
//! chunks through this API alone drops every entity.

use std::collections::{HashMap, HashSet};
use std::io::Cursor;

use bedrock_shared::world::dimension::Dimension;

use crate::db::Database;
use crate::error::Result;
use crate::key::{Key, KeyVariant};
use crate::types::ChunkPosition;

/// One chunk position's complete record group, exactly as stored on disk.
///
/// This is the raw layer's chunk container: it holds the decoded [`Key`] and
/// raw value bytes of every **chunk-scoped** record belonging to one
/// `(chunk, dimension)` pair, and interprets none of them. A group read from
/// a [`Database`] and written back reproduces every record byte for byte --
/// nothing is dropped, nothing is re-derived. Building a typed chunk model
/// on top means taking the keys that model understands out of the group
/// with [`take`] and putting their re-encoded replacements back with
/// [`insert`]; whatever is never taken passes through untouched.
///
/// This does **not** include entities. `digp`/`actorprefix` keys are a
/// different wire shape this layer does not parse (see the module docs),
/// so a group built here never contains them and writing one back never
/// touches them either -- they are neither preserved nor destroyed by this
/// type, simply untouched, and the future actor-storage layer owns them.
///
/// [`take`]: ChunkRecords::take
/// [`insert`]: ChunkRecords::insert
#[derive(Debug, Clone)]
pub struct ChunkRecords {
    position: ChunkPosition,
    dimension: Dimension,
    records: HashMap<KeyVariant, Vec<u8>>,
    /// Variants [`take`](Self::take)n out and not since put back with
    /// [`insert`](Self::insert) -- on [`write`](Self::write), these are
    /// deleted from the target database rather than merely omitted, so a
    /// caller that takes a record and never reinserts it actually purges
    /// the stale copy instead of leaving it behind on disk.
    removed: HashSet<KeyVariant>,
}

impl ChunkRecords {
    /// Creates an empty record group for `position`/`dimension`. Used to
    /// start a brand-new chunk that has no on-disk records yet, and
    /// internally while reading an existing one.
    pub fn new(position: ChunkPosition, dimension: Dimension) -> Self {
        Self {
            position,
            dimension,
            records: HashMap::new(),
            removed: HashSet::new(),
        }
    }

    /// The chunk position this group belongs to.
    pub fn position(&self) -> ChunkPosition {
        self.position
    }

    /// The dimension this group belongs to.
    pub fn dimension(&self) -> Dimension {
        self.dimension
    }

    /// Number of records currently held (subchunks count individually, one
    /// per index).
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Whether this group holds no records at all.
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Whether a record for `variant` is present in this group.
    pub fn contains(&self, variant: KeyVariant) -> bool {
        self.records.contains_key(&variant)
    }

    /// Borrows the raw bytes stored for `variant`, if present.
    pub fn get(&self, variant: KeyVariant) -> Option<&[u8]> {
        self.records.get(&variant).map(Vec::as_slice)
    }

    /// Removes and returns the raw bytes stored for `variant`, if present.
    ///
    /// This is how a higher layer claims a record for interpretation: once
    /// taken, the record no longer passes through untouched on the next
    /// [`write`](Self::write). The caller is expected to [`insert`](Self::insert) a
    /// re-encoded replacement -- but if it does not, the omission is not
    /// silent: `write` deletes the on-disk key for a taken-and-not-replaced
    /// variant, so an intentional drop actually removes the stale record
    /// rather than merely leaving it unwritten-but-present.
    pub fn take(&mut self, variant: KeyVariant) -> Option<Vec<u8>> {
        let taken = self.records.remove(&variant);
        if taken.is_some() {
            self.removed.insert(variant);
        }
        taken
    }

    /// Inserts (or replaces) the raw bytes for a record, returning the
    /// previous bytes for that variant if any were present.
    ///
    /// `key` must belong to this group's own chunk position and dimension --
    /// that pair is fixed for the whole group and is not something an
    /// individual record can override. Inserting a variant that was
    /// previously [`take`](Self::take)n cancels the pending deletion `write` would
    /// otherwise have applied for it.
    ///
    /// # Panics
    ///
    /// Panics if `key.chunk`/`key.dimension` differ from this group's own
    /// position and dimension. Passing a key from a different chunk or
    /// dimension is a caller bug, not a data-quality question, so it panics
    /// rather than silently misfiling the record.
    pub fn insert(&mut self, key: Key, bytes: Vec<u8>) -> Option<Vec<u8>> {
        assert_eq!(
            key.chunk, self.position,
            "key belongs to a different chunk position than this record group"
        );
        assert_eq!(
            key.dimension, self.dimension,
            "key belongs to a different dimension than this record group"
        );
        self.removed.remove(&key.data);
        self.records.insert(key.data, bytes)
    }

    /// Iterates over every record currently held, as the on-disk [`Key`] it
    /// was (or will be) stored under paired with its raw bytes.
    pub fn iter(&self) -> ChunkRecordsIter<'_> {
        ChunkRecordsIter {
            position: self.position,
            dimension: self.dimension,
            inner: self.records.iter(),
        }
    }

    /// Writes every record in this group into `db`, reproducing the on-disk
    /// key for each one, then deletes the key for every variant
    /// [`take`](Self::take)n out and not since [`insert`](Self::insert)ed back. A group
    /// read from a database and written back through this reproduces every
    /// record byte for byte, which is architecture decision 3's raw-layer
    /// guarantee at its rawest -- this type adds grouping on top of that
    /// guarantee, not a relaxation of it. Entities under `digp`/`actorprefix`
    /// are outside this group entirely (see the module docs) and this never
    /// touches them, in either direction.
    pub fn write(&self, db: &Database) -> Result<()> {
        let mut key_buf = Vec::new();
        for (key, bytes) in self.iter() {
            key_buf.clear();
            key.serialize(&mut key_buf)?;
            db.insert(&key_buf, bytes)?;
        }
        for &variant in &self.removed {
            key_buf.clear();
            let key = Key {
                chunk: self.position,
                dimension: self.dimension,
                data: variant,
            };
            key.serialize(&mut key_buf)?;
            db.remove(&key_buf)?;
        }
        Ok(())
    }

    /// Reads one chunk position's record group from `db`.
    ///
    /// `db` exposes no key-prefix iteration, only a full linear scan (see
    /// [`Database::keys`]), so this necessarily visits every key in the
    /// database to find the ones belonging to `position`/`dimension` --
    /// O(keys in the world), not O(records in the chunk). Calling this in a
    /// loop over many chunks repeats that full scan every time; use
    /// [`read_many`](Self::read_many) or [`read_all`](Self::read_all)
    /// instead, which do the scan once no matter how many chunks are
    /// requested.
    ///
    /// Returns `Ok(None)` if no record for this position/dimension exists at
    /// all -- an absent chunk is an ordinary outcome, not an error. The
    /// returned group never includes entities: see the module docs for why
    /// `digp`/`actorprefix` are out of scope here.
    pub fn read(
        db: &Database,
        position: ChunkPosition,
        dimension: Dimension,
    ) -> Result<Option<Self>> {
        let mut groups = Self::read_many(db, [(position, dimension)])?;
        Ok(groups.remove(&(position, dimension)))
    }

    /// Reads the record groups for every `(position, dimension)` pair in
    /// `wanted`, in one linear scan of `db`. A pair with no records on disk
    /// is simply absent from the returned map.
    ///
    /// This is the efficient way to build groups for many chunks: the scan
    /// cost is paid once for the whole call, not once per chunk. As with
    /// [`read`](Self::read), no returned group includes entities -- see the
    /// module docs.
    pub fn read_many<I>(
        db: &Database,
        wanted: I,
    ) -> Result<HashMap<(ChunkPosition, Dimension), Self>>
    where
        I: IntoIterator<Item = (ChunkPosition, Dimension)>,
    {
        let wanted: HashSet<(ChunkPosition, Dimension)> = wanted.into_iter().collect();
        Self::scan(db, |id| wanted.contains(&id))
    }

    /// Reads every chunk's record group present in `db`, in one linear
    /// scan.
    ///
    /// This holds the raw bytes of the entire world in memory at once,
    /// which is the right trade-off for a whole-world pass (a migration
    /// tool, a bulk export) and the wrong one for touching a handful of
    /// chunks, where [`read_many`](Self::read_many) does the same single
    /// scan without materializing the rest of the world. No returned group
    /// includes entities -- see the module docs.
    pub fn read_all(db: &Database) -> Result<HashMap<(ChunkPosition, Dimension), Self>> {
        Self::scan(db, |_| true)
    }

    /// Shared scan behind [`read_many`](Self::read_many) and
    /// [`read_all`](Self::read_all): one pass over every key in `db`,
    /// keeping the chunk-scoped ones whose `(chunk, dimension)` id passes
    /// `accept`.
    ///
    /// A key that does not decode as a [`Key`] at all -- a global key such
    /// as the local-player record, or an entity key (`digp`/`actorprefix`,
    /// a different wire shape this crate does not parse; see the module
    /// docs) -- is simply not a member of any chunk's group and is skipped
    /// rather than erroring: decision 11 only requires failing loudly on a
    /// malformed *member* of a group, and a key that never parses as a
    /// chunk key was never a candidate member in the first place. Likewise
    /// [`KeyVariant::LocalPlayer`] is excluded even though it does parse --
    /// its `Key` carries a fixed placeholder position rather than real
    /// coordinates, so treating it as chunk-scoped would silently attach it
    /// to whatever chunk happens to sit at that placeholder.
    fn scan(
        db: &Database,
        mut accept: impl FnMut((ChunkPosition, Dimension)) -> bool,
    ) -> Result<HashMap<(ChunkPosition, Dimension), Self>> {
        let mut groups: HashMap<(ChunkPosition, Dimension), Self> = HashMap::new();
        let mut keys = db.keys()?;

        for kv in &mut keys {
            let key_bytes = kv.key();
            let mut cursor = Cursor::new(key_bytes.as_ref());
            let Ok(key) = Key::deserialize(&mut cursor) else {
                continue;
            };

            if !key.data.is_chunk_scoped() {
                continue;
            }

            let id = (key.chunk, key.dimension);
            if !accept(id) {
                continue;
            }

            groups
                .entry(id)
                .or_insert_with(|| Self::new(key.chunk, key.dimension))
                .records
                .insert(key.data, kv.value().into());
        }

        Ok(groups)
    }
}

/// Iterator over a [`ChunkRecords`]' records, yielding the on-disk [`Key`]
/// paired with its raw bytes. See [`ChunkRecords::iter`].
pub struct ChunkRecordsIter<'a> {
    position: ChunkPosition,
    dimension: Dimension,
    inner: std::collections::hash_map::Iter<'a, KeyVariant, Vec<u8>>,
}

impl<'a> Iterator for ChunkRecordsIter<'a> {
    type Item = (Key, &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        let (variant, bytes) = self.inner.next()?;
        Some((
            Key {
                chunk: self.position,
                dimension: self.dimension,
                data: *variant,
            },
            bytes.as_slice(),
        ))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ExactSizeIterator for ChunkRecordsIter<'_> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'a> IntoIterator for &'a ChunkRecords {
    type Item = (Key, &'a [u8]);
    type IntoIter = ChunkRecordsIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
