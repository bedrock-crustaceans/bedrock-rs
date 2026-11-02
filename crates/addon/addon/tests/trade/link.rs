//! The reference of a trade item to its item resolves in the link pass, through
//! `StackSet::link`.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{PackOrigin, PackRepository, PackSelection, StackSet, WorldPacks};

use super::pack_common::{Spec, bp};

/// A stack of one behaviour pack built from `spec`.
fn stacks(spec: &Spec) -> StackSet {
    let id = spec.id();
    let mut repository = PackRepository::new();
    repository.add_pack(spec.open(), PackOrigin::World);
    let world = WorldPacks {
        behavior: vec![PackSelection {
            pack_id: id.uuid,
            version: id.version,
            subpack: None,
        }],
        resource: Vec::new(),
    };
    repository.stacks(&world, LoadOptions::default())
}

/// A table that wants a vanilla item, a vanilla item with an aux suffix and an item nothing
/// defines, in both forms of an entry.
const TABLE: &[u8] =
    br#"{"format_version": "1.21.30", "tiers": [{"total_exp_required": 0, "groups": [{"trades": [{
    "wants": [
        {"item": "minecraft:wheat"},
        {"choice": [{"item": "minecraft:wool:14"}, {"item": "demo:no_such_item"}]}
    ],
    "gives": [{"item": "demo:no_such_reward:2"}]
}]}]}]}"#;

#[test]
fn t1_an_item_nothing_defines_is_an_unresolved_reference() {
    let spec = bp(1).file("trading/demo_trades.json", TABLE);
    let set = stacks(&spec);
    // The table loads: the reference is stored verbatim and checked later.
    let tables = set.behavior.trade_tables();
    assert_eq!(tables.iter().count(), 1);
    assert_eq!(tables.diagnostics().count(), 0);
    let d = set.link();
    let unresolved: Vec<String> = d
        .iter()
        .filter(|f| f.code.name() == "UnresolvedReference")
        .map(|f| {
            // The game's validator line: a content-log error.
            assert_eq!(f.severity, Severity::Error);
            assert_eq!(
                f.location
                    .path
                    .as_ref()
                    .map(bedrock_addon::storage::PackPath::as_str),
                Some("trading/demo_trades.json")
            );
            f.location.json_path.to_string()
        })
        .collect();
    // The vanilla items resolve, with or without an aux suffix; the two others do not.
    assert_eq!(
        unresolved,
        [
            "tiers/0/groups/0/trades/0/wants/1/choice/1/item",
            "tiers/0/groups/0/trades/0/gives/0/item",
        ],
        "{d:#?}"
    );
}
