//! The one reader of a PNG header the crates share: the pack icon lints, the glyph sheet
//! lint and the skin pack's creator rule ask it for the size ([`png_size`]), the texture
//! readers for the colour type as well ([`header`]).

/// The eight bytes every PNG file starts with.
pub const PNG_SIGNATURE: [u8; 8] = *b"\x89PNG\r\n\x1a\n";

/// What a PNG's `IHDR` chunk says of the image, as far as the crates ask: its size, the bits
/// per sample and the colour type (PNG, ISO/IEC 15948, 11.2.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PngHeader {
    /// The width in pixels.
    pub width: u32,
    /// The height in pixels.
    pub height: u32,
    /// The bits per sample (or per palette index).
    pub bit_depth: u8,
    /// The colour type: 0 grey, 2 true colour, 3 colour-mapped, 4 grey with alpha, 6 true
    /// colour with alpha; any other number is kept as written.
    pub color_type: u8,
}

/// The header of a PNG image; `None` for bytes that do not start as a PNG file does (the
/// signature, then an `IHDR` chunk with the size, the bit depth and the colour type). Nothing
/// past those fields is read: no image is decoded, and the colour type is not judged.
pub fn header(bytes: &[u8]) -> Option<PngHeader> {
    let (width, height) = png_size(bytes)?;
    let [bit_depth, color_type] = *bytes.get(24..26)? else {
        return None;
    };
    Some(PngHeader {
        width,
        height,
        bit_depth,
        color_type,
    })
}

/// The width and height of a PNG image, from its header; `None` for bytes that do not start
/// as a PNG file does (the signature, then an `IHDR` chunk with both numbers). Nothing past
/// the header is read: no image is decoded.
#[must_use]
pub fn png_size(bytes: &[u8]) -> Option<(u32, u32)> {
    let rest = bytes.strip_prefix(&PNG_SIGNATURE)?;
    // The first chunk: length, `IHDR`, width, height.
    let header = rest.get(4..16)?;
    let (name, size) = header.split_at(4);
    if name != b"IHDR" {
        return None;
    }
    let (width, height) = size.split_at(4);
    Some((
        u32::from_be_bytes(width.try_into().ok()?),
        u32::from_be_bytes(height.try_into().ok()?),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The header of a `width` by `height` PNG.
    fn png(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = PNG_SIGNATURE.to_vec();
        bytes.extend_from_slice(&13u32.to_be_bytes());
        bytes.extend_from_slice(b"IHDR");
        bytes.extend_from_slice(&width.to_be_bytes());
        bytes.extend_from_slice(&height.to_be_bytes());
        bytes.extend_from_slice(&[8, 6, 0, 0, 0]);
        bytes
    }

    #[test]
    fn png_size_reads_the_header_and_nothing_else() {
        assert_eq!(png_size(&png(512, 128)), Some((512, 128)));
        assert_eq!(png_size(&png(u32::MAX, 1)), Some((u32::MAX, 1)));
        // The sixteen bytes after the signature are enough, and one fewer is not.
        assert_eq!(png_size(&png(3, 4)[..24]), Some((3, 4)));
        assert_eq!(png_size(&png(3, 4)[..23]), None);
        assert_eq!(png_size(&PNG_SIGNATURE), None);
        assert_eq!(png_size(b"\xff\xd8\xff\xe0\x00\x10JFIF"), None);
        let mut other_chunk = png(3, 4);
        other_chunk[12..16].copy_from_slice(b"IDAT");
        assert_eq!(png_size(&other_chunk), None);
        assert_eq!(png_size(b""), None);
    }

    #[test]
    fn the_header_reads_the_size_bit_depth_and_colour_type() {
        let expected = PngHeader {
            width: 512,
            height: 128,
            bit_depth: 8,
            color_type: 6,
        };
        assert_eq!(header(&png(512, 128)), Some(expected));
        // The two bytes past the size are needed: twenty-six bytes are enough, one fewer is not.
        assert_eq!(header(&png(512, 128)[..26]), Some(expected));
        assert_eq!(header(&png(512, 128)[..25]), None);
        // A colour type the format does not define is the header's as written.
        let mut odd = png(1, 1);
        odd[25] = 7;
        assert_eq!(header(&odd).map(|h| h.color_type), Some(7));
        assert_eq!(header(&PNG_SIGNATURE), None);
        assert_eq!(header(b"\xff\xd8\xff\xe0\x00\x10JFIF"), None);
        let mut other_chunk = png(3, 4);
        other_chunk[12..16].copy_from_slice(b"IDAT");
        assert_eq!(header(&other_chunk), None);
        assert_eq!(header(b""), None);
    }

    #[test]
    fn the_size_is_the_header_s_where_both_read() {
        for (width, height) in [(1, 1), (16, 9), (u32::MAX, 0)] {
            let bytes = png(width, height);
            assert_eq!(
                header(&bytes).map(|h| (h.width, h.height)),
                png_size(&bytes)
            );
        }
    }
}
