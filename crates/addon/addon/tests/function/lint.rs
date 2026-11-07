//! The lints against the core `vanilla` command table: the unknown command, which
//! is vanilla's own check (fixtures f07 to f13 in `lines.rs`), and the arity of a line, which
//! is ours and only informs.

use bedrock_addon::diagnostic::codes as core_codes;
use bedrock_addon::diagnostic::{Diagnostic, DiagnosticCode, Severity};
use bedrock_addon::document::Content;
use bedrock_addon::families::function::{Function, FunctionLine};
use bedrock_addon::load::LoadContext;
use bedrock_addon::storage::PackPath;
use bedrock_addon::vanilla;
use bedrock_addon::version::MinEngineVersion;
use std::fmt::Write as _;

use super::common::{commands, fixture_dir, fixture_stack, function};

/// The text of each line `findings` point at, through their spans in `file` of the top pack.
fn lines_of<'a>(
    fixture: &str,
    pack: &str,
    file: &str,
    findings: impl IntoIterator<Item = &'a Diagnostic>,
) -> Vec<String> {
    let bytes = std::fs::read(fixture_dir(fixture).join(pack).join(file)).unwrap();
    findings
        .into_iter()
        .map(|finding| {
            assert_eq!(finding.severity, Severity::Info);
            assert_eq!(
                finding.code,
                DiagnosticCode::from(&core_codes::INVALID_VALUE)
            );
            let span = finding.location.span.expect("a line span");
            let range = span.offset as usize..(span.offset + span.len) as usize;
            String::from_utf8(bytes[range].to_vec()).unwrap()
        })
        .collect()
}

#[test]
fn a_line_no_overload_can_take_is_an_information_and_stays_a_command() {
    // `say`, `list now`, `tp @s ~ ~ ~`, `kill @e[type=cow, r=5]`, `say he said "hi`,
    // `setblock ~ ~ ~`, `TP`, under min_engine_version 1.21.0 (command version 39).
    let stack = fixture_stack("x_arity");
    let document = function(&stack, "new");
    assert_eq!(document.commands.len(), 7);
    assert!(document.dropped.is_empty());
    assert_eq!(
        lines_of(
            "x_arity",
            "p0",
            "functions/new.mcfunction",
            document.diagnostics()
        ),
        ["say", "list now", "setblock ~ ~ ~", "TP"]
    );
    // The lint decides nothing: the function loads and is listed.
    assert!(document.is_auto_completed());
    assert!(stack.functions().rejected().is_empty());
}

#[test]
fn a_command_with_older_overloads_is_not_judged_under_an_older_command_version() {
    // `setblock ~ ~ ~` and `say` under min_engine_version 1.19.60 (command version 26):
    // `setblock` had other overloads up to 28.
    let stack = fixture_stack("x_arity");
    let document = function(&stack, "old");
    assert_eq!(commands(document), ["setblock ~ ~ ~", "say"]);
    assert_eq!(
        lines_of(
            "x_arity",
            "p1",
            "functions/old.mcfunction",
            document.diagnostics()
        ),
        ["say"]
    );
}

#[test]
fn the_lines_of_a_function_without_command_version_are_not_judged() {
    let stack = fixture_stack("x_arity");
    let document = function(&stack, "unversioned");
    assert_eq!(commands(document), ["say", "list now"]);
    assert!(document.diagnostics().is_empty());
}

#[test]
fn the_unknown_command_check_reads_the_core_command_table() {
    // Every command of the table, by its name and by each alias, is a known first word: a
    // file with one line per name loads with every line kept, under the current command
    // version (where one unknown word would reject it) and under 11 (where it would be dropped).
    let names: Vec<&str> = vanilla::commands()
        .iter()
        .flat_map(|command| std::iter::once(command.name).chain(command.aliases.iter().copied()))
        .collect();
    assert_eq!(vanilla::commands().len(), 84);
    assert!(names.len() > 84, "the aliases are run too");
    assert!(names.contains(&"tp") && names.contains(&"?"));
    let text = names.iter().fold(String::new(), |mut out, name| {
        writeln!(out, "{name} x").unwrap();
        out
    });
    let path = PackPath::new("functions/every.mcfunction").unwrap();
    let mut old = LoadContext::detached();
    old.min_engine_version = Some(MinEngineVersion::new(1, 15, 0));
    for cx in [LoadContext::detached(), old] {
        let document = Function::parse(&path, text.as_bytes(), &cx)
            .unwrap_or_else(|rejected| panic!("{rejected}"));
        let words: Vec<&str> = document.commands.iter().map(FunctionLine::name).collect();
        assert_eq!(words, names);
        assert!(document.dropped.is_empty());
        // Nothing but the arity lint may speak.
        for finding in document.diagnostics() {
            assert_eq!(
                finding.code,
                DiagnosticCode::from(&core_codes::INVALID_VALUE)
            );
        }
    }
    // The same file with one word outside the table is rejected.
    let bad = format!("{text}bogus x\n");
    assert!(Function::parse(&path, bad.as_bytes(), &LoadContext::detached()).is_err());
}

/// A host whose provider has no `say`: the first word is asked of the context's provider, so
/// a `say` line is an unknown command, rejected under the current command version as a word
/// outside the table is.
struct NoSay;

impl bedrock_addon::load::BuiltIns for NoSay {
    fn contains(&self, _kind: &'static str, _id: &str) -> bool {
        false
    }
    fn command(&self, name: &str) -> Option<&vanilla::Command> {
        if name.eq_ignore_ascii_case("say") {
            None
        } else {
            vanilla::GeneratedBuiltIns.command(name)
        }
    }
}

#[test]
fn the_unknown_command_check_asks_the_context_s_provider() {
    let path = PackPath::new("functions/hello.mcfunction").unwrap();
    let text = b"say hi\ntp @s ~ ~ ~\n";
    let tables = Function::parse(&path, text, &LoadContext::detached())
        .unwrap_or_else(|rejected| panic!("{rejected}"));
    assert_eq!(commands(&tables), ["say hi", "tp @s ~ ~ ~"]);
    let mut hosted = LoadContext::detached();
    hosted.set_built_ins(Some(std::sync::Arc::new(NoSay)));
    assert!(Function::parse(&path, text, &hosted).is_err());
    // Below command version 12 the line is dropped and the function kept.
    hosted.min_engine_version = Some(MinEngineVersion::new(1, 15, 0));
    let kept =
        Function::parse(&path, text, &hosted).unwrap_or_else(|rejected| panic!("{rejected}"));
    assert_eq!(commands(&kept), ["tp @s ~ ~ ~"]);
    assert_eq!(kept.dropped.len(), 1);
}
