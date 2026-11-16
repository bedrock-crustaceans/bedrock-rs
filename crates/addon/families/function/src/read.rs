//! Reading a function file the way vanilla's function manager does, line by line, and a tick
//! list the way it reads `tick.json`: a derived document read by the kit, its one member's
//! entries read by one function.

use std::borrow::Cow;

use super::lint::has_wrong_arity;
use super::{Function, FunctionLine, TickFunction, TickList, codes};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Code, CodeDef, Diagnostic, DiagnosticDetail, Diagnostics, JsonPath, Phase, Severity, Span,
    builders,
};
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::id::FunctionPath;
use bedrock_addon_core::json::kit::{Node, Read, Texts, WrongAt};
use bedrock_addon_core::json::{MAX_JSON_DEPTH, TooDeep, Value, check_depth, strip_bom};
use bedrock_addon_core::load::BuiltIns;
use bedrock_addon_core::schema::{self, Schema, read_document, read_each};
#[cfg(test)]
use bedrock_addon_core::vanilla::Command;
use bedrock_addon_core::versioning::{NullHandling, Reading};
use facet::Facet;

/// What vanilla removes from both ends of a line: ASCII whitespace.
fn is_trimmed(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\u{b}' | '\u{c}' | '\r')
}

/// `line` as vanilla looks at it: trimmed.
pub(super) fn trimmed(line: &str) -> &str {
    line.trim_matches(is_trimmed)
}

/// From this command version on (`min_engine_version` 1.16.0), a function with a failing line
/// is not created; below it the failing lines are dropped and the function is kept.
const DONT_REGISTER_BROKEN_FUNCTION_COMMANDS: u8 = 12;

/// The one-based number of the line at `index`.
fn line_number(index: usize) -> u32 {
    u32::try_from(index + 1).unwrap_or(u32::MAX)
}

/// Reads the lines of the function `name` under `command_version` (`None`: the pack has no
/// usable `min_engine_version`, and no line is checked), each first word looked up in
/// `built_ins` (the context's provider). An error-level finding means vanilla does not create
/// the function.
pub(super) fn parse_function(
    name: FunctionPath,
    bytes: &[u8],
    command_version: Option<u8>,
    built_ins: &dyn BuiltIns,
) -> (Function, Diagnostics) {
    // The byte order mark is stripped before the split.
    let content = strip_bom(bytes);
    let mut lossy = false;
    let mut function = Function {
        name,
        command_version,
        commands: Vec::new(),
        dropped: Vec::new(),
    };
    // With errors the function is created only below command version 12. Vanilla logs the
    // header and each error line as warnings either way; where the errors keep the function
    // from existing they are errors here, because an error-level finding rejects the document,
    // and a kept function's lines stay warnings.
    let strict =
        command_version.is_some_and(|version| version >= DONT_REGISTER_BROKEN_FUNCTION_COMMANDS);
    let severity = if strict {
        Severity::Error
    } else {
        Severity::Warning
    };
    let mut errors = Vec::new();
    // The crate's own findings, which never decide whether the function exists.
    let mut lints = Vec::new();
    // Vanilla splits the content on line feeds and keeps the empty lines: physical lines.
    let mut offset = bytes.len() - content.len();
    for (index, physical) in content.split(|byte| *byte == b'\n').enumerate() {
        let start = offset;
        offset += physical.len() + 1;
        // The line ending is not part of the line as kept: the CR of a CR LF ending goes with
        // the LF (vanilla trims it away before it looks at the line).
        let physical = physical.strip_suffix(b"\r").unwrap_or(physical);
        let span = Span {
            offset: u32::try_from(start).unwrap_or(u32::MAX),
            len: u32::try_from(physical.len()).unwrap_or(u32::MAX),
        };
        let raw = String::from_utf8_lossy(physical);
        lossy |= matches!(raw, Cow::Owned(_));
        let line = trimmed(&raw);
        // An empty line, or one whose first byte is `#`, is skipped.
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let number = line_number(index);
        let kept = FunctionLine {
            line: number,
            raw: raw.clone().into_owned(),
        };
        // Without a command version vanilla creates the function in its error state and never
        // reads the file: the lines are kept as written, none is checked.
        let Some(version) = command_version else {
            function.commands.push(kept);
            continue;
        };
        // The first word must be a registered command: the provider's (the game's table
        // without a host, compared without ASCII case as `find_command` does).
        let word = first_word(line);
        if let Some(command) = built_ins.command(word) {
            // A number of arguments no overload takes is the crate's own note. Whether the
            // line parses is the command system's to say, so this never drops a line. The
            // game's load checks the first word only, so the note is an information.
            if has_wrong_arity(command, &line[word.len()..], version) {
                let note = Diagnostic::of(&core_codes::INVALID_VALUE)
                    .severity(Severity::Info)
                    .phase(Phase::Validate);
                lints.push(note.span(span));
            }
            function.commands.push(kept);
        } else if is_namespaced(word) {
            lints.push(unchecked_command(word, "command").span(span));
            function.commands.push(kept);
        } else if is_feature_gated(word) {
            lints.push(unchecked_command(word, "feature-gated command").span(span));
            function.commands.push(kept);
        } else {
            let error =
                Diagnostic::of(unknown_command(word).with([number.to_string(), word.to_owned()]))
                    .phase(Phase::Validate)
                    .severity(severity);
            errors.push(error.span(span));
            function.dropped.push(kept);
        }
    }
    let mut diagnostics = Diagnostics::default();
    if lossy {
        // Vanilla keeps the bytes of a file that is not UTF-8; a `String` cannot. The game says
        // nothing, so the crate's own note is an information.
        diagnostics.push(
            Diagnostic::of(builders::syntax(
                "not UTF-8: each invalid sequence is read as U+FFFD",
            ))
            .severity(Severity::Info),
        );
    }
    if !errors.is_empty() {
        // The game logs the header, then one entry per error.
        let name = [function.name.as_str()];
        let header = if strict {
            codes::FAILED_HEADER.with(name)
        } else {
            codes::FAILED_HEADER_DROPPED.with(name)
        };
        diagnostics.push(
            Diagnostic::of(header)
                .phase(Phase::Validate)
                .severity(severity),
        );
        diagnostics.extend(errors);
    }
    diagnostics.extend(lints);
    (function, diagnostics)
}

