use crate::error::CompressionError;
use byteorder::{ReadBytesExt, WriteBytesExt};
use flate2::Compression as CompressionLevel;
use flate2::{read::DeflateDecoder, write::DeflateEncoder};
use std::io::{Cursor, Read, Write};
use std::mem::size_of;

#[derive(Debug, Clone)]
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

    /// Used in the [NetworkSettingsPacket](crate::version::v729::packets::network_settings::NetworkSettingsPacket)
    /// to identify which Compression should be used for the Connection.
    #[inline]
    pub const fn id_u16(&self) -> u16 {
        match self {
            Compression::Zlib { .. } => 0,
            Compression::Snappy { .. } => 1,
            Compression::None => u16::MAX,
        }
    }

    /// Get the compression threshold of the Compression.
    #[inline]
    pub fn threshold(&self) -> u16 {
        match self {
            Compression::Zlib { threshold, .. } => *threshold,
            Compression::Snappy { threshold } => *threshold,
            Compression::None => 0,
        }
    }

    /// Compress the given uncompressed src stream into the given dst stream
    /// with the compressed data.
    #[inline]
    pub fn compress(&self, src: Vec<u8>) -> Result<Vec<u8>, CompressionError> {
        // Add one extra byte for the compression method id
        let mut dst = Vec::with_capacity(src.len() + size_of::<u8>());

        if self.threshold() as usize >= src.len() {
            dst.write_u8(Self::ID_NONE)?;
            dst.write_all(src.as_slice())?;

            return Ok(dst);
        }

        let dst = match self {
            Compression::Zlib {
                compression_level, ..
            } => {
                dst.write_u8(Self::ID_ZLIB)?;

                let mut encoder =
                    DeflateEncoder::new(dst, CompressionLevel::new(*compression_level as u32));

                encoder
                    .write_all(src.as_slice())
                    .map_err(CompressionError::ZlibError)?;

                encoder.finish().map_err(CompressionError::ZlibError)?
            }
            Compression::Snappy { .. } => {
                dst.write_u8(Self::ID_SNAPPY)?;

                let compressed = snap::raw::Encoder::new()
                    .compress_vec(src.as_slice())
                    .map_err(|err| CompressionError::SnappyError(err.into()))?;
                dst.extend_from_slice(&compressed);

                dst
            }
            Compression::None => {
                // Compression method id for No Compression
                dst.write_u8(Self::ID_NONE)?;
                dst.write_all(src.as_slice())?;

                dst
            }
        };

        Ok(dst)
    }

    /// Decompress the given compressed src stream, failing if it inflates past `max_len` bytes.
    #[inline]
    pub fn decompress(
        &self,
        mut src: Vec<u8>,
        max_len: usize,
    ) -> Result<Vec<u8>, CompressionError> {
        let mut stream = Cursor::new(src.as_slice());

        let compression_method = stream.read_u8()?;

        src.drain(..1);

        let dst = match compression_method {
            Self::ID_ZLIB => {
                Self::read_limited(DeflateDecoder::new(src.as_slice()), src.len(), max_len)?
            }
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
            return Err(CompressionError::TooLarge(max_len));
        }
        snap::raw::Decoder::new()
            .decompress_vec(src)
            .map_err(|err| CompressionError::SnappyError(err.into()))
    }

    fn read_limited(
        decoder: impl Read,
        compressed_len: usize,
        max_len: usize,
    ) -> Result<Vec<u8>, CompressionError> {
        let mut dst = Vec::with_capacity(compressed_len.min(max_len));
        decoder.take(max_len as u64 + 1).read_to_end(&mut dst)?;
        if dst.len() > max_len {
            return Err(CompressionError::TooLarge(max_len));
        }
        Ok(dst)
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
    fn zlib_batch_inflating_past_the_limit_is_rejected() {
        let bomb = zlib().compress(vec![0u8; MAX_LEN + 1]).unwrap();
        let result = zlib().decompress(bomb, MAX_LEN);
        assert!(
            matches!(result, Err(CompressionError::TooLarge(MAX_LEN))),
            "got {:?}",
            result.map(|v| v.len())
        );
    }

    #[test]
    fn zlib_batch_at_the_limit_round_trips() {
        let batch = vec![7u8; MAX_LEN];
        let compressed = zlib().compress(batch.clone()).unwrap();
        assert_eq!(zlib().decompress(compressed, MAX_LEN).unwrap(), batch);
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
            .compress(batch.clone())
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
        let bomb = snappy.compress(vec![0u8; MAX_LEN + 1]).unwrap();
        let result = snappy.decompress(bomb, MAX_LEN);
        assert!(
            matches!(result, Err(CompressionError::TooLarge(MAX_LEN))),
            "got {:?}",
            result.map(|v| v.len())
        );
    }
}
