//! The tick list (`functions/tick.json`, vanilla's `_loadTickFiles` and
//! `_addTickFunctionsFromJson`): fixtures f16 to f21, the tick half of f24, and the merge.

use bedrock_addon::diagnostic::{DiagnosticCode, Severity, VanillaText};
use bedrock_addon::families::function::{QueueOrder, TickFunction};
use bedrock_addon::id::FunctionPath;
use bedrock_addon::pack::{Behavior, PackStack};

use super::common::{finding, fixture_stack, rendered};

/// The names of the stack's tick entries, in run order.
fn tick_names(stack: &PackStack<Behavior>) -> Vec<&str> {
    stack
        .tick_functions()
        .values
        .iter()
        .map(|entry| entry.function.as_str())
        .collect()
}

/// Vanilla's text for a tick entry that names no loaded function.
fn not_found(name: &str) -> String {
    format!("Function listed in tick.json with name {name} not found.")
}

/// Vanilla's text for a tick list without a `values` array.
const NO_VALUES: &str = "Field 'values' not found in tick.json or was found but is not an array, couldn't load tick functions.";

#[test]
fn f16_tick_lists_accumulate_lowest_priority_first() {
    // p0 lists `a`, p1 lists `b`: vanilla reads the tick files from the last pack up.
    let stack = fixture_stack("f16_tick_order");
    let tick = stack.tick_functions();
    assert_eq!(tick_names(&stack), ["b", "a"]);
    assert_eq!(tick.diagnostics().count(), 0);
    // Each entry belongs to the pack that listed it.
    assert_eq!(tick.origin(&0).unwrap().pack.index(), Some(1));
    assert_eq!(tick.origin(&1).unwrap().pack.index(), Some(0));
    assert!(tick.origin(&2).is_none());
    assert_eq!(
        tick.origin(&0).unwrap().path.as_str(),
        "functions/tick.json"
    );
    // The layers are each pack's file as written, top first.
    let layers: Vec<(Option<usize>, Vec<&str>)> = tick
        .layers()
        .iter()
        .map(|layer| {
            (
                layer.origin().pack.index(),
                layer.values.iter().map(|e| e.function.as_str()).collect(),
            )
        })
        .collect();
    assert_eq!(layers, [(Some(0), vec!["a"]), (Some(1), vec!["b"])]);
}

#[test]
fn f17_a_name_listed_twice_runs_twice() {
    let stack = fixture_stack("f17_tick_duplicate");
    assert_eq!(tick_names(&stack), ["a", "a"]);
    assert_eq!(stack.tick_functions().diagnostics().count(), 0);
}

#[test]
fn a_name_listed_by_two_packs_runs_twice() {
    // p0 and p1 both list `a`, which p1 defines.
    let stack = fixture_stack("x_tick_two_packs");
    assert_eq!(tick_names(&stack), ["a", "a"]);
    let tick = stack.tick_functions();
    assert_eq!(tick.origin(&0).unwrap().pack.index(), Some(1));
    assert_eq!(tick.origin(&1).unwrap().pack.index(), Some(0));
}

#[test]
fn f18_an_entry_naming_no_function_is_skipped_with_a_warning() {
    // `["missing", 5, "a", ""]`: an entry that is not a string is looked up as the empty name.
    let stack = fixture_stack("f18_tick_unknown");
    let tick = stack.tick_functions();
    assert_eq!(tick_names(&stack), ["a"]);
    // The file's own finding (the entry that is no string) comes before the stack's lookups.
    assert_eq!(
        rendered(tick.diagnostics()),
        [
            finding("UnresolvedReference", Severity::Warning, &not_found("")),
            finding(
                "UnresolvedReference",
                Severity::Warning,
                &not_found("missing")
            ),
            finding("UnresolvedReference", Severity::Warning, &not_found("")),
        ]
    );
    let paths: Vec<String> = tick
        .diagnostics()
        .map(|finding| finding.location.json_path.to_string())
        .collect();
    assert_eq!(paths, ["values/1", "values/0", "values/3"]);
    for finding in tick.diagnostics() {
        let location = &finding.location;
        assert_eq!(
            location.path.as_ref().unwrap().as_str(),
            "functions/tick.json"
        );
        assert_eq!(location.pack.as_ref().unwrap().index(), Some(0));
    }
    // The file as written keeps its string entries, found or not.
    let [layer] = tick.layers() else {
        panic!("one layer");
    };
    let written: Vec<&str> = layer.values.iter().map(|e| e.function.as_str()).collect();
    assert_eq!(written, ["missing", "a", ""]);
}

