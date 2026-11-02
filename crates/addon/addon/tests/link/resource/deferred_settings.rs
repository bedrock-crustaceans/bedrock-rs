//! 20 R51 and R52: the caustics texture of the deferred water settings and the blocks the
//! local lights are keyed by; R50 (the default identifiers) is the load's.

use bedrock_addon::diagnostic::{Diagnostic, DiagnosticDetail, Diagnostics, Phase, Severity};
use bedrock_addon::load::LoadOptions;

use crate::common::{link, stacks_of};

const WATER: &str = "water/water.json";
const LIGHTS: &str = "local_lighting/local_lighting.json";

/// What a finding says: its code, severity, phase, pack, file, JSON path and the kind and
/// name of its detail.
type Told = (
    &'static str,
    Severity,
    Phase,
    String,
    String,
    String,
    &'static str,
    String,
);

fn told(finding: &Diagnostic) -> Told {
    let (kind, id) = match finding.detail.as_deref() {
        Some(DiagnosticDetail::Reference { kind, id, .. }) => (*kind, id.clone()),
        _ => ("", String::new()),
    };
    (
        finding.code.name(),
        finding.severity,
        finding.phase,
        finding
            .location
            .pack
            .as_ref()
            .map(|pack| pack.name().to_owned())
            .unwrap_or_default(),
        finding
            .location
            .path
            .as_ref()
            .map(|path| path.as_str().to_owned())
            .unwrap_or_default(),
        finding.location.json_path.to_string(),
        kind,
        id,
    )
}

/// The findings at the file `path`.
fn at(found: &Diagnostics, path: &str) -> Vec<Told> {
    crate::common::in_file(found, path)
        .into_iter()
        .map(told)
        .collect()
}

// ---- R51: the caustics texture ---------------------------------------------------------------

#[cfg(all(feature = "texture_file", feature = "texture_set"))]
mod caustics {
    use bedrock_addon::pack::LinkOptions;

    use super::*;
    use crate::common::{leak, resource_pack_of, unloaded_pack};

    const MEMBER: &str = "minecraft:water_settings/caustics/texture";
    const IMAGE: &str = "textures/environment/caustics.png";

