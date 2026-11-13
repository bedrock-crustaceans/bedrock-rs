//! The behavior-tree family (`behavior_trees/`, `minecraft:behavior_tree`): every row of
//! The fixtures, the validation rules, the node catalogue of
//! The schema, the stack rules, write-back and the deviations.

#[path = "behavior_tree/common.rs"]
mod common;

#[path = "behavior_tree/catalogue.rs"]
mod catalogue;

#[path = "behavior_tree/rules.rs"]
mod rules;

#[path = "behavior_tree/located.rs"]
mod located;
