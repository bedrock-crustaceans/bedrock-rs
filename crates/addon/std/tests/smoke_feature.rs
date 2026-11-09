//! Vanilla smoke for features: the feature documents of the server install's behaviour packs
//! and its `definitions/features` (app data), of the packs and app data of client install B,
//! and of the sample packs' `behavior_pack/` (which ships no `features/`) load through the
//! `Driver` with zero error-level diagnostics; MCB documents (baked documents the
//! loaders do not decode) are counted, not failed.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo nextest run -p bedrock_addon_std --test smoke_feature`;
//! without the variable every test skips.

#[macro_use]
mod common;
#[path = "common/pairs.rs"]
mod pairs;

use bedrock_addon::diagnostic::codes as core_codes;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::load::{LoadOptions, StackOptions};
use bedrock_addon::pack::{Pack, PackOrigin, PackRepository};
use bedrock_addon_std::{DirSource, Driver, Options, read_app_data};

use common::game_files;
use common::world::base_game_world;

/// What a load found.
struct Outcome {
    features: usize,
    rejected: usize,
    mcb: usize,
    errors: Vec<String>,
    warnings: Vec<String>,
}

/// The feature MCB documents on disk below the pack roots `roots`.
fn feature_mcb(roots: &[PathBuf]) -> game_files::McbCount {
    roots
        .iter()
        .flat_map(|root| {
            [
                root.join("features"),
                root.join("__brarchive").join("features.brarchive"),
            ]
        })
        .filter(|path| path.exists())
        .map(|path| game_files::mcb_on_disk(&path))
        .sum()
}

/// The pack folders directly below `dir`.
fn pack_roots(dir: &Path) -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|entries| entries.filter_map(|e| e.ok().map(|e| e.path())).collect())
        .unwrap_or_default();
    roots.retain(|path| path.is_dir());
    roots.sort();
    roots
}

/// Loads the behaviour stack of `repository` (with `options`) and collects the feature
/// findings.
fn load_features(
    driver: &Driver,
    repository: &PackRepository,
    roots: &[PathBuf],
    options: LoadOptions,
) -> Outcome {
    let on_disk = feature_mcb(roots).total();
    let stacks = driver
        .load_stacks(
            repository.stacks(&base_game_world(repository, &options), options),
            StackOptions::default(),
        )
        .expect("the stacks load");
    let features = stacks.behavior.features();
    let mut mcb = 0;
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    for finding in features.diagnostics() {
        if finding.severity == Severity::Warning {
            warnings.push(finding.code.id().to_owned());
        }
        if finding.severity < Severity::Error {
            continue;
        }
        if finding.is(&core_codes::UNSUPPORTED_FILE_FORMAT) && mcb < on_disk {
            mcb += 1;
            continue;
        }
        errors.push(format!(
            "{} {} {}",
            finding
                .location
                .path
                .as_ref()
                .map(std::string::ToString::to_string)
                .unwrap_or_default(),
            finding.location.json_path,
            finding.code,
        ));
    }
    Outcome {
        features: features.len(),
        rejected: features.rejected().len(),
        mcb,
        errors,
        warnings,
    }
}

/// Prints the source's line and its warnings by code; requires no error outside MCB documents
/// and the warnings by code to be `warnings`, exactly.
fn report(source: &str, outcome: &Outcome, warnings: &[(&str, usize)]) {
    eprintln!(
        "{source}: {} features, {} rejected, {} MCB documents, {} errors, {} warnings",
        outcome.features,
        outcome.rejected,
        outcome.mcb,
        outcome.errors.len(),
        outcome.warnings.len()
    );
    let mut kinds: Vec<(&String, usize)> = Vec::new();
    for warning in &outcome.warnings {
        match kinds.iter_mut().find(|(kind, _)| *kind == warning) {
            Some((_, count)) => *count += 1,
            None => kinds.push((warning, 1)),
        }
    }
    for (warning, count) in kinds {
        eprintln!("  {count} × warning: {warning}");
    }
    for error in &outcome.errors {
        eprintln!("  {error}");
    }
    assert!(outcome.errors.is_empty(), "{source}: {:#?}", outcome.errors);
    game_files::assert_warnings(
        source,
        &game_files::count_names(&outcome.warnings),
        warnings,
    );
}

/// A driver whose packs are base-game packs.
fn base_game_driver() -> Driver {
    let mut options = Options::default();
    options.open.origin = PackOrigin::Package;
    Driver::new(options).expect("a driver")
}

/// The behaviour packs and the app data of an install at `root`.
fn install(root: &Path, source: &str, warnings: &[(&str, usize)]) {
    let mut driver = base_game_driver();
    let repository = driver
        .repository([DirSource::packs(
            root.join("behavior_packs"),
            PackOrigin::Package,
        )])
        .expect("the repository");
    let options = LoadOptions {
        app_data: Some(read_app_data(root).expect("the app data")),
        ..LoadOptions::default()
    };
    let outcome = load_features(
        &driver,
        &repository,
        &pack_roots(&root.join("behavior_packs")),
        options,
    );
    assert!(outcome.features > 0, "{source} has features");
    report(source, &outcome, warnings);
}

