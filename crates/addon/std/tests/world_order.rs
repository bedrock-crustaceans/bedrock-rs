//! The smoke tests' world list (`common::world::base_game_world`): every pack of a vanilla
//! install, with the base-game layer in the order the game stacks its base-game packs. Needs no
//! game files: the install is a handful of manifests in a temporary directory.

mod common;

use std::fs;
use std::path::Path;

use bedrock_addon::load::{Experiment, LoadOptions};
use bedrock_addon::pack::{PackOrigin, PackRepository};
use bedrock_addon_std::{DirSource, Driver, Options, Strategy};

use common::world::base_game_world;

const VANILLA: &str = "fe9f8597-5454-481a-8730-8d070a8e2e58";
const VANILLA_RESOURCE: &str = "0575c61f-a5da-4b7f-9961-ffda2908861e";
const SERVER_LIBRARY: &str = "37ca4282-69e9-4402-b40b-0b07de579f02";
const SERVER_UI_LIBRARY: &str = "f53f490f-9d47-45ca-86de-0946b6d1c366";
const EXPERIMENTAL_POI: &str = "32dd7cc5-e6fb-46f3-ace9-15aa250989a6";
const EXPERIMENTAL_CAMERAS: &str = "05112080-727b-4ed4-98d4-b099fc75f2be";
const CHEMISTRY: &str = "6baf8b62-8948-4c99-bb1e-a0cb35dc4579";
const CHEMISTRY_RESOURCE: &str = "0fba4063-dba1-4281-9b89-ff9390653530";
const OTHER_RESOURCE: &str = "5c1b1a3e-0b3c-4b7e-9a57-6f1f0d2c8e11";

/// A manifest of `module` type (`data` or `resources`) named `name`, with an optional
/// dependency on the pack `depends_on` at 0.0.1.
fn manifest(
    name: &str,
    uuid: &str,
    version: [u16; 3],
    module: &str,
    depends_on: Option<&str>,
) -> String {
    let [major, minor, patch] = version;
    let dependencies = depends_on.map_or(String::new(), |uuid| {
        format!(r#", "dependencies": [{{"uuid": "{uuid}", "version": [0, 0, 1]}}]"#)
    });
    // The module's UUID is the pack's with its first digit changed.
    let module_uuid = format!("a{}", &uuid[1..]);
    format!(
        r#"{{"format_version": 2, "header": {{"name": "{name}", "description": "", "uuid": "{uuid}", "version": [{major}, {minor}, {patch}], "min_engine_version": [1, 26, 0]}}, "modules": [{{"type": "{module}", "uuid": "{module_uuid}", "version": [1, 0, 0]}}]{dependencies}}}"#
    )
}

fn write(dir: &Path, folder: &str, manifest: &str) {
    let path = dir.join(folder).join("manifest.json");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, manifest).unwrap();
}

/// A vanilla install whose folders sort in none of the orders the stack has: the behaviour
/// packs `vanilla` (which depends on its resource pack), two slices, both script libraries,
/// two experimental packs and `chemistry` (with its resource pack); the resource packs
/// `vanilla`, `chemistry` and one nothing depends on.
fn install(root: &Path) -> (Driver, PackRepository) {
    install_with_poi_in(root, "experimental_poi")
}

/// [`install`] with the POI pack in the folder `poi_folder` (and so named so).
fn install_with_poi_in(root: &Path, poi_folder: &str) -> (Driver, PackRepository) {
    let behavior = root.join("behavior_packs");
    let resource = root.join("resource_packs");
    let data = |name: &str, uuid: &str, version: [u16; 3], depends_on: Option<&str>| {
        write(
            &behavior,
            name,
            &manifest(name, uuid, version, "data", depends_on),
        );
    };
    data("chemistry", CHEMISTRY, [0, 0, 1], Some(CHEMISTRY_RESOURCE));
    data(
        "experimental_cameras",
        EXPERIMENTAL_CAMERAS,
        [0, 0, 1],
        None,
    );
    data(poi_folder, EXPERIMENTAL_POI, [0, 0, 1], None);
    data("server_library", SERVER_LIBRARY, [0, 0, 1], None);
    data("server_ui_library", SERVER_UI_LIBRARY, [0, 0, 1], None);
    data("vanilla", VANILLA, [0, 0, 1], Some(VANILLA_RESOURCE));
    data("vanilla_1.19.0", VANILLA, [1, 19, 0], None);
    data("vanilla_1.20.50", VANILLA, [1, 20, 50], None);
    let resources = |name: &str, uuid: &str| {
        write(
            &resource,
            name,
            &manifest(name, uuid, [0, 0, 1], "resources", None),
        );
    };
    resources("a_other", OTHER_RESOURCE);
    resources("chemistry", CHEMISTRY_RESOURCE);
    resources("vanilla", VANILLA_RESOURCE);
    let mut driver = Driver::new(Options::default().with_strategy(Strategy::Sequential)).unwrap();
    let repository = driver
        .repository([
            DirSource::packs(behavior, PackOrigin::Package),
            DirSource::packs(resource, PackOrigin::Package),
        ])
        .unwrap();
    (driver, repository)
}

