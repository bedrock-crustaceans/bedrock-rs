//! Single files through `Content::parse` and back: the Lines level of a function (its command
//! lines only), the Value level of a tick list, the default paths and the files that are no
//! function.

use bedrock_addon::diagnostic::codes as core_codes;
use bedrock_addon::diagnostic::{Code, JsonPath, Severity};
use bedrock_addon::document::{Content, Placed};
use bedrock_addon::families::function::{
    Function, FunctionLine, FunctionState, TickFunction, TickList,
};
use bedrock_addon::id::FunctionPath;
use bedrock_addon::json::{EncodeError, Unrepresentable, Value, WriteStyle};
use bedrock_addon::load::LoadContext;
use bedrock_addon::storage::PackPath;

use super::common::fixture_dir;

/// The function file `text` at `functions/<name>.mcfunction` of a pack loaded with `cx`.
fn parse_with(name: &str, text: &[u8], cx: &LoadContext) -> Function {
    let path = PackPath::new(&format!("functions/{name}.mcfunction")).unwrap();
    Function::parse(&path, text, cx)
        .unwrap_or_else(|rejected| panic!("{rejected}"))
        .into_value()
}

/// The function file `text` of a pack at the engine version.
fn parse(name: &str, text: &[u8]) -> Function {
    parse_with(name, text, &LoadContext::detached())
}

/// The bytes of `path` in fixture `fixture`'s top pack.
fn fixture_file(fixture: &str, path: &str) -> Vec<u8> {
    std::fs::read(fixture_dir(fixture).join("p0").join(path)).unwrap()
}

/// Writes `function`, reads the text back and requires the second writing to be identical.
fn rewritten(function: &Function) -> (String, Function) {
    let text = function.to_json().unwrap();
    let again = parse(function.name.as_str(), text.as_bytes());
    assert_eq!(again.to_json().unwrap(), text, "the rewrite differs");
    (text, again)
}

/// The raw texts of `function`'s command lines.
fn raws(function: &Function) -> Vec<&str> {
    function
        .commands
        .iter()
        .map(|line| line.raw.as_str())
        .collect()
}

#[test]
fn a_vanilla_function_round_trips_byte_for_byte() {
    // `functions/scriptevent/tick.mcfunction` of the GDK test asset pack
    // `ArchivedPackLoadingUnbakedAssets/BP`, which has no final line ending.
    let capture = b"scriptevent demo:tickjson message from tick.json";
    let function = parse("scriptevent/tick", capture);
    assert_eq!(function.name, FunctionPath::parse("scriptevent/tick"));
    assert_eq!(function.command_version, Some(46));
    assert_eq!(function.commands.len(), 1);
    assert_eq!(function.commands[0].name(), "scriptevent");
    let style = WriteStyle {
        trailing_newline: false,
        ..WriteStyle::default()
    };
    let mut written = Vec::new();
    function.write_json(&mut written, style).unwrap();
    assert_eq!(written, capture, "the writing differs from the capture");
}

#[test]
fn a_function_is_written_as_its_command_lines() {
    let text = "say a\ntp @s ~ ~1 ~\nsay b\n";
    let function = parse("x", text.as_bytes());
    assert_eq!(function.to_json().unwrap(), text);
    let (_, again) = rewritten(&function);
    assert_eq!(again, function);
}

#[test]
fn comments_and_blank_lines_are_not_written() {
    // `# c`, `   # c`, an empty line, `   `, `say hi`.
    let function = parse(
        "x",
        &fixture_file("f06_comments_blank", "functions/x.mcfunction"),
    );
    let (text, again) = rewritten(&function);
    assert_eq!(text, "say hi\n");
    // The command is the same; its line number is the written file's.
    assert_eq!(raws(&again), raws(&function));
    assert_eq!(function.commands[0].line, 5);
    assert_eq!(again.commands[0].line, 1);
}

#[test]
fn a_line_is_written_as_written_untrimmed() {
    let function = parse("x", b"  say padded  \r\n\tsay tabbed\t\r\n");
    let (text, again) = rewritten(&function);
    // CR LF is written as LF.
    assert_eq!(text, "  say padded  \n\tsay tabbed\t\n");
    assert_eq!(again, function);
}