    /// Water settings `demo:water` whose caustics are the texture `texture`.
    fn water(texture: &str) -> &'static [u8] {
        leak(&format!(
            r#"{{"format_version": "1.26.20", "minecraft:water_settings": {{
                "description": {{"identifier": "demo:water"}}, "caustics": {{"texture": "{texture}"}}}}}}"#
        ))
    }

    /// The header of a PNG of `width` by `height`: all the pass reads of an image.
    fn png(width: u32, height: u32) -> &'static [u8] {
        let mut png = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 0, 0, 0, 13];
        png.extend_from_slice(b"IHDR");
        png.extend_from_slice(&width.to_be_bytes());
        png.extend_from_slice(&height.to_be_bytes());
        png.extend_from_slice(&[8, 6, 0, 0, 0, 0, 0, 0, 0]);
        Box::leak(png.into_boxed_slice())
    }

    /// The findings at the water file of a world of one resource pack with the settings
    /// (naming `textures/environment/caustics`) and `files`.
    fn linked(files: &[(&'static str, &'static [u8])]) -> Vec<Told> {
        let mut all = vec![(WATER, water("textures/environment/caustics"))];
        all.extend_from_slice(files);
        let set = stacks_of(
            Vec::new(),
            vec![resource_pack_of(2, &all)],
            LoadOptions::default(),
        );
        at(&set.link(), WATER)
    }

    fn error(code: &'static str) -> Told {
        (
            code,
            Severity::Error,
            Phase::Link,
            "rp 2".to_owned(),
            WATER.to_owned(),
            MEMBER.to_owned(),
            "texture",
            "textures/environment/caustics".to_owned(),
        )
    }

    #[test]
    fn render_r51_a_caustics_image_of_whole_frames_is_no_finding() {
        // One frame, and four frames stacked.
        assert_eq!(linked(&[(IMAGE, png(16, 16))]), []);
        assert_eq!(linked(&[(IMAGE, png(16, 64))]), []);
    }

    #[test]
    fn render_r51_a_caustics_texture_no_pack_has_is_a_kept_error_with_the_clients_text() {
        let set = stacks_of(
            Vec::new(),
            vec![resource_pack_of(
                2,
                &[(WATER, water("textures/environment/caustics"))],
            )],
            LoadOptions::default(),
        );
        let found = set.link();
        assert_eq!(at(&found, WATER), [error("TextureNotFound")]);
        let texts: Vec<String> = crate::common::in_file(&found, WATER)
            .iter()
            .map(|finding| finding.code.to_string())
            .collect();
        assert_eq!(
            texts,
            [
                "Resource pack caustics texture could not be loaded. No texture could be found at \"textures/environment/caustics\"."
            ]
        );
    }

    #[test]
    fn render_r51_an_image_that_is_no_whole_number_of_frames_or_does_not_decode_is_a_kept_error() {
        // Both sides of "the height is a multiple of the width".
        assert_eq!(linked(&[(IMAGE, png(16, 40))]), [error("InvalidTexture")]);
        assert_eq!(linked(&[(IMAGE, png(16, 48))]), []);
        // A file that is no PNG under a PNG's name.
        assert_eq!(
            linked(&[(IMAGE, b"not an image")]),
            [error("InvalidTexture")]
        );
    }

    #[test]
    fn render_r51_the_highest_packs_image_is_the_one_judged() {
        let water = (WATER, water("textures/environment/caustics"));
        let judged = |top: &'static [u8], bottom: &'static [u8]| {
            let set = stacks_of(
                Vec::new(),
                vec![
                    resource_pack_of(2, &[water, (IMAGE, top)]),
                    resource_pack_of(3, &[(IMAGE, bottom)]),
                ],
                LoadOptions::default(),
            );
            at(&set.link(), WATER)
        };
        assert_eq!(judged(png(16, 16), png(16, 40)), []);
        assert_eq!(judged(png(16, 40), png(16, 16)), [error("InvalidTexture")]);
    }

    #[test]
    fn render_r51_an_image_whose_bytes_the_pack_does_not_hold_is_skipped_with_the_frames_note() {
        // The settings are resident in the higher pack; the image is in a pack a host opened
        // from a folder and did not load.
        let set = stacks_of(
            Vec::new(),
            vec![
                resource_pack_of(2, &[(WATER, water("textures/environment/caustics"))]),
                unloaded_pack(3, "resources", &[(IMAGE, png(16, 40))]),
            ],
            LoadOptions::default(),
        );
        let found = set.link();
        assert_eq!(at(&found, WATER), []);
        assert_eq!(
            at(&found, IMAGE)
                .iter()
                .map(|told| (told.0, told.1))
                .collect::<Vec<_>>(),
            [("FileNotLoaded", Severity::Info)]
        );
        // Without the lints the file is skipped in silence.
        let quiet = set.link_with(&LinkOptions {
            lints: false,
            ..Default::default()
        });
        assert_eq!(at(&quiet, IMAGE), []);
        assert_eq!(at(&quiet, WATER), []);
    }

    #[test]
    fn render_r51_water_settings_without_a_caustics_texture_name_nothing() {
        let plain = r#"{"format_version": "1.26.20", "minecraft:water_settings": {"description": {"identifier": "demo:water"}}}"#;
        assert_eq!(at(&link(&[], &[(WATER, plain)]), WATER), []);
    }
}

#[cfg(not(all(feature = "texture_file", feature = "texture_set")))]
#[test]
fn render_without_the_texture_feature_a_caustics_texture_is_not_asked() {
    let water = r#"{"format_version": "1.26.20", "minecraft:water_settings": {
        "description": {"identifier": "demo:water"}, "caustics": {"texture": "textures/none"}}}"#;
    assert_eq!(at(&link(&[], &[(WATER, water)]), WATER), []);
}

// ---- R52: the local lights -------------------------------------------------------------------

