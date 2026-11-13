//! The behaviour stacks of the aim-assist stack tests.

use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{Behavior, PackStack};

use super::pack_common::bp;

/// A text that lives as long as the test binary (the pack helper takes `'static` files).
pub fn leak(text: String) -> &'static str {
    Box::leak(text.into_boxed_str())
}

/// A behaviour stack of packs, top first; each pack is its `(path, text)` files.
pub fn stack(packs: &[&[(&'static str, String)]]) -> PackStack<Behavior> {
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
    PackStack::<Behavior>::of(packs, LoadOptions::default()).unwrap()
}
