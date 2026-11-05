//! Loading an atlas: each pack's copy read leniently, every copy merged with `patch_json`
//! highest priority first, the merged document read once, and the image paths of the result
//! looked up in the stack.

use indexmap::IndexMap;

use super::atlas::{AtlasItem, AtlasKind, TextureAtlas, TextureElement, VariationEntry};
use super::atlas_read::read_atlas;
use super::patch::patch_json;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Phase, Severity,
};
use bedrock_addon_core::document::{Extra, Merge, Merged, Origin, RawLayer};
use bedrock_addon_core::id::AtlasKey;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::load::WalkFile;
use bedrock_addon_support::texture::document::origin_of;
use bedrock_addon_support::texture::files::{ATLAS_IMAGE, StackFiles};
use bedrock_addon_support::texture::jsoncpp::read_lenient;

impl bedrock_addon_core::__private::Sealed for TextureAtlas {}

/// Keyed by the tile's name: `Merged::origin` names the highest pack that wrote the tile.
impl Merge for TextureAtlas {
    type Key = AtlasKey;
}

/// One pack's copy of an atlas file, as the parse job leaves it.
#[derive(Debug)]
pub(crate) struct AtlasCopy {
    /// The copy, unless it is no JSON.
    pub(super) layer: Option<RawLayer>,
    /// What reading the copy found.
    pub(super) findings: Diagnostics,
    /// The warnings among `findings` of the numbers no double holds: the merge takes one
    /// back where a member speaks of the number itself.
    pub(super) clamp: Diagnostics,
}

/// Reads one pack's copy the way the game's JSON reader does. A copy that is no JSON is left
/// out of the merge; content after the root value and a root that is no object are kept, each
/// with an information of the crate's own (the game is silent about all three).
pub(crate) fn parse_copy(file: &WalkFile) -> AtlasCopy {
    let origin = origin_of(file);
    let mut findings = Diagnostics::default();
    // The copy is read with the numbers no double holds clamped, so neither its layer
    // nor the merged atlas keeps an infinity; the warning names this pack's file.
    let finite = bedrock_addon_core::document::FiniteText::of(&file.bytes);
    let layer = match read_lenient(finite.bytes()) {
        Ok(read) => {
            // Both are informations of the crate's own: the game has no text for either.
            if read.trailing {
                let code = DiagnosticCode::MalformedInput {
                    message: "content after the JSON value is ignored".to_owned(),
                    text: None,
                };
                findings.push(Diagnostic::info(code).in_file(&origin));
            }
            if !read.value.is_object() && !read.value.is_null() {
                let code = DiagnosticCode::InvalidValue { text: None };
                findings.push(Diagnostic::info(code).in_file(&origin));
            }
            Some(RawLayer::new(origin.clone(), read.value))
        }
        Err(message) => {
            // The game skips the copy when the JSON reader fails, without a line, so the crate
            // reports an information.
            let code = DiagnosticCode::MalformedInput {
                message: message.to_string(),
                text: None,
            };
            findings.push(Diagnostic::info(code).in_file(&origin));
            None
        }
    };
    let clamp = finite.settle_told(&mut findings, Some(&origin));
    AtlasCopy {
        layer,
        findings,
        clamp,
    }
}

/// An atlas no pack builds: no name and no tile.
fn unbuilt(kind: AtlasKind) -> TextureAtlas {
    TextureAtlas {
        kind,
        texture_name: String::new(),
        padding: None,
        num_mip_levels: None,
        default_leather_color: None,
        default_horse_leather_color: None,
        texture_data: None,
        ignored: Extra::default(),
        extra: Extra::default(),
    }
}

/// The merged atlas of the stack from its copies in walk order (lowest pack first), and
/// whether an atlas was built: no copy at all, or a merged document without `texture_name`,
/// builds none.
pub(crate) fn merge_atlas(
    kind: AtlasKind,
    copies: Vec<AtlasCopy>,
    files: &StackFiles,
) -> (Merged<TextureAtlas, RawLayer>, bool) {
    let mut diagnostics = Diagnostics::default();
    let mut layers = Vec::new();
    let mut clamp = Diagnostics::default();
    for copy in copies {
        diagnostics.extend(copy.findings);
        clamp.extend(copy.clamp);
        layers.extend(copy.layer);
    }
    // Top first, as `Merged::layers` promises and as the game folds them.
    layers.reverse();
    if layers.is_empty() {
        let merged = Merged::new(unbuilt(kind), IndexMap::new(), layers, diagnostics);
        return (merged, false);
    }
    // Each lower copy is the base and the merge so far is the patch, starting from `null`.
    let merged_json = layers.iter().fold(Value::NULL, |patch, layer: &RawLayer| {
        patch_json(&layer.json, patch)
    });
    let top = layers.first().map(|layer| layer.origin.clone());
    let mut found = Diagnostics::default();
    let atlas = read_atlas(kind, &merged_json, &mut found);
    let mut origins = IndexMap::new();
    if let Some(atlas) = &atlas {
        for (key, _) in atlas.iter() {
            if let Some(origin) = tile_origin(&layers, key.as_str()) {
                origins.insert(key.clone(), origin.clone());
            }
        }
    }
    // A finding of the merged document belongs to the pack whose tile it is about, or to the
    // top copy.
    for mut finding in found {
        let tile = tile_of(&finding.location.json_path);
        let origin = tile
            .and_then(|tile| origins.get(&AtlasKey::parse(tile)))
            .or_else(|| tile.and_then(|tile| tile_origin(&layers, tile)))
            .or(top.as_ref());
        if let Some(origin) = origin {
            finding.location.pack = Some(origin.pack.clone());
            finding.location.path = Some(origin.path.clone());
        }
        diagnostics.push(finding);
    }
    if let Some(atlas) = &atlas {
        for (key, item) in atlas.iter() {
            if let Some(origin) = origins.get(key) {
                check_images(key.as_str(), item, origin, files, &mut diagnostics);
            }
        }
    }
    // A float member that read a number no double holds has said so: once.
    bedrock_addon_core::json::drop_spoken_for(&mut diagnostics, &clamp);
    let built = atlas.is_some();
    let value = atlas.unwrap_or_else(|| unbuilt(kind));
    (Merged::new(value, origins, layers, diagnostics), built)
}

