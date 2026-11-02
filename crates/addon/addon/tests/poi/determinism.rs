//! The determinism rule for the POI group with the experiment on: the stack
//! loader's result is the same for every order in which reads and jobs complete. The
//! crate-wide `tests/determinism.rs` loads without experiments, where the POI fragment only
//! contributes the gate's finding, so the fragment and a pack with every
//! cross-document finding are driven here through the same test responder.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::load::{
    Experiment, Experiments, LoadOptions, RepositoryLoader, Residency, RootId, Source, StackLoader,
    StackOptions,
};
use bedrock_addon::pack::{OpenOptions, PackOrigin, WorldPacks};

use super::common::{block_of, fixture, is_closing, tag_of};
use super::responder::{Order, Responder, manifest};

/// Two behaviour packs at root 1: `top/` (the determinism fragment, a type that takes a block,
/// a tag cycle, an edge to a missing tag, a file that is neither kind) and `low/` (a type the
/// top pack shadows, a second document of a tag).
fn poi_packs() -> Responder {
    let fragment =
        |name: &str| fixture(&format!("determinism/behavior_pack/poi/{name}.json")).into_bytes();
    let mut responder = Responder::new(Order::Issue);
    responder
        .file(1, "top/manifest.json", manifest(1, "data"))
        .file(
            1,
            "top/poi/blocks/determinism_type.json",
            fragment("blocks/determinism_type"),
        )
        .file(
            1,
            "top/poi/tags/determinism_tag.json",
            fragment("tags/determinism_tag"),
        )
        .file(
            1,
            "top/poi/blocks/taker.json",
            block_of("test:taker", "", r#"["minecraft:anvil", "minecraft:bell"]"#).into_bytes(),
        )
        .file(
            1,
            "top/poi/tags/t1.json",
            tag_of("test:t1", r#", "parents": ["test:t2", "test:missing"]"#).into_bytes(),
        )
        .file(
            1,
            "top/poi/tags/t2.json",
            tag_of("test:t2", r#", "parents": ["test:t1"]"#).into_bytes(),
        )
        .file(
            1,
            "top/poi/blocks/neither.json",
            &br#"{"format_version": "1.26.40"}"#[..],
        )
        .file(1, "low/manifest.json", manifest(2, "data"))
        .file(
            1,
            "low/poi/blocks/determinism_type.json",
            block_of(
                "determinism:forge",
                r#""tickets": 9,"#,
                r#"["minecraft:bell"]"#,
            )
            .into_bytes(),
        )
        .file(
            1,
            "low/poi/tags/t1.json",
            tag_of("test:t1", r#", "children": ["determinism:smithy"]"#).into_bytes(),
        );
    responder
}

/// The world list naming both packs, `top` first.
fn world() -> WorldPacks {
    let list = format!(
        r#"[{{"pack_id": "{}", "version": [1, 0, 0]}}, {{"pack_id": "{}", "version": [1, 0, 0]}}]"#,
        uuid::Uuid::from_u128(1),
        uuid::Uuid::from_u128(2)
    );
    WorldPacks::parse(Some(list.as_bytes()), None).unwrap()
}

/// The group loaded with completions in `order`: its `Debug`, and what it holds.
fn load(template: &Responder, order: Order, residency: Residency) -> (String, Vec<String>) {
    let mut responder = template.reordered(order);
    let mut repository = RepositoryLoader::new(
        vec![Source::Directory {
            root: RootId(1),
            origin: PackOrigin::World,
        }],
        OpenOptions::default(),
    );
    let repository = responder.drive(&mut repository);
    let mut experiments = Experiments::new();
    experiments.insert(Experiment::Poi);
    let options = LoadOptions {
        experiments,
        ..LoadOptions::default()
    };
    let set = repository.stacks(&world(), options);
    let stack_options = StackOptions {
        behavior: residency,
        resource: residency,
        ..StackOptions::default()
    };
    let mut loader = StackLoader::new(set, stack_options);
    let set = responder.drive(&mut loader);
    let poi = set.behavior.poi();
    let findings = poi
        .diagnostics()
        .map(|d| format!("{} {}", d.severity, d.code.name()))
        .collect();
    assert_eq!(
        (poi.types.len(), poi.tags.len(), poi.rejected().count()),
        (2, 3, 1),
        "{order:?}"
    );
    assert_eq!(
        poi.diagnostics()
            .filter(|d| d.severity == Severity::Error && !is_closing(d))
            .count(),
        1,
        "the file that is neither kind"
    );
    (format!("{poi:?}"), findings)
}

#[test]
fn every_completion_order_gives_the_same_group() {
    let template = poi_packs();
    for residency in [Residency::Transient, Residency::All] {
        let reference = load(&template, Order::Issue, residency);
        // The stack holds what the test is about: a shadowed type, a claimed block, a cycle,
        // a missing tag.
        for code in [
            "info ShadowedDefinition",
            "info PoiBlockClaimedTwice",
            "info ReferenceCycle",
            "info UnresolvedReference",
        ] {
            assert!(
                reference.1.iter().any(|finding| finding == code),
                "{code}: {:?}",
                reference.1
            );
        }
        for order in Order::ALL {
            assert_eq!(
                load(&template, order, residency),
                reference,
                "{order:?}, {residency:?}"
            );
        }
    }
}
