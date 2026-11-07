//! The line rules of a function file (vanilla's `_processFunctionEntry`): fixtures f06 to f15.

use bedrock_addon::diagnostic::codes as core_codes;
use bedrock_addon::diagnostic::{Code, DiagnosticDetail, Severity};
use bedrock_addon::families::function::{Function, FunctionLine, FunctionState};
use bedrock_addon::id::FunctionPath;

use super::common::{Rendered, finding, fixture_stack, function, rendered};

/// The command line `raw` at line `line`.
fn line(line: u32, raw: &str) -> FunctionLine {
    FunctionLine {
        line,
        raw: raw.to_owned(),
    }
}

/// Vanilla's header before a function's line errors.
fn header(name: &str) -> String {
    format!("Function {name} failed to load correctly with error(s):")
}

#[test]
fn f06_comments_and_blank_lines_are_not_commands() {
    // `# c`, `   # c`, an empty line, `   `, then `say hi` on line 5.
    let stack = fixture_stack("f06_comments_blank");
    let document = function(&stack, "x");
    assert_eq!(
        *document.value(),
        Function {
            name: FunctionPath::parse("x"),
            command_version: Some(39),
            commands: vec![line(5, "say hi")],
            dropped: Vec::new(),
        }
    );
    assert!(document.diagnostics().is_empty());
    assert_eq!(stack.functions().diagnostics().count(), 0);
}

#[test]
fn f10_below_command_version_12_a_bad_line_is_dropped_and_the_function_kept() {
    // min_engine_version 1.15.0 (command version 11): `say a`, `bogus`, `say b`.
    let stack = fixture_stack("f10_lenient_1_15");
    let document = function(&stack, "x");
    assert_eq!(
        *document.value(),
        Function {
            name: FunctionPath::parse("x"),
            command_version: Some(11),
            commands: vec![line(1, "say a"), line(3, "say b")],
            dropped: vec![line(2, "bogus")],
        }
    );
    // The errors are still logged, at vanilla's level (Warning), header first.
    assert_eq!(
        rendered(document.diagnostics()),
        [
            finding(
                "function/failed_header_dropped",
                Severity::Warning,
                &header("x")
            ),
            finding(
                "function/unknown_command",
                Severity::Warning,
                "Error on line 2: unknown or invalid command 'bogus'"
            ),
        ]
    );
    assert!(stack.functions().rejected().is_empty());
    // A function kept despite errors is not in the `FunctionName` soft enum.
    assert!(!document.is_auto_completed());
}

#[test]
fn a_function_without_a_dropped_line_is_auto_completed() {
    let stack = fixture_stack("f06_comments_blank");
    assert!(function(&stack, "x").is_auto_completed());
}

#[test]
fn f11_from_command_version_12_a_bad_line_rejects_the_function() {
    // The file of f10 under min_engine_version 1.16.0 (command version 12).
    let stack = fixture_stack("f11_strict_1_16");
    let functions = stack.functions();
    assert!(functions.get(&FunctionPath::parse("x")).is_none());
    assert!(functions.is_empty());
    let [rejected] = functions.rejected() else {
        panic!("one rejected file, not {}", functions.rejected().len());
    };
    assert_eq!(rejected.origin.path.as_str(), "functions/x.mcfunction");
    // The same errors as f10; the function is not created, so they are errors here.
    let expected = [
        finding("function/failed_header", Severity::Error, &header("x")),
        finding(
            "function/unknown_command",
            Severity::Error,
            "Error on line 2: unknown or invalid command 'bogus'",
        ),
    ];
    assert_eq!(rendered(&rejected.diagnostics), expected);
    assert_eq!(rendered(functions.diagnostics()), expected);
}

/// The findings of the one file fixture `name` rejects, header included.
fn rejected_findings(name: &str) -> Vec<Rendered> {
    let stack = fixture_stack(name);
    let functions = stack.functions();
    assert!(functions.is_empty(), "{name}: the function is absent");
    let [rejected] = functions.rejected() else {
        panic!(
            "{name}: one rejected file, not {}",
            functions.rejected().len()
        );
    };
    rendered(&rejected.diagnostics)
}

/// The unknown-command error with `text`.
fn unknown(severity: Severity, text: &str) -> Rendered {
    finding("function/unknown_command", severity, text)
}

/// The unknown-command error with the comment hint and `text`.
fn unknown_comment(severity: Severity, text: &str) -> Rendered {
    finding("function/unknown_command_comment", severity, text)
}

/// The unknown-command error with the slash hint and `text`.
fn unknown_slash(severity: Severity, text: &str) -> Rendered {
    finding("function/unknown_command_slash", severity, text)
}

