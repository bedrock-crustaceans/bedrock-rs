use std::{io::{Read, Write}, iter::FusedIterator};

use byteorder::{ReadBytesExt, WriteBytesExt};
use nbtx::LittleEndian;

use crate::{
    error::{Error, Result},
    subchunk::Layer,
};

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
    pub const fn words_count(&self) -> usize {
        let per_word = 32 / self.bits;
        4096 / per_word as usize
    }

    /// Creates an iterator over this array.
    pub fn iter(&self) -> PackedArrayIter<'_> {
        PackedArrayIter::from(self)
    }

    pub const fn bits(&self) -> u32 {
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
            return None
        }

        let blocks_per_word = u32::BITS / self.bits;
        let mask = !(!0u32 << self.bits);

        let word_index = index as u32 % blocks_per_word;
        let array_index = index as u32 / blocks_per_word;
        let word = self.words[array_index as usize];

        Some(((word >> self.bits * word_index) & mask) as u16)
    }

    /// Sets the value at `index`. Note that the passed value will be clamped to the bit size.
    /// I.e. passing 42 to a 4-bit packed array will set result in the value being set to 16.
    ///
    /// # Panics
    ///
    /// This function panics if the index is greater than or equal to 4096.
    pub fn set(&mut self, index: usize, value: u16) {
        assert!(
            index < 4096,
            "packed array index out of bounds, got 4096 < {index}"
        );

        let blocks_per_word = u32::BITS / self.bits;
        let base_mask = !(!0u32 << self.bits);

        let word_index = index as u32 % blocks_per_word;
        let array_index = index as u32 / blocks_per_word;
        let word = self.words[array_index as usize];

        let mask = base_mask << self.bits * word_index;

        // Zero all bits in the location
        let zeroed = word & !mask;
        // Clamp value to correct amount of bits
        let clamped = value as u32 & base_mask;
        // Then set the zeroed bits to the clamped value
        let set = zeroed | (clamped << self.bits * word_index);

        self.words[array_index as usize] = set;
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
    type IntoIter = PackedArrayIter<'a>;
    type Item = u16;

    fn into_iter(self) -> Self::IntoIter {
        PackedArrayIter::from(self)
    }
}

/// The type of array used in the subchunk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArrayType {
    Greedy(Box<[u16; 4096]>),
    Lazy(PackedArray),
}

impl ArrayType {
    pub fn get(&self, pos: usize) -> Option<u16> {
        match self {
            Self::Greedy(array) => array.get(pos).copied(),
            Self::Lazy(array) => array.get(pos)
        }
    }

    pub fn set(&mut self, pos: usize, value: u16) {
        match self {
            Self::Greedy(array) => array[pos] = value,
            Self::Lazy(array) => array.set(pos, value)
        }
    }
}

impl Layer {
    pub(crate) fn pack_array<W>(
        mut writer: W,
        array: &[u16; 4096],
        max_index: usize,
        is_network: bool,
    ) -> Result<()>
    where
        W: Write,
    {
        // Determine the required bits per index
        let index_size = {
            let mut bits_per_block = 0;
            // Loop over allowed values.
            for b in [1, 2, 3, 4, 5, 6, 8, 16] {
                if 2usize.pow(b) >= max_index {
                    bits_per_block = b;
                    break;
                }
            }

            bits_per_block as u8
        };

        writer.write_u8(index_size << 1 | is_network as u8)?;

        // Amount of indices that fit in a single 32-bit integer.
        let per_word = u32::BITS / index_size as u32;

        let mut offset = 0;
        while offset < 4096 {
            let mut word = 0;
            for w in 0..per_word {
                if offset == 4096 {
                    break;
                }

                let index = array[offset] as u32;
                word |= index << (w * index_size as u32);

                offset += 1;
            }

            writer.write_u32::<LittleEndian>(word)?;
        }

        Ok(())
    }

    pub(crate) fn unpack_array<R>(mut reader: R) -> Result<PackedResult>
    where
        R: Read,
    {
        let index_size = reader.read_u8()? >> 1;
        if index_size == 0 {
            return Ok(PackedResult::Empty);
        } else if index_size == 0x7f {
            return Ok(PackedResult::Inherit);
        } else if ![1, 2, 3, 4, 5, 6, 8, 16].contains(&index_size) {
            return Err(Error::InvalidIndexSize(index_size));
        }

        let per_word = u32::BITS / index_size as u32;
        let word_count = 4096u32.div_ceil(per_word);
        let mask = !(!0u32 << index_size);

        let mut indices = Box::new([0u16; 4096]);
        let mut offset = 0;

        for _ in 0..word_count {
            let mut word = reader.read_u32::<LittleEndian>()?;

            for _ in 0..per_word {
                if offset == 4096 {
                    break;
                }

                indices[offset] = (word & mask) as u16;
                word >>= index_size;

                offset += 1;
            }
        }

        Ok(PackedResult::Data(indices))
    }
}

/// Return value from packed array deserialisation.
#[derive(Debug, PartialEq, Eq)]
pub enum PackedResult {
    /// The packed array was empty.
    Empty,
    /// This array inherits from the previously processed array.
    Inherit,
    /// New data for the array.
    Data(Box<[u16; 4096]>),
}
