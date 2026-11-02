use crate::bits::VALID_BITS;
use crate::error::Result;
use crate::greedy::GreedyArray;
use std::cmp::Ordering;
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

    /// Returns the value at `index`, or `None` if the index is greater than or equal to 4096.
    pub fn get(&self, index: usize) -> Option<u16> {
        if index >= 4096 {
            return None;
        }

        let bits = u32::from(self.bits);
        let blocks_per_word = 32 / bits;
        let mask = !(!0u32 << self.bits);

        #[expect(
            clippy::cast_possible_truncation,
            reason = "index is below 4096, checked above"
        )]
        let index = index as u32;
        let word_index = index % blocks_per_word;
        let array_index = index / blocks_per_word;
        let word = self.words[array_index as usize];

        let shift = bits * word_index;

        #[expect(
            clippy::cast_possible_truncation,
            reason = "the mask keeps at most 16 bits, the largest valid bit size"
        )]
        let value = ((word >> shift) & mask) as u16;
        Some(value)
    }

    /// Unpacks and then packs the array to use the new bit size. The indices will saturate if the bit size is too small for the array.
    ///
    /// This function may decide to use a different bit size to create a valid packed array.
    pub fn repack(&mut self, mut bits: u8) {
        // Ensure the bit size is valid.
        bits = VALID_BITS
            .iter()
            .find_map(|&b| match b.cmp(&bits) {
                // If the requested size is valid, leave it unchanged.
                Ordering::Equal => Some(bits),
                // If the requested bit size was not found, take the first one larger than it.
                Ordering::Greater => Some(b),
                Ordering::Less => None,
            })
            .unwrap_or(16); // Clamp the bit size to at most 16.

        // Unpack the whole array and repack it again
        let greedy = GreedyArray::unpack(self.words(), self.bits);

        let blocks_per_word = 32 / bits;
        let total_words = 4096usize.div_ceil(blocks_per_word as usize);

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
        #[expect(
            clippy::cast_possible_truncation,
            reason = "the bit width of a u16 is at most 16"
        )]
        let required_bits = value.bit_width() as u8;
        if required_bits > self.bits {
            // Needs re-encoding. The function will automatically select a proper bit size that fits the value.
            self.repack(required_bits);
        }

        let bits = u32::from(self.bits);
        let blocks_per_word = 32 / bits;
        let base_mask = !(!0u32 << self.bits);

        #[expect(
            clippy::cast_possible_truncation,
            reason = "index is below 4096, checked above"
        )]
        let index = index as u32;
        let word_index = index % blocks_per_word;
        let array_index = index / blocks_per_word;
        let word = self.words[array_index as usize];

        let mask = base_mask << (bits * word_index);

        // Zero all bits in the location
        let zeroed = word & !mask;
        // Clamp value to correct amount of bits
        let clamped = u32::from(value) & base_mask;
        // Then set the zeroed bits to the clamped value
        let set = zeroed | (clamped << (bits * word_index));

        self.words[array_index as usize] = set;

        true
    }

    /// Reads the packed words of an array that uses `bits` bits per block.
    ///
    /// # Errors
    ///
    /// Returns an error if the reader runs out of data or fails to read.
    pub fn from_disk<R>(reader: &mut Cursor<R>, bits: u8) -> Result<Self>
    where
        Cursor<R>: Read,
    {
        let per_word = u32::BITS / u32::from(bits);
        let word_count = 4096u32.div_ceil(per_word);

        let mut words = vec![0; word_count as usize];
        reader.read_exact(bytemuck::cast_slice_mut::<u32, u8>(&mut words))?;

        Ok(Self { bits, words })
    }

    /// Writes the packed words as they are, without re-packing.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to `writer` fails.
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

impl Iterator for LazyArrayIter<'_> {
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

impl FusedIterator for LazyArrayIter<'_> {}

impl ExactSizeIterator for LazyArrayIter<'_> {
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
