//! The voxel-shape family (`shapes/`, `minecraft:voxel_shape`): its rules, the chain, round trips
//! and the stack accessor.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg(feature = "voxel_shape")]

#[path = "voxel_shape/link.rs"]
mod link;

#[path = "pack/common.rs"]
mod pack_common;

#[path = "voxel_shape/stack.rs"]
mod stack;
