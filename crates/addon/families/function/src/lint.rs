//! The crate's own lint of a command line's arity against the core `vanilla` command table:
//! a line with a number of arguments no overload of its command takes cannot parse. Vanilla
//! has no such check apart from parsing the line, which is the command system's; this lint
//! only raises an information, and only where it is sure.

use super::grammar::grammar_ranges;
use bedrock_addon_core::vanilla::{Command, Overload};
use bedrock_addon_core::version::CURRENT_COMMAND_VERSION;

/// Whether the arguments `arguments` of a line of `command` (the trimmed line after its first
/// word) are a number of arguments no overload of the command takes under `command_version`.
/// `false` whenever that cannot be said for certain: the arguments cannot be counted, or the
/// table's overloads are not vanilla's under that command version.
pub(super) fn has_wrong_arity(command: &Command, arguments: &str, command_version: u8) -> bool {
    // The table holds the newest grammar: under an older command version the command had
    // other overloads, which the table does not list.
    if command_version < table_grammar_since(command.name) {
        return false;
    }
    let Some(count) = argument_count(arguments) else {
        return false;
    };
    !command.overloads.iter().any(|overload| {
        let (least, most) = bounds(overload);
        least <= count && most.is_none_or(|most| count <= most)
    })
}

/// The fewest and the most arguments `overload` takes; no most when a parameter takes the rest
/// of the line.
fn bounds(overload: &Overload) -> (usize, Option<usize>) {
    let mut least = 0;
    let mut most = Some(0);
    for param in overload.params {
        let width = width(param.type_name);
        if !param.optional {
            least += width;
        }
        most = if is_rest_of_line(param.type_name) {
            None
        } else {
            most.map(|most| most + width)
        };
    }
    (least, most)
}

/// How many arguments a parameter of this type is: three coordinates for a position, one for
/// everything else.
fn width(type_name: &str) -> usize {
    match type_name {
        "POSITION" | "POSITION_FLOAT" => 3,
        _ => 1,
    }
}

/// The command version from which the table's overloads of `command` are the ones vanilla
/// registers: the newest bound of the command's grammar ranges, 1 for a command without any.
fn table_grammar_since(command: &str) -> u8 {
    grammar_ranges()
        .iter()
        .filter(|range| range.command == command)
        .flat_map(|range| {
            // A range that starts late changes the grammar where it starts; one that ends
            // before the current version changes it after its end.
            let starts = (range.from > 1).then_some(range.from);
            let ends = range
                .to
                .filter(|to| *to < CURRENT_COMMAND_VERSION)
                .map(|to| to + 1);
            [starts, ends]
        })
        .flatten()
        .max()
        .unwrap_or(1)
}

/// Whether a parameter of this type takes the rest of the line: a message, raw text, or the
/// command or sub-command chain after `execute`.
fn is_rest_of_line(type_name: &str) -> bool {
    matches!(
        type_name,
        "MESSAGE_ROOT" | "RAWTEXT" | "CODEBUILDERARGS" | "EXECUTECHAINEDOPTION_0"
    )
}

/// How many arguments `text` holds, or `None` when it cannot be split for certain.
///
/// Arguments are separated by spaces and tabs. A quoted string, a JSON object and a bracketed
/// list are one argument each, whatever they hold; the brackets after a selector (`@e[…]`,
/// also `@e […]`) belong to the selector, and those glued to another word (`stone[…]`) are an
/// argument of their own. Coordinates written without spaces (`~~~`, `~1^-2`) are one argument
/// each.
fn argument_count(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut count = 0;
    let mut at = 0;
    // The last argument was a selector without brackets: brackets that follow are its own.
    let mut bare_selector = false;
    loop {
        while bytes.get(at).is_some_and(|byte| is_space(*byte)) {
            at += 1;
        }
        let Some(first) = bytes.get(at) else {
            return Some(count);
        };
        match first {
            b'"' => {
                at = ends_argument(bytes, quoted(bytes, at)?)?;
                count += 1;
                bare_selector = false;
            }
            b'{' => {
                at = ends_argument(bytes, bracketed(bytes, at)?)?;
                count += 1;
                bare_selector = false;
            }
            b'[' => {
                at = ends_argument(bytes, bracketed(bytes, at)?)?;
                if !bare_selector {
                    count += 1;
                }
                bare_selector = false;
            }
            _ => {
                let start = at;
                while bytes
                    .get(at)
                    .is_some_and(|byte| !is_space(*byte) && !matches!(byte, b'[' | b'{' | b'"'))
                {
                    at += 1;
                }
                let word = &text[start..at];
                let selector = word.starts_with('@');
                match bytes.get(at) {
                    Some(b'[') => {
                        at = ends_argument(bytes, bracketed(bytes, at)?)?;
                        count += if selector { 1 } else { 2 };
                        bare_selector = false;
                    }
                    // A quote or a brace inside a word.
                    Some(b'{' | b'"') => return None,
                    _ => {
                        count += if word.contains(['~', '^']) {
                            coordinates(word)?
                        } else {
                            1
                        };
                        bare_selector = selector;
                    }
                }
            }
        }
    }
}

