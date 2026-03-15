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
    let mut indices = Box::new([0; 4096]);
    UnpackedArray::unpack_nonsimd(packed.bits() as u8, packed.words(), indices.as_mut());

    println!("regular indices: {indices:?}");
}

fn unpack_vectorized(packed: &PackedArray) {
    let mut indices = Box::new([0; 4096]);
    unsafe { UnpackedArray::unpack_avx(packed.bits() as u8, packed.words(), indices.as_mut()); }

    println!("regular indices: {indices:?}");
}

fn benchmark(c: &mut Criterion) {
    let tmp = extract_test_db();
    let tmp_path = tmp.path().join("test_level/db");
    let tmp_path = tmp_path.to_str().unwrap();

    let database = Database::open(tmp_path).unwrap();
    let mut keys = database.keys();

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

    let mut group = c.benchmark_group("packed_array");
    for (key, chunk) in &chunks {
        let subchunk = SubChunk::from_disk::<Packed, _>(chunk.as_slice()).unwrap();
        let layer = subchunk.layer(0);
        let array = layer.indices();
        let BitArray::Packed(array) = array else { unreachable!() };
        
        unsafe { unpack_vectorized(array) };
        unpack_regular(array);

        // group.throughput(Throughput::ElementsAndBytes {
        //     bytes: 4 * array.words_count() as u64,
        //     elements: 4096
        // });

        // group.bench_with_input(
        //     BenchmarkId::new("unpack_regular", key),
        //     array,
        //     |b, i| b.iter(|| unpack_regular(i))
        // );

        // assert!(is_x86_feature_detected!("avx2"), "this benchmark requires AVX2 support");

        // group.bench_with_input(
        //     BenchmarkId::new("unpack_vectorized", key),
        //     array,
        //     |b, i| b.iter(|| unpack_vectorized(i))
        // );
    }
    group.finish();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
