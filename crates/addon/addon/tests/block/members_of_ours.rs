//! The members of ours that no cereal document takes (`deprecated`,
//! `deprecated_event_handlers`, and `ordered_v1_custom_components`, which the 1.26.20 document
//! deprecates by name): the writer never puts them in a cereal document, which the game
//! refuses for them, and writes them in the shapes the legacy reader takes (as the trade family does).

use bedrock_addon::document::Content;
use bedrock_addon::families::block::Block;
use bedrock_addon::id::BlockId;
use bedrock_addon::json::EncodeError;

use super::common::{allow_listed, context, document, loads};

/// The written text with its whitespace collapsed.
fn flat(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// An allow-listed 1.19.40 block's legacy events and components are written back as a
/// 1.19.40 document of `events` and legacy components, which reads back as the same block.
#[test]
fn legacy_events_and_components_are_written_in_their_legacy_shape() {
    let text = document(
        "1.19.40",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:queued_ticking": {"interval_range": [5, 9], "looping": true, "on_tick": {"event": "e"}}, "minecraft:on_interact": {"event": "e"}}, "events": {"e": {"sequence": [{"die": {}}, {"trigger": "e2"}], "randomize": [{"weight": 2, "run_command": {"command": ["say hi"]}}]}, "e2": {}}}"#,
    );
    let block = loads(&text, &allow_listed());
    assert!(!block.deprecated_event_handlers.is_empty());
    let written = block.to_json().expect("the block writes");
    let flat_text = flat(&written);
    assert!(
        flat_text.contains(r#""format_version": "1.19.40""#),
        "{written}"
    );
    for absent in ["deprecated", "ordered_v1_custom_components"] {
        assert!(!written.contains(absent), "{absent}: {written}");
    }
    assert!(flat_text.contains(r#""events": {"#), "{written}");
    assert!(flat_text.contains(r#""weight": 2.0"#), "{written}");
    assert!(
        flat_text.contains(r#""interval_range": [ 5, 9 ]"#),
        "{written}"
    );
    assert!(
        flat_text.contains(r#""minecraft:on_interact""#),
        "{written}"
    );
    let again = loads(&written, &allow_listed());
    assert_eq!(again.value(), block.value());
}

/// A 1.21.60 block's v1 `minecraft:custom_components` list is written back as the list, at
/// 1.21.60, never as `ordered_v1_custom_components`.
#[test]
fn a_v1_custom_components_list_is_written_as_the_list() {
    let text = document(
        "1.21.60",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:custom_components": ["demo:glow", "demo:spin"], "minecraft:friction": 0.4}}"#,
    );
    let block = loads(&text, &context());
    assert_eq!(
        block.ordered_v1_custom_components,
        ["demo:glow", "demo:spin"]
    );
    let written = block.to_json().expect("the block writes");
    assert!(
        !written.contains("ordered_v1_custom_components"),
        "{written}"
    );
    assert!(
        flat(&written).contains(r#""minecraft:custom_components": [ "demo:glow", "demo:spin" ]"#),
        "{written}"
    );
    assert!(
        flat(&written).contains(r#""format_version": "1.21.60""#),
        "{written}"
    );
    assert_eq!(loads(&written, &context()).value(), block.value());
}

/// A block of the newest format that holds a member of ours has no text the game reads:
/// the writer refuses it at the member.
#[test]
fn a_member_of_ours_in_a_newest_block_is_not_representable() {
    let mut block = Block::new(BlockId::parse("demo:x".to_owned()));
    block
        .ordered_v1_custom_components
        .push("demo:glow".to_owned());
    match block.to_json() {
        Err(EncodeError::NotRepresentable { at, .. }) => {
            assert_eq!(
                at.to_string(),
                "minecraft:block/ordered_v1_custom_components"
            );
        }
        other => panic!("{other:?}"),
    }
}
