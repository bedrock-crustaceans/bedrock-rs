//! The rule of the loading messages that waits for the link pass: a tip that no `.lang` file
//! translates. It calls `StackSet::link()` and holds its result to the rule.

use bedrock_addon::diagnostic::{Diagnostics, Severity};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{PackOrigin, PackRepository, WorldPacks};

use super::common::{pack_uuid, pack_with};

/// The link findings of one resource pack with `files`.
fn link(files: &[(&str, &[u8])]) -> Diagnostics {
    let pack = pack_with(0, "resources", &[], files);
    let entries = format!(
        r#"[{{"pack_id": "{}", "version": [1, 0, 0]}}]"#,
        pack_uuid(0)
    );
    let world = WorldPacks::parse(None, Some(entries.as_bytes())).expect("a world list");
    let mut repository = PackRepository::new();
    repository.add_pack(pack, PackOrigin::User);
    repository.stacks(&world, LoadOptions::default()).link()
}

fn found(d: &Diagnostics, code: &str) -> Vec<(Severity, String)> {
    d.iter()
        .filter(|f| f.code.name() == code)
        .map(|f| {
            let path = f.location.path.as_ref().map(ToString::to_string);
            (f.severity, path.unwrap_or_default())
        })
        .collect()
}

#[test]
fn l9_a_loading_message_no_lang_file_translates() {
    // `tips.known` is translated, `tips.unknown` is not: the game shows the key itself.
    let d = link(&[
        (
            "loading_messages.json",
            br#"{ "beginner_loading_messages": ["tips.known", "tips.unknown"] }"#,
        ),
        ("texts/languages.json", br#"["en_US"]"#),
        ("texts/en_US.lang", b"tips.known=A tip\n"),
    ]);
    assert_eq!(
        found(&d, "UnresolvedLangKey"),
        [(Severity::Info, "loading_messages.json".to_owned())],
        "{d:?}"
    );
}
