//! Each flipbook is held to the terrain atlas (its tile, the tile's index, the
//! index's variant) and to its image: that a pack has it, and from the image's header that it
//! has a frame and every frame the flipbook plays. The load reads no image, so the rule reads
//! the bytes the stack holds; an image that is not resident is told once by the pass and held
//! to nothing.
//!
//! The flipbooks, the terrain atlas and the texture files are three families; this rule is
//! where they meet.

use std::collections::HashMap;

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::document::{Merged, RawLayer};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};

use super::super::{Findings, LinkCx, Target, kinds};
use crate::diagnostic::{
    Code, Diagnostic, DiagnosticDetail, Diagnostics, JsonPath, Phase, Severity,
};
use crate::document::Origin;
use crate::families::flipbooks::codes;
use crate::families::flipbooks::{Flipbook, FlipbookKey, Flipbooks};
use crate::families::texture_atlas::{AtlasItem, TextureAtlas};
use crate::families::texture_file::{ImageFormat, ImageInfo};
use crate::id::{AtlasKey, Kind, kind};
use bedrock_addon_shared_shapes::texture::files::{Found, IMAGE, StackFiles};

/// What the link pass tells the checker about an image file: the header of the file at a path
/// of a pack, or `None` when the pass did not read it or it is no image. The checker asks only
/// for the images the flipbooks name.
type ImageHeaders<'a> = &'a dyn Fn(&PackRef, &PackPath) -> Option<ImageInfo>;

/// The flipbooks of the resource stack: the checker, with the header of each image a flipbook
/// names read from the bytes the stack holds. A tile no pack defines and the host says the game
/// has is the game's: its indices are not known here and nothing is told. A tile the atlas
/// lacks is told by the view the pass has of the terrain atlas
/// ([`LinkCx::terrain_atlas_view`]): an information while the only atlas is an add-on's own.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    let flipbooks = cx.resource().flipbooks();
    if flipbooks.entries.is_empty() {
        return;
    }
    let files = cx.resource().texture_files().files();
    // The header of each image by its pack's place and its path; `None` for an image that is
    // not resident or does not read.
    let mut headers: HashMap<(Option<usize>, &str), Option<ImageInfo>> = HashMap::new();
    for (pack, path) in flipbook_images(flipbooks, files) {
        headers
            .entry((pack.index(), path.as_str()))
            .or_insert_with(|| {
                let format = ImageFormat::from_path(path.as_str())?;
                let bytes = cx.bytes(PackSide::Resource, pack, path, out)?;
                ImageInfo::read(format, &bytes)
            });
    }
    let atlas = cx.resource().texture_atlases().terrain();
    let found = check(flipbooks, atlas, files, &|pack, path| {
        headers
            .get(&(pack.index(), path.as_str()))
            .copied()
            .flatten()
    });
    // The tile check follows the view of the terrain atlas, as a block's tile does: with
    // an atlas of an add-on's own alone a tile it lacks may be the game's, an information.
    // The index and the variant of a tile the atlas has are the stack's own.
    let view = cx.terrain_atlas_view(out);
    for mut finding in found {
        if names_a_tile_of_the_game(cx, &finding) {
            continue;
        }
        if lacks_a_tile(&finding) {
            if !view.judges() {
                continue;
            }
            finding.severity = view.severity(finding.severity);
        }
        out.push(finding);
    }
}

/// Whether `finding` is the checker's "the terrain atlas lacks this tile".
fn lacks_a_tile(finding: &Diagnostic) -> bool {
    // By legacy name: the variant and the definitions that were it.
    matches!(finding.code.name(), "UnresolvedReference")
        && matches!(
            finding.detail.as_deref(),
            Some(DiagnosticDetail::Reference { kind, .. }) if *kind == kinds::ATLAS_KEY
        )
}

