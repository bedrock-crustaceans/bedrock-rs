use std::{marker::PhantomData, num::NonZeroU8, ops::Deref, sync::Arc, time::Duration};

use leveldb::{
    Compressor, CompressorList, DB, DBIterator, Options, Status, StatusCode,
    compressor::SnappyCompressor,
};
use miniz_oxide::{
    deflate::{compress_to_vec, compress_to_vec_zlib},
    inflate::{decompress_to_vec, decompress_to_vec_zlib},
};

pub use leveldb::CompactionMode;

use crate::{error::Result, iter::Keys};

/// On-disk block compression identifiers used by the world format.
///
/// Uncompressed blocks are stored under id 0, which the database always provides. Snappy is
/// id 1: the game does not emit it, but standard `LevelDB` and third-party tooling do, so such
/// tables must still decode. `zlib` (with header) is id 2 and raw deflate (no zlib header or
/// trailer) is id 4.
/// Blocks are written with raw deflate; the game selects the smaller of the raw and
/// compressed payloads and falls back to id 0 when the raw payload wins.
const COMPRESSOR_ZLIB: NonZeroU8 = NonZeroU8::new(2).unwrap();
const COMPRESSOR_RAW_DEFLATE: NonZeroU8 = NonZeroU8::new(4).unwrap();

/// Tables are written with a 32 KiB block size. Every read decompresses a whole block, so
/// larger blocks make point lookups of single chunk records much slower.
const BLOCK_SIZE: usize = 32 * 1024;

/// How a [`Database`] is opened.
#[derive(Debug, Clone, Copy)]
pub struct OpenOptions {
    /// Where compaction runs. [`CompactionMode::Background`] starts a dedicated compaction thread,
    /// [`CompactionMode::Inline`] compacts on whichever thread is writing and [`CompactionMode::Manual`]
    /// only compacts when [`Database::compact`] is called.
    pub compaction_mode: CompactionMode,
    /// Deflate level from 0 (store) to 10 (slowest). Level 1 writes about three times faster than
    /// level 6 for roughly 7% more disk space.
    pub compression_level: u8,
}

impl Default for OpenOptions {
    fn default() -> Self {
        Self {
            compaction_mode: CompactionMode::Background,
            compression_level: 1,
        }
    }
}

/// `zlib`-framed deflate, matching id 2.
struct ZlibCompressor(u8);

impl Compressor for ZlibCompressor {
    fn encode(&self, block: &[u8]) -> leveldb::Result<Vec<u8>> {
        Ok(compress_to_vec_zlib(block, self.0))
    }

    fn decode(&self, block: &[u8]) -> leveldb::Result<Vec<u8>> {
        decompress_to_vec_zlib(block).map_err(|err| Status {
            code: StatusCode::CompressionError,
            err: err.to_string(),
        })
    }
}

/// Raw deflate without a `zlib` header or trailer, matching id 4.
struct RawDeflateCompressor(u8);

impl Compressor for RawDeflateCompressor {
    fn encode(&self, block: &[u8]) -> leveldb::Result<Vec<u8>> {
        Ok(compress_to_vec(block, self.0))
    }

    fn decode(&self, block: &[u8]) -> leveldb::Result<Vec<u8>> {
        decompress_to_vec(block).map_err(|err| Status {
            code: StatusCode::CompressionError,
            err: err.to_string(),
        })
    }
}

fn options(open: OpenOptions) -> Options {
    let level = open.compression_level.min(10);
    let mut list = CompressorList::new();
    list.set(SnappyCompressor);
    list.set_with_id(COMPRESSOR_ZLIB, ZlibCompressor(level));
    list.set_with_id(COMPRESSOR_RAW_DEFLATE, RawDeflateCompressor(level));

    Options {
        compressor_list: Arc::new(list),
        compressor: Some(COMPRESSOR_RAW_DEFLATE),
        block_size: BLOCK_SIZE,
        create_if_missing: true,
        compaction_mode: open.compaction_mode,
        ..Default::default()
    }
}

/// An owned copy of a value read out of the database.
#[derive(Debug)]
pub struct Buffer<'db>(Vec<u8>, PhantomData<&'db ()>);

impl Buffer<'_> {
    pub(crate) fn new(data: Vec<u8>) -> Self {
        Self(data, PhantomData)
    }
}

impl Deref for Buffer<'_> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<[u8]> for Buffer<'_> {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<Buffer<'_>> for Vec<u8> {
    fn from(buf: Buffer<'_>) -> Self {
        buf.0
    }
}

/// A group of inserts and removals applied together by [`Database::write`]. Writing a chunk's
/// records as one batch is much cheaper than inserting them one at a time.
#[derive(Default)]
pub struct WriteBatch(leveldb::WriteBatch);

