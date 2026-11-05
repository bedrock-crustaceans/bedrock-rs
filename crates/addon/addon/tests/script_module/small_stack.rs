//! Stage 1, discovery and resolution of a large manifest against a catalog with a long chain
//! of requirements, on a thread with 1 MiB of stack: nothing here recurses over its input or
//! keeps a large value in a frame. Run once with full debug info before a merge (COMMON
//! "Build profiles differ").

use bedrock_addon::families::script::{
    CatalogEntry, CatalogModule, ModuleCatalog, ScriptContext, ScriptDiscovery, ScriptFiles,
    ScriptModule, ScriptVersion,
};
use bedrock_addon::load::LoadContext;
use bedrock_addon::storage::PackPath;
use uuid::Uuid;

/// The stack of the working thread: half the default of a spawned thread.
const SMALL_STACK: usize = 1024 * 1024;

/// How long the chain of requiring modules is, and how many dependencies the manifest has.
const CHAIN: usize = 20_000;
const DEPENDENCIES: usize = 5_000;

/// `@chain/m0` requires `@chain/m1`, which requires `@chain/m2`, and so on; with `cycle` the
/// last requires the first.
fn chain(cycle: bool) -> ModuleCatalog {
    let mut catalog = ModuleCatalog::new();
    for i in 0..CHAIN {
        let mut version = CatalogEntry::new(ScriptVersion::new(1, u16::MAX, 0));
        let next = if i + 1 < CHAIN {
            Some(i + 1)
        } else {
            cycle.then_some(0)
        };
        if let Some(next) = next {
            version.requires = vec![(format!("@chain/m{next}"), ScriptVersion::new(1, 0, 0))];
        }
        catalog.insert(
            &format!("@chain/m{i}"),
            CatalogModule {
                uuid: Some(Uuid::from_u128(1000 + i as u128)),
                bindings: None,
                versions: vec![version],
            },
        );
    }
    catalog
}

/// A format-2 manifest with `DEPENDENCIES` dependencies on `@chain/m0`, each at another
/// version of major 1.
fn manifest() -> String {
    let dependencies: Vec<String> = (0..DEPENDENCIES)
        .map(|i| {
            format!(
                r#"{{ "module_name": "@chain/m0", "version": "1.{}.{}", "optional": false }}"#,
                i / 100,
                i % 100
            )
        })
        .collect();
    format!(
        r#"{{
  "format_version": 2,
  "header": {{ "name": "p", "uuid": "11111111-1111-1111-1111-111111111111", "version": [1, 0, 0],
              "min_engine_version": [1, 21, 0] }},
  "modules": [ {{ "type": "script", "uuid": "22222222-2222-2222-2222-222222222222", "version": [3, 2, 1],
                 "entry": "scripts/main.js" }} ],
  "dependencies": [{}]
}}"#,
        dependencies.join(",\n")
    )
}

fn on_a_small_stack(work: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .stack_size(SMALL_STACK)
        .spawn(work)
        .expect("a thread")
        .join()
        .expect("the work finishes");
}

fn discover_the_large_pack(cycle: bool) {
    let text = manifest();
    let document = ScriptModule::parse(
        &PackPath::from_static("manifest.json"),
        text.as_bytes(),
        &LoadContext::detached(),
    )
    .unwrap_or_else(|rejected| panic!("refused: {:?}", rejected.diagnostics.first_error()));
    // One finding per dependency: the legacy reader ignores `optional`.
    assert_eq!(document.diagnostics().len(), DEPENDENCIES);
    let module = document.into_value().expect("a script module");
    assert_eq!(module.dependencies.len(), DEPENDENCIES);
    let catalog = chain(cycle);
    let found = ScriptDiscovery::new(&catalog).discover(
        &module,
        &ScriptFiles::from_members(["main.js"]),
        &ScriptContext::default(),
    );
    assert!(
        !found.diagnostics().has_errors(),
        "{:?}",
        found.diagnostics().first_error()
    );
    let plugin = found.plugin().expect("the script runs");
    assert_eq!(plugin.dependencies.len(), DEPENDENCIES);
    assert!(
        plugin
            .dependencies
            .iter()
            .all(|d| d.resolved == ScriptVersion::new(1, u16::MAX, 0))
    );
}

#[test]
fn a_large_manifest_and_a_long_chain_fit_a_small_stack() {
    on_a_small_stack(|| discover_the_large_pack(false));
}

#[test]
fn a_cycle_of_requirements_ends_on_a_small_stack() {
    on_a_small_stack(|| discover_the_large_pack(true));
}
