//! The fixtures of the trim tests, read into plain rows.

#![allow(dead_code, reason = "each test file uses part of the helpers")]

use bedrock_addon::families::item::trim::{MaterialEntry, PatternEntry, TrimData};
use bedrock_addon::id::{ItemId, TrimMaterialId, TrimPatternId};
use facet::Facet;

/// `table/tr01_vanilla_table.json`: the tables, in build order.
#[derive(Debug, Facet)]
pub struct Table {
    pub patterns: Vec<PatternRow>,
    pub materials: Vec<MaterialRow>,
}

/// One pattern as text: `(pattern id, template item)`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Facet)]
pub struct PatternRow {
    pub pattern_id: String,
    pub item_name: String,
}

/// One material as text: `(material id, colour, ingredient item)`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Facet)]
pub struct MaterialRow {
    pub material_id: String,
    pub color: String,
    pub item_name: String,
}

/// An item identifier.
pub fn item(text: &str) -> ItemId {
    ItemId::new(text).unwrap()
}

/// The expected vanilla table.
pub fn vanilla_table() -> Table {
    facet_json::from_str(include_str!(
        "../fixtures/item/trim/table/tr01_vanilla_table.json"
    ))
    .unwrap()
}

/// The patterns of `data` as text, in its own order.
pub fn pattern_rows(data: &TrimData) -> Vec<PatternRow> {
    data.patterns()
        .map(|pattern| PatternRow {
            pattern_id: pattern.pattern_id.as_str().to_owned(),
            item_name: pattern.item_name.as_str().to_owned(),
        })
        .collect()
}

/// The materials of `data` as text, in its own order.
pub fn material_rows(data: &TrimData) -> Vec<MaterialRow> {
    data.materials()
        .iter()
        .map(|material| MaterialRow {
            material_id: material.material_id.as_str().to_owned(),
            color: material.color.as_str().to_owned(),
            item_name: material.item_name.as_str().to_owned(),
        })
        .collect()
}

/// `packet/pk01_trim_data.json`: a decoded `TrimDataPacket` payload, copied unchanged from
/// chorus-kotlin's `files/trim_data.json` (its key names are that dump's; the member order
/// is the wire order: "Item Name", "Pattern Id" and "Material Id", "Color", "Item Name").
#[derive(Debug, Facet)]
pub struct Payload {
    pub patterns: Vec<PayloadPattern>,
    pub materials: Vec<PayloadMaterial>,
}

/// One entry of the payload's "TrimPattern List".
#[derive(Debug, Facet)]
pub struct PayloadPattern {
    #[facet(rename = "itemName")]
    pub item_name: String,
    #[facet(rename = "patternId")]
    pub pattern_id: String,
}

/// One entry of the payload's "TrimMaterial List".
#[derive(Debug, Facet)]
pub struct PayloadMaterial {
    #[facet(rename = "materialId")]
    pub material_id: String,
    pub color: String,
    #[facet(rename = "itemName")]
    pub item_name: String,
}

/// The decoded payload, as the values `TrimData::from_packet` takes, in the payload's order.
pub fn payload() -> (Vec<PatternEntry>, Vec<MaterialEntry>) {
    let payload: Payload = facet_json::from_str(include_str!(
        "../fixtures/item/trim/packet/pk01_trim_data.json"
    ))
    .unwrap();
    let patterns = payload
        .patterns
        .iter()
        .map(|entry| pattern_entry(&entry.item_name, &entry.pattern_id))
        .collect();
    let materials = payload
        .materials
        .iter()
        .map(|entry| material_entry(&entry.material_id, &entry.color, &entry.item_name))
        .collect();
    (patterns, materials)
}

/// A payload pattern.
pub fn pattern_entry(template_item: &str, pattern: &str) -> PatternEntry {
    PatternEntry {
        template_item: ItemId::parse(template_item),
        pattern: TrimPatternId::parse(pattern),
    }
}

/// A payload material.
pub fn material_entry(id: &str, color: &str, item: &str) -> MaterialEntry {
    MaterialEntry {
        id: TrimMaterialId::parse(id),
        color: color.to_owned(),
        item: ItemId::parse(item),
    }
}
