//! Stacks of the camera tests: behaviour packs of `(path, text)` files, top first.

use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{Behavior, PackStack};

use super::common::leak;
use super::pack_common::bp;

/// A behaviour stack of packs, top first; each pack is its `(path, text)` files.
pub fn stack(packs: &[&[(&'static str, String)]]) -> PackStack<Behavior> {
    stack_with(packs, LoadOptions::default())
}

/// [`stack`] loaded with `options`.
pub fn stack_with(
    packs: &[&[(&'static str, String)]],
    options: LoadOptions,
) -> PackStack<Behavior> {
    let packs: Vec<_> = packs
        .iter()
        .enumerate()
        .map(|(index, files)| {
            let mut spec = bp(index as u128 + 1);
            for (path, text) in *files {
                spec = spec.file(path, leak(text.clone()).as_bytes());
            }
            spec.open()
        })
        .collect();
    PackStack::<Behavior>::of(packs, options).unwrap()
}
