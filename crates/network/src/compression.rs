use bedrockrs_proto_core::error::CompressionError;
use byteorder::{ReadBytesExt, WriteBytesExt};
use flate2::Compression as CompressionLevel;
use flate2::{read::DeflateDecoder, write::DeflateEncoder};
use snap::{read::FrameDecoder as SnapDecoder, write::FrameEncoder as SnapEncoder};
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
                    .map_err(|err| CompressionError::ZlibError(Box::new(err)))?;

                encoder
                    .finish()
                    .map_err(|err| CompressionError::ZlibError(Box::new(err)))?
            }
            Compression::Snappy { .. } => {
                dst.write_u8(Self::ID_SNAPPY)?;

                let mut encoder = SnapEncoder::new(dst);

                encoder
                    .write_all(src.as_slice())
                    .map_err(CompressionError::SnappyError)?;

                encoder
                    .into_inner()
                    .map_err(|err| CompressionError::SnappyError(err.into_error()))?
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

    /// Decompress the given compressed src stream into the given dst stream
    /// with the decompressed data
    #[inline]
    pub fn decompress(&self, mut src: Vec<u8>) -> Result<Vec<u8>, CompressionError> {
        let mut stream = Cursor::new(src.as_slice());

        let compression_method = stream.read_u8()?;

        src.drain(..1);

        let dst = match compression_method {
            Self::ID_ZLIB => {
                let mut dst = Vec::with_capacity(src.len());

                let mut decoder = DeflateDecoder::new(src.as_slice());
                decoder.read_to_end(&mut dst)?;

                dst
            }
            Self::ID_SNAPPY => {
                let mut dst = Vec::with_capacity(src.len());

                let mut decoder = SnapDecoder::new(src.as_slice());
                decoder.read_to_end(&mut dst)?;

                dst
            }
            Self::ID_NONE => src,
            other => return Err(CompressionError::UnknownCompressionMethod(other)),
        };

        Ok(dst)
    }
}