/// The names of the packs of both stacks of the world the helper builds, top first.
fn stack_names(repository: &PackRepository, options: &LoadOptions) -> (Vec<String>, Vec<String>) {
    let stacks = repository.stacks(&base_game_world(repository, options), options.clone());
    let names = |packs: Vec<bedrock_addon::pack::PackRef>| {
        packs
            .iter()
            .map(|pack| format!("{} {}", pack.name(), pack.id().version))
            .collect::<Vec<_>>()
    };
    (
        names(stacks.behavior.packs().collect()),
        names(stacks.resource.packs().collect()),
    )
}

/// The server's behaviour stack: the experiment packs whose experiment is on, the two
/// script libraries, the slices newest first, `vanilla`. The packs vanilla's layer does not
/// hold (an experimental pack whose experiment is off, `chemistry`) are the world's list, in
/// the repository's order, ABOVE the layer: every pack is loaded, and those packs can shadow
/// the layer (in the game a pack whose experiment is off is not loaded, so never wins).
#[test]
fn the_world_lists_the_base_game_layer_in_vanillas_order_and_then_the_other_packs() {
    let root = tempfile::tempdir().unwrap();
    let (_driver, repository) = install(root.path());
    let options = LoadOptions {
        experiments: [Experiment::Poi].into_iter().collect(),
        ..LoadOptions::default()
    };
    let (behavior, resource) = stack_names(&repository, &options);
    assert_eq!(
        behavior,
        [
            "chemistry 0.0.1",
            "experimental_cameras 0.0.1",
            "experimental_poi 0.0.1",
            "server_library 0.0.1",
            "server_ui_library 0.0.1",
            "vanilla_1.20.50 1.20.50",
            "vanilla_1.19.0 1.19.0",
            "vanilla 0.0.1",
        ]
    );
    // The resource packs the world's behaviour packs depend on, then the rest. (`vanilla`'s
    // resource pack is a base-game behaviour pack's dependency, which the server's stack does
    // not pull in.)
    assert_eq!(
        resource,
        ["chemistry 0.0.1", "a_other 0.0.1", "vanilla 0.0.1"]
    );
}

/// Which experimental packs head the base-game layer follows the world's experiments, in
/// vanilla's order of them (cameras before POI), whatever order the repository registered
/// them in: here the POI pack's folder sorts first.
#[test]
fn the_experiments_of_the_world_decide_which_experimental_packs_lead() {
    let root = tempfile::tempdir().unwrap();
    let (_driver, repository) = install_with_poi_in(root.path(), "a_experimental_poi");
    let registered: Vec<String> = repository
        .packs()
        .map(|pack| pack.id().uuid.to_string())
        .filter(|uuid| [EXPERIMENTAL_POI, EXPERIMENTAL_CAMERAS].contains(&uuid.as_str()))
        .collect();
    assert_eq!(
        registered,
        [EXPERIMENTAL_POI, EXPERIMENTAL_CAMERAS],
        "the repository registers POI first"
    );
    let both = LoadOptions {
        experiments: [Experiment::Poi, Experiment::ExperimentalCreatorCameras]
            .into_iter()
            .collect(),
        ..LoadOptions::default()
    };
    let (behavior, _) = stack_names(&repository, &both);
    assert_eq!(
        behavior[..4],
        [
            "chemistry 0.0.1",
            "experimental_cameras 0.0.1",
            "a_experimental_poi 0.0.1",
            "server_library 0.0.1"
        ]
    );
    let (behavior, _) = stack_names(&repository, &LoadOptions::default());
    assert_eq!(
        behavior,
        [
            "a_experimental_poi 0.0.1",
            "chemistry 0.0.1",
            "experimental_cameras 0.0.1",
            "server_library 0.0.1",
            "server_ui_library 0.0.1",
            "vanilla_1.20.50 1.20.50",
            "vanilla_1.19.0 1.19.0",
            "vanilla 0.0.1",
        ]
    );
    // Off, both are outside the layer and listed in the repository's order, POI first.
    let world = base_game_world(&repository, &both);
    assert_eq!(
        world.behavior.len(),
        1,
        "only chemistry is outside the layer"
    );
}

