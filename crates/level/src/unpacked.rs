use crate::{error::Result, packed::PackedArray};
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::Read;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnpackedArray {
    array: Box<[u16; 4096]>,
}

impl UnpackedArray {
    pub fn get(&self, index: usize) -> Option<u16> {
        self.array.get(index).copied()
    }

    pub fn set(&mut self, index: usize, value: u16) -> bool {
        if let Some(index) = self.array.get_mut(index) {
            *index = value;
            true
        } else {
            false
        }
    }

    pub fn to_disk(&self, writer: &mut Vec<u8>, bits: u32) -> Result<()> {
        // Amount of indices that fit in a single 32-bit integer.
        let per_word = u32::BITS / bits;

        let mut offset = 0;
        while offset < 4096 {
            let mut word = 0;
            for w in 0..per_word {
                let index = self.array[offset] as u32;
                word |= index << (w * bits);

                offset += 1;
            }

            writer.write_u32::<LittleEndian>(word)?;
        }

        Ok(())
    }

    pub(crate) fn from_disk<R: Read>(mut reader: R, bits: u8) -> Result<Self> {
        let per_word = u32::BITS / bits as u32;
        let word_count = 4096u32.div_ceil(per_word);

        let mut words = vec![0; word_count as usize];
        reader.read_exact(bytemuck::cast_slice_mut::<u32, u8>(&mut words))?;

        Ok(Self::unpack(&words, bits))
    }

    #[inline]
    pub(crate) fn unpack(words: &[u32], bits: u8) -> Self {
        let mut array = Box::new([0u16; 4096]);

        if is_x86_feature_detected!("avx2") {
            unsafe { Self::unpack_avx(bits, words, &mut array) };
        } else {
            Self::unpack_nonsimd(bits, words, array.as_mut());
        }

        Self { array }
    }

    /// # Safety
    ///
    ///
    /// This function must only be called if the host CPU supports the `avx2` feature. Calling this function
    /// otherwise will result in an abort due to illegal instructions.
    ///
    /// # Panics
    ///
    /// This function panics if `words` is too small for `bits`. In other words, `words.len() >= 4096 / (32 / bits)`.
    /// Furthermore, `BITS` must be valid, i.e. it is either 1, 2, 3, 4, 5, 6, 8 or 16.
    #[target_feature(enable = "avx2")]
    pub fn unpack_avx(bits: u8, words: &[u32], indices: &mut [u16; 4096]) {
        // Bits are compile-time constants using generics to enable more bitsize-specific optimisations.
        match bits {
            1 => Self::unpack_oct::<1>(words, indices),
            2 => Self::unpack_oct::<2>(words, indices),
            3 => todo!(),
            4 => Self::unpack_oct::<4>(words, indices),
            5 => todo!(),
            6 => todo!(),
            8 => Self::unpack_oct::<8>(words, indices),
            16 => Self::unpack_oct::<16>(words, indices),
            _ => unimplemented!(),
        }
    }

    // TODO: This function is even faster but the output format makes it difficult to use.
    // #[inline]
    // #[target_feature(enable = "avx2")]
    // pub unsafe fn unpack_oct_interleaved<const BITS: i32>(mut words: &[u32], indices: &mut [u16; 4096]) {
    //     use std::arch::x86_64::{
    //         __m256i, _mm256_loadu_si256, _mm256_set1_epi32, _mm256_srlv_epi32,
    //         _mm256_and_si256, _mm256_packus_epi32, _mm256_permutex_epi64,
    //         _mm256_castsi256_si128, _mm256_srli_epi32
    //     };
    //
    //     const SIMD_LANES: u32 = 8;
    //
    //     let blocks_per_word = u32::BITS / BITS as u32;
    //     let word_count = 4096 / blocks_per_word;
    //     words = &words[..word_count as usize];
    //
    //     let vmask = _mm256_set1_epi32(!(!0u32 << BITS as u32) as i32);
    //     let vshift = _mm256_set1_epi32(BITS as i32);
    //
    //     let mut offset = 0;
    //     let (chunks, rem) = words.as_chunks::<8>();
    //     for chunk in chunks {
    //         let mut vwords = unsafe {
    //             _mm256_loadu_si256(chunk.as_ptr().cast::<__m256i>())
    //         };
    //
    //         for _ in 0..blocks_per_word {
    //             let vmasked = _mm256_and_si256(vwords, vmask);
    //             let vpacked = _mm256_packus_epi32(vmasked, vmasked);
    //             let vperm = unsafe { _mm256_permutex_epi64::<0b11011000>(vpacked) };
    //
    //             let vtrunc = _mm256_castsi256_si128(vperm);
    //             debug_assert!(offset <= 4088);
    //             unsafe {
    //                 std::ptr::copy_nonoverlapping(
    //                     (&raw const vtrunc).cast::<u16>(),
    //                     indices.as_mut_ptr().add(offset),
    //                     8
    //                 );
    //             }
    //
    //             offset += 8;
    //
    //             vwords = _mm256_srli_epi32::<BITS>(vwords);
    //         }
    //     }
    // }

