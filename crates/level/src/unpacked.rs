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
                if offset == 4096 {
                    break;
                }

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

        let per_word = u32::BITS / bits as u32;

        let mask = !(!0u32 << bits);
        let vmask = _mm256_set1_epi32(mask as i32);
        let vbits = _mm_cvtsi32_si128(bits as i32);

        let mut offset = 0;
        let mut i = 0;
        let mut w = 0;

        while w + 8 < words.len() {
            // Loads 8 words into a SIMD register.
            // Safety:
            // This is safe because `AVX2` is supported by the caller and words has at least 8 remaining words.
            // Additionally this has no alignment requirements since it is an unaligned read.
            let mut vwords = unsafe { _mm256_loadu_si256(words.as_ptr().add(w).cast::<__m256i>()) };
            
            while i < per_word {
                // Bitwise and of all words with the mask.
                let vindices = _mm256_and_si256(vwords, vmask);
                // Pack the 32-bit words into twice as many 16-bit ones.
                // They are now in the order [A0 A1 A2 A3 B0 B1 B2 B3 | A4 A5 A6 A7 B4 B5 B6 B7]
                let vpack = _mm256_packus_epi32(vindices, vindices);
                // Permute the shorts to get the order
                // [A0 A1 A2 A3 A4 A5 A6 A7 ...]
                let vperm = _mm256_permute4x64_epi64(vpack, 0b11011000);

                unsafe {
                    _mm_storeu_si128(
                        indices.as_mut_ptr().add(offset).cast::<__m128i>(),
                        _mm256_castsi256_si128(vperm)
                    );
                }

                // Shift all words to the right by `bits` bits.
                vwords = _mm256_srl_epi32(vwords, vbits);

                i += 1;
            }

            i = 0;
            w += 8;
        }

        // Do remaining words normally if the word count is not a multiple of 8.
        if w != words.len() {
            Self::unpack_nonsimd(bits, &words[w..], &mut indices[offset..])
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