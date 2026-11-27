//! Stacks of the fixture packs of `tests/fixtures/script/script_bundle/packs/`: the packs, the
//! behaviour stacks of them, and the bundle checks that load a case through `scripts()`.

use bedrock_addon::families::script::ScriptBundle;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{Behavior, OpenOptions, Pack, PackOrigin, PackStack};
use bedrock_addon::storage::PackPath;

use super::common::{case_files, expected, script_manifest, shape_of};

/// A pack of `files`, opened as a pack of `origin`.
pub fn pack_of(files: Vec<(String, Vec<u8>)>, origin: PackOrigin) -> Pack {
    let files = files
        .into_iter()
        .map(|(path, bytes)| (PackPath::new(&path).expect("a pack path"), bytes));
    let options = OpenOptions {
        origin,
        ..OpenOptions::default()
    };
    Pack::from_files(files, &options).unwrap_or_else(|error| panic!("{error}"))
}

/// The behaviour stack of the fixture packs `cases`, the first on top.
pub fn stack_of(cases: &[&str]) -> PackStack<Behavior> {
    let packs = cases
        .iter()
        .map(|case| pack_of(case_files(case), PackOrigin::User));
    PackStack::<Behavior>::of(packs, LoadOptions::default()).expect("behaviour packs")
}

/// A behaviour stack of one script pack with `files` (paths in the pack) beside its manifest.
pub fn script_stack(files: &[(&str, &[u8])]) -> PackStack<Behavior> {
    let mut all = vec![(
        "manifest.json".to_owned(),
        script_manifest("probe", 7, "scripts/main.js"),
    )];
    all.extend(
        files
            .iter()
            .map(|(path, bytes)| ((*path).to_owned(), bytes.to_vec())),
    );
    PackStack::<Behavior>::of([pack_of(all, PackOrigin::User)], LoadOptions::default())
        .expect("a behaviour pack")
}

/// The bundle of the pack named `SB_<case>` in `stack`.
pub fn bundle_of<'a>(stack: &'a PackStack<Behavior>, case: &str) -> &'a ScriptBundle {
    let name = format!("SB_{case}");
    stack
        .scripts()
        .iter()
        .find(|bundle| bundle.pack.name() == name)
        .unwrap_or_else(|| panic!("no bundle of {name}"))
}

/// Loads the fixture pack `case` alone and requires its bundle to be what its expected file
/// says, part by part.
pub fn check(case: &str) {
    let stack = stack_of(&[case]);
    check_in(&stack, case);
}

/// Requires the bundle of `case` in `stack` to be what its expected file says.
pub fn check_in(stack: &PackStack<Behavior>, case: &str) {
    let found = shape_of(bundle_of(stack, case));
    let wanted = expected(case);
    assert_eq!(found.members, wanted.members, "{case}: members");
    assert_eq!(found.others, wanted.others, "{case}: others");
    assert_eq!(found.main, wanted.main, "{case}: main");
    assert_eq!(
        found.reachable_static, wanted.reachable_static,
        "{case}: reachable_static"
    );
    assert_eq!(
        found.reachable_dynamic, wanted.reachable_dynamic,
        "{case}: reachable_dynamic"
    );
    for (index, (found, wanted)) in found.imports.iter().zip(&wanted.imports).enumerate() {
        assert_eq!(found, wanted, "{case}: import {index}");
    }
    assert_eq!(found.imports.len(), wanted.imports.len(), "{case}: imports");
    for (index, (found, wanted)) in found
        .diagnostics
        .iter()
        .zip(&wanted.diagnostics)
        .enumerate()
    {
        assert_eq!(found, wanted, "{case}: finding {index}");
    }
    assert_eq!(
        found.diagnostics.len(),
        wanted.diagnostics.len(),
        "{case}: findings {:#?}",
        found.diagnostics
    );
}
