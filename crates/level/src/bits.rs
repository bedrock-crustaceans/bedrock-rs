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

/// An iterator over a bit array.
pub enum BitArrayIter<'a> {
    /// See [`GreedyArray`].
    Greedy(GreedyArrayIter<'a>),
    /// See [`LazyArray`].
    Lazy(LazyArrayIter<'a>),
}

impl Iterator for BitArrayIter<'_> {
    type Item = u16;

    fn next(&mut self) -> Option<u16> {
        match self {
            BitArrayIter::Greedy(iter) => iter.next(),
            BitArrayIter::Lazy(iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            BitArrayIter::Greedy(iter) => iter.size_hint(),
            BitArrayIter::Lazy(iter) => iter.size_hint(),
        }
    }
}

impl ExactSizeIterator for BitArrayIter<'_> {
    fn len(&self) -> usize {
        match self {
            BitArrayIter::Greedy(iter) => iter.len(),
            BitArrayIter::Lazy(iter) => iter.len(),
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

    /// Creates an array where every index is zero, which is what a layer without index data means.
    pub fn zeroed<M: UnpackingMethod>() -> Self {
        if M::IS_LAZY {
            BitArray::Lazy(LazyArray::new(1, vec![0; 4096 / 32]))
        } else {
            BitArray::Greedy(GreedyArray::from(Box::new([0u16; 4096])))
        }
    }

    /// The smallest bit size able to index a palette of `palette_size` entries, or 0 when
    /// the palette has at most one entry and no indices need to be stored.
    pub fn bits_for(palette_size: usize) -> u8 {
        if palette_size <= 1 {
            return 0;
        }
        VALID_BITS
            .into_iter()
            .find(|&bits| 1usize << bits >= palette_size)
            .unwrap_or(16)
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

    /// Serializes this array in disk format, packed with the smallest bit size that fits `palette_size`.
    /// A palette of at most one entry writes no index data at all.
    pub fn to_disk<W>(&self, writer: &mut Cursor<W>, palette_size: usize) -> Result<()>
    where
        Cursor<W>: Write,
    {
        let bits = Self::bits_for(palette_size);
        writer.write_u8(bits << 1)?;
        if bits == 0 {
            return Ok(());
        }

        match self {
            BitArray::Greedy(array) => array.to_disk(writer, bits as u32),
            BitArray::Lazy(array) if array.bits() == bits => array.to_disk(writer),
            BitArray::Lazy(array) => GreedyArray::from(array).to_disk(writer, bits as u32),
        }
    }

    /// Gets the value at the specified index.
    pub fn get(&self, index: usize) -> Option<u16> {
        match self {
            Self::Greedy(array) => array.get(index),
            Self::Lazy(array) => array.get(index),
        }
    }

    /// Sets the value at the specified index.
    pub fn set(&mut self, pos: usize, value: u16) -> bool {
        match self {
            Self::Greedy(array) => array.set(pos, value),
            Self::Lazy(array) => array.set(pos, value),
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
        }
    }
}