#[test]
fn server_install_features_load_without_errors() {
    let server = skip!(game_files::SERVER_INSTALL);
    // The 207 shadowed definitions are informations: the game drops the lower definition
    // without a message, so the crate only notes it.
    install(&server, "server install", &[]);
}

#[test]
fn client_install_b_features_load_without_errors() {
    let client = skip!(game_files::CLIENT_INSTALL_B);
    // `FieldSinceEngine` is an information of the crate's own (the game is silent about it):
    // no warning is left.
    install(&client, "client install B", &[]);
}

#[test]
fn client_install_a_features_load_without_errors() {
    let client = skip!(game_files::CLIENT_INSTALL_A);
    install(&client, "client install A", &[]);
}

#[test]
fn sample_packs_features_load_without_errors() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    let mut driver = base_game_driver();
    let pack: Pack = driver
        .pack_dir(samples.join("behavior_pack"))
        .expect("the behaviour pack opens");
    let mut repository = PackRepository::new();
    repository.add_pack(pack, PackOrigin::Package);
    let outcome = load_features(
        &driver,
        &repository,
        &[samples.join("behavior_pack")],
        LoadOptions::default(),
    );
    report("sample packs", &outcome, &[]);
}

/// Every `features` occurrence of the Molang pairs file (the packs of client install B, 155)
/// names a file that loads with no error-level finding, and its Molang member reads to the
/// pair's text and effective version.
#[test]
fn molang_pairs_feature_molang_loads_with_its_text_and_version() {
    use bedrock_addon::document::Content;
    use bedrock_addon::families::feature::Feature;

    let pairs_path = skip!(game_files::MOLANG_PAIRS);
    let install_b = skip!(game_files::CLIENT_INSTALL_B);
    let found = pairs::check(&pairs_path, &install_b, "features", |file| {
        let document = Feature::parse(&file.at, file.text.as_bytes(), &file.cx)
            .map_err(|rejected| rejected.diagnostics)?;
        let diagnostics = document.diagnostics().clone();
        let mut members = pairs::Members::new();
        // The pointers start at the payload key, which names the type.
        let root: bedrock_addon::json::Value =
            bedrock_addon::json::parse_at(bedrock_addon::json::JsonSlice::new(file.text))
                .expect("JSON");
        let key = root
            .as_object()
            .and_then(|o| o.keys().find(|k| *k != "format_version").cloned())
            .expect("a payload key");
        let key = Some(key.as_str());
        macro_rules! walk {
            ($feature:expr, $($variant:ident),* $(,)?) => {
                match $feature {
                    $(Feature::$variant(inner) => pairs::members(inner, key, &mut members),)*
                    other => panic!("{}: a feature type this test does not walk: {other:?}", file.at),
                }
            };
        }
        walk!(
            &mut document.into_value(),
            Aggregate,
            BeardsAndShavers,
            CaveCarver,
            ConditionalList,
            Fossil,
            Geode,
            GrowingPlant,
            HeightDifferenceFilter,
            HorizontalTreeDecoration,
            MultiBlock,
            Multiface,
            MultipartBlockColumn,
            NetherCaveCarver,
            Ore,
            PartiallyExposedBlob,
            RectLayout,
            ScanSurface,
            Scatter,
            SculkPatch,
            Search,
            Sequence,
            SingleBlock,
            SnapToSurface,
            StructureTemplate,
            SurfaceRelativeThreshold,
            Tree,
            UnderwaterCaveCarver,
            VegetationPatch,
            WeightedRandom,
        );
        // A scatter feature written in the root layout (below 1.21.10) is upgraded by moving its
        // six root parameters into `distribution` (the first upgrade step): the pairs point at
        // the members as written.
        let written_distribution = root
            .as_object()
            .and_then(|o| o.get("minecraft:scatter_feature"))
            .and_then(|payload| payload.as_object())
            .is_none_or(|payload| payload.contains_key("distribution"));
        if !written_distribution {
            let moved = "/minecraft:scatter_feature/distribution/";
            members = members
                .into_iter()
                .map(|(pointer, member)| match pointer.strip_prefix(moved) {
                    Some(rest) => (format!("/minecraft:scatter_feature/{rest}"), member),
                    None => (pointer, member),
                })
                .collect();
        }
        Ok((members, diagnostics))
    });
    eprintln!(
        "Molang pairs: {} feature occurrences checked in {} files",
        found.compared, found.files
    );
    assert!(
        found.failures.is_empty(),
        "{} failures: {:#?}",
        found.failures.len(),
        &found.failures[..found.failures.len().min(20)]
    );
    assert_eq!(
        found.compared, 155,
        "the feature occurrences of the Molang pairs file"
    );
    assert_eq!(
        found.files, 87,
        "the files of those occurrences in the Molang pairs file"
    );
}