#[test]
fn dropped_lines_are_not_written() {
    // `say a`, `bogus`, `say b` under min_engine_version 1.15.0.
    let mut cx = LoadContext::detached();
    cx.min_engine_version = Some(bedrock_addon::version::MinEngineVersion::new(1, 15, 0));
    let bytes = fixture_file("f10_lenient_1_15", "functions/x.mcfunction");
    let function = parse_with("x", &bytes, &cx);
    assert_eq!(function.dropped.len(), 1);
    assert_eq!(function.to_json().unwrap(), "say a\nsay b\n");
}

#[test]
fn the_unchecked_lines_of_a_function_without_command_version_are_written() {
    let mut cx = LoadContext::detached();
    cx.min_engine_version = None;
    let function = parse_with("x", b"bogus\nsay hi\n", &cx);
    assert_eq!(function.state(), FunctionState::EngineVersionNotInitialized);
    assert_eq!(function.to_json().unwrap(), "bogus\nsay hi\n");
}

#[test]
fn the_trailing_newline_follows_the_style() {
    let mut function = Function::new(FunctionPath::parse("x"));
    function.commands.push(FunctionLine::new("say a"));
    function.commands.push(FunctionLine::new("say b"));
    let write = |function: &Function, trailing_newline| {
        let mut out = b"kept".to_vec();
        let style = WriteStyle {
            trailing_newline,
            ..WriteStyle::default()
        };
        function.write_json(&mut out, style).unwrap();
        String::from_utf8(out).unwrap()
    };
    assert_eq!(write(&function, true), "keptsay a\nsay b\n");
    assert_eq!(write(&function, false), "keptsay a\nsay b");
    // A function without commands is an empty file either way.
    let empty = Function::new(FunctionPath::parse("x"));
    assert_eq!(write(&empty, true), "kept");
    assert_eq!(write(&empty, false), "kept");
}

#[test]
fn a_function_built_in_code_reads_back_as_built() {
    let mut function = Function::new(FunctionPath::parse("a/b"));
    assert_eq!(function.command_version, Some(46));
    assert_eq!(function.state(), FunctionState::Valid);
    function.commands.push(FunctionLine::new("say a"));
    assert_eq!(function.commands[0].line, 0);
    let (_, again) = rewritten(&function);
    assert_eq!(raws(&again), ["say a"]);
    assert_eq!(again.name, function.name);
}

#[test]
fn a_line_that_would_not_read_back_as_one_command_is_not_written() {
    // A line break splits the command; a blank line and a `#` line read back as nothing; a
    // final CR reads back as part of the line ending.
    for bad in ["say a\nsay b", "", "   ", "# say", "  # say", "say a\r"] {
        let mut function = Function::new(FunctionPath::parse("x"));
        function.commands.push(FunctionLine::new("say ok"));
        function.commands.push(FunctionLine::new(bad));
        let mut out = b"kept".to_vec();
        let error = function
            .write_json(&mut out, WriteStyle::default())
            .expect_err(bad);
        let EncodeError::NotRepresentable { at, what } = error else {
            panic!("{error:?}");
        };
        assert_eq!(what, Unrepresentable::Unsupported, "{bad:?}");
        let mut expected = JsonPath::new();
        expected.push("commands");
        expected.push_index(1);
        assert_eq!(at, expected, "{bad:?}");
        // Nothing is appended on failure.
        assert_eq!(out, b"kept", "{bad:?}");
    }
    // A CR inside a line is part of the line: it reads back as written.
    let function = parse("x", b"say a\rb\n");
    assert_eq!(raws(&function), ["say a\rb"]);
    assert_eq!(function.to_json().unwrap(), "say a\rb\n");
}

