//! Implementation details shared with `bedrock_addon`: the seal of the public traits. Hidden
//! from the documentation; nothing here is covered by SemVer.

/// The seal of `Content`, `Definition` and `Merge`: implemented by `bedrock_addon`'s
/// types, reachable so that crate can implement the traits, hidden so nobody else does.
pub trait Sealed {}
