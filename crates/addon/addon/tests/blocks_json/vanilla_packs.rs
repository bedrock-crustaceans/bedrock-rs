//! The legacy-name tables against vanilla's own data: the game's pre-split files are
//! split through the tables, every face is resolved through the terrain atlas the file was
//! written for, and the result is compared with the flattened entries vanilla wrote.
//!
//! Needs `BEDROCK_ADDON_CORPUS`; skipped with a message without it.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use bedrock_addon::families::blocks_json::ClientBlock;
use bedrock_addon::json::Value;

use super::common::{Loaded, base, load};

/// The corpus root, or `None` with the skip message printed.
fn corpus() -> Option<PathBuf> {
    let root = std::env::var_os("BEDROCK_ADDON_CORPUS")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);
    if root.is_none() {
        eprintln!("skipped: BEDROCK_ADDON_CORPUS is not set");
    }
    root
}

fn text(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

/// A terrain atlas: `texture_data` of each file in turn, a later file's item replacing an
/// earlier one's.
fn atlas(files: &[PathBuf]) -> BTreeMap<String, Value> {
    let mut items = BTreeMap::new();
    for file in files {
        let json = facet_json::from_str_jsonc::<Value>(text(file).trim_start_matches('\u{feff}'))
            .unwrap_or_else(|error| panic!("{}: {error}", file.display()));
        let data = json
            .as_object()
            .and_then(|root| root.get("texture_data"))
            .and_then(Value::as_object)
            .unwrap_or_else(|| panic!("{}: no texture_data", file.display()));
        for (name, item) in data {
            items.insert(name.as_str().to_owned(), item.clone());
        }
    }
    items
}

/// What the client draws for the texture `name` of a `blocks.json` entry: element `N` of the
/// atlas item for `item:N`, element 0 when the item has no element `N` (families/blocks_json
/// `idx = (count > N && N > -1) ? N: 0`), the item's one value otherwise.
fn drawn(atlas: &BTreeMap<String, Value>, name: &str) -> String {
    let (item, index) = match name.rsplit_once(':') {
        Some((item, digits))
            if !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()) =>
        {
            (item, digits.parse::<usize>().unwrap_or(0))
        }
        _ => (name, 0),
    };
    let Some(textures) = atlas
        .get(item)
        .and_then(Value::as_object)
        .and_then(|item| item.get("textures"))
    else {
        // An item the atlas lacks is drawn with the missing texture, whatever its name.
        return "<missing texture>".to_owned();
    };
    let element = match textures.as_array() {
        Some(elements) => match elements.get(index).or_else(|| elements.get(0)) {
            Some(element) => element,
            None => return format!("<{item} has no elements>"),
        },
        None => textures,
    };
    format!("{element:?}")
}

/// The six face textures of a member, when the block has it.
type SixNames<'a> = Option<[&'a str; 6]>;

const FACES: [&str; 6] = ["up", "down", "north", "south", "west", "east"];

/// One side of a comparison: a loaded file and the atlas its texture names are keys of.
struct Side {
    blocks: Loaded,
    atlas: BTreeMap<String, Value>,
}

/// What a cross-check found.
#[derive(Debug, Default)]
struct Outcome {
    /// Face comparisons made.
    comparisons: usize,
    /// `(block, member, face)` of every face the split draws differently, with both drawings.
    differences: BTreeMap<(String, &'static str, &'static str), (String, String)>,
    /// The legacy keys whose split was compared, each with its blocks.
    aliases: BTreeMap<String, BTreeSet<String>>,
    /// The split blocks the flattened side has no textures for.
    not_comparable: BTreeSet<String>,
}