#[test]
fn only_a_file_at_functions_name_mcfunction_parses_as_a_function() {
    let cx = LoadContext::detached();
    for path in [
        "a.mcfunction",
        "functions/a.txt",
        "functions/a.MCFUNCTION",
        "functions/.mcfunction",
        "Functions/a.mcfunction",
        "other/functions/a.mcfunction",
    ] {
        let rejected =
            Function::parse(&PackPath::new(path).unwrap(), b"say hi\n", &cx).expect_err(path);
        let [finding] = rejected.diagnostics.iter().collect::<Vec<_>>()[..] else {
            panic!("{path}: one finding");
        };
        assert_eq!(finding.severity, Severity::Error, "{path}");
        assert_eq!(
            finding.code,
            Code::from(&core_codes::IGNORED_FILE),
            "{path}"
        );
        assert_eq!(rejected.origin.path.as_str(), path);
    }
}

#[test]
fn a_function_is_placed_at_functions_name_mcfunction() {
    let function = Function::new(FunctionPath::parse("a/b"));
    assert_eq!(function.default_path().as_str(), "functions/a/b.mcfunction");
    // The path names the function again.
    let again = parse_with("a/b", b"", &LoadContext::detached());
    assert_eq!(again.name, function.name);
    // A name no pack path can hold gets a placeholder.
    let malformed = Function::new(FunctionPath::parse("a:b"));
    assert_eq!(malformed.default_path().as_str(), "functions/_.mcfunction");
    assert_eq!(
        TickList::default().default_path().as_str(),
        "functions/tick.json"
    );
}

#[test]
fn a_name_ending_in_a_slash_equals_the_bare_name_deviation() {
    // `functions/a/.mcfunction` is vanilla's function `a/`, distinct from `a`. A function path
    // compares by its segments, so the two are one key here.
    let function = parse("a/", b"say hi\n");
    assert_eq!(function.name.as_str(), "a/");
    assert_eq!(function.name, FunctionPath::parse("a"));
}

/// The tick list `text` of a pack at the engine version.
fn parse_tick(text: &str) -> TickList {
    let path = PackPath::new("functions/tick.json").unwrap();
    TickList::parse(&path, text.as_bytes(), &LoadContext::detached())
        .unwrap_or_else(|rejected| panic!("{rejected}"))
        .into_value()
}

#[test]
fn a_vanilla_tick_list_equals_the_hand_built_one() {
    // `functions/tick.json` of the GDK test asset pack `ArchivedPackLoadingUnbakedAssets/BP`.
    let parsed = parse_tick("{ \"values\": [\"scriptevent/tick\"] }\n");
    let built = TickList {
        values: vec![TickFunction::new(FunctionPath::parse("scriptevent/tick"))],
        ..TickList::default()
    };
    assert_eq!(parsed, built);
    assert_eq!(parsed.values[0].command_version, 46);
    assert_eq!(
        parsed.to_json().unwrap(),
        "{\n  \"values\": [\n    \"scriptevent/tick\"\n  ]\n}\n"
    );
}

#[test]
fn a_tick_list_round_trips_with_its_other_members() {
    // Vanilla reads `values` only; the other members are kept, a member named `extra` too.
    let text = r#"{
        // comments are read
        "note": { "by": "someone" },
        "values": ["a", "b/c", "a"],
        "extra": [1, 2]
    }"#;
    let list = parse_tick(text);
    let names: Vec<&str> = list.values.iter().map(|e| e.function.as_str()).collect();
    assert_eq!(names, ["a", "b/c", "a"]);
    let kept: Vec<&str> = list.extra.iter().map(|(key, _)| key).collect();
    assert_eq!(kept, ["note", "extra"]);
    assert!(list.extra.get("extra").is_some_and(Value::is_array));
    // Unknown members are written after `values`.
    let written = list.to_json().unwrap();
    let expected = "{\n  \"values\": [\n    \"a\",\n    \"b/c\",\n    \"a\"\n  ],\n  \"note\": {\n    \"by\": \"someone\"\n  },\n  \"extra\": [\n    1,\n    2\n  ]\n}\n";
    assert_eq!(written, expected);
    let again = parse_tick(&written);
    assert_eq!(again, list);
    assert_eq!(again.to_json().unwrap(), written);
}

#[test]
fn an_empty_tick_list_keeps_its_empty_array() {
    let list = parse_tick("{\"values\": []}");
    assert_eq!(list, TickList::default());
    assert_eq!(list.to_json().unwrap(), "{\n  \"values\": []\n}\n");
}

