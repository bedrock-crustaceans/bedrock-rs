//! The stack helper of the deferred-settings tests: a resource stack of packs built from texts.

use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{PackStack, Resource};

use super::pack_common::rp;

/// A text that lives as long as the test binary (the pack helper takes `'static` files).
pub fn leak(text: String) -> &'static str {
    Box::leak(text.into_boxed_str())
}

/// A resource stack of packs, top first; each pack is its `(path, text)` files.
pub fn stack(packs: &[&[(&str, String)]]) -> PackStack<Resource> {
    let packs: Vec<_> = packs
        .iter()
        .enumerate()
        .map(|(index, files)| {
            let mut spec = rp(index as u128 + 1);
            for (path, text) in *files {
                spec = spec.file(leak((*path).to_owned()), leak(text.clone()).as_bytes());
            }
            spec.open()
        })
        .collect();
    PackStack::<Resource>::of(packs, LoadOptions::default()).unwrap()
}
