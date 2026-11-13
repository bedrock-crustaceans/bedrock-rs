//! Helpers of the material tests: files read on their own, in-line resource stacks (the lowest
//! pack first: `pack_0` is the lowest priority) and the shape a
//! test compares of a finding.

#![allow(dead_code, reason = "each test file uses part of the helpers")]

use std::fmt::Write as _;
use std::path::PathBuf;

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, Severity};
use bedrock_addon_core::document::{Content, Document, Rejected};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_family_material::{MaterialFile, MaterialGroup, MaterialList, Materials};

/// The `tests/fixtures` folder of `bedrock_addon`: this file is compiled by the family crate and
/// included by `bedrock_addon`'s own test root, which owns the folder.
fn fixtures_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let own = manifest.join("tests/fixtures");
    if own.is_dir() {
        own
    } else {
        manifest.join("../../addon/tests/fixtures")
    }
}

/// The bytes of the fixture file `name` (`tests/fixtures/render/materials/<name>`).
pub fn fixture(name: &str) -> Vec<u8> {
    let path = fixtures_root().join("render/materials").join(name);
    std::fs::read(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

/// The fixture file `name` as text.
pub fn fixture_text(name: &str) -> String {
    String::from_utf8(fixture(name)).expect("a UTF-8 fixture")
}

/// `text` read on its own as the material file at `path`.
pub fn parse_at(path: &str, text: &str) -> Result<Document<MaterialFile>, Rejected> {
    let path = PackPath::new(path).expect("a pack path");
    MaterialFile::parse(&path, text.as_bytes(), &LoadContext::detached())
}

/// `text` read on its own as `materials/test.material`.
pub fn parse(text: &str) -> Result<Document<MaterialFile>, Rejected> {
    parse_at("materials/test.material", text)
}

/// `text` read on its own, which must load.
#[track_caller]
pub fn file(text: &str) -> Document<MaterialFile> {
    match parse(text) {
        Ok(document) => document,
        Err(rejected) => panic!("rejected: {:#?}", rejected.diagnostics),
    }
}

/// A wrapped `1.0.0` file whose materials are `members` (the text between the braces).
pub fn wrapped(members: &str) -> String {
    format!(r#"{{"materials": {{"version": "1.0.0", {members}}}}}"#)
}

/// A flat file whose materials are `members`.
pub fn flat(members: &str) -> String {
    format!("{{{members}}}")
}

/// The findings of reading `text` on its own, loaded or rejected.
pub fn found(text: &str) -> Vec<Rendered> {
    match parse(text) {
        Ok(document) => rendered(document.diagnostics().iter()),
        Err(rejected) => rendered(rejected.diagnostics.iter()),
    }
}

/// `text` read on its own as the list at `path`.
pub fn parse_list(path: &str, text: &str) -> Result<Document<MaterialList>, Rejected> {
    let path = PackPath::new(path).expect("a pack path");
    MaterialList::parse(&path, text.as_bytes(), &LoadContext::detached())
}

/// What a test compares of a finding: the code's name, the severity, the place in the document
/// and the rendered text.
pub type Rendered = (&'static str, Severity, String, String);

/// Every finding of `diagnostics`, in order.
pub fn rendered<'a>(diagnostics: impl IntoIterator<Item = &'a Diagnostic>) -> Vec<Rendered> {
    diagnostics
        .into_iter()
        .map(|finding| {
            (
                finding.code.id(),
                finding.severity,
                finding.location.json_path.to_string(),
                finding.code.to_string(),
            )
        })
        .collect()
}

/// One expected finding.
pub fn finding(code: &'static str, severity: Severity, at: &str, text: &str) -> Rendered {
    (code, severity, at.to_owned(), text.to_owned())
}

/// The findings of `diagnostics` with the code `code`.
pub fn of_code(diagnostics: &Diagnostics, code: &str) -> Vec<Rendered> {
    rendered(diagnostics.iter().filter(|f| f.code.id() == code))
}

/// The findings of the whole group.
pub fn all(materials: &Materials) -> Vec<Rendered> {
    rendered(materials.diagnostics())
}

/// The names of `group`, sorted.
pub fn names(group: &MaterialGroup) -> Vec<String> {
    let mut names: Vec<String> = group.names().collect();
    names.sort();
    names
}

/// The name of the pack a finding names (`pack 1` is the lowest pack of [`stack`]).
pub fn pack_name(finding: &Diagnostic) -> Option<String> {
    finding
        .location
        .pack
        .as_ref()
        .map(|pack| pack.name().to_owned())
}

/// The shortest of three runs of `run`, which returns the time of the part it measures.
pub fn best_of_three(run: impl Fn() -> std::time::Duration) -> std::time::Duration {
    (0..3).map(|_| run()).min().unwrap_or_default()
}

/// The JSON value of `text` in a form that compares member for member: object members sorted
/// by key, numbers by their value.
pub fn canonical(text: &str) -> String {
    fn walk(value: &bedrock_addon_core::json::Value, out: &mut String) {
        if let Some(object) = value.as_object() {
            let mut members: Vec<_> = object.iter().collect();
            members.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));
            out.push('{');
            for (key, member) in members {
                write!(out, "{:?}:", key.as_str()).unwrap();
                walk(member, out);
                out.push(',');
            }
            out.push('}');
        } else if let Some(array) = value.as_array() {
            out.push('[');
            for element in array.as_slice() {
                walk(element, out);
                out.push(',');
            }
            out.push(']');
        } else if let Some(number) = value.as_number() {
            write!(out, "{}", number.to_f64_lossy()).unwrap();
        } else if let Some(text) = value.as_string() {
            write!(out, "{:?}", text.as_str()).unwrap();
        } else if let Some(flag) = value.as_bool() {
            out.push_str(&flag.to_string());
        } else {
            out.push_str("null");
        }
    }
    let value =
        bedrock_addon_core::json::from_str::<bedrock_addon_core::json::Value>(text).expect("JSON");
    let mut out = String::new();
    walk(&value, &mut out);
    out
}

/// How a written text differs from the text that was read, as JSON values: nothing, or the
/// rewrites the reader reports. Member order is no difference, and of a member the input
/// writes twice the last is the input's (as the client's reader has it).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rewrites {
    /// Array elements and variant bodies of the input that are not written: the entries the
    /// typed lists cannot hold, dropped with a warning; of a list file, the elements
    /// and entries the client cannot use and the defines that are no strings.
    pub dropped: usize,
    /// Numbers past the floats or the doubles, written as the largest finite one.
    pub clamped: usize,
    /// Numbers no float holds exactly, written as the shortest decimal of the float the client
    /// reads them as (`0.30000000000000004` as `0.3`).
    pub rounded: usize,
}

