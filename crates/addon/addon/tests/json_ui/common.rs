//! Fixture loading for the JSON UI tests: a case is a stack directory
//! `tests/fixtures/ui/ui/<case>/pack_<n>/ui/…` (`pack_0` is the lowest priority) with an
//! `expected.json` that says what the merge must give.

#![allow(dead_code, reason = "each test module uses part of the helpers")]
use std::fmt::Write as _;
use std::path::PathBuf;

use bedrock_addon::diagnostic::{Diagnostic, Severity};
use bedrock_addon::document::{Content, Merged, RawLayer};
use bedrock_addon::families::json_ui::{Control, UiDefinitions, UiFile};
use bedrock_addon::json::Value;
use bedrock_addon::pack::{OpenOptions, Pack, PackStack, Resource};
use bedrock_addon::storage::PackPath;

#[allow(unused_imports, reason = "each test file uses part of the helpers")]
pub use bedrock_addon_test_support::json;

/// The fixture directory of the family.
pub fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/ui/ui")
}

/// Every file below `dir`, as paths relative to it, sorted.
fn files_below(dir: &std::path::Path) -> Vec<(String, Vec<u8>)> {
    fn walk(root: &std::path::Path, dir: &std::path::Path, out: &mut Vec<(String, Vec<u8>)>) {
        for entry in std::fs::read_dir(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display())) {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(root, &path, out);
            } else {
                let relative = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                out.push((relative, std::fs::read(&path).unwrap()));
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, dir, &mut out);
    out.sort();
    out
}

/// A format-2 resource-pack manifest for pack `n`.
fn manifest(n: u128) -> String {
    let uuid = uuid::Uuid::from_u128(n + 1);
    let module = uuid::Uuid::from_u128((n + 1) ^ (0xabcd_u128 << 64));
    format!(
        r#"{{"format_version": 2, "header": {{"name": "pack {n}", "description": "", "uuid": "{uuid}", "version": [1, 0, 0], "min_engine_version": [1, 21, 0]}}, "modules": [{{"type": "resources", "uuid": "{module}", "version": [1, 0, 0]}}]}}"#
    )
}

/// Resource pack `n` holding `files`.
pub fn pack_of(n: usize, files: Vec<(String, Vec<u8>)>) -> Pack {
    let mut all: Vec<(PackPath, bytes::Bytes)> = vec![(
        PackPath::from_static("manifest.json"),
        bytes::Bytes::from(manifest(n as u128)),
    )];
    all.extend(
        files
            .into_iter()
            .filter(|(path, _)| path != "manifest.json")
            .map(|(path, bytes)| (PackPath::new(&path).unwrap(), bytes::Bytes::from(bytes))),
    );
    Pack::from_files(all, &OpenOptions::default()).expect("the pack opens")
}

/// A resource stack of in-memory packs, given lowest priority first.
pub fn stack_of(packs: &[&[(&str, &str)]]) -> PackStack<Resource> {
    let mut packs: Vec<Pack> = packs
        .iter()
        .enumerate()
        .map(|(n, files)| {
            pack_of(
                n,
                files
                    .iter()
                    .map(|(path, text)| ((*path).to_owned(), text.as_bytes().to_vec()))
                    .collect(),
            )
        })
        .collect();
    packs.reverse();
    PackStack::of(packs, bedrock_addon::load::LoadOptions::default()).expect("resource packs")
}

/// The stack of fixture case `case`, and how many packs it has.
pub fn case_stack(case: &str) -> (PackStack<Resource>, usize) {
    let dir = fixtures().join(case);
    let mut folders: Vec<String> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{case}: {e}"))
        .filter_map(|entry| entry.unwrap().file_name().into_string().ok())
        .filter(|name| name.starts_with("pack_"))
        .collect();
    folders.sort();
    let mut packs: Vec<Pack> = folders
        .iter()
        .enumerate()
        .map(|(n, folder)| {
            assert_eq!(
                *folder,
                format!("pack_{n}"),
                "{case}: packs are numbered from 0"
            );
            pack_of(n, files_below(&dir.join(folder)))
        })
        .collect();
    let count = packs.len();
    packs.reverse();
    (
        PackStack::of(packs, bedrock_addon::load::LoadOptions::default()).expect("resource packs"),
        count,
    )
}