/// The first word of a trimmed line: everything up to its first space.
pub(super) fn first_word(line: &str) -> &str {
    line.split(' ').next().unwrap_or(line)
}

/// The unknown-command text for the first word `word`: with the comment hint when the word
/// starts with `//`, with the slash hint when it otherwise starts with `/` and is at least two
/// bytes long, without a hint for anything else (a lone `/` included). Both hints follow the
/// unknown-command text only and look at the first word, not the line.
fn unknown_command(word: &str) -> &'static CodeDef<2> {
    if word.starts_with("//") {
        &codes::UNKNOWN_COMMAND_COMMENT
    } else if word.len() >= 2 && word.starts_with('/') {
        &codes::UNKNOWN_COMMAND_SLASH
    } else {
        &codes::UNKNOWN_COMMAND
    }
}

/// The command of the core `vanilla` command table that `word` names, by its name
/// or an alias, compared without ASCII case: what the provider's default answers
/// (`GeneratedBuiltIns::command`). The table stands for vanilla's check that a first word is a
/// valid command.
///
/// Whether vanilla folds case there is not known. Folding never rejects a function vanilla may
/// load; the command system, which parses the lines, decides for good.
#[cfg(test)]
pub(super) fn find_command(word: &str) -> Option<&'static Command> {
    bedrock_addon_core::vanilla::GeneratedBuiltIns.command(word)
}

/// Whether `word` has the `namespace:name` form of a command a script registers. No vanilla
/// command has a `:`.
fn is_namespaced(word: &str) -> bool {
    word.contains(':')
}

/// The commands vanilla registers only when the world has a feature the public command table
/// does not cover: those of Education Edition and of the developer features.
///
/// This list is the crate's own, not part of the public command table: the table holds the
/// commands Mojang publishes, and vanilla's check also accepts these when the feature is on. A
/// function that uses one may load in vanilla, so its line is never an unknown command here.
/// `agent` and `summon_noai` are the two the grammar ranges name.
const FEATURE_GATED_COMMANDS: [&str; 7] = [
    "ability",
    "agent",
    "immutableworld",
    "worldbuilder",
    "classroommode",
    "code",
    "summon_noai",
];

/// Whether `word` names a feature-gated command, compared as the table's names are.
fn is_feature_gated(word: &str) -> bool {
    FEATURE_GATED_COMMANDS
        .iter()
        .any(|command| command.eq_ignore_ascii_case(word))
}

/// The information for a line whose first word could not be checked against the command table
/// and is kept: a namespaced command (`kind` `command`: which commands scripts register is not
/// known when a pack loads) or a feature-gated one (`kind` `feature-gated command`: it exists
/// only in a world with education or developer features).
///
/// The command table holds vanilla's public commands only and the game has no line for such a
/// word, so the crate's own note is an information.
fn unchecked_command(word: &str, kind: &'static str) -> Diagnostic {
    Diagnostic::of(&core_codes::UNRESOLVED_REFERENCE)
        .phase(Phase::Validate)
        .detail(DiagnosticDetail::Reference {
            kind,
            id: word.to_owned(),
            hint: None,
        })
}