#[test]
fn f07_a_double_slash_line_gets_the_comment_hint() {
    // `// note` under min_engine_version 1.16.0.
    assert_eq!(
        rejected_findings("f07_double_slash"),
        [
            finding("function/failed_header", Severity::Error, &header("x")),
            unknown_comment(
                Severity::Error,
                "Error on line 1: unknown or invalid command '//' (If you intended to make a comment use '#' instead of '//')"
            ),
        ]
    );
}

#[test]
fn f08_a_leading_slash_gets_the_slash_hint() {
    // `/say hi` under min_engine_version 1.16.0.
    assert_eq!(
        rejected_findings("f08_leading_slash"),
        [
            finding("function/failed_header", Severity::Error, &header("x")),
            unknown_slash(
                Severity::Error,
                "Error on line 1: unknown or invalid command '/say' (Commands in functions may not begin with '/')"
            ),
        ]
    );
}

#[test]
fn f09_a_lone_slash_gets_no_hint() {
    // `/ say hi`: the first word is `/`, one byte.
    assert_eq!(
        rejected_findings("f09_lone_slash"),
        [
            finding("function/failed_header", Severity::Error, &header("x")),
            unknown(
                Severity::Error,
                "Error on line 1: unknown or invalid command '/'"
            ),
        ]
    );
}

#[test]
fn the_hints_look_at_the_first_word_only() {
    // `//note x`, `/x`, `/`, `say // no`, `say /no`, `x//`, `x/` under 1.15.0: every line is
    // read, the bad ones dropped.
    let stack = fixture_stack("x_hints");
    let document = function(&stack, "x");
    assert_eq!(
        rendered(document.diagnostics())[1..],
        [
            unknown_comment(
                Severity::Warning,
                "Error on line 1: unknown or invalid command '//note' (If you intended to make a comment use '#' instead of '//')"
            ),
            unknown_slash(
                Severity::Warning,
                "Error on line 2: unknown or invalid command '/x' (Commands in functions may not begin with '/')"
            ),
            unknown(
                Severity::Warning,
                "Error on line 3: unknown or invalid command '/'"
            ),
            unknown(
                Severity::Warning,
                "Error on line 6: unknown or invalid command 'x//'"
            ),
            unknown(
                Severity::Warning,
                "Error on line 7: unknown or invalid command 'x/'"
            ),
        ]
    );
    assert_eq!(
        document.commands,
        [line(4, "say // no"), line(5, "say /no")]
    );
}

#[test]
fn f12_an_error_names_the_physical_line() {
    // A blank line, a comment, then `bogus` on line 3.
    assert_eq!(
        rejected_findings("f12_line_numbers"),
        [
            finding("function/failed_header", Severity::Error, &header("x")),
            unknown(
                Severity::Error,
                "Error on line 3: unknown or invalid command 'bogus'"
            ),
        ]
    );
}

#[test]
fn f13_every_bad_line_is_reported_after_one_header() {
    // `bogus`, `say ok`, `worse`.
    assert_eq!(
        rejected_findings("f13_all_errors_reported"),
        [
            finding("function/failed_header", Severity::Error, &header("a/b")),
            unknown(
                Severity::Error,
                "Error on line 1: unknown or invalid command 'bogus'"
            ),
            unknown(
                Severity::Error,
                "Error on line 3: unknown or invalid command 'worse'"
            ),
        ]
    );
}

#[test]
fn f14_a_bom_and_crlf_file_loads_like_the_lf_file() {
    // `crlf` is the bytes of `lf` behind a UTF-8 BOM, every line ended by CR LF.
    let stack = fixture_stack("f14_crlf_bom");
    let crlf = function(&stack, "crlf");
    let lf = function(&stack, "lf");
    assert_eq!(lf.commands, [line(1, "say a"), line(4, "say b")]);
    assert_eq!(crlf.commands, lf.commands);
    assert!(crlf.dropped.is_empty());
    assert_eq!(stack.functions().diagnostics().count(), 0);
    assert!(stack.functions().rejected().is_empty());
}

#[test]
fn f15_without_min_engine_version_no_line_is_checked() {
    // `bogus`, `say hi`, `// x` in a pack without `min_engine_version`: vanilla never reads the
    // file. The lines are kept, unchecked, for write-back.
    let stack = fixture_stack("f15_no_mev");
    let document = function(&stack, "x");
    assert_eq!(
        *document.value(),
        Function {
            name: FunctionPath::parse("x"),
            command_version: None,
            commands: vec![line(1, "bogus"), line(2, "say hi"), line(3, "// x")],
            dropped: Vec::new(),
        }
    );
    // `/function x` is `commands.function.noEngineVersionSpecified`.
    assert_eq!(document.state(), FunctionState::EngineVersionNotInitialized);
    assert!(document.diagnostics().is_empty());
    assert_eq!(stack.functions().diagnostics().count(), 0);
    // Such a function gets no auto-completion.
    assert!(!document.is_auto_completed());
}

