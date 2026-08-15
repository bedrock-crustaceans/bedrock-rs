use crate::greedy::{GreedyArray, GreedyArrayIter};
use crate::lazy::{LazyArray, LazyArrayIter};
use crate::{
    UnpackingMethod,
    error::{Error, Result},
};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::Read;
use std::io::{Cursor, Write};
use std::iter::FusedIterator;

/// Valid bit sizes to use for indices.
pub const VALID_BITS: [u8; 8] = [1, 2, 3, 4, 5, 6, 8, 16];

/// The number of 32-bit words needed to pack 4096 indices at the given bit width. Bit
/// sizes that do not divide 32 evenly (3, 5, 6) need one extra, partially-filled word.
///
/// Shared by `greedy.rs` and `lazy.rs`'s tests so their expectations for a given width
/// cannot drift apart from each other or from the packing code itself.
#[cfg(test)]
pub(crate) fn expected_word_count(bits: u8) -> usize {
    let per_word = u32::BITS / bits as u32;
    4096u32.div_ceil(per_word) as usize
}

/// An iterator over a bit array.
pub enum BitArrayIter<'a> {
    /// See [`GreedyArray`].
    Greedy(GreedyArrayIter<'a>),
    /// See [`LazyArray`].
    Lazy(LazyArrayIter<'a>),
    /// See [`BitArray::Empty`]: yields 4096 zeros.
    Empty(std::iter::RepeatN<u16>),
}

impl Iterator for BitArrayIter<'_> {
    type Item = u16;

    fn next(&mut self) -> Option<u16> {
        match self {
            BitArrayIter::Greedy(iter) => iter.next(),
            BitArrayIter::Lazy(iter) => iter.next(),
            BitArrayIter::Empty(iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            BitArrayIter::Greedy(iter) => iter.size_hint(),
            BitArrayIter::Lazy(iter) => iter.size_hint(),
            BitArrayIter::Empty(iter) => iter.size_hint(),
        }
    }
}

impl ExactSizeIterator for BitArrayIter<'_> {
    fn len(&self) -> usize {
        match self {
            BitArrayIter::Greedy(iter) => iter.len(),
            BitArrayIter::Lazy(iter) => iter.len(),
            BitArrayIter::Empty(iter) => iter.len(),
        }
    }
}

impl FusedIterator for BitArrayIter<'_> {}

/// The method by which the indices are encoded.
pub enum IndicesType {
    /// This chunk contains regular data.
    Data(BitArray),
    /// This chunk contains no data
    Empty,
    /// Inherits data from the previous chunk.
    Inherit,
}

/// The type of array used in the subchunk. This can either be an unpacked array that is
/// expanded on subchunk deserialization or a packed array that is expanded lazily.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BitArray {
    /// See [`GreedyArray`].
    Greedy(GreedyArray),
    /// See [`LazyArray`].
    Lazy(LazyArray),
    /// A zero bits-per-index array: every index is 0, and the palette this array indexes
    /// into holds exactly one entry. Carries no packed words at all -- the on-disk form is
    /// the header byte alone, with nothing else belonging to the index array.
    Empty,
}

