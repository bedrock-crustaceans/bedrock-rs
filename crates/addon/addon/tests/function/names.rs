//! Which files are functions, what they are called and which pack's copy the stack keeps:
//! fixtures f01 to f05 and f24.

use bedrock_addon::families::function::{Function, FunctionState};
use bedrock_addon::id::FunctionPath;

use super::common::{commands, fixture_stack, function, function_names};

#[test]
fn f01_a_nested_file_is_named_by_its_path_below_functions() {
    let stack = fixture_stack("f01_nested_name");
    assert_eq!(function_names(&stack), ["a/b"]);
    let document = function(&stack, "a/b");
    assert_eq!(document.name, FunctionPath::parse("a/b"));
    assert_eq!(document.origin().path.as_str(), "functions/a/b.mcfunction");
    assert!(stack.functions().rejected().is_empty());
    assert_eq!(stack.functions().diagnostics().count(), 0);
}

#[test]
fn f02_a_name_keeps_its_case_and_is_looked_up_exactly() {
    let stack = fixture_stack("f02_case_kept");
    assert_eq!(function_names(&stack), ["Foo/Bar"]);
    let functions = stack.functions();
    assert!(functions.contains(&FunctionPath::parse("Foo/Bar")));
    // `/function foo/bar` is `commands.function.functionNameNotFound`: no such function.
    assert!(functions.get(&FunctionPath::parse("foo/bar")).is_none());
    assert!(functions.get(&FunctionPath::parse("FOO/BAR")).is_none());
}

#[test]
fn f03_only_the_exact_mcfunction_extension_is_a_function() {
    // `functions/x.MCFUNCTION` and `functions/y.txt`: ignored without a word.
    let stack = fixture_stack("f03_extension_case");
    let functions = stack.functions();
    assert!(functions.is_empty());
    assert!(functions.rejected().is_empty());
    assert_eq!(functions.diagnostics().count(), 0);
    // Nothing but the stack's own info for a pack without an icon.
    let report = stack.report();
    let names: Vec<&str> = report.iter().map(|f| f.code.id()).collect();
    assert_eq!(names, ["core/pack_icon_missing"]);
}

#[test]
fn a_file_with_an_empty_name_is_not_a_function() {
    // `functions/.mcfunction`: vanilla's name is empty, and an empty name is ignored.
    let stack = fixture_stack("x_empty_name");
    assert_eq!(function_names(&stack), ["a"]);
    assert!(stack.functions().rejected().is_empty());
    assert_eq!(stack.functions().diagnostics().count(), 0);
}

#[test]
fn f04_the_first_pack_of_the_stack_wins_without_a_message() {
    let stack = fixture_stack("f04_first_pack_wins");
    let document = function(&stack, "x");
    assert_eq!(commands(document), ["say p0"]);
    assert_eq!(document.origin().pack.index(), Some(0));
    assert_eq!(stack.functions().len(), 1);
    assert_eq!(stack.functions().diagnostics().count(), 0);
}

#[test]
fn f05_a_pack_without_min_engine_version_still_shadows_the_name() {
    // p0 has no `min_engine_version`, p1 is 1.21.0; both define `x`.
    let stack = fixture_stack("f05_shadow_without_mev");
    let document = function(&stack, "x");
    assert_eq!(document.origin().pack.index(), Some(0));
    assert_eq!(document.command_version, None);
    assert_eq!(document.state(), FunctionState::EngineVersionNotInitialized);
    assert_eq!(stack.functions().len(), 1);
}

#[test]
fn a_function_of_a_pack_with_min_engine_version_is_valid() {
    let stack = fixture_stack("f04_first_pack_wins");
    let document = function(&stack, "x");
    assert_eq!(document.command_version, Some(39));
    assert_eq!(document.state(), FunctionState::Valid);
}

#[test]
fn f24_a_name_with_a_space_loads_but_cannot_be_called() {
    // `functions/my func.mcfunction`: `/function "my func"` is
    // `commands.function.invalidCharacters`.
    let stack = fixture_stack("f24_path_chars");
    assert_eq!(function_names(&stack), ["my func"]);
    let document = function(&stack, "my func");
    assert_eq!(commands(document), ["say hi"]);
    assert!(!document.is_callable());
    assert_eq!(stack.functions().diagnostics().count(), 0);
}

/// Whether `/function` accepts a function called `name`.
fn callable(name: &str) -> bool {
    Function::new(FunctionPath::parse(name)).is_callable()
}

#[test]
fn f24_function_accepts_letters_digits_and_six_punctuation_characters() {
    assert!(callable("AZaz09"));
    assert!(callable("a/b"));
    for allowed in ["(", ")", "-", ".", "/", "_", "A", "Z", "a", "z", "0", "9"] {
        assert!(callable(allowed), "{allowed:?}");
        assert!(callable(&format!("x{allowed}y")), "{allowed:?}");
    }
}

#[test]
fn f24_function_rejects_every_other_byte() {
    // The neighbours of each accepted range, and the rest of the punctuation between them.
    for rejected in [
        " ", "!", "\"", "#", "$", "%", "&", "'", "*", "+", ",", ":", ";", "<", "=", ">", "?", "@",
        "[", "\\", "]", "^", "`", "{", "|", "}", "~", "\u{7f}", "é",
    ] {
        assert!(!callable(rejected), "{rejected:?}");
        assert!(!callable(&format!("x{rejected}y")), "{rejected:?}");
    }
}

#[test]
fn the_registry_lists_the_top_packs_functions_first() {
    // Vanilla walks the stack from index 0: p0 defines `a`, p1 defines `b`.
    let stack = fixture_stack("f16_tick_order");
    assert_eq!(function_names(&stack), ["a", "b"]);
}
