//! The crate's own checks of a loaded structure beyond vanilla's load rules.

use super::Structure;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity,
};

/// An info for the first index of each layer outside `-1..len` of the `default` palette
/// (the crate's own note; vanilla is silent:
/// placement keeps the world block for every negative index and places air for an index past
/// the palette). Nothing without a `default` palette, the one vanilla places.
pub(super) fn index_notes(structure: &Structure) -> Diagnostics {
    let mut notes = Diagnostics::default();
    let Some(palette) = structure.structure.palette.get(Structure::DEFAULT_PALETTE) else {
        return notes;
    };
    let len = palette.block_palette.len();
    for (layer, indices) in structure.structure.block_indices.iter().enumerate() {
        let outside = indices.iter().position(|&index| {
            index < -1 || usize::try_from(index).is_ok_and(|index| index >= len)
        });
        if let Some(position) = outside {
            let mut at = JsonPath::new();
            at.push("structure");
            at.push("block_indices");
            at.push_index(layer);
            at.push_index(position);
            notes.push(Diagnostic::new(
                DiagnosticCode::InvalidValue { text: None },
                Severity::Info,
                Phase::Validate,
                Location::new(None, None, at, None),
            ));
        }
    }
    notes
}