impl WriteBatch {
    pub fn new() -> Self {
        Self::default()
    }

    /// Queues an insertion of `value` at `key`.
    pub fn insert<K: AsRef<[u8]>, V: AsRef<[u8]>>(&mut self, key: K, value: V) {
        self.0.put(key.as_ref(), value.as_ref());
    }

    /// Queues the removal of `key`.
    pub fn remove<K: AsRef<[u8]>>(&mut self, key: K) {
        self.0.delete(key.as_ref());
    }

    /// The amount of queued operations.
    pub fn len(&self) -> usize {
        self.0.count() as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A `LevelDB` database.
pub struct Database {
    db: DB,
}

impl Database {
    /// Opens a `LevelDB` database at the specified `path`. This `path` should point to the `db` directory
    /// of a world, not the world itself.
    ///
    /// # Errors
    ///
    /// Returns an error if the database cannot be opened, for example when the path does not
    /// exist or another process holds its lock.
    pub fn open<P: AsRef<str>>(path: P) -> Result<Self> {
        Self::open_with(path, OpenOptions::default())
    }

    /// Opens a `LevelDB` database like [`open`](Self::open) with the given [`OpenOptions`].
    ///
    /// # Errors
    ///
    /// Returns an error under the same conditions as [`open`](Self::open).
    pub fn open_with<P: AsRef<str>>(path: P, options: OpenOptions) -> Result<Self> {
        let db = DB::open(path.as_ref(), self::options(options))?;
        Ok(Self { db })
    }

    /// Runs any compaction the database needs, on the calling thread unless the database was opened
    /// with [`CompactionMode::Background`]. Databases opened with [`CompactionMode::Manual`] should
    /// call this periodically, since reads slow down as uncompacted tables pile up.
    ///
    /// # Errors
    ///
    /// Returns an error if the compaction fails.
    pub fn compact(&self) -> Result<()> {
        self.db.maybe_compact()?;
        Ok(())
    }

    /// Compacts on the calling thread for roughly `budget` and returns whether more is pending, so
    /// the backlog can be drained a little at a time instead of blocking until
    /// [`compact`](Self::compact) finishes. A compaction that outlives the budget is paused and
    /// resumed by the next call. With [`CompactionMode::Background`] it only wakes the compaction thread.
    ///
    /// # Errors
    ///
    /// Returns an error if the compaction fails.
    pub fn compact_step(&self, budget: Duration) -> Result<bool> {
        Ok(self.db.compact_step(budget)?)
    }

    /// Applies every write in `batch` atomically, as a single write to the log.
    ///
    /// # Errors
    ///
    /// Returns an error if the write to the log fails.
    pub fn write(&self, batch: WriteBatch) -> Result<()> {
        self.db.write(batch.0, false)?;
        Ok(())
    }

    /// Applies `batch` like [`write`](Self::write) and waits until the log is flushed to disk, so
    /// the write survives a crash of the machine, not only of the process. Slower; meant for
    /// saves the user asked for.
    ///
    /// # Errors
    ///
    /// Returns an error if the write to the log or the flush fails.
    pub fn write_sync(&self, batch: WriteBatch) -> Result<()> {
        self.db.write(batch.0, true)?;
        Ok(())
    }

    /// Creates a forward iterator over the underlying database handle.
    ///
    /// # Errors
    ///
    /// Returns an error if the database cannot create an iterator.
    pub(crate) fn new_iter(&self) -> Result<DBIterator> {
        Ok(self.db.new_iter()?)
    }

    /// Creates an iterator over all the keys in this database.
    ///
    /// # Errors
    ///
    /// Returns an error if the database cannot create an iterator.
    pub fn keys(&self) -> Result<Keys<'_>> {
        Keys::new(self)
    }

    /// Attempts to retrieve the given key from the database.
    ///
    /// # Errors
    ///
    /// Returns an error if reading from the database fails. A missing key is `Ok(None)`.
    pub fn get<K>(&self, key: K) -> Result<Option<Buffer<'_>>>
    where
        K: AsRef<[u8]>,
    {
        let mut buf = Vec::new();
        if self.db.get_into(key.as_ref(), &mut buf)? {
            Ok(Some(Buffer::new(buf)))
        } else {
            Ok(None)
        }
    }

    /// Inserts a key-value pair into the database.
    ///
    /// # Errors
    ///
    /// Returns an error if the write fails.
    pub fn insert<K, V>(&self, key: K, value: V) -> Result<()>
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>,
    {
        self.db.put(key.as_ref(), value.as_ref())?;
        Ok(())
    }

    /// Removes a key from the database.
    ///
    /// # Errors
    ///
    /// Returns an error if the write fails.
    pub fn remove<K>(&self, key: K) -> Result<()>
    where
        K: AsRef<[u8]>,
    {
        self.db.delete(key.as_ref())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use leveldb::CompressorId;

    /// A value that shrinks well past the builder's keep-compressed threshold, so its block is
    /// stamped with the write compressor's id rather than falling back to id 0.
    fn compressible_value(seed: u8) -> Vec<u8> {
        let mut v = Vec::with_capacity(64 * 1024);
        for i in 0..(64 * 1024) {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "i is non-negative and only its low nibble is kept"
            )]
            v.push(seed ^ (i as u8 & 0x0f));
        }
        v
    }

