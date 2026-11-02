//! The committed module catalog and core tables against the facts 13 records for
//! v1.26.50.4.

use bedrock_addon::families::script::meta::{self, Channel, ModuleShape, ModuleVersion, PeerReq};
use bedrock_addon::vanilla;
use bedrock_addon::version::SemVersion;

fn version_text(version: &ModuleVersion) -> String {
    SemVersion::from(version).to_string()
}

fn versions(module: &str) -> Vec<String> {
    meta::module(module)
        .unwrap()
        .versions
        .iter()
        .map(version_text)
        .collect()
}

fn find_version(module: &str, version: &str) -> &'static ModuleVersion {
    meta::module(module)
        .unwrap()
        .versions
        .iter()
        .find(|v| version_text(v) == version)
        .unwrap_or_else(|| panic!("{module} {version}"))
}

fn peer_summary(peers: &[PeerReq]) -> Vec<(String, Vec<String>, bool, Option<String>)> {
    peers
        .iter()
        .map(|p| {
            (
                p.module.to_owned(),
                p.versions.iter().map(|v| (*v).to_owned()).collect(),
                p.types_only,
                p.wrapper_name.map(str::to_owned),
            )
        })
        .collect()
}

#[test]
fn the_catalog_carries_the_published_metadata_tag() {
    assert_eq!(meta::samples_tag(), "v1.26.50.4");
}

#[test]
fn eleven_modules_sorted_by_name_with_their_uuids() {
    //.
    let modules: Vec<(&str, String)> = meta::modules()
        .iter()
        .map(|m| (m.name, uuid::Uuid::from_u128(m.uuid).to_string()))
        .collect();
    assert_eq!(
        modules,
        [
            ("@minecraft/common", "77ec12b4-1b2b-4c98-8d34-d1cd63f849d5"),
            (
                "@minecraft/debug-utilities",
                "1796ea86-0daf-4409-99ee-fd6467cf1203"
            ),
            (
                "@minecraft/diagnostics",
                "b7b26c0f-2267-4713-8534-51ca98c51ee9"
            ),
            ("@minecraft/server", "b26a4d4c-afdf-4690-88f8-931846312678"),
            (
                "@minecraft/server-admin",
                "53d7f2bf-bf9c-49c4-ad1f-7c803d947920"
            ),
            (
                "@minecraft/server-bindings",
                "0196a2fa-8fb5-79a7-b29e-ce3cbc644f82"
            ),
            (
                "@minecraft/server-gametest",
                "6f4b6893-1bb6-42fd-b458-7fa3d0c89616"
            ),
            (
                "@minecraft/server-graphics",
                "3c7acfd1-b8e2-48a3-98af-cfeb489d1457"
            ),
            (
                "@minecraft/server-net",
                "777b1798-13a6-401c-9cba-0cf17e31a81b"
            ),
            (
                "@minecraft/server-ui",
                "2bd50a27-ab5f-4f40-a596-3641627c635e"
            ),
            (
                "@minecraft/server-ui-bindings",
                "ceded7ee-4d9d-4bcc-badf-0ecddb8e238a"
            ),
        ]
        .map(|(name, uuid)| (name, uuid.to_owned()))
    );
}

#[test]
fn modules_are_found_by_name_and_by_uuid() {
    assert_eq!(
        meta::module("@minecraft/server-net").map(|m| m.name),
        Some("@minecraft/server-net")
    );
    assert!(meta::module("@minecraft/server-editor").is_none());
    assert!(
        meta::module("@Minecraft/Server").is_none(),
        "names compare exactly"
    );
    // The only upper-case UUID in the files is found from either case.
    let upper = uuid::Uuid::parse_str("2BD50A27-AB5F-4F40-A596-3641627C635E").unwrap();
    assert_eq!(
        meta::module_by_uuid(upper).map(|m| m.name),
        Some("@minecraft/server-ui")
    );
    assert!(meta::module_by_uuid(uuid::Uuid::nil()).is_none());
}

#[test]
fn shipped_versions_per_module() {
    //, column v1.26.50.4.
    let mut server: Vec<String> = (0..=19).map(|minor| format!("1.{minor}.0")).collect();
    server.extend((0..=10).map(|minor| format!("2.{minor}.0")));
    server.push("2.11.0-beta".to_owned());
    assert_eq!(versions("@minecraft/server"), server);
    assert_eq!(versions("@minecraft/server-bindings"), server);
    let ui = [
        "1.0.0",
        "1.1.0",
        "1.2.0",
        "1.3.0",
        "2.0.0",
        "2.1.0",
        "2.2.0",
        "2.3.0-beta",
    ];
    assert_eq!(versions("@minecraft/server-ui"), ui);
    assert_eq!(versions("@minecraft/server-ui-bindings"), ui);
    assert_eq!(
        versions("@minecraft/common"),
        ["1.0.0", "1.1.0", "1.2.0", "1.3.0"]
    );
    assert_eq!(
        versions("@minecraft/server-graphics"),
        ["1.0.0", "1.1.0-beta"]
    );
    for module in [
        "@minecraft/server-admin",
        "@minecraft/server-gametest",
        "@minecraft/server-net",
        "@minecraft/debug-utilities",
        "@minecraft/diagnostics",
    ] {
        assert_eq!(versions(module), ["1.0.0-beta"], "{module}");
    }
    let total: usize = meta::modules().iter().map(|m| m.versions.len()).sum();
    assert_eq!(total, 91);
}