/// A tick list as read from one file: the list, and for each of its entries the index the
/// entry has in the file's `values` array.
pub(super) struct ParsedTick {
    /// The list; `None` when an error-level finding rejects the file.
    pub(super) list: Option<TickList>,
    /// The index in `values` of each kept entry.
    pub(super) indices: Vec<usize>,
    /// The findings.
    pub(super) diagnostics: Diagnostics,
}

/// Reads a `tick.json` of a pack with `command_version`.
pub(super) fn parse_tick(bytes: &[u8], command_version: u8) -> ParsedTick {
    let mut parsed = ParsedTick {
        list: None,
        indices: Vec::new(),
        diagnostics: Diagnostics::default(),
    };
    let root = match read_json(strip_bom(bytes)) {
        Ok(root) => root,
        Err(error) => {
            let message = error.to_string();
            // The game's text is `File %s failed to load with error(s) %s`, logged as an error:
            // the first `%s` is the literal `tick.json`, the second the reader's errors; the
            // pack's tick list is skipped.
            parsed.diagnostics.push(
                Diagnostic::of(codes::TICK_NOT_JSON.with([TICK_FILE, message.as_str()]))
                    .detail(DiagnosticDetail::Syntax(message)),
            );
            return parsed;
        }
    };
    // The `values` of a root that is no object is a null value, which fails the check as a
    // missing member does.
    if !root.is_object() {
        parsed.diagnostics.push(at_values(
            codes::TICK_NO_VALUES.with([VALUES]),
            Severity::Error,
            None,
        ));
        return parsed;
    }
    let file = {
        let mut read = Read::new(Reading::OWN, &mut parsed.diagnostics)
            .null(NullHandling::AbsentSilently)
            .texts(TICK_TEXTS)
            .phase(Phase::Validate);
        read_document::<TickFile>(&Node::new(&root, JsonPath::new()), &mut read)
    };
    let Some(file) = file else {
        return parsed;
    };
    let mut list = TickList {
        values: Vec::new(),
        extra: file.extra,
    };
    for (index, name) in file.values.into_iter().enumerate() {
        if let Some(name) = name {
            let function = FunctionPath::parse(name.as_str());
            list.values.push(TickFunction {
                function,
                command_version,
            });
            parsed.indices.push(index);
        }
    }
    parsed.list = Some(list);
    parsed
}

/// `tick.json` as written: `values`, the one member vanilla reads, and the members it does not
/// read, kept for write-back and told nothing.
#[derive(Facet, Schema)]
#[facet(schema::unknown = "silent")]
struct TickFile {
    /// JSON `values`: each entry's name, `None` for an entry that is no string. A missing
    /// member, `null` (read as absent) and a value that is no array fail the check with the
    /// same text (Error).
    #[facet(schema::read_with = entries)]
    values: Vec<Option<String>>,
    /// The other members.
    extra: UnknownMembers,
}

