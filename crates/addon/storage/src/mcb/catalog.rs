//! `minecraft:crafting_items_catalog` bodies (1.21.60, 1.21.70, 1.21.110, 1.26.10, 1.26.30,
//! 1.26.40 and 1.26.50, every version the game bakes; the bakes below 1.26.50 carry `categories`
//! only, over the same element types).
//!
//! A body is read in the order of the document's members: a count of
//! categories; per category `category_name` (the enum baked as its name, a string) and a count
//! of groups; per group a flag for the optional `group_identifier` (its `name`, then a flag for
//! the optional `icon`, an item), then a count of items. An item is the struct of one string,
//! written here as the string, its short form.
// Checked against the game's four crafting catalogues; the 1.26.40 one equals its loose twin.

use super::McbValue;
use super::reader::{Decoded, Members, Reader, object, string};

/// The four category names the enum binds (`bindConst<0..3>`).
const CATEGORIES: [&str; 4] = ["construction", "nature", "equipment", "items"];

/// The payload of a catalogue document.
pub(super) fn decode(bytes: &[u8]) -> Decoded<McbValue> {
    let reader = &mut Reader::new(bytes);
    let categories = reader.list(category)?;
    reader.end()?;
    Ok(object(vec![("categories".into(), categories)]))
}

/// One category.
fn category(reader: &mut Reader<'_>) -> Decoded<McbValue> {
    let name = reader.string()?;
    if !CATEGORIES.contains(&&*name) {
        return Err(super::reader::UNSUPPORTED);
    }
    let groups = reader.list(group)?;
    Ok(object(vec![
        ("category_name".into(), string(name)),
        ("groups".into(), groups),
    ]))
}

/// One group.
fn group(reader: &mut Reader<'_>) -> Decoded<McbValue> {
    let mut members: Members = Vec::new();
    if reader.flag()? {
        let mut identifier: Members = vec![("name".into(), string(reader.string()?))];
        if reader.flag()? {
            identifier.push(("icon".into(), string(reader.string()?)));
        }
        members.push(("group_identifier".into(), object(identifier)));
    }
    members.push((
        "items".into(),
        reader.list(|reader| Ok(string(reader.string()?)))?,
    ));
    Ok(object(members))
}