#[test]
fn a_tick_entry_must_match_a_name_byte_for_byte() {
    // `a/b` exists; `A/B`, `a//b` and `a/b/` are other names.
    let stack = fixture_stack("x_tick_exact");
    assert_eq!(tick_names(&stack), ["a/b"]);
    assert_eq!(
        rendered(stack.tick_functions().diagnostics()),
        [
            finding("UnresolvedReference", Severity::Warning, &not_found("A/B")),
            finding("UnresolvedReference", Severity::Warning, &not_found("a//b")),
            finding("UnresolvedReference", Severity::Warning, &not_found("a/b/")),
        ]
    );
}

#[test]
fn a_tick_entry_resolves_against_the_functions_vanilla_creates() {
    // p0 (1.21.0) rejects `broken` and `fallback`; p1 defines `fallback` and, without
    // `min_engine_version`, cannot run `unversioned`. p0 lists all three.
    let stack = fixture_stack("x_tick_resolution");
    // `fallback` exists through p1, `unversioned` exists in its error state (vanilla skips it
    // when the tick runs); `broken` exists nowhere.
    assert_eq!(tick_names(&stack), ["fallback", "unversioned"]);
    assert_eq!(
        rendered(stack.tick_functions().diagnostics()),
        [finding(
            "UnresolvedReference",
            Severity::Warning,
            &not_found("broken")
        )]
    );
    // The function files' own findings are the function registry's, not the tick list's.
    assert_eq!(stack.functions().rejected().len(), 2);
}

#[test]
fn f19_a_tick_list_without_a_values_array_is_skipped() {
    // p0 `{}`, p1 `{"values": null}`, p2 `{"values": "a"}`, p3 `[]`; p4 lists `a`.
    let stack = fixture_stack("f19_tick_bad_values");
    let tick = stack.tick_functions();
    assert_eq!(tick_names(&stack), ["a"]);
    // Bottom-up: p3 first.
    assert_eq!(
        rendered(tick.diagnostics()),
        [
            finding("MissingRequiredField", Severity::Error, NO_VALUES),
            finding("InvalidType", Severity::Error, NO_VALUES),
            finding("MissingRequiredField", Severity::Error, NO_VALUES),
            finding("MissingRequiredField", Severity::Error, NO_VALUES),
        ]
    );
    let packs: Vec<Option<usize>> = tick
        .diagnostics()
        .map(|finding| finding.location.pack.as_ref().unwrap().index())
        .collect();
    assert_eq!(packs, [Some(3), Some(2), Some(1), Some(0)]);
    // Only p4's file is a layer.
    assert_eq!(tick.layers().len(), 1);
}

#[test]
fn f20_a_tick_list_that_is_not_json_is_skipped() {
    // p0 `{`, p1 an array nested deeper than the reader goes; p2 lists `a`.
    let stack = fixture_stack("f20_tick_bad_json");
    let tick = stack.tick_functions();
    assert_eq!(tick_names(&stack), ["a"]);
    let findings: Vec<_> = tick.diagnostics().collect();
    assert_eq!(findings.len(), 2);
    for finding in findings {
        assert_eq!(finding.severity, Severity::Error);
        let DiagnosticCode::MalformedInput {
            message,
            text: Some(VanillaText::Formatted { args, .. }),
        } = &finding.code
        else {
            panic!("{:?}", finding.code);
        };
        // The file name is the literal `tick.json`; the second argument is the reader's errors.
        assert_eq!(args[..], ["tick.json".to_owned(), message.clone()]);
        assert!(!message.is_empty());
        assert!(
            finding
                .code
                .to_string()
                .starts_with("File tick.json failed to load with error(s) ")
        );
    }
    assert_eq!(tick.layers().len(), 1);
}

