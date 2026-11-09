//! The legacy `biomes_client.json` family (`bedrock_addon::families::biomes_client`): the file
//! merged over the resource stack and read from the merged value.

#![cfg(feature = "biomes_client")]

#[path = "biomes_client/characterize.rs"]
mod characterize;
#[path = "biomes_client/common.rs"]
mod common;
#[path = "biomes_client/legacy.rs"]
mod legacy;
