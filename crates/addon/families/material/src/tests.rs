//! Unit tests of the material internals the integration tests cannot name: the cost of each
//! step that walks a file, a chain or a stack, measured on inputs built beforehand. Each
//! measures 25 000 and 100 000 entries (5 000 and 20 000 where the input is whole packs of
//! text), one test at a time, best of three, and requires four times the entries to take
//! under eight times as long (a linear step takes four to five, a quadratic one sixteen).

use bedrock_addon_core::test_cost::{alone_for, assert_linear_at};
use std::time::{Duration, Instant};

use indexmap::IndexMap;

use super::file::{
    KeyRules, Listed, MaterialDef, MaterialFile, MaterialFileForm, MaterialKey, Variant,
};
use super::group::{FileSource, MaterialGroup, MaterialGroupKind};
use super::list::ListEntry;
use super::patch::patch_with;
use super::read::{check_duplicates, read_file};
use super::set::NameSet;
use super::{combine_parsed, parse_bytes};
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::pack_identity::PackPath;

/// The sizes of a cost test: `SINGLE` entries against four times as many.
const SINGLE: usize = 25_000;
/// The sizes of the tests whose input is three packs of text parsed seven times over.
const SMALL: usize = 5_000;
/// [`assert_linear_at`] at [`SINGLE`].
#[track_caller]
fn assert_linear<I, T>(what: &str, input: impl Fn(usize) -> I, step: impl Fn(&I) -> T) {
    assert_linear_at(what, SINGLE, input, step);
}

fn value(text: &str) -> Value {
    bedrock_addon_core::json::from_str::<Value>(text).unwrap()
}

/// An object of `n` members `<prefix><i><suffix>: body`.
fn object(n: usize, prefix: &str, suffix: &str, body: &str) -> Value {
    let members: Vec<String> = (0..n)
        .map(|i| format!("\"{prefix}{i:06}{suffix}\": {body}"))
        .collect();
    value(&format!("{{{}}}", members.join(",")))
}

fn path(text: &str) -> PackPath {
    PackPath::new(text).unwrap()
}

