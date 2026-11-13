//! The recipe family (`recipes/`, `minecraft:recipe_*`): every row of the fixtures,
//! The validation rules, the stack rules, Molang `data`, round trips and the deviations.

#![cfg(feature = "recipe")]

#[path = "../../families/recipe/tests/recipe/common.rs"]
mod common;

#[path = "recipe/stack.rs"]
mod stack;

#[path = "recipe/stacks.rs"]
mod stacks;

#[path = "recipe/link.rs"]
mod link;

#[path = "recipe/wide_data.rs"]
mod wide_data;
