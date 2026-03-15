use std::fs::File;

use bedrockrs_level::{
    Packed, Unpacked,
    bits::BitArray,
    db::Database,
    key::{Key, KeyVariant},
    packed::PackedArray,
    subchunk::SubChunk,
    unpacked::UnpackedArray,
};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use flate2::read::GzDecoder;
use tar::Archive;

fn unpack_regular(bits: u32, packed: &PackedArray) {
    let mut indices = [0; 4096];
    UnpackedArray::unpack_nonsimd(bits as u8, packed.words(), &mut indices);
}

fn unpack_oct(bits: u32, packed: &PackedArray) {
    let mut indices = [0; 4096];
    match bits {
        1 => unsafe {
            UnpackedArray::unpack_oct::<1>(packed.words(), &mut indices);
        },
        2 => unsafe {
            UnpackedArray::unpack_oct::<2>(packed.words(), &mut indices);
        },
        4 => unsafe {
            UnpackedArray::unpack_oct::<4>(packed.words(), &mut indices);
        },
        8 => unsafe {
            UnpackedArray::unpack_oct::<8>(packed.words(), &mut indices);
        },
        _ => unimplemented!(),
    }
}

// fn unpack_oct_interleaved(bits: u32, packed: &PackedArray) {
//     let mut indices = [0; 4096];
//     match bits {
//         1 => unsafe { UnpackedArray::unpack_oct_interleaved::<1>(packed.words(), &mut indices); },
//         2 => unsafe { UnpackedArray::unpack_oct_interleaved::<2>(packed.words(), &mut indices); },
//         4 => unsafe { UnpackedArray::unpack_oct_interleaved::<4>(packed.words(), &mut indices); },
//         _ => unimplemented!()
//     }
// }

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
        0b11111111000000000000000000000000,
    ];

    words.resize(4096, 0b11111111000000000000000000000000);

    let array = PackedArray::new(1, words);

    let mut group = c.benchmark_group("unpack");
    for bits in [1, 2, 4, 8] {
        group.throughput(Throughput::ElementsAndBytes {
            bytes: 4096 / (32 / bits as u64),
            elements: 4096,
        });
        group.bench_with_input(BenchmarkId::new("nonvectorized", bits), &array, |b, i| {
            b.iter(|| unpack_regular(bits, i))
        });
        group.bench_with_input(
            BenchmarkId::new("vectorized_simd256", bits),
            &array,
            |b, i| b.iter(|| unpack_oct(bits, i)),
        );
        // group.bench_with_input(
        //     BenchmarkId::new("vectorized_simd256_interleaved", bits),
        //     &array,
        //     |b, i| b.iter(|| unpack_oct_interleaved(bits, i))
        // );
    }
    group.finish();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
