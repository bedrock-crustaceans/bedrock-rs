//! The full-stack smoke: whole trees of the game's own packs, which are not in the repository,
//! are loaded the way a server loads its world, every family at once, and what the load, the
//! link pass and the writer come to is pinned, number by number.
//!
//! Six trees: a Bedrock Dedicated Server install (`server_install`), two client installs
//! (`client_install_a`, `client_install_b`), the sample packs (`samples`), and the two halves
//! of a split client pack tree (`split_server`, `split_client`). An install is a repository
//! over its `behavior_packs/` and `resource_packs/`, the world's lists by
//! `common::world::base_game_world` (the base-game layer in vanilla's order, every other pack
//! of the install above it), its app data, both stacks through `Driver` and `load_all()`,
//! every Molang field compiled. The sample packs are one behaviour and one resource pack.
//!
//! Per tree, three aspects (`full/support.rs`, `full/write.rs`):
//!
//! - **load**: the error-level findings are the pinned ones, each with a sentence saying why
//!   the game reports it; the count of what every collection holds and of every finding by severity,
//!   code and folder is pinned; with no experiment, and with every experiment on;
//! - **link**: `link()` has no error-level finding; its findings are pinned by code, kind of
//!   the target and folder of the source (which name the rule), each number with
//!   a sentence saying what of the game's files raises it; `link_with` without the lints
//!   returns the findings above the information level; a host that answers for the kinds the
//!   crate has no list of leaves the pinned rest; `Strategy::Rayon` gives the sequential
//!   load and pass;
//! - **write**: every pack goes through `PackWriter::from_pack` and `into_pack` with its
//!   identity, files and bytes, and the world of the copies is the tree's world; every typed
//!   document is put back through `PackWriter::put` and reads as its value, and the world of
//!   the rewritten packs holds the same documents.
//!
//! A test binary may use 600 s of CPU, and a tree loaded four times with its documents
//! written twice needs more: the aspects are binaries of their own. This one is the load of
//! the server install, the load and link of it on a thread with 1 MiB of stack, and the tests
//! of the harness that need no game files. The others: `full_server_install_link`,
//! `full_server_install_write`, and `full_<tree>`, `full_<tree>_link`, `full_<tree>_write` for
//! the five other trees.
//!
//! Run each with `BEDROCK_ADDON_CORPUS=<folder> cargo nextest run -p bedrock_addon_std --test
//! <binary> -- --show-output`; without the variable every test that needs the game's files
//! prints `SKIPPED (not a pass):` and passes, which is no pass of the smoke. The pinned numbers are the
//! files of `full/pins/`; `full/support.rs` says how they are taken again.

#[macro_use]
mod common;
#[macro_use]
#[path = "full/support.rs"]
mod support;
#[path = "full/write.rs"]
mod write;

use std::collections::{BTreeMap, BTreeSet};

use bedrock_addon::load::{CollectionSet, LoadOptions};
use bedrock_addon::pack::{PackStack, StackSet};
use support::{Counts, bare, differences, every_key, no_key};

load_tests!(server_install, SERVER_INSTALL);

mod server_install_default_world {
    default_world_test!(SERVER_INSTALL, pinned);
}

#[test]
fn a_pin_file_that_does_not_exist_is_a_failure_and_no_empty_set_of_pins() {
    let missing = support::missing_pin_file("no_tree.no_aspect", false).expect("a failure");
    assert!(missing.contains("no_tree.no_aspect.txt"), "{missing}");
    assert!(missing.contains("there is no pin file"), "{missing}");
    assert_eq!(support::missing_pin_file("server_install.load", true), None);
}

#[test]
fn the_server_install_loads_and_links_on_a_thread_with_one_mebibyte_of_stack() {
    support::check_on_a_small_stack(support::SERVER_INSTALL, 1024 * 1024);
}

// ---- the harness, without game files ----------------------------------------------------------