/// The tile a finding at `path` is about: the member below `texture_data`.
fn tile_of(path: &JsonPath) -> Option<&str> {
    use bedrock_addon_core::diagnostic::JsonPathSegment;
    let mut segments = path.segments();
    match (segments.next(), segments.next()) {
        (Some(JsonPathSegment::Key("texture_data")), Some(JsonPathSegment::Key(tile))) => {
            Some(tile)
        }
        _ => None,
    }
}

/// The highest copy that writes the tile `key` (a `null` writes nothing).
fn tile_origin<'a>(layers: &'a [RawLayer], key: &str) -> Option<&'a Origin> {
    layers.iter().find_map(|layer| {
        let data = layer.json.as_object()?.get("texture_data")?.as_object()?;
        data.get(key)
            .filter(|tile| !tile.is_null())
            .map(|_| &layer.origin)
    })
}

/// The place of a layer's image in the stack, as the game computes it: the
/// index of the highest pack that has it, `None` (vanilla's −1) for a layer without a path or
/// with one no pack has.
pub(super) fn stack_index(path: Option<&str>, files: &StackFiles) -> Option<usize> {
    files
        .first(path?, &ATLAS_IMAGE)
        .map(|found| found.stack_index)
}

/// The paths of the entries of a `variations` array.
pub(super) fn variation_paths(variations: &[VariationEntry]) -> Vec<Option<&str>> {
    variations
        .iter()
        .map(|entry| match entry {
            VariationEntry::Path(path) => Some(path.as_str()),
            VariationEntry::Layer(layer) => layer.path.as_deref(),
            VariationEntry::Invalid(_) => None,
        })
        .collect()
}

/// Which entries of a `variations` array vanilla keeps: those whose image is at the smallest
/// stack index, where an image in no pack (and a layer without a path) counts as −1 and so
/// beats every image that was found.
pub(super) fn kept_variations(paths: &[Option<&str>], files: &StackFiles) -> Vec<bool> {
    // −1 sorts before every index: `None` < `Some(_)`.
    let places: Vec<Option<usize>> = paths.iter().map(|path| stack_index(*path, files)).collect();
    let smallest = places.iter().min().copied().flatten();
    places.iter().map(|place| *place == smallest).collect()
}

/// The stack-stage findings of one tile: an image path no pack has (an information: vanilla's
/// loader says nothing and the tile falls back), and each variation the pack filter drops.
fn check_images(
    key: &str,
    item: &AtlasItem,
    origin: &Origin,
    files: &StackFiles,
    out: &mut Diagnostics,
) {
    let many = matches!(item.textures, super::atlas::AtlasTextures::Many(_));
    for (index, element) in item.elements().iter().enumerate() {
        let mut at = JsonPath::new();
        at.push("texture_data");
        at.push(key);
        at.push("textures");
        if many {
            at.push_index(index);
        }
        let missing = |path: Option<&str>, at: &JsonPath, out: &mut Diagnostics| {
            if path.is_some() && stack_index(path, files).is_none() {
                let code = DiagnosticCode::TextureNotFound { text: None };
                out.push(stage(code, Severity::Info, origin, at.clone()));
            }
        };
        match element {
            TextureElement::Path(path) => missing(Some(path), &at, out),
            TextureElement::Layer(layer) => {
                at.push("path");
                missing(layer.path.as_deref(), &at, out);
            }
            TextureElement::Variations { variations, .. } => {
                at.push("variations");
                let paths = variation_paths(variations);
                let kept = kept_variations(&paths, files);
                for (entry, (path, kept)) in paths.iter().zip(kept).enumerate() {
                    let mut at = at.clone();
                    at.push_index(entry);
                    if !kept {
                        let code = DiagnosticCode::DroppedField { text: None };
                        out.push(stage(code, Severity::Info, origin, at.clone()));
                    }
                    missing(*path, &at, out);
                }
            }
            TextureElement::Invalid(_) => {}
        }
    }
}

/// A stack-stage finding.
fn stage(code: DiagnosticCode, severity: Severity, origin: &Origin, at: JsonPath) -> Diagnostic {
    let mut finding = Diagnostic::error(code)
        .at(at)
        .severity(severity)
        .in_file(origin);
    finding.phase = Phase::Validate;
    finding
}
