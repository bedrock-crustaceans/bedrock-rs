//! The one reader of an image's size the crates share: the pack icon lints, the glyph sheet
//! lint and the skin pack's creator rule all ask the same question of a PNG header.

/// The eight bytes every PNG file starts with.
const PNG_SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";

/// The width and height of a PNG image, from its header; `None` for bytes that do not start
/// as a PNG file does (the signature, then an `IHDR` chunk with both numbers). Nothing past
/// the header is read: no image is decoded.
#[must_use]
pub fn png_size(bytes: &[u8]) -> Option<(u32, u32)> {
    let rest = bytes.strip_prefix(PNG_SIGNATURE)?;
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
        assert_eq!(png_size(PNG_SIGNATURE), None);
        assert_eq!(png_size(b"\xff\xd8\xff\xe0\x00\x10JFIF"), None);
        let mut other_chunk = png(3, 4);
        other_chunk[12..16].copy_from_slice(b"IDAT");
        assert_eq!(png_size(&other_chunk), None);
        assert_eq!(png_size(b""), None);
    }
}
