//! The structure-set registry: J1, the game's registry
//! keeps the set recorded first under a name and logs "Structure set '%s' has
//! already been registered" (level 3) for a later one. The packs' sets are recorded
//! first, one per identifier (the first of each identifier is kept),
//! then the game's own `minecraft:trial_chambers`: a
//! pack's `minecraft:trial_chambers` stays effective and the game's record gets the text. J3,
//! the spacing/separation check on the parser test's own input (`parser-9.json`).

use bedrock_addon::diagnostic::{Diagnostic, Severity};
use bedrock_addon::families::jigsaw;
use bedrock_addon::families::jigsaw::StructureSet;
use bedrock_addon::id::StructureSetId;
use bedrock_addon::pack::{Behavior, PackStack};

use super::common::{document, fixture, pack, rejected, stack};

/// A valid set payload of `id`, spacing `spacing`.
fn set(id: &str, spacing: u16) -> String {
    document(
        "1.21.20",
        "minecraft:structure_set",
        &format!(
            r#"{{"description": {{"identifier": "{id}"}}, "placement": {{"type": "minecraft:random_spread", "salt": 1, "separation": 4, "spacing": {spacing}}}, "structures": [{{"structure": "demo:a", "weight": 1}}]}}"#
        ),
    )
}

/// A stack of `packs`, top first, each a list of (file name, identifier, spacing).
fn stack_of(packs: &[&[(&str, &str, u16)]]) -> PackStack<Behavior> {
    let packs = packs
        .iter()
        .enumerate()
        .map(|(i, sets)| {
            let files = sets
                .iter()
                .map(|(path, id, spacing)| {
                    (
                        format!("worldgen/structure_sets/{path}.json"),
                        set(id, *spacing).into_bytes(),
                    )
                })
                .collect();
            pack(10 * (u128::try_from(i).unwrap() + 1), files)
        })
        .collect();
    stack(packs)
}

/// The J1 findings of the group.
fn j1(stack: &PackStack<Behavior>) -> Vec<Diagnostic> {
    stack
        .jigsaw()
        .structure_sets
        .diagnostics()
        .filter(|f| f.code.is(&jigsaw::codes::SET_ALREADY_REGISTERED))
        .cloned()
        .collect()
}

#[test]
fn j1_a_pack_s_trial_chambers_set_stays_and_the_game_s_record_is_refused_with_the_text() {
    let stack = stack_of(&[&[
        ("chambers", "minecraft:trial_chambers", 34),
        ("mine", "demo:mine", 12),
    ]]);
    let jigsaw = stack.jigsaw();
    let chambers = jigsaw
        .structure_sets
        .get(&StructureSetId::parse("minecraft:trial_chambers"))
        .expect("the pack's set is effective");
    assert_eq!(chambers.placement.spacing, 34);
    assert!(jigsaw.structure_sets.rejected().is_empty());
    let found = j1(&stack);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].severity, Severity::Error);
    assert_eq!(
        found[0].code.to_string(),
        "Structure set 'minecraft:trial_chambers' has already been registered"
    );
    assert_eq!(
        found[0]
            .location
            .path
            .as_ref()
            .map(bedrock_addon::storage::PackPath::as_str),
        Some("worldgen/structure_sets/chambers.json")
    );
    assert_eq!(found[0].location.json_path.to_string(), "");
}

#[test]
fn j1_two_pack_files_of_one_identifier_are_no_collision_the_first_stays_silently() {
    // Within one pack and across two packs: the game's map of structure sets keeps the first.
    for packs in [
        &[&[("a", "demo:ruins", 12), ("b", "demo:ruins", 20)][..]][..],
        &[
            &[("a", "demo:ruins", 12)][..],
            &[("a", "demo:ruins", 20)][..],
        ][..],
    ] {
        let stack = stack_of(packs);
        assert!(j1(&stack).is_empty());
        let ruins = stack
            .jigsaw()
            .structure_sets
            .get(&StructureSetId::parse("demo:ruins"))
            .map(|set| set.placement.spacing);
        assert_eq!(ruins, Some(12));
    }
}

#[test]
fn j3_the_parser_test_s_set_with_too_much_separation_is_refused() {
    // The game's parser test for a set with too much separation (separation 4, spacing 7).
    let payload = String::from_utf8(fixture("engine/parser-9.json")).unwrap();
    let text = format!(r#"{{"format_version": "1.21.20", "minecraft:structure_set": {payload}}}"#);
    let d = rejected::<StructureSet>("worldgen/structure_sets/trail_ruins.json", &text);
    let found: Vec<_> = d
        .iter()
        .filter(|f| f.code.name() == "InvalidValue")
        .collect();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].severity, Severity::Error);
    assert_eq!(
        found[0].location.json_path.to_string(),
        "minecraft:structure_set/placement"
    );
    assert_eq!(
        found[0].code.to_string(),
        "Invalid spacing and separation combination for structure set 'minecraft:trail_ruins'. Value 'separation' must be less than half of 'spacing'."
    );
}
