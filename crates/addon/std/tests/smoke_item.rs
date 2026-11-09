//! Item smoke over the game's own files: the vanilla behaviour-pack items of the sample packs
//! and of the server install load through `Driver` into `bp.items()` with zero error-level
//! diagnostics. MCB documents (baked items in the server install's brarchives) are decoded and
//! read like the JSON files.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo nextest run -p bedrock_addon_std --test smoke_item`;
//! without the variable every test skips.

#[macro_use]
mod common;

use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::{Severity, codes};
use bedrock_addon::load::{LoadOptions, StackOptions};
use bedrock_addon::pack::PackOrigin;
use bedrock_addon_std::{DirSource, Driver, Options, Strategy};

use common::game_files;
use common::world::world_list;

/// The pack folders directly below `dir`.
fn pack_roots(dir: &Path) -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|entries| entries.filter_map(|e| e.ok().map(|e| e.path())).collect())
        .unwrap_or_default();
    roots.retain(|path| path.is_dir());
    roots.sort();
    roots
}

/// The item MCB documents on disk in the packs below the packs directories `dirs` (`items/`
/// and the `items` brarchive), counted with the game-files helper, as `smoke_block` and
/// `smoke_entity` bound theirs, so an unsupported non-MCB file cannot pass silently.
fn item_mcb(dirs: &[&Path]) -> usize {
    dirs.iter()
        .flat_map(|dir| pack_roots(dir))
        .flat_map(|root| {
            [
                root.join("items"),
                root.join("__brarchive").join("items.brarchive"),
            ]
        })
        .filter(|path| path.exists())
        .map(|path| game_files::mcb_on_disk(&path).total())
        .sum()
}

/// Loads the base-game behaviour stack composed from the packs directories `dirs` (all
/// `Package` origin) and checks its items: how many loaded, no error, nothing rejected. The
/// MCB documents of the brarchives are decoded and read like the JSON files: none is an
/// `UnsupportedFileFormat` finding.
fn check_items(
    source: &str,
    dirs: &[&Path],
    world_behavior: &[u8],
    undefined_categories: usize,
) -> (usize, usize) {
    let on_disk = item_mcb(dirs);
    let mut driver = Driver::new(Options::default().with_strategy(Strategy::Sequential)).unwrap();
    let repository = driver
        .repository(
            dirs.iter()
                .map(|dir| DirSource::packs(dir.to_path_buf(), PackOrigin::Package)),
        )
        .unwrap();
    let stacks = driver
        .load_stacks(
            repository.stacks(
                &world_list(&repository, world_behavior, &LoadOptions::default()),
                LoadOptions::default(),
            ),
            StackOptions::default(),
        )
        .unwrap();
    let items = stacks.behavior.items();
    let mut mcb = 0;
    let mut errors = Vec::new();
    for finding in items.diagnostics() {
        if finding.severity < Severity::Error {
            continue;
        }
        if finding.is(&codes::UNSUPPORTED_FILE_FORMAT) {
            mcb += 1;
        }
        errors.push(finding.to_string());
    }
    let warnings = items
        .diagnostics()
        .filter(|f| f.severity == Severity::Warning)
        .count();
    eprintln!(
        "{source}: {} items over {} packs, {} rejected files, {mcb} MCB documents refused ({on_disk} on disk), {} errors, {warnings} warnings",
        items.len(),
        stacks.behavior.packs().len(),
        items.rejected().len(),
        errors.len(),
    );
    for error in &errors {
        eprintln!("{source}: {error}");
    }
    assert!(errors.is_empty(), "{source}: {} errors", errors.len());
    // No item warning: an item of 1.21.60 to 1.21.109 without a category holds `Undefined`,
    // which the newest format cannot spell. The game rewrites it without a word, so the crate
    // tells it as an information, whose count is pinned there.
    game_files::assert_warnings(
        source,
        &game_files::warnings_by_code(items.diagnostics()),
        &[],
    );
    let categories = items
        .diagnostics()
        .filter(|f| {
            f.severity == Severity::Info
                && f.code.id() == "core/value_rewritten"
                && f.location
                    .json_path
                    .to_string()
                    .ends_with("description/menu_category")
        })
        .count();
    assert_eq!(
        categories, undefined_categories,
        "{source}: Undefined categories"
    );
    assert_eq!(items.rejected().len(), 0, "{source}: no file is rejected");
    (items.len(), on_disk)
}

