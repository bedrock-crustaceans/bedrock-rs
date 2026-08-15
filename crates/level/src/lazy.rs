use crate::bits::VALID_BITS;
use crate::error::Result;
use crate::greedy::GreedyArray;
use std::io::{Cursor, Read, Write};
use std::iter::FusedIterator;

/// An array that is still packed. Words are unpacked only as needed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LazyArray {
    /// The amount of bits per block.
    bits: u8,
    /// The words containing block indices.
    words: Vec<u32>,
}

impl LazyArray {
    /// Creates a new array from the given words.
    pub const fn new(bits: u8, words: Vec<u32>) -> Self {
        Self { bits, words }
    }

    /// Returns the amount of words that are in this array.
    pub fn word_count(&self) -> usize {
        self.words.len()
    }

    /// Creates an iterator over this array.
    pub fn iter(&self) -> LazyArrayIter<'_> {
        LazyArrayIter::from(self)
    }

    /// The amount of bits that are currently being used by this array.
    pub fn bits(&self) -> u8 {
        self.bits
    }

    /// The packed words that this array consists of.
    pub fn words(&self) -> &[u32] {
        &self.words
    }

    /// Returns the value at `index`.
    ///
    /// # Panics
    ///
    /// This function panics if the index is greater than or equal to 4096.
    pub fn get(&self, index: usize) -> Option<u16> {
        if index >= 4096 {
            return None;
        }

        let blocks_per_word = 32 / self.bits as u32;
        let mask = !(!0u32 << self.bits);

        let word_index = index as u32 % blocks_per_word;
        let array_index = index as u32 / blocks_per_word;
        let word = self.words[array_index as usize];

        let shift = self.bits as u32 * word_index;

        Some(((word >> shift) & mask) as u16)
    }

    /// Unpacks and then packs the array to use the new bit size. The indices will saturate if the bit size is too small for the array.
    ///
    /// This function may decide to use a different bit size to create a valid packed array.
    pub fn repack(&mut self, mut bits: u8) {
        // Ensure the bit size is valid.
        bits = VALID_BITS
            .iter()
            .find_map(|&b| {
                if b == bits {
                    // If the requested size is valid, leave it unchanged.
                    Some(bits)
                } else if b > bits {
                    // If the requested bit size was not found, take the first one larger than it.
                    Some(b)
                } else {
                    None
                }
            })
            .unwrap_or(16); // Clamp the bit size to at most 16.

        // Unpack the whole array and repack it again
        let greedy = GreedyArray::unpack(self.words(), self.bits);

        let blocks_per_word = 32u32 / bits as u32;
        // `div_ceil`, not `/`: bit sizes that do not divide 32 evenly (3, 5, 6) need one
        // extra, partially-filled word to hold all 4096 indices.
        let total_words = 4096u32.div_ceil(blocks_per_word) as usize;

        self.words.resize(total_words, 0);
        self.bits = bits;

        greedy.pack_into(&mut self.words, bits);
    }

    /// Sets the value at `index`. Note that the passed value will be clamped to the bit size.
    /// I.e. passing 42 to a 4-bit packed array will set result in the value being set to 16.
    ///
    /// # Returns
    ///
    /// This function returns whether the change was successful.
    pub fn set(&mut self, index: usize, value: u16) -> bool {
        if index >= 4096 {
            return false;
        }

        // The amount of bits required to store the given value. This is not necessarily a
        // valid bit size. `ilog2` panics on 0, and 0 needs no bits at all to represent.
        let required_bits = if value == 0 {
            0
        } else {
            value.ilog2() as u8 + 1
        };
        if required_bits > self.bits {
            // Needs re-encoding. The function will automatically select a proper bit size that fits the value.
            self.repack(required_bits);
        }

        let blocks_per_word = 32 / self.bits as u32;
        let base_mask = !(!0u32 << self.bits);

        let word_index = index as u32 % blocks_per_word;
        let array_index = index as u32 / blocks_per_word;
        let word = self.words[array_index as usize];

        let mask = base_mask << (self.bits as u32 * word_index);

        // Zero all bits in the location
        let zeroed = word & !mask;
        // Clamp value to correct amount of bits
        let clamped = value as u32 & base_mask;
        // Then set the zeroed bits to the clamped value
        let set = zeroed | (clamped << (self.bits as u32 * word_index));

        self.words[array_index as usize] = set;

        true
    }

    pub fn from_disk<R>(reader: &mut Cursor<R>, bits: u8) -> Result<Self>
    where
        Cursor<R>: Read,
    {
        let per_word = u32::BITS / bits as u32;
        let word_count = 4096u32.div_ceil(per_word);

        let mut words = vec![0; word_count as usize];
        reader.read_exact(bytemuck::cast_slice_mut::<u32, u8>(&mut words))?;

        Ok(Self { bits, words })
    }

    pub fn to_disk<W>(&self, writer: &mut Cursor<W>) -> Result<()>
    where
        Cursor<W>: Write,
    {
        writer.write_all(bytemuck::cast_slice::<u32, u8>(&self.words))?;
        Ok(())
    }
}