    /// A value with no exploitable redundancy, so its block cannot beat the threshold and is
    /// stored under id 0.
    fn incompressible_value(seed: u64) -> Vec<u8> {
        let mut state = seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut v = Vec::with_capacity(64 * 1024);
        for _ in 0..(64 * 1024) {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            #[expect(
                clippy::cast_possible_truncation,
                reason = "each step contributes the low byte of the xorshift state"
            )]
            v.push(state as u8);
        }
        v
    }

    /// Writes `entries` into a fresh table stamped with `write_compressor`, forcing a flush so the
    /// data leaves the memtable and the block compressor actually runs.
    fn write_table(
        path: &str,
        write_compressor: Option<NonZeroU8>,
        entries: &[(Vec<u8>, Vec<u8>)],
    ) {
        let mut opt = options(OpenOptions::default());
        opt.compressor = write_compressor;
        let db = DB::open(path, opt).unwrap();
        for (key, value) in entries {
            db.put(key, value).unwrap();
        }
        db.flush().unwrap();
    }

    /// Reads every entry back through the standard Bedrock options and asserts byte-identity.
    fn assert_reads_back(path: &str, entries: &[(Vec<u8>, Vec<u8>)]) {
        let db = Database::open(path).unwrap();
        for (key, value) in entries {
            let got = db.get(key).unwrap().expect("value present");
            assert_eq!(&*got, value.as_slice());
        }
    }

    fn compressible_entries() -> Vec<(Vec<u8>, Vec<u8>)> {
        (0u8..80)
            .map(|i| (format!("key{i:03}").into_bytes(), compressible_value(i)))
            .collect()
    }

    fn roundtrip_with_write_compressor(write_compressor: Option<NonZeroU8>) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("db");
        let path = path.to_str().unwrap();
        let entries = compressible_entries();
        write_table(path, write_compressor, &entries);
        assert_reads_back(path, &entries);
    }

    #[test]
    fn reads_back_raw_deflate_tables() {
        roundtrip_with_write_compressor(Some(COMPRESSOR_RAW_DEFLATE));
    }

    #[test]
    fn reads_back_zlib_tables() {
        roundtrip_with_write_compressor(Some(COMPRESSOR_ZLIB));
    }

    #[test]
    fn reads_back_snappy_tables() {
        roundtrip_with_write_compressor(Some(SnappyCompressor::ID));
    }

    #[test]
    fn reads_back_uncompressed_tables() {
        roundtrip_with_write_compressor(None);
    }

    #[test]
    fn reads_back_incompressible_values_stored_uncompressed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("db");
        let path = path.to_str().unwrap();
        let entries: Vec<_> = (0u64..64)
            .map(|i| (format!("key{i:03}").into_bytes(), incompressible_value(i)))
            .collect();
        write_table(path, Some(COMPRESSOR_RAW_DEFLATE), &entries);
        assert_reads_back(path, &entries);
    }

    #[test]
    fn corrupt_block_surfaces_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("db");
        let path = path.to_str().unwrap();
        // A handful of small entries produce a single table with one data block, so the corruption
        // below lands in that block while the trailing index and footer stay intact and the read
        // still resolves the block handle.
        let entries: Vec<_> = (0u8..8)
            .map(|i| (vec![b'k', i], compressible_value(i)))
            .collect();
        write_table(path, Some(COMPRESSOR_RAW_DEFLATE), &entries);

        let table = std::fs::read_dir(path)
            .unwrap()
            .filter_map(|e| e.ok().map(|e| e.path()))
            .find(|p| p.extension().is_some_and(|ext| ext == "ldb"))
            .expect("flushed table");
        // Flip bytes just inside the first data block; the stored block checksum no longer matches,
        // so a read of a key in that block must fail rather than return a bad value or panic.
        let mut bytes = std::fs::read(&table).unwrap();
        for byte in bytes.iter_mut().skip(4).take(16) {
            *byte ^= 0xff;
        }
        std::fs::write(&table, &bytes).unwrap();

        let db = Database::open(path).unwrap();
        assert!(db.get(&entries[0].0).is_err());
    }
}
