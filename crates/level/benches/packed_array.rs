use std::fs::File;

use bedrockrs_level::{
    Packed, Unpacked, bits::BitArray, db::Database, key::{Key, KeyVariant}, packed::PackedArray, subchunk::SubChunk, unpacked::UnpackedArray
};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
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

fn unpack_regular(packed: &PackedArray) {
    let mut indices = [0; 4096];
    UnpackedArray::unpack_nonsimd(packed.bits() as u8, packed.words(), &mut indices);
}

fn unpack_oct1(packed: &PackedArray) {
    let mut indices = [0; 4096];
    unsafe { UnpackedArray::unpack_oct::<1>(packed.words(), &mut indices); }
}

fn benchmark(c: &mut Criterion) {
    // Generate some fake data with a recognisable pattern.
    let mut words = vec![
        0b10101010101010101010101010101010,
        0b10000000000000000000000000000000,
        0b11000000000000000000000000000000,
        0b11100000000000000000000000000000,
        0b11110000000000000000000000000000,
        0b11111000000000000000000000000000,
        0b11111100000000000000000000000000,
        0b11111110000000000000000000000000,
        0b11111111000000000000000000000000
    ];

    words.resize(128, 0b11111111000000000000000000000000);

    let array = PackedArray::new(
        1, words
    );

    let mut group = c.benchmark_group("unpack");
    group.throughput(Throughput::ElementsAndBytes {
        bytes: array.word_count() as u64,
        elements: 4096
    });
    group.bench_with_input(
        BenchmarkId::new("nonvectorized", array.word_count()),
        &array,
        |b, i| b.iter(|| unpack_regular(i))
    );
    group.bench_with_input(
        BenchmarkId::new("vectorized_oct1bit", array.word_count()),
        &array,
        |b, i| b.iter(|| unpack_oct1(i))
    );
    group.finish();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
