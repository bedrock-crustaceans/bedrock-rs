//! The version chain of spawn groups, a single struct with no `Upgrade` step: `spawn_groups`
//! (`v1_8_0`). With one struct per chain, the struct as written is the newest one: the document
//! holds it, and nothing upgrades.

use super::SpawnGroup;

/// The `spawn_groups` family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[doc(hidden)]
pub struct GroupFamily;

bedrock_addon_core::version_chain! {
    family = GroupFamily;
    /// A spawn-group document.
    #[derive(Clone, Debug, PartialEq)]
    pub enum GroupAny {
        /// Format 1.8.0 and later.
        V1_8_0(SpawnGroup),
    }
}
