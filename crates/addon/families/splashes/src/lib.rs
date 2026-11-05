//! The splash texts of the title screen (`splashes.json`): the yellow texts, merged across the
//! stack's resource packs. One fixed-path file with its own reader in the client; it carries no
//! `format_version`.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod read;

use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Severity};
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::document::{
    Content, Document, JsonContent, Merge, Merged, Origin, Placed, Rejected, unversioned,
};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::json::kit::Written;
use bedrock_addon_core::json::{EncodeError, WriteStyle};
use bedrock_addon_core::load::{self, CollectionLoader, FileCopy, LoadContext, StackCx, Walk};
use bedrock_addon_core::pack_identity::{PackPath, PackSide};
use bedrock_addon_support::fixed_path::{Authored, file_finding, json_text};
use bedrock_addon_support::jsoncpp::read;
use indexmap::IndexMap;

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
    pub extra: UnknownMembers,
    /// The root as the author wrote it, kept when the fields cannot say it (a root that is no object).
    /// `to_json` writes it for as long as the fields are still the game's reading of it; a
    /// value that was changed, or built by hand, writes its fields. The entries the game
    /// skipped are in this root alone, so one edit of any field drops every one of them from
    /// the written file, not only those beside the edited member. `Debug` shows the root;
    /// `PartialEq` leaves it out: a loaded value that kept one equals a hand-built value with
    /// the same fields.
    pub written: Written,
}

/// One entry of `conditional` in a `splashes.json`: splashes shown only to some clients.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ConditionalSplashes {
    /// JSON `requires`: an object ([`SplashRequires`]). An entry without one is never used.
    pub requires: Option<Value>,
    /// JSON `splashes`: an array of strings.
    pub splashes: Option<Value>,
    /// The other members.
    pub extra: UnknownMembers,
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
    pub extra: UnknownMembers,
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

impl bedrock_addon_core::__private::Sealed for Splashes {}

/// Keyed by the splash text: `Merged::origin` names the highest pack that added it.
impl Merge for Splashes {
    type Key = String;
}

/// Reads one copy of `splashes.json`.
fn read_splashes(origin: Origin, bytes: &[u8]) -> Result<Document<Splashes>, Rejected> {
    // A copy that is not JSON is skipped; the other packs' copies still count. Vanilla says
    // nothing. The copy is refused, so the finding is an error.
    let mut root = match read(bytes) {
        Ok(root) => root,
        Err(message) => {
            let code = DiagnosticCode::MalformedInput {
                message: message.to_string(),
                text: None,
            };
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
    let splashes = Splashes::from_value(&root, &mut diagnostics);
    Ok(unversioned::document(splashes, origin, diagnostics))
}

impl Content for Splashes {
    const NAME: &'static str = "splashes";
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// Reads one `splashes.json`. Only a file that is not JSON is rejected, with an error (the
    /// game skips such a copy); a root that is no object is an empty copy that replaces the lower packs'
    /// splashes.
    fn parse(path: &PackPath, bytes: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
        read_splashes(unversioned::lone(path), bytes)
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        json_text(&self.to_value(), out, style)
    }
}

impl JsonContent for Splashes {}

impl Placed for Splashes {
    /// `splashes.json`.
    fn default_path(&self) -> PackPath {
        PackPath::from_static(SPLASHES_PATH)
    }
}

/// The pool a client picks its splash from, given every pack's copy top first (the layers of
/// `rp.splashes()`): each copy's [step](Splashes::apply), lowest pack first.
pub fn splash_pool(layers: &[Document<Splashes>], client: &SplashClient) -> Vec<String> {
    let mut pool = Vec::new();
    for layer in layers.iter().rev() {
        layer.apply(&mut pool, client);
    }
    pool
}

/// The `Combine` job of `rp.splashes()`: the pool of the full game (no platform, store or
/// treatment), each copy's step taken lowest pack first.
fn combine_splashes(parsed: Vec<FileCopy<Splashes>>, _: &StackCx<'_>) -> Merged<Splashes> {
    let client = SplashClient::default();
    let mut pool: Vec<String> = Vec::new();
    let mut origins: IndexMap<String, Origin> = IndexMap::new();
    let mut layers = Vec::new();
    let mut diagnostics = Diagnostics::default();
    // Whether a lower pack had a copy, read or not.
    let mut lower = false;
    let mut last: Option<Origin> = None;
    for (_, copy) in parsed {
        match copy {
            Ok(layer) => {
                diagnostics.extend(layer.diagnostics().iter().cloned());
                let (clears, added) = layer.step(&client);
                if clears {
                    pool.clear();
                    origins.clear();
                    if lower {
                        diagnostics.push(file_finding(
                            DiagnosticCode::SplashesReplaced,
                            Severity::Info,
                            &layer.origin().pack,
                            &layer.origin().path,
                        ));
                    }
                }
                for text in added {
                    origins.insert(text.clone(), layer.origin().clone());
                    pool.push(text);
                }
                last = Some(layer.origin().clone());
                layers.push(layer);
            }
            Err(rejected) => diagnostics.extend(rejected.diagnostics),
        }
        lower = true;
    }
    if let (true, Some(top)) = (pool.is_empty(), last) {
        // The stack has the file and the full game is left without a splash.
        diagnostics.push(file_finding(
            DiagnosticCode::NoSplashes,
            Severity::Info,
            &top.pack,
            &top.path,
        ));
    }
    layers.reverse();
    Merged::new(Splashes::of_texts(&pool), origins, layers, diagnostics)
}

/// The loader of `rp.splashes()`: `splashes.json` of every resource pack, lowest pack first.
#[doc(hidden)]
pub const LOADER: CollectionLoader<Merged<Splashes>, FileCopy<Splashes>> =
    load::merged::<Splashes, _>(Walk::exact_files(&[SPLASHES_PATH]), combine_splashes);

#[cfg(test)]
mod tests {
    use bedrock_addon_core::load::LoadOptions;

    use super::*;
    use bedrock_addon_core::test_cost::assert_linear_at;

    /// The pool of a stack is collected in time linear in its splashes: a hundred copies that
    /// each add their share.
    #[test]
    fn the_splash_pool_is_collected_in_linear_time() {
        let path = PackPath::from_static(SPLASHES_PATH);
        let build = |entries: usize| -> Vec<Document<Splashes>> {
            (0..100)
                .map(|copy| {
                    let texts: Vec<String> = (0..entries / 100)
                        .map(|i| format!("\"splash {copy} {i}\""))
                        .collect();
                    let text = format!(
                        r#"{{ "canMerge": true, "splashes": [{}] }}"#,
                        texts.join(",")
                    );
                    read_splashes(unversioned::lone(&path), text.as_bytes()).expect("a copy")
                })
                .collect()
        };
        let options = LoadOptions::default();
        assert_linear_at("combine_splashes", 25_000, build, |copies| {
            let parsed = copies
                .iter()
                .cloned()
                .map(|copy| (path.clone(), Ok(copy)))
                .collect();
            let merged = combine_splashes(parsed, &StackCx::new(&options, &[]));
            assert_eq!(merged.texts().len(), copies.len() * copies[0].texts().len());
        });
    }
}
