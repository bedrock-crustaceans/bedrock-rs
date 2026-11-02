//! The `corpus_pairs.json` check of one family: every occurrence of the family in the game's
//! client packs names a file that loads through the addon with no error-level finding, and the Molang member at the occurrence's JSON pointer reads to the
//! pair's text and effective version. The family's smoke test includes this file with
//! `#[path = "common/pairs.rs"] mod pairs;` and hands it the parse of one file.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::{Diagnostics, JsonPath, Severity};
use bedrock_addon::json::{JsonSlice, Value, parse_at};
use bedrock_addon::load::{LoadContext, LoadOptions};
use bedrock_addon::molang::{MolangFieldMut, MolangFields, RawVersion, visit_fields};
use bedrock_addon::pack::PackType;
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::{MinEngineVersion, SemVersion};

/// Every Molang member of a document: JSON pointer → (text, raw version).
pub type Members = BTreeMap<String, (String, Option<RawVersion>)>;

/// One file of the family, as the parse closure gets it.
pub struct File<'a> {
    /// The file on disk.
    #[allow(
        dead_code,
        reason = "for a family that tells its files apart afterwards"
    )]
    pub path: &'a Path,
    /// The pack-relative path.
    pub at: PackPath,
    /// The file's text.
    pub text: &'a str,
    /// The pack's load context (its `min_engine_version`, a base-game pack).
    pub cx: LoadContext,
}

/// Collects the Molang members of `value`, each at `/<prefix>/<path>` (or `/<path>` without a
/// prefix).
pub fn members<T: MolangFields + ?Sized>(value: &mut T, prefix: Option<&str>, into: &mut Members) {
    visit_fields(value, &JsonPath::new(), |field| {
        let source = match field.value {
            MolangFieldMut::Value(value) => value.source().cloned(),
            MolangFieldMut::Source(source) => Some(source.clone()),
            MolangFieldMut::List(_) => None,
        };
        if let Some(source) = source {
            let pointer = match prefix {
                Some(prefix) => format!("/{prefix}/{}", field.path),
                None => format!("/{}", field.path),
            };
            into.insert(pointer, (source.as_str().to_owned(), source.raw_version()));
        }
    });
}

/// The error-level findings of `d`, as text.
pub fn errors(d: &Diagnostics) -> Vec<String> {
    d.iter()
        .filter(|f| f.severity == Severity::Error)
        .map(|f| format!("{}: {} ({:?})", f.location.json_path, f.code, f.code))
        .collect()
}

/// What [`check`] found.
#[allow(dead_code, reason = "each family reads the counts it asserts")]
pub struct Checked {
    /// Occurrences that are no keys whose member read to the pair's text and version.
    pub checked: usize,
    /// Key occurrences: matched ([`Keys::Members`]) or only counted ([`Keys::Counted`]).
    pub keys: usize,
    /// Occurrences left to another family ([`Rules::foreign`]).
    pub foreign: usize,
    /// Distinct files loaded.
    pub files: usize,
    /// The file and pointer (as the pairs write it) of every occurrence in `checked`.
    pub matched: Vec<(PathBuf, String)>,
    /// Every mismatch, unreadable file and error-level finding.
    pub failures: Vec<String>,
}

/// What a key occurrence (`"is_key": true`: the pair's text is a member's name) is held to.
#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code, reason = "each family uses one of them")]
pub enum Keys {
    /// The family has none: one is a failure of the test.
    Refused,
    /// Checked like any other occurrence, against the member `parse` returns at the key's
    /// pointer. A member without a version (a declared name that is no expression) is held to
    /// its text only.
    Members,
    /// Counted, not read: the loaded document keeps the names in a form `parse` does not
    /// return.
    Counted,
}

/// How one family's occurrences are matched against what its `parse` returns.
#[derive(Clone, Copy)]
pub struct Rules {
    /// The family's key occurrences.
    pub keys: Keys,
    /// The form pointers are compared in: applied to the occurrence's pointer and to the
    /// members' (a family the game reads case-folded lower-cases both).
    pub pointer: fn(&str) -> String,
    /// The form texts are compared in: applied to the pair's text and to the member's, after
    /// trimming (the pairs hold the text trimmed).
    pub text: fn(&str) -> String,
    /// Whether the occurrence at this pointer is another family's (a geometry file kept in a
    /// client entity folder): counted in [`Checked::foreign`], its file not read.
    pub foreign: fn(&str) -> bool,
}

impl Rules {
    /// Pointers and texts compared as written, no occurrence foreign.
    pub const fn as_written(keys: Keys) -> Rules {
        Rules {
            keys,
            pointer: str::to_owned,
            text: str::to_owned,
            foreign: |_| false,
        }
    }
}

