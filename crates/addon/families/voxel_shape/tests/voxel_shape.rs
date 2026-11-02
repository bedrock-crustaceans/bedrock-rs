//! The voxel-shape family (`shapes/`, `minecraft:voxel_shape`): its rules, the chain, round trips
//! and the stack accessor.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[path = "voxel_shape/common.rs"]
mod common;

#[path = "voxel_shape/constraints.rs"]
mod constraints;

#[path = "voxel_shape/corners.rs"]
mod corners;

#[path = "voxel_shape/envelope.rs"]
mod envelope;

#[path = "voxel_shape/geometry.rs"]
mod geometry;

#[path = "voxel_shape/namespace.rs"]
mod namespace;

#[path = "voxel_shape/round_trip.rs"]
mod round_trip;

#[path = "voxel_shape/rows.rs"]
mod rows;

#[path = "voxel_shape/structure.rs"]
mod structure;

#[path = "voxel_shape/validate.rs"]
mod validate;

#[path = "voxel_shape/versions.rs"]
mod versions;
