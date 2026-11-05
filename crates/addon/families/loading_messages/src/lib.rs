//! The tips of the loading screens (`loading_messages.json`), read from the highest resource
//! pack's copy. One fixed-path file with its own reader in the client; it carries no
//! `format_version`.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod read;

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, JsonPath};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::document::{
    Content, Document, JsonContent, Merge, Merged, Origin, Placed, Rejected, unversioned,
};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::json::{EncodeError, WriteStyle};
use bedrock_addon_core::load::{self, CollectionLoader, FileCopy, LoadContext, StackCx, Walk};
use bedrock_addon_core::pack_identity::{PackPath, PackSide};
use bedrock_addon_support::fixed_path::{Authored, json_text, malformed};
use bedrock_addon_support::jsoncpp::read;
use indexmap::IndexMap;

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

impl bedrock_addon_core::__private::Sealed for LoadingMessages {}

/// Vanilla's assert for a file that is not JSON.
const LOADING_NOT_JSON: &str = "Failed to read loading messages file";

/// Keyed by the member name of a list: `Merged::origin` names the pack whose file is read.
impl Merge for LoadingMessages {
    type Key = String;
}

/// Reads one `loading_messages.json`.
fn read_loading(origin: Origin, bytes: &[u8]) -> Result<Document<LoadingMessages>, Rejected> {
    let mut root = match read(bytes) {
        Ok(root) => root,
        Err(message) => {
            // No tips at all.
            let code = malformed(message.to_string(), Some(LOADING_NOT_JSON));
            return Err(unversioned::reject(origin, Diagnostic::error(code)));
        }
    };
    let mut diagnostics = Diagnostics::default();
    // A number no double holds is read as the largest finite one, with a warning.
    bedrock_addon_core::json::clamp_non_finite_warned(
        &mut root,
        None,
        &JsonPath::new(),
        &mut diagnostics,
    );
    let messages = LoadingMessages::from_value(&root, &mut diagnostics);
    Ok(unversioned::document(messages, origin, diagnostics))
}

impl Content for LoadingMessages {
    const NAME: &'static str = "loading messages";
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// Reads one `loading_messages.json`. A file that is not JSON is rejected: the game then
    /// has no tips.
    fn parse(path: &PackPath, bytes: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
        read_loading(unversioned::lone(path), bytes)
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        json_text(&self.to_value(), out, style)
    }
}

impl JsonContent for LoadingMessages {}

impl Placed for LoadingMessages {
    /// `loading_messages.json`.
    fn default_path(&self) -> PackPath {
        PackPath::from_static(LOADING_MESSAGES_PATH)
    }
}

/// The `Combine` job of `rp.loading_messages()`: the highest pack's file, whole. A highest
/// copy that is not JSON leaves the stack without tips; the lower packs' copies are layers
/// the game never reads, and only the highest copy's findings are the stack's.
/// The game reads the first pack's file from the top that has it.
fn combine_loading(
    parsed: Vec<FileCopy<LoadingMessages>>,
    _: &StackCx<'_>,
) -> Merged<LoadingMessages> {
    let mut layers = Vec::new();
    let mut diagnostics = Diagnostics::default();
    let mut origins = IndexMap::new();
    let mut value = LoadingMessages::default();
    let count = parsed.len();
    for (place, (_, copy)) in parsed.into_iter().enumerate() {
        let top = place + 1 == count;
        match copy {
            Ok(layer) => {
                if top {
                    diagnostics.extend(layer.diagnostics().iter().cloned());
                    value = layer.value().clone();
                    for list in MessageList::ALL {
                        if layer.list(list).is_some() {
                            origins.insert(list.key().to_owned(), layer.origin().clone());
                        }
                    }
                    if layer.store_progress_tooltips.is_some() {
                        let key = "store_progress_tooltips".to_owned();
                        origins.insert(key, layer.origin().clone());
                    }
                }
                layers.push(layer);
            }
            Err(rejected) if top => diagnostics.extend(rejected.diagnostics),
            Err(_) => {}
        }
    }
    layers.reverse();
    Merged::new(value, origins, layers, diagnostics)
}

/// The loader of `rp.loading_messages()`: `loading_messages.json` of every resource pack.
#[doc(hidden)]
pub const LOADER: CollectionLoader<Merged<LoadingMessages>, FileCopy<LoadingMessages>> =
    load::merged::<LoadingMessages, _>(
        Walk::exact_files(&[LOADING_MESSAGES_PATH]),
        combine_loading,
    );
