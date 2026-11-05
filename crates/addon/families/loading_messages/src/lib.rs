//! The tips of the loading screens (`loading_messages.json`), read from the highest resource
//! pack's copy. One fixed-path file with its own reader in the client; it carries no
//! `format_version`.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod load;
mod read;

use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::Value;

#[doc(hidden)]
pub use load::LOADER;

/// The path of the loading-screen tips, in every resource pack.
pub const LOADING_MESSAGES_PATH: &str = "loading_messages.json";

/// One `loading_messages.json`: the tips of the loading screens.
///
/// JSON: `{"beginner_loading_messages": ["tips.game.1"], "mid_game_loading_messages": [..],
/// "late_game_loading_messages": [..], "realms_loading_messages": [..]}`; an element is a
/// text (a localisation key or the text itself) or `{"message": "..", "condition":
/// "$marketplace_not_disabled"}`. The game reads the highest pack's copy only. A list that is
/// not an array is read as the game iterates it: an object gives its member values in key
/// order, anything else nothing.
#[derive(Clone, Debug, Default, PartialEq)]
#[doc(alias = "loading_messages.json")]
pub struct LoadingMessages {
    /// JSON `loading_messages`: the older single list. When it has an element, it is the tips
    /// of every stage and the three stage lists are not read.
    pub loading_messages: Option<Vec<LoadingMessage>>,
    /// JSON `beginner_loading_messages`: the tips of a new player; its length is how many
    /// loads count as a beginner's.
    pub beginner_loading_messages: Option<Vec<LoadingMessage>>,
    /// JSON `mid_game_loading_messages`.
    pub mid_game_loading_messages: Option<Vec<LoadingMessage>>,
    /// JSON `late_game_loading_messages`.
    pub late_game_loading_messages: Option<Vec<LoadingMessage>>,
    /// JSON `realms_loading_messages`.
    pub realms_loading_messages: Option<Vec<LoadingMessage>>,
    /// JSON `addons_loading_messages`.
    pub addons_loading_messages: Option<Vec<LoadingMessage>>,
    /// JSON `creative_loading_messages`.
    pub creative_loading_messages: Option<Vec<LoadingMessage>>,
    /// JSON `editor_loading_messages`.
    pub editor_loading_messages: Option<Vec<LoadingMessage>>,
    /// JSON `store_progress_tooltips`: the store's progress texts, each element as written
    /// ([`tooltips`](Self::tooltips) gives the game's reading).
    pub store_progress_tooltips: Option<Vec<Value>>,
    /// The other members.
    pub extra: Extra,
    /// The root as the author wrote it, kept when the fields cannot say it (a root that is no
    /// object, a list that is no array, a `message` or `condition` that is no string).
    /// `to_json` writes it for as long as the fields are still the game's reading of it; a
    /// value that was changed, or built by hand, writes its fields. The entries the game
    /// skipped are in this root alone, so one edit of any field drops every one of them from
    /// the written file, not only those beside the edited member. `PartialEq` and `Debug`
    /// include the root: a loaded value that kept one differs from a hand-built value with
    /// the same fields.
    pub written: Option<Value>,
}

/// One element of a loading-message list.
#[derive(Clone, Debug, PartialEq)]
pub enum LoadingMessage {
    /// A text: a localisation key, or the text itself.
    Text(String),
    /// `{"message": "..", "condition": ".."}`: a text shown while a condition holds.
    Conditional(ConditionalMessage),
    /// Anything else, as written: the game skips it (an object without `message`, a number).
    Other(Value),
}

/// A loading message with a condition.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ConditionalMessage {
    /// JSON `message`: the text, read as the game reads any value as a string.
    pub message: String,
    /// JSON `condition`. The game knows one condition, [`MARKETPLACE_NOT_DISABLED`]; a
    /// message with any other is never shown.
    pub condition: Option<String>,
    /// The other members.
    pub extra: Extra,
}

/// The one condition a loading message may name.
pub const MARKETPLACE_NOT_DISABLED: &str = "$marketplace_not_disabled";

/// The lists of a `loading_messages.json`, by the member that holds each.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum MessageList {
    /// `loading_messages`.
    Legacy,
    /// `beginner_loading_messages`.
    Beginner,
    /// `mid_game_loading_messages`.
    MidGame,
    /// `late_game_loading_messages`.
    LateGame,
    /// `realms_loading_messages`.
    Realms,
    /// `addons_loading_messages`.
    Addons,
    /// `creative_loading_messages`.
    Creative,
    /// `editor_loading_messages`.
    Editor,
}
