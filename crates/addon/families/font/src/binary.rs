//! The binary files of an MSDF font, `<prefix>XX.fontdata` and `remapping.dat`, and the shape
//! of a glyph sheet. A stack load reads none of them: whoever holds a file's bytes calls these.

use indexmap::IndexMap;

use super::GlyphPage;
use bedrock_addon_core::diagnostic::builders::{invalid, malformed};
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, JsonPath};

/// The size of a metrics file: the header and 256 advances.
const FONTDATA_SIZE: usize = 4 + 256 * 4;
/// The game's assert for a metrics file whose header is not 0.
const FONTDATA_VERSION: &str = "Version of MSDF data file should be 0";

impl GlyphPage {
    /// Reads a `.fontdata` file.
    ///
    /// A header other than 0 refuses the file (the page then has no widths), and so does a
    /// file too short to hold the header. A file shorter than 1028 bytes leaves the missing
    /// advances at 0, and bytes after the 1028th are ignored, each with an information. An advance
    /// that is 0 or less is an info: the game uses it as the character's width (its own files
    /// have such slots for characters they do not draw).
    ///
    /// # Errors
    ///
    /// The findings of a refused file, the header's error among them.
    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, Diagnostics), Diagnostics> {
        let mut diagnostics = Diagnostics::default();
        if bytes.len() != FONTDATA_SIZE {
            // A file of the wrong size.
            let message = format!("{} bytes, not {FONTDATA_SIZE}", bytes.len());
            diagnostics.push(Diagnostic::info(malformed(message, None)));
        }
        // A header the file does not hold whole is whatever the game's stack held: in practice
        // not 0.
        let header = bytes.first_chunk::<4>().map(|raw| u32::from_le_bytes(*raw));
        if header != Some(0) {
            // A header other than 0.
            let message = match header {
                Some(header) => format!("header {header}"),
                None => "no header".to_owned(),
            };
            diagnostics.push(Diagnostic::info(malformed(message, Some(FONTDATA_VERSION))));
            return Err(diagnostics);
        }
        let mut advances = [0.0_f32; 256];
        for (i, (slot, raw)) in advances
            .iter_mut()
            .zip(bytes[4..].as_chunks::<4>().0)
            .enumerate()
        {
            *slot = f32::from_le_bytes(*raw);
            // A NaN is neither positive nor 0, so it is reported as well.
            if *slot <= 0.0 {
                diagnostics.push(Diagnostic::info(invalid()).at(JsonPath::new().index(i)));
            }
        }
        Ok((Self { advances }, diagnostics))
    }

    /// The file's bytes: the header 0 and the 256 advances.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(FONTDATA_SIZE);
        bytes.extend(0_u32.to_le_bytes());
        for advance in &self.advances {
            bytes.extend(advance.to_le_bytes());
        }
        bytes
    }

    /// The width, in pixels of the 8-pixel font height, of the page's character `index`.
    #[must_use]
    pub fn width(&self, index: u8) -> f32 {
        self.advances[usize::from(index)] * 8.0
    }
}

/// One record of a `remapping.dat`: the glyph drawn for a character.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RemapEntry {
    /// The character in the text.
    pub from: u16,
    /// The character whose glyph is drawn instead; its page selects the atlas.
    pub to: u16,
}

/// A `remapping.dat` beside an MSDF font's pages: which glyph each character is drawn with.
///
/// Binary, little-endian, no header: records of two `u16`, `from` then `to`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[doc(alias = "remapping.dat")]
pub struct GlyphRemapping(pub Vec<RemapEntry>);

impl GlyphRemapping {
    /// Reads a `remapping.dat`. Bytes after the last whole record are ignored, and of two
    /// records for one character the later stands; each is an information.
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> (Self, Diagnostics) {
        let mut diagnostics = Diagnostics::default();
        let mut seen = std::collections::HashSet::new();
        let mut records = Vec::with_capacity(bytes.len() / 4);
        for (i, raw) in bytes.as_chunks::<4>().0.iter().enumerate() {
            let entry = RemapEntry {
                from: u16::from_le_bytes([raw[0], raw[1]]),
                to: u16::from_le_bytes([raw[2], raw[3]]),
            };
            if !seen.insert(entry.from) {
                let code = malformed(format!("character {:#06X} mapped twice", entry.from), None);
                diagnostics.push(Diagnostic::info(code).at(JsonPath::new().index(i)));
            }
            records.push(entry);
        }
        let rest = bytes.len() % 4;
        if rest != 0 {
            let code = malformed(format!("{rest} bytes after the last record"), None);
            diagnostics.push(Diagnostic::info(code));
        }
        (Self(records), diagnostics)
    }

    /// The file's bytes.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.0.len() * 4);
        for entry in &self.0 {
            bytes.extend(entry.from.to_le_bytes());
            bytes.extend(entry.to.to_le_bytes());
        }
        bytes
    }

    /// The table the game ends up with: each character once, with the glyph of its last
    /// record.
    #[must_use]
    pub fn to_map(&self) -> IndexMap<u16, u16> {
        self.0.iter().map(|entry| (entry.from, entry.to)).collect()
    }
}

// The PNG header reader is core's, shared with the other glyph-sheet checks.
pub use bedrock_addon_shared_shapes::png_size;

/// Checks a glyph sheet (`font/glyph_XX.png`, `font/default8.png`) given its bytes: the game
/// cuts a sheet into 16 by 16 cells of a sixteenth of its width, so a sheet that is not square
/// or whose side is no multiple of 16 draws wrong (`GlyphSheetShape`, an info). An image that
/// is no PNG is not judged.
#[must_use]
pub fn check_glyph_sheet(bytes: &[u8]) -> Diagnostics {
    let mut diagnostics = Diagnostics::default();
    if let Some((width, height)) = png_size(bytes)
        && (width != height || width % 16 != 0 || width == 0)
    {
        diagnostics.push(Diagnostic::info(DiagnosticCode::GlyphSheetShape));
    }
    diagnostics
}