/// Whether `finding` is the checker's "the terrain atlas lacks this tile" for a tile the host
/// answers for.
fn names_a_tile_of_the_game(cx: &LinkCx<'_>, finding: &Diagnostic) -> bool {
    // By legacy name: the variant and the definitions that were it.
    if !matches!(finding.code.name(), "UnresolvedReference") {
        return false;
    }
    match finding.detail.as_deref() {
        Some(DiagnosticDetail::Reference { kind, id, .. }) if *kind == kinds::ATLAS_KEY => {
            matches!(
                cx.terrain_atlas_key(&AtlasKey::parse(id.as_str())),
                Target::BuiltIn
            )
        }
        _ => false,
    }
}

/// The findings of the flipbooks against the terrain atlas and the image files. Loading reads
/// no image, so the frame rules look at the headers `headers` gives:
/// [`flipbook_images`] lists the files to read, `ImageInfo::read` reads one.
fn check(
    flipbooks: &Merged<Flipbooks, RawLayer>,
    atlas: Option<&TextureAtlas>,
    files: &StackFiles,
    headers: ImageHeaders<'_>,
) -> Diagnostics {
    let mut out = Diagnostics::default();
    for (index, flipbook) in flipbooks.entries.iter().enumerate() {
        let key = flipbook.merge_key();
        let Some(origin) = flipbooks.origin(&key) else {
            continue;
        };
        let mut check = Check {
            out: &mut out,
            origin,
            index,
        };
        if let Some(atlas) = atlas {
            check.tile(flipbook, &key, atlas);
        }
        check.image(flipbook, files, headers);
    }
    out
}

/// The image file of each flipbook that has one, as the pack and the path in it: what the rule
/// reads the headers of before it calls [`check`].
fn flipbook_images<'a>(
    flipbooks: &'a Flipbooks,
    files: &'a StackFiles,
) -> Vec<(&'a PackRef, &'a PackPath)> {
    flipbooks
        .entries
        .iter()
        .filter_map(|flipbook| {
            let found = flipbook_image(flipbook, files)?;
            let pack = files.packs.get(found.stack_index)?;
            Some((&pack.pack, &found.file.path))
        })
        .collect()
}

/// The image the client loads for a flipbook: a path that already ends in an image extension
/// is loaded as written.
fn flipbook_image<'a>(flipbook: &Flipbook, files: &'a StackFiles) -> Option<Found<'a>> {
    let path = flipbook.flipbook_texture();
    if ImageFormat::from_path(path).is_some() {
        files.first(path, &[""])
    } else {
        files.first(path, &IMAGE)
    }
}

/// The findings of one flipbook entry.
#[derive(Debug)]
struct Check<'a> {
    out: &'a mut Diagnostics,
    origin: &'a Origin,
    index: usize,
}

