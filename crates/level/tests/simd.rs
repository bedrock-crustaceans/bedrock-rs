use bedrockrs_level::unpacked::UnpackedArray;

#[inline]
#[target_feature(enable = "avx2")]
pub fn unpack_oct<const BITS: u8>(words: &[u32], indices: &mut [u16; 4096]) {
    use std::arch::x86_64::{
        __m256i, _mm256_loadu_si256, _mm256_set_epi32, _mm256_set1_epi32, _mm256_srlv_epi32,
        _mm256_and_si256, _mm256_packus_epi32, _mm256_permutex_epi64
    };

    let blocks_per_word = u32::BITS / BITS as u32;

    let mut w = 0;
    let mut i = 0;

    match BITS {
        1 => {
            let bits = BITS as i32;

            let vmask = _mm256_set1_epi32(!(!0u32 << bits) as i32);

            // Shifts each of the 8 lanes to their respective location in the word
            let vshift = _mm256_set_epi32(
                0,
                bits,
                2 * bits,
                3 * bits,
                4 * bits,
                5 * bits,
                6 * bits,
                7 * bits
            );

            // Shifts all lanes to their next location in the word.
            let vshiftall = _mm256_set1_epi32(8 * bits);

            // Decode 32 blocks per word. 
            // We do this in 4 sets of 8
            while w < words.len() {
                let mut vword = _mm256_set1_epi32(words[w] as i32);
                vword = _mm256_srlv_epi32(vword, vshift);

                while i + 8 < blocks_per_word as usize {
                    // println!("at set {i}");

                    let vmasked = _mm256_and_si256(vword, vmask);
                    let vpack = _mm256_packus_epi32(vmasked, vmasked);
                    let vshorts = unsafe { _mm256_permutex_epi64(vpack, 0b11011000) };
                    let offset = w * blocks_per_word as usize + i;
                    println!("offset: {offset}");

                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            &vshorts as *const __m256i as *const u16,
                            indices.as_mut_ptr().add(offset),
                            8
                        );
                    }

                    // // BPW 32
                    // 0-8, 8-16, 16-24

                    i += 8;
                    vword = _mm256_srlv_epi32(vword, vshiftall);
                }

                // Last set does not need a shift all at the end
                let vmasked = _mm256_and_si256(vword, vmask);
                let vpack = _mm256_packus_epi32(vmasked, vmasked);
                let vshorts = unsafe { _mm256_permutex_epi64(vpack, 0b11011000) };
                let offset = (w + 1) * blocks_per_word as usize - 8;
                println!("end offset: {offset}");

                unsafe {
                    std::ptr::copy_nonoverlapping(
                        &vshorts as *const __m256i as *const u16,
                        indices.as_mut_ptr().add(offset),
                        8
                    );
                }

                i = 0;
                w += 1;
            }
        },
        _ => unreachable!("invalid BITS generic for `unpack_oct`")
    }
}

#[test]
fn simd_test() {
    const BITS: u32 = 1;
    const BLOCKS_PER_WORD: u32 = u32::BITS / BITS;

    // let mut words: Vec<u32> = vec![0b10101010101010101010101010101010; 4096 / BLOCKS_PER_WORD as usize];
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

    // let mut words = vec![];
    words.resize(128, 0b11111111000000000000000000000000);

    let mut indices_simd = Box::new([0; 4096]);
    let mut indices_regular = Box::new([0; 4096]);

    unsafe {
        UnpackedArray::unpack_oct::<1>(&words, &mut indices_simd);
    }

    UnpackedArray::unpack_nonsimd(1, &words, indices_regular.as_mut_slice());
    assert_eq!(indices_simd, indices_regular);
}