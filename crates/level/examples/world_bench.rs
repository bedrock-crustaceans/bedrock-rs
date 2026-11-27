//! Measures the world stack on a real world: open, iterate every record, decode and re-encode every
//! sub chunk and `Data3D` record, and read chunks from many threads at once.
//!
//! ```text
//! cargo run --release -p bedrock_level --example world_bench -- <world dir> [--runs N]
//!     [--threads 1,12] [--verify] [--write <out dir>]
//! ```
//!
//! The world is copied into a temporary directory for every run, since opening a database may
//! rewrite its log. `--verify` prints digests of the decoded data so two builds can be compared;
//! `--write` saves a copy with every sub chunk and `Data3D` record re-encoded, to check that other
//! readers accept what this crate writes. Times are the median of the runs.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use bedrock_level::Greedy;
use bedrock_level::biome::Biomes;
use bedrock_level::chunk::ChunkRecords;
use bedrock_level::db::{Database, WriteBatch};
use bedrock_level::subchunk::SubChunk;
use bedrock_level::types::ChunkPosition;
use bedrock_shared::world::dimension::Dimension;

const SUBCHUNK: u8 = 0x2f;
const DATA_3D: u8 = 0x2b;
const AIR: &str = "minecraft:air";

struct Args {
    world: PathBuf,
    runs: usize,
    threads: Vec<usize>,
    verify: bool,
    write: Option<PathBuf>,
}

fn args() -> Args {
    let mut it = std::env::args().skip(1);
    let mut args = Args {
        world: PathBuf::new(),
        runs: 3,
        threads: vec![1, 12],
        verify: false,
        write: None,
    };
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--runs" => args.runs = it.next().and_then(|v| v.parse().ok()).expect("--runs N"),
            "--threads" => {
                args.threads = it
                    .next()
                    .expect("--threads 1,12")
                    .split(',')
                    .map(|v| v.parse().expect("thread count"))
                    .collect();
            }
            "--verify" => args.verify = true,
            "--write" => args.write = Some(it.next().expect("--write <dir>").into()),
            _ => args.world = arg.into(),
        }
    }
    assert!(
        args.world.join("db").is_dir(),
        "usage: world_bench <world dir> [--runs N] [--threads 1,12] [--verify] [--write <dir>]"
    );
    args
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).unwrap();
        }
    }
}

fn fresh_copy(world: &Path, run: usize) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("world_bench_{}_{run}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    copy_dir(world, &dir);
    dir
}

fn open(dir: &Path) -> Database {
    Database::open(dir.join("db").to_str().unwrap()).unwrap()
}

fn timed<T>(f: impl FnOnce() -> T) -> (T, Duration) {
    let start = Instant::now();
    let out = f();
    (out, start.elapsed())
}

const FNV0: u64 = 0xcbf2_9ce4_8422_2325;

