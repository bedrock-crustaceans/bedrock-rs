//! Loading `credits/credits.json`: `Content` of the file, its walk over the game's own data, and
//! the copy the stack keeps.

use bedrock_addon_core::load::{
    AppDataPlace, CollectionLoader, ListDepth, LoadContext, StackCx, Walk, WalkFile, WalkOrder,
};
use bedrock_addon_core::pack_identity::{PackPath, PackSide};
use indexmap::IndexMap;

use super::read::{EMPTY_FILE, NOT_JSON};
use super::{CREDITS_PATH, Credits};
use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath, Severity};
use bedrock_addon_core::document::{Content, Document, Merge, Merged, Origin, Placed, Rejected};
use bedrock_addon_core::json::{EncodeError, WriteStyle};
use bedrock_addon_support::fixed_path::{
    Authored, FileCopy, document, finding, json_string, json_text, lone, malformed, origin, reject,
};
use bedrock_addon_support::jsoncpp::read;

impl bedrock_addon_core::__private::Sealed for Credits {}

/// Keyed by the section heading; the game's data is the only origin.
impl Merge for Credits {
    type Key = String;
}

/// Reads one `credits/credits.json` with the game's own checks: an information each for a file
/// that loads, an error for the two that refuse it.
fn read_credits(origin: Origin, bytes: &[u8]) -> FileCopy<Credits> {
    let refuse = |message: String, text| {
        let code = malformed(message, Some(text));
        Err(reject(
            origin.clone(),
            finding(code, Severity::Error, JsonPath::new()),
        ))
    };
    if bytes.is_empty() {
        return refuse("the file is empty".to_owned(), EMPTY_FILE);
    }
    let mut root = match read(bytes) {
        Ok(root) => root,
        Err(message) => return refuse(message.to_string(), NOT_JSON),
    };
    let mut diagnostics = Diagnostics::default();
    // A number no double holds is read as the largest finite one, with a warning.
    bedrock_addon_core::json::clamp_non_finite_warned(
        &mut root,
        None,
        &JsonPath::new(),
        &mut diagnostics,
    );
    let credits = Credits::from_value(&root, &mut diagnostics);
    Ok(document(credits, origin, diagnostics))
}

impl Content for Credits {
    const NAME: &'static str = "credits";
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// Reads one `credits/credits.json`, wherever it lies, with the checks the game runs on
    /// its own file, each an information. An empty file and one that is not JSON are rejected, with
    /// an error.
    fn parse(path: &PackPath, bytes: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
        read_credits(lone(path), bytes)
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        json_text(&self.to_value(), out, style)
    }

    fn to_json(&self) -> Result<String, EncodeError> {
        json_string(&self.to_value())
    }
}

impl Placed for Credits {
    /// `credits/credits.json`.
    fn default_path(&self) -> PackPath {
        PackPath::from_static(CREDITS_PATH)
    }
}

fn is_credits(path: &PackPath) -> bool {
    path.as_str() == CREDITS_PATH
}

fn parse_credits(file: &WalkFile, _: &LoadContext) -> FileCopy<Credits> {
    read_credits(
        origin(file.pack.clone(), file.side, &file.path),
        &file.bytes,
    )
}

/// The `Combine` job of `rp.credits()`: the game's own copy when the app data holds it, else
/// no credits; no pack's copy is read. A pack that carries the file in vain is the stack's
/// finding (`PackStack::report`), not this collection's.
// The game reads the file from its own data, never from a pack.
fn combine_credits(parsed: Vec<FileCopy<Credits>>, _: &StackCx<'_>) -> Merged<Credits> {
    let mut layers = Vec::new();
    let mut diagnostics = Diagnostics::default();
    let mut value = Credits::default();
    for copy in parsed {
        match copy {
            Ok(layer) => {
                diagnostics.extend(layer.diagnostics().iter().cloned());
                value = layer.value().clone();
                layers.push(layer);
            }
            Err(rejected) => diagnostics.extend(rejected.diagnostics),
        }
    }
    Merged::new(value, IndexMap::new(), layers, diagnostics)
}

/// The loader of `rp.credits()`: `credits/credits.json` of the app data only.
pub const fn load_credits() -> CollectionLoader<Merged<Credits>, FileCopy<Credits>> {
    CollectionLoader {
        walk: Walk {
            directories: &[],
            depth: ListDepth::Children,
            accept: is_credits,
            order: WalkOrder::BottomUp,
            directory_major: false,
            app_data: &["credits"],
            app_data_place: AppDataPlace::After,
            in_pack: None,
            shadowed: None,
            read_error: None,
        },
        parse: parse_credits,
        combine: combine_credits,
        needs: &[],
        then: None,
    }
}