/// A merged file as the JSON it writes back to.
pub fn file_json(file: &UiFile) -> Value {
    json(&file.to_json().expect("a ui file writes"))
}

/// `namespace.name` of a definition.
pub fn def_name(control: &Control) -> String {
    format!("{}.{}", control.namespace, control.key.name)
}

fn severity_name(severity: Severity) -> &'static str {
    match severity {
        Severity::Debug => "debug",
        Severity::Info => "info",
        Severity::Warning => "warning",
        Severity::Error => "error",
    }
}

/// The fixture number of the pack a finding is about (`pack_<n>`).
fn pack_number(diagnostic: &Diagnostic, count: usize) -> Option<usize> {
    let index = diagnostic.location.pack.as_ref()?.index()?;
    Some(count - 1 - index)
}

fn text_of(value: &Value, key: &str) -> Option<String> {
    value
        .as_object()?
        .get(key)?
        .as_string()
        .map(|text| text.as_str().to_owned())
}

/// Every `uiPackError` of the stack, in order, as the expectation file spells it.
fn pack_errors_of(ui: &Merged<UiDefinitions, RawLayer>, count: usize) -> facet_value::VArray {
    let mut actual_errors = facet_value::VArray::new();
    let mut reports: Vec<_> = ui.reports.iter().collect();
    reports.sort_by_key(|report| count - 1 - report.pack.index().unwrap());
    for report in reports {
        for error in &report.errors {
            let mut entry = facet_value::VObject::new();
            entry.insert(
                "pack",
                i64::try_from(count - 1 - report.pack.index().unwrap()).unwrap(),
            );
            entry.insert(
                "type",
                error.kind.key().strip_prefix("uiPackError.").unwrap(),
            );
            let mut args = facet_value::VArray::new();
            for arg in &error.args {
                args.push(arg.as_str());
            }
            entry.insert("args", args);
            entry.insert("severity", severity_name(error.severity));
            actual_errors.push(entry);
        }
    }
    actual_errors
}

/// Holds every finding of the stack to the `wanted` entries, in any order, and none left over.
fn check_diagnostics(
    case: &str,
    ui: &Merged<UiDefinitions, RawLayer>,
    wanted: &Value,
    count: usize,
) {
    let mut actual: Vec<&Diagnostic> = ui.diagnostics().collect();
    for want in wanted.as_array().expect("an array").as_slice() {
        let code = text_of(want, "code").expect("a code");
        let severity = text_of(want, "severity").expect("a severity");
        let text = text_of(want, "text");
        let at = text_of(want, "at");
        let pack = want
            .as_object()
            .and_then(|object| object.get("pack"))
            .and_then(|pack| pack.as_number()?.to_u64());
        let found = actual.iter().position(|diagnostic| {
            diagnostic.code.id() == code
                && severity_name(diagnostic.severity) == severity
                && text
                    .as_ref()
                    .is_none_or(|text| diagnostic.code.to_string() == *text)
                && at
                    .as_ref()
                    .is_none_or(|at| diagnostic.location.json_path.to_string() == *at)
                && pack.is_none_or(|pack| {
                    usize::try_from(pack)
                        .is_ok_and(|pack| pack_number(diagnostic, count) == Some(pack))
                })
        });
        match found {
            Some(index) => {
                actual.remove(index);
            }
            None => panic!(
                "{case}: no finding {code} ({severity}) {text:?} at {at:?}; left:\n{}",
                show(&actual)
            ),
        }
    }
    assert!(
        actual.is_empty(),
        "{case}: unexpected findings:\n{}",
        show(&actual)
    );
}

