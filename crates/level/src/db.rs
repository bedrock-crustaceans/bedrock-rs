use std::{marker::PhantomData, num::NonZeroU8, ops::Deref, sync::Arc, time::Duration};

use leveldb::{Compressor, CompressorList, DB, DBIterator, Options, Status, StatusCode};
use miniz_oxide::{
    deflate::{compress_to_vec, compress_to_vec_zlib},
    inflate::{decompress_to_vec, decompress_to_vec_zlib},
};

pub use leveldb::CompactionMode;

use crate::{error::Result, iter::Keys};

/// On-disk block compression identifiers used by the world format.
///
/// Uncompressed blocks are stored under id 0, which the database always provides.
/// `zlib` (with header) is id 2 and raw deflate (no zlib header or trailer) is id 4.
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

/// A LevelDB database.
pub struct Database {
    db: DB,
}

impl Database {
    /// Opens a LevelDB database at the specified `path`. This `path` should point to the `db` directory
    /// of a world, not the world itself.
    pub fn open<P: AsRef<str>>(path: P) -> Result<Self> {
        Self::open_with(path, OpenOptions::default())
    }

    /// Opens a LevelDB database like [`open`](Self::open) with the given [`OpenOptions`].
    pub fn open_with<P: AsRef<str>>(path: P, options: OpenOptions) -> Result<Self> {
        let db = DB::open(path.as_ref(), self::options(options))?;
        Ok(Self { db })
    }

    /// Runs any compaction the database needs, on the calling thread unless the database was opened
    /// with [`CompactionMode::Background`]. Databases opened with [`CompactionMode::Manual`] should
    /// call this periodically, since reads slow down as uncompacted tables pile up.
    pub fn compact(&self) -> Result<()> {
        self.db.maybe_compact()?;
        Ok(())
    }

    /// Compacts on the calling thread for roughly `budget` and returns whether more is pending, so
    /// the backlog can be drained a little at a time instead of blocking until
    /// [`compact`](Self::compact) finishes. A compaction that outlives the budget is paused and
    /// resumed by the next call. With [`CompactionMode::Background`] it only wakes the compaction thread.
    pub fn compact_step(&self, budget: Duration) -> Result<bool> {
        Ok(self.db.compact_step(budget)?)
    }

    /// Applies every write in `batch` atomically, as a single write to the log.
    pub fn write(&self, batch: WriteBatch) -> Result<()> {
        self.db.write(batch.0, false)?;
        Ok(())
    }

    /// Creates a forward iterator over the underlying database handle.
    pub(crate) fn new_iter(&self) -> Result<DBIterator> {
        Ok(self.db.new_iter()?)
    }

    /// Creates an iterator over all the keys in this database.
    pub fn keys(&self) -> Result<Keys<'_>> {
        Keys::new(self)
    }

    /// Attempts to retrieve the given key from the database.
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
    pub fn insert<K, V>(&self, key: K, value: V) -> Result<()>
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>,
    {
        self.db.put(key.as_ref(), value.as_ref())?;
        Ok(())
    }

    /// Removes a key from the database.
    pub fn remove<K>(&self, key: K) -> Result<()>
    where
        K: AsRef<[u8]>,
    {
        self.db.delete(key.as_ref())?;
        Ok(())
    }
}
