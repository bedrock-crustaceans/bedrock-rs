//! Load a world's two stacks from disk with the std `Driver`, then register its custom blocks
//! with their Molang permutation conditions compiled, as a server does at start-up.
//!
//! This is the start-up path of a server. The example first writes two synthetic packs and a
//! world into a temporary directory, then loads them from there; the registry it fills is a
//! stand-in for a server's own.
//!
//! Run with `cargo run -p bedrock_addon_std --example load_stacks`.

mod common;

use std::sync::Arc;

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::families::block::StateValues;
use bedrock_addon::families::block::component::LightEmission;
use bedrock_addon::load::{LoadOptions, StackOptions};
use bedrock_addon::molang::molangx::compile::Expr;
use bedrock_addon::molang::{
    CompileCache, CompileOptions, Molang, MolangCheck, MolangSource, MolangVersion,
};
use bedrock_addon::pack::{PackOrigin, StackSet};
use bedrock_addon::shared::StateValue;
use bedrock_addon_std::{DirSource, Driver, LoadError, Options, ReadError, read_world_packs};

use common::Config;

/// Why the server does not start.
#[derive(Debug, thiserror::Error)]
enum StartupError {
    #[error(transparent)]
    Load(#[from] LoadError),
    #[error(transparent)]
    Read(#[from] ReadError),
    #[error("the world's pack selection has errors")]
    PackSelection,
}

fn load_stacks(config: &Config, molang: &Arc<MolangCheck>) -> Result<StackSet, StartupError> {
    let mut driver = Driver::new(Options::default())?;
    let repo = driver.repository([
        DirSource::packs(&config.behavior_packs_directory, PackOrigin::World),
        DirSource::packs(&config.resource_packs_directory, PackOrigin::World),
    ])?;

    // With a `MolangCheck` every Molang field is compiled while the stacks load.
    let options = LoadOptions {
        molang: Some(Arc::clone(molang)),
        ..LoadOptions::default()
    };
    let stacks = driver.load_stacks(
        repo.stacks(&read_world_packs(&config.world_dir)?, options),
        StackOptions::default(),
    )?;
    let report = stacks.load_all(); // already loaded: returns the cached findings; includes stacks.selection()
    for finding in report.at_least(Severity::Warning) {
        eprintln!("content: {finding}");
    }
    if stacks.selection().has_errors() {
        return Err(StartupError::PackSelection);
    }
    Ok(stacks)
}

/// When a permutation applies.
#[derive(Debug)]
enum PermutationCondition {
    Constant(bool),
    Molang { expr: Expr, source: MolangSource },
}

impl PermutationCondition {
    /// The condition in words.
    fn describe(&self) -> String {
        match self {
            PermutationCondition::Constant(value) => format!("always {value}"),
            PermutationCondition::Molang { expr, source } => match expr.as_constant() {
                Some(value) => format!("`{}`, the constant {value}", source.as_str()),
                None => format!("`{}`, evaluated per block state", source.as_str()),
            },
        }
    }
}

/// What the stand-in registry keeps of one block.
#[derive(Debug)]
struct BlockDefinition {
    id: String,
    pack: String,
    states: Vec<(String, Vec<StateValue>)>,
    /// Each permutation's condition and the light level it sets, if any.
    permutations: Vec<(PermutationCondition, Option<u8>)>,
}

/// The Molang side of the stand-in server: one cache and the options block conditions compile
/// with.
struct MolangRuntime {
    cache: Arc<CompileCache>,
    block_condition_options: CompileOptions,
}

fn register_blocks(stacks: &StackSet, registry: &mut Vec<BlockDefinition>, molang: &MolangRuntime) {
    for (id, block) in stacks.behavior.blocks() {
        if id.is_vanilla() {
            continue; // a server keeps the vanilla blocks in a layer of its own
        }
        let states = block
            .states_with_traits()
            .iter()
            .map(|(name, values)| (name.clone(), StateValues::values(values)))
            .collect();
        let mut permutations = Vec::new();
        for permutation in &block.permutations {
            let condition = match &permutation.condition {
                Molang::Expr(source) => {
                    let compiled = molang
                        .cache
                        .compile_source(source, &molang.block_condition_options);
                    // `expr_or_zero` is `None` only for array / resource expressions, which no block condition can use
                    compiled
                        .expr_or_zero()
                        .map_or(PermutationCondition::Constant(false), |expr| {
                            PermutationCondition::Molang {
                                expr: expr.clone(),
                                source: source.clone(),
                            }
                        })
                }
                Molang::Const(value) => PermutationCondition::Constant(*value != 0.0),
                Molang::Bool(value) => PermutationCondition::Constant(*value),
                _ => PermutationCondition::Constant(false), // `Molang` is non_exhaustive
            };
            let light = permutation.components.get::<LightEmission>().map(|l| l.0.0);
            permutations.push((condition, light));
        }
        registry.push(BlockDefinition {
            id: id.as_str().to_owned(),
            pack: block.origin().pack.name().to_owned(),
            states,
            permutations,
        });
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = tempfile::tempdir()?;
    let config = common::write_server(server.path())?;

    let check = Arc::new(MolangCheck::new());
    let stacks = load_stacks(&config, &check)?;
    println!(
        "loaded {} behaviour pack(s) and {} resource pack(s)",
        stacks.behavior.packs().len(),
        stacks.resource.packs().len()
    );

    let molang = MolangRuntime {
        cache: Arc::clone(check.cache()), // the conditions were compiled once already, by the load
        block_condition_options: CompileOptions::server(MolangVersion::LATEST),
    };
    let mut registry = Vec::new();
    register_blocks(&stacks, &mut registry, &molang);
    for block in &registry {
        println!(
            "registered {} from {}: {} state(s), {} permutation(s)",
            block.id,
            block.pack,
            block.states.len(),
            block.permutations.len()
        );
        for (condition, light) in &block.permutations {
            println!("  while {}: light {light:?}", condition.describe());
        }
    }

    assert_eq!(registry.len(), 1, "{registry:?}");
    let lamp = &registry[0];
    assert_eq!(
        (lamp.id.as_str(), lamp.pack.as_str()),
        ("demo:ruby_lamp", "Ruby behaviour")
    );
    assert_eq!(
        lamp.states,
        [(
            "demo:lit".to_owned(),
            vec![StateValue::Bool(false), StateValue::Bool(true)]
        )]
    );
    let [(PermutationCondition::Molang { source, .. }, light)] = lamp.permutations.as_slice()
    else {
        panic!(
            "one permutation with a compiled condition: {:?}",
            lamp.permutations
        );
    };
    assert_eq!(
        (source.as_str(), *light),
        ("q.block_state('demo:lit')", Some(15))
    );
    Ok(())
}
