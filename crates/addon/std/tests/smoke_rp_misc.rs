#![allow(clippy::pedantic, missing_docs)]
//! `rp_misc` smoke over the game's own files: the splashes, loading messages, fonts, emoticons
//! and UI side files of the client installs' resource packs load through the `Driver` with zero
//! error-level diagnostics; the game's own credits and texture mapping
//! parse; the three skin packs load against their files; the UI side files of the sample packs
//! load. The sample packs have no `splashes.json`, `loading_messages.json`, `font/`,
//! `skins.json` or `credits/`: those parts say so and are skipped there.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo test -p bedrock_addon_std --test
//! smoke_rp_misc`; without the variable the tests skip.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::{Diagnostic, DiagnosticCode, Severity};
use bedrock_addon::document::Content;
use bedrock_addon::families::font::{
    GlyphPage, GlyphRemapping, SMOOTH_PREFIX, check_glyph_sheet, remapping_path,
};
use bedrock_addon::families::legacy_path_mapping::LegacyPathMapping;
use bedrock_addon::families::loading_messages::MessageList;
use bedrock_addon::families::skin::SkinPack;
use bedrock_addon::families::splashes::{SplashClient, splash_pool};
use bedrock_addon::load::{LoadContext, LoadOptions, StackOptions};
use bedrock_addon::pack::{OpenOptions, PackOrigin, PackStack, PackType, StackSet};
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::SemVersion;
use bedrock_addon_std::{DirSource, Driver, Options};

use common::game_files;
use common::world::base_game_world;

/// The folders of the resource packs of a client install, in no order: for counting what is on
/// disk.
fn pack_dirs(root: &Path) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(root.join("resource_packs"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|dir| dir.join("manifest.json").is_file())
        .collect();
    dirs.sort();
    dirs
}

/// The stacks of the install at `root`, every pack a base-game pack, the resource side in the
/// client's base-game order (`common::world::base_game_world`: the splash pool, the loading
/// messages and the fonts depend on it).
fn install(root: &Path, options: &LoadOptions) -> StackSet {
    let mut driver_options = Options::default();
    driver_options.open.origin = PackOrigin::Package;
    let mut driver = Driver::new(driver_options).expect("a driver");
    let repository = driver
        .repository([
            DirSource::packs(root.join("behavior_packs"), PackOrigin::Package),
            DirSource::packs(root.join("resource_packs"), PackOrigin::Package),
        ])
        .expect("the repository");
    let world = base_game_world(&repository, options);
    driver
        .load_stacks(
            repository.stacks(&world, options.clone()),
            StackOptions::default(),
        )
        .expect("the stacks load")
}

/// The resource stack of the one pack at `dir`, loaded by the driver that opened it.
fn stack(dir: &Path, options: &LoadOptions) -> StackSet {
    let mut driver = Driver::new(Options::default().with_open(OpenOptions {
        origin: PackOrigin::Package,
        ..OpenOptions::default()
    }))
    .unwrap();
    let pack = driver
        .pack_dir(dir)
        .unwrap_or_else(|e| panic!("{}: {e:?}", dir.display()));
    let set = StackSet {
        behavior: PackStack::of([], options.clone()).unwrap(),
        resource: PackStack::of([pack], options.clone()).unwrap(),
    };
    driver.load_stacks(set, StackOptions::default()).unwrap()
}

/// Counts findings by code and severity; panics on an error.
fn tally<'a>(
    what: &str,
    findings: impl Iterator<Item = &'a Diagnostic>,
) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    let mut errors = Vec::new();
    for finding in findings {
        if finding.severity == Severity::Error {
            errors.push(finding.to_string());
        }
        let key = format!("{} ({})", finding.code.name(), finding.severity);
        *counts.entry(key).or_default() += 1;
    }
    assert!(
        errors.is_empty(),
        "{what}: {} errors:\n{}",
        errors.len(),
        errors.join("\n")
    );
    counts
}

/// The files below `dir`, at any depth, as paths relative to it.
fn files_below(dir: &Path) -> Vec<String> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(root, &path, out);
            } else {
                out.push(
                    path.strip_prefix(root)
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/"),
                );
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, dir, &mut out);
    out.sort();
    out
}

