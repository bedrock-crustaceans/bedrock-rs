//! `ModuleCatalog`: a catalog built by hand, and the one compiled in from the published
//! metadata (`script::meta`), with the BDS probes that have a counterpart on it.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::families::script::{
    CatalogEntry, CatalogModule, Discovery, ModuleCatalog, ScriptContext, ScriptDiscovery,
    ScriptFiles, ScriptModule, ScriptVersion, meta,
};
use bedrock_addon::storage::PackPath;
use uuid::Uuid;

use crate::common::code_name;

fn v(text: &str) -> ScriptVersion {
    ScriptVersion::parse(text).unwrap_or_else(|| panic!("{text} is no version"))
}

fn versions(catalog: &ModuleCatalog, module: &str, hidden: bool) -> Vec<String> {
    catalog
        .module(module)
        .unwrap_or_else(|| panic!("{module} is not in the catalog"))
        .versions
        .iter()
        .filter(|version| version.hidden == hidden)
        .map(|version| version.version.to_string())
        .collect()
}

fn requires(catalog: &ModuleCatalog, module: &str, version: &str) -> Vec<String> {
    catalog
        .module(module)
        .unwrap_or_else(|| panic!("{module} is not in the catalog"))
        .versions
        .iter()
        .find(|entry| entry.version == v(version))
        .unwrap_or_else(|| panic!("{module} has no {version}"))
        .requires
        .iter()
        .map(|(name, at)| format!("{name} {at}"))
        .collect()
}

/// Discovery of a format-2 pack with `dependencies` against `catalog`, without a permission
/// list.
fn discover(catalog: &ModuleCatalog, dependencies: &str, cx: &ScriptContext<'_>) -> Discovery {
    let text = format!(
        r#"{{
  "format_version": 2,
  "header": {{ "name": "p", "uuid": "11111111-1111-1111-1111-111111111111", "version": [1, 0, 0],
              "min_engine_version": [1, 21, 0] }},
  "modules": [ {{ "type": "script", "uuid": "22222222-2222-2222-2222-222222222222", "version": [3, 2, 1],
                 "entry": "scripts/main.js" }} ],
  "dependencies": [{dependencies}]
}}"#
    );
    let module = ScriptModule::parse(
        &PackPath::from_static("manifest.json"),
        text.as_bytes(),
        &bedrock_addon::load::LoadContext::detached(),
    )
    .unwrap_or_else(|rejected| panic!("refused: {:?}", rejected.diagnostics))
    .into_value()
    .expect("a script module");
    ScriptDiscovery::new(catalog).discover(&module, &ScriptFiles::from_members(["main.js"]), cx)
}

fn dependency(module: &str, version: &str) -> String {
    format!(r#"{{ "module_name": "{module}", "version": "{version}" }}"#)
}

fn resolved(found: &Discovery) -> Vec<String> {
    found
        .plugin()
        .unwrap_or_else(|| panic!("the script must run: {:?}", found.diagnostics()))
        .dependencies
        .iter()
        .map(|d| format!("{} {}", d.module_name, d.resolved))
        .collect()
}

fn codes(found: &Discovery) -> Vec<String> {
    found.diagnostics().iter().map(code_name).collect()
}

const BETA: ScriptContext<'static> = ScriptContext {
    beta_apis: true,
    editor_world: false,
    allow_internal: false,
    is_package_origin: false,
    allowed_modules: None,
};

// ---- a catalog built by hand ----

#[test]
fn a_catalog_sorts_its_versions_and_finds_modules_by_name_and_uuid() {
    let uuid = Uuid::from_u128(7);
    let mut catalog = ModuleCatalog::new();
    catalog.insert(
        "@demo/module",
        CatalogModule {
            uuid: Some(uuid),
            bindings: None,
            versions: ["2.0.0", "1.0.0", "2.1.0-beta", "1.5.0"]
                .map(|text| CatalogEntry::new(v(text)))
                .to_vec(),
        },
    );
    assert_eq!(
        versions(&catalog, "@demo/module", false),
        ["1.0.0", "1.5.0", "2.0.0", "2.1.0-beta"]
    );
    assert_eq!(catalog.name_of(uuid), Some("@demo/module"));
    assert_eq!(catalog.name_of(Uuid::from_u128(8)), None);
    assert!(
        catalog.module("@Demo/Module").is_none(),
        "names are compared byte for byte"
    );
    assert_eq!(catalog.modules().count(), 1);

    // Inserting the name again replaces the module and its UUID.
    catalog.insert(
        "@demo/module",
        CatalogModule {
            uuid: Some(Uuid::from_u128(9)),
            bindings: None,
            versions: vec![CatalogEntry::new(v("3.0.0"))],
        },
    );
    assert_eq!(versions(&catalog, "@demo/module", false), ["3.0.0"]);
    assert_eq!(catalog.name_of(uuid), None);
    assert_eq!(catalog.name_of(Uuid::from_u128(9)), Some("@demo/module"));
    assert_eq!(catalog.modules().count(), 1);
}

