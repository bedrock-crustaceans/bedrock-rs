use std::fs::File;

use bedrockrs_level::{
    db::Database,
    key::{Key, KeyVariant},
    subchunk::{Greedy, Lazy, SubChunk},
};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use flate2::read::GzDecoder;
use tar::Archive;
use vek::Vec3;

fn extract_test_db() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("Failed to create temp dir");

    let tar_gz = File::open("tests/level.tar.gz").expect("Seed missing");
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);

    archive.unpack(tmp.path()).expect("Failed to unpack seed");

    tmp
}

fn lazy_load_benchmark(data: &[u8]) {
    let _chunk = SubChunk::deserialize_from_disk::<Lazy, _>(data).unwrap();
}

fn lazy_iter_benchmark(data: &SubChunk) {
    let iter = data.layer(0).iter();
    for block in iter {
        let name = &block.name;
        std::hint::black_box(name);
    }
}

fn greedy_load_benchmark(data: &[u8]) {
    let _chunk = SubChunk::deserialize_from_disk::<Greedy, _>(data).unwrap();
}

fn greedy_iter_benchmark(data: &SubChunk) {
    let iter = data.layer(0).iter();
    for block in iter {
        let name = &block.name;
        std::hint::black_box(name);
    }
}

fn benchmark(c: &mut Criterion) {
    let tmp = extract_test_db();
    let tmp_path = tmp.path().join("test_level/db");
    let tmp_path = tmp_path.to_str().unwrap();

    let database = Database::open(tmp_path).unwrap();
    let mut keys = database.iter();

    // Find some usable subchunks.
    let chunks = keys
        .filter_map(|kv| {
            let key = Key::deserialize(kv.key()).ok()?;
            let data = Vec::from(kv.value());

            if let KeyVariant::SubChunk { index } = key.data {
                Some((Vec3::new(key.chunk.x, index as i32, key.chunk.y), data))
            } else {
                None
            }
        })
        .take(1)
        .collect::<Vec<_>>();

    let mut group1 = c.benchmark_group("deserialize_benches");
    for (key, chunk) in &chunks {
        group1.throughput(criterion::Throughput::Bytes(chunk.len() as u64));
        group1.bench_with_input(
            BenchmarkId::from_parameter(format!("lazy {key}")),
            chunk,
            |b, chunk| b.iter(|| lazy_load_benchmark(chunk)),
        );
        group1.bench_with_input(
            BenchmarkId::from_parameter(format!("greedy {key}")),
            chunk,
            |b, chunk| b.iter(|| greedy_load_benchmark(chunk)),
        );
    }
    group1.finish();

    let mut group2 = c.benchmark_group("iter_benches");
    for (key, chunk) in &chunks {
        let slice = chunk.as_slice();
        let greedy_chunk = SubChunk::deserialize_from_disk::<Greedy, _>(slice).unwrap();

        let lazy_chunk = SubChunk::deserialize_from_disk::<Lazy, _>(slice).unwrap();

        group2.throughput(criterion::Throughput::Elements(4096));
        group2.bench_with_input(
            BenchmarkId::from_parameter(format!("greedy {key}")),
            &greedy_chunk,
            |b, chunk| b.iter(|| greedy_iter_benchmark(chunk)),
        );
        group2.bench_with_input(
            BenchmarkId::from_parameter(format!("lazy {key}")),
            &lazy_chunk,
            |b, chunk| b.iter(|| lazy_iter_benchmark(chunk)),
        );
    }
    group2.finish();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
