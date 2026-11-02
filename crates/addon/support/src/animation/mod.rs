//! What the `animation` and `animation_controller` families share: the schema walk both loaders
//! run, the particle, sound and event values both files hold, and the reading, Molang check,
//! validation, writing and loading both file kinds go through.

pub mod event;
pub mod load;
pub mod molang;
pub mod read;
pub mod types;
pub mod validate;
pub mod walk;
pub mod write;

#[cfg(test)]
mod tests;

pub use types::{EventString, ParticleEffect, SoundEffect};