#[test]
fn a_catalog_module_starts_from_its_default() {
    // No UUID, its own bindings, no version: a literal names only what it sets.
    assert_eq!(
        CatalogModule::default(),
        CatalogModule {
            uuid: None,
            bindings: None,
            versions: Vec::new(),
        }
    );
    let mut catalog = ModuleCatalog::new();
    catalog.insert(
        "@demo/module",
        CatalogModule {
            versions: vec![CatalogEntry::new(v("1.0.0"))],
            ..CatalogModule::default()
        },
    );
    assert_eq!(versions(&catalog, "@demo/module", false), ["1.0.0"]);
    assert_eq!(catalog.module("@demo/module").and_then(|m| m.uuid), None);
}

#[test]
fn an_empty_catalog_knows_no_module() {
    let found = discover(
        &ModuleCatalog::new(),
        &dependency("@minecraft/server", "2.0.0"),
        &ScriptContext::default(),
    );
    assert!(found.plugin().is_none());
    assert_eq!(codes(&found), ["script/plugin_unknown_module"]);
    // Without a dependency a script needs no module.
    let alone = discover(&ModuleCatalog::new(), "", &ScriptContext::default());
    assert!(alone.plugin().is_some());
    assert!(alone.diagnostics().is_empty());
}

#[test]
fn a_requirement_on_a_module_the_catalog_lacks_fails_the_context() {
    // Vanilla: "module [..] depends on unknown module [..]" at context creation.
    let mut catalog = ModuleCatalog::new();
    let mut version = CatalogEntry::new(v("1.0.0"));
    version.requires = vec![("@demo/absent".to_owned(), v("1.0.0"))];
    catalog.insert(
        "@demo/module",
        CatalogModule {
            uuid: Some(Uuid::from_u128(7)),
            bindings: Some("@demo/bindings".to_owned()),
            versions: vec![version],
        },
    );
    let found = discover(
        &catalog,
        &dependency("@demo/module", "1.0.0"),
        &ScriptContext::default(),
    );
    assert!(found.plugin().is_none());
    let texts: Vec<String> = found
        .diagnostics()
        .iter()
        .map(|d| d.code.to_string())
        .collect();
    assert_eq!(
        texts,
        ["Plugin [p] - module [@demo/bindings - 1.0.0] depends on unknown module [@demo/absent]."]
    );
    assert_eq!(
        found.diagnostics().iter().next().map(|d| d.severity),
        Some(Severity::Error)
    );
}

// ---- the compiled-in catalog ----

#[test]
fn the_shipped_catalog_holds_every_module_and_version_of_the_metadata() {
    let catalog = ModuleCatalog::shipped();
    // Every module of the table, and the editor module no table has.
    assert_eq!(catalog.modules().count(), meta::modules().len() + 1);
    for entry in meta::modules() {
        let module = catalog
            .module(entry.name)
            .unwrap_or_else(|| panic!("{} is missing", entry.name));
        assert_eq!(
            module.uuid,
            Some(Uuid::from_u128(entry.uuid)),
            "{}",
            entry.name
        );
        assert_eq!(
            catalog.name_of(Uuid::from_u128(entry.uuid)),
            Some(entry.name)
        );
        for shipped in entry.versions {
            let version = ScriptVersion::with_pre(
                shipped.major,
                shipped.minor,
                shipped.patch,
                shipped.pre.unwrap_or_default(),
            );
            assert!(
                module
                    .versions
                    .iter()
                    .any(|v| v.version == version && !v.hidden),
                "{} {version}",
                entry.name
            );
        }
        assert!(
            module
                .versions
                .windows(2)
                .all(|pair| pair[0].version < pair[1].version),
            "{}: ascending and distinct",
            entry.name
        );
    }
}

