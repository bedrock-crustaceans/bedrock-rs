//! The decoded payload of `TrimDataPacket`. The addon never
//! decodes the packet: the caller converts the protocol crate's packet into these values.

use super::{FormatCode, TrimData, TrimMaterial, TrimPattern, texts};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, JsonPath, Location, Phase, Severity, VanillaText,
};
use bedrock_addon_core::id::{ItemId, TrimMaterialId, TrimPatternId};

/// One pattern of the payload's `TrimPattern List`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatternEntry {
    /// The smithing template item ("Item Name", written first).
    pub template_item: ItemId,
    /// The pattern ("Pattern Id").
    pub pattern: TrimPatternId,
}

/// One material of the payload's `TrimMaterial List`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaterialEntry {
    /// The material ("Material Id").
    pub id: TrimMaterialId,
    /// The formatting code of the hover text ("Color"), as sent (`§h`).
    pub color: String,
    /// The ingredient item ("Item Name").
    pub item: ItemId,
}

/// The finding, in the words of vanilla's content-log error, for the entry `index` of the
/// pattern list, whose template item an earlier entry already named.
/// The game logs it at the content log's error level and goes on to the next entry, so the
/// finding is a kept error: an error that rejects nothing.
fn duplicate_template(index: usize, item: &ItemId) -> Diagnostic {
    let mut at = JsonPath::new();
    at.push("patterns");
    at.push_index(index);
    Diagnostic::new(
        DiagnosticCode::DuplicateIdentifier {
            text: Some(VanillaText::Formatted {
                template: texts::DUPLICATE_TEMPLATE,
                args: vec![item.as_str().to_owned()],
            }),
        },
        Severity::Error,
        Phase::Parse,
        Location::new(None, None, at, None),
    )
    .into_kept()
}

impl TrimData {
    /// The registries a client builds from a server's lists (the `TrimDataPacket` payload).
    ///
    /// The materials are taken as sent. The patterns arrive in no particular order (the server
    /// walks a hash map), so they are sorted by template item.
    ///
    /// A template item sent twice keeps its later pattern, and a kept error with the text of
    /// vanilla's content-log error for it is in [`diagnostics`](Self::diagnostics) at
    /// `patterns/<index>`. The client writes
    /// its two maps one by one, so the identifier of the overwritten entry still finds its
    /// template item through [`pattern`](Self::pattern).
    #[must_use]
    pub fn from_packet(patterns: &[PatternEntry], materials: &[MaterialEntry]) -> Self {
        let mut data = TrimData::default();
        // The client registers the entries one by one, in wire order.
        for (index, entry) in patterns.iter().enumerate() {
            let overwrites = data.insert_pattern(TrimPattern {
                pattern_id: entry.pattern.clone(),
                item_name: entry.template_item.clone(),
            });
            if overwrites {
                data.diagnostics
                    .push(duplicate_template(index, &entry.template_item));
            }
        }
        data.by_item
            .sort_by(|a, _, b, _| a.as_str().cmp(b.as_str()));
        // The material list is taken over as a whole.
        data.materials = materials
            .iter()
            .map(|entry| TrimMaterial {
                material_id: entry.id.clone(),
                color: FormatCode::new(entry.color.as_str()),
                item_name: entry.item.clone(),
            })
            .collect();
        data
    }

    /// The two lists a server sends for these registries: the patterns one per template
    /// item, the materials in order.
    pub fn to_packet(&self) -> (Vec<PatternEntry>, Vec<MaterialEntry>) {
        // The patterns come from walking the template-item map; the materials are the list.
        let patterns = self
            .by_item
            .values()
            .map(|pattern| PatternEntry {
                template_item: pattern.item_name.clone(),
                pattern: pattern.pattern_id.clone(),
            })
            .collect();
        let materials = self
            .materials
            .iter()
            .map(|material| MaterialEntry {
                id: material.material_id.clone(),
                color: material.color.as_str().to_owned(),
                item: material.item_name.clone(),
            })
            .collect();
        (patterns, materials)
    }
}
