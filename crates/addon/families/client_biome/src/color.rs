//! The colours of client biomes: an RGB and an RGBA colour with channels 0 to 255, kept in the
//! form they were written.

use crate::codes;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, Diagnostic, Diagnostics, Severity};
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::schema::FromJson;
use bedrock_addon_shared_shapes::HexOr;

/// An opaque RGB colour: `"#RRGGBB"` or `[r, g, b]` with integer
/// channels, kept in the form it was written.
///
/// The game scales an array's channels by 1/255 and clamps them to `[0, 1]`, so a channel
/// outside `0..=255` is the nearest bound ([`HexOr::rgb`]).
pub type Color255Rgb = HexOr<[i32; 3]>;

/// A colour with alpha: `"#RRGGBBAA"` or `[r, g, b, a]` ([`HexOr::rgba`]).
pub type Color255Rgba = HexOr<[i32; 4]>;

// The game's colour pattern for biome colour members.
pub(crate) use bedrock_addon_shared_shapes::color::is_hex;

/// A colour member of a client biome (`schema::read_with`): `"#RRGGBB"` or three integers; any
/// other form is vanilla's one error for it, at the member. An integer channel outside
/// `0..=255` is clamped by the game without a word, so the crate tells it with an information.
pub(crate) fn colour(node: &Node<'_>, read: &mut Read<'_>) -> Option<Color255Rgb> {
    if read.stopped() {
        return None;
    }
    // Whatever the colour's own read tells of a form it refuses is the game's one line.
    let mut told = Diagnostics::default();
    let colour = Color255Rgb::from_json(node, &mut Read::new(read.reading(), &mut told));
    match colour {
        Some(HexOr::Hex(text)) if !is_hex(&text, 6) => {}
        Some(colour) => {
            if let HexOr::Array(channels) = &colour {
                for (index, _) in channels
                    .iter()
                    .enumerate()
                    .filter(|(_, channel)| !(0..=255).contains(*channel))
                {
                    // The game clamps the channel without a log line.
                    read.push(
                        Diagnostic::of(&core_codes::INVALID_VALUE)
                            .severity(Severity::Info)
                            .at(node.path().index(index)),
                    );
                }
            }
            return Some(colour);
        }
        None => {}
    }
    read.refuse(node, Code::from(&codes::COLOR));
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An eight-digit text and an array resolve to their channels, the array clamped.
    #[test]
    fn rgba_resolves_hex_and_clamped_arrays() {
        assert_eq!(
            Color255Rgba::Hex("#01020304".into()).rgba(),
            Some([1, 2, 3, 4])
        );
        assert_eq!(
            Color255Rgba::Array([-5, 300, 7, 255]).rgba(),
            Some([0, 255, 7, 255])
        );
        assert_eq!(Color255Rgba::Hex("#010203".into()).rgba(), None);
    }

    /// Equal colours hash alike, so they can key a set.
    #[test]
    fn colours_can_key_a_set() {
        let set: std::collections::HashSet<Color255Rgb> =
            [Color255Rgb::Array([1, 2, 3]), Color255Rgb::Array([1, 2, 3])].into();
        assert_eq!(set.len(), 1);
    }
}
