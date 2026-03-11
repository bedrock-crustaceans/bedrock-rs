use crate::error::Result;
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

    pub fn to_disk<W: Write>(&self, mut writer: W, bits: u32) -> Result<()> {
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
        let mask = !(!0u32 << bits);

        let mut indices = Box::new([0u16; 4096]);
        let mut offset = 0;

        let mut words = vec![0; word_count as usize];
        reader.read_exact(bytemuck::cast_slice_mut::<u32, u8>(&mut words))?;

        for mut word in words {
            for _ in 0..per_word {
                if offset == 4096 {
                    break;
                }

                indices[offset] = (word & mask) as u16;
                word >>= bits;
                offset += 1;
            }
        }

        Ok(Self { array: indices })
    }
}

impl<'a> IntoIterator for &'a UnpackedArray {
    type Item = u16;
    type IntoIter = std::iter::Copied<std::slice::Iter<'a, u16>>;

    fn into_iter(self) -> Self::IntoIter {
        self.array.iter().copied()
    }
}