impl BitArray {
    /// Creates an iterator over this array.
    #[inline]
    pub fn iter(&self) -> BitArrayIter<'_> {
        self.into_iter()
    }

    /// Whether this array uses the lazy unpacking strategy.
    #[inline]
    pub fn is_lazy(&self) -> bool {
        matches!(self, BitArray::Lazy(_))
    }

    /// Deserializes a data bit array.
    #[inline]
    fn from_data_helper<M: UnpackingMethod, R>(reader: &mut Cursor<R>, bits: u8) -> Result<Self>
    where
        Cursor<R>: Read,
    {
        if !VALID_BITS.contains(&bits) {
            return Err(Error::InvalidBitSize(bits));
        }

        Ok(if M::IS_LAZY {
            BitArray::Lazy(LazyArray::from_disk(reader, bits)?)
        } else {
            BitArray::Greedy(GreedyArray::from_disk(reader, bits)?)
        })
    }

    /// Deserializes an array from disk format.
    pub fn from_disk<M: UnpackingMethod, R>(reader: &mut Cursor<R>) -> Result<IndicesType>
    where
        Cursor<R>: Read,
    {
        let bits = reader.read_u8()? >> 1;
        Ok(match bits {
            0x00 => IndicesType::Empty,
            0x7f => IndicesType::Inherit,
            bits => IndicesType::Data(BitArray::from_data_helper::<M, _>(reader, bits)?),
        })
    }

    /// Serializes this array in disk format.
    ///
    /// [`BitArray::Empty`] writes only the header byte: a zero bits-per-index layer carries
    /// no packed words on disk, regardless of `palette_size`.
    pub fn to_disk<W>(&self, writer: &mut Cursor<W>, palette_size: usize) -> Result<()>
    where
        Cursor<W>: Write,
    {
        if let BitArray::Empty = self {
            writer.write_u8(0)?;
            return Ok(());
        }

        // The width written is the *smallest* valid bit size that can still address every
        // palette entry (`VALID_BITS` is ascending, and this stops at the first match) --
        // not merely *a* valid size. Writing a wider-than-necessary index array is not
        // wrong on its own terms, but it is not what the format's packed-word design is
        // for, and it would not match what the game itself writes for the same palette.
        let bits = VALID_BITS
            .into_iter()
            .find(|&b| 2usize.pow(b as u32) >= palette_size)
            .unwrap_or(16);

        writer.write_u8(bits << 1)?;

        match self {
            BitArray::Greedy(array) => array.to_disk(writer, bits as u32),
            BitArray::Lazy(array) => array.to_disk(writer),
            BitArray::Empty => unreachable!("handled above"),
        }
    }

    /// Gets the value at the specified index.
    pub fn get(&self, index: usize) -> Option<u16> {
        match self {
            Self::Greedy(array) => array.get(index),
            Self::Lazy(array) => array.get(index),
            Self::Empty => (index < 4096).then_some(0),
        }
    }

    /// Sets the value at the specified index.
    ///
    /// [`BitArray::Empty`] carries no storage, so the first write to one materializes it
    /// into a real (initially all-zero) [`GreedyArray`] before applying the write.
    pub fn set(&mut self, pos: usize, value: u16) -> bool {
        if let Self::Empty = self {
            *self = Self::Greedy(GreedyArray::from(Box::new([0u16; 4096])));
        }

        match self {
            Self::Greedy(array) => array.set(pos, value),
            Self::Lazy(array) => array.set(pos, value),
            Self::Empty => unreachable!("materialized above"),
        }
    }
}

impl<'a> IntoIterator for &'a BitArray {
    type Item = u16;
    type IntoIter = BitArrayIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            BitArray::Lazy(array) => BitArrayIter::Lazy(array.into_iter()),
            BitArray::Greedy(array) => BitArrayIter::Greedy(array.into_iter()),
            BitArray::Empty => BitArrayIter::Empty(std::iter::repeat_n(0u16, 4096)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_iter_yields_4096_zeros() {
        let array = BitArray::Empty;
        let values: Vec<u16> = array.iter().collect();

        assert_eq!(values.len(), 4096);
        assert!(values.iter().all(|&v| v == 0));
    }

    #[test]
    fn empty_set_materializes_into_greedy_and_leaves_other_indices_zero() {
        let mut array = BitArray::Empty;

        assert!(array.set(17, 5));

        assert!(matches!(array, BitArray::Greedy(_)));
        assert_eq!(array.get(17), Some(5));
        assert_eq!(array.get(0), Some(0));
        assert_eq!(array.get(4095), Some(0));
        assert_eq!(array.iter().filter(|&v| v != 0).count(), 1);
    }

    /// `to_disk` writes the *smallest* valid bit width that fits `palette_size`, not merely
    /// a valid one large enough -- pins the direction of the header-byte fix (the prior
    /// overwrite-loop picked the largest valid width for every non-trivial palette).
    #[test]
    fn to_disk_writes_the_minimal_valid_width_for_the_palette_size() {
        let cases: [(usize, u8); 9] = [
            (1, 1),
            (2, 1),
            (3, 2),
            (4, 2),
            (5, 3),
            (8, 3),
            (9, 4),
            (256, 8),
            (257, 16),
        ];

        for (palette_size, expected_bits) in cases {
            let array = BitArray::Greedy(GreedyArray::from(Box::new([0u16; 4096])));
            let mut writer = Cursor::new(Vec::new());
            array.to_disk(&mut writer, palette_size).unwrap();

            let header = writer.into_inner()[0];
            assert_eq!(
                header >> 1,
                expected_bits,
                "palette_size={palette_size}: expected {expected_bits} bits, header byte was {header}"
            );
        }
    }
}
