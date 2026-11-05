//! The biome family (`bedrock_addon::families::biome`): every row of its fixtures, the steps, the
//! chain, the validation rules, round trips, Molang and the stack's registry.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[path = "biome/steps.rs"]
mod steps;

#[path = "biome/texts.rs"]
mod texts;

#[path = "biome/scalars.rs"]
mod scalars;
