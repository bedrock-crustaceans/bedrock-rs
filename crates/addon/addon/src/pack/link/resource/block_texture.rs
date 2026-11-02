//! The textures of `blocks.json` against the terrain atlas.

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::id::AtlasKey;

use super::super::{At, Findings, LinkCx, Target, kinds, unresolved};
use super::blocks_json::{Writers, entry_key, entry_path};
use super::told::{Told, formatted};
use crate::families::blocks_json::FaceTextures;
use crate::families::texture_atlas::TextureAtlas;

/// A tile without elements: the texture name. The game logs this at the warning level of the
/// content log.
const NO_ELEMENTS: &str =
    "Terrain texture <{}> doesn't have any elements. Using the missing texture.";
/// A tile the atlas lacks: the tile's name. The atlas writes this to the developer log only,
/// and then hands back the `missing` item.
const NO_SUCH_ITEM: &str = "This texture item doesn't exist: Texture [{}]";

/// The two members of an entry that name textures.
const MEMBERS: [&str; 2] = ["textures", "carried_textures"];

/// A texture name of `blocks.json` as the client reads it: the tile of the terrain atlas, and
/// the element of the tile when the name is `<tile>:<N>` with a tile of letters, digits and
/// `_` and a number of digits. Any other name is a tile's name whole.
// The game matches the name with the pattern `^(\w+):([0-9]+)$`.
fn tile_and_element(name: &str) -> (&str, Option<&str>) {
    match name.rsplit_once(':') {
        Some((tile, digits))
            if !tile.is_empty()
                && tile
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
                && !digits.is_empty()
                && digits.bytes().all(|byte| byte.is_ascii_digit()) =>
        {
            (tile, Some(digits))
        }
        _ => (name, None),
    }
}

/// What is wrong with a texture name, if anything.
#[derive(Debug)]
enum Miss {
    /// No tile of the name: the client draws the missing texture.
    MissingTile(Option<bedrock_addon_core::diagnostic::ReferenceHint>),
    /// A tile without elements: the client draws the missing texture.
    EmptyTile,
    /// The tile has elements and not the one the name selects: the client draws element 0.
    WrongElement,
}

/// Checks the texture name `name` against the terrain atlas of the stack (`None`: the stack
/// builds none and only the host knows the game's tiles).
fn miss(cx: &LinkCx<'_>, atlas: Option<&TextureAtlas>, name: &str) -> Option<Miss> {
    let (tile, element) = tile_and_element(name);
    let Some(item) = atlas.and_then(|atlas| atlas.get(tile)) else {
        return match cx.terrain_atlas_key(&AtlasKey::parse(tile)) {
            Target::Missing(hint) => Some(Miss::MissingTile(hint)),
            _ => None,
        };
    };
    let count = item.elements().len();
    if count == 0 {
        return Some(Miss::EmptyTile);
    }
    // A number beyond what a machine word holds is beyond every tile.
    let beyond = element.is_some_and(|digits| digits.parse::<usize>().map_or(true, |n| n >= count));
    beyond.then_some(Miss::WrongElement)
}

