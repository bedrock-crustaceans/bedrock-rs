//! Image files as the texture rules need them: the format by extension and the size and
//! channel count from the file's header. No pixel is decoded.

/// The format of an image file, by its extension. The order is the order vanilla tries the
/// extensions in for a path without one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum ImageFormat {
    /// `.tga`.
    Tga,
    /// `.png`.
    Png,
    /// `.jpg`.
    Jpg,
    /// `.jpeg`.
    Jpeg,
    /// `.hdr`.
    Hdr,
    /// `.ktx`.
    Ktx,
}

impl ImageFormat {
    /// Every format, in the game's lookup order (`.tga .png .jpg .jpeg .hdr .ktx`).
    pub const ALL: [ImageFormat; 6] = [
        ImageFormat::Tga,
        ImageFormat::Png,
        ImageFormat::Jpg,
        ImageFormat::Jpeg,
        ImageFormat::Hdr,
        ImageFormat::Ktx,
    ];

    /// The extension, with its dot.
    #[must_use]
    pub fn extension(self) -> &'static str {
        match self {
            Self::Tga => ".tga",
            Self::Png => ".png",
            Self::Jpg => ".jpg",
            Self::Jpeg => ".jpeg",
            Self::Hdr => ".hdr",
            Self::Ktx => ".ktx",
        }
    }

    /// The format of a file path, by its extension compared byte for byte (vanilla finds no
    /// `.PNG`).
    #[must_use]
    pub fn from_path(path: &str) -> Option<ImageFormat> {
        Self::ALL
            .into_iter()
            .find(|format| path.ends_with(format.extension()))
    }

    /// The format of a file path whose extension matches in any case.
    pub fn from_path_ignoring_case(path: &str) -> Option<ImageFormat> {
        let lower = path.to_ascii_lowercase();
        Self::from_path(&lower)
    }
}

/// What an image file's header says.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct ImageInfo {
    /// The width in pixels.
    pub width: u32,
    /// The height in pixels.
    pub height: u32,
    /// The number of channels of the decoded image, when the header gives it: a
    /// colour-mapped PNG has its palette's three, a `tRNS` chunk adds the alpha channel to a
    /// colour-mapped, grey or true-colour PNG, and a TGA has the four the game decodes it to.
    pub channels: Option<u8>,
}

impl ImageInfo {
    /// The header of `bytes` read as `format`; `None` when it is not such a file.
    #[must_use]
    pub fn read(format: ImageFormat, bytes: &[u8]) -> Option<ImageInfo> {
        match format {
            ImageFormat::Png => png(bytes),
            ImageFormat::Tga => tga(bytes),
            ImageFormat::Jpg | ImageFormat::Jpeg => jpeg(bytes),
            ImageFormat::Hdr => hdr(bytes),
            ImageFormat::Ktx => ktx(bytes),
        }
    }
}

/// A big-endian `u32` at `at`.
fn be32(bytes: &[u8], at: usize) -> Option<u32> {
    let word = bytes.get(at..at + 4)?;
    Some(u32::from_be_bytes(word.try_into().ok()?))
}

/// A little-endian `u32` at `at`.
fn le32(bytes: &[u8], at: usize) -> Option<u32> {
    let word = bytes.get(at..at + 4)?;
    Some(u32::from_le_bytes(word.try_into().ok()?))
}

/// A 16-bit value at `at`.
fn word16(bytes: &[u8], at: usize, big: bool) -> Option<u32> {
    let pair: [u8; 2] = bytes.get(at..at + 2)?.try_into().ok()?;
    Some(u32::from(if big {
        u16::from_be_bytes(pair)
    } else {
        u16::from_le_bytes(pair)
    }))
}

/// PNG: the signature, then `IHDR` with the size and the colour type. The count is the
/// decoded image's: a colour-mapped image (colour type 3) has its palette's three channels,
/// and a `tRNS` chunk before the image data adds the alpha channel to a colour-mapped, a
/// grey or a true-colour image (four, two and four channels).
// PNG (ISO/IEC 15948) 11.2.2 and 11.3.2.1: a palette entry is three samples; `tRNS` gives a
// palette an alpha per entry and a grey or true-colour image one transparent colour, which a
// decoder hands back as an alpha channel. The game's own packs put colour-mapped images on
// `color`, which takes three or four channels. None of the game's PNGs is grey or true
// colour with a `tRNS` chunk, so that the game counts the chunk there, as a decoder does, is
// inferred.
fn png(bytes: &[u8]) -> Option<ImageInfo> {
    let header = crate::png::header(bytes)?;
    let color_type = header.color_type;
    let samples = match color_type {
        0 => 1,
        4 => 2,
        2 | 3 => 3,
        6 => 4,
        _ => return None,
    };
    // The types 4 and 6 carry their alpha in the samples and take no `tRNS`.
    let alpha = matches!(color_type, 0 | 2 | 3) && has_transparency(bytes);
    Some(ImageInfo {
        width: header.width,
        height: header.height,
        channels: Some(samples + u8::from(alpha)),
    })
}

