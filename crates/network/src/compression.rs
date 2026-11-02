use crate::error::CompressionError;
use byteorder::{ReadBytesExt, WriteBytesExt};
use flate2::Compression as CompressionLevel;
use flate2::{Compress, Decompress, FlushCompress, FlushDecompress, Status};
use std::cell::RefCell;
use std::io::{Cursor, Write};
use std::mem::size_of;

thread_local! {
    static INFLATE: RefCell<Decompress> = RefCell::new(Decompress::new(false));
    static DEFLATE: RefCell<Option<(u32, Compress)>> = const { RefCell::new(None) };
    static SNAPPY: RefCell<snap::raw::Encoder> = RefCell::new(snap::raw::Encoder::new());
}

#[derive(Debug, Clone, PartialEq)]
pub enum Compression {
    Zlib {
        threshold: u16,
        /// Needs to be a number between 0 and 9.
        /// Indicates how compressed the data becomes.
        ///
        /// - 0 = None
        /// - 1 = Fastest
        /// - 6 = Default
        /// - 9 = Best
        compression_level: u8,
    },
    Snappy {
        threshold: u16,
    },
    None,
}

impl Compression {
    const ID_ZLIB: u8 = 0;
    const ID_SNAPPY: u8 = 1;
    const ID_NONE: u8 = u8::MAX;
    const ALGORITHM_ZLIB: u16 = 0;
    const ALGORITHM_SNAPPY: u16 = 1;
    const ALGORITHM_NONE: u16 = u16::MAX;

    /// Used in the [`NetworkSettingsPacket`](crate::version::v729::packets::network_settings::NetworkSettingsPacket)
    /// to identify which `Compression` should be used for the `Connection`.
    #[inline]
    #[must_use]
    pub const fn id_u16(&self) -> u16 {
        match self {
            Compression::Zlib { .. } => Self::ALGORITHM_ZLIB,
            Compression::Snappy { .. } => Self::ALGORITHM_SNAPPY,
            Compression::None => Self::ALGORITHM_NONE,
        }
    }

    /// Builds the compression a `NetworkSettings` packet negotiates from its `algorithm` id and
    /// `threshold`.
    ///
    /// # Errors
    ///
    /// Returns [`CompressionError::UnknownNetworkAlgorithm`] when `algorithm` is not zlib, snappy
    /// or none.
    pub fn from_network_settings(algorithm: u16, threshold: u16) -> Result<Self, CompressionError> {
        match algorithm {
            Self::ALGORITHM_ZLIB => Ok(Compression::Zlib {
                threshold,
                compression_level: 6,
            }),
            Self::ALGORITHM_SNAPPY => Ok(Compression::Snappy { threshold }),
            Self::ALGORITHM_NONE => Ok(Compression::None),
            other => Err(CompressionError::UnknownNetworkAlgorithm(other)),
        }
    }

    #[must_use]
    pub fn id(&self) -> u8 {
        match self {
            Compression::Zlib { .. } => Self::ID_ZLIB,
            Compression::Snappy { .. } => Self::ID_SNAPPY,
            Compression::None => Self::ID_NONE,
        }
    }

    /// Get the compression threshold of the Compression.
    #[inline]
    #[must_use]
    pub fn threshold(&self) -> u16 {
        match self {
            Compression::Zlib { threshold, .. } | Compression::Snappy { threshold } => *threshold,
            Compression::None => 0,
        }
    }

    /// Compress the given uncompressed src stream into the given dst stream
    /// with the compressed data.
    ///
    /// # Errors
    ///
    /// Returns a [`CompressionError`] when writing the batch or the zlib or snappy encoder fails.
    #[inline]
    pub fn compress(&self, src: &[u8]) -> Result<Vec<u8>, CompressionError> {
        // Add one extra byte for the compression method id
        let mut dst = Vec::with_capacity(src.len() + size_of::<u8>());

        if self.threshold() as usize >= src.len() {
            dst.write_u8(Self::ID_NONE)?;
            dst.write_all(src)?;

            return Ok(dst);
        }

        let dst = match self {
            Compression::Zlib {
                compression_level, ..
            } => {
                dst.write_u8(Self::ID_ZLIB)?;

                Self::deflate(src, u32::from(*compression_level), dst)?
            }
            Compression::Snappy { .. } => {
                dst.write_u8(Self::ID_SNAPPY)?;

                let start = dst.len();
                dst.resize(start + snap::raw::max_compress_len(src.len()), 0);
                let written = SNAPPY
                    .with_borrow_mut(|encoder| encoder.compress(src, &mut dst[start..]))
                    .map_err(|err| CompressionError::SnappyError(err.into()))?;
                dst.truncate(start + written);

                dst
            }
            Compression::None => {
                // Compression method id for No Compression
                dst.write_u8(Self::ID_NONE)?;
                dst.write_all(src)?;

                dst
            }
        };

        Ok(dst)
    }

