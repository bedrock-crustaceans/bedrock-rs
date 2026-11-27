//! Write-back as authored: what `to_json` writes for a file that loads is the file,
//! as JSON values, whatever the game makes of its members. The documents are the review's
//! probes: wrong-typed members, containers of another kind, skipped and empty entries, roots of
//! another kind.

use bedrock_addon_family_splashes::Splashes;

use bedrock_addon_test_support::as_authored;

#[test]
fn splashes_are_written_as_authored() {
    as_authored::<Splashes>(&[
        "{}",
        r#"{"splashes":["a"]}"#,
        r#"{"canMerge":"true","splashes":{"k":"v"},"conditional":5,"x":1}"#,
        r#"["b"]"#,
        "5",
    ]);
}