/// Whether a PNG has a `tRNS` chunk before its image data. The chunks are walked by their
/// lengths from the one after the signature; bytes that end early have none.
fn has_transparency(bytes: &[u8]) -> bool {
    let mut at = 8usize;
    while let Some(header) = bytes.get(at..).and_then(|rest| rest.get(..8)) {
        match &header[4..] {
            b"tRNS" => return true,
            b"IDAT" | b"IEND" => return false,
            _ => {}
        }
        let length = u32::from_be_bytes([header[0], header[1], header[2], header[3]]);
        let Some(next) = next_chunk(at, length) else {
            return false;
        };
        at = next;
    }
    false
}

/// Where the chunk after the one at `at` with `length` bytes of data starts: past its length,
/// type, data and CRC. `None` when that is beyond what an offset can hold.
fn next_chunk(at: usize, length: u32) -> Option<usize> {
    at.checked_add(usize::try_from(length).ok()?)?
        .checked_add(12)
}

/// TGA: an 18-byte header with the image type, the size and the bits per pixel. The game
/// decodes every TGA to four channels, whatever depth the header states (inferred for all
/// but a true-colour image of 24 bits).
// The game's own packs put 285 true-colour TGAs of 24 bits on
// `metalness_emissive_roughness_subsurface`, the one layer that takes four channels only, and
// a colour-mapped one of 8-bit indices and 24-bit entries on `normal`, which takes three or
// four; the game draws them all. So its reader does not hand on the depth of the file. None
// of the game's TGAs is grey or of 15 or 16 bits.
fn tga(bytes: &[u8]) -> Option<ImageInfo> {
    let kind = *bytes.get(2)?;
    if !matches!(kind, 1 | 2 | 3 | 9 | 10 | 11) {
        return None;
    }
    if !matches!(*bytes.get(16)?, 8 | 15 | 16 | 24 | 32) {
        return None;
    }
    Some(ImageInfo {
        width: word16(bytes, 12, false)?,
        height: word16(bytes, 14, false)?,
        channels: Some(4),
    })
}

/// JPEG: the first start-of-frame segment.
fn jpeg(bytes: &[u8]) -> Option<ImageInfo> {
    if !bytes.starts_with(&[0xFF, 0xD8]) {
        return None;
    }
    let mut at = 2;
    loop {
        if *bytes.get(at)? != 0xFF {
            return None;
        }
        let marker = *bytes.get(at + 1)?;
        match marker {
            // Fill bytes and markers without a length.
            0xFF => at += 1,
            0x01 | 0xD0..=0xD8 => at += 2,
            0xC0..=0xCF if !matches!(marker, 0xC4 | 0xC8 | 0xCC) => {
                return Some(ImageInfo {
                    height: word16(bytes, at + 5, true)?,
                    width: word16(bytes, at + 7, true)?,
                    channels: Some(*bytes.get(at + 9)?),
                });
            }
            _ => at += 2 + usize::try_from(word16(bytes, at + 2, true)?).ok()?,
        }
    }
}

/// Radiance HDR: the text header up to an empty line, then the resolution line.
fn hdr(bytes: &[u8]) -> Option<ImageInfo> {
    if !bytes.starts_with(b"#?") {
        return None;
    }
    let head = bytes.get(..bytes.len().min(4096))?;
    let text = String::from_utf8_lossy(head);
    let mut lines = text.lines().skip_while(|line| !line.is_empty());
    lines.next()?;
    let mut parts = lines.next()?.split_ascii_whitespace();
    let (_, height, _, width) = (parts.next()?, parts.next()?, parts.next()?, parts.next()?);
    Some(ImageInfo {
        width: width.parse().ok()?,
        height: height.parse().ok()?,
        channels: Some(3),
    })
}