#[test]
fn sample_packs_items_load_without_errors() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    // the sample pack has its own UUID: the world selects it, from the vanilla (`Package`)
    // location, so it loads as a base-game pack
    let world = br#"[{"pack_id": "ee649bcf-256c-4013-9068-6a802b89d756", "version": [0, 0, 1]}]"#;
    let (_, mcb) = check_items("sample packs", &[&samples], world, 24);
    assert_eq!(mcb, 0, "the sample packs bake nothing");
}

#[test]
fn server_install_items_load_without_errors() {
    let server = skip!(game_files::SERVER_INSTALL);
    // The stack's items, the 21 baked ones among them (`trial_key`, `wind_charge`, the
    // bundles, `breeze_rod`).
    assert_eq!(
        check_items(
            "server install",
            &[&server.join("behavior_packs")],
            b"[]",
            30
        ),
        (77, 21)
    );
}

/// Every item occurrence of the Molang pairs file (the `repair_amount` strings of client
/// install B) loads with the pair's text and its version: the item's `format_version` window.
#[test]
fn every_item_molang_occurrence_loads_with_its_text_and_version() {
    use bedrock_addon::document::Content;
    use bedrock_addon::families::item::Item;
    use bedrock_addon::families::item::component::Repairable;
    use bedrock_addon::json::Value;
    use bedrock_addon::load::LoadContext;
    use bedrock_addon::molang::{Molang, RawVersion};
    use bedrock_addon::storage::PackPath;

    let pairs = skip!(game_files::MOLANG_PAIRS);
    let install_b = skip!(game_files::CLIENT_INSTALL_B);
    let text = std::fs::read_to_string(pairs).unwrap();
    let pairs_document: Value =
        bedrock_addon::json::parse_at(bedrock_addon::json::JsonSlice::new(&text)).unwrap();
    let mut cx = LoadContext::detached();
    cx.is_base_game_pack = true;
    let (mut checked, mut archived) = (0, 0);
    for pair in pairs_document
        .as_object()
        .unwrap()
        .get("pairs")
        .unwrap()
        .as_array()
        .unwrap()
    {
        let pair = pair.as_object().unwrap();
        let expr = pair.get("expr").unwrap().as_string().unwrap().as_str();
        for occurrence in pair.get("occurrences").unwrap().as_array().unwrap() {
            let o = occurrence.as_object().unwrap();
            let field = |key: &str| o.get(key).unwrap().as_string().unwrap().as_str().to_owned();
            if field("family") != "items" {
                continue;
            }
            let path = install_b
                .join(field("pack_type"))
                .join(field("pack"))
                .join(field("file"));
            let Ok(text) = std::fs::read_to_string(&path) else {
                archived += 1;
                continue;
            };
            let pointer = field("pointer");
            let index: usize = pointer
                .split('/')
                .nth(5)
                .and_then(|i| i.parse().ok())
                .unwrap_or_else(|| panic!("{pointer}"));
            let document = Item::parse(
                &PackPath::new(&field("file")).unwrap(),
                text.as_bytes(),
                &cx,
            )
            .unwrap_or_else(|r| panic!("{}: {:#?}", path.display(), r.diagnostics));
            let repairable = document.components.get::<Repairable>().unwrap();
            let entry = &repairable.repair_items[index];
            // `…/repair_amount`, or an item descriptor's `…/items/<j>/tags`
            let source = if pointer.ends_with("/tags") {
                let j: usize = pointer
                    .split('/')
                    .nth(7)
                    .and_then(|j| j.parse().ok())
                    .unwrap();
                match &entry.items[j] {
                    bedrock_addon::shared::ItemDescriptor::Tags { tags, .. } => tags.clone(),
                    other => panic!("{}: {pointer} is {other:?}", path.display()),
                }
            } else {
                let Some(Molang::Expr(source)) = &entry.repair_amount else {
                    panic!("{}: {pointer} is no expression", path.display());
                };
                source.clone()
            };
            assert_eq!(source.as_str(), expr, "{}", path.display());
            let version = o
                .get("version_effective")
                .unwrap()
                .as_number()
                .unwrap()
                .to_i64()
                .unwrap();
            assert_eq!(
                source.raw_version(),
                Some(RawVersion(i16::try_from(version).unwrap())),
                "{}: {pointer}",
                path.display()
            );
            checked += 1;
        }
    }
    eprintln!("Molang pairs: {checked} item occurrences checked, {archived} only in brarchives");
    assert!(checked > 0);
}