#[test]
fn the_shipped_catalog_names_a_stubs_bindings_module() {
    let catalog = ModuleCatalog::shipped();
    let bindings = |module: &str| catalog.module(module).and_then(|m| m.bindings.clone());
    assert_eq!(
        bindings("@minecraft/server").as_deref(),
        Some("@minecraft/server-bindings")
    );
    assert_eq!(
        bindings("@minecraft/server-ui").as_deref(),
        Some("@minecraft/server-ui-bindings")
    );
    assert_eq!(bindings("@minecraft/server-bindings"), None);
    assert_eq!(bindings("@minecraft/server-gametest"), None);
}

#[test]
fn the_shipped_catalog_adds_what_the_engine_serves_beyond_the_metadata() {
    let catalog = ModuleCatalog::shipped();
    let listed = versions(&catalog, "@minecraft/server", false);
    // BDS lists 0.1.0 and the alpha [bds-run]; neither has a metadata file.
    assert_eq!(listed.first().map(String::as_str), Some("0.1.0"));
    assert_eq!(listed.last().map(String::as_str), Some("3.0.0-alpha"));
    assert!(listed.contains(&"2.11.0-beta".to_owned()));
    // The internal version resolves and is not listed.
    assert_eq!(
        versions(&catalog, "@minecraft/server", true),
        ["2.11.0-internal"]
    );
    for (module, version) in meta::engine_only() {
        if let Some(entry) = catalog.module(module) {
            assert!(
                entry.versions.iter().any(|v| v.version == self::v(version)),
                "{module} {version}"
            );
        }
    }
}

#[test]
fn a_shipped_version_requires_what_its_bindings_peers_say() {
    let catalog = ModuleCatalog::shipped();
    // One accepted version: that version. Type-only peers (`vanilla-data`) are no module.
    assert_eq!(
        requires(&catalog, "@minecraft/server", "2.0.0"),
        ["@minecraft/common 1.2.0"]
    );
    assert_eq!(
        requires(&catalog, "@minecraft/server-ui", "1.3.0"),
        ["@minecraft/common 1.0.0", "@minecraft/server 1.3.0"]
    );
    assert_eq!(
        requires(&catalog, "@minecraft/server-ui", "2.2.0"),
        ["@minecraft/common 1.0.0", "@minecraft/server 2.0.0"]
    );
    assert_eq!(
        requires(&catalog, "@minecraft/server-graphics", "1.0.0"),
        ["@minecraft/common 1.0.0", "@minecraft/server 2.3.0"]
    );
    // A stub takes its bindings module's peers, where the beta names the server beta
    // [bds-run B09]: several accepted versions in one major are the newest of them.
    assert_eq!(
        requires(&catalog, "@minecraft/server-ui", "2.3.0-beta"),
        ["@minecraft/common 1.0.0", "@minecraft/server 2.11.0-beta"]
    );
    // Accepted versions in several majors bind to none [bds-run B07].
    assert_eq!(
        requires(&catalog, "@minecraft/server-gametest", "1.0.0-beta"),
        ["@minecraft/common 1.0.0"]
    );
    assert_eq!(
        requires(&catalog, "@minecraft/server-admin", "1.0.0-beta"),
        ["@minecraft/common 1.0.0"]
    );
    // What the engine serves beyond the metadata requires nothing we know of.
    assert_eq!(
        requires(&catalog, "@minecraft/server", "0.1.0"),
        [] as [&str; 0]
    );
}

#[test]
fn every_shipped_requirement_names_a_shipped_module() {
    let catalog = ModuleCatalog::shipped();
    let mut count = 0;
    for (name, module) in catalog.modules() {
        for version in &module.versions {
            for (required, _) in &version.requires {
                count += 1;
                assert!(
                    catalog.module(required).is_some(),
                    "{name} {} requires {required}",
                    version.version
                );
            }
        }
    }
    assert!(count > 100, "the requirements were read: {count}");
}

#[test]
fn every_shipped_version_resolves_when_it_is_the_only_request() {
    let catalog = ModuleCatalog::shipped();
    let cx = ScriptContext {
        beta_apis: true,
        allow_internal: true,
        editor_world: true,
        ..ScriptContext::default()
    };
    let mut count = 0;
    for (name, module) in catalog.modules() {
        for version in &module.versions {
            let found = discover(
                &catalog,
                &dependency(name, &version.version.to_string()),
                &cx,
            );
            assert!(
                !found.diagnostics().has_errors(),
                "{name} {}: {:?}",
                version.version,
                found.diagnostics()
            );
            assert_eq!(resolved(&found).len(), 1);
            count += 1;
        }
    }
    assert!(count > 80, "{count} versions");
}

