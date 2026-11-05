//! What the `animation` and `animation_controller` families share: the schema walk both loaders
//! run, the particle, sound and event values both files hold, and the reading, Molang check,
//! validation, writing and registry combine both file kinds go through.

pub mod event;
pub mod molang;
pub mod read;
pub mod registry;
pub mod types;
pub mod validate;
pub mod walk;
pub mod write;

#[cfg(test)]
mod tests;

pub use types::{EventString, ParticleEffect, SoundEffect};
