//! The `load` rows of the atlases and textures (20 R33, R36, R37): each is told when the
//! texture group loads, with the row's code and severity, and the link pass adds nothing.

use bedrock_addon::diagnostic::{Diagnostic, Severity};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::StackSet;

use crate::common::{in_file, stacks};

/// A world whose one resource pack has `files`.
fn world(files: &[(&'static str, &str)]) -> StackSet {
    stacks(&[], files, LoadOptions::default())
}

/// What the load says at `file`: code, severity, JSON path.
fn loaded(set: &StackSet, file: &str) -> Vec<(&'static str, Severity, String)> {
    let rp = &set.resource;
    rp.texture_atlases()
        .diagnostics()
        .chain(rp.texture_lists().diagnostics())
        .chain(rp.texture_sets().diagnostics())
        .filter(|finding| {
            finding
                .location
                .path
                .as_ref()
                .is_some_and(|path| path.as_str() == file)
        })
        .map(said)
        .collect()
}

fn said(finding: &Diagnostic) -> (&'static str, Severity, String) {
    (
        finding.code.name(),
        finding.severity,
        finding.location.json_path.to_string(),
    )
}

/// What the link pass says at `file`.
fn linked(set: &StackSet, file: &str) -> Vec<(&'static str, Severity, String)> {
    in_file(&set.link(), file).into_iter().map(said).collect()
}

#[test]
fn r33_an_atlas_layer_path_no_pack_has_is_told_at_load_and_not_by_the_link_pass() {
    for (file, name) in [
        ("textures/terrain_texture.json", "atlas.terrain"),
        ("textures/item_texture.json", "atlas.items"),
    ] {
        let atlas = format!(
            r#"{{ "texture_name": "{name}", "texture_data": {{ "x": {{ "textures": "textures/nowhere" }} }} }}"#
        );
        let set = world(&[(file, &atlas)]);
        assert_eq!(
            loaded(&set, file),
            [(
                "TextureNotFound",
                Severity::Info,
                "texture_data/x/textures".to_owned()
            )],
            "{file}"
        );
        assert_eq!(linked(&set, file), [], "{file}");
    }
}

#[test]
fn r36_a_list_entry_no_file_of_the_pack_matches_is_told_at_load_and_not_by_the_link_pass() {
    const LIST: &str = "textures/textures_list.json";
    let set = world(&[(LIST, r#"[ "textures/blocks/gone" ]"#)]);
    assert_eq!(
        loaded(&set, LIST),
        [("TextureNotFound", Severity::Info, "0".to_owned())]
    );
    assert_eq!(linked(&set, LIST), []);
}

#[test]
fn r37_a_layer_image_the_set_s_pack_lacks_is_told_at_load_and_not_by_the_link_pass() {
    const SET: &str = "textures/blocks/stone.texture_set.json";
    let set = world(&[(
        SET,
        r#"{ "format_version": "1.16.100", "minecraft:texture_set": { "color": "stone" } }"#,
    )]);
    assert_eq!(
        loaded(&set, SET),
        [(
            "TextureNotFound",
            Severity::Info,
            "minecraft:texture_set/color".to_owned()
        )]
    );
    assert_eq!(set.resource.texture_sets().rejected_sets().count(), 1);
    assert_eq!(linked(&set, SET), []);
}
