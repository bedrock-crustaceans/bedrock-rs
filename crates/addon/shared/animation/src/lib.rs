//! What the `animation` and `animation_controller` families share: the schema-walk rules both
//! loaders read with, the particle, sound and event values both files hold, and the reading,
//! Molang check, validation, writing and registry combine both file kinds go through.
//! A family composes these; the crate depends on `bedrock_addon_core` only.
//!
//! Both loaders walk their schema trees on core's node-tree walker (`json::node_tree`) under
//! [`walk::RULES`]; [`walk`] holds those rules, the time-key and axis patterns and the
//! [`walk::atof`] both families read keys with. The bone keyframes are the `animation` family's
//! own.
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

    #[test]
    fn public_types_implement_debug() {
        assert_debug::<crate::write::Object>();
        assert_debug::<crate::read::Parsed<()>>();
        assert_debug::<crate::molang::MolangRun<'static>>();
    }
}
