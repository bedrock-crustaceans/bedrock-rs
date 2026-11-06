//! `minecraft:poi_block` and `minecraft:poi_tag` bodies (1.26.40).
//!
//! The reader walks the members in the document's order and finds every member written,
//! defaults included. Type: `description.identifier` (string),
//! `tickets` (`u8`), `usable_distance` (`u8`), `blocks` (count, strings), `tags` (count,
//! strings). Tag: `description.identifier`, `children` (count, strings), `parents` (count,
//! strings).
// Checked against the 20 documents of the game's `experimental_poi` pack.

use super::McbValue;
use super::reader::{Decoded, Reader, object, string};

/// The payload of a POI type document.
pub(super) fn decode_type(bytes: &[u8]) -> Decoded<McbValue> {
    let reader = &mut Reader::new(bytes);
    let description = description(reader)?;
    let tickets = McbValue::UInt(u64::from(reader.u8()?));
    let usable_distance = McbValue::UInt(u64::from(reader.u8()?));
    let blocks = names(reader)?;
    let tags = names(reader)?;
    reader.end()?;
    Ok(object(vec![
        ("description".into(), description),
        ("tickets".into(), tickets),
        ("usable_distance".into(), usable_distance),
        ("blocks".into(), blocks),
        ("tags".into(), tags),
    ]))
}

/// The payload of a POI tag document.
pub(super) fn decode_tag(bytes: &[u8]) -> Decoded<McbValue> {
    let reader = &mut Reader::new(bytes);
    let description = description(reader)?;
    let children = names(reader)?;
    let parents = names(reader)?;
    reader.end()?;
    Ok(object(vec![
        ("description".into(), description),
        ("children".into(), children),
        ("parents".into(), parents),
    ]))
}

/// `{"identifier": …}`.
fn description(reader: &mut Reader<'_>) -> Decoded<McbValue> {
    Ok(object(vec![(
        "identifier".into(),
        string(reader.string()?),
    )]))
}

/// A list of names.
fn names(reader: &mut Reader<'_>) -> Decoded<McbValue> {
    reader.list(|reader| Ok(string(reader.string()?)))
}