/// The entries of `values`. An entry that is no string reads as the empty name, which no
/// function has, so it is looked up and not found; it cannot be kept as a name.
fn entries(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<Option<String>>> {
    read_each(node, read, |entry, read| {
        if let Some(name) = entry.value().as_string() {
            return Some(Some(name.as_str().to_owned()));
        }
        read.push(not_found("", entry.path().index_after(VALUES)));
        Some(None)
    })
}

/// The game's words for `values` missing or no array: one text under two codes.
const TICK_TEXTS: Texts = Texts {
    missing_named: Some(&codes::TICK_NO_VALUES),
    wrong_at: Some(WrongAt(values_not_array)),
    ..Texts::CODES
};

/// `values` that is no array: the one member whose type the read checks.
fn values_not_array(_: &JsonPath) -> Code {
    codes::TICK_VALUES_NOT_ARRAY.with([VALUES])
}

/// The one member of a tick list.
pub(super) const VALUES: &str = "values";

/// A finding at `values`, or at its entry `index`; the loader adds the pack and the file.
fn at_values(code: Code, severity: Severity, index: Option<usize>) -> Diagnostic {
    let mut at = JsonPath::new().child(VALUES);
    if let Some(index) = index {
        at.push_index(index);
    }
    Diagnostic::error(code)
        .at(at)
        .phase(Phase::Validate)
        .severity(severity)
}

/// The warning for the tick entry at `index` of `values` whose name `name` no loaded function
/// has.
pub(super) fn not_found(name: &str, index: Option<usize>) -> Diagnostic {
    at_values(
        codes::TICK_NOT_FOUND.with([name]),
        // The game logs this as a warning, skips the entry and goes on with the next.
        Severity::Warning,
        index,
    )
}

/// The file name vanilla's JSON error names, whatever the pack.
const TICK_FILE: &str = "tick.json";

/// `bytes` as a JSON value, or the reader's message: nested deeper than the reader goes, not
/// UTF-8, or not JSON.
// Comments are accepted, as the game's JSON reader accepts them in every family.
fn read_json(bytes: &[u8]) -> Result<Value, ReadJsonError> {
    check_depth(bytes, MAX_JSON_DEPTH)?;
    let text = std::str::from_utf8(bytes)?;
    // Of a member written twice the last stands.
    bedrock_addon_core::json::from_str_jsonc(&bedrock_addon_core::json::last_wins(text))
        .map_err(|error| ReadJsonError::NotJson(error.kind.to_string()))
}

/// Why bytes are no JSON value; the display is the reader's message that vanilla's text names.
#[derive(Debug, thiserror::Error)]
enum ReadJsonError {
    #[error(transparent)]
    TooDeep(#[from] TooDeep),
    #[error(transparent)]
    NotUtf8(#[from] std::str::Utf8Error),
    #[error("{0}")]
    NotJson(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_json_names_why_the_bytes_are_no_value() {
        fn assert_error<E: std::error::Error>(_: &E) {}
        assert_error(&read_json(b"{").unwrap_err());
        assert!(read_json(b"{\"values\": []}").is_ok());
        assert_eq!(
            read_json(&[b'['; 4096]).unwrap_err().to_string(),
            check_depth(&[b'['; 4096], MAX_JSON_DEPTH)
                .unwrap_err()
                .to_string()
        );
        assert_eq!(
            read_json(b"\"\xff\"").unwrap_err().to_string(),
            std::str::from_utf8(core::hint::black_box(b"\"\xff\"" as &[u8]))
                .unwrap_err()
                .to_string()
        );
        assert!(!read_json(b"{").unwrap_err().to_string().is_empty());
    }

    /// The kept entries' names and command versions, their indices in `values`, and the other
    /// members' keys.
    type Kept = (Vec<String>, Vec<usize>, Vec<String>);

    /// The findings of `bytes` read as a tick list (code, severity, path, phase, span, detail),
    /// and what the list keeps.
    fn tick(bytes: &[u8]) -> (Vec<String>, Option<Kept>) {
        let parsed = parse_tick(bytes, 46);
        let found = parsed
            .diagnostics
            .iter()
            .map(|finding| {
                format!(
                    "{} {:?} {} {:?} {:?} {:?} {:?}",
                    finding.code.id(),
                    finding.severity,
                    finding.location.json_path,
                    finding.phase,
                    finding.location.span,
                    finding.code.args(),
                    finding.detail,
                )
            })
            .collect();
        let list = parsed.list.map(|list| {
            let names = list
                .values
                .iter()
                .map(|entry| format!("{} {}", entry.function.as_str(), entry.command_version))
                .collect();
            let extra = list.extra.iter().map(|(key, _)| key.to_owned()).collect();
            (names, parsed.indices, extra)
        });
        (found, list)
    }

    #[test]
    fn a_tick_list_without_a_values_array_is_one_error_at_values() {
        let no_values = |code: &str| {
            vec![format!(
                "{code} Error values Validate None [\"values\"] None"
            )]
        };
        for bytes in [&b"{}"[..], b"{\"values\": null}", b"[]", b"3", b"null"] {
            assert_eq!(
                tick(bytes),
                (no_values("function/tick_no_values"), None),
                "{}",
                String::from_utf8_lossy(bytes)
            );
        }
        for bytes in [
            &b"{\"values\": \"a\"}"[..],
            b"{\"values\": {}}",
            b"{\"values\": 1}",
        ] {
            assert_eq!(
                tick(bytes),
                (no_values("function/tick_values_not_array"), None),
                "{}",
                String::from_utf8_lossy(bytes)
            );
        }
    }

    #[test]
    fn a_tick_entry_that_is_no_string_is_told_at_its_index_and_left_out() {
        let read = tick(br#"{"m": [1], "values": ["a", 5, null, "b", {}], "n": null}"#);
        let not_found = |index: usize| {
            format!("function/tick_not_found Warning values/{index} Validate None [\"\"] None")
        };
        assert_eq!(
            read,
            (
                vec![not_found(1), not_found(2), not_found(4)],
                Some((
                    vec!["a 46".to_owned(), "b 46".to_owned()],
                    vec![0, 3],
                    vec!["m".to_owned(), "n".to_owned()],
                )),
            )
        );
    }

    #[test]
    fn the_last_of_a_member_written_twice_is_read() {
        let read = tick(br#"{"values": 3, "values": ["a"]}"#);
        assert_eq!(
            read,
            (vec![], Some((vec!["a 46".to_owned()], vec![0], vec![])))
        );
    }
}