#[test]
fn versions_are_in_version_order_with_one_beta_last() {
    for module in meta::modules() {
        let sems: Vec<SemVersion> = module.versions.iter().map(SemVersion::from).collect();
        assert!(
            sems.windows(2).all(|pair| pair[0] < pair[1]),
            "{}",
            module.name
        );
        let betas: Vec<usize> = module
            .versions
            .iter()
            .enumerate()
            .filter(|(_, v)| v.channel == Channel::Beta)
            .map(|(i, _)| i)
            .collect();
        assert!(betas.len() <= 1, "{}", module.name);
        if let Some(&beta) = betas.first() {
            assert_eq!(beta, module.versions.len() - 1, "{}", module.name);
            assert_eq!(module.versions[beta].pre, Some("beta"));
        }
    }
}

#[test]
fn stubs_point_at_the_same_version_of_their_bindings() {
    // example: `server` 2.11.0-beta has parent index 31.
    let server = meta::module("@minecraft/server").unwrap();
    assert_eq!(
        server.shape,
        ModuleShape::Stub {
            parent: "@minecraft/server-bindings"
        }
    );
    let beta = find_version("@minecraft/server", "2.11.0-beta");
    assert_eq!(beta.parent_version, Some(31));
    assert_eq!(beta.wrapper, Some("@minecraft/server-bindings-wrapper"));
    for stub in meta::modules() {
        let ModuleShape::Stub { parent } = stub.shape else {
            continue;
        };
        let parent = meta::module(parent).unwrap();
        for version in stub.versions {
            let index = usize::from(version.parent_version.unwrap());
            assert_eq!(version_text(&parent.versions[index]), version_text(version));
        }
    }
}

#[test]
fn wrapper_modules_follow_the_major() {
    // stub table.
    assert_eq!(
        find_version("@minecraft/server", "1.19.0").wrapper,
        Some("@minecraft/server-bindings-v1-wrapper")
    );
    assert_eq!(
        find_version("@minecraft/server", "2.10.0").wrapper,
        Some("@minecraft/server-bindings-v2-wrapper")
    );
    assert_eq!(
        find_version("@minecraft/server-ui", "1.3.0").wrapper,
        Some("@minecraft/server-ui-bindings-v1-wrapper")
    );
    assert_eq!(
        find_version("@minecraft/server-ui", "2.3.0-beta").wrapper,
        Some("@minecraft/server-ui-bindings-wrapper")
    );
    assert_eq!(
        find_version("@minecraft/server-bindings", "2.10.0").wrapper,
        None
    );
}

#[test]
fn peers_follow_13() {
    let common = |v: &str| {
        (
            "@minecraft/common".to_owned(),
            vec![v.to_owned()],
            false,
            None,
        )
    };
    let vanilla_data = (
        "@minecraft/vanilla-data".to_owned(),
        vec!["1.20.70".to_owned()],
        true,
        None,
    );
    let server = |versions: &[&str], wrapper: Option<&str>| {
        (
            "@minecraft/server".to_owned(),
            versions.iter().map(|v| (*v).to_owned()).collect(),
            false,
            wrapper.map(str::to_owned),
        )
    };
    // `server-bindings` ≤ 1.17.0: common 1.2.0; from 1.18.0 also vanilla-data; the stub
    // inherits them.
    assert_eq!(
        peer_summary(find_version("@minecraft/server", "1.17.0").peers),
        [common("1.2.0")]
    );
    assert_eq!(
        peer_summary(find_version("@minecraft/server", "1.18.0").peers),
        [common("1.2.0"), vanilla_data.clone()]
    );
    // rule 5: `server-ui 2.2.0` peers common [1.0.0] and server [2.0.0] with the
    // stub's `wrapperName`.
    assert_eq!(
        peer_summary(find_version("@minecraft/server-ui", "2.2.0").peers),
        [
            common("1.0.0"),
            server(&["2.0.0"], Some("@minecraft/server-wrapper"))
        ]
    );
    assert_eq!(
        peer_summary(find_version("@minecraft/server-ui-bindings", "2.2.0").peers),
        [common("1.0.0"), server(&["2.0.0"], None)]
    );
    // The beta stub's own `server [2.0.0]` replaces its bindings' `server [2.0.0,
    // 2.11.0-beta]` the same way.
    assert_eq!(
        peer_summary(find_version("@minecraft/server-ui-bindings", "2.3.0-beta").peers),
        [common("1.0.0"), server(&["2.0.0", "2.11.0-beta"], None)]
    );
    assert_eq!(
        peer_summary(find_version("@minecraft/server-ui", "2.3.0-beta").peers),
        [
            common("1.0.0"),
            server(&["2.0.0"], Some("@minecraft/server-wrapper"))
        ]
    );
    // example: `server-net`.
    assert_eq!(
        peer_summary(find_version("@minecraft/server-net", "1.0.0-beta").peers),
        [
            common("1.0.0"),
            server(&["1.17.0", "2.0.0"], None),
            (
                "@minecraft/server-admin".to_owned(),
                vec!["1.0.0-beta".to_owned()],
                false,
                None
            )
        ]
    );
}

