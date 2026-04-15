use crate::ProtoCodec;
use crate::endian::{ProtoCodecBE, ProtoCodecLE, ProtoCodecVAR};
use crate::error::ProtoCodecError;
use seq_macro::seq;
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
    pub fn initialize<F: Fn() -> Result<T, ProtoCodecError>>(
        &mut self,
        op: F,
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

macro_rules! impl_proto_slice {
    ($name:ident, 0) => {
        impl<T> $name for [T; 0] {
            fn serialize<W: Write>(&self, _stream: &mut W) -> Result<(), ProtoCodecError> {
                Ok(())
            }

            fn deserialize<R: Read>(_stream: &mut R) -> Result<Self, ProtoCodecError> {
                Ok([])
            }

            fn size_hint(&self) -> usize {
                0
            }
        }
    };
    ($name:ident, $size:literal) => {
        impl<T: $name> $name for [T; $size] {
            fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
                seq!(N in 0..$size {
                    self[N].serialize(stream)?;
                });

                Ok(())
            }

            fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
                seq!(N in 0..$size {
                    let buf = [
                        #( T::deserialize(stream)?, )*
                    ];
                });

                Ok(buf)
            }

            fn size_hint(&self) -> usize {
                self.iter().fold(0, |acc, x| acc + x.size_hint())
            }
        }
    };
}

impl_proto_slice!(ProtoCodecLE, 0);
impl_proto_slice!(ProtoCodecLE, 1);
impl_proto_slice!(ProtoCodecLE, 2);
impl_proto_slice!(ProtoCodecLE, 3);
impl_proto_slice!(ProtoCodecLE, 4);
impl_proto_slice!(ProtoCodecLE, 5);
impl_proto_slice!(ProtoCodecLE, 6);
impl_proto_slice!(ProtoCodecLE, 7);
impl_proto_slice!(ProtoCodecLE, 8);
impl_proto_slice!(ProtoCodecLE, 9);
impl_proto_slice!(ProtoCodecLE, 10);
impl_proto_slice!(ProtoCodecLE, 11);
impl_proto_slice!(ProtoCodecLE, 12);

impl_proto_slice!(ProtoCodecBE, 0);
impl_proto_slice!(ProtoCodecBE, 1);
impl_proto_slice!(ProtoCodecBE, 2);
impl_proto_slice!(ProtoCodecBE, 3);
impl_proto_slice!(ProtoCodecBE, 4);
impl_proto_slice!(ProtoCodecBE, 5);
impl_proto_slice!(ProtoCodecBE, 6);
impl_proto_slice!(ProtoCodecBE, 7);
impl_proto_slice!(ProtoCodecBE, 8);
impl_proto_slice!(ProtoCodecBE, 9);
impl_proto_slice!(ProtoCodecBE, 10);
impl_proto_slice!(ProtoCodecBE, 11);
impl_proto_slice!(ProtoCodecBE, 12);

impl_proto_slice!(ProtoCodecVAR, 0);
impl_proto_slice!(ProtoCodecVAR, 1);
impl_proto_slice!(ProtoCodecVAR, 2);
impl_proto_slice!(ProtoCodecVAR, 3);
impl_proto_slice!(ProtoCodecVAR, 4);
impl_proto_slice!(ProtoCodecVAR, 5);
impl_proto_slice!(ProtoCodecVAR, 6);
impl_proto_slice!(ProtoCodecVAR, 7);
impl_proto_slice!(ProtoCodecVAR, 8);
impl_proto_slice!(ProtoCodecVAR, 9);
impl_proto_slice!(ProtoCodecVAR, 10);
impl_proto_slice!(ProtoCodecVAR, 11);
impl_proto_slice!(ProtoCodecVAR, 12);
