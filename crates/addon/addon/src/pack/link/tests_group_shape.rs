//! What a unit test in a rule group's directory can reach: the fixtures of the frame's own
//! tests (`super::tests::support` from a group's `mod.rs`, `super::super::tests::support`
//! from a test module inside it), the context, the sink and a pass over rules of its own.

mod tests {
    use bedrock_addon_core::diagnostic::{JsonPath, Severity};
    use bedrock_addon_core::id::BlockId;

    #[cfg(feature = "block")]
    use super::super::tests::support::all_said;
    use super::super::tests::support::{block, hosted, origin, set, set_with};
    use super::super::{At, Findings, LinkCx, LinkOptions, kinds, link_with_rules, unresolved};

    /// A rule as a group writes one: every block of a fixed list that is missing.
    fn rule(cx: &LinkCx<'_>, out: &mut Findings) {
        let origin = origin("entities/pet.json");
        for name in ["demo:lamp", "minecraft:stone", "demo:none"] {
            if missing(cx, name) {
                let at = At::new(&origin, JsonPath::new(), Severity::Warning);
                out.push(unresolved(at, kinds::BLOCK, name, None));
            }
        }
    }

    #[cfg(feature = "block")]
    fn missing(cx: &LinkCx<'_>, name: &str) -> bool {
        cx.block(&BlockId::parse(name)).is_missing()
    }

    #[cfg(not(feature = "block"))]
    fn missing(_: &LinkCx<'_>, name: &str) -> bool {
        let _ = BlockId::parse(name);
        false
    }

    #[test]
    fn a_groups_unit_test_builds_a_world_and_runs_its_rule_with_the_frames_fixtures() {
        let lamp = block("demo:lamp", "{}");
        let files = [("blocks/lamp.json", lamp.as_str())];
        let found = link_with_rules(&set(&files, &[]), LinkOptions::default(), &[rule]);
        #[cfg(feature = "block")]
        assert_eq!(
            all_said(&found),
            [(
                "UnresolvedReference",
                Severity::Warning,
                String::new(),
                "block",
                "demo:none".to_owned()
            )]
        );
        #[cfg(not(feature = "block"))]
        assert!(found.is_empty());
        // The host of the fixtures answers for the rule too.
        let world = set_with(&files, &[], &hosted(&[("block", "demo:none")]));
        let found = link_with_rules(&world, LinkOptions::default(), &[rule]);
        #[cfg(feature = "block")]
        assert_eq!(
            all_said(&found)
                .iter()
                .map(|said| said.4.as_str())
                .collect::<Vec<_>>(),
            ["minecraft:stone"]
        );
        #[cfg(not(feature = "block"))]
        assert!(found.is_empty());
    }
}