/// Every pack of the repository is on a stack, once: the list is for loading every file.
#[test]
fn every_pack_of_the_repository_is_listed_once() {
    let root = tempfile::tempdir().unwrap();
    let (_driver, repository) = install(root.path());
    let options = LoadOptions::default();
    let stacks = repository.stacks(&base_game_world(&repository, &options), options);
    let mut listed: Vec<String> = stacks
        .behavior
        .packs()
        .chain(stacks.resource.packs())
        .map(|pack| format!("{} {}", pack.id().uuid, pack.id().version))
        .collect();
    listed.sort();
    let mut registered: Vec<String> = repository
        .packs()
        .map(|pack| format!("{} {}", pack.id().uuid, pack.id().version))
        .collect();
    registered.sort();
    assert_eq!(listed, registered);
}

const OREUI: &str = "fdcdb085-9f06-464f-bf0e-25c0b37ff4a5";
const VANILLA_BASE: &str = "9ae3d2cc-7811-404c-b578-cb27689c8ac5";
const PERSONA: &str = "2099de18-429a-465a-a49b-fc4710a17bb3";
const VANILLA_MUSIC: &str = "6989c411-4355-4756-9163-51c1df5ef677";
const VANILLA_TRIAL: &str = "bd264cd8-8969-4e5e-865a-21096f5ad838";
const PLATFORM_DESKTOP: &str = "133464ef-3b92-44ed-8546-ec6473265119";
const PLATFORM_XBOX: &str = "1feb601d-6865-431f-a4bb-17e63d9248a0";

/// The client's resource stack: after the packs the world's behaviour packs pull in and
/// the packs on no base-game stack (in the repository's order; chemistry is one), the layer: the experiment packs in vanilla's order of them, `vanilla_music`,
/// the platform packs, `vanilla_trial`, the resource slices newest first, `oreui`, `vanilla`,
/// `vanilla_base`, `persona`. The folders here sort in another order throughout.
#[test]
fn the_resource_packs_are_in_the_client_base_game_order() {
    let root = tempfile::tempdir().unwrap();
    let resource = root.path().join("resource_packs");
    for (name, uuid, version) in [
        ("oreui", OREUI, [0, 0, 1]),
        ("persona", PERSONA, [0, 0, 1]),
        ("vanilla_1.14", VANILLA_RESOURCE, [1, 14, 0]),
        ("vanilla_1.21.100", VANILLA_RESOURCE, [1, 21, 100]),
        ("vanilla_1.21.20", VANILLA_RESOURCE, [1, 21, 20]),
        ("vanilla_base", VANILLA_BASE, [0, 0, 1]),
        ("vanilla_music", VANILLA_MUSIC, [0, 0, 1]),
        ("b_trial", VANILLA_TRIAL, [0, 0, 1]),
        ("c_platform_xbox", PLATFORM_XBOX, [0, 0, 1]),
        ("d_platform_desktop", PLATFORM_DESKTOP, [0, 0, 1]),
        (
            "zz_chemistry_1.20.50",
            "0674721c-a0aa-41a1-9ba8-1ed33ea3e7ed",
            [1, 20, 50],
        ),
    ] {
        write(
            &resource,
            name,
            &manifest(name, uuid, version, "resources", None),
        );
    }
    // Two experiment packs of vanilla's table, registered against its order.
    let experimental: Vec<String> = bedrock_addon::pack::VANILLA_EXPERIMENTAL_PACKS
        .iter()
        .filter(|(_, pack_type, _)| *pack_type == bedrock_addon::pack::PackType::Resource)
        .take(2)
        .map(|(uuid, _, _)| uuid.to_string())
        .collect();
    assert_eq!(experimental.len(), 2);
    write(
        &resource,
        "y_experimental_first",
        &manifest(
            "y_experimental_first",
            &experimental[0],
            [0, 0, 1],
            "resources",
            None,
        ),
    );
    write(
        &resource,
        "x_experimental_second",
        &manifest(
            "x_experimental_second",
            &experimental[1],
            [0, 0, 1],
            "resources",
            None,
        ),
    );
    let (_driver, repository) = install(root.path());
    let (_, resource) = stack_names(&repository, &LoadOptions::default());
    assert_eq!(
        resource,
        [
            "chemistry 0.0.1",
            "a_other 0.0.1",
            "zz_chemistry_1.20.50 1.20.50",
            "y_experimental_first 0.0.1",
            "x_experimental_second 0.0.1",
            "vanilla_music 0.0.1",
            "d_platform_desktop 0.0.1",
            "c_platform_xbox 0.0.1",
            "b_trial 0.0.1",
            "vanilla_1.21.100 1.21.100",
            "vanilla_1.21.20 1.21.20",
            "vanilla_1.14 1.14.0",
            "oreui 0.0.1",
            "vanilla 0.0.1",
            "vanilla_base 0.0.1",
            "persona 0.0.1",
        ]
    );
}
