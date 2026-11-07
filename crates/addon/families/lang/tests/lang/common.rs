//! Fixture loading for the text tests: the files under `tests/fixtures/text/text/`, a reader
//! for the small TOML subset the `*.expected.toml` and `format/*.toml` files use, and the
//! `.lang` harness that compares a file's table and findings with its expectation.

#![allow(dead_code, reason = "each test module uses part of the helpers")]

use std::collections::BTreeMap;
use std::path::PathBuf;

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, Severity};
use bedrock_addon_family_lang::{KeyFilter, LangFile, Language, Translations};

/// The fixture directory of the family.
pub fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../addon/tests/fixtures/text/text")
}

/// The bytes of fixture `name`.
pub fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(fixtures().join(name)).unwrap_or_else(|e| panic!("{name}: {e}"))
}

/// The text of fixture `name`.
pub fn fixture_text(name: &str) -> String {
    String::from_utf8(fixture(name)).unwrap_or_else(|e| panic!("{name}: {e}"))
}

/// The language the `.lang` fixtures are read as.
pub fn en_us() -> Language {
    Language::new("en_US")
}

// ---- the TOML subset -----------------------------------------------------------------------

/// A value of the subset: a basic string, a boolean, or a one-line array of strings.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Str(String),
    Bool(bool),
    List(Vec<String>),
}

/// The `key = value` pairs of one table, in file order.
#[derive(Clone, Debug, Default)]
pub struct Table {
    pub pairs: Vec<(String, Value)>,
}

impl Table {
    /// The string under `key`.
    pub fn str(&self, key: &str) -> Option<&str> {
        self.pairs.iter().find_map(|(k, v)| match v {
            Value::Str(text) if k == key => Some(text.as_str()),
            _ => None,
        })
    }

    /// The boolean under `key`.
    pub fn bool(&self, key: &str) -> Option<bool> {
        self.pairs.iter().find_map(|(k, v)| match v {
            Value::Bool(flag) if k == key => Some(*flag),
            _ => None,
        })
    }

    /// The array under `key`; empty when absent.
    pub fn list(&self, key: &str) -> Vec<String> {
        self.pairs
            .iter()
            .find_map(|(k, v)| match v {
                Value::List(items) if k == key => Some(items.clone()),
                _ => None,
            })
            .unwrap_or_default()
    }

    /// Every string pair, as a map.
    pub fn strings(&self) -> BTreeMap<String, String> {
        self.pairs
            .iter()
            .filter_map(|(k, v)| match v {
                Value::Str(text) => Some((k.clone(), text.clone())),
                _ => None,
            })
            .collect()
    }
}

/// A document of the subset: the root pairs, the `[name]` tables and the `[[name]]` tables,
/// each in file order. A `[name.sub]` header after a `[[name]]` belongs to that element and is
/// stored under `name.sub` right after it.
#[derive(Clone, Debug, Default)]
pub struct Toml {
    pub root: Table,
    pub tables: Vec<(String, Table)>,
    pub arrays: Vec<(String, Table)>,
}

impl Toml {
    /// The `[name]` table; empty when absent.
    pub fn table(&self, name: &str) -> Table {
        self.tables
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, table)| table.clone())
            .unwrap_or_default()
    }

    /// The `[[name]]` tables, in order.
    pub fn array(&self, name: &str) -> Vec<&Table> {
        self.arrays
            .iter()
            .filter(|(n, _)| n == name)
            .map(|(_, table)| table)
            .collect()
    }
}

/// Reads the subset: `#` comments, `[table]`, `[[array]]`, bare or quoted keys, basic strings
/// with `\n \t \r \\ \" \uXXXX`, `true` / `false`, and one-line arrays of strings.
pub fn toml(text: &str) -> Toml {
    enum At {
        Root,
        Table(usize),
        Array(usize),
    }
    let mut doc = Toml::default();
    let mut at = At::Root;
    for (number, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(name) = line.strip_prefix("[[").and_then(|l| l.strip_suffix("]]")) {
            doc.arrays.push((name.trim().to_owned(), Table::default()));
            at = At::Array(doc.arrays.len() - 1);
            continue;
        }
        if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            doc.tables.push((name.trim().to_owned(), Table::default()));
            at = At::Table(doc.tables.len() - 1);
            continue;
        }
        let mut chars = line.char_indices().peekable();
        let key = if line.starts_with('"') {
            chars.next();
            string(&mut chars, line, number)
        } else {
            let end = line
                .find(|c: char| c == '=' || c.is_whitespace())
                .unwrap_or_else(|| panic!("line {}: no `=`: {raw}", number + 1));
            for _ in 0..line[..end].chars().count() {
                chars.next();
            }
            line[..end].to_owned()
        };
        skip_blanks(&mut chars);
        assert_eq!(
            chars.next().map(|(_, c)| c),
            Some('='),
            "line {}: no `=`: {raw}",
            number + 1
        );
        skip_blanks(&mut chars);
        let value = match chars.next().map(|(_, c)| c) {
            Some('"') => Value::Str(string(&mut chars, line, number)),
            Some('[') => {
                let mut items = Vec::new();
                loop {
                    skip_blanks(&mut chars);
                    match chars.next().map(|(_, c)| c) {
                        Some('"') => items.push(string(&mut chars, line, number)),
                        Some(',') => {}
                        Some(']') => break,
                        other => panic!("line {}: {other:?} in an array: {raw}", number + 1),
                    }
                }
                Value::List(items)
            }
            Some('t') => Value::Bool(true),
            Some('f') => Value::Bool(false),
            other => panic!("line {}: value starts with {other:?}: {raw}", number + 1),
        };
        let table = match at {
            At::Root => &mut doc.root,
            At::Table(index) => &mut doc.tables[index].1,
            At::Array(index) => &mut doc.arrays[index].1,
        };
        table.pairs.push((key, value));
    }
    doc
}

