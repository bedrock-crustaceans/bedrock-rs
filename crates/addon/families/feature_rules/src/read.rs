//! Reading [`FeatureRule`] from the text of a payload as vanilla's cereal schema does: an
//! unknown member warns, every other failure skips the file. Read member by member by core's
//! reader (`versioning::read_payload`).
//!
//! What is the family's own: a `distribution`'s three coordinates are required in a document
//! (`schema::read_with = distribution`; they are optional in the shared type, which a scatter
//! feature of an old format fills in part), and a loaded rule's biome filter is checked.

use super::FeatureRule;
use bedrock_addon_core::diagnostic::{Class, Diagnostics};
use bedrock_addon_core::json::JsonSlice;
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::schema::{FromJson, Schema, SchemaCx};
use bedrock_addon_core::versioning::{ParseCx, read_payload};
use bedrock_addon_shared_scatter::ScatterParams;

/// Reads one `minecraft:feature_rules` payload; `None` (with an error in `d`) when vanilla's
/// cereal pass would fail the file.
pub(crate) fn parse_rule(
    payload: JsonSlice<'_>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Option<FeatureRule> {
    let mut rule: FeatureRule = read_payload(payload, cx, d)?;
    if let Some(filter) = &rule.conditions.biome_filter {
        let at = cx.at.with(&["conditions", "minecraft:biome_filter"]);
        d.extend(filter.check(&at, cx.load.grade(Class::Unknown)));
    }
    // The bind's minimums: below one vanilla's constraint violation is fatal under the slice's
    // error mask and fails the file; the crate reports it as `InvalidValue`, without vanilla's
    // generic cereal text.
    let mut bounds = SchemaCx::new(crate::READING, cx.at.clone(), d);
    rule.check(&mut bounds);
    if bounds.failed() {
        return None;
    }
    rule.source_format_version = cx.original_version;
    Some(rule)
}

/// `distribution` (`schema::read_with` of `distribution`): the scatter parameters, each
/// coordinate present.
pub(crate) fn distribution(node: &Node<'_>, read: &mut Read<'_>) -> Option<ScatterParams> {
    let params = ScatterParams::from_json(node, read)?;
    let members = read.object(node)?;
    let mut present = true;
    for (key, coordinate) in [("x", &params.x), ("y", &params.y), ("z", &params.z)] {
        if coordinate.is_none() {
            read.missing_member(&members, key);
            present = false;
        }
    }
    present.then_some(params)
}
