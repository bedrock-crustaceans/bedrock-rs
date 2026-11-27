//! The culling rules a block names against the culling rules of the resource stack.

use bedrock_addon_core::diagnostic::builders;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{JsonPath, Severity};
use bedrock_addon_core::document::{ComponentMap, Document};
use bedrock_addon_core::id::CullingRulesId;

use super::told::{Told, formatted};
use crate::families::block::Block;
use crate::families::block::component::{BlockComponent, Geometry, GeometryParams};
use crate::families::block_culling::{Condition, CullingRules};
use crate::pack::link::prelude::*;

/// The culling name check. The two spaces in the text are the game's.
const MISSING_ASSET: &str =
    " -> components -> minecraft:geometry -> culling:  Missing referenced asset {}";
/// The geometry check of the culling rule.
const NO_CULLING_LAYER: &str =
    "culling condition specified is same_culling_layer, but geometry has no culling layer defined";

/// The culling layer of a block that names none.
const CULLING_LAYER_UNDEFINED: &str = "minecraft:culling_layer.undefined";

/// The component sections of a block with the path the block holds each at: the base
/// components, then those of each permutation. A rule names a member below one by
/// `Diagnostics::place_of` at the member's full path (the load takes out a permutation a gate
/// or a rule skips, and an upgrade step moves members inside a component).
fn sections(document: &Document<Block>) -> Vec<(JsonPath, &ComponentMap<BlockComponent>)> {
    let block = document.value();
    let mut sections = Vec::with_capacity(1 + block.permutations.len());
    let mut base = JsonPath::new();
    base.push("minecraft:block");
    base.push("components");
    sections.push((base, &block.components));
    for (index, permutation) in block.permutations.iter().enumerate() {
        let mut path = JsonPath::new();
        path.push("minecraft:block");
        path.push("permutations");
        path.push_index(index);
        path.push("components");
        sections.push((path, &permutation.components));
    }
    sections
}

/// A `minecraft:geometry` of a block that names culling rules.
#[derive(Debug)]
pub(super) struct Culled<'a> {
    /// The block's document.
    pub(super) block: &'a Document<Block>,
    /// The path the block holds the component at; a member of it is named in the file by
    /// `place_of` at the member's path.
    pub(super) path: JsonPath,
    /// The component.
    pub(super) geometry: &'a GeometryParams,
    /// The culling name as written; never empty.
    pub(super) culling: &'a str,
}

/// Every `minecraft:geometry` of the behaviour stack's blocks, in its base components or in a
/// permutation, whose `culling` is a name: an empty name is no reference.
pub(super) fn culled<'a>(cx: &LinkCx<'a>) -> Vec<Culled<'a>> {
    let mut all = Vec::new();
    for (_, block) in cx.behavior().blocks() {
        for (mut path, components) in sections(block) {
            let Some(Geometry::Params(geometry)) = components.get::<Geometry>() else {
                continue;
            };
            let Some(culling) = geometry.culling.as_deref() else {
                continue;
            };
            if culling.is_empty() {
                continue;
            }
            path.push("minecraft:geometry");
            all.push(Culled {
                block,
                path,
                geometry,
                culling,
            });
        }
    }
    all
}

/// The culling rules a geometry component names, when the resource stack has them.
pub(super) fn rules_of<'a>(
    cx: &LinkCx<'a>,
    culled: &Culled<'_>,
) -> Option<&'a Document<CullingRules>> {
    cx.resource()
        .block_culling()
        .get(&CullingRulesId::parse(culled.culling))
}

/// The path of the member `member` of rule `index` of a culling document.
pub(super) fn rule_path(index: usize, member: &[&str]) -> JsonPath {
    let mut path = JsonPath::new();
    path.push("minecraft:block_culling_rules");
    path.push("rules");
    path.push_index(index);
    for key in member {
        path.push(key);
    }
    path
}

/// A block's `minecraft:geometry` `culling` names the culling rules of the resource stack,
/// compared as written. A name no rules document has is an `UnresolvedReference` kept error
/// with vanilla's text, at the member: the block loads and the client builds no model for it.
pub(super) fn culling_names(cx: &LinkCx<'_>, out: &mut Findings) {
    for culled in culled(cx) {
        let id = CullingRulesId::parse(culled.culling);
        let Target::Missing(hint) = cx.culling_rules(&id) else {
            continue;
        };
        let mut held = culled.path;
        held.push("culling");
        let path = culled.block.diagnostics().place_of(&held);
        // The game logs the miss at the error level and the block loads: a kept error.
        let at = At::new(culled.block.origin(), path, Severity::Error)
            .with_text(formatted(MISSING_ASSET, &[culled.culling]));
        out.push(unresolved(at, kinds::CULLING_RULES, culled.culling, hint).into_kept());
    }
}

/// A culling rule whose `condition` is `same_culling_layer` needs a culling layer on the
/// block that uses the rules: a block whose `minecraft:geometry` has no `culling_layer`, or
/// the undefined one, keeps the rule and its condition never holds. `NoEffect` warning with
/// vanilla's text, at the rule's `condition`, once per rule.
pub(super) fn culling_layers(cx: &LinkCx<'_>, out: &mut Findings) {
    let mut told = Told::default();
    for culled in culled(cx) {
        let layered = culled
            .geometry
            .culling_layer
            .as_ref()
            .is_some_and(|layer| layer != CULLING_LAYER_UNDEFINED);
        if layered {
            continue;
        }
        let Some(document) = rules_of(cx, &culled) else {
            continue;
        };
        for (index, rule) in document.value().rules.iter().enumerate() {
            if rule.condition != Some(Condition::SameCullingLayer) {
                continue;
            }
            let origin = document.origin();
            if !told.first(origin, "", "condition", &index.to_string()) {
                continue;
            }
            let code = builders::worded(
                &core_codes::NO_EFFECT,
                &core_codes::NO_EFFECT_WORDED,
                Some(formatted(NO_CULLING_LAYER, &[]).as_str()),
            );
            let path = rule_path(index, &["condition"]);
            // The game logs this at the warning level: a warning.
            out.push(At::new(origin, path, Severity::Warning).diagnostic(code));
        }
    }
}

/// Every template of this module with the number of its arguments, for the arity test.
#[cfg(test)]
const TEMPLATES: &[(&str, usize)] = &[(MISSING_ASSET, 1), (NO_CULLING_LAYER, 0)];

#[cfg(test)]
mod tests {
    use super::TEMPLATES;
    use crate::pack::link::resource::told::formatted;

    #[test]
    fn every_template_takes_the_arguments_it_is_given() {
        for (template, arity) in TEMPLATES {
            assert_eq!(template.matches("{}").count(), *arity, "{template}");
            let args = vec!["x"; *arity];
            assert!(!formatted(template, &args).contains("{}"));
        }
    }
}
