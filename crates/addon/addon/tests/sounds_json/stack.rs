//! How one `sounds.json` reads outside a stack and in one: a single copy does not judge a
//! `base` it cannot see, and the member vanilla's own file carries beside `sound`.

use bedrock_addon::diagnostic::Kind;
use bedrock_addon::document::Content;
use bedrock_addon::families::sounds_json::SoundBindings;
use bedrock_addon::load::LoadContext;
use bedrock_addon::storage::PackPath;

use super::common::{BINDINGS, Findable, copies, expect_clean, found};

#[test]
fn a_single_copy_does_not_judge_a_base_it_cannot_see() {
    let text = r#"{"block_sounds": {"copper_chest": {"base": "copper"}, "loop": {"base": "loop"},
        "known": {"base": "stone"}, "stone": {"events": {"hit": "hit.stone"}}}}"#;
    let document = SoundBindings::parse(
        &PackPath::new(BINDINGS).unwrap(),
        text.as_bytes(),
        &LoadContext::detached(),
    )
    .expect("the file loads");
    let codes: Vec<&str> = document.diagnostics().iter().map(|d| d.code.id()).collect();
    // A cycle is in the file itself; the unknown base may be a lower pack's.
    assert_eq!(codes, ["sounds_json/recursive_base"]);
    assert_eq!(
        document.block_sound("known", "hit").unwrap().sound,
        "hit.stone"
    );
    assert!(document.block_sound("copper_chest", "hit").is_none());
    // The stack, which sees every copy, reports it.
    let stack = copies(BINDINGS, &[text]);
    assert_eq!(
        found(stack.sounds_json(), "sounds_json/unknown_base").len(),
        1
    );
    // And resolves it when a lower pack defines the base.
    let lower = r#"{"block_sounds": {"copper": {"events": {"hit": "hit.copper"}}}}"#;
    let stack = copies(BINDINGS, &[lower, text]);
    assert!(
        stack
            .sounds_json()
            .findings()
            .iter()
            .all(|d| d.code.kind() != Kind::UnresolvedReference)
    );
    assert_eq!(
        stack
            .sounds_json()
            .block_sound("copper_chest", "hit")
            .unwrap()
            .sound,
        "hit.copper"
    );
}

#[test]
fn the_sounds_member_vanilla_writes_beside_sound_is_known_ignored() {
    // bedrock-samples v1.26.50.4 `sounds.json`, `interactive_sounds.block_sounds.amethyst_block`.
    let stack = copies(
        BINDINGS,
        &[
            r#"{"interactive_sounds": {"block_sounds": {"amethyst_block": {"events": {
            "default": "",
            "fall": {"sound": "fall.amethyst_block", "sounds": "fall.amethyst_block", "volume": 0.4},
            "jump": {"sound": "step.amethyst_block", "volume": 1.0}}, "pitch": 1.0, "volume": 1.0}}}}"#,
        ],
    );
    let sounds = stack.sounds_json();
    expect_clean(sounds);
    let block = &sounds.interactive_sounds.block_sounds["amethyst_block"];
    assert_eq!(block.sounds.events["fall"].sound, "fall.amethyst_block");
    assert_eq!(block.sounds.events["fall"].volume.min, 0.4);
    assert_eq!(block.sounds.events["default"].sound, "");
    assert_eq!(block.sounds.events["jump"].pitch.max, 1.0);
}
