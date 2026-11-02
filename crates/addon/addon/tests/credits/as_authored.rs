//! Write-back as authored: what `to_json` writes for a file that loads is the file,
//! as JSON values, whatever the game makes of its members. The documents are the review's
//! probes: wrong-typed members, containers of another kind, skipped and empty entries, roots of
//! another kind.

use bedrock_addon::families::credits::Credits;

use super::common::as_authored;

#[test]
fn credits_are_written_as_authored() {
    as_authored::<Credits>(&[
        "[]",
        r#"[{"section":"A"}]"#,
        r#"[{"section": 5}]"#,
        r#"[{"section":"A","disciplines":[{"discipline": true}]}]"#,
        r#"[{"section":"A","disciplines":[{"discipline":"d","titles":[{"title":"t","names":[1,"x",true]}]}]}]"#,
        r#"[{"section":"A","disciplines":[{"discipline":"d","titles":[{"title":"t","names":"n"}]}]}]"#,
        r#"[{"section":"A","disciplines":7}]"#,
        r#"[{"section":5,"disciplines":[{"discipline":true,"titles":[{"title":null,"names":[1,"x",true]}]}]}]"#,
        r#"[{"section":"A","disciplines":7}, {}, 3, {"section":"B","disciplines":[{}, {"discipline":"d","titles":[{"title":"t","names":"n"}, 4]}]}]"#,
        "{}",
        "5",
    ]);
}
