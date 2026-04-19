use crate::ProtoCodec;
use crate::endian::{ProtoCodecBE, ProtoCodecLE, ProtoCodecVAR};
use crate::error::ProtoCodecError;
use std::io::{Read, Write};
use std::mem::MaybeUninit;

/// Manages partially initialized arrays and drops items on panic or error.
struct ArrayInitGuard<T, const N: usize> {
    /// The amount of items that have already been initialized.
    init: usize,
    /// The partially initialized array.
    array: [MaybeUninit<T>; N],
}

impl<T, const N: usize> ArrayInitGuard<T, N> {
    /// Creates a new, fully uninitialized array.
    pub const fn new() -> Self {
        Self {
            init: 0,
            array: [const { MaybeUninit::<T>::uninit() }; N],
        }
    }

    /// Attempts to initialize an array on the stack, using a fallible `op`.
    ///
    /// If a call to `op` fails or panics, this function will return the error and all previously initialized
    /// items will be dropped.
    pub fn initialize<F: FnMut() -> Result<T, ProtoCodecError>>(
        &mut self,
        mut op: F,
    ) -> Result<[T; N], ProtoCodecError> {
        self.array.iter_mut().try_for_each(|x| {
            let init = op()?;
            x.write(init);

            self.init += 1;
            Ok::<_, ProtoCodecError>(())
        })?;

        // Set `init` to 0 to ensure that the guard does not drop anything when it is destroyed.
        self.init = 0;

        // Safety: `[MaybeUninit<T>; N]` and `[T; N]` have the same layout and we know all
        // items have been initialized.
        Ok(unsafe { (self.array.as_ptr() as *const [T; N]).read() })
    }
}

impl<T, const N: usize> Drop for ArrayInitGuard<T, N> {
    fn drop(&mut self) {
        let init = std::mem::take(&mut self.init);

        // Safety: This is safe since we know that all items up to `init` have been initialized.
        // If the array has fully been initialized, `init` will be set to zero and this code does not run.
        self.array[..init]
            .iter_mut()
            .for_each(|x| unsafe { x.assume_init_drop() })
    }
}

impl<T: ProtoCodec, const N: usize> ProtoCodec for [T; N] {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        self.iter().try_for_each(|x| x.serialize(stream))
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        // FIXME: We can use the safe equivalent below when the `array_try_from_fn` feature is stabilised.
        // std::array::try_from_fn(|| T::deserialize(stream))

        let mut guard = const { ArrayInitGuard::new() };
        guard.initialize(|| T::deserialize(stream))
    }

    fn size_hint(&self) -> usize {
        self.iter().fold(0, |acc, x| acc + x.size_hint())
    }
}

impl<T: ProtoCodecLE, const N: usize> ProtoCodecLE for [T; N] {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        self.iter().try_for_each(|x| x.serialize(stream))
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        // FIXME: We can use the safe equivalent below when the `array_try_from_fn` feature is stabilised.
        // std::array::try_from_fn(|| T::deserialize(stream))

        let mut guard = const { ArrayInitGuard::new() };
        guard.initialize(|| T::deserialize(stream))
    }

    fn size_hint(&self) -> usize {
        self.iter().fold(0, |acc, x| acc + x.size_hint())
    }
}

impl<T: ProtoCodecBE, const N: usize> ProtoCodecBE for [T; N] {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        self.iter().try_for_each(|x| x.serialize(stream))
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        // FIXME: We can use the safe equivalent below when the `array_try_from_fn` feature is stabilised.
        // std::array::try_from_fn(|| T::deserialize(stream))

        let mut guard = const { ArrayInitGuard::new() };
        guard.initialize(|| T::deserialize(stream))
    }

    fn size_hint(&self) -> usize {
        self.iter().fold(0, |acc, x| acc + x.size_hint())
    }
}

impl<T: ProtoCodecVAR, const N: usize> ProtoCodecVAR for [T; N] {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        self.iter().try_for_each(|x| x.serialize(stream))
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        // FIXME: We can use the safe equivalent below when the `array_try_from_fn` feature is stabilised.
        // std::array::try_from_fn(|| T::deserialize(stream))

        let mut guard = const { ArrayInitGuard::new() };
        guard.initialize(|| T::deserialize(stream))
    }

    fn size_hint(&self) -> usize {
        self.iter().fold(0, |acc, x| acc + x.size_hint())
    }
}
