//! `eat_block` with an unknown block in a pair, through a world (link rule E10).

use bedrock_addon::diagnostic::Severity;

/// `v12_component_value_messages`, `eat_block` with an unknown block in a pair,
/// through a world: resolving the name needs the block registry, so the link pass tells it
/// (link rule E10). The text and the dropped pair are unit-tested on
/// `goal::convert::eat_block_pairs`.
// now links a world of that document. The expectations are as they were.)
#[test]
fn v12_eat_block_unknown_block_is_reported_with_vanillas_text() {
    let d = link_entity(
        "1.26.40",
        r#""minecraft:behavior.eat_block": {"eat_and_replace_block_pairs": [{"eat_block": "demo:nothing", "replace_block": "air"}]}"#,
    );
    let found = d
        .iter()
        .find(|f| f.code.name() == "UnresolvedReference")
        .unwrap_or_else(|| panic!("{d:?}"));
    assert_eq!(found.severity, Severity::Error);
    assert!(
        bedrock_addon::diagnostic::testing::same_line(
            &found.code,
            "UnresolvedReference",
            "Block demo:nothing on \"eat_and_replace_block_pairs\" on \"minecraft:behavior.eat_block\" is not a valid block."
        ),
        "{:?}",
        found.code
    );
}

/// The link findings of a world whose one behaviour pack has the entity document of
/// [`load_entity`]'s shape at `entities/goal.json`.
fn link_entity(version: &str, components: &str) -> bedrock_addon::diagnostic::Diagnostics {
    use bedrock_addon::load::{
        LoadOptions, RepositoryLoader, RootId, Source, StackLoader, StackOptions,
    };
    use bedrock_addon::pack::{OpenOptions, PackOrigin, WorldPacks};

    use super::responder::{Order, Responder, manifest};

    let text = format!(
        r#"{{"format_version": "{version}", "minecraft:entity": {{"description": {{"identifier": "demo:goal"}}, "components": {{{components}}}}}}}"#
    );
    let mut responder = Responder::new(Order::Issue);
    responder
        .file(1, "pack/manifest.json", manifest(1, "data"))
        .file(1, "pack/entities/goal.json", text);
    let mut repository = RepositoryLoader::new(
        vec![Source::Directory {
            root: RootId(1),
            origin: PackOrigin::World,
        }],
        OpenOptions::default(),
    );
    let repository = responder.drive(&mut repository);
    let world = WorldPacks::parse(
        Some(
            format!(
                r#"[{{"pack_id": "{}", "version": [1, 0, 0]}}]"#,
                uuid::Uuid::from_u128(1)
            )
            .as_bytes(),
        ),
        None,
    )
    .unwrap();
    let set = repository.stacks(&world, LoadOptions::default());
    let mut loader = StackLoader::new(set, StackOptions::default());
    responder.drive(&mut loader).link()
}