#[test]
fn f21_the_queue_order_follows_the_listing_packs_command_version() {
    // p0 (1.18.10) lists `a`, p1 (1.18.0) lists `b`.
    let stack = fixture_stack("f21_tick_queue_order");
    let tick = stack.tick_functions();
    assert_eq!(
        tick.values,
        [
            TickFunction {
                function: FunctionPath::parse("b"),
                command_version: 17,
            },
            TickFunction {
                function: FunctionPath::parse("a"),
                command_version: 18,
            },
        ]
    );
    assert_eq!(tick.values[0].queue_order(), QueueOrder::NextToExecute);
    assert_eq!(tick.values[1].queue_order(), QueueOrder::AfterCaller);
}

#[test]
fn f21_the_queue_order_changes_at_command_version_18() {
    assert_eq!(
        QueueOrder::from_command_version(17),
        QueueOrder::NextToExecute
    );
    assert_eq!(
        QueueOrder::from_command_version(18),
        QueueOrder::AfterCaller
    );
    assert_eq!(
        QueueOrder::from_command_version(8),
        QueueOrder::NextToExecute
    );
    assert_eq!(
        QueueOrder::from_command_version(46),
        QueueOrder::AfterCaller
    );
}

#[test]
fn a_tick_list_of_a_pack_without_min_engine_version_is_command_version_46() {
    let stack = fixture_stack("x_tick_no_mev");
    let tick = stack.tick_functions();
    assert_eq!(
        tick.values,
        [TickFunction {
            function: FunctionPath::parse("a"),
            command_version: 46,
        }]
    );
    assert_eq!(tick.values[0].queue_order(), QueueOrder::AfterCaller);
}

#[test]
fn f24_a_function_that_cannot_be_called_can_still_tick() {
    let stack = fixture_stack("f24_path_chars");
    assert_eq!(tick_names(&stack), ["my func"]);
    assert_eq!(stack.tick_functions().diagnostics().count(), 0);
}

#[test]
fn only_functions_tick_json_is_a_tick_list() {
    // `tick.json` at the pack root and `functions/sub/tick.json` are not read.
    let stack = fixture_stack("x_tick_place");
    let tick = stack.tick_functions();
    assert!(tick.values.is_empty());
    assert!(tick.layers().is_empty());
    assert_eq!(tick.diagnostics().count(), 0);
}

#[test]
fn a_stack_without_a_tick_list_runs_nothing() {
    let stack = fixture_stack("f01_nested_name");
    let tick = stack.tick_functions();
    assert!(tick.values.is_empty());
    assert!(tick.layers().is_empty());
    assert_eq!(tick.diagnostics().count(), 0);
}

#[test]
fn a_missing_entry_is_reported_at_its_index_in_its_own_file() {
    // p1's list is not JSON; p0 lists `[1, "missing"]` and defines nothing.
    let stack = fixture_stack("x_tick_indices");
    let tick = stack.tick_functions();
    assert!(tick.values.is_empty());
    let at: Vec<(Option<usize>, String, Severity)> = tick
        .diagnostics()
        .map(|finding| {
            (
                finding.location.pack.as_ref().unwrap().index(),
                finding.location.json_path.to_string(),
                finding.severity,
            )
        })
        .collect();
    assert_eq!(
        at,
        [
            (Some(1), String::new(), Severity::Error),
            (Some(0), "values/0".to_owned(), Severity::Warning),
            (Some(0), "values/1".to_owned(), Severity::Warning),
        ]
    );
}