fn skip_blanks(chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>) {
    while chars.peek().is_some_and(|(_, c)| c.is_whitespace()) {
        chars.next();
    }
}

/// A basic string, after its opening quote.
fn string(
    chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
    line: &str,
    number: usize,
) -> String {
    let mut out = String::new();
    loop {
        match chars.next().map(|(_, c)| c) {
            Some('"') => return out,
            Some('\\') => match chars.next().map(|(_, c)| c) {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('\\') => out.push('\\'),
                Some('"') => out.push('"'),
                Some('u') => {
                    let hex: String = (0..4)
                        .filter_map(|_| chars.next().map(|(_, c)| c))
                        .collect();
                    let code = u32::from_str_radix(&hex, 16)
                        .unwrap_or_else(|e| panic!("line {}: \\u{hex}: {e}", number + 1));
                    out.push(char::from_u32(code).unwrap());
                }
                other => panic!("line {}: escape {other:?}: {line}", number + 1),
            },
            Some(c) => out.push(c),
            None => panic!("line {}: unterminated string: {line}", number + 1),
        }
    }
}

// ---- the `.lang` harness -------------------------------------------------------------------

/// A `.lang` fixture, read: the file, its findings, and the table its expectation's filter
/// selects.
pub struct Read {
    pub file: LangFile,
    pub diagnostics: Diagnostics,
    pub table: Translations,
    pub expected: Toml,
}

/// One finding as an expectation file states it: the code's name, the severity word and the
/// vanilla text (empty when the finding is ours and carries none).
pub type Finding = (String, String, String);

/// The vanilla text a finding carries, rendered; empty when it carries none.
pub fn vanilla_text(diagnostic: &Diagnostic) -> String {
    diagnostic.vanilla().unwrap_or_default()
}

/// The findings as `(code, severity, vanilla text)`, in order.
pub fn findings(diagnostics: &Diagnostics) -> Vec<Finding> {
    diagnostics
        .iter()
        .map(|d| {
            (
                d.code.id().to_owned(),
                d.severity.to_string(),
                vanilla_text(d),
            )
        })
        .collect()
}

/// The `[[diagnostics]]` of an expectation file.
pub fn expected_findings(expected: &Toml) -> Vec<Finding> {
    expected
        .array("diagnostics")
        .into_iter()
        .map(|table| {
            (
                table.str("code").expect("a finding has a code").to_owned(),
                table
                    .str("severity")
                    .expect("a finding has a severity")
                    .to_owned(),
                table.str("text").unwrap_or_default().to_owned(),
            )
        })
        .collect()
}

/// Reads `lang/<name>.lang` as vanilla does and checks it against `lang/<name>.expected.toml`:
/// the `[entries]` (stored key → value) under the file's `prefix` / `allowed` / `blocked`, the
/// `[[diagnostics]]` in order, and `may_contain_identifier` when the file states it.
pub fn lang(name: &str) -> Read {
    let bytes = fixture(&format!("lang/{name}.lang"));
    let expected = toml(&fixture_text(&format!("lang/{name}.expected.toml")));
    let (file, diagnostics) = LangFile::from_bytes(en_us(), &bytes);
    let allowed = expected.root.list("allowed");
    let blocked = expected.root.list("blocked");
    let allowed: Vec<&str> = allowed.iter().map(String::as_str).collect();
    let blocked: Vec<&str> = blocked.iter().map(String::as_str).collect();
    let filter = KeyFilter {
        exclude: &blocked,
        include: &allowed,
        prefix: expected.root.str("prefix").unwrap_or_default(),
    };
    let table = file.select(&filter);
    assert_eq!(
        table.strings,
        expected.table("entries").strings(),
        "{name}: the entries differ from the expectation"
    );
    assert_eq!(
        findings(&diagnostics),
        expected_findings(&expected),
        "{name}: the findings differ from the expectation"
    );
    if let Some(flag) = expected.root.bool("may_contain_identifier") {
        assert_eq!(table.may_contain_identifier, flag, "{name}: the `~` flag");
    }
    Read {
        file,
        diagnostics,
        table,
        expected,
    }
}

/// The findings with `code`'s name at `severity`.
pub fn with_code<'d>(
    diagnostics: &'d Diagnostics,
    code: &str,
    severity: Severity,
) -> Vec<&'d Diagnostic> {
    diagnostics
        .iter()
        .filter(|d| d.code.id() == code && d.severity == severity)
        .collect()
}