/// What separates arguments.
const fn is_space(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t')
}

/// `end` when an argument may end there: at the end of the text or before a space.
fn ends_argument(bytes: &[u8], end: usize) -> Option<usize> {
    bytes
        .get(end)
        .is_none_or(|byte| is_space(*byte))
        .then_some(end)
}

/// The index after the quoted string that opens at `open`; a backslash escapes the next byte.
fn quoted(bytes: &[u8], open: usize) -> Option<usize> {
    let mut at = open + 1;
    loop {
        match bytes.get(at)? {
            b'"' => return Some(at + 1),
            b'\\' => at += 2,
            _ => at += 1,
        }
    }
}

/// The index after the bracket or brace that opens at `open`, its nested brackets, braces and
/// strings skipped; `None` when they do not balance.
fn bracketed(bytes: &[u8], open: usize) -> Option<usize> {
    let mut closers = Vec::new();
    let mut at = open;
    loop {
        match bytes.get(at)? {
            b'[' => closers.push(b']'),
            b'{' => closers.push(b'}'),
            b'"' => {
                at = quoted(bytes, at)?;
                continue;
            }
            close @ (b']' | b'}') => {
                if closers.pop() != Some(*close) {
                    return None;
                }
                if closers.is_empty() {
                    return Some(at + 1);
                }
            }
            _ => {}
        }
        at += 1;
    }
}

/// How many coordinates `word` holds: each a number, or `~` or `^` with an optional number
/// (`~1~-2~` is three). `None` when the word is anything else.
fn coordinates(word: &str) -> Option<usize> {
    let bytes = word.as_bytes();
    let mut count = 0;
    let mut at = 0;
    while at < bytes.len() {
        at = if matches!(bytes[at], b'~' | b'^') {
            number(bytes, at + 1).unwrap_or(at + 1)
        } else {
            number(bytes, at)?
        };
        count += 1;
    }
    Some(count)
}

