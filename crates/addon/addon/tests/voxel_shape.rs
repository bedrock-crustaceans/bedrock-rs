//! The voxel-shape family (`shapes/`, `minecraft:voxel_shape`): its rules, the chain, round trips
//! and the stack accessor.

#![cfg(feature = "voxel_shape")]

#[path = "voxel_shape/link.rs"]
mod link;

#[path = "pack/common.rs"]
mod pack_common;

#[path = "voxel_shape/stack.rs"]
mod stack;

#[path = "common/in_files.rs"]
mod in_files;
