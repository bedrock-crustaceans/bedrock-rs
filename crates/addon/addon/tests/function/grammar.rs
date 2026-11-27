//! The command version as the grammar gate: fixture f22 and the table of overload ranges
//!.

use bedrock_addon::families::function::{GrammarRange, grammar_ranges};
use bedrock_addon::vanilla;

use super::common::{commands, fixture_stack, function};

/// The ranges of `command`.
fn ranges_of(command: &str) -> Vec<&'static GrammarRange> {
    grammar_ranges()
        .iter()
        .filter(|range| range.command == command)
        .collect()
}

#[test]
fn f22_each_function_carries_the_command_version_that_selects_its_grammar() {
    // `setblock ~ ~ ~ stone 1` in `old` (p1, min_engine_version 1.19.60) and in `new` (p0,
    // 1.19.70). Vanilla parses the first and not the second, which it does not create. Parsing
    // is the command system's: both functions load here, each with the command
    // version that decides, and the table says what it decides.
    let stack = fixture_stack("f22_grammar_gate");
    let old = function(&stack, "old");
    let new = function(&stack, "new");
    assert_eq!(old.command_version, Some(26));
    assert_eq!(new.command_version, Some(32));
    assert_eq!(commands(old), ["setblock ~ ~ ~ stone 1"]);
    assert_eq!(commands(new), ["setblock ~ ~ ~ stone 1"]);
    assert_eq!(stack.functions().diagnostics().count(), 0);
    // The data-value form of `setblock` is registered for command versions 1 to 28.
    let [data_value] = ranges_of("setblock")[..] else {
        panic!("one setblock range");
    };
    assert_eq!((data_value.from, data_value.to), (1, Some(28)));
    assert!(data_value.contains(old.command_version.unwrap()));
    assert!(!data_value.contains(new.command_version.unwrap()));
}

/// The table: command, first version, last version (`None`: no upper bound), overloads.
const RANGES: [(&str, u8, Option<u8>, u8); 18] = [
    ("execute", 1, Some(24), 1),
    ("execute", 25, Some(46), 1),
    ("execute", 25, Some(28), 1),
    ("setblock", 1, Some(28), 1),
    ("fill", 1, Some(28), 2),
    ("clone", 1, Some(28), 1),
    ("testforblock", 1, Some(28), 1),
    ("locate", 1, Some(20), 1),
    ("locate", 21, None, 2),
    ("structure", 1, Some(21), 4),
    ("structure", 22, Some(46), 3),
    ("summon", 1, Some(30), 1),
    ("summon", 31, None, 3),
    ("summon_noai", 31, None, 5),
    ("damage", 1, Some(37), 4),
    ("damage", 38, Some(46), 3),
    ("locate", 44, None, 2),
    ("agent", 1, Some(46), 4),
];

#[test]
fn the_ranges_are_the_forty_overloads_vanilla_registers_for_part_of_the_versions() {
    let listed: Vec<(&str, u8, Option<u8>, u8)> = grammar_ranges()
        .iter()
        .map(|range| (range.command, range.from, range.to, range.overloads))
        .collect();
    assert_eq!(listed, RANGES);
    // 704 call sites register an overload; 664 of them for every version.
    let overloads: u32 = grammar_ranges()
        .iter()
        .map(|range| u32::from(range.overloads))
        .sum();
    assert_eq!(overloads, 40);
    for range in grammar_ranges() {
        assert!(!range.syntax.is_empty(), "{}", range.command);
    }
}

#[test]
fn a_range_holds_both_its_ends() {
    let [legacy, chain, retired] = ranges_of("execute")[..] else {
        panic!("three execute ranges");
    };
    // The legacy `execute <target> <position> <command>`: 1 to 24.
    assert!(!legacy.contains(0));
    assert!(legacy.contains(1));
    assert!(legacy.contains(24));
    assert!(!legacy.contains(25));
    // The sub-command chain: 25 to 46.
    assert!(!chain.contains(24));
    assert!(chain.contains(25));
    assert!(chain.contains(46));
    assert!(!chain.contains(47));
    assert!(retired.contains(28));
    assert!(!retired.contains(29));
    // No upper bound: `summon` with a rotation, from 31.
    let rotation = ranges_of("summon")[1];
    assert!(!rotation.contains(30));
    assert!(rotation.contains(31));
    assert!(rotation.contains(u8::MAX));
}

#[test]
fn the_ranged_commands_are_commands_of_the_vanilla_table() {
    for range in grammar_ranges() {
        // The developer command and the Education Edition command are not in the public table.
        let listed = !matches!(range.command, "summon_noai" | "agent");
        assert_eq!(
            vanilla::command(range.command).is_some(),
            listed,
            "{}",
            range.command
        );
    }
}
