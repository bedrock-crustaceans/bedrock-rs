//! The rules of the stack the texture sets are held to: each layer's image in the set's own
//! pack, and the capability lints of the packs that ship sets.

use indexmap::IndexMap;

use super::codes;
use super::types::TextureSet;
use super::{locate, payload_at};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Code, Diagnostic, Diagnostics, JsonPath, Location, Phase, Severity,
};
use bedrock_addon_core::document::{Document, Rejected};
use bedrock_addon_core::load::StackPack;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::version::SemVersion;
use bedrock_addon_shared_shapes::texture::files::{
    PackFiles, StackFiles, TEXTURE_SET, VERBATIM_OR_IMAGE,
};

/// One pack's copy of a texture set: the document, or why the client shows the missing
/// texture instead.
pub type SetCopy = Result<Document<TextureSet>, Rejected>;

/// The texture sets of one pack, by the texture path each sits beside (the file's key without
/// `.texture_set.json`).
pub type PackSets = IndexMap<String, SetCopy>;

/// The texture path a texture-set file at `key` belongs to.
pub(super) fn texture_path(key: &str) -> Option<&str> {
    key.strip_suffix(TEXTURE_SET)
}

/// The path a layer named `name` of the texture set beside `texture` is loaded from, before
/// the extensions: the set's folder, a `/`, and the name as written.
// The value is joined to the set's folder as written and is not normalised.
pub fn layer_path(texture: &str, name: &str) -> String {
    match texture.rsplit_once('/') {
        Some((folder, _)) => format!("{folder}/{name}"),
        None => name.to_owned(),
    }
}

/// The layer images of one pack: a set with a layer whose image is not in its own pack is not
/// loaded. The whole texture is replaced by the missing texture, and an image of the same name
/// in another pack is not used.
pub(super) fn check_layer_images(sets: &mut PackSets, stack_index: usize, files: &StackFiles) {
    for (texture, copy) in sets.iter_mut() {
        let Ok(document) = copy else {
            continue;
        };
        let mut missing = Diagnostics::default();
        for (layer, name) in document.textures() {
            let path = layer_path(texture, name.as_str());
            if files
                .in_pack(stack_index, &path, &VERBATIM_OR_IMAGE)
                .is_none()
            {
                let at = payload_at().child(layer.key());
                let code = Code::from(&codes::SET_NO_IMAGE);
                // The game logs this at the content log's information level; the set is still
                // replaced by the missing texture.
                missing.push(Diagnostic::info(code).at(at).phase(Phase::Validate));
            }
        }
        if missing.is_empty() {
            continue;
        }
        let origin = document.origin().clone();
        let mut diagnostics = document.diagnostics().clone();
        locate(&mut missing, &origin);
        for finding in missing {
            diagnostics.push(finding);
        }
        *copy = Err(Rejected::new(origin, diagnostics));
    }
}

/// The capability that makes the client use the layers besides `color` in the deferred
/// renderer.
const PBR: &str = "pbr";
/// The capability of the ray-traced renderer.
const RAYTRACED: &str = "raytraced";
/// From this `min_engine_version` on, a resource pack without `pbr` turns Vibrant Visuals off
/// for the whole stack.
const PBR_REQUIRED_FROM: SemVersion = SemVersion::new(1, 21, 110);

/// One pack as the capability lints see it.
pub(super) struct LintPack<'a> {
    /// The stack entry.
    pub(super) entry: &'a StackPack,
    /// The pack's files.
    pub(super) files: &'a PackFiles,
}

/// The pack lints of the texture sets: one for each pack that turns Vibrant Visuals off, and one
/// for each pack that ships texture sets in a stack where no pack declares a capability that
/// uses them.
pub(super) fn capability_lints(packs: &[LintPack<'_>], d: &mut Diagnostics) {
    // The capabilities are the manifest's as the pack was opened with it: a
    // `manifest.json` of the active subpack is no manifest.
    let declares = |pack: &LintPack<'_>, name: &str| {
        let mut declared = pack.entry.capabilities.iter();
        declared.any(|capability| &**capability == name)
    };
    let any_renderer = packs
        .iter()
        .any(|pack| declares(pack, PBR) || declares(pack, RAYTRACED));
    for pack in packs {
        let at_manifest = |code: Code| {
            let location = Location::new(
                Some(pack.entry.pack.clone()),
                Some(PackPath::from_static("manifest.json")),
                JsonPath::new(),
                None,
            );
            Diagnostic::new(code, Severity::Info, Phase::Validate, location)
        };
        let context = &pack.entry.context;
        let from_required = context
            .min_engine_version
            .as_ref()
            .is_some_and(|version| version.version >= PBR_REQUIRED_FROM);
        if !context.is_base_game_pack && from_required && !declares(pack, PBR) {
            d.push(at_manifest(Code::from(
                &core_codes::VIBRANT_VISUALS_DISABLED_BY_PACK,
            )));
        }
        let ships_sets = pack
            .files
            .files
            .keys()
            .any(|key| key.ends_with(TEXTURE_SET));
        if ships_sets && !any_renderer {
            // The game has no sentence for this lint; the text is core's.
            d.push(at_manifest(Code::from(&core_codes::CAPABILITY_HINT)));
        }
    }
}