#[test]
fn on_the_shipped_catalog_a_release_is_served_by_the_newest_of_its_major() {
    let catalog = ModuleCatalog::shipped();
    let found = discover(
        &catalog,
        &dependency("@minecraft/server", "2.0.0"),
        &ScriptContext::default(),
    );
    assert_eq!(resolved(&found), ["@minecraft/server 2.10.0"]);
    assert!(found.diagnostics().is_empty());
    let found = discover(
        &catalog,
        &dependency("@minecraft/server", "1.5.1"),
        &ScriptContext::default(),
    );
    assert_eq!(resolved(&found), ["@minecraft/server 1.19.0"]);
    let beta = discover(&catalog, &dependency("@minecraft/server", "beta"), &BETA);
    assert_eq!(resolved(&beta), ["@minecraft/server 2.11.0-beta"]);
    let ui = discover(
        &catalog,
        &format!(
            "{}, {}",
            dependency("@minecraft/server", "1.0.0"),
            dependency("@minecraft/server-ui", "1.3.0")
        ),
        &ScriptContext::default(),
    );
    assert_eq!(
        resolved(&ui),
        ["@minecraft/server 1.19.0", "@minecraft/server-ui 1.3.0"]
    );
}

#[test]
fn on_the_shipped_catalog_the_uuid_dependency_is_0_1_0() {
    let catalog = ModuleCatalog::shipped();
    let found = discover(
        &catalog,
        r#"{ "uuid": "b26a4d4c-afdf-4690-88f8-931846312678", "version": [0, 1, 0] }"#,
        &ScriptContext::default(),
    );
    assert_eq!(codes(&found), ["script/plugin_dependency_by_uuid"]);
    assert_eq!(resolved(&found), ["@minecraft/server 0.1.0"]);
}

#[test]
fn on_the_shipped_catalog_the_server_ui_uuid_dependency_is_0_1_0() {
    let catalog = ModuleCatalog::shipped();
    assert_eq!(
        versions(&catalog, "@minecraft/server-ui", false)
            .first()
            .map(String::as_str),
        Some("0.1.0")
    );
    assert_eq!(
        requires(&catalog, "@minecraft/server-ui", "0.1.0"),
        [] as [&str; 0]
    );
    let found = discover(
        &catalog,
        r#"{ "uuid": "2bd50a27-ab5f-4f40-a596-3641627c635e", "version": [0, 1, 0] }"#,
        &ScriptContext::default(),
    );
    assert_eq!(codes(&found), ["script/plugin_dependency_by_uuid"]);
    assert_eq!(resolved(&found), ["@minecraft/server-ui 0.1.0"]);
    // By name the 0.1.0 is behind the Beta APIs gate like every major 0.
    let named = discover(
        &catalog,
        &dependency("@minecraft/server-ui", "0.1.0"),
        &ScriptContext::default(),
    );
    assert_eq!(codes(&named), ["script/plugin_beta_apis"]);
    let named = discover(
        &catalog,
        &dependency("@minecraft/server-ui", "0.1.0"),
        &BETA,
    );
    assert_eq!(resolved(&named), ["@minecraft/server-ui 0.1.0"]);
}

#[test]
fn the_shipped_catalog_serves_the_editor_module() {
    let catalog = ModuleCatalog::shipped();
    assert_eq!(
        versions(&catalog, "@minecraft/server-editor", false),
        ["0.1.0-beta"]
    );
    // The internal version resolves and is not listed, like `@minecraft/server`'s.
    assert_eq!(
        versions(&catalog, "@minecraft/server-editor", true),
        ["0.1.0-internal"]
    );
    // No document has the module's UUID: it is nil, and no UUID finds the module.
    let module = catalog
        .module("@minecraft/server-editor")
        .expect("the module");
    assert_eq!(module.uuid, None);
    assert_eq!(catalog.name_of(Uuid::nil()), None);
    assert_eq!(module.bindings, None);
}

