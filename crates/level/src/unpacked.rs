use crate::{error::Result, packed::PackedArray};
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::{Read, Write};

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
            unsafe { Self::unpack_avx(bits, &words, &mut array) };
        } else {
            Self::unpack_nonsimd(bits, &words, array.as_mut());
        }

        Self { array }
    }

    #[target_feature(enable = "avx2")]
    pub fn unpack_avx(bits: u8, words: &[u32], indices: &mut [u16; 4096]) {
        use std::arch::x86_64::{
            __m256i, __m128i, _mm256_loadu_si256, _mm256_set1_epi32, _mm256_and_si256, 
            _mm256_srl_epi32, _mm_cvtsi32_si128, _mm256_packus_epi32, _mm256_permute4x64_epi64, 
            _mm_storeu_si128, _mm256_castsi256_si128
        };

        match bits {
            1 => Self::unpack_oct::<1>(words, indices),
            2 => Self::unpack_oct::<2>(words, indices),
            _ => todo!()
        }
    }

    #[inline]
    #[target_feature(enable = "avx2")]
    pub fn unpack_oct<const BITS: u8>(words: &[u32], indices: &mut [u16; 4096]) {
        use std::arch::x86_64::{
            __m256i, _mm256_set_epi32, _mm256_set1_epi32, _mm256_srlv_epi32,
            _mm256_and_si256, _mm256_packus_epi32, _mm256_permutex_epi64,
            _mm
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

                        unsafe {
                            std::ptr::copy_nonoverlapping(
                                &vshorts as *const __m256i as *const u16,
                                indices.as_mut_ptr().add(offset),
                                8
                            );
                        }

                        i += 8;
                        vword = _mm256_srlv_epi32(vword, vshiftall);
                    }

                    // Last set does not need a shift all at the end
                    let vmasked = _mm256_and_si256(vword, vmask);
                    let vpack = _mm256_packus_epi32(vmasked, vmasked);
                    let vshorts = unsafe { _mm256_permutex_epi64(vpack, 0b11011000) };
                    let offset = (w + 1) * blocks_per_word as usize - 8;

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

    // #[inline]
    // #[target_feature(enable = "avx2")]
    // fn unpack_eight(bits: u8, words: &[u32], indices: &mut [u16; 4096]) {
    //     const SIMD_LANES: u32 = 8;

    //     // Use specialised decoding functions for specific bit sizes.

    //     Bits	BPB	    SIMD lanes
    //     1	    32	    8		
    //     2	    16	    8		
    //     3 	    10	    8		
    //     4	    8	    8		
    //     5	    6	
    //     6	    5	
    //     8	    4	    4		
    //     16	    2	    None		
    // }

    /// This function should only be called when the amount of blocks per word is greater than or equal to 4
    /// and less than 8 (or in Rust range notation 4..8), otherwise it will give incorrect results.
    #[inline]
    #[target_feature(enable = "avx2")]
    fn unpack_quad(bits: u8, words: &[u32], indices: &mut [u16; 4096]) {
        use std::arch::x86_64::{
            __m128i, _mm_set_epi32, _mm_set1_epi32, _mm_srl_epi32, _mm_and_si128
        };

        const SIMD_LANES: u32 = 4;

        let per_word = u32::BITS / bits as u32;
        let simd_iters = per_word / SIMD_LANES;
        let rem_iters = per_word - SIMD_LANES * simd_iters;

        let mask = !(!0u32 << bits);

        let vmask = _mm_set1_epi32(mask as i32);
        let vshift = _mm_set_epi32(
            0,
            bits as i32,
            2 * bits as i32,
            3 * bits as i32
        );

        for word in words.iter().copied() {
            let vword = _mm_set1_epi32(word as i32);
            let vbits = _mm_srl_epi32(vword, vshift);
            let vmasked = _mm_and_si128(vbits, vmask);

            for _ in 0..rem_iters {
                
            }
        }
    }

    #[inline]
    pub fn unpack_nonsimd(bits: u8, words: &[u32], indices: &mut [u16]) {
        let per_word = u32::BITS / bits as u32;
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