impl Check<'_> {
    /// A finding at member `key` of the entry, or at element `element` of that member.
    fn report(
        &mut self,
        key: &str,
        element: Option<usize>,
        code: Code,
        severity: Severity,
    ) -> Diagnostic {
        let mut at = JsonPath::new();
        at.push_index(self.index);
        at.push(key);
        if let Some(element) = element {
            at.push_index(element);
        }
        Diagnostic::error(code)
            .severity(severity)
            .at(at)
            .in_file(self.origin)
            .phase(Phase::Link)
    }

    /// Reports a finding at member `key` of the entry.
    fn push(&mut self, key: &str, code: Code, severity: Severity) {
        let finding = self.report(key, None, code, severity);
        self.out.push(finding);
    }

    /// The tile, its index and its variant.
    fn tile(&mut self, flipbook: &Flipbook, key: &FlipbookKey, atlas: &TextureAtlas) {
        let FlipbookKey { tile, index, .. } = key;
        let Some(item) = atlas.get(tile) else {
            // The game falls back to the atlas's `missing` tile and says so only when the
            // index is outside that tile's UV sets ("Texture %s is missing from atlas
            // textures", at the error level); inside it the missing tile is animated without
            // a word, which the crate reports as an information. An atlas without a `missing`
            // tile uses the built-in one, a single UV set.
            let missing = atlas.get("missing").map_or(1, AtlasItem::frame_count);
            let logged = usize::try_from(*index).map_or(true, |place| place >= missing);
            let (code, severity) = if logged {
                (codes::TILE_MISSING.with([tile.clone()]), Severity::Error)
            } else {
                (
                    Code::from(&core_codes::UNRESOLVED_REFERENCE),
                    Severity::Info,
                )
            };
            let mut finding = self.report("atlas_tile", None, code, severity);
            finding.detail = Some(Box::new(DiagnosticDetail::Reference {
                kind: <kind::AtlasTexture as Kind>::NAME,
                id: tile.clone(),
                hint: None,
            }));
            self.out.push(finding);
            return;
        };
        let count = item.frame_count();
        let Some(place) = usize::try_from(*index).ok().filter(|place| *place < count) else {
            // The game skips the entry.
            let args = [tile.clone(), index.to_string(), count.to_string()];
            let code = codes::BAD_ATLAS_INDEX.with(args);
            self.push("atlas_index", code, Severity::Error);
            return;
        };
        if let Some(variant) = flipbook.atlas_tile_variant {
            let variants = item.layers(place).len();
            if usize::try_from(variant).map_or(true, |variant| variant >= variants) {
                // The game reads the variant unchecked and uses variant 0 for one below 0 or
                // beyond the UV sets without a word: the crate reports an information.
                let code = Code::from(&core_codes::CLAMPED_VALUE);
                self.push("atlas_tile_variant", code, Severity::Info);
            }
        }
    }

    /// The image and its frames. The game has debug assertions only here, so each finding is an
    /// information carrying the assertion's text.
    fn image(&mut self, flipbook: &Flipbook, files: &StackFiles, headers: ImageHeaders<'_>) {
        let path = flipbook.flipbook_texture();
        let Some(found) = flipbook_image(flipbook, files) else {
            let code = codes::NO_METADATA.with([path.to_owned()]);
            self.push("flipbook_texture", code, Severity::Info);
            return;
        };
        // The checks need the image's size: an image whose header was not read, or does not
        // decode, has none.
        let info = files
            .packs
            .get(found.stack_index)
            .and_then(|pack| headers(&pack.pack, &found.file.path));
        let Some(info) = info else {
            return;
        };
        let frames = info.height.checked_div(info.width).unwrap_or(0);
        if frames < 1 {
            let code = Code::from(&codes::NO_FRAMES);
            self.push("flipbook_texture", code, Severity::Info);
        }
        for (place, frame) in flipbook.frames.iter().flatten().enumerate() {
            // The game's assertion reads the element before it is assigned, so it never fires,
            // and it has no lower bound; this check is the crate's own.
            if u32::try_from(*frame).map_or(true, |frame| frame >= frames) {
                let code = Code::from(&codes::FRAME_OUT_OF_RANGE);
                let finding = self.report("frames", Some(place), code, Severity::Info);
                self.out.push(finding);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bedrock_addon_core::diagnostic::Severity;
    use bedrock_addon_core::load::LoadOptions;

    #[cfg(feature = "texture_set")]
    use super::super::texture_set_texture_file::run as layer_images;
    use super::run as flipbooks;
    use crate::pack::link::tests::support::{File, pack, set_of};
    use crate::pack::link::{LinkOptions, link_with_rules};

    const FLIPBOOKS: &str = "textures/flipbook_textures.json";
    const ATLAS: &str = "textures/terrain_texture.json";
    const LAVA: &str = "textures/blocks/lava.png";
    const SET: &str = "textures/blocks/lava.texture_set.json";
    const TERRAIN: &str = r#"{ "texture_name": "atlas.terrain", "texture_data": {
        "lava": { "textures": "textures/blocks/lava" } } }"#;

    #[cfg(feature = "texture_set")]
    const RULES: [fn(&crate::pack::link::LinkCx<'_>, &mut crate::pack::link::Findings); 2] =
        [flipbooks, layer_images];
    #[cfg(not(feature = "texture_set"))]
    const RULES: [fn(&crate::pack::link::LinkCx<'_>, &mut crate::pack::link::Findings); 1] =
        [flipbooks];

    /// What the two rules say of a world whose one resource pack has `files`: code, severity,
    /// file and JSON path.
    fn told(files: &[File<'_>]) -> Vec<(&'static str, Severity, String, String)> {
        let set = set_of(
            Vec::new(),
            vec![pack(2, "resources", files)],
            &LoadOptions::default(),
        );
        link_with_rules(&set, LinkOptions::default(), &RULES)
            .iter()
            .map(|finding| {
                (
                    finding.code.id(),
                    finding.severity,
                    finding
                        .location
                        .path
                        .as_ref()
                        .map_or_else(String::new, |path| path.as_str().to_owned()),
                    finding.location.json_path.to_string(),
                )
            })
            .collect()
    }

    fn not_loaded(file: &str) -> (&'static str, Severity, String, String) {
        (
            "core/file_not_loaded",
            Severity::Info,
            file.to_owned(),
            String::new(),
        )
    }

    // Without the header the frame rules have no size: a frame far outside any image is not
    // told, the tile rules still are.
    #[test]
    fn a_flipbook_image_that_is_not_resident_is_told_once_and_only_the_frame_rules_wait() {
        let flipbooks = r#"[
            { "flipbook_texture": "textures/blocks/lava", "atlas_tile": "lava", "frames": [99] },
            { "flipbook_texture": "textures/blocks/lava", "atlas_tile": "lava", "atlas_index": 3 },
            { "flipbook_texture": "textures/blocks/gone", "atlas_tile": "ghost" } ]"#;
        assert_eq!(
            told(&[
                (FLIPBOOKS, Some(flipbooks)),
                (ATLAS, Some(TERRAIN)),
                (LAVA, None),
            ]),
            [
                not_loaded(LAVA),
                (
                    "flipbooks/bad_atlas_index",
                    Severity::Error,
                    FLIPBOOKS.to_owned(),
                    "1/atlas_index".to_owned()
                ),
                // The atlas is this pack's own and the pack is not the game's: a tile it
                // lacks may be the game's, an information (the tile check follows the view
                // of the terrain atlas).
                (
                    "core/unresolved_reference",
                    Severity::Info,
                    FLIPBOOKS.to_owned(),
                    "2/atlas_tile".to_owned()
                ),
                (
                    "flipbooks/no_metadata",
                    Severity::Info,
                    FLIPBOOKS.to_owned(),
                    "2/flipbook_texture".to_owned()
                ),
            ]
        );
    }

    #[cfg(feature = "texture_set")]
    #[test]
    fn a_layer_image_that_is_not_resident_is_told_once_and_held_to_nothing() {
        let layers =
            r#"{ "format_version": "1.16.100", "minecraft:texture_set": { "color": "lava" } }"#;
        assert_eq!(
            told(&[(SET, Some(layers)), (LAVA, None)]),
            [not_loaded(LAVA)]
        );
        // The image a flipbook and a texture set share is one file.
        let flipbooks =
            r#"[ { "flipbook_texture": "textures/blocks/lava", "atlas_tile": "lava" } ]"#;
        assert_eq!(
            told(&[
                (FLIPBOOKS, Some(flipbooks)),
                (ATLAS, Some(TERRAIN)),
                (SET, Some(layers)),
                (LAVA, None),
            ]),
            [not_loaded(LAVA)]
        );
    }

    /// The flipbook rule looks up a tile, an image file and the image's header for each
    /// entry: `n` entries over `n` tiles and `n` images against four times as many.
    #[test]
    fn n_flipbooks_over_n_tiles_and_n_images_cost_n() {
        fn input(n: usize) -> crate::pack::StackSet {
            let leak = |text: String| -> &'static str { Box::leak(text.into_boxed_str()) };
            let mut entries = Vec::with_capacity(n);
            let mut tiles = Vec::with_capacity(n);
            let mut files: Vec<File<'static>> = Vec::with_capacity(n + 2);
            for i in 0..n {
                // Every other entry names a tile the atlas lacks.
                let tile = if i % 2 == 0 { "t" } else { "none" };
                entries.push(format!(
                    r#"{{ "flipbook_texture": "textures/blocks/f{i}", "atlas_tile": "{tile}{i}" }}"#
                ));
                tiles.push(format!(
                    r#""t{i}": {{ "textures": "textures/blocks/f{i}" }}"#
                ));
                files.push((leak(format!("textures/blocks/f{i}.png")), None));
            }
            let flipbooks = leak(format!("[{}]", entries.join(",")));
            let atlas = leak(format!(
                r#"{{ "texture_name": "atlas.terrain", "texture_data": {{ {} }} }}"#,
                tiles.join(",")
            ));
            files.push((FLIPBOOKS, Some(flipbooks)));
            files.push((ATLAS, Some(atlas)));
            let set = set_of(
                Vec::new(),
                vec![pack(2, "resources", &files)],
                &LoadOptions::default(),
            );
            // The load is not the rule's cost.
            assert_eq!(set.resource.flipbooks().entries.len(), n);
            set
        }
        crate::test_cost::assert_linear_at(
            "link: n flipbooks over n tiles and n images",
            4_000,
            input,
            |set: &crate::pack::StackSet| {
                let found = link_with_rules(set, LinkOptions::default(), &[flipbooks]);
                let entries = set.resource.flipbooks().entries.len();
                // One `FileNotLoaded` per image, one missing tile for every other entry.
                assert_eq!(found.len(), entries + entries / 2);
                found
            },
        );
    }
}

/// The checker over the three collections a stack of in-line packs loads.
#[cfg(test)]
mod checker {
    use bedrock_addon_core::load::{
        CollectionLoader, ListedFile, LoadContext, LoadOptions, StackCx, StackPack, WalkFile,
    };
    use bedrock_addon_core::pack_identity::{
        PackIdVersion, PackOrigin, PackPath, PackRef, PackSide, PackType,
    };
    use bedrock_addon_core::version::SemVersion;

    use super::{check, flipbook_images};
    use crate::diagnostic::Severity::{self, Error, Info, Warning};
    use crate::families::texture_file::{ImageFormat, ImageInfo};
    use crate::families::{flipbooks, texture_atlas, texture_file};
    use bedrock_addon_shared_shapes::texture::image::png_header;

    const FLIPBOOKS: &str = "textures/flipbook_textures.json";
    const TERRAIN: &str = "textures/terrain_texture.json";

    /// What `loader` makes of a stack of in-line packs, the first on top: the loader's own
    /// parse and combine, over the files a walk of such a stack hands them (lowest pack first,
    /// each pack's files bytewise). Every file is listed by the stack; only the files of the
    /// walk are parsed, so an image's bytes never reach a loader.
    fn load<C, P>(loader: CollectionLoader<C, P>, packs: &[&[(&str, &[u8])]]) -> C {
        let refs: Vec<PackRef> = (0..packs.len())
            .map(|index| {
                let uuid = uuid::Uuid::from_u128(index as u128 + 1);
                let id = PackIdVersion::new(uuid, SemVersion::new(1, 0, 0), PackType::Resource);
                PackRef::new(index, id, format!("pack {}", index + 1))
            })
            .collect();
        let context = LoadContext::detached();
        let mut parsed = Vec::new();
        for (index, files) in packs.iter().enumerate().rev() {
            let mut files: Vec<_> = files.to_vec();
            files.sort_by_key(|(path, _)| *path);
            for (path, bytes) in files {
                let path = PackPath::new(path).unwrap();
                if !loader.walk.accept.takes(&path) {
                    continue;
                }
                let file = WalkFile {
                    pack: refs[index].clone(),
                    side: PackSide::Resource,
                    path: path.clone(),
                    key: path,
                    bytes: bytes.to_vec().into(),
                };
                parsed.push((loader.parse)(&file, &context));
            }
        }
        let stack: Vec<StackPack> = refs
            .into_iter()
            .zip(packs)
            .map(|(pack, files)| {
                let mut listed: Vec<ListedFile> = files
                    .iter()
                    .map(|(path, _)| PackPath::new(path).unwrap())
                    .map(|path| ListedFile {
                        key: path.clone(),
                        path,
                    })
                    .collect();
                listed.sort_by(|a, b| a.key.cmp(&b.key));
                StackPack::new(pack, PackOrigin::User, context.clone()).with_files(listed)
            })
            .collect();
        let options = LoadOptions::default();
        (loader.combine)(parsed, &StackCx::new(&options, &stack))
    }

    type Found = (&'static str, Severity, String, String);

    /// The findings of the link stage of a stack of in-line packs.
    fn linked(packs: &[&[(&str, &[u8])]]) -> Vec<Found> {
        let atlases = load(texture_atlas::LOADER, packs);
        let flipbooks = load(flipbooks::LOADER, packs);
        let files = load(texture_file::LOADER, packs);
        // Loading alone reports none of them.
        let loaded: Vec<_> = atlases
            .diagnostics()
            .chain(flipbooks.diagnostics())
            .collect();
        assert_eq!(
            loaded.iter().filter(|f| f.severity > Info).count(),
            0,
            "{loaded:#?}"
        );
        // The link pass reads the headers of the images the flipbooks name, and of no other.
        let named = flipbook_images(&flipbooks, files.files());
        let headers = |pack: &PackRef, path: &PackPath| {
            assert!(
                named.iter().any(|(p, at)| *p == pack && *at == path),
                "{path} is asked for and not listed"
            );
            let (name, bytes) = packs[pack.index()?]
                .iter()
                .find(|(name, _)| *name == path.as_str())?;
            ImageInfo::read(ImageFormat::from_path(name)?, bytes)
        };
        check(&flipbooks, atlases.terrain(), files.files(), &headers)
            .iter()
            .map(|f| {
                assert_eq!(f.phase, crate::diagnostic::Phase::Link);
                assert_eq!(f.location.path.as_ref().unwrap().as_str(), FLIPBOOKS);
                (
                    f.code.id(),
                    f.severity,
                    f.location.json_path.to_string(),
                    f.to_string(),
                )
            })
            .collect()
    }

    fn found(code: &'static str, severity: Severity, at: &str, text: &str) -> Found {
        let level = match severity {
            Error => "error",
            Warning => "warning",
            _ => "info",
        };
        let line = format!("[{level}] pack 1 | {FLIPBOOKS} | {at} | {text}");
        (code, severity, at.to_owned(), line)
    }

    const DIRT: &[u8] = br#"{ "texture_name": "atlas.terrain", "texture_data": {
        "dirt": { "textures": ["textures/s", { "variations": ["textures/s", "textures/s", "textures/s"] }] },
        "missing": { "textures": "textures/s" } } }"#;
    const FRAME: &str = "frame index out of range!";

    /// One pack with the atlas `DIRT`, a 16x32 strip `textures/s.png` and `flipbooks`.
    fn with_dirt(flipbooks: &[u8]) -> Vec<Found> {
        let strip = png_header(16, 32, 6);
        linked(&[&[
            (TERRAIN, DIRT),
            (FLIPBOOKS, flipbooks),
            ("textures/s.png", &strip),
        ]])
    }

    #[test]
    fn a_flipbook_of_a_tile_index_and_variant_the_atlas_has_is_no_finding() {
        assert_eq!(
            with_dirt(
                br#"[ { "atlas_tile": "dirt", "flipbook_texture": "textures/s", "frames": [0, 1] },
                      { "atlas_tile": "dirt", "atlas_index": 1, "atlas_tile_variant": 2, "flipbook_texture": "textures/s" },
                      { "atlas_tile": "dirt", "atlas_index": 1, "atlas_tile_variant": 0, "flipbook_texture": "textures/s" } ]"#
            ),
            []
        );
    }

    #[test]
    fn an_unknown_tile_is_an_unresolved_reference_at_any_index() {
        // The game falls back to the atlas's `missing` tile and logs an error only when
        // `atlas_index` is outside that tile's UV-set count.
        // Silent inside the `missing` tile (one element here): an information; outside it the
        // game's content-log error.
        let silent = "unresolved reference: atlas key 'ghost'";
        let text = "Texture ghost is missing from atlas textures: atlas key 'ghost'";
        assert_eq!(
            with_dirt(
                br#"[ { "atlas_tile": "ghost", "flipbook_texture": "textures/s" },
                      { "atlas_tile": "ghost", "atlas_index": 1, "flipbook_texture": "textures/s" } ]"#
            ),
            [
                found("core/unresolved_reference", Info, "0/atlas_tile", silent),
                found("flipbooks/tile_missing", Error, "1/atlas_tile", text),
            ]
        );
    }

    #[test]
    fn an_atlas_index_outside_the_tile_is_an_error() {
        let text = |index: i32| {
            format!(
                "Texture dirt is using an invalid atlas index {index} for the expected UV count 2"
            )
        };
        assert_eq!(
            with_dirt(
                br#"[ { "atlas_tile": "dirt", "atlas_index": 2, "flipbook_texture": "textures/s" },
                      { "atlas_tile": "dirt", "atlas_index": -1, "flipbook_texture": "textures/s" },
                      { "atlas_tile": "dirt", "atlas_index": 1, "flipbook_texture": "textures/s" } ]"#
            ),
            [
                found("flipbooks/bad_atlas_index", Error, "0/atlas_index", &text(2)),
                found("flipbooks/bad_atlas_index", Error, "1/atlas_index", &text(-1)),
            ]
        );
    }

    #[test]
    fn a_variant_outside_the_index_is_clamped() {
        // The game clamps to variant 0 silently. Index 1 has three variants, index 0 one.
        let clamped = |at: &str| {
            found(
                "core/clamped_value",
                Info,
                at,
                "value clamped into its range",
            )
        };
        assert_eq!(
            with_dirt(
                br#"[ { "atlas_tile": "dirt", "atlas_index": 1, "atlas_tile_variant": 3, "flipbook_texture": "textures/s" },
                      { "atlas_tile": "dirt", "atlas_index": 1, "atlas_tile_variant": -1, "flipbook_texture": "textures/s" },
                      { "atlas_tile": "dirt", "atlas_tile_variant": 1, "flipbook_texture": "textures/s" } ]"#
            ),
            [
                clamped("0/atlas_tile_variant"),
                clamped("1/atlas_tile_variant"),
                clamped("2/atlas_tile_variant"),
            ]
        );
    }

    #[test]
    fn without_a_terrain_atlas_the_tile_is_not_checked() {
        let strip = png_header(16, 32, 6);
        assert_eq!(
            linked(&[&[
                (
                    FLIPBOOKS,
                    br#"[ { "atlas_tile": "ghost", "flipbook_texture": "textures/s" } ]"#
                ),
                ("textures/s.png", &strip),
            ]]),
            []
        );
    }

    #[test]
    fn a_frame_index_outside_the_image_is_an_info() {
        // The game's assertion never fires (it reads the element before it is assigned). Two
        // frames.
        assert_eq!(
            with_dirt(
                br#"[ { "atlas_tile": "dirt", "flipbook_texture": "textures/s", "frames": [0, 1, 2, -1] } ]"#
            ),
            [
                found("flipbooks/frame_out_of_range", Info, "0/frames/2", FRAME),
                found("flipbooks/frame_out_of_range", Info, "0/frames/3", FRAME),
            ]
        );
    }

    #[test]
    fn an_image_wider_than_tall_has_no_frame() {
        // The game has a debug assertion only.
        let wide = png_header(32, 16, 6);
        let square = png_header(16, 16, 6);
        let book: &[u8] = br#"[ { "atlas_tile": "x", "flipbook_texture": "textures/s" } ]"#;
        assert_eq!(
            linked(&[&[(FLIPBOOKS, book), ("textures/s.png", &wide)]]),
            [found(
                "flipbooks/no_frames",
                Info,
                "0/flipbook_texture",
                "Flipbook texture should have at least one frame. "
            )]
        );
        // A square image is one frame.
        assert_eq!(
            linked(&[&[(FLIPBOOKS, book), ("textures/s.png", &square)]]),
            []
        );
    }

    #[test]
    fn an_image_no_pack_has_is_an_info() {
        // The game has a debug assertion only, so the crate reports an information.
        assert_eq!(
            linked(&[&[(
                FLIPBOOKS,
                br#"[ { "atlas_tile": "x", "flipbook_texture": "textures/blocks/nope" } ]"#
            )]]),
            [found(
                "flipbooks/no_metadata",
                Info,
                "0/flipbook_texture",
                "Texture metadata for texture: textures/blocks/nope is invalid"
            )]
        );
    }

    #[test]
    fn the_highest_pack_wins_and_inside_a_pack_the_extension_order() {
        let book: &[u8] =
            br#"[ { "atlas_tile": "x", "flipbook_texture": "textures/lava", "frames": [3] } ]"#;
        // A 16x64 TGA: four frames.
        let mut tga = vec![0u8; 18];
        tga[2] = 2;
        tga[12] = 16;
        tga[14] = 64;
        tga[16] = 32;
        let png = png_header(16, 32, 6);
        // A 16x16 JPEG: one frame.
        let jpg = [0xFF, 0xD8, 0xFF, 0xC0, 0, 11, 8, 0, 16, 0, 16, 3];
        let out_of_range = [found(
            "flipbooks/frame_out_of_range",
            Info,
            "0/frames/0",
            FRAME,
        )];
        // The top pack's `.tga` beats its `.png`, and the lower pack's `.jpg`.
        assert_eq!(
            linked(&[
                &[
                    (FLIPBOOKS, book),
                    ("textures/lava.tga", &tga),
                    ("textures/lava.png", &png)
                ],
                &[("textures/lava.jpg", &jpg)],
            ]),
            []
        );
        assert_eq!(
            linked(&[
                &[(FLIPBOOKS, book), ("textures/lava.png", &png)],
                &[("textures/lava.jpg", &jpg)]
            ]),
            out_of_range
        );
        assert_eq!(
            linked(&[&[(FLIPBOOKS, book)], &[("textures/lava.jpg", &jpg)]]),
            out_of_range
        );
        // The higher pack wins over the extension order.
        assert_eq!(
            linked(&[
                &[(FLIPBOOKS, book), ("textures/lava.png", &png)],
                &[("textures/lava.tga", &tga)]
            ]),
            out_of_range
        );
    }

    #[test]
    fn a_path_with_an_image_extension_is_loaded_as_written() {
        let strip = png_header(16, 32, 6);
        assert_eq!(
            linked(&[&[
                (
                    FLIPBOOKS,
                    br#"[ { "atlas_tile": "x", "flipbook_texture": "textures/s.png" } ]"#
                ),
                ("textures/s.png", &strip),
            ]]),
            []
        );
    }
}
