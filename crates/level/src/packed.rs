use crate::error::Result;
use std::io::{Read, Write};
use std::iter::FusedIterator;

/// An array that is still packed. Words are unpacked as needed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackedArray {
    /// The amount of bits per block.
    bits: u32,
    /// The words containing block indices.
    words: Vec<u32>,
}

impl PackedArray {
    /// Creates a new array from the given words.
    pub const fn new(bits: u32, words: Vec<u32>) -> Self {
        Self { bits, words }
    }

    /// Returns the amount of words that are in this array.
    pub fn word_count(&self) -> usize {
        self.words.len()
    }

    /// Creates an iterator over this array.
    pub fn iter(&self) -> PackedArrayIter<'_> {
        PackedArrayIter::from(self)
    }

    pub fn bits(&self) -> u32 {
        self.bits
    }

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

        let blocks_per_word = u32::BITS / self.bits;
        let mask = !(!0u32 << self.bits);

        let word_index = index as u32 % blocks_per_word;
        let array_index = index as u32 / blocks_per_word;
        let word = self.words[array_index as usize];

        let shift = self.bits * word_index;

        Some(((word >> shift) & mask) as u16)
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

        let blocks_per_word = u32::BITS / self.bits;
        let base_mask = !(!0u32 << self.bits);

        let word_index = index as u32 % blocks_per_word;
        let array_index = index as u32 / blocks_per_word;
        let word = self.words[array_index as usize];

        let mask = base_mask << (self.bits * word_index);

        // Zero all bits in the location
        let zeroed = word & !mask;
        // Clamp value to correct amount of bits
        let clamped = value as u32 & base_mask;
        // Then set the zeroed bits to the clamped value
        let set = zeroed | (clamped << (self.bits * word_index));

        self.words[array_index as usize] = set;

        true
    }

    pub fn from_disk<R: Read>(mut reader: R, bits: u8) -> Result<Self> {
        let per_word = u32::BITS / bits as u32;
        let word_count = 4096u32.div_ceil(per_word);

        let mut words = vec![0; word_count as usize];
        reader.read_exact(bytemuck::cast_slice_mut::<u32, u8>(&mut words))?;

        Ok(Self {
            bits: bits as u32,
            words,
        })
    }

    pub fn to_disk<W: Write>(&self, mut writer: W) -> Result<()> {
        writer.write_all(bytemuck::cast_slice::<u32, u8>(&self.words))?;
        Ok(())
    }
}

/// An iterator over [`PackedArray`].
pub struct PackedArrayIter<'a> {
    /// The current index in the array.
    index: usize,
    /// The array to iterate over.
    array: &'a PackedArray,
}

impl<'a> Iterator for PackedArrayIter<'a> {
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

impl<'a> FusedIterator for PackedArrayIter<'a> {}

impl<'a> ExactSizeIterator for PackedArrayIter<'a> {
    fn len(&self) -> usize {
        4096 - self.index
    }
}

impl<'a> From<&'a PackedArray> for PackedArrayIter<'a> {
    fn from(array: &'a PackedArray) -> Self {
        PackedArrayIter { index: 0, array }
    }
}

impl<'a> IntoIterator for &'a PackedArray {
    type Item = u16;
    type IntoIter = PackedArrayIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        PackedArrayIter::from(self)
    }
}
