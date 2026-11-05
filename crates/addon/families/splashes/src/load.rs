//! Loading `splashes.json`: `Content` of the file, its walk, and the pool the stack collects
//! from the copies.

use bedrock_addon_core::load::{CollectionLoader, LoadContext, StackCx, WalkFile};
use bedrock_addon_core::pack_identity::{PackPath, PackSide};
use indexmap::IndexMap;

use super::{SPLASHES_PATH, SplashClient, Splashes};
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Severity};
use bedrock_addon_core::document::{
    Content, Document, JsonContent, Merge, Merged, Origin, Placed, Rejected, unversioned,
};
use bedrock_addon_core::json::{EncodeError, WriteStyle};
use bedrock_addon_support::fixed_path::{
    Authored, FileCopy, file_finding, json_text, malformed, root_file,
};
use bedrock_addon_support::jsoncpp::read;

impl bedrock_addon_core::__private::Sealed for Splashes {}

/// Keyed by the splash text: `Merged::origin` names the highest pack that added it.
impl Merge for Splashes {
    type Key = String;
}

/// Reads one copy of `splashes.json`.
fn read_splashes(origin: Origin, bytes: &[u8]) -> FileCopy<Splashes> {
    // A copy that is not JSON is skipped; the other packs' copies still count. Vanilla says
    // nothing. The copy is refused, so the finding is an error.
    let mut root = match read(bytes) {
        Ok(root) => root,
        Err(message) => {
            let code = malformed(message.to_string(), None);
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

fn is_splashes(path: &PackPath) -> bool {
    path.as_str() == SPLASHES_PATH
}

fn parse_splashes(file: &WalkFile, _: &LoadContext) -> FileCopy<Splashes> {
    read_splashes(unversioned::origin_of(file), &file.bytes)
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
    for copy in parsed {
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
pub const LOADER: CollectionLoader<Merged<Splashes>, FileCopy<Splashes>> = CollectionLoader {
    walk: root_file(is_splashes),
    parse: parse_splashes,
    combine: combine_splashes,
    needs: &[],
    then: None,
};

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
            let parsed = copies.iter().cloned().map(Ok).collect();
            let merged = combine_splashes(parsed, &StackCx::new(&options, &[]));
            assert_eq!(merged.texts().len(), copies.len() * copies[0].texts().len());
        });
    }
}