#[test]
fn patching_a_hundred_thousand_keys_is_linear() {
    // Every key of the base matches a key of the patch under another spelling.
    assert_linear(
        "patch_with, every key rekeyed",
        |n| {
            (
                object(n, "k", ":p", r#"{"a": 1}"#),
                object(n, "k", ":q", r#"{"b": 2}"#),
            )
        },
        |(base, patch)| {
            let mut told = 0usize;
            let merged = patch_with(base.clone(), patch.clone(), &mut |_, _| told += 1);
            assert_eq!(told, merged.as_object().unwrap().len());
            merged
        },
    );
    // No key matches: the base's keys are all added.
    assert_linear(
        "patch_with, no key shared",
        |n| (object(n, "a", "", "{}"), object(n, "b", "", "{}")),
        |(base, patch)| patch_with(base.clone(), patch.clone(), &mut |_, _| {}),
    );
    // Every key of the base has one name: each merges into the first.
    assert_linear(
        "patch_with, one name",
        |n| (object(n, "x:", "", "{}"), value("{}")),
        |(base, patch)| patch_with(base.clone(), patch.clone(), &mut |_, _| {}),
    );
}

#[test]
fn checking_a_hundred_thousand_keys_for_duplicates_is_linear() {
    for (what, prefix) in [("one name", "x:"), ("all different", "x")] {
        assert_linear(
            what,
            |n| object(n, prefix, "", "{}"),
            |map| {
                let mut found = Diagnostics::default();
                check_duplicates(map.as_object().unwrap(), "materials/a.material", &mut found);
                found.len()
            },
        );
    }
}

#[test]
fn reading_a_hundred_thousand_materials_is_linear() {
    let body = r#"{"+defines": ["A"], "states": ["Blending"], "+states": ["Wireframe"],
                   "variants": [{"v": {"blendSrc": "One"}}], "samplerStates": [{"samplerIndex": 0}],
                   "vrGeometryShader": "x"}"#;
    assert_linear(
        "read_file",
        |n| object(n, "m", "", body),
        |map| {
            let mut found = Diagnostics::default();
            let rules = |_: &str| Some(KeyRules::Versioned);
            let map = map.as_object().unwrap();
            let read = read_file(map, None, "a", &rules, &|_| false, &mut found);
            assert_eq!(read.refused.len(), map.len());
            // One finding per refused material.
            assert_eq!(found.len(), read.refused.len());
            read.file.materials.len()
        },
    );
}

/// A definition that adds one variant and one define.
fn adding(variant: &str) -> MaterialDef {
    MaterialDef {
        defines: Listed {
            add: Some(vec![format!("D_{variant}")]),
            ..Listed::default()
        },
        variants: Listed {
            add: Some(vec![Variant(IndexMap::from([(
                variant.to_owned(),
                MaterialDef::default(),
            )]))]),
            ..Listed::default()
        },
        ..MaterialDef::default()
    }
}

/// A file of `n` materials whose keys `key(i)` gives.
fn file_of(n: usize, key: impl Fn(usize) -> String) -> MaterialFile {
    MaterialFile {
        form: MaterialFileForm::Wrapped {
            version: bedrock_addon_core::version::SemVersion::new(1, 0, 0),
        },
        materials: (0..n)
            .map(|i| (MaterialKey::parse(&key(i)), adding(&format!("v{i:06}"))))
            .collect(),
        extra: bedrock_addon_core::document::UnknownMembers::default(),
    }
}

/// A chain: each material inherits from the one before it.
fn chain(n: usize) -> MaterialFile {
    file_of(n, |i| match i {
        0 => "m000000".to_owned(),
        _ => format!("m{i:06}:m{:06}", i - 1),
    })
}

/// Loads `files` into a new group, one after the other, and gives the group with the number
/// of findings.
fn loaded(files: &[(&MaterialFile, &Vec<bool>)]) -> (MaterialGroup, usize) {
    let mut group = MaterialGroup::new(MaterialGroupKind::Common);
    let mut found = Vec::new();
    for (index, (file, refused)) in files.iter().enumerate() {
        let path = path(&format!("materials/f{index}.material"));
        let source = FileSource {
            index: u32::try_from(index).unwrap(),
            path: &path,
            file,
            refused,
        };
        group.load_file(&source, vec!["BASE".to_owned()], &mut found);
    }
    (group, found.len())
}

#[test]
fn creating_a_chain_of_a_hundred_thousand_materials_is_linear() {
    // Each link adds a variant to the ones it inherits: the last has them all, and no link
    // copied its parent's.
    assert_linear(
        "load_file, a chain",
        |n| (chain(n), vec![false; n]),
        |(file, refused)| {
            let (group, found) = loaded(&[(file, refused)]);
            assert_eq!((group.len(), found), (refused.len(), 0));
            let last = group.iter().last().unwrap();
            assert_eq!(last.variant_count(), refused.len());
            group
        },
    );
}

#[test]
fn a_cycle_of_a_hundred_thousand_materials_is_reported_in_linear_time() {
    assert_linear(
        "load_file, a cycle",
        |n| {
            let file = file_of(n, |i| format!("m{i:06}:m{:06}", (i + 1) % n));
            (file, vec![false; n])
        },
        |(file, refused)| {
            let (group, found) = loaded(&[(file, refused)]);
            // No material, one finding each.
            assert_eq!((group.len(), found), (0, refused.len()));
            found
        },
    );
}

#[test]
fn a_hundred_thousand_children_of_one_material_share_its_variants() {
    assert_linear(
        "load_file, siblings",
        |n| {
            let mut file = file_of(n, |i| format!("m{i:06}:root"));
            let mut root = MaterialDef::default();
            root.variants.replace = Some(
                (0..200)
                    .map(|v| Variant(IndexMap::from([(format!("r{v}"), MaterialDef::default())])))
                    .collect(),
            );
            file.materials.insert(MaterialKey::new("root"), root);
            (file, vec![false; n + 1])
        },
        |(file, refused)| {
            let (group, found) = loaded(&[(file, refused)]);
            assert_eq!((group.len(), found), (refused.len(), 0));
            assert_eq!(group.iter().last().unwrap().variant_count(), 201);
            group
        },
    );
}

#[test]
fn a_refused_material_above_a_hundred_thousand_descendants_is_linear() {
    assert_linear(
        "load_file, a refused root",
        |n| {
            let mut refused = vec![false; n];
            refused[0] = true;
            (chain(n), refused)
        },
        |(file, refused)| {
            let (group, found) = loaded(&[(file, refused)]);
            // Nothing is created; each descendant is reported.
            assert_eq!((group.len(), found), (0, refused.len() - 1));
            found
        },
    );
}

#[test]
fn redefining_and_extending_a_hundred_thousand_materials_is_linear() {
    assert_linear(
        "load_file, a second and a third file",
        |n| {
            let again = file_of(n, |i| format!("m{i:06}"));
            let extend = file_of(n, |i| format!("+m{i:06}"));
            (chain(n), again, extend, vec![false; n])
        },
        |(first, again, extend, refused)| {
            let (group, found) = loaded(&[(first, refused), (again, refused), (extend, refused)]);
            // The second file's definitions are all ignored; the third extends each,
            // replacing the variant each already has.
            assert_eq!((group.len(), found), (refused.len(), 2 * refused.len()));
            let last = group.iter().last().unwrap();
            assert_eq!(last.file.as_str(), "materials/f2.material");
            assert_eq!(last.variant_count(), refused.len());
            group
        },
    );
}

#[test]
fn a_name_set_shares_what_it_inherits() {
    // Twenty thousand sets, each one name larger than the last, in the time and the space of
    // one set and a logarithm: a set that copied its parent would take a quadratic time.
    assert_linear(
        "NameSet::with along a chain",
        |n| n,
        |n| {
            let mut sets = Vec::with_capacity(*n);
            let mut set = NameSet::default();
            for i in 0..*n {
                set = set.with(&format!("v{i:06}")).0;
                sets.push(set.clone());
            }
            assert_eq!(sets[n / 2].len(), n / 2 + 1);
            sets
        },
    );
}

#[test]
fn the_defines_of_a_list_entry_with_a_hundred_thousand_names_are_linear() {
    assert_linear(
        "ListEntry::base_defines",
        |n| ListEntry {
            path: String::new(),
            defines: Some((0..n).map(|i| format!("D{i}")).collect()),
            add_defines: Some((0..n).map(|i| format!("A{i}")).collect()),
            sub_defines: Some((0..n).map(|i| format!("D{}", n - 1 - i)).collect()),
            extra: bedrock_addon_core::document::UnknownMembers::default(),
        },
        |entry| {
            let defines = entry.base_defines();
            assert_eq!(defines.len(), entry.add_defines.as_ref().unwrap().len());
            defines
        },
    );
}

/// The text of a wrapped file of `n` materials `m<i>`, each inheriting `parent(i)`.
fn file_text(n: usize, key: impl Fn(usize) -> String, body: &str) -> String {
    let members: Vec<String> = (0..n).map(|i| format!("\"{}\": {body}", key(i))).collect();
    format!(
        "{{\"materials\": {{\"version\": \"1.0.0\", {}}}}}",
        members.join(",")
    )
}

fn chain_key(i: usize) -> String {
    match i {
        0 => "m000000".to_owned(),
        _ => format!("m{i:06}:m{:06}", i - 1),
    }
}

#[test]
fn combining_three_packs_of_twenty_thousand_materials_is_linear() {
    // Three copies of one file, each key of the middle copy spelt differently from the lowest
    // (a finding for each), parsed beforehand: the merge, the read and the creation are measured.
    assert_linear_at(
        "combine, three copies of a chain",
        SMALL,
        |n| {
            let low = file_text(n, |i| format!("m{i:06}"), r#"{"+defines": ["LOW"]}"#);
            let mid = file_text(n, chain_key, r#"{"+variants": [{"v": {}}]}"#);
            let top = file_text(n, chain_key, r#"{"depthFunc": "Always"}"#);
            let listing = r#"[{"path": "materials/a.material"}]"#;
            let parsed = || {
                vec![
                    parse_bytes(2, "materials/a.material", low.as_bytes()),
                    parse_bytes(2, "materials/common.json", listing.as_bytes()),
                    parse_bytes(1, "materials/a.material", mid.as_bytes()),
                    parse_bytes(0, "materials/a.material", top.as_bytes()),
                ]
            };
            // `combine` consumes its input: one set per measured run.
            (
                std::cell::RefCell::new((0..7).map(|_| parsed()).collect::<Vec<_>>()),
                n,
            )
        },
        |(parsed, n)| {
            let input = parsed.borrow_mut().pop().expect("an input per run");
            let materials = combine_parsed(input);
            assert_eq!(materials.common.len(), *n);
            // Per key but the first (spelt the same in every copy, and without a parent): one
            // finding for the changed spelling, one for the variant it inherits and adds again:
            // both are informations of the crate's own.
            let informations = materials
                .diagnostics()
                .filter(|f| f.severity == bedrock_addon_core::diagnostic::Severity::Info)
                .count();
            assert_eq!(informations, 2 * (n - 1));
            assert_eq!(materials.diagnostics().count(), 2 * (n - 1));
            materials
        },
    );
}

#[test]
fn a_list_of_twenty_thousand_entries_is_processed_in_linear_time() {
    assert_linear_at(
        "combine, a long list",
        SMALL,
        |n| {
            // A tenth of the entries repeat one path; the others name files no pack
            // has.
            let entries: Vec<String> = (0..n)
                .map(|i| match i % 10 {
                    0 => r#"{"path": "materials/a.material", "+defines": ["X"]}"#.to_owned(),
                    // Long paths that differ at their end: a scan of the paths seen so far
                    // compares each in full.
                    _ => format!(
                        r#"{{"path": "materials/{}{i:06}.material"}}"#,
                        "missing_".repeat(100)
                    ),
                })
                .collect();
            let listing = format!("[{}]", entries.join(","));
            let file = file_text(10, chain_key, "{}");
            let parsed = || {
                vec![
                    parse_bytes(0, "materials/a.material", file.as_bytes()),
                    parse_bytes(0, "materials/fancy.json", listing.as_bytes()),
                ]
            };
            (
                std::cell::RefCell::new((0..7).map(|_| parsed()).collect::<Vec<_>>()),
                n,
            )
        },
        |(parsed, n)| {
            let input = parsed.borrow_mut().pop().expect("an input per run");
            let materials = combine_parsed(input);
            assert_eq!(materials.fancy.len(), 10);
            assert_eq!(materials.diagnostics().count(), n / 10 - 1);
            materials
        },
    );
}

#[test]
fn resolving_the_end_of_a_chain_walks_it_once() {
    assert_linear(
        "resolve, the last of a chain",
        |n| {
            let file = file_text(
                n,
                chain_key,
                r#"{"+defines": ["A"], "+states": ["Blending"]}"#,
            );
            let listing = r#"[{"path": "materials/a.material"}]"#;
            let materials = combine_parsed(vec![
                parse_bytes(0, "materials/a.material", file.as_bytes()),
                parse_bytes(0, "materials/common.json", listing.as_bytes()),
            ]);
            (materials, format!("m{:06}", n - 1))
        },
        |(materials, last)| {
            // A hundred resolutions: one alone is too short to measure.
            (0..100)
                .map(|_| {
                    let resolved = materials.resolve(MaterialGroupKind::Common, last).unwrap();
                    assert_eq!(resolved.states, ["Blending"]);
                    resolved.defines.len()
                })
                .sum::<usize>()
        },
    );
}

#[test]
fn resolving_every_material_of_a_chain_walks_it_once() {
    // Each link is resolved from its parent's state: no link walks back to the root.
    assert_linear(
        "resolve_all, every material of a chain",
        |n| {
            let file = file_text(
                n,
                chain_key,
                r#"{"+defines": ["A"], "+states": ["Blending"]}"#,
            );
            let listing = r#"[{"path": "materials/a.material"}]"#;
            let materials = combine_parsed(vec![
                parse_bytes(0, "materials/a.material", file.as_bytes()),
                parse_bytes(0, "materials/common.json", listing.as_bytes()),
            ]);
            (materials, n)
        },
        |(materials, n)| {
            let all = materials.resolve_all(MaterialGroupKind::Common);
            assert_eq!(all.len(), *n);
            let (name, last) = all.last().unwrap();
            assert_eq!(name, &format!("m{:06}", n - 1));
            assert_eq!(
                (&last.states, &last.defines),
                (&vec!["Blending".to_owned()], &vec!["A".to_owned()])
            );
            all.len()
        },
    );
}

#[test]
fn the_names_of_a_group_are_looked_up_without_a_scan() {
    assert_linear(
        "MaterialGroup::contains for every name",
        |n| loaded(&[(&chain(n), &vec![false; n])]).0,
        |group| {
            let names = group.len();
            let found = (0..names)
                .filter(|i| group.contains(&format!("m{i:06}")))
                .filter(|i| group.contains(&format!("m{i:06}.v{i:06}")))
                .filter(|i| !group.contains(&format!("m{i:06}.v{:06}", i + 1)))
                .count();
            assert_eq!(found, names);
            found
        },
    );
}

#[test]
fn a_material_prints_and_compares_as_its_own_link() {
    let (group, _) = loaded(&[(&chain(1000), &vec![false; 1000])]);
    let last = group.get("m000999").unwrap();
    let printed = format!("{last:?}");
    assert!(printed.len() < 300, "{printed}");
    assert!(printed.contains("1000 names"), "{printed}");
    assert!(!printed.contains("m000001"), "{printed}");
    // Equal to itself, different from its parent, without walking either chain.
    assert_eq!(last, &last.clone());
    assert_ne!(last, group.get("m000998").unwrap());
}

#[test]
fn every_material_of_two_loads_of_a_chain_compares_in_linear_time() {
    // Two loads share no variant set: a comparison that read what each link inherits would
    // read the whole chain's variants once per link.
    assert_linear(
        "Material == Material, every link of two loads of a chain",
        |n| {
            let (file, refused) = (chain(n), vec![false; n]);
            let one = loaded(&[(&file, &refused)]).0;
            let other = loaded(&[(&file, &refused)]).0;
            (one, other)
        },
        |(one, other)| {
            let equal = one.iter().zip(other.iter()).filter(|(a, b)| a == b).count();
            assert_eq!(equal, one.len());
            assert_eq!(one, other);
            equal
        },
    );
}

#[test]
#[allow(clippy::disallowed_methods, reason = "a cost test reads the clock")]
fn two_loads_of_a_chain_of_twenty_thousand_compare_in_under_a_second() {
    let (file, refused) = (chain(20_000), vec![false; 20_000]);
    let one = loaded(&[(&file, &refused)]).0;
    let other = loaded(&[(&file, &refused)]).0;
    // A guard against a hang, not a cost bound: the comparison takes about 5 ms.
    let ((equal, groups), took) = alone_for(|| {
        let started = Instant::now();
        let equal = one.iter().zip(other.iter()).filter(|(a, b)| a == b).count();
        let groups = one == other;
        ((equal, groups), started.elapsed())
    });
    assert_eq!((equal, groups), (20_000, true));
    assert!(took < Duration::from_secs(1), "{took:?}");
}

#[test]
fn a_material_is_equal_to_the_same_link_of_another_load_and_to_no_other() {
    let load = |file: &MaterialFile| loaded(&[(file, &vec![false; file.materials.len()])]).0;
    let one = load(&chain(4));
    let same = load(&chain(4));
    assert_eq!(one.get("m000003"), same.get("m000003"));
    assert_eq!(one, same);
    // Another parent, by name.
    let mut other = file_of(4, |i| match i {
        0 => "m000000".to_owned(),
        3 => "m000003:m000001".to_owned(),
        _ => format!("m{i:06}:m{:06}", i - 1),
    });
    let reparented = load(&other);
    assert_ne!(one.get("m000003"), reparented.get("m000003"));
    assert_eq!(one.get("m000002"), reparented.get("m000002"));
    assert_ne!(one, reparented);
    // Another variant of its own, the same number of them.
    other = chain(4);
    other.materials[3] = adding("other");
    let varied = load(&other);
    assert_ne!(one.get("m000003"), varied.get("m000003"));
    assert_eq!(
        one.get("m000003").unwrap().variant_count(),
        varied.get("m000003").unwrap().variant_count()
    );
    // `variants` in place of `+variants`: the inherited ones are dropped.
    other = chain(4);
    let own = other.materials[3].variants.add.take();
    other.materials[3].variants.replace = own;
    let replaced = load(&other);
    assert_ne!(one.get("m000003"), replaced.get("m000003"));
    // The link above it differs in what it adds: this link, compared by its parent's name, is
    // the same link; the groups differ.
    other = chain(4);
    other.materials[1] = adding("other");
    let above = load(&other);
    assert_ne!(one.get("m000001"), above.get("m000001"));
    assert_eq!(one.get("m000003"), above.get("m000003"));
    assert_ne!(one, above);
}

#[test]
fn a_name_of_a_hundred_thousand_dots_is_looked_up_in_linear_time() {
    // Every dot of the name is a place where a material's name could end: each is tried
    // without reading the name again from its start.
    assert_linear(
        "MaterialGroup::contains, a name of n dots",
        |n| {
            let group = loaded(&[(&chain(10), &vec![false; 10])]).0;
            // A material of the group followed by dots, and dots alone.
            (group, format!("m000003{}", ".x".repeat(n)), "a.".repeat(n))
        },
        |(group, after_a_material, dots)| {
            assert!(!group.contains(after_a_material));
            assert!(!group.contains(dots));
            // The lookup still finds what is there.
            assert!(group.contains("m000003.v000003"));
            assert!(!group.contains("m000003.v000004"));
        },
    );
}

#[test]
fn the_spans_of_a_hundred_thousand_findings_are_found_in_linear_time() {
    // A finding in every material of one file and one at every entry of one long array: each
    // level of the text is split once for all the paths through it, never once per path.
    assert_linear(
        "span::spans_of, a path per material and per entry of one array",
        |n| {
            let materials: Vec<String> = (0..n)
                .map(|i| format!("\"m{i:06}\": {{\"blendSrc\": \"Bad\"}}"))
                .collect();
            let entries: Vec<String> = (0..n).map(|i| i.to_string()).collect();
            let text = format!(
                "{{{}, \"long\": {{\"defines\": [{}]}}}}",
                materials.join(","),
                entries.join(",")
            );
            let mut paths = Vec::with_capacity(2 * n);
            for i in 0..n {
                let mut at = bedrock_addon_core::diagnostic::JsonPath::new();
                at.push(&format!("m{i:06}"));
                at.push("blendSrc");
                paths.push(at);
            }
            for i in 0..n {
                let mut at = bedrock_addon_core::diagnostic::JsonPath::new();
                at.push("long");
                at.push("defines");
                at.push_index(i);
                paths.push(at);
            }
            (text, paths)
        },
        |(text, paths)| {
            let wanted: Vec<&bedrock_addon_core::diagnostic::JsonPath> = paths.iter().collect();
            let spans = super::span::spans_of(text.as_bytes(), &wanted);
            assert_eq!(spans.len(), paths.len());
            assert!(spans.iter().all(Option::is_some));
            // The last entry of the array is the token of its number.
            let last = spans[spans.len() - 1].expect("a span");
            let start = last.offset as usize;
            let token = &text[start..start + last.len as usize];
            assert_eq!(token, (paths.len() / 2 - 1).to_string());
        },
    );
}