/// An iterator over a [`LazyArray`].
pub struct LazyArrayIter<'a> {
    /// The current index in the array.
    index: usize,
    /// The array to iterate over.
    array: &'a LazyArray,
}

impl<'a> Iterator for LazyArrayIter<'a> {
    type Item = u16;

    fn next(&mut self) -> Option<u16> {
        let item = self.array.get(self.index);
        self.index += 1;
        item
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<'a> FusedIterator for LazyArrayIter<'a> {}

impl<'a> ExactSizeIterator for LazyArrayIter<'a> {
    fn len(&self) -> usize {
        4096 - self.index
    }
}

impl<'a> From<&'a LazyArray> for LazyArrayIter<'a> {
    fn from(array: &'a LazyArray) -> Self {
        LazyArrayIter { index: 0, array }
    }
}

impl<'a> IntoIterator for &'a LazyArray {
    type Item = u16;
    type IntoIter = LazyArrayIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        LazyArrayIter::from(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bits::{VALID_BITS, expected_word_count};

    fn deterministic_values(bits: u8) -> Vec<u16> {
        let max = if bits == 16 {
            u16::MAX as u32
        } else {
            (1u32 << bits) - 1
        };
        (0..4096u32).map(|i| (i % (max + 1)) as u16).collect()
    }

    fn build(bits: u8, values: &[u16]) -> LazyArray {
        let words = vec![0u32; expected_word_count(bits)];
        let mut array = LazyArray::new(bits, words);
        for (i, &v) in values.iter().enumerate() {
            assert!(array.set(i, v));
        }
        array
    }

    #[test]
    fn word_count_matches_expected_for_every_valid_width() {
        for bits in VALID_BITS {
            let expected_words = expected_word_count(bits);
            let values = deterministic_values(bits);
            let array = build(bits, &values);
            assert_eq!(
                array.word_count(),
                expected_words,
                "bits={bits}: word count"
            );
            for (i, &v) in values.iter().enumerate() {
                assert_eq!(array.get(i), Some(v), "bits={bits}: index {i}");
            }
        }
    }

    #[test]
    fn decode_encode_byte_identity_for_every_valid_width() {
        for bits in VALID_BITS {
            let values = deterministic_values(bits);
            let array = build(bits, &values);

            let mut first = Cursor::new(Vec::new());
            array.to_disk(&mut first).unwrap();
            let first_bytes = first.into_inner();

            let decoded =
                LazyArray::from_disk(&mut Cursor::new(first_bytes.as_slice()), bits).unwrap();

            let mut second = Cursor::new(Vec::new());
            decoded.to_disk(&mut second).unwrap();
            let second_bytes = second.into_inner();

            assert_eq!(
                first_bytes, second_bytes,
                "bits={bits}: decode -> encode is not byte-identical"
            );
            for (i, &v) in values.iter().enumerate() {
                assert_eq!(decoded.get(i), Some(v), "bits={bits}: index {i}");
            }
        }
    }

    /// `repack` re-packs into a new bit width via `GreedyArray::pack_into`. Exercise it into
    /// each of the three widths that need a padded final word (3, 5, 6 bits), from a source
    /// width that does not, to isolate the repack path itself from `from_disk`/`to_disk`.
    #[test]
    fn repack_into_a_padded_width_preserves_every_value_and_sizes_correctly() {
        for bits in [3u8, 5, 6] {
            let expected_words = expected_word_count(bits);
            let values = deterministic_values(bits);
            let mut array = build(8, &values); // start at a width with no padding
            array.repack(bits);

            assert_eq!(array.bits(), bits);
            assert_eq!(
                array.word_count(),
                expected_words,
                "bits={bits}: word count after repack"
            );
            for (i, &v) in values.iter().enumerate() {
                assert_eq!(array.get(i), Some(v), "bits={bits}: index {i} after repack");
            }
        }
    }

    /// The trailing, partially-filled word's unused high bits must be zero after a repack
    /// into a padded width, not whatever `Vec::resize` or an earlier pack left behind.
    #[test]
    fn repack_zero_fills_the_padded_final_word() {
        for bits in [3u8, 5, 6] {
            let per_word = 32u32 / bits as u32;
            let word_count = expected_word_count(bits);
            let used_in_last_word = 4096 - (word_count - 1) * per_word as usize;

            // Max value that fits, so any bit that should be zero but isn't would show up.
            let max = (1u16 << bits) - 1;
            let values = vec![max; 4096];
            let mut array = build(8, &values);
            array.repack(bits);

            let last_word = array.words()[word_count - 1];
            let used_bits = used_in_last_word as u32 * bits as u32;
            assert_eq!(
                last_word & (u32::MAX << used_bits),
                0,
                "bits={bits}: padded high bits of the final word were not zero-filled after repack"
            );
        }
    }
}