impl Rewrites {
    /// No difference at all.
    pub fn is_none(&self) -> bool {
        *self == Self::default()
    }

    /// Adds the counts of `other`.
    pub fn add(&mut self, other: Self) {
        self.dropped += other.dropped;
        self.clamped += other.clamped;
        self.rounded += other.rounded;
    }
}

/// The differences between the JSON value of `input` (comments allowed, a BOM skipped) and of
/// `written`. Any difference that is none of [`Rewrites`]'s is an error naming its place: a
/// member that is added, a value of another type or text, a number that changed otherwise, an
/// element that is added or reordered.
pub fn rewrites(input: &str, written: &str) -> Result<Rewrites, String> {
    rewrites_at(input, written).map(|(found, _)| found)
}

/// [`rewrites`] with the place of each dropped element or member in the input, as a finding
/// spells a path (`x/defines/1`).
pub fn rewrites_at(input: &str, written: &str) -> Result<(Rewrites, Vec<String>), String> {
    use bedrock_addon_core::json::{Value, from_str, from_str_jsonc, last_wins};

    /// The counts with the places of the drops.
    #[derive(Default)]
    struct Found {
        counts: Rewrites,
        dropped_at: Vec<String>,
    }

    impl Found {
        fn dropped(&mut self, place: &str) {
            self.counts.dropped += 1;
            self.dropped_at
                .push(place.trim_start_matches('/').to_owned());
        }
    }

    fn walk(input: &Value, written: &Value, at: &str, out: &mut Found) -> Result<(), String> {
        if let (Some(a), Some(b)) = (input.as_object(), written.as_object()) {
            for (key, _) in b {
                if !a.contains_key(key.as_str()) {
                    return Err(format!("{at}/{}: a member the input lacks", key.as_str()));
                }
            }
            for (key, member) in a {
                let place = format!("{at}/{}", key.as_str());
                match b.get(key.as_str()) {
                    Some(other) => walk(member, other, &place, out)?,
                    // A variant body that is no object is the one member the reader drops.
                    None if !member.is_object() => out.dropped(&place),
                    None => return Err(format!("{place}: an object member is not written")),
                }
            }
            return Ok(());
        }
        if let (Some(a), Some(b)) = (input.as_array(), written.as_array()) {
            // The written elements are the input's in order, some of the input's left out.
            let mut rest = b.as_slice().iter().peekable();
            for (index, element) in a.as_slice().iter().enumerate() {
                let mut trial = Found::default();
                let place = format!("{at}/{index}");
                let same = rest
                    .peek()
                    .is_some_and(|next| walk(element, next, &place, &mut trial).is_ok());
                if same {
                    rest.next();
                    out.counts.add(trial.counts);
                    out.dropped_at.append(&mut trial.dropped_at);
                } else {
                    out.dropped(&place);
                }
            }
            return match rest.next() {
                None => Ok(()),
                Some(_) => Err(format!(
                    "{at}: an element the input lacks, or another order"
                )),
            };
        }
        if let (Some(a), Some(b)) = (input.as_number(), written.as_number()) {
            let (a, b) = (a.to_f64_lossy(), b.to_f64_lossy());
            // The writer writes the shortest decimal that reads back to the float: the two
            // are the same float, not the same double.
            #[allow(
                clippy::cast_possible_truncation,
                clippy::float_cmp,
                reason = "the float the client reads"
            )]
            let same_float = a as f32 == b as f32;
            #[expect(clippy::float_cmp, reason = "the same double is the same number")]
            let equal = a == b;
            if equal {
            } else if !a.is_finite() || a.abs() > f64::from(f32::MAX) {
                out.counts.clamped += 1;
            } else if same_float {
                out.counts.rounded += 1;
            } else {
                return Err(format!("{at}: {a} is written as {b}"));
            }
            return Ok(());
        }
        let same = match (input.as_string(), written.as_string()) {
            (Some(a), Some(b)) => a.as_str() == b.as_str(),
            (None, None) => {
                (input.is_null() && written.is_null())
                    || (input.as_bool().is_some() && input.as_bool() == written.as_bool())
            }
            _ => false,
        };
        if same {
            Ok(())
        } else {
            Err(format!("{at}: {input:?} is written as {written:?}"))
        }
    }

    let input = input.strip_prefix('\u{feff}').unwrap_or(input);
    let input = from_str_jsonc::<Value>(&last_wins(input)).map_err(|error| error.to_string())?;
    let written = from_str::<Value>(written).map_err(|error| error.to_string())?;
    let mut out = Found::default();
    walk(&input, &written, "", &mut out)?;
    Ok((out.counts, out.dropped_at))
}

/// The written text `written` is the input `input` as a JSON value, member for member.
#[track_caller]
pub fn assert_written_as_authored(input: &str, written: &str) {
    match rewrites(input, written) {
        Ok(found) if found.is_none() => {}
        other => panic!("not written as authored: {other:?}\n{input}\n{written}"),
    }
}
