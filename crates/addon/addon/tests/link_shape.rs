//! The shape of the link pass's sources: what a reader finding a rule by its file name relies
//! on. The tests read the crate's own source files, which library code may never do.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// The rule files of every group directory below `src/pack/link/`, as (basename, group), the
/// group's `mod.rs` and `tests.rs` left out.
fn rule_files() -> Vec<(String, String)> {
    let link = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/pack/link");
    let mut files = Vec::new();
    for group in fs::read_dir(&link).unwrap() {
        let group = group.unwrap().path();
        if !group.is_dir() {
            continue;
        }
        let group_name = group.file_name().unwrap().to_string_lossy().into_owned();
        for file in fs::read_dir(&group).unwrap() {
            let path = file.unwrap().path();
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            if path.extension().is_some_and(|extension| extension == "rs")
                && name != "mod.rs"
                && name != "tests.rs"
            {
                files.push((name, group_name.clone()));
            }
        }
    }
    files
}

/// A rule file is named for the modules it relates (`<a>_<b>.rs`), so one name in two groups
/// is two rules a search, a stack trace or a review cannot tell apart by name.
#[test]
fn no_two_link_rule_files_share_a_name() {
    let files = rule_files();
    assert!(
        files.len() > 50,
        "the scan found {} rule files",
        files.len()
    );
    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (name, group) in files {
        groups.entry(name).or_default().push(group);
    }
    let shared: Vec<String> = groups
        .into_iter()
        .filter(|(_, groups)| groups.len() > 1)
        .map(|(name, mut groups)| {
            groups.sort();
            format!("{name} in {}", groups.join(", "))
        })
        .collect();
    assert_eq!(shared, [""; 0], "rule files sharing a name");
}
