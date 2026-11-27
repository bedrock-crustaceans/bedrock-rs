use bedrock_level::{greedy::GreedyArray, lazy::LazyArray};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};

fn unpack_regular(bits: u32, packed: &LazyArray) {
    let mut indices = [0; 4096];
    #[expect(clippy::cast_possible_truncation, reason = "bits is 1, 2, 4 or 8")]
    let bits = bits as u8;
    GreedyArray::unpack_nonsimd(bits, packed.words(), &mut indices);
}

#[cfg(target_arch = "x86_64")]
fn unpack_oct(bits: u32, packed: &LazyArray) {
    let mut indices = [0; 4096];
    match bits {
        1 => unsafe {
            GreedyArray::unpack_oct::<1>(packed.words(), &mut indices);
        },
        2 => unsafe {
            GreedyArray::unpack_oct::<2>(packed.words(), &mut indices);
        },
        4 => unsafe {
            GreedyArray::unpack_oct::<4>(packed.words(), &mut indices);
        },
        8 => unsafe {
            GreedyArray::unpack_oct::<8>(packed.words(), &mut indices);
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
        0b1010_1010_1010_1010_1010_1010_1010_1010,
        0b1000_0000_0000_0000_0000_0000_0000_0000,
        0b1100_0000_0000_0000_0000_0000_0000_0000,
        0b1110_0000_0000_0000_0000_0000_0000_0000,
        0b1111_0000_0000_0000_0000_0000_0000_0000,
        0b1111_1000_0000_0000_0000_0000_0000_0000,
        0b1111_1100_0000_0000_0000_0000_0000_0000,
        0b1111_1110_0000_0000_0000_0000_0000_0000,
        0b1111_1111_0000_0000_0000_0000_0000_0000,
    ];

    words.resize(4096, 0b1111_1111_0000_0000_0000_0000_0000_0000);

    let array = LazyArray::new(1, words);

    let mut group = c.benchmark_group("unpack");
    for bits in [1, 2, 4, 8] {
        group.throughput(Throughput::ElementsAndBytes {
            bytes: 4096 / (32 / u64::from(bits)),
            elements: 4096,
        });
        group.bench_with_input(BenchmarkId::new("nonvectorized", bits), &array, |b, i| {
            b.iter(|| unpack_regular(bits, i));
        });
        #[cfg(target_arch = "x86_64")]
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
