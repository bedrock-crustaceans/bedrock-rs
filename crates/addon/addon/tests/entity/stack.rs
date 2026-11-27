//! `bp.entities()`: the registry of a behaviour stack, through the accessor list and the
//! loaders.

use bedrock_addon::id::EntityId;
use bedrock_addon::load::{
    LoadOptions, RepositoryLoader, RootId, Source, StackLoader, StackOptions,
};
use bedrock_addon::pack::{OpenOptions, PackOrigin, StackSet, WorldPacks};

use super::responder::{Order, Responder, manifest};

/// Two behaviour packs (1 above 2), each defining `demo:shared`; pack 2 also `demo:low`, pack 1
/// a beta document.
fn stacks(beta_apis: bool) -> StackSet {
    let doc = |id: &str, health: u32| {
        format!(
            r#"{{"format_version": "1.26.50", "minecraft:entity": {{"description": {{"identifier": "{id}"}}, "components": {{"minecraft:health": {{"value": {health}}}}}}}}}"#
        )
    };
    let mut responder = Responder::new(Order::Issue);
    responder
        .file(1, "top/manifest.json", manifest(1, "data"))
        .file(1, "top/entities/shared.json", doc("demo:shared", 1))
        .file(
            1,
            "top/entities/beta.json",
            r#"{"format_version": "beta", "minecraft:entity": {"description": {"identifier": "demo:beta"}, "components": {}}}"#,
        )
        .file(1, "low/manifest.json", manifest(2, "data"))
        .file(1, "low/entities/sub/shared.json", doc("demo:shared", 2))
        .file(1, "low/entities/low.json", doc("demo:low", 3));
    let mut repository = RepositoryLoader::new(
        vec![Source::Directory {
            root: RootId(1),
            origin: PackOrigin::World,
        }],
        OpenOptions::default(),
    );
    let repository = responder.drive(&mut repository);
    let entry = |n: u128| {
        format!(
            r#"{{"pack_id": "{}", "version": [1, 0, 0]}}"#,
            uuid::Uuid::from_u128(n)
        )
    };
    let world = WorldPacks::parse(
        Some(format!("[{}, {}]", entry(1), entry(2)).as_bytes()),
        None,
    )
    .unwrap();
    let options = LoadOptions {
        beta_apis,
        ..LoadOptions::default()
    };
    let set = repository.stacks(&world, options);
    let mut loader = StackLoader::new(set, StackOptions::default());
    responder.drive(&mut loader)
}

#[test]
fn the_higher_pack_defines_an_identifier_and_every_file_below_entities_is_read() {
    let set = stacks(false);
    let entities = set.behavior.entities();
    assert_eq!(entities.len(), 2, "{entities:?}");
    let shared = entities.get(&EntityId::parse("demo:shared")).unwrap();
    assert_eq!(shared.origin().path.as_str(), "entities/shared.json");
    assert!(entities.contains(&EntityId::parse("demo:low")));
    // Without Beta APIs the beta document is refused.
    assert_eq!(entities.rejected().len(), 1);
}

#[test]
fn beta_documents_go_to_the_beta_registry() {
    let set = stacks(true);
    let entities = set.behavior.entities();
    let beta = entities.beta().expect("a beta registry with Beta APIs");
    assert!(beta.contains(&EntityId::parse("demo:beta")));
    assert!(!entities.contains(&EntityId::parse("demo:beta")));
}
