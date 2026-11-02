//! The fixtures bt22 and bt24. bt22's findings are the link pass's: the
//! row runs through `StackSet::link`; the checker itself is also pinned in
//! `src/families/behavior_tree/tests.rs`. bt24 is about what a server sends, which is no load finding.

use bedrock_addon::diagnostic::Phase;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{PackOrigin, PackRepository, WorldPacks};

use super::common::*;

/// The link findings of fixture `name`'s stack, in `expected.toml`'s terms.
fn link_fixture(name: &str) -> Vec<Finding> {
    let packs = fixture_packs(name);
    let entries: Vec<String> = packs
        .iter()
        .map(|pack| {
            format!(
                r#"{{"pack_id": "{}", "version": [1, 0, 0]}}"#,
                pack.id().uuid
            )
        })
        .collect();
    let world = WorldPacks::parse(Some(format!("[{}]", entries.join(",")).as_bytes()), None)
        .expect("a world list");
    let mut repository = PackRepository::new();
    for pack in packs {
        repository.add_pack(pack, PackOrigin::User);
    }
    repository
        .stacks(&world, LoadOptions::default())
        .link()
        .iter()
        .filter(|finding| finding.phase == Phase::Link)
        .map(finding_of)
        .collect()
}

#[test]
fn bt22_an_unresolved_subtree_and_a_cycle_are_link_warnings() {
    let mut found: Vec<Finding> = link_fixture("bt22_subtree_unresolved")
        .into_iter()
        .filter(|finding| finding.file.starts_with("behavior_trees/"))
        .collect();
    let mut wanted = expected_findings("bt22_subtree_unresolved", true);
    assert_eq!(wanted.len(), 3);
    assert!(
        same_findings(&mut found, &mut wanted),
        "--- linked ---\n{found:#?}\n--- expected ---\n{wanted:#?}"
    );
}

#[test]
fn bt21_a_subtree_of_a_built_in_tree_resolves() {
    let found = link_fixture("bt21_subtree");
    assert!(
        found
            .iter()
            .all(|finding| finding.kind != "UnresolvedReference"),
        "{found:#?}"
    );
}

/// Until the link pass runs, the trees of bt22 load and loading says nothing of their
/// subtrees: vanilla checks neither the target nor a cycle at load.
#[test]
fn bt22_the_trees_load_and_loading_does_not_resolve_subtrees() {
    let stack = check("bt22_subtree_unresolved");
    let trees = stack.behavior_trees();
    for name in ["orphan", "a", "b"] {
        silent(trees.get(&id(name)).unwrap().diagnostics());
    }
    assert!(trees.diagnostics().next().is_none());
    assert_eq!(expected_findings("bt22_subtree_unresolved", false).len(), 0);
}

/// The fixtures bt24, the addon half: ours sends no `AddBehaviorTreePacket` on join, as retail
/// 1.26.45 (a development server sends one per tree with a non-empty raw text: 31 with
/// vanilla's content). Sending is not this crate's, and the packet is not part of this build; what a development server would send is every tree's `raw`, which this pins.
#[test]
fn bt24_the_payload_a_development_server_would_send_is_the_raw_text_deviation() {
    let stack = fixture_stack("bt01_vanilla_22");
    let trees = stack.behavior_trees();
    let payloads: Vec<&str> = trees.iter().map(|(_, tree)| tree.raw.as_str()).collect();
    assert_eq!(payloads.len(), 31);
    assert!(payloads.iter().all(|payload| !payload.is_empty()));
    // the payload of `collect_items` is its file, byte for byte (CRLF line ends)
    let file =
        std::fs::read(fixtures().join("bt01_vanilla_22/pack/behavior_trees/collect_items.json"))
            .unwrap();
    assert_eq!(
        trees.get(&id("collect_items")).unwrap().raw.as_bytes(),
        file
    );
    assert!(file.windows(2).any(|pair| pair == b"\r\n"));
}