#[test]
fn a_kept_error_is_no_refusal_of_the_link_pass_and_a_rejecting_error_is() {
    use bedrock_addon::diagnostic::{Code, Diagnostic, Location, Phase, Severity, codes};
    let error = || {
        Diagnostic::new(
            Code::from(&codes::UNRECOGNIZED_EVENT_STRING),
            Severity::Error,
            Phase::Link,
            Location::default(),
        )
    };
    let kept = error().into_kept();
    assert!(
        support::refusing_lines([&kept]).is_empty(),
        "a kept error refuses nothing: the game logs it and goes on"
    );
    assert_eq!(support::refusing_lines([&error(), &kept]).len(), 1);
}

#[test]
fn the_clean_default_world_counts_a_kept_link_error_that_the_refusal_check_lets_through() {
    use bedrock_addon::diagnostic::{Code, Diagnostic, Location, Phase, Severity, codes};
    let kept = Diagnostic::new(
        Code::from(&codes::UNRECOGNIZED_EVENT_STRING),
        Severity::Error,
        Phase::Link,
        Location::default(),
    )
    .into_kept();
    assert!(support::refusing_lines([&kept]).is_empty());
    assert_eq!(
        support::error_level_lines([&kept]).len(),
        1,
        "a kept error is an error-level finding, which a clean world has none of"
    );
}

fn pinned(entries: &[(&str, usize, &str)]) -> BTreeMap<String, (usize, String)> {
    entries
        .iter()
        .map(|(key, count, note)| ((*key).to_owned(), (*count, (*note).to_owned())))
        .collect()
}

fn found(entries: &[(&str, usize)]) -> Counts {
    entries
        .iter()
        .map(|(key, count)| ((*key).to_owned(), *count))
        .collect()
}

#[test]
fn equal_counts_have_no_difference_from_their_pins() {
    let pins = pinned(&[("W A @ x", 2, ""), ("I B @ y", 1, "why")]);
    assert!(differences(&pins, &found(&[("W A @ x", 2), ("I B @ y", 1)])).is_empty());
}

#[test]
fn a_count_one_more_or_one_less_a_new_key_and_a_lost_key_are_each_told() {
    let pins = pinned(&[("W A @ x", 2, ""), ("I B @ y", 1, "")]);
    assert_eq!(
        differences(&pins, &found(&[("W A @ x", 3), ("I B @ y", 1)])),
        ["  W A @ x: pinned 2, found 3"]
    );
    assert_eq!(
        differences(&pins, &found(&[("W A @ x", 1), ("I B @ y", 1)])),
        ["  W A @ x: pinned 2, found 1"]
    );
    assert_eq!(
        differences(
            &pins,
            &found(&[("W A @ x", 2), ("I B @ y", 1), ("E C @ z", 1)])
        ),
        ["  E C @ z: not pinned, found 1"]
    );
    assert_eq!(
        differences(&pins, &found(&[("W A @ x", 2)])),
        ["  I B @ y: pinned 1, found none"]
    );
}

#[test]
fn a_pinned_count_that_needs_a_sentence_and_has_none_is_named() {
    let pins = pinned(&[("W A @ x", 2, ""), ("I B @ y", 1, "why")]);
    assert_eq!(bare(&pins, every_key), ["W A @ x"]);
    assert!(bare(&pins, no_key).is_empty());
    assert!(bare(&pins, |key| key.starts_with("I ")).is_empty());
}

/// `crafting_catalog` as `CraftingCatalog`.
fn variant(accessor: &str) -> String {
    accessor
        .split('_')
        .map(|word| {
            let mut letters = word.chars();
            letters.next().map_or_else(String::new, |first| {
                first.to_uppercase().chain(letters).collect()
            })
        })
        .collect()
}