/// KTX 1 (`pixelWidth` at 36, either byte order) or KTX 2 (`pixelWidth` at 20).
fn ktx(bytes: &[u8]) -> Option<ImageInfo> {
    let (width, height) = if bytes.starts_with(b"\xABKTX 11\xBB\r\n\x1a\n") {
        if le32(bytes, 12)? == 0x0403_0201 {
            (le32(bytes, 36)?, le32(bytes, 40)?)
        } else {
            (be32(bytes, 36)?, be32(bytes, 40)?)
        }
    } else if bytes.starts_with(b"\xABKTX 20\xBB\r\n\x1a\n") {
        (le32(bytes, 20)?, le32(bytes, 24)?)
    } else {
        return None;
    };
    Some(ImageInfo {
        width,
        height,
        channels: None,
    })
}

/// The first bytes of a PNG of this size and colour type (the header is all the reader looks
/// at): for the tests of this crate and of the crates that read images.
#[doc(hidden)]
pub fn png_header(width: u32, height: u32, color_type: u8) -> Vec<u8> {
    let mut bytes = b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR".to_vec();
    bytes.extend(width.to_be_bytes());
    bytes.extend(height.to_be_bytes());
    bytes.extend([8, color_type, 0, 0, 0]);
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_format_is_the_extension_compared_exactly() {
        assert_eq!(
            ImageFormat::from_path("textures/a.png"),
            Some(ImageFormat::Png)
        );
        assert_eq!(
            ImageFormat::from_path("textures/a.jpeg"),
            Some(ImageFormat::Jpeg)
        );
        assert_eq!(ImageFormat::from_path("textures/a.PNG"), None);
        assert_eq!(ImageFormat::from_path("textures/a.json"), None);
        assert_eq!(
            ImageFormat::from_path_ignoring_case("textures/a.PNG"),
            Some(ImageFormat::Png)
        );
    }

    #[test]
    fn a_png_header_gives_size_and_channels() {
        let info = ImageInfo::read(ImageFormat::Png, &png_header(16, 32, 6)).unwrap();
        assert_eq!((info.width, info.height, info.channels), (16, 32, Some(4)));
        let grey = ImageInfo::read(ImageFormat::Png, &png_header(1, 1, 0)).unwrap();
        assert_eq!(grey.channels, Some(1));
        assert_eq!(ImageInfo::read(ImageFormat::Png, b"not a png"), None);
        assert_eq!(
            ImageInfo::read(ImageFormat::Png, &png_header(1, 1, 6)[..20]),
            None
        );
    }

    #[test]
    fn a_tga_header_gives_size_and_channels() {
        let mut bytes = vec![0u8; 18];
        bytes[2] = 2;
        bytes[12..14].copy_from_slice(&16u16.to_le_bytes());
        bytes[14..16].copy_from_slice(&64u16.to_le_bytes());
        bytes[16] = 32;
        let info = ImageInfo::read(ImageFormat::Tga, &bytes).unwrap();
        assert_eq!((info.width, info.height, info.channels), (16, 64, Some(4)));
        bytes[2] = 0;
        assert_eq!(ImageInfo::read(ImageFormat::Tga, &bytes), None);
    }

    /// The CRC-32 of a PNG chunk (its type and data).
    fn crc32(bytes: &[u8]) -> u32 {
        let mut crc = u32::MAX;
        for byte in bytes {
            crc ^= u32::from(*byte);
            for _ in 0..8 {
                crc = (crc >> 1) ^ (0xEDB8_8320 & 0u32.wrapping_sub(crc & 1));
            }
        }
        !crc
    }

    /// One PNG chunk: length, type, data, CRC.
    fn chunk(kind: [u8; 4], data: &[u8]) -> Vec<u8> {
        let mut body = kind.to_vec();
        body.extend(data);
        let mut bytes = u32::try_from(data.len()).unwrap().to_be_bytes().to_vec();
        bytes.extend(&body);
        bytes.extend(crc32(&body).to_be_bytes());
        bytes
    }

    /// A whole colour-mapped PNG of 2 x 1 pixels: a palette of two colours (red, blue), with
    /// `alpha` as the `tRNS` chunk when given, and the two indices in a stored zlib block.
    /// Decoded it is two pixels of three channels, or of four with `alpha`.
    fn palette_png(alpha: Option<&[u8]>) -> Vec<u8> {
        let mut bytes = b"\x89PNG\r\n\x1a\n".to_vec();
        // 2 x 1, 8 bits, colour type 3, deflate, adaptive filtering, no interlace.
        bytes.extend(chunk(*b"IHDR", &[0, 0, 0, 2, 0, 0, 0, 1, 8, 3, 0, 0, 0]));
        bytes.extend(chunk(*b"PLTE", &[255, 0, 0, 0, 0, 255]));
        if let Some(alpha) = alpha {
            bytes.extend(chunk(*b"tRNS", alpha));
        }
        // The scan line: filter 0, index 0, index 1. Adler-32 of [0, 0, 1] is 0x0004_0002.
        let data = [
            0x78, 0x01, 0x01, 3, 0, 0xFC, 0xFF, 0, 0, 1, 0x00, 0x04, 0x00, 0x02,
        ];
        bytes.extend(chunk(*b"IDAT", &data));
        bytes.extend(chunk(*b"IEND", &[]));
        bytes
    }

    /// A whole uncompressed TGA of 2 x 1 pixels: true colour of `depth` bits (image type 2),
    /// or, with `map_entry`, colour-mapped (image type 1) with a palette of two entries of
    /// that many bits and 8-bit indices.
    fn tga_image(map_entry: Option<u8>, depth: u8) -> Vec<u8> {
        let mut bytes = vec![0u8; 18];
        bytes[12] = 2;
        bytes[14] = 1;
        bytes[16] = depth;
        if let Some(entry) = map_entry {
            bytes[1] = 1;
            bytes[2] = 1;
            // The colour map: first entry 0, two entries, `entry` bits each.
            bytes[5] = 2;
            bytes[7] = entry;
            bytes.resize(18 + 2 * usize::from(entry / 8), 0x7F);
            bytes.extend([0, 1]);
        } else {
            bytes[2] = 2;
            bytes.resize(18 + 2 * usize::from(depth / 8), 0x7F);
        }
        bytes
    }

    // A colour-mapped PNG decodes to its palette's colours, three channels, and to four when a
    // `tRNS` chunk gives the palette alpha; its count is not the width of its index.
    #[test]
    fn a_colour_mapped_png_has_the_channels_of_its_palette() {
        let opaque = ImageInfo::read(ImageFormat::Png, &palette_png(None)).unwrap();
        assert_eq!(
            (opaque.width, opaque.height, opaque.channels),
            (2, 1, Some(3))
        );
        let alpha = ImageInfo::read(ImageFormat::Png, &palette_png(Some(&[0, 255]))).unwrap();
        assert_eq!(alpha.channels, Some(4));
        // The chunk counts only before the image data, and only for a palette.
        let mut late = palette_png(None);
        let end = late.len() - 12;
        late.splice(end..end, chunk(*b"tRNS", &[0, 255]));
        assert_eq!(
            ImageInfo::read(ImageFormat::Png, &late).unwrap().channels,
            Some(3)
        );
        // A header that is cut off after `IHDR` still gives the size and the palette's three.
        let cut = &palette_png(Some(&[0, 255]))[..33];
        assert_eq!(
            ImageInfo::read(ImageFormat::Png, cut).unwrap().channels,
            Some(3)
        );
        // The other colour types are counted by their samples.
        for (color_type, channels) in [(0, 1), (4, 2), (2, 3), (6, 4)] {
            let info = ImageInfo::read(ImageFormat::Png, &png_header(1, 1, color_type)).unwrap();
            assert_eq!(info.channels, Some(channels), "colour type {color_type}");
        }
    }

    // The game decodes a TGA to four channels whatever its header's depth (its own packs put
    // 24-bit true-colour TGAs on the one layer that takes four channels only); a colour-mapped
    // image's count is not its index width.
    #[test]
    fn a_tga_has_the_four_channels_the_game_decodes_it_to() {
        let rgb = ImageInfo::read(ImageFormat::Tga, &tga_image(None, 24)).unwrap();
        assert_eq!((rgb.width, rgb.height, rgb.channels), (2, 1, Some(4)));
        let rgba = ImageInfo::read(ImageFormat::Tga, &tga_image(None, 32)).unwrap();
        assert_eq!(rgba.channels, Some(4));
        let mapped = ImageInfo::read(ImageFormat::Tga, &tga_image(Some(24), 8)).unwrap();
        assert_eq!((mapped.width, mapped.height), (2, 1));
        assert_eq!(mapped.channels, Some(4));
        // The run-length forms are the same images.
        let mut packed = tga_image(None, 24);
        packed[2] = 10;
        assert_eq!(
            ImageInfo::read(ImageFormat::Tga, &packed).unwrap().channels,
            Some(4)
        );
        // A depth no TGA has is still no image.
        let mut odd = tga_image(None, 24);
        odd[16] = 7;
        assert_eq!(ImageInfo::read(ImageFormat::Tga, &odd), None);
    }

    /// The first chunks of a 1 x 1 PNG of `color_type` with a `tRNS` chunk of `alpha` before
    /// its image data.
    fn png_with_trns(color_type: u8, alpha: &[u8]) -> Vec<u8> {
        let mut bytes = b"\x89PNG\r\n\x1a\n".to_vec();
        bytes.extend(chunk(
            *b"IHDR",
            &[0, 0, 0, 1, 0, 0, 0, 1, 8, color_type, 0, 0, 0],
        ));
        bytes.extend(chunk(*b"tRNS", alpha));
        bytes.extend(chunk(*b"IDAT", &[]));
        bytes
    }

    // A `tRNS` chunk gives a grey or a true-colour image one
    // transparent colour, and a decoder that honours it hands back one channel more, as it
    // does for a palette: RGB with `tRNS` is four channels (PNG optimisers write that for an
    // RGBA image with one transparent colour), grey with `tRNS` two.
    #[test]
    fn a_trns_chunk_adds_the_alpha_channel_to_a_grey_and_a_true_colour_png() {
        let of = |bytes: &[u8]| ImageInfo::read(ImageFormat::Png, bytes).unwrap().channels;
        assert_eq!(of(&png_with_trns(2, &[0, 1, 0, 2, 0, 3])), Some(4));
        assert_eq!(of(&png_with_trns(0, &[0, 9])), Some(2));
        // The types that have their alpha in the samples take no `tRNS`: counted by their samples.
        assert_eq!(of(&png_with_trns(4, &[0, 9])), Some(2));
        assert_eq!(of(&png_with_trns(6, &[0, 9])), Some(4));
        // Without the chunk, the count is the samples'.
        assert_eq!(of(&png_header(1, 1, 2)), Some(3));
        assert_eq!(of(&png_header(1, 1, 0)), Some(1));
    }

    // A chunk length that carries the walk to the end of the
    // address space ends the walk; no sum of offsets is taken unchecked.
    #[test]
    fn a_chunk_length_at_the_end_of_the_address_space_ends_the_walk() {
        for length in [u32::MAX, u32::MAX - 20, 0xFFFF_FFEB, 0x7FFF_FFFF] {
            let mut bytes = b"\x89PNG\r\n\x1a\n".to_vec();
            bytes.extend(chunk(*b"IHDR", &[0, 0, 0, 1, 0, 0, 0, 1, 8, 3, 0, 0, 0]));
            bytes.extend(length.to_be_bytes());
            bytes.extend(b"junk");
            let info = ImageInfo::read(ImageFormat::Png, &bytes).unwrap();
            assert_eq!(info.channels, Some(3), "{length:#x}");
        }
        assert_eq!(next_chunk(usize::MAX - 3, 0), None);
        assert_eq!(next_chunk(usize::MAX - 20, 9), None);
        assert_eq!(next_chunk(8, 13), Some(33));
    }

    #[test]
    fn a_jpeg_frame_segment_gives_size_and_channels() {
        // SOI, an APP0 segment of 4 bytes, SOF0: precision 8, height 16, width 24, 3 channels.
        let bytes = [
            0xFF, 0xD8, 0xFF, 0xE0, 0, 4, 1, 2, 0xFF, 0xC0, 0, 11, 8, 0, 16, 0, 24, 3,
        ];
        let info = ImageInfo::read(ImageFormat::Jpg, &bytes).unwrap();
        assert_eq!((info.width, info.height, info.channels), (24, 16, Some(3)));
        assert_eq!(ImageInfo::read(ImageFormat::Jpeg, &bytes[..9]), None);
    }

    #[test]
    fn an_hdr_resolution_line_gives_the_size() {
        let bytes = b"#?RADIANCE\nFORMAT=32-bit_rle_rgbe\n\n-Y 8 +X 4\n";
        let info = ImageInfo::read(ImageFormat::Hdr, bytes).unwrap();
        assert_eq!((info.width, info.height), (4, 8));
    }

    #[test]
    fn a_ktx_header_gives_the_size() {
        let mut bytes = b"\xABKTX 11\xBB\r\n\x1a\n".to_vec();
        bytes.extend(0x0403_0201u32.to_le_bytes());
        bytes.resize(36, 0);
        bytes.extend(8u32.to_le_bytes());
        bytes.extend(4u32.to_le_bytes());
        let info = ImageInfo::read(ImageFormat::Ktx, &bytes).unwrap();
        assert_eq!((info.width, info.height, info.channels), (8, 4, None));
    }
}