    /// Decompress the given compressed src stream, failing if it inflates past `max_len` bytes.
    ///
    /// # Errors
    ///
    /// Returns a [`CompressionError`] when the batch is empty, its method id is unknown or differs
    /// from this compression, it fails to inflate, or it inflates past `max_len`.
    #[inline]
    pub fn decompress(
        &self,
        mut src: Vec<u8>,
        max_len: usize,
    ) -> Result<Vec<u8>, CompressionError> {
        let mut stream = Cursor::new(src.as_slice());

        let compression_method = stream.read_u8()?;

        src.drain(..1);

        if compression_method != Self::ID_NONE && compression_method != self.id() {
            return Err(CompressionError::UnexpectedMethod {
                negotiated: self.id(),
                found: compression_method,
            });
        }

        let dst = match compression_method {
            Self::ID_ZLIB => Self::inflate_limited(&src, max_len)?,
            Self::ID_SNAPPY => Self::decompress_snappy(&src, max_len)?,
            Self::ID_NONE => src,
            other => return Err(CompressionError::UnknownCompressionMethod(other)),
        };

        Ok(dst)
    }

    fn decompress_snappy(src: &[u8], max_len: usize) -> Result<Vec<u8>, CompressionError> {
        let len = snap::raw::decompress_len(src)
            .map_err(|err| CompressionError::SnappyError(err.into()))?;
        if len > max_len {
            return Err(CompressionError::DecompressedSizeExceeded(max_len));
        }
        snap::raw::Decoder::new()
            .decompress_vec(src)
            .map_err(|err| CompressionError::SnappyError(err.into()))
    }

