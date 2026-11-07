//! The culling rules a block names against the block's geometry.

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::diagnostic::builders;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::id::GeometryId;

use super::super::{At, Findings, LinkCx, Target, kinds, unresolved};
use super::block_culling::{culled, rule_path, rules_of};
use super::told::{Told, formatted};

/// The bone check: the named bone must exist in the geometry.
const NO_SUCH_BONE: &str = "Did not find any bones with name `{}`";
/// The cube-index check: the cube of the bone must exist.
const CUBE_OUT_OF_BOUNDS: &str = "Cube index `{}` is out of bounds for schematic part `{}`";

/// Every rule of the culling rules a block's `minecraft:geometry` names hides a part of that
/// block's geometry: its `bone` is a bone of the geometry and its `cube`, when it has one, an
/// index below the number of the bone's cubes. A bone the geometry lacks is an
/// `UnresolvedReference` error, a cube beyond the bone's an `InvalidValue` error, each with
/// vanilla's text at the member of the rule and once per member: the client logs the miss and
/// goes on with the next rule, so each is a kept error.
///
/// A bone is found by its name without case, as the geometry keeps its bones. A geometry no
/// pack of the stack defines has no bones to compare with: that it is missing is another
/// rule's to say.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    let mut told = Told::default();
    for culled in culled(cx) {
        let Some(document) = rules_of(cx, &culled) else {
            continue;
        };
        let geometry = GeometryId::parse(culled.geometry.identifier.as_str());
        if !matches!(cx.geometry(&geometry), Target::Defined(_)) {
            continue;
        }
        let origin = document.origin();
        for (index, rule) in document.value().rules.iter().enumerate() {
            let part = &rule.geometry_part;
            let Some(bone) = cx.geometry_bone(&geometry, &part.bone) else {
                if told.first(origin, "", "bone", &index.to_string()) {
                    // The game logs the miss at the error level and goes on with the next rule:
                    // a kept error.
                    let at = At::new(
                        origin,
                        rule_path(index, &["geometry_part", "bone"]),
                        Severity::Error,
                    )
                    .with_text(formatted(NO_SUCH_BONE, &[&part.bone]));
                    out.push(unresolved(at, kinds::BONE, &part.bone, None).into_kept());
                }
                continue;
            };
            let Some(cube) = part.cube else {
                continue;
            };
            let cubes = bone.cubes.as_ref().map_or(0, Vec::len);
            // A cube below 0 is refused when the file is read.
            let inside = usize::try_from(cube).is_ok_and(|cube| cube < cubes);
            if !inside && told.first(origin, "", "cube", &index.to_string()) {
                let number = cube.to_string();
                let code = builders::worded(
                    &core_codes::INVALID_VALUE,
                    &core_codes::INVALID_VALUE_WORDED,
                    Some(formatted(CUBE_OUT_OF_BOUNDS, &[&number, &part.bone]).as_str()),
                );
                // The game logs the miss at the error level and goes on with the next rule:
                // a kept error.
                let at = At::new(
                    origin,
                    rule_path(index, &["geometry_part", "cube"]),
                    Severity::Error,
                );
                out.push(at.reference(code, kinds::CUBE, &number, None).into_kept());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::tests::support::{leak, set};
    use super::super::super::{LinkOptions, link_with_rules};
    use super::{CUBE_OUT_OF_BOUNDS, NO_SUCH_BONE, run};

    #[test]
    fn every_template_takes_the_arguments_it_is_given() {
        assert_eq!(NO_SUCH_BONE.matches("{}").count(), 1);
        assert_eq!(CUBE_OUT_OF_BOUNDS.matches("{}").count(), 2);
    }

    /// `n` blocks, each with its own geometry and its own culling rules of two rules: per
    /// rule the geometry's bone is looked up, one rule in four names a bone the geometry
    /// lacks.
    #[test]
    fn n_blocks_with_their_rules_cost_n() {
        fn text(text: &str) -> &'static str {
            std::str::from_utf8(leak(text)).unwrap_or_default()
        }
        let input = |n: usize| {
            let mut behavior: Vec<(&'static str, &'static str)> = Vec::new();
            let mut resource: Vec<(&'static str, &'static str)> = Vec::new();
            for index in 0..n {
                behavior.push((
                    text(&format!("blocks/b{index}.json")),
                    text(&format!(
                        r#"{{"format_version": "1.26.20", "minecraft:block": {{"description": {{"identifier": "demo:b{index}"}}, "components": {{"minecraft:geometry": {{"identifier": "geometry.g{index}", "culling": "demo:c{index}"}}, "minecraft:material_instances": {{"*": {{"texture": "t"}}}}}}}}}}"#
                    )),
                ));
                resource.push((
                    text(&format!("models/blocks/g{index}.geo.json")),
                    text(&format!(
                        r#"{{"format_version": "1.21.0", "minecraft:geometry": [{{"description": {{"identifier": "geometry.g{index}", "texture_width": 16, "texture_height": 16}}, "bones": [{{"name": "body", "cubes": [{{"origin": [0, 0, 0], "size": [1, 1, 1], "uv": [0, 0]}}]}}]}}]}}"#
                    )),
                ));
                let other = if index % 2 == 0 { "ghost" } else { "body" };
                resource.push((
                    text(&format!("block_culling/c{index}.json")),
                    text(&format!(
                        r#"{{"format_version": "1.21.80", "minecraft:block_culling_rules": {{"description": {{"identifier": "demo:c{index}"}}, "rules": [{{"geometry_part": {{"bone": "body", "cube": 0, "face": "up"}}, "direction": "up"}}, {{"geometry_part": {{"bone": "{other}"}}, "direction": "down"}}]}}}}"#
                    )),
                ));
            }
            let world = set(&behavior, &resource);
            // The collections are loaded before the clock starts: the rule is what is timed.
            assert_eq!(world.behavior.blocks().iter().count(), n);
            assert_eq!(world.resource.block_culling().iter().count(), n);
            assert_eq!(world.resource.geometries().iter().count(), n);
            (world, n.div_ceil(2))
        };
        crate::test_cost::assert_linear_at(
            "link: n blocks with their culling rules",
            2_400,
            input,
            |(world, missing)| {
                let found = link_with_rules(world, LinkOptions::default(), &[run]);
                assert_eq!(found.len(), *missing);
                found
            },
        );
    }
}
