//! Helpers more than one family shares that are no vocabulary of `bedrock_addon_core` and have no
//! `shared/*` crate yet: a family imports the core, the shared crates and this crate and nothing
//! else (`bedrock_addon/tests/module_independence.rs`).

pub mod biome_json;
pub mod chain_file;
pub mod fixed_path;
#[cfg(test)]
mod helper_pins;
pub mod jsoncpp;
pub mod lossy;
pub mod puv;
pub mod scatter;
pub mod schema_walk;
pub mod shape;
pub mod signed_zero;
pub mod sound;
pub mod text;
pub mod texture;

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    fn assert_debug<T: Debug>() {}
    fn assert_hashable<T: Eq + std::hash::Hash>() {}
    fn assert_eq_type<T: Eq>() {}
    fn assert_comparable<T: Clone + PartialEq>() {}

    #[test]
    fn public_types_implement_debug() {
        assert_debug::<crate::shape::Shape>();
        assert_debug::<crate::shape::Member>();
        assert_debug::<crate::fixed_path::Object>();
        assert_debug::<crate::sound::report::Reporter<'static>>();
        assert_debug::<crate::text::lang_scan::End>();
        assert_debug::<crate::text::lang_scan::Kind>();
        assert_debug::<crate::text::lang_scan::Segment>();
    }

    #[test]
    fn data_types_are_comparable_and_hashable() {
        assert_hashable::<crate::schema_walk::Pattern>();
        assert_hashable::<crate::schema_walk::Group>();
        assert_hashable::<crate::schema_walk::Kind>();
        assert_hashable::<crate::sound::fork::Stored>();
        assert_eq_type::<crate::texture::files::PackFiles>();
        assert_eq_type::<crate::texture::files::StackFiles>();
        assert_eq_type::<crate::texture::files::Found<'static>>();
        assert_comparable::<crate::texture::jsoncpp::Lenient>();
        assert_comparable::<crate::texture::files::StackFiles>();
    }
}