    /// # Safety
    ///
    ///
    /// This function must only be called if the host CPU supports the `avx2` feature. Calling this function
    /// otherwise will result in an abort due to illegal instructions.
    ///
    /// # Panics
    ///
    /// This function panics if `words` is too small for `BITS`. In other words, `words.len() >= 4096 / (32 / BITS)`.
    /// Furthermore, `BITS` must be valid, i.e. it is either 1, 2, 4 or 8.
    #[inline]
    #[target_feature(enable = "avx2")]
    pub fn unpack_oct<const BITS: u8>(mut words: &[u32], indices: &mut [u16; 4096]) {
        use std::arch::x86_64::{
            __m256i, _mm_loadu_epi32, _mm_set_epi32, _mm_set1_epi32, _mm_storeu_epi16,
            _mm256_and_si256, _mm256_packus_epi32, _mm256_permutex_epi64, _mm256_set_epi32,
            _mm256_set_m128i, _mm256_set1_epi32, _mm256_srl_epi32, _mm256_srlv_epi32,
        };

        const SIMD_LANES: u32 = 8;

        let blocks_per_word = u32::BITS / BITS as u32;
        let sets_per_word = blocks_per_word / SIMD_LANES;

        // Limit words to max size for these bits
        let max_len = 4096 / blocks_per_word;
        assert!(words.len() >= max_len as usize);
        words = &words[..max_len as usize];

        match BITS {
            1 | 2 | 4 => {
                let bits = BITS as i32;

                let vmask = _mm256_set1_epi32(!(!0u32 << bits) as i32);

                // Shifts each of the 8 lanes to their respective location in the word
                // This is in reverse because the first argument is for the last line, etc.
                let vshift = _mm256_set_epi32(
                    7 * bits,
                    6 * bits,
                    5 * bits,
                    4 * bits,
                    3 * bits,
                    2 * bits,
                    bits,
                    0,
                );

                // Shifts all lanes to their next location in the word.
                let vshiftall = _mm_set1_epi32(8 * bits);

                let mut w = 0;
                let mut offset = 0;

                // Decode 32 blocks per word.
                // We do this in 4 sets of 8 for BITS = 1
                // or 2 sets of 8 for BITS = 2
                // or 1 set of 8 for BITS = 4
                while w < words.len() {
                    let mut vword = _mm256_set1_epi32(words[w] as i32);

                    // Use variable shifts since each lane needs to be shifted by a different amount.
                    vword = _mm256_srlv_epi32(vword, vshift);

                    let mut s = 0;
                    while s < sets_per_word - 1 {
                        // Applies the mask to extract the bits
                        let vmasked = _mm256_and_si256(vword, vmask);
                        let vpack = _mm256_packus_epi32(vmasked, vmasked);
                        // Fix the ordering of the lanes.
                        let vshorts = unsafe { _mm256_permutex_epi64(vpack, 0b11011000) };

                        debug_assert!(offset <= 4080);
                        unsafe {
                            std::ptr::copy_nonoverlapping(
                                &vshorts as *const __m256i as *const u16,
                                indices.as_mut_ptr().add(offset),
                                8,
                            );
                        }

                        // This uses an `srl` intrinsics instead of an `srlv` one because
                        // `vshiftall` is the same for all lanes. This gives a 24% performance
                        // boost for BITS = 1 and 15% for BITS = 2.
                        vword = _mm256_srl_epi32(vword, vshiftall);

                        offset += 8;
                        s += 1;
                    }

                    // Last set does not need a shift all at the end
                    let vmasked = _mm256_and_si256(vword, vmask);
                    let vpack = _mm256_packus_epi32(vmasked, vmasked);
                    let vshorts = unsafe { _mm256_permutex_epi64::<0b11011000>(vpack) };

                    debug_assert!(offset <= 4088);
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            &vshorts as *const __m256i as *const u16,
                            indices.as_mut_ptr().add(offset),
                            8,
                        );
                    }

                    offset += 8;
                    w += 1;
                }
            }
            8 => {
                // When BITS = 8 we load two words per iteration. Since each word has 4 blocks,
                // we put the first word in the lower 4 lanes and the second word in the upper 4 lanes.
                // Then we perform the regular unpack algorithm on both words at the same time, unpacking
                // 8 blocks from 64 bits in a single operation.

                let vmask = _mm256_set1_epi32(!(!0u32 << BITS as u32) as i32);

                let bits = BITS as i32;
                // Create a 128-bit vector with the correct shifts...
                let vshift_half = _mm_set_epi32(3 * bits, 2 * bits, bits, 0);
                // ... and then make another copy of
                let vshift = _mm256_set_m128i(vshift_half, vshift_half);

                let mut offset = 0;

                // 4 blocks per word so we load two words at once
                // Safety: This is safe because 4096 / (32 / 8) = 128 is divisible by 2
                // so we have no remainder.
                let chunks = unsafe { words.as_chunks_unchecked::<2>() };
                for chunk in chunks {
                    let [a, b] = chunk;

                    // Broadcast the first word into 4 lanes
                    let vlo = _mm_set1_epi32(*a as i32);
                    // Broadcast the second word into 4 lanes
                    let vhi = _mm_set1_epi32(*b as i32);

                    // Then combine these two into one larger vector
                    let vwords = _mm256_set_m128i(vhi, vlo);

                    let vshifted = _mm256_srlv_epi32(vwords, vshift);
                    let vmasked = _mm256_and_si256(vshifted, vmask);

                    let vpack = _mm256_packus_epi32(vmasked, vmasked);
                    let vperm = unsafe { _mm256_permutex_epi64::<0b11011000>(vpack) };

                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            &vperm as *const __m256i as *const u16,
                            indices.as_mut_ptr().add(offset),
                            8,
                        );
                    }

