//! What the `animation` and `animation_controller` families share: the schema walk both loaders
//! run, the particle, sound and event values both files hold, and the reading, Molang check,
//! validation, writing and registry combine both file kinds go through.
//! A family composes these; the crate depends on `bedrock_addon_core` only.
//!
//! [`walk`] is one of the schema-walk forks the one node-tree walker of core is to replace; it
//! moved here as it is, and the time-key grammar and [`walk::atof`] both families read keys with
//! stay in this crate when it goes. The bone keyframes are the `animation` family's own.
//!
//! The Molang check ([`molang::run`]) compiles only with this crate's `molang` feature, which
//! each family forwards from its own.

pub mod event;
pub mod molang;
pub mod read;
pub mod registry;
pub mod types;
pub mod validate;
pub mod walk;
pub mod write;

#[cfg(test)]
mod helper_pins;
#[cfg(test)]
mod tests;

pub use types::{EventString, ParticleEffect, SoundEffect};

#[cfg(test)]
mod type_tests {
    use std::fmt::Debug;

    fn assert_debug<T: Debug>() {}
    fn assert_hashable<T: Eq + std::hash::Hash>() {}

    #[test]
    fn public_types_implement_debug() {
        assert_debug::<crate::write::Object>();
        assert_debug::<crate::read::Parsed<()>>();
        assert_debug::<crate::molang::MolangRun<'static>>();
    }

    #[test]
    fn data_types_are_comparable_and_hashable() {
        assert_hashable::<crate::walk::Pattern>();
        assert_hashable::<crate::walk::Group>();
        assert_hashable::<crate::walk::Kind>();
    }
}
