use crate::packed::{PackedArray, PackedArrayIter};
use crate::unpacked::UnpackedArray;
use crate::{
    PackingMethod,
    error::{Error, Result},
};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::iter::{Copied, FusedIterator};
use std::{
    io::{Read, Write},
    slice,
};

/// Valid bit sizes to use for indices.
const VALID_BITS: [u8; 8] = [1, 2, 3, 4, 5, 6, 8, 16];

pub enum BitArrayIter<'a> {
    Unpacked(Copied<slice::Iter<'a, u16>>),
    Packed(PackedArrayIter<'a>),
}

impl Iterator for BitArrayIter<'_> {
    type Item = u16;

    fn next(&mut self) -> Option<u16> {
        match self {
            BitArrayIter::Unpacked(iter) => iter.next(),
            BitArrayIter::Packed(iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            BitArrayIter::Unpacked(iter) => iter.size_hint(),
            BitArrayIter::Packed(iter) => iter.size_hint(),
        }
    }
}

impl ExactSizeIterator for BitArrayIter<'_> {
    fn len(&self) -> usize {
        match self {
            BitArrayIter::Unpacked(iter) => iter.len(),
            BitArrayIter::Packed(iter) => iter.len(),
        }
    }
}

impl FusedIterator for BitArrayIter<'_> {}

pub enum IndicesType {
    Data(BitArray),
    Empty,
    Inherit,
}

/// The type of array used in the subchunk. This can either be an unpacked array that is
/// expanded on subchunk deserialization or a packed array that is expanded lazily.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BitArray {
    Unpacked(UnpackedArray),
    Packed(PackedArray),
}

impl BitArray {
    /// Creates an iterator over this array.
    pub fn iter(&self) -> BitArrayIter<'_> {
        self.into_iter()
    }

    fn data_from_disk<M: PackingMethod, R: Read>(mut reader: R, bits: u8) -> Result<Self> {
        if !VALID_BITS.contains(&bits) {
            return Err(Error::InvalidBitSize(bits));
        }

        Ok(if M::IS_PACKED {
            BitArray::Packed(PackedArray::from_disk(&mut reader, bits)?)
        } else {
            BitArray::Unpacked(UnpackedArray::from_disk(&mut reader, bits)?)
        })
    }

    /// Deserializes this array in disk format.
    pub fn from_disk<M: PackingMethod, R: Read>(mut reader: R) -> Result<Self> {
        let bits = reader.read_u8()? >> 1;
        match bits {
            0x00 => Err(Error::Invalid("chunk layer packed array cannot be empty")),
            0x7f => Err(Error::Invalid("chunk layers do not support inheritance")),
            bits => BitArray::data_from_disk::<M, _>(&mut reader, bits),
        }
    }

    pub fn from_disk_typed<M: PackingMethod, R: Read>(mut reader: R) -> Result<IndicesType> {
        let bits = reader.read_u8()? >> 1;
        Ok(match bits {
            0x00 => IndicesType::Empty,
            0x7f => IndicesType::Inherit,
            bits => IndicesType::Data(BitArray::data_from_disk::<M, _>(&mut reader, bits)?),
        })
    }

    /// Serializes this array in disk format.
    pub fn to_disk<W: Write>(&self, mut writer: W, palette_size: usize) -> Result<()> {
        let mut bits = 0;
        for b in VALID_BITS {
            if 2usize.pow(b as u32) >= palette_size {
                bits = b;
            }
        }

        writer.write_u8(bits << 1)?;

        match self {
            BitArray::Unpacked(array) => array.to_disk(writer, bits as u32),
            BitArray::Packed(array) => array.to_disk(writer),
        }
    }

    /// Gets the value at the specified index.
    pub fn get(&self, index: usize) -> Option<u16> {
        match self {
            Self::Unpacked(array) => array.get(index),
            Self::Packed(array) => array.get(index),
        }
    }

    /// Sets the value at the specified index.
    pub fn set(&mut self, pos: usize, value: u16) -> bool {
        match self {
            Self::Unpacked(array) => array.set(pos, value),
            Self::Packed(array) => array.set(pos, value),
        }
    }
}

impl<'a> IntoIterator for &'a BitArray {
    type Item = u16;
    type IntoIter = BitArrayIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            BitArray::Packed(array) => BitArrayIter::Packed(array.into_iter()),
            BitArray::Unpacked(array) => BitArrayIter::Unpacked(array.into_iter()),
        }
    }
}
