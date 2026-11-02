//! Write-back as authored: what `to_json` writes for a file that loads is the file,
//! as JSON values, whatever the game makes of its members. The documents are the review's
//! probes: wrong-typed members, containers of another kind, skipped and empty entries, roots of
//! another kind.

use bedrock_addon::families::loading_messages::LoadingMessages;

use super::common::as_authored;

#[test]
fn loading_messages_are_written_as_authored() {
    as_authored::<LoadingMessages>(&[
        "{}",
        r#"{"beginner_loading_messages":["a"]}"#,
        r#"{"beginner_loading_messages":[{"message": 5, "condition": true}]}"#,
        r#"{"beginner_loading_messages":[{"message":5,"condition":true,"k":1}, 7, {"condition":"x"}]}"#,
        r#"{"beginner_loading_messages":{"b":"x","a":"y"},"mid_game_loading_messages":5,"store_progress_tooltips":{"k":1}}"#,
        r#"{"mid_game_loading_messages": 5}"#,
        "[]",
        r#"["a"]"#,
        "5",
    ]);
}