/// What one client install holds, exactly: a number that moves is a change of the game's files
/// or of a reader, and either is worth a look.
struct Expected {
    /// The texts of vanilla's own `splashes.json`.
    vanilla_splashes: usize,
    /// The pool of the full game: `beta`'s copy replaces vanilla's with its one text.
    splash_pool: usize,
    /// The tips of the highest file (`beta`'s, which uses the legacy list), per list in
    /// `MessageList::ALL`'s order.
    tips: [usize; 8],
    /// The sections of the game's own credits.
    credits_sections: usize,
    /// The paths that have a UI side file, and the resource packs of the install.
    side_files: usize,
    packs: usize,
    /// The resource packs on the stack `base_game_world` builds.
    on_stack: usize,
}

/// What client install B holds.
const INSTALL_B: Expected = Expected {
    vanilla_splashes: 396,
    splash_pool: 1,
    tips: [16, 16, 16, 16, 8, 0, 16, 0],
    credits_sections: 22,
    side_files: 390,
    packs: 72,
    on_stack: 65,
};

/// What client install A holds.
const INSTALL_A: Expected = Expected {
    vanilla_splashes: 395,
    splash_pool: 1,
    tips: [17, 17, 17, 17, 8, 0, 17, 0],
    credits_sections: 17,
    side_files: 393,
    packs: 78,
    on_stack: 72,
};

/// The findings of a collection as `code (severity)` with their counts, for an exact match.
fn exactly(counts: &BTreeMap<String, usize>, want: &[(&str, usize)], what: &str) {
    let want: BTreeMap<String, usize> = want
        .iter()
        .map(|(key, count)| ((*key).to_owned(), *count))
        .collect();
    assert_eq!(counts, &want, "{what}");
}

