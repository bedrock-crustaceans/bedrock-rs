//! Helpers more than one family shares that are no vocabulary of `bedrock_addon_core` and have no
//! `shared/*` crate yet: a family imports the core, the shared crates and this crate and nothing
//! else (`bedrock_addon/tests/module_independence.rs`).

pub mod biome_json;
pub mod chain_file;
pub mod fixed_path;
#[cfg(test)]
mod helper_pins;
pub mod jsoncpp;
pub mod lossy;
pub mod puv;
pub mod scatter;
pub mod shape;
pub mod signed_zero;
pub mod sound;
pub mod texture;