/// Each texture of a `blocks.json` entry (`textures`, `carried_textures`) names a tile of the
/// merged terrain atlas, `<tile>:<N>` its element `N`. The client resolves them when it loads
/// a block's graphics, for every block the registry has, so the grading follows the kind of
/// miss and not where the block comes from: a tile the atlas lacks is an
/// `UnresolvedReference` information with the atlas's developer-log text (the missing texture
/// is drawn); a tile without elements is a warning with the content-log text. A name that
/// selects an element its tile does not have is an `UnresolvedReference` information of the
/// crate's own, since the game says nothing of it. Each is told
/// at the member of the entry of the pack that wrote it, once per tile.
///
/// The tiles are judged by the view the pass has of the terrain atlas
/// ([`LinkCx::terrain_atlas_view`]): a stack without the atlas file has nothing to compare
/// with, unless a host answers for the game's tiles; a tile the atlas lacks is an information
/// in every view.
// The merged entry holds its textures as the six faces, so the finding is at the member and
// not at the face the file wrote.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    let view = cx.terrain_atlas_view(out);
    if !view.judges() {
        return;
    }
    let atlas = cx.resource().texture_atlases().terrain();
    let mut told = Told::default();
    let mut writers = Writers::default();
    for (id, document) in cx.resource().client_blocks() {
        // The client drops an entry of a block its registry lacks before any texture is read,
        // so only the `blocks.json` key rule tells it.
        if matches!(cx.block(id), Target::NotInView | Target::Missing(_)) {
            continue;
        }
        let block = document.value();
        let members = [&block.textures, &block.carried_textures];
        for (member, textures) in MEMBERS.into_iter().zip(members) {
            let Some(textures) = textures else {
                continue;
            };
            let mut seen: Vec<&str> = Vec::with_capacity(6);
            for name in FaceTextures::faces(textures) {
                if seen.contains(&name) {
                    continue;
                }
                seen.push(name);
                let Some(miss) = miss(cx, atlas, name) else {
                    continue;
                };
                // The member is the pack's that wrote it, which may be a lower one than the
                // merged entry's.
                let origin = writers.of(cx, out, id, member, document.origin());
                let key = entry_key(id, &origin);
                if !told.first(&origin, key, member, tile_and_element(name).0) {
                    continue;
                }
                // An element beyond the tile draws element 0 without a log line: the crate
                // reports an information. A tile without elements is logged by the game at
                // the warning level at draw time: a warning.
                let severity = match miss {
                    Miss::MissingTile(_) | Miss::WrongElement => Severity::Info,
                    Miss::EmptyTile => Severity::Warning,
                };
                let at = At::new(&origin, entry_path(key, Some(member)), severity);
                out.push(match miss {
                    Miss::MissingTile(hint) => unresolved(
                        at.with_text(formatted(NO_SUCH_ITEM, &[tile_and_element(name).0])),
                        kinds::ATLAS_KEY,
                        name,
                        hint,
                    ),
                    Miss::EmptyTile => unresolved(
                        at.with_text(formatted(NO_ELEMENTS, &[name])),
                        kinds::ATLAS_KEY,
                        name,
                        None,
                    ),
                    Miss::WrongElement => unresolved(at, kinds::ATLAS_KEY, name, None),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Write as _;

    use super::super::super::tests::support::set;
    use super::super::super::{LinkOptions, link_with_rules};
    use super::{NO_ELEMENTS, NO_SUCH_ITEM, run, tile_and_element};

    #[test]
    fn a_name_is_a_tile_or_a_tile_and_its_element() {
        assert_eq!(tile_and_element("stone"), ("stone", None));
        assert_eq!(tile_and_element("wool:12"), ("wool", Some("12")));
        assert_eq!(tile_and_element("Wool_2:0"), ("Wool_2", Some("0")));
        // Not the client's pattern: the whole text is the tile's name.
        assert_eq!(tile_and_element("wool:"), ("wool:", None));
        assert_eq!(tile_and_element(":3"), (":3", None));
        assert_eq!(tile_and_element("wool:x"), ("wool:x", None));
        assert_eq!(tile_and_element("a-b:3"), ("a-b:3", None));
        assert_eq!(tile_and_element("a:b:3"), ("a:b:3", None));
        assert_eq!(NO_ELEMENTS.matches("{}").count(), 1);
        assert_eq!(NO_SUCH_ITEM.matches("{}").count(), 1);
    }

    /// `n` entries of `n` custom blocks (an entry of a block the registry lacks is not
    /// looked at), each with its own tile: one in four names a tile the atlas lacks
    /// (two lookups: the atlas, then the names for the near miss) and one in four an element
    /// beyond the tile.
    #[test]
    fn n_textures_cost_n() {
        let input = |n: usize| {
            let mut blocks = String::from(r#"{"format_version": "1.21.40""#);
            let mut atlas = String::from(
                r#"{"resource_pack_name": "t", "texture_name": "atlas.terrain", "texture_data": {"first": {"textures": "textures/first"}"#,
            );
            for index in 0..n {
                let name = match index % 4 {
                    0 => format!("Tile_{index}"),
                    1 => format!("tile_{index}:1"),
                    _ => format!("tile_{index}:0"),
                };
                let _ = write!(blocks, r#", "demo:b{index}": {{"textures": "{name}"}}"#);
                let _ = write!(
                    atlas,
                    r#", "tile_{index}": {{"textures": ["textures/t{index}"]}}"#
                );
            }
            blocks.push('}');
            atlas.push_str("}}");
            let definitions: Vec<(&'static str, String)> = (0..n)
                .map(|index| {
                    let path: &'static str =
                        Box::leak(format!("blocks/b{index}.json").into_boxed_str());
                    let text = format!(
                        r#"{{"format_version": "1.26.20", "minecraft:block": {{"description": {{"identifier": "demo:b{index}"}}, "components": {{}}}}}}"#
                    );
                    (path, text)
                })
                .collect();
            let behavior: Vec<(&'static str, &str)> = definitions
                .iter()
                .map(|(path, text)| (*path, text.as_str()))
                .collect();
            let world = set(
                &behavior,
                &[
                    ("blocks.json", blocks.as_str()),
                    ("textures/terrain_texture.json", atlas.as_str()),
                ],
            );
            assert_eq!(world.resource.client_blocks().iter().count(), n);
            assert!(world.resource.texture_atlases().terrain().is_some());
            (world, n.div_ceil(4) + (n + 2) / 4)
        };
        crate::test_cost::assert_linear_at(
            "link: n blocks.json textures",
            6_000,
            input,
            |(world, missing)| {
                let found = link_with_rules(world, LinkOptions::default(), &[run]);
                assert_eq!(found.len(), *missing);
                found
            },
        );
    }
}