#[test]
fn a_line_is_trimmed_of_ascii_whitespace_and_split_at_the_first_space() {
    // Under 1.15.0, so every line is read:
    // 1 `say<TAB>hi`: only a space ends the first word;
    // 2 `<TAB>say tabbed<TAB>`, 3 `  say padded  `, 6 `<VT>say vt<FF>`: trimmed for the check,
    //   kept as written;
    // 4 `say hi # not a comment`: only a leading `#` is a comment;
    // 5 `<NBSP>say nbsp`: a no-break space is not trimmed.
    let stack = fixture_stack("x_words");
    let document = function(&stack, "x");
    assert_eq!(
        document.commands,
        [
            line(2, "\tsay tabbed\t"),
            line(3, "  say padded  "),
            line(4, "say hi # not a comment"),
            line(6, "\u{b}say vt\u{c}"),
        ]
    );
    assert_eq!(
        document.dropped,
        [line(1, "say\thi"), line(5, "\u{a0}say nbsp")]
    );
    assert_eq!(
        rendered(document.diagnostics())[1..],
        [
            unknown(
                Severity::Warning,
                "Error on line 1: unknown or invalid command 'say\thi'"
            ),
            unknown(
                Severity::Warning,
                "Error on line 5: unknown or invalid command '\u{a0}say'"
            ),
        ]
    );
    // The command and its name are the trimmed line and its first word.
    assert_eq!(document.commands[0].command(), "say tabbed");
    assert_eq!(document.commands[0].name(), "say");
    assert_eq!(document.commands[2].command(), "say hi # not a comment");
    assert_eq!(document.dropped[0].name(), "say\thi");
}

#[test]
fn a_command_is_found_by_name_or_alias_whatever_its_case() {
    // `SAY upper`, `tp @s ~ ~ ~`, `? say`, `Tp @s ~ ~ ~` under 1.21.0.
    let stack = fixture_stack("x_case_alias");
    let document = function(&stack, "x");
    assert_eq!(document.commands.len(), 4);
    assert!(document.diagnostics().is_empty());
    assert!(stack.functions().rejected().is_empty());
}

#[test]
fn a_namespaced_first_word_is_kept_with_an_information() {
    // `demo:custom arg` then `say hi` under 1.21.0: only a script can register a namespaced
    // command, which nothing here can know, so the line is kept and the function loads.
    let stack = fixture_stack("x_custom_command");
    let document = function(&stack, "x");
    assert_eq!(
        document.commands,
        [line(1, "demo:custom arg"), line(2, "say hi")]
    );
    assert!(document.dropped.is_empty());
    let [warning] = document.diagnostics().iter().collect::<Vec<_>>()[..] else {
        panic!("one finding, not {}", document.diagnostics().len());
    };
    assert_eq!(warning.severity, Severity::Info);
    assert_eq!(warning.code, Code::from(&core_codes::UNRESOLVED_REFERENCE));
    assert_eq!(
        warning.detail.as_deref(),
        Some(&DiagnosticDetail::Reference {
            kind: "command",
            id: "demo:custom".to_owned(),
            hint: None,
        })
    );
    assert!(document.is_auto_completed());
}

#[test]
fn a_file_that_is_not_utf8_loads_with_replacement_characters_deviation() {
    // `say caf<0xE9>`, `say ok`: vanilla keeps the bytes; a `String` cannot.
    let stack = fixture_stack("x_not_utf8");
    let document = function(&stack, "x");
    assert_eq!(
        document.commands,
        [line(1, "say caf\u{fffd}"), line(2, "say ok")]
    );
    let [warning] = document.diagnostics().iter().collect::<Vec<_>>()[..] else {
        panic!("one finding, not {}", document.diagnostics().len());
    };
    assert_eq!(warning.severity, Severity::Info);
    assert!(warning.code.is(&core_codes::SYNTAX));
}

#[test]
fn an_empty_or_comment_only_file_is_a_function_without_commands() {
    let stack = fixture_stack("x_shapes");
    for name in ["empty", "comments"] {
        let document = function(&stack, name);
        assert!(document.commands.is_empty(), "{name}");
        assert_eq!(document.state(), FunctionState::Valid, "{name}");
        assert!(document.is_auto_completed(), "{name}");
    }
    // The last line needs no line ending.
    assert_eq!(
        function(&stack, "no_newline").commands,
        [line(1, "say a"), line(2, "say b")]
    );
    assert_eq!(stack.functions().diagnostics().count(), 0);
}