/// Loads case `case` and holds it to its `expected.json`:
/// - `files`: the merged JSON of every listed file, exactly;
/// - `defs`: the flattened body of every definition, by `namespace.name`, exactly;
/// - `globals`: the merged global variables, exactly;
/// - `pack_errors` (none when absent): every `uiPackError`, in order, with its pack, type,
///   arguments and severity;
/// - `diagnostics` (none when absent): every finding of the stack, in any order, by code and
///   severity, and by text, pack and breadcrumb where the entry gives them.
pub fn check(case: &str) -> PackStack<Resource> {
    let (stack, count) = case_stack(case);
    let expected = json(
        &std::fs::read_to_string(fixtures().join(case).join("expected.json"))
            .unwrap_or_else(|e| panic!("{case}/expected.json: {e}")),
    );
    let expected = expected.as_object().expect("an object");
    for key in expected.keys() {
        assert!(
            ["files", "defs", "globals", "pack_errors", "diagnostics"].contains(&key.as_str()),
            "{case}: unknown expectation `{key}`"
        );
    }
    let ui: &Merged<UiDefinitions, RawLayer> = stack.ui();

    if let Some(files) = expected.get("files") {
        let mut actual = facet_value::VObject::new();
        for (path, file) in &ui.files {
            actual.insert(path.as_str(), file_json(file));
        }
        assert_eq!(&actual.into_value(), files, "{case}: merged files");
    }
    if let Some(defs) = expected.get("defs") {
        let mut actual = facet_value::VObject::new();
        for control in ui.controls() {
            actual.insert(def_name(control), control.body());
        }
        assert_eq!(&actual.into_value(), defs, "{case}: flattened definitions");
    }
    if let Some(globals) = expected.get("globals") {
        let mut actual = facet_value::VObject::new();
        for (name, value) in &ui.global_variables.variables {
            actual.insert(name.as_str(), value.clone());
        }
        assert_eq!(&actual.into_value(), globals, "{case}: global variables");
    }

    let actual_errors = pack_errors_of(ui, count);
    let empty = facet_value::VArray::new().into_value();
    // An argument written `"*"` stands for any text (the reader's message of a parse error).
    let mut actual_errors = actual_errors.into_value();
    let wanted_errors = expected.get("pack_errors").unwrap_or(&empty);
    let wanted_list = wanted_errors.as_array().expect("an array").as_slice();
    if let Some(actual_list) = actual_errors.as_array_mut() {
        for (actual, wanted) in actual_list.as_mut_slice().iter_mut().zip(wanted_list) {
            let wanted_args = wanted.as_object().and_then(|o| o.get("args")?.as_array());
            let actual_args = actual
                .as_object_mut()
                .and_then(|o| o.get_mut("args")?.as_array_mut());
            if let (Some(actual_args), Some(wanted_args)) = (actual_args, wanted_args) {
                for (actual, wanted) in actual_args
                    .as_mut_slice()
                    .iter_mut()
                    .zip(wanted_args.as_slice())
                {
                    if wanted.as_string().is_some_and(|text| text.as_str() == "*") {
                        *actual = Value::from("*");
                    }
                }
            }
        }
    }
    assert_eq!(&actual_errors, wanted_errors, "{case}: uiPackErrors");

    check_diagnostics(
        case,
        ui,
        expected.get("diagnostics").unwrap_or(&empty),
        count,
    );
    stack
}

/// The findings one per line, for a failure message.
pub fn show(diagnostics: &[&Diagnostic]) -> String {
    diagnostics.iter().fold(String::new(), |mut out, d| {
        writeln!(
            out,
            "  {} ({}) at `{}`: {:?}",
            d.code.id(),
            severity_name(d.severity),
            d.location.json_path,
            d.code.to_string()
        )
        .unwrap();
        out
    })
}

/// The findings of a stack as `(code name, severity, text)`.
pub fn findings(stack: &PackStack<Resource>) -> Vec<(&'static str, Severity, String)> {
    stack
        .ui()
        .diagnostics()
        .map(|d| (d.code.id(), d.severity, d.code.to_string()))
        .collect()
}

/// The texts of the findings named `code`.
pub fn texts(stack: &PackStack<Resource>, code: &str) -> Vec<String> {
    findings(stack)
        .into_iter()
        .filter(|(name, _, _)| *name == code)
        .map(|(_, _, text)| text)
        .collect()
}

/// A stack of one pack that lists `ui/a.json` holding `text`.
pub fn one_file(text: &str) -> PackStack<Resource> {
    stack_of(&[&[
        ("ui/_ui_defs.json", r#"{ "ui_defs": ["ui/a.json"] }"#),
        ("ui/a.json", text),
    ]])
}

/// The definition `namespace.name` of a stack.
pub fn control<'a>(stack: &'a PackStack<Resource>, namespace: &str, name: &str) -> &'a Control {
    stack
        .ui()
        .get(namespace, name)
        .unwrap_or_else(|| panic!("no definition {namespace}.{name}"))
}
