//! The structure a feature places (`structure_template_feature.structure_name`).

use super::super::{At, Findings, LinkCx, Target, kinds, unresolved};
use crate::diagnostic::{DiagnosticCode, JsonPath, Severity, VanillaText};
use crate::document::Document;
use crate::families::feature::Feature;
use crate::id::StructureId;
use bedrock_addon_core::diagnostic::builders;
use bedrock_addon_core::diagnostic::codes as core_codes;

/// vanilla's text for a `ground_level` above the structure's height.
// The game's text: the value, the height, the value it is clamped to.
const GROUND_LEVEL: &str = "minecraft:structure_template_feature: ground_level ({}) exceeds structure height ({}). Clamped to {}.";

/// `minecraft:structure_template_feature/<key>`.
fn member(key: &str) -> JsonPath {
    let mut path = JsonPath::new();
    path.push("minecraft:structure_template_feature");
    path.push(key);
    path
}

/// The structure a `structure_template_feature` places, by the name `/structure load`
/// takes (`<folder>:<path>`, a file directly in `structures/` being `mystructure:<name>`). A
/// name no structure of the stack has, and the game does not, is an `UnresolvedReference`
/// information.
///
/// A `ground_level` above the height of a structure of the
/// stack is an `InvalidValue` warning with vanilla's text; the game clamps it to the height
/// when it places the feature.
pub(super) fn feature_template(cx: &LinkCx<'_>, out: &mut Findings, document: &Document<Feature>) {
    let Feature::StructureTemplate(feature) = document.value() else {
        return;
    };
    let name = feature.structure_name.as_str();
    let origin = document.origin();
    match cx.structure(&StructureId::parse(name)) {
        Target::Missing(hint) => {
            // The game sends this line to the feature-placement failure log, not the content
            // log: the crate reports an information.
            let at = At::new(origin, member("structure_name"), Severity::Info);
            out.push(unresolved(at, kinds::STRUCTURE, name, hint));
        }
        Target::Defined(template) => {
            let Some(level) = feature.ground_level else {
                return;
            };
            let Some(structure) = cx.behavior().structures().get(&template.path) else {
                return;
            };
            let [_, height, _] = structure.value().size;
            if level.0 > height {
                let text = VanillaText::Formatted {
                    template: GROUND_LEVEL,
                    args: vec![level.0.to_string(), height.to_string(), height.to_string()],
                };
                // The game logs this at the warning level and clamps the value: a warning.
                let at = At::new(origin, member("ground_level"), Severity::Warning);
                out.push(at.diagnostic(DiagnosticCode::from(builders::worded(
                    &core_codes::INVALID_VALUE,
                    &core_codes::INVALID_VALUE_WORDED,
                    Some(&text),
                ))));
            }
        }
        _ => {}
    }
}
