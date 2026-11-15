//! What the `animation` and `animation_controller` families share: the schema-tree reading both
//! loaders read with, the particle, sound and event values both files hold, and the reading,
//! Molang check, validation, writing and registry combine both file kinds go through.
//! A family composes these; the crate depends on `bedrock_addon_core` and the shared shapes.
//!
//! Both loaders read their documents as `Schema` types on core's derived read, as the game's
//! schema node tree reads them ([`walk::read`]: [`Reading::TREE`](bedrock_addon_core::versioning::Reading::TREE) and [`walk::PROFILE`]);
//! [`walk`] holds them, the time-key and axis patterns and the [`walk::atof`] both families read
//! keys with, and [`read`] the reads of the values both files hold. The bone keyframes are the
//! `animation` family's own.
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
