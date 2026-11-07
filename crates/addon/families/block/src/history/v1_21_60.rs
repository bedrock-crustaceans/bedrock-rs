//! Format 1.21.60: `menu_category.group` is namespaced.

use super::legacy::{BlockPayload, component_path, misread, strings_told};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, Phase};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx};

/// A block document of `[1.21.60, 1.21.90)`.
pub type Block = Legacy<BlockPayload, 1, 21, 60>;

/// The v1 list.
const LIST: &str = "minecraft:custom_components";

/// the `minecraft:custom_components` list becomes keyed custom
/// components with empty parameters, the names kept in order in
/// `ordered_v1_custom_components` (the document's for the base, the permutation's for a
/// permutation).
impl LegacyStep<1, 21, 60> for BlockPayload {
    type Next = super::v1_21_90::Block;

    fn step(this: Block, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut block = this.0;
        let names = convert(&mut block.components, None, cx.diagnostics);
        block.ordered_v1_custom_components.extend(names);
        for (index, permutation) in block.permutations.iter_mut().enumerate() {
            let names = convert(&mut permutation.components, Some(index), cx.diagnostics);
            permutation.ordered_v1_custom_components.extend(names);
        }
        Legacy(block)
    }
}

/// Replaces the list in the components `map` of `section` by one `"ns:name": {}` entry per
/// name, at the list's place. A list that is no array names no component and an element that
/// is no string is left out: the list is removed either way, and each is told where it stands
/// (the crate's own findings; the game converts only a well-formed list of names).
fn convert(
    map: &mut indexmap::IndexMap<String, Value>,
    section: Option<usize>,
    d: &mut Diagnostics,
) -> Vec<String> {
    let Some(index) = map.get_index_of(LIST) else {
        return Vec::new();
    };
    let (_, list) = map.shift_remove_index(index).unwrap_or_default();
    let at = component_path(section, LIST);
    let names = if let Some(list) = list.as_array() {
        strings_told(list, &at, d)
    } else {
        misread(&at, d);
        Vec::new()
    };
    for (offset, name) in names.iter().enumerate() {
        // A component of the file under a name of the list: the step's entry stands (the
        // game keeps the entry alone), and the file's, when it holds anything
        // else, is told where it is (the crate's own finding). It is taken out first: an entry is
        // put at the list's place, not moved there.
        if let Some(had) = map.shift_remove(name)
            && !had.is_null()
            && !had.as_object().is_some_and(facet_value::VObject::is_empty)
        {
            d.push(
                Diagnostic::info(bedrock_addon_core::diagnostic::DiagnosticCode::from(
                    &core_codes::DROPPED_FIELD,
                ))
                .at(component_path(section, name))
                .phase(Phase::Upgrade),
            );
        }
        let at = (index + offset).min(map.len());
        map.shift_insert(at, name.clone(), facet_value::VObject::new().into());
    }
    names
}
