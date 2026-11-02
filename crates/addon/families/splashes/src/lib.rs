//! The splash texts of the title screen (`splashes.json`): the yellow texts, merged across the
//! stack's resource packs. One fixed-path file with its own reader in the client; it carries no
//! `format_version`.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod load;
mod read;

use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::Value;

#[doc(hidden)]
pub use load::load_splashes;
pub use load::splash_pool;

/// The path of the splash texts, in every resource pack.
pub const SPLASHES_PATH: &str = "splashes.json";

/// One `splashes.json`: the yellow texts of the title screen.
///
/// JSON: `{"canMerge": true, "splashes": ["Awesome!"], "conditional": [{"requires":
/// {"platforms": ["win10"]}, "splashes": ["On PC!"]}]}`. The game reads every pack's copy,
/// lowest pack first: a copy whose `canMerge` is not `true` first empties what the lower
/// packs collected, then every copy adds its `splashes` and the `splashes` of each
/// `conditional` entry whose requirements hold. The members are kept as written, because the
/// game converts whatever it finds; the methods give its reading.
///
/// On a stack ([`splash_pool`], `rp.splashes()`) the merged value holds the collected pool in
/// `splashes` and nothing else.
#[derive(Clone, Debug, Default, PartialEq)]
#[doc(alias = "splashes.json")]
pub struct Splashes {
    /// JSON `canMerge`. Only the boolean `true` keeps the lower packs' splashes.
    pub can_merge: Option<Value>,
    /// JSON `splashes`: an array of strings.
    pub splashes: Option<Value>,
    /// JSON `conditional`: an array of [`ConditionalSplashes`] objects.
    pub conditional: Option<Value>,
    /// The other members.
    pub extra: Extra,
    /// The root as the author wrote it, kept when the fields cannot say it (a root that is no object).
    /// `to_json` writes it for as long as the fields are still the game's reading of it; a
    /// value that was changed, or built by hand, writes its fields. The entries the game
    /// skipped are in this root alone, so one edit of any field drops every one of them from
    /// the written file, not only those beside the edited member. `PartialEq` and `Debug`
    /// include the root: a loaded value that kept one differs from a hand-built value with
    /// the same fields.
    pub written: Option<Value>,
}

/// One entry of `conditional` in a `splashes.json`: splashes shown only to some clients.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ConditionalSplashes {
    /// JSON `requires`: an object ([`SplashRequires`]). An entry without one is never used.
    pub requires: Option<Value>,
    /// JSON `splashes`: an array of strings.
    pub splashes: Option<Value>,
    /// The other members.
    pub extra: Extra,
}

/// `requires` of a conditional splash entry: every member given must hold.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SplashRequires {
    /// JSON `platforms`: holds when absent, not an array or empty, else when the client's
    /// platform is one of its strings.
    pub platforms: Option<Value>,
    /// JSON `treatments`: as `platforms`, against the client's flighting treatments.
    pub treatments: Option<Value>,
    /// JSON `stores`: as `platforms`, against the client's store.
    pub stores: Option<Value>,
    /// JSON `trialMode`: a boolean must equal whether the client is a trial; anything else
    /// holds.
    pub trial_mode: Option<Value>,
    /// The other members.
    pub extra: Extra,
}

/// What a client is, as far as conditional splashes ask. The default is what a lint assumes:
/// the full game, with no platform, store or treatment.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SplashClient {
    /// Whether the client is a trial. A trial collects only conditional splashes and never
    /// empties the pool.
    pub trial: bool,
    /// The platform names the client answers to.
    pub platforms: Vec<String>,
    /// The flighting treatments the client has.
    pub treatments: Vec<String>,
    /// The store names the client answers to.
    pub stores: Vec<String>,
}
