//! The locale overlay `texts/<code>/`: a path rule, not a document.

use bedrock_addon_core::pack_identity::PackPath;

use super::Language;

/// Where one pack is asked for the file `path` by a client whose language is `language`, in
/// lookup order; the first path the pack has is the file it gives.
///
/// The selected subpack comes first (its own overlay, then its root), then the pack's overlay
/// `texts/<code>/<path>`, then the pack root. A file the overlay lacks falls through to the
/// root copy of the same pack, never to a lower pack. `subpack` is the folder of the selected
/// subpack below the pack root (`subpacks/hd`), if any. A code that cannot name a folder has
/// no overlay.
// `ResourcePack::getResource(path, out, subpackIndex)`: the selected subpack (recursively,
// with its own overlay first), then this pack's overlay, then the pack root.
// The overlay is `texts` joined with the chosen language's code as
// `languages.json` lists it (`ResourcePack::setLocale`).
#[must_use]
pub fn locale_overlay_paths(
    language: &Language,
    path: &PackPath,
    subpack: Option<&PackPath>,
) -> Vec<PackPath> {
    let overlay = overlay_folder(language);
    let mut paths = Vec::with_capacity(4);
    for root in subpack.into_iter().chain([&PackPath::ROOT]) {
        if let Some(overlay) = &overlay {
            paths.push(root.join(overlay).join(path));
        }
        paths.push(root.join(path));
    }
    paths
}

/// `texts/<code>`, when the code is one folder name.
fn overlay_folder(language: &Language) -> Option<PackPath> {
    let code = PackPath::new(language.as_str()).ok()?;
    (code.as_str() == language.as_str() && code.segments().count() == 1)
        .then(|| PackPath::from_static("texts").join(&code))
}