/// Compares every block the pre-split file's legacy names were split onto with the flattened
/// side's entry of that block, face by face, `textures` and `carried_textures`.
fn cross_check(split: &Side, flattened: &Side) -> Outcome {
    let mut outcome = Outcome::default();
    for (id, document) in split.blocks.registry() {
        let block: &ClientBlock = document.value();
        // A split product is the block whose texture names carry a variant: the pre-split
        // files name no element themselves (asserted by the callers).
        let is_split = block
            .face_textures()
            .iter()
            .chain(&block.carried_face_textures())
            .any(|name| name.contains(':'));
        if !is_split {
            continue;
        }
        let name = id.as_str().trim_start_matches("minecraft:").to_owned();
        let key = document
            .origin()
            .key
            .as_deref()
            .unwrap_or_default()
            .to_lowercase();
        let Some(flat) = flattened.blocks.registry().get(id).map(|d| d.value()) else {
            outcome.not_comparable.insert(name);
            continue;
        };
        let members: [(&'static str, SixNames<'_>, SixNames<'_>); 2] = [
            (
                "textures",
                block.textures.as_ref().map(|t| t.faces()),
                flat.textures.as_ref().map(|t| t.faces()),
            ),
            (
                "carried_textures",
                block.carried_textures.as_ref().map(|t| t.faces()),
                flat.carried_textures.as_ref().map(|t| t.faces()),
            ),
        ];
        let mut compared = false;
        for (member, ours, theirs) in members {
            let (Some(ours), Some(theirs)) = (ours, theirs) else {
                continue;
            };
            compared = true;
            for (face, (ours, theirs)) in FACES.into_iter().zip(ours.into_iter().zip(theirs)) {
                outcome.comparisons += 1;
                let (a, b) = (drawn(&split.atlas, ours), drawn(&flattened.atlas, theirs));
                if a != b {
                    outcome
                        .differences
                        .insert((name.clone(), member, face), (a, b));
                }
            }
        }
        if compared {
            outcome.aliases.entry(key).or_default().insert(name);
        } else {
            outcome.not_comparable.insert(name);
        }
    }
    outcome
}

/// No texture name of a pre-split file names an atlas element itself.
fn assert_names_no_element(file: &str) {
    let json = facet_json::from_str_jsonc::<Value>(file.trim_start_matches('\u{feff}')).unwrap();
    for (key, entry) in json.as_object().unwrap() {
        let Some(entry) = entry.as_object() else {
            continue;
        };
        for member in ["textures", "carried_textures"] {
            let Some(value) = entry.get(member) else {
                continue;
            };
            let names: Vec<&str> = match (value.as_string(), value.as_object()) {
                (Some(name), _) => vec![name.as_str()],
                (_, Some(faces)) => faces
                    .iter()
                    .filter_map(|(_, name)| name.as_string().map(|s| s.as_str()))
                    .collect(),
                _ => Vec::new(),
            };
            assert!(
                names.iter().all(|name| !name.contains(':')),
                "{}/{member}: {names:?}",
                key.as_str()
            );
        }
    }
}

/// Removes the differences `allowed` names from `outcome`, requiring each to be there (an
/// allowance nothing needs is stale); how many were removed.
fn allow(outcome: &mut Outcome, allowed: &[(&str, &'static str, &[&'static str])]) -> usize {
    let mut removed = 0;
    for (block, member, faces) in allowed {
        for face in *faces {
            let at = ((*block).to_owned(), *member, *face);
            assert!(
                outcome.differences.remove(&at).is_some(),
                "{at:?} is allowed to differ and does not"
            );
            removed += 1;
        }
    }
    removed
}

fn names(outcome: &Outcome) -> Vec<&str> {
    outcome.aliases.keys().map(String::as_str).collect()
}

/// The retail game's `resource_packs/vanilla/blocks.json`
/// (format `[1, 1, 0]`, every legacy name still in use) split through the tables and drawn
/// with that pack's atlas, against the flattened entries of vanilla's resource pack drawn with its
/// atlas.
#[test]
fn the_split_of_vanilla_s_legacy_file_draws_what_vanilla_s_flattened_file_draws() {
    let Some(corpus) = corpus() else { return };
    let vanilla = corpus.join("gdk-retail/resource_packs/vanilla");
    let samples = corpus.join("bedrock-samples/resource_pack");
    let legacy = text(&vanilla.join("blocks.json"));
    assert_names_no_element(&legacy);
    let split = Side {
        blocks: load(&[base(&legacy)]),
        atlas: atlas(&[vanilla.join("textures/terrain_texture.json")]),
    };
    assert_eq!(
        split.blocks.block("stone").version().effective.to_string(),
        "1.1.0",
        "the file is below every gate"
    );
    let flattened = Side {
        blocks: load(&[base(&text(&samples.join("blocks.json")))]),
        atlas: atlas(&[samples.join("textures/terrain_texture.json")]),
    };
    let mut outcome = cross_check(&split, &flattened);
    let found = outcome.differences.len();
    // Vanilla's own changes of look since the split, and only those:
    let allowed = allow(
        &mut outcome,
        &[
            // the quartz block's bottom is now drawn with its top texture;
            ("quartz_block", "textures", &["down"]),
            // smooth quartz is now the quartz block's bottom texture on every face (the legacy
            // atlas drew element 3 of `quartz_block_*`, the top texture);
            ("smooth_quartz", "textures", &FACES),
            // the bottoms of chiseled and cut red sandstone are now drawn with the top texture.
            ("chiseled_red_sandstone", "textures", &["down"]),
            ("cut_red_sandstone", "textures", &["down"]),
        ],
    );
    eprintln!(
        "vanilla: {} comparisons over {:?}, {found} differences, {allowed} allowed; not comparable: {:?}",
        outcome.comparisons,
        names(&outcome),
        outcome.not_comparable
    );
    assert!(
        outcome.differences.is_empty(),
        "{} faces are drawn differently: {:#?}",
        outcome.differences.len(),
        outcome.differences
    );
    // The 44 legacy names the file uses (the tables mark the other 15 as unverified, or as
    // confirmed by the chemistry check below); `underwater_tnt` has no entry in vanilla's pack.
    // A name in this list is compared, which confirms a position only where the data draws
    // it unlike the others: the 7 of `skull` and 3 of `purpur_block` it does not.
    assert_eq!(
        names(&outcome),
        [
            "anvil",
            "carpet",
            "cobblestone_wall",
            "concrete",
            "coral",
            "coral_block",
            "coral_fan",
            "coral_fan_dead",
            "coral_fan_hang",
            "coral_fan_hang2",
            "coral_fan_hang3",
            "dirt",
            "double_plant",
            "double_wooden_slab",
            "fence",
            "leaves",
            "leaves2",
            "light_block",
            "log",
            "log2",
            "monster_egg",
            "planks",
            "prismarine",
            "purpur_block",
            "quartz_block",
            "red_flower",
            "red_sandstone",
            "sand",
            "sandstone",
            "sapling",
            "shulker_box",
            "skull",
            "sponge",
            "stained_glass",
            "stained_glass_pane",
            "stained_hardened_clay",
            "stone",
            "stonebrick",
            "structure_void",
            "tallgrass",
            "tnt",
            "wood",
            "wooden_slab",
            "wool"
        ]
    );
    assert_eq!(
        outcome.not_comparable.iter().collect::<Vec<_>>(),
        ["underwater_tnt"]
    );
    assert_eq!((outcome.comparisons, found, allowed), (2124, 9, 9));
}