#[test]
fn a_line_finding_carries_the_span_of_its_line() {
    // A BOM, `say a` CR LF, `  bogus  ` CR LF, `say b`, under 1.15.0.
    let stack = fixture_stack("x_span");
    let document = function(&stack, "x");
    let bytes =
        std::fs::read(super::common::fixture_dir("x_span").join("p0/functions/x.mcfunction"))
            .unwrap();
    let [header, bogus] = document.diagnostics().iter().collect::<Vec<_>>()[..] else {
        panic!("two findings, not {}", document.diagnostics().len());
    };
    assert_eq!(header.location.span, None);
    let span = bogus.location.span.expect("the line's span");
    assert_eq!((span.offset, span.len), (10, 9));
    let range = span.offset as usize..(span.offset + span.len) as usize;
    assert_eq!(&bytes[range], b"  bogus  ");
    assert_eq!(
        bogus.location.path.as_ref().unwrap().as_str(),
        "functions/x.mcfunction"
    );
    assert_eq!(bogus.location.pack.as_ref().unwrap().index(), Some(0));
}

#[test]
fn the_six_ascii_whitespace_characters_are_trimmed_and_nothing_else() {
    // Space, tab, line feed, vertical tab, form feed and carriage return, at either end.
    let all = FunctionLine::new(" \t\n\u{b}\u{c}\rsay hi \t\n\u{b}\u{c}\r");
    assert_eq!(all.command(), "say hi");
    assert_eq!(all.name(), "say");
    for space in [' ', '\t', '\n', '\u{b}', '\u{c}', '\r'] {
        let line = FunctionLine::new(format!("{space}list{space}"));
        assert_eq!(line.command(), "list", "{space:?}");
        assert_eq!(line.name(), "list", "{space:?}");
    }
    // Other white space is part of the command: a no-break space, a line separator.
    for other in ['\u{a0}', '\u{2028}', '\u{3000}'] {
        let raw = format!("{other}list{other}");
        assert_eq!(FunctionLine::new(raw.clone()).command(), raw, "{other:?}");
    }
}

/// The commands vanilla registers only with a world feature the public command table does not
/// cover (education, developer builds).
const FEATURE_GATED: [&str; 7] = [
    "ability",
    "agent",
    "immutableworld",
    "worldbuilder",
    "classroommode",
    "code",
    "summon_noai",
];

#[test]
fn a_feature_gated_command_is_kept_with_an_information_deviation() {
    // One line per gated command, then `say hi`: `new` under min_engine_version 1.21.0 (command
    // version 39, where an unknown command rejects the function) and `old` under 1.15.0 (11,
    // where it drops the line). Vanilla's `isValidCommand` knows these names when the world
    // has the feature; the table does not, so neither rule may fire.
    let stack = fixture_stack("x_gated_commands");
    for (name, version) in [("new", 39), ("old", 11)] {
        let document = function(&stack, name);
        assert_eq!(document.command_version, Some(version), "{name}");
        assert_eq!(document.commands.len(), 8, "{name}");
        assert!(document.dropped.is_empty(), "{name}");
        let warned: Vec<&str> = document
            .diagnostics()
            .iter()
            .map(|finding| {
                assert_eq!(finding.severity, Severity::Info, "{name}");
                assert_eq!(
                    finding.code,
                    Code::from(&core_codes::UNRESOLVED_REFERENCE),
                    "{name}"
                );
                assert!(finding.location.span.is_some(), "{name}");
                let Some(DiagnosticDetail::Reference { kind, id, hint }) =
                    finding.detail.as_deref()
                else {
                    panic!("{name}: no reference detail");
                };
                assert_eq!(*kind, "feature-gated command", "{name}");
                assert_eq!(*hint, None, "{name}");
                id.as_str()
            })
            .collect();
        assert_eq!(warned, FEATURE_GATED, "{name}");
        assert!(document.is_auto_completed(), "{name}");
    }
    assert!(stack.functions().rejected().is_empty());
}

#[test]
fn a_word_that_only_resembles_a_feature_gated_command_is_unknown() {
    // `agents`, `Agent x` and `abilit` under 1.21.0: the list is exact names, case folded as
    // the table's are.
    let stack = fixture_stack("x_gated_commands_near");
    let functions = stack.functions();
    assert!(functions.get(&FunctionPath::parse("near")).is_none());
    let [rejected] = functions.rejected() else {
        panic!("one rejected file");
    };
    // The header, the two errors, then the warning for `Agent`.
    assert_eq!(rejected.diagnostics.len(), 4);
    assert_eq!(
        rendered(&rejected.diagnostics)[1..3],
        [
            unknown(
                Severity::Error,
                "Error on line 1: unknown or invalid command 'agents'"
            ),
            unknown(
                Severity::Error,
                "Error on line 3: unknown or invalid command 'abilit'"
            ),
        ]
    );
}
