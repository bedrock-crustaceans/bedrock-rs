use std::fs::File;

use bedrockrs_level::{db::Database, key::{Key, KeyVariant}, subchunk::SubChunk};
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
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

fn paletted_benchmark(data: &[u8]) {
    let _chunk = SubChunk::deserialize_disk(data).unwrap();
}

fn unpacked_benchmark(data: &[u8]) {
    let _chunk = SubChunk::deserialize_disk(data).unwrap();
}

fn benchmark(c: &mut Criterion) {
    let dir = extract_test_db();
    let tmp = extract_test_db();
    let tmp_path = tmp.path().join("test_level/db");
    let tmp_path = tmp_path.to_str().unwrap();

    let database = Database::open(tmp_path).unwrap();
    let mut keys = database.iter();

    // Find all subchunks in test DB
    let chunks = keys.filter_map(|kv| {
        let key = Key::deserialize(kv.key()).ok()?;
        let data = Vec::from(kv.value());

        if let KeyVariant::SubChunk { index } = key.data {
            Some((Vec3::new(key.chunk.x, index as i32, key.chunk.y), data))
        } else {
            None
        }
    }).take(3).collect::<Vec<_>>();

    let mut group = c.benchmark_group("unpacked_benchmark");
    for (key, chunk) in &chunks {
        group.throughput(criterion::Throughput::Bytes(chunk.len() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(key), 
        chunk,
        |b, chunk| {
            b.iter(|| unpacked_benchmark(chunk))
        });
    }
}

criterion_group!(benches, benchmark);
criterion_main!(benches);