/// The same check for the names only the chemistry packs use: the retail game's `vanilla` and
/// `chemistry` files (both `[1, 1, 0]`) split and drawn with the two packs' atlases, against
/// the flattened entries of the `chemistry_1.20.50`, `_1.20.60` and `_1.21.30` slices drawn
/// with the atlas of the whole chemistry stack. Only the blocks those slices flatten compare.
#[test]
fn the_split_of_the_chemistry_pack_s_legacy_file_draws_what_its_slices_draw() {
    let Some(corpus) = corpus() else { return };
    let packs = corpus.join("gdk-retail/resource_packs");
    let pack = |name: &str| packs.join(name);
    let atlas_of = |name: &str| pack(name).join("textures/terrain_texture.json");
    let vanilla = text(&pack("vanilla").join("blocks.json"));
    let chemistry = text(&pack("chemistry").join("blocks.json"));
    assert_names_no_element(&chemistry);
    let slices = [
        "chemistry_1.20.50",
        "chemistry_1.20.60",
        "chemistry_1.21.30",
    ];
    let slice_files: Vec<String> = slices
        .iter()
        .map(|name| text(&pack(name).join("blocks.json")))
        .collect();
    let mut atlases = vec![atlas_of("vanilla"), atlas_of("chemistry")];
    let split = Side {
        blocks: load(&[base(&vanilla), base(&chemistry)]),
        atlas: atlas(&atlases),
    };
    // The flattened names are keys of the atlases of the `vanilla_1.x` slices too
    // (`black_stained_glass_pane_top` is `vanilla_1.20.20`'s), oldest slice first.
    let mut vanilla_slices: Vec<(Vec<u32>, String)> = std::fs::read_dir(&packs)
        .unwrap()
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter_map(|name| {
            let version = name.strip_prefix("vanilla_")?;
            let parts: Option<Vec<u32>> =
                version.split('.').map(|part| part.parse().ok()).collect();
            Some((parts?, name))
        })
        .collect();
    vanilla_slices.sort();
    let slice_atlases = vanilla_slices
        .iter()
        .map(|(_, name)| name.as_str())
        .chain(slices)
        .map(atlas_of)
        .filter(|path| path.is_file());
    atlases.extend(slice_atlases);
    let layers: Vec<_> = slice_files.iter().map(|file| base(file)).collect();
    let flattened = Side {
        blocks: load(&layers),
        atlas: atlas(&atlases),
    };
    let mut outcome = cross_check(&split, &flattened);
    let found = outcome.differences.len();
    let allowed = allow(
        &mut outcome,
        &[
            // Nothing: `tnt` starts at variant 0, so plain TNT draws element 0 and underwater
            // TNT element 1 of the chemistry atlas items, as the 1.21.30 slice's own entries
            // do.
        ],
    );
    eprintln!(
        "chemistry: {} comparisons over {:?}, {found} differences, {allowed} allowed",
        outcome.comparisons,
        names(&outcome)
    );
    assert!(
        outcome.differences.is_empty(),
        "{} faces are drawn differently: {:#?}",
        outcome.differences.len(),
        outcome.differences
    );
    assert_eq!(
        names(&outcome),
        [
            "chemistry_table",
            "colored_torch_bp",
            "colored_torch_rg",
            "hard_stained_glass",
            "hard_stained_glass_pane",
            "tnt"
        ]
    );
    // `colored_torch_bp` starts at variant 2: blue and purple are elements 2 and 3 of the
    // atlas item the red and green torches share.
    assert_eq!(
        drawn(
            &split.atlas,
            split.blocks.block("colored_torch_blue").face_textures()[0]
        ),
        r#""textures/blocks/torch_blue""#
    );
    assert_eq!((outcome.comparisons, found, allowed), (246, 0, 0));
}