/// Checks every non-key occurrence of `family` (the pairs' family name: `features`,
/// `feature_rules`) found below `packs`. `parse` loads one file and returns its Molang members
/// with its findings, or the findings that rejected it.
#[allow(
    dead_code,
    reason = "the families with key occurrences use `check_with_keys`"
)]
pub fn check(
    pairs: &Path,
    packs: &Path,
    family: &str,
    parse: impl FnMut(File<'_>) -> Result<(Members, Diagnostics), Diagnostics>,
) -> Checked {
    check_occurrences(
        pairs,
        packs,
        family,
        &Rules::as_written(Keys::Refused),
        parse,
    )
}

/// [`check`] for a family whose pairs include occurrences that are keys (a particle curve is
/// named by the Molang variable it sets; a render controller's array by its name; a client
/// entity's script variable), under the family's [`Rules`].
#[allow(dead_code, reason = "the families without key occurrences use `check`")]
pub fn check_with_keys(
    pairs: &Path,
    packs: &Path,
    family: &str,
    rules: &Rules,
    parse: impl FnMut(File<'_>) -> Result<(Members, Diagnostics), Diagnostics>,
) -> Checked {
    check_occurrences(pairs, packs, family, rules, parse)
}

fn check_occurrences(
    pairs: &Path,
    packs: &Path,
    family: &str,
    rules: &Rules,
    mut parse: impl FnMut(File<'_>) -> Result<(Members, Diagnostics), Diagnostics>,
) -> Checked {
    let text = std::fs::read_to_string(pairs).unwrap();
    let document: Value = parse_at(JsonSlice::new(&text)).unwrap();
    let mut cache: BTreeMap<PathBuf, Result<Members, Vec<String>>> = BTreeMap::new();
    let (mut checked, mut keys, mut foreign) = (0, 0, 0);
    let mut matched = Vec::new();
    let mut failures = Vec::new();
    let pairs = document.as_object().unwrap().get("pairs").unwrap();
    for pair in pairs.as_array().unwrap() {
        let pair = pair.as_object().unwrap();
        let expr = pair.get("expr").unwrap().as_string().unwrap().as_str();
        for occurrence in pair.get("occurrences").unwrap().as_array().unwrap() {
            let o = occurrence.as_object().unwrap();
            let field = |key: &str| o.get(key).unwrap().as_string().unwrap().as_str().to_owned();
            if field("family") != family {
                continue;
            }
            let pointer = field("pointer");
            if (rules.foreign)(&pointer) {
                foreign += 1;
                continue;
            }
            let is_key = o.get("is_key").and_then(Value::as_bool) == Some(true);
            if is_key {
                assert!(
                    rules.keys != Keys::Refused,
                    "a {family} occurrence that is a key"
                );
                if rules.keys == Keys::Counted {
                    keys += 1;
                    continue;
                }
            }
            let path = packs
                .join(field("pack_type"))
                .join(field("pack"))
                .join(field("file"));
            let members = cache.entry(path.clone()).or_insert_with(|| {
                let text = std::fs::read_to_string(&path)
                    .map_err(|e| vec![format!("{}: {e}", path.display())])?;
                let version = SemVersion::parse(&field("pack_min_engine_version"))
                    .expect("a pack min_engine_version");
                let pack_type = if field("pack_type") == "resource_packs" {
                    PackType::Resource
                } else {
                    PackType::Behavior
                };
                let cx = LoadContext::for_pack(
                    &LoadOptions::default(),
                    uuid::Uuid::nil(),
                    pack_type,
                    Some(MinEngineVersion { version }),
                    true,
                );
                let at = PackPath::new(&field("file")).unwrap();
                let fail = |d: &Diagnostics, what: &str| {
                    let mut errors = errors(d);
                    if errors.is_empty() {
                        errors.push(String::from("no error-level finding"));
                    }
                    errors
                        .into_iter()
                        .map(|e| format!("{} {what}: {e}", path.display()))
                        .collect::<Vec<_>>()
                };
                match parse(File {
                    path: &path,
                    at,
                    text: &text,
                    cx,
                }) {
                    Err(rejected) => Err(fail(&rejected, "rejected")),
                    Ok((_, d)) if !errors(&d).is_empty() => Err(fail(&d, "error")),
                    Ok((members, _)) => Ok(members
                        .into_iter()
                        .map(|(pointer, member)| ((rules.pointer)(&pointer), member))
                        .collect()),
                }
            });
            let members = match members {
                Ok(members) => members,
                Err(errors) => {
                    failures.append(errors);
                    continue;
                }
            };
            let version = o
                .get("version_effective")
                .unwrap()
                .as_number()
                .unwrap()
                .to_i64()
                .unwrap();
            let expected = (
                expr.to_owned(),
                Some(RawVersion(i16::try_from(version).unwrap())),
            );
            match members.get(&(rules.pointer)(&pointer)) {
                // The pairs hold the text trimmed (their "selection" rule); the source is as
                // written. A key that is a declared name has no version of its own.
                Some((text, version))
                    if (rules.text)(text.trim()) == (rules.text)(&expected.0)
                        && (*version == expected.1 || (is_key && version.is_none())) =>
                {
                    if is_key {
                        keys += 1;
                    } else {
                        checked += 1;
                        matched.push((path.clone(), pointer));
                    }
                }
                found => failures.push(format!(
                    "{}: {pointer}: {found:?} != {expected:?}",
                    path.display()
                )),
            }
        }
    }
    Checked {
        checked,
        keys,
        foreign,
        files: cache.len(),
        matched,
        failures,
    }
}
