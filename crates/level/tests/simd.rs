#![cfg(target_arch = "x86_64")]

use bedrock_level::greedy::GreedyArray;

/// Words with every bit pattern spread around: a fixed xorshift sequence.
fn spread_words() -> Vec<u32> {
    let mut state = 0x9E37_79B9u32;
    (0..4096)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            state
        })
        .collect()
}

#[test]
fn avx_unpack_matches_nonsimd_for_every_width() {
    if !is_x86_feature_detected!("avx2") {
        eprintln!("skipped: this CPU has no AVX2");
        return;
    }

    for words in [spread_words(), vec![u32::MAX; 4096], vec![0; 4096]] {
        for bits in [1, 2, 3, 4, 5, 6, 8, 16] {
            let mut indices_simd = Box::new([0; 4096]);
            let mut indices_regular = Box::new([0; 4096]);

            // Safety: AVX2 was detected above.
            unsafe { GreedyArray::unpack_avx(bits, &words, &mut indices_simd) };
            GreedyArray::unpack_nonsimd(bits, &words, indices_regular.as_mut_slice());

            assert_eq!(
                indices_simd, indices_regular,
                "the AVX2 unpacker differs from unpack_nonsimd at {bits} bits"
            );
        }
    }
}