/// The index after the number that starts at `start`: an optional sign, digits and an optional
/// fraction, with a digit somewhere.
fn number(bytes: &[u8], start: usize) -> Option<usize> {
    let mut at = start;
    if matches!(bytes.get(at), Some(b'+' | b'-')) {
        at += 1;
    }
    let digits = |mut at: usize| {
        while bytes.get(at).is_some_and(u8::is_ascii_digit) {
            at += 1;
        }
        at
    };
    let whole = digits(at);
    let mut end = whole;
    if bytes.get(end) == Some(&b'.') {
        end = digits(end + 1);
    }
    // `1`, `1.`, `1.5` and `.5`, never a lone sign or point.
    let has_digit = whole > at || end > whole + 1;
    has_digit.then_some(end)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_addon_core::vanilla;

    #[test]
    fn words_are_split_at_spaces_and_tabs() {
        assert_eq!(argument_count(""), Some(0));
        assert_eq!(argument_count("  \t "), Some(0));
        assert_eq!(argument_count("hi"), Some(1));
        assert_eq!(argument_count("a b  c\td"), Some(4));
        assert_eq!(argument_count("  a b "), Some(2));
    }

    #[test]
    fn a_quoted_string_is_one_argument() {
        assert_eq!(argument_count(r#""a b" c"#), Some(2));
        assert_eq!(argument_count(r#""a \" b""#), Some(1));
        assert_eq!(argument_count(r#""a \\" b"#), Some(2));
        assert_eq!(argument_count(r#""""#), Some(1));
        // Not closed, or glued to a word: not counted.
        assert_eq!(argument_count(r#""open"#), None);
        assert_eq!(argument_count(r#"he said "hi"#), None);
        assert_eq!(argument_count(r#""a"b"#), None);
        assert_eq!(argument_count(r#"x"y""#), None);
    }

    #[test]
    fn a_json_object_is_one_argument() {
        assert_eq!(
            argument_count(r#"@a {"rawtext":[{"text":"a b"}]}"#),
            Some(2)
        );
        assert_eq!(argument_count(r#"{"a":"}"} x"#), Some(2));
        assert_eq!(argument_count(r#"{"a":{"b":[1, 2]}}"#), Some(1));
        assert_eq!(argument_count("{"), None);
        assert_eq!(argument_count(r#"{"a":1]"#), None);
        assert_eq!(argument_count(r#"{"a":1}x"#), None);
        assert_eq!(argument_count(r#"x{"a":1}"#), None);
    }

    #[test]
    fn a_selector_and_its_arguments_are_one_argument() {
        assert_eq!(argument_count("@e[type=cow, r=5] x"), Some(2));
        assert_eq!(argument_count("@e [type=cow, r=5] x"), Some(2));
        assert_eq!(argument_count(r#"@e[name="a ] b"]"#), Some(1));
        assert_eq!(
            argument_count("@e[scores={a=1..}, hasitem=[{item=apple}]] x"),
            Some(2)
        );
        assert_eq!(argument_count("@e[type=cow"), None);
        assert_eq!(argument_count("@e[type=cow]x"), None);
        // Only the first bracket after a selector is its arguments.
        assert_eq!(argument_count("@a [x] [y]"), Some(2));
        assert_eq!(argument_count("@a x [y]"), Some(3));
    }

    #[test]
    fn a_block_state_array_is_its_own_argument() {
        assert_eq!(
            argument_count(r#"stone ["a"=1, "b"=true] replace"#),
            Some(3)
        );
        assert_eq!(argument_count(r#"stone["a"=1] replace"#), Some(3));
        assert_eq!(argument_count("stone []"), Some(2));
        assert_eq!(argument_count("[1, 2]"), Some(1));
        assert_eq!(argument_count("a [x] [y]"), Some(3));
    }

    #[test]
    fn glued_coordinates_are_one_argument_each() {
        assert_eq!(argument_count("~ ~ ~"), Some(3));
        assert_eq!(argument_count("~~~"), Some(3));
        assert_eq!(argument_count("^^^1"), Some(3));
        assert_eq!(argument_count("~1~-2~.5"), Some(3));
        assert_eq!(argument_count("~1.5 2 ~"), Some(3));
        assert_eq!(argument_count("1~2~3"), Some(3));
        assert_eq!(argument_count("~+1^-2"), Some(2));
        assert_eq!(argument_count("~5"), Some(1));
        assert_eq!(argument_count("-5 1.5 .5"), Some(3));
        // A `~` or `^` in anything but coordinates: not counted.
        assert_eq!(argument_count("a~b"), None);
        assert_eq!(argument_count("~x"), None);
        assert_eq!(argument_count("~1."), Some(1));
        assert_eq!(argument_count("~-"), None);
        assert_eq!(argument_count("~."), None);
    }

    /// Whether `line` (a command and its arguments) has a wrong arity under `version`.
    fn wrong(line: &str, version: u8) -> bool {
        let (word, arguments) = line.split_once(' ').unwrap_or((line, ""));
        let command = super::super::parse::find_command(word).expect(word);
        has_wrong_arity(command, arguments, version)
    }

    #[test]
    fn a_required_argument_must_be_there() {
        assert!(wrong("say", 46));
        assert!(!wrong("say hi", 46));
        assert!(wrong("testfor", 46));
        assert!(!wrong("testfor @s", 46));
        // A position is three arguments.
        assert!(wrong("setblock ~ ~ ~", 46));
        assert!(!wrong("setblock ~ ~ ~ stone", 46));
        assert!(wrong("setblock ~~~", 46));
        assert!(!wrong("setblock ~~~ stone", 46));
    }

    #[test]
    fn an_optional_argument_may_be_left_out() {
        assert!(!wrong("kill", 46));
        assert!(!wrong("kill @e", 46));
        assert!(!wrong("list", 46));
        assert!(!wrong("setblock ~ ~ ~ stone [] replace", 46));
    }

    #[test]
    fn one_argument_more_than_the_longest_overload_is_wrong() {
        assert!(wrong("kill @e x", 46));
        assert!(wrong("list now", 46));
        assert!(wrong("setblock ~ ~ ~ stone [] replace more", 46));
        assert!(wrong("testfor @s @s", 46));
    }

    #[test]
    fn a_message_or_a_chained_command_takes_every_further_argument() {
        assert!(!wrong("say a b c d e f g h", 46));
        assert!(!wrong("tell @a a b c d e f g h", 46));
        assert!(wrong("tell @a", 46));
        assert!(!wrong(
            "execute as @a at @s if block ~ ~ ~ stone run say a b c",
            46
        ));
        assert!(!wrong("execute run say hi", 46));
        assert!(wrong("execute run", 46));
        assert!(wrong("execute", 46));
        // Raw text is the rest of the line too.
        assert!(!wrong("wsserver ws://host:1 more words", 46));
        assert!(wrong("wsserver", 46));
    }

    #[test]
    fn any_overload_that_fits_is_enough() {
        // `teleport <destination: target>` and `teleport <victim> <destination: x y z> …`.
        assert!(!wrong("tp @s", 46));
        assert!(!wrong("tp @s ~ ~ ~", 46));
        assert!(!wrong("tp @s ~ ~ ~ facing ~ ~ ~ true true", 46));
        assert!(wrong("tp @s ~ ~ ~ facing ~ ~ ~ true true more", 46));
        assert!(wrong("tp", 46));
    }

    #[test]
    fn arguments_that_cannot_be_counted_are_not_judged() {
        assert!(!wrong(r#"say he said "hi"#, 46));
        assert!(!wrong("kill @e[type=cow", 46));
        assert!(!wrong("list a~b", 46));
    }

    #[test]
    fn a_command_is_judged_only_where_the_tables_grammar_is_vanillas() {
        // The table holds the newest grammar. `setblock` took a data value up to 28.
        assert!(!wrong("setblock ~ ~ ~", 28));
        assert!(wrong("setblock ~ ~ ~", 29));
        // Each ranged command from its newest bound.
        for (line, since) in [
            ("execute", 29),
            ("fill", 29),
            ("clone", 29),
            ("testforblock", 29),
            ("locate", 44),
            ("structure", 22),
            ("summon", 31),
            ("damage", 38),
        ] {
            assert!(!wrong(line, since - 1), "{line} at {}", since - 1);
            assert!(wrong(line, since), "{line} at {since}");
            assert!(wrong(line, 46), "{line} at 46");
        }
        // A command without a range is judged under every command version.
        assert!(wrong("say", 8));
        assert!(wrong("say", 1));
    }

    #[test]
    fn the_newest_bound_of_each_ranged_command() {
        assert_eq!(table_grammar_since("execute"), 29);
        assert_eq!(table_grammar_since("setblock"), 29);
        assert_eq!(table_grammar_since("locate"), 44);
        assert_eq!(table_grammar_since("structure"), 22);
        assert_eq!(table_grammar_since("summon"), 31);
        assert_eq!(table_grammar_since("summon_noai"), 31);
        assert_eq!(table_grammar_since("damage"), 38);
        // Bounded by the current version only: every version.
        assert_eq!(table_grammar_since("agent"), 1);
        assert_eq!(table_grammar_since("say"), 1);
    }

    #[test]
    fn only_a_block_comes_before_a_block_state_array_in_the_table() {
        // The count takes a `[…]` after a selector for the selector's arguments; no overload
        // puts a block-state array there.
        for command in vanilla::commands() {
            for overload in command.overloads {
                for pair in overload.params.windows(2) {
                    if pair[1].type_name == "BLOCK_STATE_ARRAY" {
                        assert_eq!(pair[0].type_name, "BLOCK", "{}", command.name);
                    }
                }
            }
        }
    }

    #[test]
    fn four_parameter_types_take_the_rest_of_the_line() {
        // By the number of arguments alone the chain and the `run` overloads of `execute`
        // cover for each other, so the set is pinned here.
        for rest in [
            "MESSAGE_ROOT",
            "RAWTEXT",
            "CODEBUILDERARGS",
            "EXECUTECHAINEDOPTION_0",
        ] {
            assert!(is_rest_of_line(rest), "{rest}");
        }
        for one in [
            "INT",
            "ID",
            "SELECTION",
            "JSON_OBJECT",
            "POSITION",
            "OPTION_RUN",
        ] {
            assert!(!is_rest_of_line(one), "{one}");
        }
        // `execute as <origin> <chainedCommand>` and `execute run <command>`.
        let execute = vanilla::command("execute").unwrap();
        let of = |keyword: &str| {
            let overload = execute
                .overloads
                .iter()
                .find(|overload| overload.params[0].type_name == keyword)
                .unwrap();
            bounds(overload)
        };
        assert_eq!(of("OPTION_AS"), (3, None));
        assert_eq!(of("OPTION_RUN"), (2, None));
        // `setblock <position> <tileName> [oldBlockHandling]`: a position is three.
        let setblock = vanilla::command("setblock").unwrap();
        let shortest = setblock.overloads.iter().map(bounds).min().unwrap();
        assert_eq!(shortest, (4, Some(5)));
    }

    #[test]
    fn every_overload_of_the_table_has_at_most_one_rest_of_line_parameter_at_its_end() {
        for command in vanilla::commands() {
            for overload in command.overloads {
                let rest: Vec<usize> = overload
                    .params
                    .iter()
                    .enumerate()
                    .filter(|(_, param)| is_rest_of_line(param.type_name))
                    .map(|(index, _)| index)
                    .collect();
                assert!(rest.len() <= 1, "{}", command.name);
                if let Some(index) = rest.first() {
                    assert_eq!(*index, overload.params.len() - 1, "{}", command.name);
                }
            }
        }
    }
}
