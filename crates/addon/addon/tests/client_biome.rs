//! The client-biome family (`bedrock_addon::families::client_biome`): every row of its
//! fixtures, the steps, the chain, the validation rules, round trips and the stack's registry.

#![cfg(feature = "client_biome")]

#[path = "../../families/client_biome/tests/client_biome/common.rs"]
mod common;
#[path = "client_biome/stack.rs"]
mod stack;
#[path = "client_biome/stacks.rs"]
mod stacks;