                    offset += 8;
                }
            }
            16 => {
                // When `BITS = 16` we load 4 words and them immediately store them into the indices array.
                // This is done using SIMD because it can load and store multiple words at the same time.

                // Safety: The length of `words` is explicitly set to 4096 / (32 / 16) = 2048 above,
                // which is divisible by 4.
                let chunks = unsafe { words.as_chunks_unchecked::<4>() };

                let mut offset = 0;
                for chunk in chunks {
                    // Safety: This is safe because `chunk` is valid and always has 4 words of data.
                    // This is an unaligned load so we do not care about alignment.
                    let vwords = unsafe { _mm_loadu_epi32(chunk.as_ptr().cast::<i32>()) };

                    // Then immediately write to the output array because the data is already in the correct other.

                    // Safety: This is safe because `chunks` contains the exact amount of chunks to reach 4096 shorts
                    // and does not leave the allocation.
                    let arr_ptr = unsafe { indices.as_mut_ptr().add(offset).cast::<i16>() };

                    // Safety: By the evaluation above we know that `arr_ptr` points to a valid array with enough
                    // range lef to write into. This is an unaligned store so we do not core about alignment.
                    unsafe { _mm_storeu_epi16(arr_ptr, vwords) };

                    offset += 8;
                }
            }
            _ => unimplemented!("invalid BITS generic for `unpack_oct`"),
        }
    }

    #[inline]
    pub fn unpack_nonsimd(bits: u8, mut words: &[u32], indices: &mut [u16]) {
        let per_word = u32::BITS / bits as u32;
        let max_len = 4096 / per_word;
        words = &words[..max_len as usize];

        let mask = !(!0u32 << bits);

        let mut offset = 0;
        for mut word in words.iter().copied() {
            for _ in 0..per_word {
                if offset == 4096 {
                    break;
                }

                indices[offset] = (word & mask) as u16;
                word >>= bits;
                offset += 1;
            }
        }
    }
}

impl<'a> IntoIterator for &'a UnpackedArray {
    type Item = u16;
    type IntoIter = std::iter::Copied<std::slice::Iter<'a, u16>>;

    fn into_iter(self) -> Self::IntoIter {
        self.array.iter().copied()
    }
}

impl From<&PackedArray> for UnpackedArray {
    fn from(array: &PackedArray) -> UnpackedArray {
        let words = array.words();
        UnpackedArray::unpack(words, array.bits() as u8)
    }
}