/// Local lights for each of `blocks`.
fn lights(blocks: &[&str]) -> String {
    let members: Vec<String> = blocks
        .iter()
        .map(|block| format!(r#""{block}": {{"light_type": "point_light"}}"#))
        .collect();
    format!(
        r#"{{"format_version": "1.21.120", "minecraft:local_light_settings": {{{}}}}}"#,
        members.join(", ")
    )
}

#[cfg(feature = "block")]
mod local_lights {
    use bedrock_addon::pack::LinkOptions;

    use super::*;
    use crate::common::{hosted, link_with};

    const LAMP: &str = r#"{"format_version": "1.26.20", "minecraft:block": {"description": {"identifier": "demo:lamp"}}}"#;

    fn error(block: &str) -> Told {
        (
            "UnresolvedReference",
            Severity::Error,
            Phase::Link,
            "rp 2".to_owned(),
            LIGHTS.to_owned(),
            format!("minecraft:local_light_settings/{block}"),
            "block",
            block.to_owned(),
        )
    }

    #[test]
    fn render_r52_a_light_of_a_block_a_behaviour_pack_defines_is_no_finding() {
        let found = link(
            &[("blocks/lamp.json", LAMP)],
            &[(LIGHTS, &lights(&["demo:lamp"]))],
        );
        assert_eq!(at(&found, LIGHTS), []);
    }

    #[test]
    fn render_r52_a_light_of_a_block_of_the_game_is_no_finding() {
        // Without a host the crate's list of the game's blocks answers.
        let file = lights(&["minecraft:torch"]);
        assert_eq!(at(&link(&[], &[(LIGHTS, &file)]), LIGHTS), []);
        // With a host, the host does: yes for a block only it knows, no for one it lacks.
        let host = || hosted(&[("block", "demo:hosted")]);
        let both = lights(&["demo:hosted", "minecraft:torch"]);
        let found = link_with(&[], &[(LIGHTS, &both)], host(), &LinkOptions::default());
        assert_eq!(at(&found, LIGHTS), [error("minecraft:torch")]);
    }

    #[test]
    fn render_r52_a_light_of_no_block_is_a_kept_error_with_the_clients_text() {
        let found = link(
            &[("blocks/lamp.json", LAMP)],
            &[(LIGHTS, &lights(&["demo:lamp", "demo:nothing"]))],
        );
        assert_eq!(at(&found, LIGHTS), [error("demo:nothing")]);
        let texts: Vec<String> = crate::common::in_file(&found, LIGHTS)
            .iter()
            .map(|finding| finding.code.to_string())
            .collect();
        assert_eq!(texts, ["demo:nothing is not a block"]);
    }
}

#[cfg(not(feature = "block"))]
#[test]
fn render_without_the_block_feature_a_local_light_is_not_asked() {
    let found = link(&[], &[(LIGHTS, &lights(&["demo:nothing"]))]);
    assert_eq!(at(&found, LIGHTS), []);
}

// ---- R50: an audit ---------------------------------------------------------------------------

// The audit of the default identifier is the load's and cannot fire there either (the
// default identifier is that of a file just put into the map); its text is pinned by
// `render::deferred::tests::r50_a_default_identifier_the_map_lacks_is_the_games_could_not_find`
// and the choice of the default by `render_configs::stack::c13_…`. The link pass adds nothing.
#[test]
fn render_r50_the_default_identifiers_are_no_link_finding() {
    let lighting = r#"{"format_version": "1.26.0", "minecraft:lighting_settings": {"description": {"identifier": "demo:dusk"}}}"#;
    let set = stacks_of(
        Vec::new(),
        vec![crate::common::resource_pack(
            2,
            &[("lighting/global.json", lighting)],
        )],
        LoadOptions::default(),
    );
    let settings = set.resource.deferred_settings();
    assert_eq!(
        settings.defaults.lighting.as_ref().map(|id| id.as_str()),
        Some("demo:dusk")
    );
    assert_eq!(settings.defaults.water, None);
    assert_eq!(settings.diagnostics().count(), 0);
    assert_eq!(at(&set.link(), "lighting/global.json"), []);
}