#[test]
fn on_the_shipped_catalog_vanillas_editor_packs_resolve_in_an_editor_world() {
    let catalog = ModuleCatalog::shipped();
    // The dependencies of vanilla's `editor` pack, a pack bundled with the game.
    let editor = format!(
        "{}, {}",
        dependency("@minecraft/server-editor", "beta"),
        dependency("@minecraft/server", "beta")
    );
    let package = ScriptContext {
        editor_world: true,
        is_package_origin: true,
        ..ScriptContext::default()
    };
    let found = discover(&catalog, &editor, &package);
    assert_eq!(codes(&found), [] as [&str; 0]);
    assert_eq!(
        resolved(&found),
        [
            "@minecraft/server-editor 0.1.0-beta",
            "@minecraft/server 2.11.0-beta"
        ]
    );
    // `editor-internal` of the GDK asks for the internal version.
    let internal = discover(
        &catalog,
        &dependency("@minecraft/server-editor", "0.1.0-internal"),
        &package,
    );
    assert_eq!(codes(&internal), [] as [&str; 0]);
    assert_eq!(
        resolved(&internal),
        ["@minecraft/server-editor 0.1.0-internal"]
    );
    // The editor module has no UUID to hand on; `@minecraft/server` has.
    let uuids: Vec<(String, Option<Uuid>)> = found
        .plugin()
        .expect("the script runs")
        .dependencies
        .iter()
        .map(|d| (d.module_name.clone(), d.module_uuid))
        .collect();
    assert_eq!(
        uuids,
        [
            ("@minecraft/server-editor".to_owned(), None),
            (
                "@minecraft/server".to_owned(),
                catalog.module("@minecraft/server").and_then(|m| m.uuid)
            ),
        ]
    );
    assert!(uuids[1].1.is_some());

    // A pack that is not bundled meets the Beta APIs gate in an editor world.
    let world = ScriptContext {
        editor_world: true,
        ..ScriptContext::default()
    };
    assert_eq!(
        codes(&discover(&catalog, &editor, &world)),
        ["script/plugin_beta_apis", "script/plugin_beta_apis"]
    );
    let beta = ScriptContext {
        beta_apis: true,
        ..world
    };
    assert_eq!(
        resolved(&discover(&catalog, &editor, &beta)),
        [
            "@minecraft/server-editor 0.1.0-beta",
            "@minecraft/server 2.11.0-beta"
        ]
    );
    // Outside an editor world the dependency is dropped as before.
    let outside = discover(&catalog, &editor, &BETA);
    assert_eq!(codes(&outside), ["script/plugin_not_editor_world"]);
    assert_eq!(resolved(&outside), ["@minecraft/server 2.11.0-beta"]);
}

#[test]
fn on_the_shipped_catalog_the_probes_with_a_transitive_request_come_out_alike() {
    let catalog = ModuleCatalog::shipped();
    let both = |a: (&str, &str), b: (&str, &str)| {
        discover(
            &catalog,
            &format!("{}, {}", dependency(a.0, a.1), dependency(b.0, b.1)),
            &BETA,
        )
    };
    // Q34: server 1.19.0 and server-ui 2.0.0, whose bindings need server 2.0.0.
    let q34 = both(
        ("@minecraft/server", "1.19.0"),
        ("@minecraft/server-ui", "2.0.0"),
    );
    assert!(q34.plugin().is_none());
    assert!(
        codes(&q34)
            .iter()
            .all(|code| code == "core/script_module_version_conflict"),
        "{:?}",
        codes(&q34)
    );
    // B09: server 1.19.0 and the server-ui beta, whose bindings need the server beta.
    let b09 = both(
        ("@minecraft/server", "1.19.0"),
        ("@minecraft/server-ui", "beta"),
    );
    let texts: Vec<String> = b09
        .diagnostics()
        .iter()
        .map(|d| d.code.to_string())
        .collect();
    assert_eq!(
        texts,
        [
            "Plugin [p] - version conflict for module [@minecraft/server].\n [1.19.0] requested by [p - 3.2.1],\n [2.11.0-beta] requested by [@minecraft/server-ui-bindings - 2.3.0-beta]"
        ]
    );
    // B07: server 2.0.0 and the gametest beta run, the server at its newest release.
    let b07 = both(
        ("@minecraft/server", "2.0.0"),
        ("@minecraft/server-gametest", "beta"),
    );
    assert_eq!(
        resolved(&b07),
        [
            "@minecraft/server 2.10.0",
            "@minecraft/server-gametest 1.0.0-beta"
        ]
    );
    assert!(!b07.diagnostics().has_errors());
    // B08: the server beta and server-ui 2.0.0 run on the beta.
    let b08 = both(
        ("@minecraft/server", "2.11.0-beta"),
        ("@minecraft/server-ui", "2.0.0"),
    );
    assert_eq!(
        resolved(&b08),
        [
            "@minecraft/server 2.11.0-beta",
            "@minecraft/server-ui 2.2.0"
        ]
    );
    // P22.
    let p22 = both(
        ("@minecraft/server", "1.0.0"),
        ("@minecraft/server-ui", "1.3.0"),
    );
    assert_eq!(
        resolved(&p22),
        ["@minecraft/server 1.19.0", "@minecraft/server-ui 1.3.0"]
    );
}