/// FNV-1a, the same digest the editor's C++ benchmark prints.
fn fnv(mut hash: u64, bytes: &[u8]) -> u64 {
    for &b in bytes {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

/// The dimension, tag and sub chunk index of a chunk record key.
fn parse_key(key: &[u8]) -> Option<(i32, u8, Option<i8>)> {
    let (dim, at) = match key.len() {
        9 | 10 => (0, 8),
        13 | 14 => (i32::from_le_bytes(key[8..12].try_into().unwrap()), 12),
        _ => return None,
    };
    let tag = key[at];
    if !(0x2b..=0x41).contains(&tag) && tag != 0x76 && tag != 0x77 {
        return None;
    }
    let y = (key.len() == at + 2).then(|| key[at + 1].cast_signed());
    (y.is_some() == (tag == SUBCHUNK)).then_some((dim, tag, y))
}

fn chunk_of(key: &[u8], dim: i32) -> (i32, i32, i32) {
    let x = i32::from_le_bytes(key[0..4].try_into().unwrap());
    let z = i32::from_le_bytes(key[4..8].try_into().unwrap());
    (dim, x, z)
}

fn chunk_prefix(dim: i32, x: i32, z: i32) -> Vec<u8> {
    let mut prefix = Vec::with_capacity(12);
    prefix.extend_from_slice(&x.to_le_bytes());
    prefix.extend_from_slice(&z.to_le_bytes());
    if dim != 0 {
        prefix.extend_from_slice(&dim.to_le_bytes());
    }
    prefix
}

fn reencode_subchunk(value: &[u8]) -> Option<Vec<u8>> {
    let chunk = SubChunk::from_disk_greedy(&mut Cursor::new(value)).ok()?;
    let mut out = Cursor::new(Vec::with_capacity(value.len()));
    chunk.to_disk::<Greedy, _>(&mut out).ok()?;
    Some(out.into_inner())
}

fn reencode_data3d(value: &[u8]) -> Option<Vec<u8>> {
    let biomes = Biomes::from_disk::<Greedy, _>(&mut Cursor::new(value)).ok()?;
    let mut out = Cursor::new(Vec::with_capacity(value.len()));
    biomes.to_disk(&mut out).ok()?;
    Some(out.into_inner())
}

fn reencode(tag: u8, value: &[u8]) -> Option<Vec<u8>> {
    if tag == SUBCHUNK {
        reencode_subchunk(value)
    } else {
        reencode_data3d(value)
    }
}

#[derive(Default, PartialEq, Debug)]
struct Codec {
    subchunks: usize,
    data3d: usize,
    failed: usize,
    identical: usize,
    out_bytes: usize,
}

impl Codec {
    fn reencode(&mut self, tag: u8, value: &[u8]) {
        if tag == SUBCHUNK {
            self.subchunks += 1;
        } else {
            self.data3d += 1;
        }
        match reencode(tag, value) {
            None => self.failed += 1,
            Some(out) => {
                self.out_bytes += out.len();
                self.identical += usize::from(out == value);
            }
        }
    }
}

/// What the timed passes work on, collected by an untimed pass.
struct Contents {
    records: usize,
    bytes: usize,
    digest: u64,
    chunks: Vec<(i32, i32, i32)>,
    /// Every sub chunk and `Data3D` record: tag, key, value.
    codec_records: Vec<(u8, Vec<u8>, Vec<u8>)>,
}

fn contents(db: &Database) -> Contents {
    let mut out = Contents {
        records: 0,
        bytes: 0,
        digest: FNV0,
        chunks: Vec::new(),
        codec_records: Vec::new(),
    };
    let mut chunks = BTreeSet::new();
    let mut keys = db.keys().unwrap();
    while let Some((key, value)) = keys.next_ref() {
        out.records += 1;
        out.bytes += key.len() + value.len();
        out.digest = fnv(fnv(out.digest, key), value);
        if let Some((dim, tag, _)) = parse_key(key) {
            chunks.insert(chunk_of(key, dim));
            if tag == SUBCHUNK || tag == DATA_3D {
                out.codec_records.push((tag, key.to_vec(), value.to_vec()));
            }
        }
    }
    out.chunks = chunks.into_iter().collect();
    out
}

fn iterate_owned(db: &Database) -> usize {
    let mut bytes = 0;
    for kv in &mut db.keys().unwrap() {
        bytes += kv.key().len() + kv.value().len();
    }
    bytes
}

fn iterate_in_place(db: &Database) -> usize {
    let mut bytes = 0;
    let mut keys = db.keys().unwrap();
    while let Some((key, value)) = keys.next_ref() {
        bytes += key.len() + value.len();
    }
    bytes
}

fn iterate_and_reencode(db: &Database) -> Codec {
    let mut codec = Codec::default();
    let mut keys = db.keys().unwrap();
    while let Some((key, value)) = keys.next_ref() {
        if let Some((_, tag, _)) = parse_key(key)
            && (tag == SUBCHUNK || tag == DATA_3D)
        {
            codec.reencode(tag, value);
        }
    }
    codec
}

fn reencode_in_memory(records: &[(u8, Vec<u8>, Vec<u8>)]) -> Codec {
    let mut codec = Codec::default();
    for (tag, _, value) in records {
        codec.reencode(*tag, value);
    }
    codec
}

/// The records of one chunk, read with a single seek like the editor's `read_raw`: its sub
/// chunks by index and its `Data3D`.
struct RawChunk {
    subchunks: BTreeMap<i8, Vec<u8>>,
    data3d: Option<Vec<u8>>,
}

fn read_chunk(db: &Database, (dim, x, z): (i32, i32, i32)) -> RawChunk {
    let prefix = chunk_prefix(dim, x, z);
    let mut raw = RawChunk {
        subchunks: BTreeMap::new(),
        data3d: None,
    };
    let mut keys = db.keys().unwrap();
    keys.seek(&prefix);
    while let Some((key, value)) = keys.next_ref() {
        if !key.starts_with(&prefix[..8]) {
            break;
        }
        match parse_key(key) {
            Some((d, _, Some(y))) if d == dim => {
                raw.subchunks.insert(y, value.to_vec());
            }
            Some((d, DATA_3D, None)) if d == dim => raw.data3d = Some(value.to_vec()),
            _ => {}
        }
    }
    raw
}

/// The top non-air block of every column, from the sub chunks top down, and the biomes: the
/// work of the editor's surface summary. Returns the number of columns with a block.
fn surface(raw: &RawChunk) -> usize {
    let mut height = [i16::MIN; 256];
    let mut left = 256;
    for (&y, value) in raw.subchunks.iter().rev() {
        let Ok(chunk) = SubChunk::from_disk_greedy(&mut Cursor::new(value.as_slice())) else {
            continue;
        };
        let Some(layer) = chunk.get_layer(0) else {
            continue;
        };
        let air: Vec<bool> = layer
            .palette()
            .iter()
            .map(|b| b.name() == Some(AIR))
            .collect();
        for (column, h) in (0..).zip(height.iter_mut()) {
            if *h != i16::MIN {
                continue;
            }
            let (x, z) = (column & 15, column >> 4);
            for by in (0..16).rev() {
                let index = layer.indices().get((x << 8) | (z << 4) | by).unwrap_or(0);
                if !air[usize::from(index)] {
                    *h = i16::from(y) * 16 + i16::try_from(by).unwrap();
                    left -= 1;
                    break;
                }
            }
        }
        if left == 0 {
            break;
        }
    }
    if let Some(value) = &raw.data3d {
        let biomes = Biomes::from_disk::<Greedy, _>(&mut Cursor::new(value.as_slice())).unwrap();
        std::hint::black_box(&biomes);
    }
    256 - left
}

/// Reads every chunk, split evenly over `threads` threads; with `summarize` also builds its surface.
fn read_chunks(
    db: &Database,
    chunks: &[(i32, i32, i32)],
    threads: usize,
    summarize: bool,
) -> usize {
    let per_thread = chunks.len().div_ceil(threads).max(1);
    std::thread::scope(|scope| {
        let handles: Vec<_> = chunks
            .chunks(per_thread)
            .map(|part| {
                scope.spawn(move || {
                    part.iter()
                        .map(|&chunk| {
                            let raw = read_chunk(db, chunk);
                            if summarize {
                                surface(&raw)
                            } else {
                                raw.subchunks.len()
                            }
                        })
                        .sum::<usize>()
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).sum()
    })
}

/// Order-independent digest of a decoded sub chunk: block states are hashed sorted by name, so
/// two builds that decode the same data print the same value.
fn subchunk_digest(value: &[u8]) -> u64 {
    let Ok(chunk) = SubChunk::from_disk_greedy(&mut Cursor::new(value)) else {
        return 0;
    };
    let lazy = SubChunk::from_disk_lazy(&mut Cursor::new(value)).unwrap();
    let mut hash = fnv(
        FNV0,
        &[chunk.version().raw(), chunk.index().cast_unsigned()],
    );
    for i in 0.. {
        let Some(layer) = chunk.get_layer(i) else {
            break;
        };
        let lazy_layer = lazy.get_layer(i).unwrap();
        assert!(
            layer.indices().iter().eq(lazy_layer.indices().iter()),
            "the lazy and greedy indices differ"
        );
        for index in layer.indices() {
            hash = fnv(hash, &index.to_le_bytes());
        }
        for entry in layer.palette() {
            let Some(block) = entry.as_block() else {
                hash = fnv(hash, format!("{entry:?}").as_bytes());
                continue;
            };
            hash = fnv(hash, block.name.as_bytes());
            hash = fnv(hash, &block.version.unwrap_or(-1).to_le_bytes());
            let mut states: Vec<_> = block.states.iter().collect();
            states.sort_by(|a, b| a.0.cmp(b.0));
            for (key, state) in states {
                hash = fnv(hash, key.as_bytes());
                hash = fnv(hash, format!("{state:?}").as_bytes());
            }
        }
    }
    hash
}

/// Digests of every decoded sub chunk and every re-encoded `Data3D`, checking on the way that
/// a re-encoded sub chunk decodes to the same blocks.
fn verify(records: &[(u8, Vec<u8>, Vec<u8>)]) -> String {
    let (mut subchunks, mut data3d) = (FNV0, FNV0);
    for (tag, key, value) in records {
        if *tag == SUBCHUNK {
            let digest = subchunk_digest(value);
            subchunks = fnv(fnv(subchunks, key), &digest.to_le_bytes());
            if let Some(out) = reencode_subchunk(value) {
                assert_eq!(subchunk_digest(&out), digest, "re-encoding changed {key:?}");
            }
        } else {
            let out = reencode_data3d(value).unwrap_or_default();
            data3d = fnv(fnv(data3d, key), &out);
        }
    }
    format!("verify: sub chunk digest {subchunks:016x}, Data3D re-encoded digest {data3d:016x}")
}

/// Re-encodes every sub chunk and `Data3D` record into `db`, then copies the world to `out` and
/// reads it back.
fn write_reencoded(
    db: Database,
    dir: &Path,
    records: &[(u8, Vec<u8>, Vec<u8>)],
    out: &Path,
) -> String {
    let mut batch = WriteBatch::new();
    for (tag, key, value) in records {
        if let Some(encoded) = reencode(*tag, value) {
            batch.insert(key, encoded);
        }
        if batch.len() >= 1024 {
            db.write(std::mem::take(&mut batch)).unwrap();
        }
    }
    if !batch.is_empty() {
        db.write(batch).unwrap();
    }
    db.compact().unwrap();
    drop(db);

    let _ = std::fs::remove_dir_all(out);
    copy_dir(dir, out);
    let written = contents(&open(out));
    format!(
        "written {}: {} records, digest {:016x}",
        out.display(),
        written.records,
        written.digest
    )
}

#[derive(Default)]
struct Times {
    open: Duration,
    iterate: Duration,
    iterate_owned: Duration,
    iterate_codec: Duration,
    codec: Duration,
    /// One `ChunkRecords::read`, averaged over up to 200 chunks spread over the world.
    records_read: Duration,
    /// Per thread count: reading every chunk, and reading every chunk with its surface.
    threads: Vec<(Duration, Duration)>,
}

/// One run on a fresh copy of the world. The first run also returns what it found.
fn run(args: &Args, index: usize) -> (Times, Option<String>) {
    let dir = fresh_copy(&args.world, index);
    let mut times = Times::default();

    let db;
    (db, times.open) = timed(|| open(&dir));
    times.iterate_owned = timed(|| iterate_owned(&db)).1;
    times.iterate = timed(|| iterate_in_place(&db)).1;
    let contents = contents(&db);
    let codec;
    (codec, times.iterate_codec) = timed(|| iterate_and_reencode(&db));
    let in_memory;
    (in_memory, times.codec) = timed(|| reencode_in_memory(&contents.codec_records));
    assert_eq!(codec, in_memory);
    times.records_read = records_read(&db, &contents.chunks);
    for &threads in &args.threads {
        let raw = timed(|| read_chunks(&db, &contents.chunks, threads, false)).1;
        let surface = timed(|| read_chunks(&db, &contents.chunks, threads, true)).1;
        times.threads.push((raw, surface));
    }

    let mut report = None;
    if index == 0 {
        let mut text = format!(
            "records {}, bytes {}, chunks {}, digest {:016x}\n\
             codec: {} sub chunks, {} Data3D, {} failed, {} byte-identical, {} bytes out",
            contents.records,
            contents.bytes,
            contents.chunks.len(),
            contents.digest,
            codec.subchunks,
            codec.data3d,
            codec.failed,
            codec.identical,
            codec.out_bytes
        );
        if args.verify {
            write!(text, "\n{}", verify(&contents.codec_records)).unwrap();
        }
        if let Some(out) = &args.write {
            let written = write_reencoded(db, &dir, &contents.codec_records, out);
            write!(text, "\n{written}").unwrap();
        }
        report = Some((text, contents.chunks.len()));
    }
    let _ = std::fs::remove_dir_all(&dir);
    (
        times,
        report.map(|(text, chunks)| format!("{text}\nchunks:{chunks}")),
    )
}

/// The time of one `ChunkRecords::read`, the way `Level::chunk` reads a chunk, averaged
/// over up to 200 chunks spread over the world.
fn records_read(db: &Database, chunks: &[(i32, i32, i32)]) -> Duration {
    let step = (chunks.len() / 200).max(1);
    let picked: Vec<_> = chunks.iter().step_by(step).take(200).collect();
    let ((), total) = timed(|| {
        for &&(dim, x, z) in &picked {
            ChunkRecords::read(db, ChunkPosition(x, z), Dimension::from(dim))
                .unwrap()
                .expect("a listed chunk has records");
        }
    });
    total / u32::try_from(picked.len().max(1)).unwrap()
}

fn median(mut times: Vec<Duration>) -> Duration {
    times.sort();
    times[times.len() / 2]
}

fn main() {
    let args = args();
    let mut runs = Vec::new();
    let mut report = String::new();
    for index in 0..args.runs {
        let (times, text) = run(&args, index);
        runs.push(times);
        report = text.unwrap_or(report);
    }

    let (text, chunks) = report.rsplit_once("\nchunks:").unwrap();
    let chunks: f64 = chunks.parse().unwrap();
    println!("{text}");
    let col = |f: &dyn Fn(&Times) -> Duration| median(runs.iter().map(f).collect());
    let ms = |f: &dyn Fn(&Times) -> Duration| col(f).as_secs_f64() * 1000.0;
    println!("open                        {:8.1} ms", ms(&|t| t.open));
    println!("iterate all records         {:8.1} ms", ms(&|t| t.iterate));
    println!(
        "  through Iterator (owned)  {:8.1} ms",
        ms(&|t| t.iterate_owned)
    );
    println!(
        "iterate + decode/encode     {:8.1} ms",
        ms(&|t| t.iterate_codec)
    );
    println!("decode/encode in memory     {:8.1} ms", ms(&|t| t.codec));
    println!(
        "ChunkRecords::read, each    {:8.3} ms",
        ms(&|t| t.records_read)
    );
    for (i, threads) in args.threads.iter().enumerate() {
        let raw = chunks / col(&|t| t.threads[i].0).as_secs_f64();
        let surface = chunks / col(&|t| t.threads[i].1).as_secs_f64();
        println!("{threads:2} threads: read {raw:8.0} chunks/s, surface {surface:8.0} chunks/s");
    }
}
