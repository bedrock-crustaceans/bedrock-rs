//! The group's own parts: the cost of a pass and its order.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use crate::pack::link::tests::support::{all_said, block, pack, set_of};
use crate::pack::link::{LinkOptions, link_with_rules};

/// A world of `n` entities and `n` blocks: each entity names three blocks (its own, one of the
/// game's and one nothing defines, in another case than a block that is), an entity and
/// itself.
fn world(n: usize) -> crate::pack::StackSet {
    use bedrock_addon_core::load::LoadOptions;
    let mut files: Vec<(&'static str, Option<String>)> = Vec::new();
    let leak = |text: String| -> &'static str { Box::leak(text.into_boxed_str()) };
    for i in 0..n {
        files.push((
            leak(format!("blocks/b{i}.json")),
            Some(block(&format!("demo:block_{i}"), "{}")),
        ));
        let next = (i + 1) % n;
        files.push((
            leak(format!("entities/e{i}.json")),
            Some(format!(
                r#"{{"format_version": "1.26.50", "minecraft:entity": {{"description": {{"identifier": "demo:pet_{i}"}}, "components": {{
                    "minecraft:break_blocks": {{"breakable_blocks": ["demo:block_{i}", "minecraft:stone", "Demo:Block_{next}"]}},
                    "minecraft:trail": {{"block_type": "demo:none_{i}"}},
                    "minecraft:transformation": {{"into": "demo:pet_{next}"}},
                    "minecraft:shooter": {{"def": "demo:ghost_{i}"}}
                }}}}}}"#
            )),
        ));
    }
    let files: Vec<(&'static str, Option<&str>)> = files
        .iter()
        .map(|(path, text)| (*path, text.as_deref()))
        .collect();
    let set = set_of(
        vec![pack(1, "data", &files)],
        Vec::new(),
        &LoadOptions::default(),
    );
    // The load is not the pass's: both collections are read before the clock starts.
    assert_eq!(set.behavior.entities().len(), n);
    #[cfg(feature = "block")]
    assert_eq!(set.behavior.blocks().len(), n);
    set
}

/// What a pass over the world of `n` entities tells.
fn told(n: usize) -> usize {
    let per_entity = if cfg!(feature = "block") { 3 } else { 1 };
    n * per_entity
}

#[test]
fn a_pass_over_n_entities_with_n_references_costs_n() {
    crate::test_cost::assert_linear_at(
        "link entity group: n entities, n blocks",
        700,
        world,
        |set: &crate::pack::StackSet| {
            let found = link_with_rules(set, LinkOptions::default(), &[super::run]);
            assert_eq!(found.len(), told(set.behavior.entities().len()));
            found.len()
        },
    );
}

/// The pass over the small world, in full: each finding once, in document order.
#[test]
fn a_pass_tells_each_reference_once() {
    let set = world(2);
    let found = link_with_rules(&set, LinkOptions::default(), &[super::run]);
    let said: Vec<String> = all_said(&found)
        .into_iter()
        .map(|(_, _, at, kind, id)| format!("{kind} {id} {at}"))
        .collect();
    let c = "minecraft:entity/components/minecraft:";
    let mut expected = Vec::new();
    #[cfg(feature = "block")]
    for i in 0..2 {
        let next = (i + 1) % 2;
        expected.push(format!(
            "block Demo:Block_{next} {c}break_blocks/breakable_blocks/2"
        ));
        expected.push(format!("block demo:none_{i} {c}trail/block_type"));
    }
    for i in 0..2 {
        expected.push(format!("entity demo:ghost_{i} {c}shooter/def"));
    }
    assert_eq!(said, expected);
}