    fn deflate(src: &[u8], level: u32, mut dst: Vec<u8>) -> Result<Vec<u8>, CompressionError> {
        DEFLATE.with_borrow_mut(|slot| {
            match slot {
                Some((current, compress)) if *current == level => compress.reset(),
                _ => *slot = Some((level, Compress::new(CompressionLevel::new(level), false))),
            }
            let (_, compress) = slot.as_mut().expect("the compressor was just set");
            loop {
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "total_in never exceeds src.len(), which is a usize"
                )]
                let consumed = compress.total_in() as usize;
                let status = compress
                    .compress_vec(&src[consumed..], &mut dst, FlushCompress::Finish)
                    .map_err(|err| CompressionError::ZlibError(err.into()))?;
                if status == Status::StreamEnd {
                    return Ok(dst);
                }
                dst.reserve(dst.capacity().max(64));
            }
        })
    }

    fn inflate_limited(src: &[u8], max_len: usize) -> Result<Vec<u8>, CompressionError> {
        INFLATE.with_borrow_mut(|inflate| {
            inflate.reset(false);
            let mut dst = Vec::with_capacity(src.len().min(max_len));
            loop {
                let (in_before, out_before) = (inflate.total_in(), inflate.total_out());
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "total_in never exceeds src.len(), which is a usize"
                )]
                let consumed = in_before as usize;
                let status = inflate
                    .decompress_vec(&src[consumed..], &mut dst, FlushDecompress::None)
                    .map_err(|err| CompressionError::ZlibError(err.into()))?;
                if dst.len() > max_len {
                    return Err(CompressionError::DecompressedSizeExceeded(max_len));
                }
                if status == Status::StreamEnd {
                    return Ok(dst);
                }
                let progressed =
                    inflate.total_in() != in_before || inflate.total_out() != out_before;
                if dst.len() == dst.capacity() {
                    dst.reserve(
                        dst.capacity()
                            .clamp(1, max_len.saturating_add(1) - dst.len()),
                    );
                } else if !progressed {
                    return Err(CompressionError::ZlibError(
                        std::io::ErrorKind::UnexpectedEof.into(),
                    ));
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MAX_LEN: usize = 1024;

    fn zlib() -> Compression {
        Compression::Zlib {
            threshold: 0,
            compression_level: 9,
        }
    }

    #[test]
    fn zlib_batch_decompresses_with_no_limit() {
        let batch = vec![7u8; 4096];
        let compressed = zlib().compress(&batch).unwrap();

        assert_eq!(zlib().decompress(compressed, usize::MAX).unwrap(), batch);
    }

    #[test]
    fn snappy_batch_decompresses_with_no_limit() {
        let snappy = Compression::Snappy { threshold: 0 };
        let batch = vec![7u8; 4096];
        let compressed = snappy.compress(&batch).unwrap();

        assert_eq!(snappy.decompress(compressed, usize::MAX).unwrap(), batch);
    }

    #[test]
    fn zlib_batch_inflating_past_the_limit_is_rejected() {
        let bomb = zlib().compress(&vec![0u8; MAX_LEN + 1]).unwrap();
        let result = zlib().decompress(bomb, MAX_LEN);
        assert!(
            matches!(
                result,
                Err(CompressionError::DecompressedSizeExceeded(MAX_LEN))
            ),
            "got {:?}",
            result.map(|v| v.len())
        );
    }

    #[test]
    fn zlib_batch_at_the_limit_round_trips() {
        let batch = vec![7u8; MAX_LEN];
        let compressed = zlib().compress(&batch).unwrap();
        assert_eq!(zlib().decompress(compressed, MAX_LEN).unwrap(), batch);
    }

    #[test]
    fn zlib_decoder_state_does_not_leak_between_batches() {
        let sizes = [10, 100_000, 3, 70_000, 1, 4096];
        for (round, size) in sizes.into_iter().enumerate() {
            let batch: Vec<u8> = (0..size)
                .map(|i| u8::try_from((i * 31 + round) % 256).unwrap())
                .collect();
            let compressed = zlib().compress(&batch).unwrap();
            assert_eq!(
                zlib().decompress(compressed, usize::MAX).unwrap(),
                batch,
                "size {size}"
            );
        }
    }

    #[test]
    fn zlib_truncated_batch_is_an_error_and_the_next_batch_still_decodes() {
        let batch = vec![9u8; 5000];
        let compressed = zlib().compress(&batch).unwrap();
        let truncated = compressed[..compressed.len() - 3].to_vec();
        assert!(matches!(
            zlib().decompress(truncated, usize::MAX),
            Err(CompressionError::ZlibError(_))
        ));
        assert_eq!(zlib().decompress(compressed, usize::MAX).unwrap(), batch);
    }

    #[test]
    fn zlib_corrupt_batch_is_an_error_and_the_next_batch_still_decodes() {
        let batch = b"bedrock batch bedrock batch bedrock batch".to_vec();
        let compressed = zlib().compress(&batch).unwrap();
        let mut corrupt = vec![0u8, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
        corrupt.extend_from_slice(&compressed[1..]);
        assert!(zlib().decompress(corrupt, usize::MAX).is_err());
        assert_eq!(zlib().decompress(compressed, usize::MAX).unwrap(), batch);
    }

    #[test]
    fn zlib_limit_is_enforced_after_the_buffer_has_grown() {
        let bomb = zlib().compress(&vec![0u8; 1 << 20]).unwrap();
        assert!(matches!(
            zlib().decompress(bomb, 5000),
            Err(CompressionError::DecompressedSizeExceeded(5000))
        ));
    }

    #[test]
    fn zlib_encoder_state_does_not_leak_between_batches() {
        for (round, size) in [10usize, 100_000, 3, 70_000, 1, 4096, 300_000]
            .into_iter()
            .enumerate()
        {
            let batch: Vec<u8> = (0..size)
                .map(|i| u8::try_from((i * 31 + round) % 256).unwrap())
                .collect();
            let compressed = zlib().compress(&batch).unwrap();
            assert_eq!(compressed[0], 0);
            assert_eq!(
                zlib().decompress(compressed, usize::MAX).unwrap(),
                batch,
                "size {size}"
            );
        }
    }

    #[test]
    fn zlib_encoder_follows_a_level_change_on_the_same_thread() {
        let batch = vec![3u8; 50_000];
        for level in [9u8, 1, 6, 9] {
            let compression = Compression::Zlib {
                threshold: 0,
                compression_level: level,
            };
            let compressed = compression.compress(&batch).unwrap();
            assert_eq!(
                compression.decompress(compressed, usize::MAX).unwrap(),
                batch,
                "level {level}"
            );
        }
    }

    #[test]
    fn snappy_encoder_state_does_not_leak_between_batches() {
        let snappy = Compression::Snappy { threshold: 0 };
        for (round, size) in [10usize, 100_000, 3, 70_000, 1, 4096]
            .into_iter()
            .enumerate()
        {
            let batch: Vec<u8> = (0..size)
                .map(|i| u8::try_from(((i / 3) * 31 + round) % 256).unwrap())
                .collect();
            let compressed = snappy.compress(&batch).unwrap();
            assert_eq!(compressed[0], 1);
            assert_eq!(
                snappy.decompress(compressed, usize::MAX).unwrap(),
                batch,
                "size {size}"
            );
        }
    }

    #[test]
    fn snappy_decodes_a_raw_block() {
        let batch = b"bedrock batch bedrock batch bedrock batch".to_vec();
        let mut wire = vec![1u8];
        wire.extend(snap::raw::Encoder::new().compress_vec(&batch).unwrap());
        assert_eq!(
            Compression::Snappy { threshold: 0 }
                .decompress(wire, MAX_LEN)
                .unwrap(),
            batch
        );
    }

    #[test]
    fn snappy_encodes_a_raw_block() {
        let batch = b"bedrock batch bedrock batch bedrock batch".to_vec();
        let wire = Compression::Snappy { threshold: 0 }
            .compress(&batch)
            .unwrap();
        assert_eq!(wire[0], 1);
        assert_eq!(
            snap::raw::Decoder::new()
                .decompress_vec(&wire[1..])
                .unwrap(),
            batch
        );
    }

    #[test]
    fn snappy_batch_inflating_past_the_limit_is_rejected() {
        let snappy = Compression::Snappy { threshold: 0 };
        let bomb = snappy.compress(&vec![0u8; MAX_LEN + 1]).unwrap();
        let result = snappy.decompress(bomb, MAX_LEN);
        assert!(
            matches!(
                result,
                Err(CompressionError::DecompressedSizeExceeded(MAX_LEN))
            ),
            "got {:?}",
            result.map(|v| v.len())
        );
    }

    #[test]
    fn network_settings_ids_map_to_compression() {
        assert!(matches!(
            Compression::from_network_settings(0, 256),
            Ok(Compression::Zlib {
                threshold: 256,
                compression_level: 6
            })
        ));
        assert!(matches!(
            Compression::from_network_settings(1, 7),
            Ok(Compression::Snappy { threshold: 7 })
        ));
        assert!(matches!(
            Compression::from_network_settings(0xffff, 9),
            Ok(Compression::None)
        ));
        assert!(matches!(
            Compression::from_network_settings(2, 0),
            Err(CompressionError::UnknownNetworkAlgorithm(2))
        ));
        assert_eq!(zlib().id_u16(), 0);
        assert_eq!(Compression::Snappy { threshold: 0 }.id_u16(), 1);
        assert_eq!(Compression::None.id_u16(), 0xffff);
        assert_eq!(zlib().id(), 0);
        assert_eq!(Compression::Snappy { threshold: 0 }.id(), 1);
        assert_eq!(Compression::None.id(), 0xff);
    }

    #[test]
    fn unknown_network_algorithm_keeps_its_full_id() {
        assert!(matches!(
            Compression::from_network_settings(256, 0),
            Err(CompressionError::UnknownNetworkAlgorithm(256))
        ));
    }

    #[test]
    fn snappy_batch_on_zlib_connection_is_rejected() {
        let snappy_batch = Compression::Snappy { threshold: 0 }
            .compress(&[7u8; 64])
            .unwrap();
        let result = zlib().decompress(snappy_batch, MAX_LEN);
        assert!(
            matches!(
                result,
                Err(CompressionError::UnexpectedMethod {
                    negotiated: 0,
                    found: 1
                })
            ),
            "got {:?}",
            result.map(|v| v.len())
        );
    }

    #[test]
    fn uncompressed_marker_is_accepted_on_any_connection() {
        let mut wire = vec![0xffu8];
        wire.extend_from_slice(b"plain");
        for compression in [
            zlib(),
            Compression::Snappy { threshold: 0 },
            Compression::None,
        ] {
            assert_eq!(
                compression.decompress(wire.clone(), MAX_LEN).unwrap(),
                b"plain"
            );
        }
    }

    #[test]
    fn batch_at_threshold_is_not_compressed() {
        let compression = Compression::Zlib {
            threshold: 8,
            compression_level: 6,
        };
        let at = compression.compress(&[1u8; 8]).unwrap();
        assert_eq!(at[0], 0xff);
        let above = compression.compress(&[1u8; 9]).unwrap();
        assert_eq!(above[0], 0);
    }
}
