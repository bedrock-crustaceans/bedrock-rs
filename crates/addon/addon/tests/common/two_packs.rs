//! A stack of two in-memory packs, for a test of what a stack load makes of two copies of a
//! file: the lower pack and the higher pack, each with the files the test gives it.

#![allow(dead_code, reason = "each test binary uses part of the helpers")]

use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{Behavior, OpenOptions, Pack, PackRef, PackStack, Resource, Side};
use bedrock_addon::storage::PackPath;

/// A format-2 manifest of pack `n` with one module of `module` type (`data`, `resources`)
/// and `min_engine_version` 1.21.100.
fn manifest(n: u128, module: &str) -> String {
    format!(
        r#"{{"format_version": 2, "header": {{"name": "pack {n}", "description": "", "uuid": "{}", "version": [1, 0, 0], "min_engine_version": [1, 21, 100]}}, "modules": [{{"type": "{module}", "uuid": "{}", "version": [1, 0, 0]}}]}}"#,
        uuid::Uuid::from_u128(n),
        uuid::Uuid::from_u128(n | (0xabcd_u128 << 64)),
    )
}

/// Pack `n` of `module` type holding `files`.
fn pack(n: u128, module: &str, files: &[(&str, &str)]) -> Pack {
    let mut all = vec![(
        PackPath::from_static("manifest.json"),
        manifest(n, module).into_bytes(),
    )];
    all.extend(files.iter().map(|(path, text)| {
        (
            PackPath::new(path).expect("a pack path"),
            text.as_bytes().to_vec(),
        )
    }));
    Pack::from_files(all, &OpenOptions::default()).expect("the pack opens")
}

/// The stack of two packs of `module` type, and the two packs as the stack names them:
/// `(stack, lower, higher)`.
fn stack<S: Side>(
    module: &str,
    lower: &[(&str, &str)],
    higher: &[(&str, &str)],
) -> (PackStack<S>, PackRef, PackRef) {
    let packs = [pack(2, module, higher), pack(1, module, lower)];
    let stack = PackStack::<S>::of(packs, LoadOptions::default()).expect("a stack of two packs");
    let refs: Vec<PackRef> = stack.packs().collect();
    assert_eq!(refs.len(), 2, "both packs are on the stack");
    let (higher, lower) = (refs[0].clone(), refs[1].clone());
    (stack, lower, higher)
}

/// A resource stack of two packs: `(stack, lower, higher)`.
pub fn resource_stack(
    lower: &[(&str, &str)],
    higher: &[(&str, &str)],
) -> (PackStack<Resource>, PackRef, PackRef) {
    stack("resources", lower, higher)
}

/// A behaviour stack of two packs: `(stack, lower, higher)`.
pub fn behavior_stack(
    lower: &[(&str, &str)],
    higher: &[(&str, &str)],
) -> (PackStack<Behavior>, PackRef, PackRef) {
    stack("data", lower, higher)
}
