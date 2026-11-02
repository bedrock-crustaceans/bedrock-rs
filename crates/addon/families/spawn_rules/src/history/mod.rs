//! The two version chains of spawn rules, each a single struct with no `Upgrade` step: `spawn`
//! (`v1_8_0`) and `spawn_simplified` (`v1_21_60`). With one struct per chain, the struct as
//! written is the newest one: the enums hold it, and nothing upgrades.

use super::{SimplifiedSpawnRules, StandardSpawnRules};

/// The `spawn` family: standard spawn rules.
#[derive(Clone, Copy, Debug)]
#[doc(hidden)]
pub struct SpawnFamily;

/// The `spawn_simplified` family: simplified spawn rules.
#[derive(Clone, Copy, Debug)]
#[doc(hidden)]
pub struct SimplifiedFamily;

bedrock_addon_core::version_chain! {
    family = SpawnFamily;
    /// A standard spawn-rules document.
    #[derive(Clone, Debug, PartialEq)]
    pub enum Any {
        /// Format 1.8.0 and later.
        V1_8_0(StandardSpawnRules),
    }
}

bedrock_addon_core::version_chain! {
    family = SimplifiedFamily;
    /// A simplified spawn-rules document.
    #[derive(Clone, Debug, PartialEq)]
    pub enum SimplifiedAny {
        /// Format 1.21.60 and later.
        V1_21_60(SimplifiedSpawnRules),
    }
}
