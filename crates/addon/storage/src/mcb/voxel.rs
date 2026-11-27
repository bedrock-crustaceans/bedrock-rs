//! `minecraft:voxel_shape` bodies.
//!
//! Layout: the identifier (string), the box count, then per box six `f32`: `min` x, y, z and
//! `max` x, y, z.
// Checked against the 218 distinct voxel shapes of the game's packs, each equal to its JSON twin.

use super::McbValue;
use super::reader::{Decoded, Reader, object, string};

/// The payload of a voxel shape document.
pub(super) fn decode(bytes: &[u8]) -> Decoded<McbValue> {
    let reader = &mut Reader::new(bytes);
    let identifier = reader.string()?;
    let boxes = reader.list(|reader| {
        let min = corner(reader)?;
        let max = corner(reader)?;
        Ok(object(vec![("min".into(), min), ("max".into(), max)]))
    })?;
    reader.end()?;
    Ok(object(vec![
        (
            "description".into(),
            object(vec![("identifier".into(), string(identifier))]),
        ),
        ("shape".into(), object(vec![("boxes".into(), boxes)])),
    ]))
}

/// Three floats.
fn corner(reader: &mut Reader<'_>) -> Decoded<McbValue> {
    Ok(McbValue::Array(vec![
        McbValue::F32(reader.f32()?),
        McbValue::F32(reader.f32()?),
        McbValue::F32(reader.f32()?),
    ]))
}
