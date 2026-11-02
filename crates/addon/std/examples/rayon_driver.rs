#![allow(clippy::pedantic, missing_docs)]
//! The std driver with rayon, in three lines: a driver whose default strategy is a rayon pool,
//! a repository over two pack directories, and the world's two stacks loaded by that driver.
//!
//! What the example shows, and what it cannot: it asserts that the default strategy is
//! `Strategy::Rayon`, that the stacks hold the block the pack defines, and that a second
//! driver with `Strategy::Sequential` loads the same blocks and the same findings. It does
//! not assert that a job ran on a pool thread: the crates report no thread for a read or a
//! job (where the work runs is the driver's business, and the result does not depend on it),
//! so an example has nothing to look at.
//!
//! The example loads a world with the rayon strategy: it first writes two synthetic packs and
//! a world into a temporary directory, then loads them from there.
//!
//! Run with `cargo run -p bedrock_addon_std --example rayon_driver --features rayon`.

mod common;

use bedrock_addon::id::BlockId;
use bedrock_addon::load::{LoadOptions, StackOptions};
use bedrock_addon::pack::PackOrigin;
use bedrock_addon_std::{DirSource, Driver, Options, Strategy, read_world_packs};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = tempfile::tempdir()?;
    let config = common::write_server(server.path())?;

    // With the `rayon` feature the default strategy is a pool of rayon's default size, each
    // thread with `bedrock_addon::load::RECOMMENDED_JOB_STACK` (8 MiB) of stack.
    assert!(matches!(
        Options::default().strategy,
        Strategy::Rayon { threads: None }
    ));

    let mut driver = Driver::new(Options::default())?; // Strategy::Rayon, 8 MiB worker stacks
    let repository = driver.repository([
        DirSource::packs(&config.behavior_packs_directory, PackOrigin::World),
        DirSource::packs(&config.resource_packs_directory, PackOrigin::World),
    ])?;
    let stacks = driver.load_stacks(
        repository.stacks(
            &read_world_packs(&config.world_dir)?,
            LoadOptions::default(),
        ),
        StackOptions::default(),
    )?;

    let lamp = BlockId::from_static("demo:ruby_lamp");
    assert!(stacks.behavior.blocks().contains(&lamp), "{stacks:?}");
    assert_eq!(stacks.resource.packs().len(), 1);

    // The strategy says where the work runs, not what is loaded: one thread, in the order
    // the loader issues the work, gives the same blocks and the same findings.
    let mut sequential = Driver::new(Options::default().with_strategy(Strategy::Sequential))?;
    let repository = sequential.repository([
        DirSource::packs(&config.behavior_packs_directory, PackOrigin::World),
        DirSource::packs(&config.resource_packs_directory, PackOrigin::World),
    ])?;
    let in_order = sequential.load_stacks(
        repository.stacks(
            &read_world_packs(&config.world_dir)?,
            LoadOptions::default(),
        ),
        StackOptions::default(),
    )?;
    assert_eq!(
        format!("{:?}", stacks.behavior.blocks()),
        format!("{:?}", in_order.behavior.blocks())
    );
    assert_eq!(
        stacks.load_all().to_string(),
        in_order.load_all().to_string()
    );
    println!(
        "loaded {} block(s) from {} behaviour pack(s) with the rayon strategy, the same as in order",
        stacks.behavior.blocks().len(),
        stacks.behavior.packs().len()
    );
    Ok(())
}
