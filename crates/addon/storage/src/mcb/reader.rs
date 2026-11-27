//! The bounds-checked cursor every body decoder reads through, and the helpers that build the
//! value tree.
//!
//! A body has no type tags and no lengths but those of strings and arrays, so a byte that is
//! not what the known layout has at that place is not skipped or guessed around: the whole
//! body is `UnsupportedBody`. Nothing is allocated from a count: an array grows one decoded
//! element at a time, and every element takes at least one byte of the body.
// A body is decoded only where the known layout shows what every byte is; any other body is
// refused whole.

use super::McbValue;
use crate::{McbFault, StorageError};

/// The fault of every body this reader does not know: cut short, a count or a length the
/// layout was never seen with, an unknown component, bytes after the end.
pub(super) const UNSUPPORTED: StorageError = StorageError::Mcb {
    fault: McbFault::UnsupportedBody,
};

/// What a decoder returns.
pub(super) type Decoded<T> = Result<T, StorageError>;

/// The members of an object under construction, in order.
pub(super) type Members = Vec<(Box<str>, McbValue)>;

/// A cursor over a body.
pub(super) struct Reader<'a> {
    bytes: &'a [u8],
}

impl<'a> Reader<'a> {
    /// A cursor at the start of `bytes`.
    pub(super) fn new(bytes: &'a [u8]) -> Reader<'a> {
        Reader { bytes }
    }

    /// The next `len` bytes.
    fn take(&mut self, len: usize) -> Decoded<&'a [u8]> {
        let (head, rest) = self.bytes.split_at_checked(len).ok_or(UNSUPPORTED)?;
        self.bytes = rest;
        Ok(head)
    }

    /// The next `N` bytes.
    fn array<const N: usize>(&mut self) -> Decoded<[u8; N]> {
        let (head, rest) = self.bytes.split_first_chunk::<N>().ok_or(UNSUPPORTED)?;
        self.bytes = rest;
        Ok(*head)
    }

    /// One byte.
    pub(super) fn u8(&mut self) -> Decoded<u8> {
        let [byte] = self.array()?;
        Ok(byte)
    }

    /// A boolean, an optional's presence or a two-way variant's index: `00` or `01`.
    pub(super) fn flag(&mut self) -> Decoded<bool> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(UNSUPPORTED),
        }
    }

    /// A little-endian `i32`.
    pub(super) fn i32(&mut self) -> Decoded<i32> {
        Ok(i32::from_le_bytes(self.array()?))
    }

    /// A little-endian `u32`.
    pub(super) fn u32(&mut self) -> Decoded<u32> {
        Ok(u32::from_le_bytes(self.array()?))
    }

    /// A little-endian `u16`.
    pub(super) fn u16(&mut self) -> Decoded<u16> {
        Ok(u16::from_le_bytes(self.array()?))
    }

    /// A little-endian `f32`.
    pub(super) fn f32(&mut self) -> Decoded<f32> {
        Ok(f32::from_le_bytes(self.array()?))
    }

    /// A string: its byte length as a LEB128 varint of one or two bytes (the longest vanilla
    /// string is 241 bytes; a third length byte was never seen and is refused, as is a
    /// two-byte form of a length under 128), then that many bytes of UTF-8.
    pub(super) fn string(&mut self) -> Decoded<Box<str>> {
        let low = self.u8()?;
        let len = if low < 0x80 {
            usize::from(low)
        } else {
            let high = self.u8()?;
            if high == 0 || high >= 0x80 {
                return Err(UNSUPPORTED);
            }
            usize::from(low & 0x7F) | usize::from(high) << 7
        };
        let text = std::str::from_utf8(self.take(len)?).map_err(|_| UNSUPPORTED)?;
        Ok(text.into())
    }

    /// The element count of an array or a map: one byte under 128 (the largest vanilla count
    /// is 38; whether a larger one continues as a varint was never seen, so it is refused).
    pub(super) fn count(&mut self) -> Decoded<usize> {
        match self.u8()? {
            count @ 0..0x80 => Ok(usize::from(count)),
            _ => Err(UNSUPPORTED),
        }
    }

    /// Requires the next bytes to be `bytes`: a field whose meaning no vanilla document
    /// shows, read only at the one value it was seen with.
    pub(super) fn pinned(&mut self, bytes: &[u8]) -> Decoded<()> {
        if self.take(bytes.len())? == bytes {
            Ok(())
        } else {
            Err(UNSUPPORTED)
        }
    }

    /// Requires the body to end here.
    pub(super) fn end(&self) -> Decoded<()> {
        if self.bytes.is_empty() {
            Ok(())
        } else {
            Err(UNSUPPORTED)
        }
    }

    /// `count()` elements, each read by `element`.
    pub(super) fn list(
        &mut self,
        mut element: impl FnMut(&mut Reader<'a>) -> Decoded<McbValue>,
    ) -> Decoded<McbValue> {
        let count = self.count()?;
        let mut items = Vec::new();
        for _ in 0..count {
            items.push(element(self)?);
        }
        Ok(McbValue::Array(items))
    }
}

/// A string value.
pub(super) fn string(text: Box<str>) -> McbValue {
    McbValue::String(text)
}

/// An object value.
pub(super) fn object(members: Members) -> McbValue {
    McbValue::Object(members)
}

/// Adds `key` to `members`; a key the object already has makes the body unsupported (a map
/// of an MCB document never repeats a key).
pub(super) fn insert(members: &mut Members, key: Box<str>, value: McbValue) -> Decoded<()> {
    if members.iter().any(|(existing, _)| *existing == key) {
        return Err(UNSUPPORTED);
    }
    members.push((key, value));
    Ok(())
}

impl McbValue {
    /// Whether the value is the float `default`, bit for bit (`-0.0` is not `0.0`).
    pub(super) fn is_f32(&self, default: f32) -> bool {
        matches!(self, McbValue::F32(value) if value.to_bits() == default.to_bits())
    }

    /// Whether the value is an array of exactly the floats `default`.
    pub(super) fn is_f32s(&self, default: &[f32]) -> bool {
        matches!(self, McbValue::Array(items) if items.len() == default.len()
            && items.iter().zip(default).all(|(item, default)| item.is_f32(*default)))
    }
}

/// The FNV-1a 32 hash a component is keyed by: of its full name (`minecraft:icon`).
pub(super) const fn fnv1a_32(name: &str) -> u32 {
    let bytes = name.as_bytes();
    let mut hash: u32 = 0x811c_9dc5;
    let mut index = 0;
    while index < bytes.len() {
        hash = (hash ^ bytes[index] as u32).wrapping_mul(0x0100_0193);
        index += 1;
    }
    hash
}

/// The name of `names` that `hash` is the key of.
pub(super) fn named(hash: u32, names: &[&'static str]) -> Option<&'static str> {
    names.iter().copied().find(|name| fnv1a_32(name) == hash)
}
