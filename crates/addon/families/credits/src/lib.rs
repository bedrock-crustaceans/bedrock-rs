//! The credits (`credits/credits.json`), read from the game's own data and never from a pack.
//! One fixed-path file with its own reader in the client; it carries no `format_version`.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod read;

use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::json::kit::Written;

use crate::read::{EMPTY_FILE, NOT_JSON};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, VanillaText,
};
use bedrock_addon_core::document::unversioned::{document, lone, reject};
use bedrock_addon_core::document::{Content, Document, Merge, Merged, Origin, Placed, Rejected};
use bedrock_addon_core::json::build::{json_string, json_text};
use bedrock_addon_core::json::jsoncpp::read;
use bedrock_addon_core::json::kit::fixed_path::Authored;
use bedrock_addon_core::json::{EncodeError, WriteStyle};
use bedrock_addon_core::load::{
    self, AppDataPlace, CollectionLoader, FileCopy, LoadContext, StackCx, Walk,
};
use bedrock_addon_core::pack_identity::{PackPath, PackSide};
use indexmap::IndexMap;
pub use read::check_starter_seeds;

/// The path of the credits, in the game's own data (never in a pack).
pub const CREDITS_PATH: &str = "credits/credits.json";
/// The files of the game's own data that a resource pack carries in vain, credits aside:
/// a resource stack's report names each with an `IgnoredFile` info.
pub const APP_ONLY_FILES: [&str; 4] = [
    "CreativeStarterSeeds.txt",
    "StarterSeeds.txt",
    "manifest_publish.json",
    "texture_backwards_compatibility_mapping.json",
];

/// The credits (`credits/credits.json` of the game's own data): sections of disciplines of
/// titles of names.
///
/// JSON: `[{"section": "Mojang", "disciplines": [{"discipline": "Design", "titles":
/// [{"title": "Game Designer", "names": ["A. Person"]}]}]}]`. The game reads the file from its
/// own data and never from a pack; a pack's copy is ignored.
#[derive(Clone, Debug, Default, PartialEq)]
#[doc(alias = "credits.json")]
pub struct Credits {
    /// The sections the game shows, in order: the root's elements that are objects with a
    /// member.
    pub sections: Vec<CreditsSection>,
    /// The root as the author wrote it, kept when the fields cannot say it (a root that is no
    /// array, an element the game leaves out, a list that is no array, a heading or a name
    /// that is no string).
    /// `to_json` writes it for as long as the fields are still the game's reading of it; a
    /// value that was changed, or built by hand, writes its fields. The entries the game
    /// skipped are in this root alone, so one edit of any field drops every one of them from
    /// the written file, not only those beside the edited member. `Debug` shows the root;
    /// `PartialEq` leaves it out: a loaded value that kept one equals a hand-built value with
    /// the same fields.
    pub written: Written,
}

/// One section of the credits.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CreditsSection {
    /// JSON `section`: the heading.
    pub section: Option<String>,
    /// JSON `disciplines`.
    pub disciplines: Option<Vec<CreditsDiscipline>>,
    /// The other members.
    pub extra: UnknownMembers,
}

/// One discipline of a credits section.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CreditsDiscipline {
    /// JSON `discipline`: the heading.
    pub discipline: Option<String>,
    /// JSON `titles`.
    pub titles: Option<Vec<CreditsTitle>>,
    /// The other members.
    pub extra: UnknownMembers,
}

/// One title of a credits discipline, with the people who hold it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CreditsTitle {
    /// JSON `title`.
    pub title: Option<String>,
    /// JSON `names`, each read as the game reads any value as a string.
    pub names: Option<Vec<String>>,
    /// The other members.
    pub extra: UnknownMembers,
}

impl bedrock_addon_core::__private::Sealed for Credits {}

/// Keyed by the section heading; the game's data is the only origin.
impl Merge for Credits {
    type Key = String;
}

/// Reads one `credits/credits.json` with the game's own checks: an information each for a file
/// that loads, an error for the two that refuse it.
fn read_credits(origin: Origin, bytes: &[u8]) -> Result<Document<Credits>, Rejected> {
    let refuse = |message: String, text| {
        let code = DiagnosticCode::MalformedInput {
            message,
            text: Some(VanillaText::Fixed(text)),
        };
        Err(reject(
            origin.clone(),
            Diagnostic::error(code).at(JsonPath::new()),
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

/// The `Combine` job of `rp.credits()`: the game's own copy when the app data holds it, else
/// no credits; no pack's copy is read. A pack that carries the file in vain is the stack's
/// finding (`PackStack::report`), not this collection's.
// The game reads the file from its own data, never from a pack.
fn combine_credits(parsed: Vec<FileCopy<Credits>>, _: &StackCx<'_>) -> Merged<Credits> {
    let mut layers = Vec::new();
    let mut diagnostics = Diagnostics::default();
    let mut value = Credits::default();
    for (_, copy) in parsed {
        match copy {
            Ok(layer) => {
                diagnostics.extend(layer.diagnostics().iter().cloned());
                value = layer.value().clone();
                layers.push(layer);
            }
            Err(rejected) => diagnostics.append(rejected.diagnostics),
        }
    }
    Merged::new(value, IndexMap::new(), layers, diagnostics)
}

/// The walk of the credits: the game's own data only. The game never reads a pack's copy, so
/// the walk names no pack directory and no pack's file is read.
const WALK: Walk = Walk {
    directories: &[],
    ..Walk::exact_files(&[CREDITS_PATH]).app_data(&["credits"], AppDataPlace::After)
};

/// The loader of `rp.credits()`: `credits/credits.json` of the app data only.
#[doc(hidden)]
pub const LOADER: CollectionLoader<Merged<Credits>, FileCopy<Credits>> =
    load::merged::<Credits, _>(WALK, combine_credits);
