//! The behavior-tree family (`behavior_trees/`, `minecraft:behavior_tree`), the tests that read
//! one document: the node catalogue of the schema, the validation rules on single files and
//! the pack and path every finding of a file read alone carries.

#[path = "behavior_tree/common.rs"]
mod common;

#[path = "behavior_tree/catalogue.rs"]
mod catalogue;

#[path = "behavior_tree/rules.rs"]
mod rules;

#[path = "behavior_tree/located.rs"]
mod located;
