//! The recipe family (`recipes/`, `minecraft:recipe_*`): every row of the fixtures,
//! The validation rules, the stack rules, Molang `data`, round trips and the deviations.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[path = "recipe/texts.rs"]
mod texts;

#[path = "recipe/common.rs"]
mod common;

#[path = "recipe/rows.rs"]
mod rows;

#[path = "recipe/ingredients.rs"]
mod ingredients;

#[path = "recipe/molang.rs"]
mod molang;

#[path = "recipe/rules.rs"]
mod rules;

#[path = "recipe/round_trip.rs"]
mod round_trip;

#[path = "recipe/located.rs"]
mod located;
