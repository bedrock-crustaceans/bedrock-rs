#![allow(clippy::pedantic, missing_docs)]
//! The recipe family (`recipes/`, `minecraft:recipe_*`): every row of the fixtures,
//! The validation rules, the stack rules, Molang `data`, round trips and the deviations.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg(feature = "recipe")]

#[path = "recipe/common.rs"]
mod common;

#[path = "recipe/rows.rs"]
mod rows;

#[path = "recipe/ingredients.rs"]
mod ingredients;

#[path = "recipe/stack.rs"]
mod stack;

#[path = "recipe/molang.rs"]
mod molang;

#[path = "recipe/rules.rs"]
mod rules;

#[path = "recipe/round_trip.rs"]
mod round_trip;

#[path = "recipe/link.rs"]
mod link;

#[path = "recipe/wide_data.rs"]
mod wide_data;
