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

        let blocks_per_word = 32 / bits;
        let total_words = 4096 / blocks_per_word as usize;

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

        // The amount of bits required to store the given value. This is not necessarily a valid bit size.
        let required_bits = value.ilog2() as u8 + 1;
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