#[test]
fn every_collection_of_the_stacks_is_counted() {
    // Two stacks without a pack: every accessor answers, with nothing but the nine behaviour
    // trees the game has built in.
    let set = StackSet {
        behavior: PackStack::of([], LoadOptions::default()).expect("a stack"),
        resource: PackStack::of([], LoadOptions::default()).expect("a stack"),
    };
    let counted = support::documents(&set);
    let held: Vec<(&String, &usize)> = counted.iter().filter(|(_, count)| **count > 0).collect();
    assert_eq!(
        held,
        [(&String::from("behavior behavior_trees documents"), &9)]
    );
    // `<side> <accessor> <what>`: the accessors counted, as their `Collection` variants.
    let accessors: BTreeSet<String> = counted
        .keys()
        .map(|key| variant(key.split(' ').nth(1).expect("an accessor")))
        .collect();
    let all = format!("{:?}", CollectionSet::ALL);
    let all: BTreeSet<String> = all
        .trim_matches(['{', '}'])
        .split(", ")
        .map(str::to_owned)
        .collect();
    assert_eq!(accessors, all, "the accessors `support::documents` counts");
    // 53 of them on one side, the five of both sides on each.
    let sided: BTreeSet<String> = counted
        .keys()
        .map(|key| key.rsplit_once(' ').expect("a count's name").0.to_owned())
        .collect();
    assert_eq!((all.len(), sided.len()), (58, 63));
}

#[test]
fn the_document_types_are_the_writer_suite_s() {
    let tests = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let read = |path: std::path::PathBuf| {
        std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
    };
    let ours = read(tests.join("full/kinds.rs"));
    let theirs = read(tests.join("../../addon/tests/writer_round_trip/rows.rs"));
    let (_, table) = ours
        .split_once("` lines ----\n")
        .expect("the marker line of kinds.rs");
    // A gate line: `#[cfg(feature = "<name>")]`, alone on its line.
    // `#[cfg(feature = "x")]` or `#[cfg(all(feature = "x", feature = "y"))]`.
    let is_feature =
        |name: &str| !name.is_empty() && name.chars().all(|c| c.is_ascii_lowercase() || c == '_');
    let is_gate = |line: &str| {
        let Some(inner) = line
            .trim()
            .strip_prefix("#[cfg(")
            .and_then(|rest| rest.strip_suffix(")]"))
        else {
            return false;
        };
        let terms = inner
            .strip_prefix("all(")
            .and_then(|rest| rest.strip_suffix(')'))
            .unwrap_or(inner);
        terms.split(", ").all(|term| {
            term.strip_prefix("feature = \"")
                .and_then(|rest| rest.strip_suffix('"'))
                .is_some_and(is_feature)
        })
    };
    let gates = theirs.lines().filter(|line| is_gate(line)).count();
    assert!(
        gates > 40,
        "{gates} feature gates in the writer suite's table"
    );
    // And a block `#[cfg_attr(not(feature = "full"), allow(…))]` of several lines, which
    // names a feature this crate does not have.
    let mut ungated = String::new();
    let mut in_block = false;
    let mut lines = theirs.lines().peekable();
    while let Some(line) = lines.next() {
        if in_block {
            in_block = line.trim() != ")]";
            continue;
        }
        if line.trim() == "#[cfg_attr("
            && lines
                .peek()
                .is_some_and(|next| next.trim() == "not(feature = \"full\"),")
        {
            in_block = true;
            continue;
        }
        if !is_gate(line) {
            ungated.push_str(line);
            ungated.push('\n');
        }
    }
    assert!(
        table == ungated,
        "full/kinds.rs differs from writer_round_trip/rows.rs without its feature gates"
    );
    // And the table that results names every document type once.
    let names = write::kind_names();
    assert_eq!(
        names.len(),
        names.iter().collect::<BTreeSet<_>>().len(),
        "{names:?}"
    );
    assert!(names.len() >= 70, "{} document types", names.len());
}