#[test]
fn a_tick_entry_that_is_no_string_is_reported_and_not_kept() {
    let path = PackPath::new("functions/tick.json").unwrap();
    let document = TickList::parse(
        &path,
        br#"{"values": ["a", 5, null, {"b": 1}]}"#,
        &LoadContext::detached(),
    )
    .unwrap();
    assert_eq!(document.values.len(), 1);
    let paths: Vec<String> = document
        .diagnostics()
        .iter()
        .map(|finding| {
            assert_eq!(finding.severity, Severity::Warning);
            assert!(
                finding
                    .code
                    .is(&bedrock_addon::families::function::codes::TICK_NOT_FOUND)
            );
            finding.location.json_path.to_string()
        })
        .collect();
    assert_eq!(paths, ["values/1", "values/2", "values/3"]);
    assert_eq!(
        document.to_json().unwrap(),
        "{\n  \"values\": [\n    \"a\"\n  ]\n}\n"
    );
}

#[test]
fn a_single_tick_list_does_not_resolve_its_names() {
    // Whether `nowhere` is a function is a question for a stack.
    let path = PackPath::new("functions/tick.json").unwrap();
    let document = TickList::parse(
        &path,
        br#"{"values": ["nowhere"]}"#,
        &LoadContext::detached(),
    )
    .unwrap();
    assert!(document.diagnostics().is_empty());
    assert_eq!(document.values.len(), 1);
}

#[test]
fn a_rejected_tick_list_is_an_error_for_the_single_file() {
    let path = PackPath::new("functions/tick.json").unwrap();
    for (text, code) in [
        ("{", "MalformedInput"),
        ("", "MalformedInput"),
        ("{}", "MissingRequiredField"),
        ("null", "MissingRequiredField"),
        ("{\"values\": {}}", "InvalidType"),
        ("{\"values\": 1}", "InvalidType"),
    ] {
        let rejected =
            TickList::parse(&path, text.as_bytes(), &LoadContext::detached()).expect_err(text);
        let [finding] = rejected.diagnostics.iter().collect::<Vec<_>>()[..] else {
            panic!("{text}: one finding");
        };
        assert_eq!(finding.code.name(), code, "{text}");
        assert_eq!(finding.severity, Severity::Error, "{text}");
    }
}

#[test]
fn values_is_written_once_whatever_the_other_members_hold() {
    let mut list = TickList {
        values: vec![TickFunction::new(FunctionPath::parse("a"))],
        ..TickList::default()
    };
    list.extra
        .insert("values".to_owned(), Value::from("shadow"));
    list.extra.insert("note".to_owned(), Value::from("kept"));
    assert_eq!(
        list.to_json().unwrap(),
        "{\n  \"values\": [\n    \"a\"\n  ],\n  \"note\": \"kept\"\n}\n"
    );
}

#[test]
fn a_first_line_that_starts_with_a_byte_order_mark_is_not_written() {
    // Read back, the mark would be taken for the file's BOM and the line would lose it.
    let mut function = Function::new(FunctionPath::parse("x"));
    function.commands.push(FunctionLine::new("\u{feff}say a"));
    function.commands.push(FunctionLine::new("say b"));
    let mut out = b"kept".to_vec();
    let error = function
        .write_json(&mut out, WriteStyle::default())
        .expect_err("a leading mark");
    let EncodeError::NotRepresentable { at, what } = error else {
        panic!("{error:?}");
    };
    assert_eq!(what, Unrepresentable::Unsupported);
    let mut expected = JsonPath::new();
    expected.push("commands");
    expected.push_index(0);
    assert_eq!(at, expected);
    assert_eq!(out, b"kept");
    // On a later line the mark is part of the line and reads back as written.
    function.commands.swap(0, 1);
    let text = function.to_json().unwrap();
    assert_eq!(text, "say b\n\u{feff}say a\n");
    let again = parse_with("x", text.as_bytes(), &{
        let mut cx = LoadContext::detached();
        cx.min_engine_version = None;
        cx
    });
    assert_eq!(raws(&again), ["say b", "\u{feff}say a"]);
}
