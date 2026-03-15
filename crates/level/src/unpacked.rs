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
    pub fn unpack_oct<const BITS: u8>(mut words: &[u32], indices: &mut [u16; 4096]) {
        use std::arch::x86_64::{
            __m256i, _mm256_loadu_si256, _mm256_set_epi32, _mm256_set1_epi32, _mm256_srlv_epi32,
            _mm256_and_si256, _mm256_packus_epi32, _mm256_permutex_epi64
        };

        const SIMD_LANES: u32 = 8;

        let blocks_per_word = u32::BITS / BITS as u32;
        let sets_per_word = blocks_per_word / SIMD_LANES;

        // Limit words to max size for these bits
        let max_len = 4096 / blocks_per_word;
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
                    0
                );

                // Shifts all lanes to their next location in the word.
                let vshiftall = _mm256_set1_epi32(8 * bits);

                let mut w = 0;
                let mut offset = 0;

                // Decode 32 blocks per word.
                // We do this in 4 sets of 8
                while w < words.len() {
                    let mut vword = _mm256_set1_epi32(words[w] as i32);
                    vword = _mm256_srlv_epi32(vword, vshift);

                    let mut s = 0;
                    while s < sets_per_word - 1 {
                        let vmasked = _mm256_and_si256(vword, vmask);
                        let vpack = _mm256_packus_epi32(vmasked, vmasked);
                        let vshorts = unsafe { _mm256_permutex_epi64(vpack, 0b11011000) };

                        debug_assert!(offset <= 4080);
                        unsafe {
                            std::ptr::copy_nonoverlapping(
                                &vshorts as *const __m256i as *const u16,
                                indices.as_mut_ptr().add(offset),
                                8
                            );
                        }

                        vword = _mm256_srlv_epi32(vword, vshiftall);

                        offset += 8;
                        s += 1;
                    }

                    // Last set does not need a shift all at the end
                    let vmasked = _mm256_and_si256(vword, vmask);
                    let vpack = _mm256_packus_epi32(vmasked, vmasked);
                    let vshorts = unsafe { _mm256_permutex_epi64(vpack, 0b11011000) };

                    debug_assert!(offset <= 4088);
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            &vshorts as *const __m256i as *const u16,
                            indices.as_mut_ptr().add(offset),
                            8
                        );
                    }

                    offset += 8;
                    w += 1;
                }
            },
            _ => unreachable!("invalid BITS generic for `unpack_oct`")
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