/// One client install: the stack families, then the files outside the packs.
fn check_client(source: &str, root: &Path, want: &Expected) {
    let dirs = pack_dirs(root);
    let credits_file = root.join("credits/credits.json");
    // `read_app_data` hands over the game's own credits with the `definitions/` folders.
    let options = LoadOptions {
        app_data: Some(bedrock_addon_std::read_app_data(root).expect("the app data reads")),
        ..LoadOptions::default()
    };
    let set = install(root, &options);
    // The helper's world lists every pack of the install. The `chemistry_<version>` resource
    // slices are hidden all the same: each depends on its behaviour slice's UUID at
    // `[1, 0, 0]`, and that pack's header is the slice's version (`[1, 20, 50]` …), so the
    // dependency names a pack the repository does not have (`MissingDependency`). The game
    // hides such a pack without a word, so for an app-bundled pack the finding is an
    // information.
    let unmet: std::collections::BTreeSet<uuid::Uuid> = set
        .selection()
        .iter()
        .filter_map(|d| match &d.code {
            DiagnosticCode::MissingDependency { pack, dependency }
                if pack.pack_type == PackType::Resource
                    && dependency.version == SemVersion::new(1, 0, 0) =>
            {
                assert_eq!(d.severity, Severity::Info, "{source}: {d}");
                Some(pack.uuid)
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        unmet.len(),
        want.packs - want.on_stack,
        "{source}: one unmet dependency at [1, 0, 0] per chemistry slice"
    );
    let off_stack = dirs
        .iter()
        .filter(|dir| {
            let name = dir.file_name().unwrap().to_string_lossy();
            name.starts_with("chemistry_")
        })
        .count();
    assert_eq!(
        (set.resource.packs().count(), dirs.len(), off_stack),
        (want.on_stack, want.packs, want.packs - want.on_stack),
        "{source}: the resource packs on the stack, on disk, and the chemistry slices"
    );
    let rp = &set.resource;

    // Splashes: vanilla's list, then whatever the packs above make of it.
    let splashes = rp.splashes();
    let counts = tally("splashes", splashes.diagnostics());
    let copies = splashes.layers().len();
    let pool = splashes.texts();
    assert_eq!(copies, 2, "{source}: vanilla's splashes.json and beta's");
    assert_eq!(
        pool,
        splash_pool(splashes.layers(), &SplashClient::default())
    );
    let lowest = splashes.layers().last().unwrap();
    assert_eq!(
        (lowest.texts().len(), pool.len()),
        (want.vanilla_splashes, want.splash_pool),
        "{source}: vanilla's own list, and the pool beta's copy leaves"
    );
    exactly(&counts, &[("SplashesReplaced (info)", 1)], source);
    println!(
        "smoke_rp_misc[{source}]: splashes: {copies} copies, pool of {}, lowest copy {} texts; {counts:?}",
        pool.len(),
        lowest.texts().len()
    );

    // Loading messages: the highest pack's file.
    let loading = rp.loading_messages();
    let counts = tally("loading messages", loading.diagnostics());
    let sizes: Vec<(&str, usize)> = MessageList::ALL
        .iter()
        .map(|list| (list.key(), loading.tips(*list).len()))
        .collect();
    assert_eq!(
        loading.layers().len(),
        2,
        "{source}: vanilla's loading_messages.json and beta's"
    );
    assert_eq!(
        sizes.iter().map(|(_, tips)| *tips).collect::<Vec<_>>(),
        want.tips,
        "{source}: the tips of the highest file"
    );
    // `MissingRequiredField` informations for the two lists the `beta` file lacks: addons and
    // editor.
    exactly(&counts, &[("MissingRequiredField (info)", 2)], source);
    let lowest = loading.layers().last().unwrap();
    assert!(
        lowest.tips(MessageList::Beginner).len() > 3,
        "{source}: vanilla's own tips"
    );
    assert!(
        lowest
            .diagnostics()
            .iter()
            .all(|d| d.severity < Severity::Warning),
        "{source}: vanilla's file: {:?}",
        lowest.diagnostics()
    );
    println!(
        "smoke_rp_misc[{source}]: loading messages: {} copies, highest gives {sizes:?}; {counts:?}",
        loading.layers().len()
    );

    // Credits: the game's own file; a pack carries none.
    let credits = rp.credits();
    let counts = tally("credits", credits.diagnostics());
    assert!(credits_file.is_file(), "{source}: the game's own credits");
    assert_eq!(credits.sections.len(), want.credits_sections, "{source}");
    exactly(&counts, &[], source);
    // The game's own file raises nothing. The one file the packs carry in vain is vanilla's
    // `manifest_publish.json`, which nothing reads; it is an `IgnoredFile` information of the
    // stack. The starter seeds of the base-game packs are the game's own.
    let report = rp.report();
    let in_vain: Vec<&str> = report
        .iter()
        .filter(|d| matches!(d.code, DiagnosticCode::IgnoredFile { .. }))
        .filter_map(|d| d.location.path.as_ref().map(PackPath::as_str))
        .filter(|path| {
            *path == bedrock_addon::families::credits::CREDITS_PATH
                || bedrock_addon::families::credits::APP_ONLY_FILES.contains(path)
                || path.starts_with("persona/")
        })
        .collect();
    assert_eq!(in_vain, ["manifest_publish.json"], "{source}");
    println!(
        "smoke_rp_misc[{source}]: credits: {} sections; {counts:?}",
        credits.sections.len()
    );

    // Fonts.
    let fonts = rp.fonts();
    let counts = tally("fonts", fonts.diagnostics());
    let list = fonts.metadata.value();
    assert_eq!(
        list.fonts.as_ref().map(Vec::len),
        Some(2),
        "{source}: MinecraftTen and OldSmoothFont"
    );
    assert_eq!(
        list.font_aliases.as_ref().map(Vec::len),
        Some(4),
        "{source}"
    );
    // Vanilla's own two: the alias font `unicode` names no font, and SmoothSmall's scale
    // override is written `1`.
    let list_warnings: Vec<String> = fonts
        .metadata
        .diagnostics()
        .map(|d| d.location.json_path.to_string())
        .collect();
    assert_eq!(
        list_warnings,
        [
            "font_aliases/3/font_scale_factor_override",
            "font_aliases/2/fonts/1/font_reference"
        ],
        "{source}"
    );
    assert_eq!(
        fonts.emoticons.diagnostics().count(),
        0,
        "{source}: emoticons"
    );
    let emoticons = fonts.emoticons.code_points();
    assert_eq!(
        (emoticons.len(), fonts.emoticons.emoticons.len()),
        (158, 158),
        "{source}: every emoticon registers"
    );
    let ten = fonts
        .true_type_file(list.font("MinecraftTen").unwrap())
        .expect("minecraft-ten.ttf");
    assert_eq!(ten.path.as_str(), "font/minecraft-ten.ttf");
    let pages = fonts.msdf_pages(SMOOTH_PREFIX);
    assert_eq!(pages.len(), 128, "{source}: smooth pages");
    assert!(
        pages
            .iter()
            .all(|page| page.image.is_some() && page.metrics.is_some()),
        "{source}"
    );
    // The two of the font list (the alias font and the scale override) are informations of the
    // crate's own and are all: no finding about a font file.
    exactly(&counts, &[("InvalidValue (info)", 2)], source);
    println!(
        "smoke_rp_misc[{source}]: fonts: 2 fonts, 4 aliases, {} emoticons, {} smooth pages; {counts:?}",
        emoticons.len(),
        pages.len()
    );

    // The binary font files and the glyph sheets, from disk.
    let font_dir = root.join("resource_packs/vanilla/font");
    let (mut metrics, mut zero, mut sheets) = (0, 0, 0);
    for name in files_below(&font_dir) {
        let bytes = std::fs::read(font_dir.join(&name)).unwrap();
        if name.ends_with(".fontdata") {
            assert_eq!(bytes.len(), 1028, "{source}: {name}");
            let (page, findings) =
                GlyphPage::from_bytes(&bytes).unwrap_or_else(|e| panic!("{source}: {name}: {e:?}"));
            assert!(
                findings.iter().all(|d| d.severity == Severity::Info),
                "{source}: {name}"
            );
            zero += findings.len();
            assert_eq!(page.to_bytes(), bytes, "{source}: {name} round trip");
            metrics += 1;
        } else if name.ends_with(".png") && !name.contains('/') {
            assert_eq!(
                check_glyph_sheet(&bytes).len(),
                0,
                "{source}: {name} is a 16 x 16 grid"
            );
            sheets += 1;
        }
    }
    let remapping = std::fs::read(
        root.join("resource_packs/vanilla")
            .join(remapping_path(SMOOTH_PREFIX)),
    )
    .unwrap();
    let (table, findings) = GlyphRemapping::from_bytes(&remapping);
    assert_eq!(findings.len(), 0, "{source}: remapping.dat");
    assert_eq!(table.to_bytes(), remapping);
    assert_eq!(
        table.to_map().len(),
        table.0.len(),
        "{source}: each character once"
    );
    println!(
        "smoke_rp_misc[{source}]: {metrics} .fontdata ({zero} advances not positive), {sheets} glyph sheets, {} remapping records",
        table.0.len()
    );
    // Vanilla has 143 zero entries across its 128 pages (unused slots).
    assert_eq!(
        (metrics, zero, sheets, table.0.len()),
        (128, 143, 228, 31_232),
        "{source}: .fontdata, advances not positive, glyph sheets, remapping records"
    );

    // UI side files.
    let infos = rp.ui_texture_infos();
    let counts = tally("ui side files", infos.diagnostics());
    let on_disk: usize = dirs
        .iter()
        .map(|dir| {
            files_below(&dir.join("textures/ui"))
                .iter()
                .filter(|name| name.ends_with(".json"))
                .count()
        })
        .sum();
    assert_eq!(
        (infos.iter().count(), on_disk, dirs.len()),
        (want.side_files, want.side_files, want.packs),
        "{source}: side-file paths, files on disk, packs: no path has two copies"
    );
    assert!(infos.rejected().is_empty(), "{source}");
    // The one side file without an image is an `IgnoredFile` information.
    exactly(&counts, &[("IgnoredFile (info)", 1)], source);
    println!(
        "smoke_rp_misc[{source}]: ui side files: {} paths from {on_disk} files in {} packs; {counts:?}",
        infos.iter().count(),
        dirs.len()
    );

    // The game's own texture mapping: no pack file, parsed on its own.
    let mapping = root.join("texture_backwards_compatibility_mapping.json");
    {
        let bytes = std::fs::read(&mapping).unwrap_or_else(|e| panic!("{source}: {e}"));
        let path = PackPath::from_static("texture_backwards_compatibility_mapping.json");
        let document = LegacyPathMapping::parse(&path, &bytes, &LoadContext::detached())
            .unwrap_or_else(|rejected| panic!("{source}: {:?}", rejected.diagnostics));
        let counts = tally("texture mapping", document.diagnostics().iter());
        exactly(&counts, &[], source);
        println!("smoke_rp_misc[{source}]: texture mapping parses; {counts:?}");
    }

    // Skin packs.
    // Three skin packs: `custom` and `persona` list no skin; `vanilla` has ten, one of them
    // with a `type` that is neither `free` nor `paid` (an `InvalidValue` information).
    for (pack, skin_count, findings) in [
        ("custom", 0, &[][..]),
        ("persona", 0, &[][..]),
        ("vanilla", 10, &[("InvalidValue (info)", 1)][..]),
    ] {
        let dir = root.join("skin_packs").join(pack);
        let skins = std::fs::read(dir.join("skins.json"))
            .unwrap_or_else(|e| panic!("{source}: skin_packs/{pack}/skins.json: {e}"));
        let geometry = std::fs::read(dir.join("geometry.json")).ok();
        let paths = files_below(&dir);
        let all = SkinPack::parse(
            &PackPath::from_static("skins.json"),
            &skins,
            &LoadContext::detached(),
        )
        .unwrap_or_else(|rejected| panic!("{source}: {pack}: {:?}", rejected.diagnostics));
        let loaded = SkinPack::load(
            Some(&skins),
            geometry.as_deref(),
            paths.iter().map(String::as_str),
            &LoadContext::detached(),
        )
        .unwrap_or_else(|rejected| panic!("{source}: {pack}: {:?}", rejected.diagnostics));
        let counts = tally("skins", loaded.diagnostics().iter());
        let (written, kept) = (
            all.skins.as_ref().map_or(0, Vec::len),
            loaded.skins.as_ref().map_or(0, Vec::len),
        );
        assert_eq!(
            (written, kept),
            (skin_count, skin_count),
            "{source}: {pack}: every skin has its files: {counts:?}"
        );
        exactly(&counts, findings, &format!("{source}: skin_packs/{pack}"));
        println!("smoke_rp_misc[{source}]: skin_packs/{pack}: {kept} skins; {counts:?}");
    }
}

#[test]
fn client_install_b_loads_without_errors() {
    let Some(root) = game_files::source("gdk-retail") else {
        eprintln!("{}", game_files::missing("client install B"));
        return;
    };
    check_client("client install B", &root, &INSTALL_B);
}

#[test]
fn client_install_a_loads_without_errors() {
    let Some(root) = game_files::source("gdk-dev") else {
        eprintln!("{}", game_files::missing("client install A"));
        return;
    };
    check_client("client install A", &root, &INSTALL_A);
}

#[test]
fn sample_packs_ui_side_files_load_without_errors() {
    let Some(root) = game_files::source("bedrock-samples") else {
        eprintln!("{}", game_files::missing("sample packs"));
        return;
    };
    let dir = root.join("resource_pack");
    for absent in [
        "splashes.json",
        "loading_messages.json",
        "font",
        "skins.json",
        "credits",
    ] {
        assert!(
            !dir.join(absent).exists(),
            "the sample packs now ship {absent}: smoke it"
        );
        println!("smoke_rp_misc[sample packs]: no {absent}; skipped");
    }
    let set = stack(&dir, &LoadOptions::default());
    let rp = &set.resource;
    let infos = rp.ui_texture_infos();
    let counts = tally("ui side files", infos.diagnostics());
    let on_disk = files_below(&dir.join("textures/ui"))
        .iter()
        .filter(|name| name.ends_with(".json"))
        .count();
    assert_eq!(
        infos.iter().count(),
        on_disk,
        "every side file on disk loads"
    );
    assert!(infos.rejected().is_empty());
    assert_eq!(on_disk, 383, "side files");
    // The one side file without an image is an `IgnoredFile` information.
    exactly(&counts, &[("IgnoredFile (info)", 1)], "sample packs");
    println!("smoke_rp_misc[sample packs]: ui side files: {on_disk}; {counts:?}");
    // The families without a file are empty and silent.
    assert_eq!(rp.splashes().diagnostics().count(), 0);
    assert_eq!(rp.loading_messages().diagnostics().count(), 0);
    assert_eq!(rp.credits().diagnostics().count(), 0);
    let fonts = tally("fonts", rp.fonts().diagnostics());
    exactly(&fonts, &[], "sample packs: fonts");
    println!(
        "smoke_rp_misc[sample packs]: font files (the language sheets under texts/): {fonts:?}"
    );
}
