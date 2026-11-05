//! Loading `loading_messages.json`: `Content` of the file, its walk, and the highest pack's
//! copy the stack keeps.

use bedrock_addon_core::load::{CollectionLoader, LoadContext, StackCx, WalkFile};
use bedrock_addon_core::pack_identity::{PackPath, PackSide};
use indexmap::IndexMap;

use super::{LOADING_MESSAGES_PATH, LoadingMessages, MessageList};
use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath, Severity};
use bedrock_addon_core::document::{
    Content, Document, JsonContent, Merge, Merged, Origin, Placed, Rejected,
};
use bedrock_addon_core::json::{EncodeError, WriteStyle};
use bedrock_addon_support::fixed_path::{
    Authored, FileCopy, document, finding, json_text, lone, malformed, origin, reject, root_file,
};
use bedrock_addon_support::jsoncpp::read;

impl bedrock_addon_core::__private::Sealed for LoadingMessages {}

/// Vanilla's assert for a file that is not JSON.
const LOADING_NOT_JSON: &str = "Failed to read loading messages file";

/// Keyed by the member name of a list: `Merged::origin` names the pack whose file is read.
impl Merge for LoadingMessages {
    type Key = String;
}

/// Reads one `loading_messages.json`.
fn read_loading(origin: Origin, bytes: &[u8]) -> FileCopy<LoadingMessages> {
    let mut root = match read(bytes) {
        Ok(root) => root,
        Err(message) => {
            // No tips at all.
            let code = malformed(message.to_string(), Some(LOADING_NOT_JSON));
            return Err(reject(
                origin,
                finding(code, Severity::Error, JsonPath::new()),
            ));
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
    Ok(document(messages, origin, diagnostics))
}

impl Content for LoadingMessages {
    const NAME: &'static str = "loading messages";
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// Reads one `loading_messages.json`. A file that is not JSON is rejected: the game then
    /// has no tips.
    fn parse(path: &PackPath, bytes: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
        read_loading(lone(path), bytes)
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

fn is_loading(path: &PackPath) -> bool {
    path.as_str() == LOADING_MESSAGES_PATH
}

fn parse_loading(file: &WalkFile, _: &LoadContext) -> FileCopy<LoadingMessages> {
    read_loading(
        origin(file.pack.clone(), file.side, &file.path),
        &file.bytes,
    )
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
    for (place, copy) in parsed.into_iter().enumerate() {
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
pub const fn load_loading_messages()
-> CollectionLoader<Merged<LoadingMessages>, FileCopy<LoadingMessages>> {
    CollectionLoader {
        walk: root_file(is_loading),
        parse: parse_loading,
        combine: combine_loading,
        needs: &[],
        then: None,
    }
}
