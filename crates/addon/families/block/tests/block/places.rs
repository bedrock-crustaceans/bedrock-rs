//! Where in the file a member of the loaded block is: the moves the upgrade recorded stay with
//! the document's findings once they are put in its pack and file, for the link pass.

use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::document::Content;
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_family_block::Block;

#[test]
fn a_moved_member_is_where_the_file_has_it() {
    let text = r#"{"format_version": "1.16.100", "minecraft:block": {"description": {"identifier": "demo:x"}, "components": {"minecraft:creative_category": {"category": "nature"}}}}"#;
    let document = Block::parse(
        &PackPath::from_static("blocks/x.json"),
        text.as_bytes(),
        &LoadContext::detached(),
    )
    .expect("the block loads");
    let held = JsonPath::of_keys(["minecraft:block", "description", "menu_category"]);
    assert_eq!(
        document.diagnostics().place_of(&held),
        JsonPath::of_keys([
            "minecraft:block",
            "components",
            "minecraft:creative_category"
        ])
    );
}