#[test]
fn a_shipped_version_converts_to_a_sem_version() {
    let beta = find_version("@minecraft/server", "2.11.0-beta");
    assert_eq!(
        SemVersion::from(beta),
        SemVersion::parse("2.11.0-beta").unwrap()
    );
    assert!(SemVersion::from(beta).is_pre_release());
    let release = find_version("@minecraft/common", "1.2.0");
    assert_eq!(SemVersion::from(release), SemVersion::new(1, 2, 0));
}

#[test]
fn engine_only_versions_are_listed() {
    // `ENGINE_ONLY` (v1.26.50.4).
    assert_eq!(
        meta::engine_only(),
        [
            ("@minecraft/server", "3.0.0-alpha"),
            ("@minecraft/server", "2.11.0-internal"),
            ("@minecraft/server-editor-bindings", "0.1.0-beta"),
        ]
    );
}

#[test]
fn the_identifier_lists_have_the_counts_of_the_tag() {
    // vanilla data table, column v1.26.50.4.
    let ids = vanilla::ids();
    let counts = [
        ids.blocks.len(),
        ids.items.len(),
        ids.entities.len(),
        ids.biomes.len(),
        ids.effects.len(),
        ids.enchantments.len(),
        ids.dimensions.len(),
        ids.camera_presets.len(),
        ids.cooldown_categories.len(),
        ids.features.len(),
        ids.potion_effects.len(),
        ids.potion_types.len(),
    ];
    // Blocks: the file's 1463, the 14 registered blocks it leaves out (K121) and the 4 of the
    // 1.26.60 binary's list it lacks; items: the file's 1623 and the binary's 365
    // more (items and block items); potions: the file's 47 and `freezing`, `long_freezing`.
    assert_eq!(counts, [1481, 1988, 129, 89, 37, 42, 3, 7, 6, 25, 49, 3]);
    assert_eq!(ids.blocks[1462], "minecraft:zombie_head");
    assert_eq!(ids.blocks[1463], "minecraft:chalkboard");
    assert_eq!(
        ids.enchantment_aliases,
        [("minecraft:bow_infinity", "minecraft:infinity")]
    );
    assert_eq!(
        ids.potion_types,
        ["Consume", "ThrownLingering", "ThrownSplash"]
    );
    assert!(ids.dimensions.contains(&"minecraft:overworld"));
    assert!(ids.effects.contains(&"minecraft:regeneration"));
}

#[test]
fn the_command_table_has_the_counts_and_aliases_of_the_tag() {
    //: 84 commands, 279 overloads, 236 enums (v1.26.50.4); permission levels
    // 1 → 61, 4 → 10, 2 → 7, 0 → 5, 3 → 1; 70 require cheats.
    let commands = vanilla::commands();
    assert_eq!(commands.len(), 84);
    let overloads: usize = commands.iter().map(|c| c.overloads.len()).sum();
    assert_eq!(overloads, 279);
    assert_eq!(vanilla::command_enums().len(), 236);
    let mut levels = [0_usize; 5];
    for command in commands {
        levels[usize::from(command.permission_level)] += 1;
    }
    assert_eq!(levels, [5, 61, 7, 1, 10]);
    assert_eq!(commands.iter().filter(|c| c.requires_cheats).count(), 70);
    let aliases: Vec<(&str, &[&str])> = commands
        .iter()
        .filter(|c| !c.aliases.is_empty())
        .map(|c| (c.name, c.aliases))
        .collect();
    assert_eq!(
        aliases,
        [
            ("allowlist", &["whitelist"][..]),
            ("daylock", &["alwaysday"][..]),
            ("help", &["?"][..]),
            ("permission", &["ops"][..]),
            ("teleport", &["tp"][..]),
            ("tell", &["w", "msg"][..]),
            ("wsserver", &["connect"][..]),
        ]
    );
    assert_eq!(vanilla::command("tp").map(|c| c.name), Some("teleport"));
    assert!(commands.windows(2).all(|pair| pair[0].name < pair[1].name));
}
