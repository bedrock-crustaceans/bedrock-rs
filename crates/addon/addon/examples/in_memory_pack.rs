#![allow(clippy::pedantic, missing_docs)]
//! Open a pack from bytes already in memory, put it on a behaviour stack and look up a block.
//! No file is read: the crate is sans-IO, and `Pack::from_files` takes the bytes as they are.
//!
//! Run with `cargo run -p bedrock_addon --example in_memory_pack --no-default-features --features block`.

mod common;

use bedrock_addon::id::BlockId;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{Behavior, OpenOptions, Pack, PackStack};
use bedrock_addon::storage::PackPath;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pack = Pack::from_files(
        common::BEHAVIOR_FILES
            .iter()
            .map(|(path, bytes)| (PackPath::from_static(path), *bytes)),
        &OpenOptions::default(),
    )?;
    println!("opened {} ({})", pack.manifest().header.name, pack.id());

    let stack = PackStack::<Behavior>::of([pack], LoadOptions::default())?;
    let lamp = stack
        .blocks()
        .get(&BlockId::from_static("demo:ruby_lamp"))
        .expect("loaded");
    assert_eq!(
        lamp.origin().path,
        PackPath::from_static("blocks/ruby_lamp.json")
    );
    assert_eq!(lamp.permutations.len(), 1);
    println!(
        "{} is defined in {} and has {} permutation",
        lamp.description.identifier,
        lamp.origin(),
        lamp.permutations.len()
    );

    // The report is not empty, and nothing in it is an error: a pack without a `pack_icon.png`
    // is told so, and with no `MolangCheck` in the `LoadOptions` the permutation's condition
    // is kept as written and reported as not checked.
    for finding in stack.report().iter() {
        println!("  {finding}");
    }
    assert!(!stack.report().has_errors(), "{}", stack.report());
    Ok(())
}
