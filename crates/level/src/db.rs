use std::{marker::PhantomData, ops::Deref, sync::Arc};

use miniz_oxide::{
    deflate::{compress_to_vec, compress_to_vec_zlib},
    inflate::{decompress_to_vec, decompress_to_vec_zlib},
};
use rusty_leveldb::{
    Compressor, CompressorList, DB, DBIterator, Options, Status, StatusCode,
    compressor::{NoneCompressor, SnappyCompressor},
};

use crate::{error::Result, iter::Keys};

/// On-disk block compression identifiers used by the world format.
///
/// Uncompressed blocks are stored under id 0. Snappy is id 1: the game does not
/// emit it, but standard LevelDB and third-party tooling do, so such tables must
/// still decode. `zlib` (with header) is id 2 and raw deflate (no zlib header or
/// trailer) is id 4. Blocks are written with raw deflate; the smaller of the raw
/// and compressed payloads is kept and a block that does not shrink falls back to
/// id 0, so id 0 must always be present.
const COMPRESSOR_NONE: u8 = 0;
const COMPRESSOR_SNAPPY: u8 = 1;
const COMPRESSOR_ZLIB: u8 = 2;
const COMPRESSOR_RAW_DEFLATE: u8 = 4;

/// Tables are written with a 4 MiB block size.
const BLOCK_SIZE: usize = 4 * 1024 * 1024;

/// Default deflate compression level.
const COMPRESSION_LEVEL: u8 = 6;

/// `zlib`-framed deflate, matching id 2.
struct ZlibCompressor(u8);

impl Compressor for ZlibCompressor {
    fn encode(&self, block: Vec<u8>) -> rusty_leveldb::Result<Vec<u8>> {
        Ok(compress_to_vec_zlib(&block, self.0))
    }

    fn decode(&self, block: Vec<u8>) -> rusty_leveldb::Result<Vec<u8>> {
        decompress_to_vec_zlib(&block).map_err(|err| Status {
            code: StatusCode::CompressionError,
            err: err.to_string(),
        })
    }
}

/// Raw deflate without a `zlib` header or trailer, matching id 4.
struct RawDeflateCompressor(u8);

impl Compressor for RawDeflateCompressor {
    fn encode(&self, block: Vec<u8>) -> rusty_leveldb::Result<Vec<u8>> {
        Ok(compress_to_vec(&block, self.0))
    }

    fn decode(&self, block: Vec<u8>) -> rusty_leveldb::Result<Vec<u8>> {
        decompress_to_vec(&block).map_err(|err| Status {
            code: StatusCode::CompressionError,
            err: err.to_string(),
        })
    }
}

fn options() -> Options {
    let mut list = CompressorList::new();
    list.set_with_id(COMPRESSOR_NONE, NoneCompressor);
    list.set_with_id(COMPRESSOR_SNAPPY, SnappyCompressor);
    list.set_with_id(COMPRESSOR_ZLIB, ZlibCompressor(COMPRESSION_LEVEL));
    list.set_with_id(COMPRESSOR_RAW_DEFLATE, RawDeflateCompressor(COMPRESSION_LEVEL));

    let mut opt = Options::default();
    opt.compressor_list = Arc::new(list);
    opt.compressor = COMPRESSOR_RAW_DEFLATE;
    opt.block_size = BLOCK_SIZE;
    opt.create_if_missing = true;

    opt
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

/// A LevelDB database.
pub struct Database {
    db: DB,
}

impl Database {
    /// Opens a LevelDB database at the specified `path`. This `path` should point to the `db` directory
    /// of a world, not the world itself.
    pub fn open<P: AsRef<str>>(path: P) -> Result<Self> {
        let db = DB::open(path.as_ref(), options())?;
        Ok(Self { db })
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A value that shrinks well past the builder's keep-compressed threshold, so its block is
    /// stamped with the write compressor's id rather than falling back to id 0.
    fn compressible_value(seed: u8) -> Vec<u8> {
        let mut v = Vec::with_capacity(64 * 1024);
        for i in 0..(64 * 1024) {
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
            v.push(state as u8);
        }
        v
    }

    /// Writes `entries` into a fresh table stamped with `write_compressor`, forcing a flush so the
    /// data leaves the memtable and the block compressor actually runs.
    fn write_table(path: &str, write_compressor: u8, entries: &[(Vec<u8>, Vec<u8>)]) {
        let mut opt = options();
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

    fn roundtrip_with_write_compressor(write_compressor: u8) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("db");
        let path = path.to_str().unwrap();
        let entries = compressible_entries();
        write_table(path, write_compressor, &entries);
        assert_reads_back(path, &entries);
    }

    #[test]
    fn reads_back_raw_deflate_tables() {
        roundtrip_with_write_compressor(COMPRESSOR_RAW_DEFLATE);
    }

    #[test]
    fn reads_back_zlib_tables() {
        roundtrip_with_write_compressor(COMPRESSOR_ZLIB);
    }

    #[test]
    fn reads_back_snappy_tables() {
        roundtrip_with_write_compressor(COMPRESSOR_SNAPPY);
    }

    #[test]
    fn reads_back_uncompressed_tables() {
        roundtrip_with_write_compressor(COMPRESSOR_NONE);
    }

    #[test]
    fn reads_back_incompressible_values_stored_uncompressed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("db");
        let path = path.to_str().unwrap();
        let entries: Vec<_> = (0u64..64)
            .map(|i| (format!("key{i:03}").into_bytes(), incompressible_value(i)))
            .collect();
        write_table(path, COMPRESSOR_RAW_DEFLATE, &entries);
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
        write_table(path, COMPRESSOR_RAW_DEFLATE, &entries);

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