#[test]
fn two_values_are_told_apart_by_every_member_they_differ_in() {
    let members = |a: &str, b: &str| write::differing_members(a, b);
    let read = r#"Item { description: Description { identifier: Id<item>("demo:a"), menu_category: MenuCategory { category: Some(Undefined), group: None } }, legacy: Legacy { shared: {"minecraft:icon": "a", "minecraft:use_modifiers": {"use_duration": 1.6}} }, version: FormatVersion { major: 1, minor: 21, patch: 100 } }"#;
    assert_eq!(members(read, read), Vec::<String>::new());
    // One member, a field of a nested struct.
    let category = read.replace("Some(Undefined)", "Some(ItemCommandOnly)");
    assert_eq!(members(read, &category), ["menu_category/category"]);
    // The `category` of another struct is not that one.
    assert_eq!(
        members("L { category: Some(\"Items\") }", "L { category: None }"),
        ["category"]
    );
    // A second difference behind the first is named too: a key added inside a JSON text
    // the value holds, and a number of a version by the version's name.
    let both = category
        .replace(
            r#"{"use_duration": 1.6}"#,
            r#"{"use_duration": 1.6, "emit_vibrations": true}"#,
        )
        .replace("minor: 21", "minor: 26");
    assert_eq!(
        members(read, &both),
        [
            "menu_category/category",
            "shared/minecraft:use_modifiers",
            "version"
        ]
    );
    // A value of another shape, a string with brackets and a comma in it, and no name at all.
    assert_eq!(
        members(read, &read.replace("group: None", "group: Some(\"x, y)\")")),
        ["group"]
    );
    assert_eq!(members("Text(\"a\")", "Text(\"b\")"), ["?"]);
    assert_eq!(
        members("A { raw: \"{\\\"k\\\": 1}\" }", "A { raw: \"\" }"),
        ["raw"]
    );
}

/// A behaviour pack of `uuid` at 0.0.1 named `name`, with one entity `demo:<name>`, written
/// below `dir/behavior_packs/<name>`.
fn write_entity_pack(dir: &std::path::Path, name: &str, uuid: &str) {
    let pack = dir.join("behavior_packs").join(name);
    std::fs::create_dir_all(pack.join("entities")).expect("the pack folder");
    std::fs::write(
        pack.join("manifest.json"),
        format!(
            r#"{{"format_version": 2, "header": {{"name": "{name}", "description": "", "uuid": "{uuid}",
              "version": [0, 0, 1], "min_engine_version": [1, 21, 0]}},
              "modules": [{{"type": "data", "uuid": "00000000-0000-4000-8000-00000000000{}", "version": [0, 0, 1]}}]}}"#,
            name.len() % 10
        ),
    )
    .expect("the manifest");
    std::fs::write(
        pack.join("entities").join(format!("{name}.json")),
        format!(
            r#"{{"format_version": "1.21.0", "minecraft:entity": {{"description": {{"identifier": "demo:{name}",
              "is_spawnable": false, "is_summonable": true}}, "components": {{}}}}}}"#
        ),
    )
    .expect("the entity");
}

#[test]
fn a_run_without_experiments_leaves_an_experimental_pack_off_the_stack_and_unread() {
    let dir = tempfile::tempdir().expect("a folder");
    // The Brain Prototype behaviour pack's UUID (`VANILLA_EXPERIMENTAL_PACKS`), and a pack of
    // no experiment beside it.
    write_entity_pack(dir.path(), "brain", "c5b9eaf5-41cc-4a37-838b-3f6ef6fdf7e3");
    write_entity_pack(dir.path(), "plain", "5e1f0a3c-3a61-4b1e-9e0e-3c1f5b0a6d21");
    std::fs::create_dir_all(dir.path().join("resource_packs")).expect("the resource folder");
    let tree = support::Tree {
        name: "experimental_gate",
        entry: "",
        below: "",
        samples: false,
        client: false,
    };
    let ids = |set: &StackSet| -> Vec<String> {
        set.behavior
            .entities()
            .iter()
            .map(|(id, _)| id.to_string())
            .collect()
    };
    let off = support::stacks(&tree, dir.path(), support::Run::SERVER);
    let off_ids = ids(&off);
    assert!(off_ids.contains(&"demo:plain".to_owned()), "{off_ids:?}");
    assert!(
        !off_ids.contains(&"demo:brain".to_owned()),
        "the Brain Prototype pack is read with its experiment off: {off_ids:?}"
    );
    let on = support::stacks(
        &tree,
        dir.path(),
        support::Run {
            experiments: true,
            ..support::Run::SERVER
        },
    );
    let on_ids = ids(&on);
    assert!(on_ids.contains(&"demo:brain".to_owned()), "{on_ids:?}");
    assert!(on_ids.contains(&"demo:plain".to_owned()), "{on_ids:?}");
}
