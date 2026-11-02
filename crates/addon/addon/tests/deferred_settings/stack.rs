//! The rows that need the whole stack (`rp.deferred_settings()`): c10–c13, c15, c20, k03,
//! h04, h05, l09, t02, t03; validation.md E8–E11, H3, H4, P3, T0, T3; and the rows that wait
//! for the link pass (w06, t04, t05).

use bedrock_addon::diagnostic::{Diagnostics, Phase, Severity};
use bedrock_addon::families::deferred_settings::{LightType, Mers, Rgb8, ShadowStyle};
use bedrock_addon::id::{
    AtmosphereSettingsId, BlockId, ColorGradingSettingsId, CubemapSettingsId, LightingSettingsId,
    WaterSettingsId,
};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{PackOrigin, PackRepository, WorldPacks};
use bedrock_addon::storage::PackPath;

use super::common::*;
use super::pack_common::{bp, rp};

fn lighting(identifier: &str, desaturation: f32) -> String {
    doc(
        "1.26.0",
        LIGHTING,
        &named(
            identifier,
            &format!(r#""emissive": {{"desaturation": {desaturation}}}"#),
        ),
    )
}

fn water(identifier: &str) -> String {
    doc("1.26.20", WATER, &named(identifier, ""))
}

fn shadows(texel_size: u16) -> String {
    doc(
        "1.21.80",
        SHADOWS,
        &format!(r#""texel_size": {texel_size}"#),
    )
}

fn pbr(metalness: u8) -> String {
    doc(
        "1.21.40",
        PBR,
        &format!(
            r#""blocks": {{"global_metalness_emissive_roughness_subsurface": [{metalness}, 0, 255, 0]}}"#
        ),
    )
}

fn local(members: &str) -> String {
    doc("1.21.120", LOCAL, members)
}

fn lighting_id(text: &str) -> LightingSettingsId {
    LightingSettingsId::parse(text)
}

const BROKEN: &str = "{ not json";

#[test]
fn an_empty_stack_has_empty_settings_and_the_fixed_cubemap_default() {
    let stack = stack(&[&[]]);
    let settings = stack.deferred_settings();
    assert!(settings.lighting.is_empty() && settings.water.is_empty());
    assert!(settings.atmosphere.is_empty() && settings.color_grading.is_empty());
    assert!(settings.cubemaps.is_empty());
    assert!(settings.shadow_settings().is_none());
    assert!(settings.pbr_fallback_settings().is_none());
    assert!(settings.local_lighting.lights.is_empty());
    assert_eq!(settings.defaults.lighting, None);
    assert_eq!(settings.defaults.water, None);
    assert_eq!(settings.defaults.atmosphere, None);
    assert_eq!(settings.defaults.color_grading, None);
    assert_eq!(
        settings.defaults.cubemap,
        CubemapSettingsId::parse("minecraft:default_cubemap")
    );
    assert!(settings.default_lighting().is_none());
    assert_eq!(settings.diagnostics().count(), 0);
}

#[test]
fn every_payload_is_read_from_its_folder() {
    let stack = stack(&[&[
        ("lighting/a.json", lighting("t:l", 0.5)),
        (
            "atmospherics/a.json",
            doc("1.26.20", ATMOSPHERE, &named("t:a", "")),
        ),
        ("water/a.json", water("t:w")),
        (
            "color_grading/a.json",
            doc("1.21.90", GRADING, &named("t:g", "")),
        ),
        (
            "cubemaps/a.json",
            doc("1.21.130", CUBEMAP, &named("t:c", "")),
        ),
        ("shadows/global.json", shadows(8)),
        ("pbr/global.json", pbr(7)),
        (
            "local_lighting/local_lighting.json",
            local(r#""minecraft:torch": {"light_type": "point_light"}"#),
        ),
    ]]);
    let settings = stack.deferred_settings();
    assert_eq!(
        settings.diagnostics().count(),
        0,
        "{:?}",
        settings.diagnostics().collect::<Vec<_>>()
    );
    assert_eq!(
        origin(settings.lighting.get(&lighting_id("t:l")).unwrap()),
        (Some(0), "lighting/a.json")
    );
    assert!(
        settings
            .atmosphere
            .contains(&AtmosphereSettingsId::parse("t:a"))
    );
    assert!(settings.water.contains(&WaterSettingsId::parse("t:w")));
    assert!(
        settings
            .color_grading
            .contains(&ColorGradingSettingsId::parse("t:g"))
    );
    assert!(settings.cubemaps.contains(&CubemapSettingsId::parse("t:c")));
    assert_eq!(settings.shadow_settings().unwrap().texel_size, Some(8));
    assert_eq!(
        settings.pbr_fallback_settings().unwrap().blocks().metalness,
        7
    );
    assert_eq!(
        settings.local_lighting.lights[&BlockId::parse("minecraft:torch")].light_type,
        Some(LightType::PointLight)
    );
    // None of these files is a default file.
    assert_eq!(settings.defaults.lighting, None);
    assert_eq!(settings.defaults.water, None);
}

#[expect(clippy::float_cmp, reason = "the reader gives exact values")]
#[test]
fn c10_the_higher_pack_wins_per_identifier_deviation() {
    // vanilla = "silent" about the hidden definitions.
    let stack = stack(&[
        &[("lighting/top.json", lighting("t:l", 0.1))],
        &[
            ("lighting/middle.json", lighting("t:l", 0.2)),
            ("lighting/other.json", lighting("t:only_middle", 0.3)),
        ],
        &[
            ("lighting/bottom.json", lighting("t:l", 0.4)),
            ("lighting/own.json", lighting("t:only_bottom", 0.5)),
        ],
    ]);
    let settings = stack.deferred_settings();
    assert_eq!(settings.lighting.len(), 3);
    let kept = settings.lighting.get(&lighting_id("t:l")).unwrap();
    assert_eq!(origin(kept), (Some(0), "lighting/top.json"));
    assert_eq!(kept.emissive().desaturation(), 0.1);
    assert_eq!(
        origin(
            settings
                .lighting
                .get(&lighting_id("t:only_middle"))
                .unwrap()
        ),
        (Some(1), "lighting/other.json")
    );
    assert_eq!(
        origin(
            settings
                .lighting
                .get(&lighting_id("t:only_bottom"))
                .unwrap()
        ),
        (Some(2), "lighting/own.json")
    );
    assert_eq!(
        located(settings.diagnostics(), Severity::Info),
        [
            (
                "ShadowedDefinition",
                Some(1),
                "lighting/middle.json".to_owned(),
                "definition shadowed by a higher-priority source".to_owned()
            ),
            (
                "ShadowedDefinition",
                Some(2),
                "lighting/bottom.json".to_owned(),
                "definition shadowed by a higher-priority source".to_owned()
            ),
        ]
    );
    assert_eq!(settings.diagnostics().count(), 2);
}

#[test]
fn c11_e8_the_second_file_of_an_identifier_in_one_pack_is_an_error() {
    let stack = stack(&[
        &[
            ("lighting/a.json", lighting("t:l", 0.1)),
            ("lighting/b.json", lighting("t:l", 0.2)),
            ("lighting/c.json", lighting("t:l", 0.3)),
            ("lighting/d.json", lighting("t:other", 0.4)),
        ],
        &[
            ("lighting/y.json", lighting("t:l", 0.5)),
            ("lighting/z.json", lighting("t:l", 0.6)),
        ],
    ]);
    let settings = stack.deferred_settings();
    // The first file in directory order stays.
    let kept = settings.lighting.get(&lighting_id("t:l")).unwrap();
    assert_eq!(origin(kept), (Some(0), "lighting/a.json"));
    assert_eq!(settings.lighting.len(), 2);
    let text = "Identifier t:l already exists in resource pack";
    assert_eq!(
        located(settings.diagnostics(), Severity::Error),
        [
            (
                "DuplicateIdentifier",
                Some(0),
                "lighting/b.json".to_owned(),
                text.to_owned()
            ),
            (
                "DuplicateIdentifier",
                Some(0),
                "lighting/c.json".to_owned(),
                text.to_owned()
            ),
            // The check is per pack: the lower pack's second file too.
            (
                "DuplicateIdentifier",
                Some(1),
                "lighting/z.json".to_owned(),
                text.to_owned()
            ),
        ]
    );
    assert_eq!(settings.lighting.rejected().len(), 3);
    // The lower pack's first file is hidden by the higher pack, not a duplicate.
    assert_eq!(
        located(settings.diagnostics(), Severity::Info),
        [(
            "ShadowedDefinition",
            Some(1),
            "lighting/y.json".to_owned(),
            "definition shadowed by a higher-priority source".to_owned()
        )]
    );
}

#[test]
fn c12_a_file_that_fails_does_not_hide_a_lower_packs() {
    let stack = stack(&[
        &[(
            "water/foo.json",
            doc(
                "1.26.20",
                WATER,
                &named("t:w", r#""waves": {"octaves": 99}"#),
            ),
        )],
        &[("water/bar.json", water("t:w"))],
        &[("water/baz.json", water("t:w"))],
    ]);
    let settings = stack.deferred_settings();
    let kept = settings.water.get(&WaterSettingsId::parse("t:w")).unwrap();
    assert_eq!(origin(kept), (Some(1), "water/bar.json"));
    assert_eq!(settings.water.rejected().len(), 1);
    assert_eq!(
        settings.water.rejected()[0].origin.path.as_str(),
        "water/foo.json"
    );
    assert_eq!(
        located(settings.diagnostics(), Severity::Error)
            .iter()
            .map(|f| (f.0, f.1, f.2.as_str()))
            .collect::<Vec<_>>(),
        [("InvalidValue", Some(0), "water/foo.json")]
    );
    assert_eq!(
        located(settings.diagnostics(), Severity::Info)
            .iter()
            .map(|f| (f.0, f.1, f.2.as_str()))
            .collect::<Vec<_>>(),
        [("ShadowedDefinition", Some(2), "water/baz.json")]
    );
}

#[test]
fn c13_the_default_identifier_is_the_top_loadable_default_files() {
    let stack = stack(&[
        &[("lighting/global.json", BROKEN.to_owned())],
        &[("lighting/global.json", lighting("t:middle", 0.2))],
        &[("lighting/global.json", lighting("t:bottom", 0.3))],
    ]);
    let settings = stack.deferred_settings();
    assert_eq!(settings.defaults.lighting, Some(lighting_id("t:middle")));
    assert_eq!(
        origin(settings.default_lighting().unwrap()),
        (Some(1), "lighting/global.json")
    );
    // The lower default file is still a definition of its own identifier.
    assert!(settings.lighting.contains(&lighting_id("t:bottom")));
    assert_eq!(
        located(settings.diagnostics(), Severity::Error)
            .iter()
            .map(|f| (f.0, f.1, f.2.as_str()))
            .collect::<Vec<_>>(),
        [("MalformedInput", Some(0), "lighting/global.json")]
    );
}

#[expect(clippy::float_cmp, reason = "the reader gives exact values")]
#[test]
fn the_top_packs_default_file_decides_when_it_loads() {
    let stack = stack(&[
        &[
            ("lighting/aaa.json", lighting("t:first_by_path", 0.1)),
            ("lighting/global.json", lighting("t:top", 0.2)),
        ],
        &[("lighting/global.json", lighting("t:lower", 0.3))],
    ]);
    let settings = stack.deferred_settings();
    assert_eq!(settings.defaults.lighting, Some(lighting_id("t:top")));
    assert_eq!(
        settings
            .default_lighting()
            .unwrap()
            .emissive()
            .desaturation(),
        0.2
    );
}

#[test]
fn each_payload_has_its_own_default_file() {
    let stack = stack(&[&[
        ("lighting/global.json", lighting("t:l", 0.5)),
        (
            "atmospherics/atmospherics.json",
            doc("1.26.20", ATMOSPHERE, &named("t:a", "")),
        ),
        ("water/water.json", water("t:w")),
        (
            "color_grading/color_grading.json",
            doc("1.21.90", GRADING, &named("t:g", "")),
        ),
        // Another payload's default name is no default file here.
        ("water/global.json", water("t:not_default")),
        ("lighting/water.json", lighting("t:not_default", 0.1)),
        // Nor is the name in a sub-folder.
        ("lighting/sub/global.json", lighting("t:nested", 0.1)),
    ]]);
    let settings = stack.deferred_settings();
    assert_eq!(settings.defaults.lighting, Some(lighting_id("t:l")));
    assert_eq!(
        settings.defaults.atmosphere,
        Some(AtmosphereSettingsId::parse("t:a"))
    );
    assert_eq!(settings.defaults.water, Some(WaterSettingsId::parse("t:w")));
    assert_eq!(
        settings.defaults.color_grading,
        Some(ColorGradingSettingsId::parse("t:g"))
    );
    assert!(settings.default_atmosphere().is_some());
    assert!(settings.default_water().is_some());
    assert!(settings.default_color_grading().is_some());
    // Files of sub-folders are definitions (a directory pack lists them).
    assert!(settings.lighting.contains(&lighting_id("t:nested")));
}

#[test]
fn a_default_file_without_an_identifier_leaves_the_choice_to_a_lower_pack() {
    let stack = stack(&[
        &[("lighting/global.json", doc("1.26.0", LIGHTING, ""))],
        &[("lighting/global.json", lighting("t:lower", 0.3))],
    ]);
    let settings = stack.deferred_settings();
    assert_eq!(settings.defaults.lighting, Some(lighting_id("t:lower")));
    // The unnamed file loaded: an information (ours), no error, and no
    // entry.
    assert_eq!(settings.lighting.len(), 1);
    assert_eq!(
        located(settings.diagnostics(), Severity::Info)
            .iter()
            .map(|f| (f.0, f.1, f.2.as_str()))
            .collect::<Vec<_>>(),
        [("MissingIdentifier", Some(0), "lighting/global.json")]
    );
    assert_eq!(located(settings.diagnostics(), Severity::Error), []);
    assert!(settings.lighting.rejected().is_empty());
}

#[test]
fn a_default_file_skipped_as_a_duplicate_decides_nothing() {
    let stack = stack(&[
        &[
            ("lighting/a.json", lighting("t:l", 0.1)),
            ("lighting/global.json", lighting("t:l", 0.2)),
        ],
        &[("lighting/global.json", lighting("t:lower", 0.3))],
    ]);
    let settings = stack.deferred_settings();
    assert_eq!(settings.defaults.lighting, Some(lighting_id("t:lower")));
}

#[test]
fn l09_a_default_file_that_fails_leaves_the_games_values() {
    let stack = stack(&[&[
        ("lighting/global.json", BROKEN.to_owned()),
        ("lighting/other.json", lighting("t:other", 0.5)),
    ]]);
    let settings = stack.deferred_settings();
    assert_eq!(settings.defaults.lighting, None);
    assert!(settings.default_lighting().is_none());
    assert!(settings.lighting.contains(&lighting_id("t:other")));
    // No "could not find" message: nothing named a default.
    assert_eq!(located(settings.diagnostics(), Severity::Warning), []);
}

#[test]
fn k03_the_default_cubemap_is_the_fixed_identifier() {
    let cubemap = |identifier: &str| doc("1.21.130", CUBEMAP, &named(identifier, ""));
    let with = stack(&[
        &[("cubemaps/night.json", cubemap("t:night"))],
        &[(
            "cubemaps/any_name.json",
            cubemap("minecraft:default_cubemap"),
        )],
    ]);
    let settings = with.deferred_settings();
    assert_eq!(
        origin(settings.default_cubemap().unwrap()),
        (Some(1), "cubemaps/any_name.json")
    );
    let without = stack(&[&[("cubemaps/night.json", cubemap("t:night"))]]);
    let settings = without.deferred_settings();
    assert!(settings.default_cubemap().is_none());
    assert_eq!(
        settings.defaults.cubemap.as_str(),
        "minecraft:default_cubemap"
    );
    // K2: the game's own values stand in, without a message.
    assert_eq!(settings.diagnostics().count(), 0);
}

#[test]
fn h05_the_highest_loadable_single_file_wins_deviation() {
    // vanilla = "silent" about the lower files.
    let fallthrough = stack(&[
        &[("shadows/global.json", shadows(0))],
        &[("shadows/global.json", shadows(32))],
        &[("shadows/global.json", shadows(64))],
    ]);
    let settings = fallthrough.deferred_settings();
    let used = settings.shadow_settings().unwrap();
    assert_eq!(origin(used), (Some(1), "shadows/global.json"));
    assert_eq!(used.texel_size, Some(32));
    assert_eq!(
        located(settings.diagnostics(), Severity::Error)
            .iter()
            .map(|f| (f.0, f.1))
            .collect::<Vec<_>>(),
        [("InvalidValue", Some(0))]
    );
    assert_eq!(
        located(settings.diagnostics(), Severity::Info)
            .iter()
            .map(|f| (f.0, f.1, f.2.as_str()))
            .collect::<Vec<_>>(),
        [("ShadowedDefinition", Some(2), "shadows/global.json")]
    );
    let all = stack(&[
        &[("shadows/global.json", shadows(8))],
        &[("shadows/global.json", shadows(32))],
        &[("shadows/global.json", shadows(64))],
    ]);
    let settings = all.deferred_settings();
    assert_eq!(
        origin(settings.shadow_settings().unwrap()),
        (Some(0), "shadows/global.json")
    );
    assert_eq!(
        located(settings.diagnostics(), Severity::Info)
            .iter()
            .map(|f| (f.0, f.1))
            .collect::<Vec<_>>(),
        [
            ("ShadowedDefinition", Some(1)),
            ("ShadowedDefinition", Some(2))
        ]
    );
    assert_eq!(settings.diagnostics().count(), 2);
}

#[test]
fn p3_the_highest_loadable_pbr_file_wins_deviation() {
    let stack = stack(&[
        &[("pbr/global.json", BROKEN.to_owned())],
        &[("pbr/global.json", pbr(11))],
        &[("pbr/global.json", pbr(22))],
    ]);
    let settings = stack.deferred_settings();
    let used = settings.pbr_fallback_settings().unwrap();
    assert_eq!(origin(used), (Some(1), "pbr/global.json"));
    assert_eq!(used.blocks().metalness, 11);
    assert_eq!(used.items(), Mers::DEFAULT);
    assert_eq!(
        located(settings.diagnostics(), Severity::Info)
            .iter()
            .map(|f| (f.0, f.1, f.2.as_str()))
            .collect::<Vec<_>>(),
        [("ShadowedDefinition", Some(2), "pbr/global.json")]
    );
}

#[test]
fn t02_local_lights_merge_per_block_deviation() {
    // vanilla = "silent" about the hidden entries.
    let stack = stack(&[
        &[(
            "local_lighting/local_lighting.json",
            local(r#""minecraft:torch": {"light_color": [255, 0, 0]}"#),
        )],
        &[(
            "local_lighting/local_lighting.json",
            local(
                r#""minecraft:torch": {"light_color": [0, 0, 255]},
                   "minecraft:lantern": {"light_color": [1, 1, 1]}"#,
            ),
        )],
        &[(
            "local_lighting/local_lighting.json",
            local(
                r#""minecraft:torch": {"light_color": [0, 255, 0]},
                   "minecraft:lantern": {"light_color": [2, 2, 2]},
                   "minecraft:sea_pickle": {"light_type": "point_light"}"#,
            ),
        )],
    ]);
    let settings = stack.deferred_settings();
    let lights = &settings.local_lighting.lights;
    let block = |name: &str| BlockId::parse(name.to_owned());
    assert_eq!(lights.len(), 3);
    assert_eq!(
        lights[&block("minecraft:torch")].light_color,
        Some(Rgb8::new(255, 0, 0))
    );
    assert_eq!(
        lights[&block("minecraft:lantern")].light_color,
        Some(Rgb8::new(1, 1, 1))
    );
    assert_eq!(
        lights[&block("minecraft:sea_pickle")].light_type,
        Some(LightType::PointLight)
    );
    let pack = |name: &str| {
        settings
            .local_lighting
            .origin(&BlockId::parse(name.to_owned()))
            .and_then(|origin| origin.pack.index())
    };
    assert_eq!(pack("minecraft:torch"), Some(0));
    assert_eq!(pack("minecraft:lantern"), Some(1));
    assert_eq!(pack("minecraft:sea_pickle"), Some(2));
    assert_eq!(settings.local_lighting.layers().len(), 3);
    let found: Vec<_> = settings
        .diagnostics()
        .map(|f| {
            (
                f.code.name(),
                f.severity,
                f.location
                    .pack
                    .as_ref()
                    .and_then(bedrock_addon::pack::PackRef::index),
                f.location.json_path.to_string(),
            )
        })
        .collect();
    assert_eq!(
        found,
        [
            (
                "ShadowedDefinition",
                Severity::Info,
                Some(1),
                "minecraft:local_light_settings/minecraft:torch".to_owned()
            ),
            (
                "ShadowedDefinition",
                Severity::Info,
                Some(2),
                "minecraft:local_light_settings/minecraft:torch".to_owned()
            ),
            (
                "ShadowedDefinition",
                Severity::Info,
                Some(2),
                "minecraft:local_light_settings/minecraft:lantern".to_owned()
            ),
        ]
    );
}

#[test]
fn t3a_a_local_light_file_that_fails_contributes_nothing() {
    let stack = stack(&[
        &[(
            "local_lighting/local_lighting.json",
            local(
                r#""minecraft:torch": {"light_color": [255, 0, 0]},
                   "minecraft:lantern": {"light_type": "spot_light"}"#,
            ),
        )],
        &[(
            "local_lighting/local_lighting.json",
            local(r#""minecraft:torch": {"light_color": [0, 0, 255]}"#),
        )],
    ]);
    let settings = stack.deferred_settings();
    let lights = &settings.local_lighting.lights;
    assert_eq!(lights.len(), 1);
    assert_eq!(
        lights[&BlockId::parse("minecraft:torch")].light_color,
        Some(Rgb8::new(0, 0, 255))
    );
    assert_eq!(
        located(settings.diagnostics(), Severity::Error)
            .iter()
            .map(|f| (f.0, f.1))
            .collect::<Vec<_>>(),
        [("InvalidEnumValue", Some(0))]
    );
    assert_eq!(located(settings.diagnostics(), Severity::Info), []);
}

#[test]
fn c20_h04_t03_c15_files_the_game_does_not_read_are_not_read_deviation() {
    // vanilla = "not read". Each of these would be an error if it were parsed.
    let stack = stack(&[&[
        ("lighting/extra.JSON", BROKEN.to_owned()),
        ("lighting/notes.txt", BROKEN.to_owned()),
        ("lighting/json", BROKEN.to_owned()),
        ("water/water.jsonc", BROKEN.to_owned()),
        ("shadows/other.json", BROKEN.to_owned()),
        ("shadows/sub/global.json", BROKEN.to_owned()),
        ("pbr/blocks.json", BROKEN.to_owned()),
        ("local_lighting/extra.json", BROKEN.to_owned()),
        ("point_lights/global.json", BROKEN.to_owned()),
        ("lighting/ok.json", lighting("t:l", 0.5)),
        ("shadows/global.json", shadows(8)),
    ]]);
    let settings = stack.deferred_settings();
    assert_eq!(located(settings.diagnostics(), Severity::Error), []);
    assert_eq!(located(settings.diagnostics(), Severity::Warning), []);
    assert_eq!(settings.lighting.len(), 1);
    assert_eq!(settings.shadow_settings().unwrap().texel_size, Some(8));
    assert_eq!(
        settings.shadow_settings().unwrap().shadow_style(),
        ShadowStyle::SoftShadows
    );
    // E11, H3, P3, T0 and `point_lights/`: one `IgnoredFile` info each (ours; vanilla is
    // silent), in the order of the pack's listing; the two files that are read get none.
    let ignored = |path: &str| {
        (
            "IgnoredFile",
            Some(0),
            path.to_owned(),
            "file ignored".to_owned(),
        )
    };
    assert_eq!(
        located(settings.diagnostics(), Severity::Info),
        [
            ignored("lighting/extra.JSON"),
            ignored("lighting/json"),
            ignored("lighting/notes.txt"),
            ignored("local_lighting/extra.json"),
            ignored("pbr/blocks.json"),
            ignored("point_lights/global.json"),
            ignored("shadows/other.json"),
            ignored("shadows/sub/global.json"),
            ignored("water/water.jsonc"),
        ]
    );
}

#[test]
fn ignored_files_are_reported_per_pack_from_the_top_and_only_below_the_settings_folders_deviation()
{
    let stack = stack(&[
        &[
            ("cubemaps/readme.md", BROKEN.to_owned()),
            ("lighting/ok.json", lighting("t:l", 0.5)),
        ],
        &[
            // No settings folder: a file named like one, a folder that only starts like one.
            ("lighting", BROKEN.to_owned()),
            ("lighting_notes/a.txt", BROKEN.to_owned()),
            ("textures/pbr/blocks.txt", BROKEN.to_owned()),
            ("atmospherics/a.JSON", BROKEN.to_owned()),
            ("color_grading/deep/b.txt", BROKEN.to_owned()),
        ],
        &[("pbr/global.json.bak", BROKEN.to_owned())],
    ]);
    let settings = stack.deferred_settings();
    assert_eq!(located(settings.diagnostics(), Severity::Error), []);
    let ignored = |pack: usize, path: &str| {
        (
            "IgnoredFile",
            Some(pack),
            path.to_owned(),
            "file ignored".to_owned(),
        )
    };
    assert_eq!(
        located(settings.diagnostics(), Severity::Info),
        [
            ignored(0, "cubemaps/readme.md"),
            ignored(1, "atmospherics/a.JSON"),
            ignored(1, "color_grading/deep/b.txt"),
            ignored(2, "pbr/global.json.bak"),
        ]
    );
}

#[test]
fn a_nested_local_light_file_is_read_by_its_suffix_deviation() {
    // The game compares the end of the path with `local_lighting/local_lighting.json`.
    let stack = stack(&[&[
        (
            "local_lighting/more/local_lighting/local_lighting.json",
            local(r#""minecraft:lantern": {}"#),
        ),
        ("local_lighting/more/local_lighting.json", BROKEN.to_owned()),
    ]]);
    let settings = stack.deferred_settings();
    assert_eq!(settings.local_lighting.lights.len(), 1);
    // T0: the file whose path does not end that way is not read, with an info.
    assert_eq!(
        located(settings.diagnostics(), Severity::Info),
        [(
            "IgnoredFile",
            Some(0),
            "local_lighting/more/local_lighting.json".to_owned(),
            "file ignored".to_owned()
        )]
    );
    assert_eq!(settings.diagnostics().count(), 1);
}

#[test]
fn two_thousand_files_of_one_pack_and_a_lower_packs_combine() {
    // The content of a large stack; the cost of the combine is measured on the combine itself
    // (src/render/deferred/tests.rs, the_files_of_a_stack_are_combined_…).
    let mut files: Vec<(String, String)> = (0..2_000)
        .map(|i| {
            // Every fourth file repeats an identifier of its pack.
            let id = if i % 4 == 3 { i - 1 } else { i };
            (
                format!("lighting/f{i:05}.json"),
                lighting(&format!("t:l{id}"), 0.5),
            )
        })
        .collect();
    files.push((
        "lighting/global.json".to_owned(),
        lighting("t:default", 0.5),
    ));
    let borrowed: Vec<(&str, String)> = files
        .iter()
        .map(|(path, text)| (path.as_str(), text.clone()))
        .collect();
    let lower: Vec<(&str, String)> = borrowed.iter().take(500).cloned().collect();
    let stack = stack(&[borrowed.as_slice(), lower.as_slice()]);
    let settings = stack.deferred_settings();
    assert_eq!(settings.lighting.len(), 1_501);
    // The second file of an identifier in its pack: 500 in the top pack, 125 in the lower.
    assert_eq!(
        located(settings.diagnostics(), Severity::Error).len(),
        500 + 125
    );
    // The lower pack's other 375 files are shadowed.
    assert_eq!(located(settings.diagnostics(), Severity::Info).len(), 375);
    assert_eq!(settings.defaults.lighting, Some(lighting_id("t:default")));
}

// ---- the link pass -------------------------------------------------------------------------

/// The link findings of one resource pack with `files` over one behaviour pack with
/// `behavior`.
fn link(files: &[(&'static str, String)], behavior: &[(&'static str, String)]) -> Diagnostics {
    let mut resource = rp(1);
    for (path, text) in files {
        resource = resource.file(path, leak(text.clone()).as_bytes());
    }
    let mut data = bp(2);
    for (path, text) in behavior {
        data = data.file(path, leak(text.clone()).as_bytes());
    }
    let (resource, data) = (resource.open(), data.open());
    let entry = |pack: &bedrock_addon::pack::Pack| {
        format!(
            r#"[{{"pack_id": "{}", "version": [1, 0, 0]}}]"#,
            pack.id().uuid
        )
    };
    let world = WorldPacks::parse(
        Some(entry(&data).as_bytes()),
        Some(entry(&resource).as_bytes()),
    )
    .expect("a world list");
    let mut repository = PackRepository::new();
    repository.add_pack(resource, PackOrigin::User);
    repository.add_pack(data, PackOrigin::User);
    repository.stacks(&world, LoadOptions::default()).link()
}

/// `(code name, file path, JSON path, message)` of each link finding, every one a kept error.
fn link_warnings(d: &Diagnostics) -> Vec<(&'static str, String, String, String)> {
    d.iter()
        .filter(|f| f.phase == Phase::Link)
        .inspect(|f| assert!(f.severity == Severity::Error && f.kept, "{f:?}"))
        .map(|f| {
            (
                f.code.name(),
                f.location
                    .path
                    .as_ref()
                    .map(PackPath::to_string)
                    .unwrap_or_default(),
                f.location.json_path.to_string(),
                f.code.to_string(),
            )
        })
        .collect()
}

#[test]
fn t04_t05_a_local_light_of_no_block_is_an_unresolved_reference_warning() {
    let block = r#"{"format_version": "1.26.20", "minecraft:block": {"description": {"identifier": "demo:lamp"}}}"#;
    let d = link(
        &[(
            "local_lighting/local_lighting.json",
            local(
                r#""demo:nothing": {"light_type": "point_light"},
                   "demo:lamp": {"light_type": "point_light"},
                   "minecraft:torch": {"light_type": "point_light"}"#,
            ),
        )],
        &[("blocks/lamp.json", block.to_owned())],
    );
    // The custom block of the behaviour pack and the vanilla block resolve.
    assert_eq!(
        link_warnings(&d),
        [(
            "UnresolvedReference",
            "local_lighting/local_lighting.json".to_owned(),
            "minecraft:local_light_settings/demo:nothing".to_owned(),
            "demo:nothing is not a block".to_owned()
        )]
    );
}

#[test]
fn w06_a_caustics_texture_that_is_missing_or_misshapen_is_a_warning() {
    // A 16 x 40 PNG header: the height is no multiple of the width.
    let mut png = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 0, 0, 0, 13];
    png.extend_from_slice(b"IHDR");
    png.extend_from_slice(&16_u32.to_be_bytes());
    png.extend_from_slice(&40_u32.to_be_bytes());
    png.extend_from_slice(&[8, 6, 0, 0, 0, 0, 0, 0, 0]);
    let caustics = |identifier: &str, texture: &str| {
        doc(
            "1.26.20",
            WATER,
            &named(
                identifier,
                &format!(r#""caustics": {{"texture": "{texture}"}}"#),
            ),
        )
    };
    let mut resource = rp(1)
        .file(
            "water/missing.json",
            leak(caustics("t:missing", "textures/environment/none")).as_bytes(),
        )
        .file(
            "water/uneven.json",
            leak(caustics("t:uneven", "textures/environment/uneven")).as_bytes(),
        )
        .file(
            "water/default.json",
            leak(doc("1.26.20", WATER, &named("t:default", ""))).as_bytes(),
        );
    resource = resource.file(
        "textures/environment/uneven.png",
        Box::leak(png.into_boxed_slice()),
    );
    let resource = resource.open();
    let world = WorldPacks::parse(
        None,
        Some(
            format!(
                r#"[{{"pack_id": "{}", "version": [1, 0, 0]}}]"#,
                resource.id().uuid
            )
            .as_bytes(),
        ),
    )
    .expect("a world list");
    let mut repository = PackRepository::new();
    repository.add_pack(resource, PackOrigin::User);
    let d = repository.stacks(&world, LoadOptions::default()).link();
    assert_eq!(
        link_warnings(&d),
        [
            (
                "TextureNotFound",
                "water/missing.json".to_owned(),
                "minecraft:water_settings/caustics/texture".to_owned(),
                "Resource pack caustics texture could not be loaded. No texture could be found at \"textures/environment/none\".".to_owned()
            ),
            (
                "InvalidTexture",
                "water/uneven.json".to_owned(),
                "minecraft:water_settings/caustics/texture".to_owned(),
                "Caustics texture \"textures/environment/uneven\" provided in resource pack does not evenly divide into frames. Ensure the texture height is evenly divisible by the width.".to_owned()
            ),
        ]
    );
}

// ---- the capability hint -------------------------------------------------------------------

/// A resource pack `n` with `min_engine_version`, `capabilities` and `files`.
fn capable(
    n: u128,
    min_engine_version: [u16; 3],
    capabilities: &[&str],
    files: &[(&'static str, String)],
) -> bedrock_addon::pack::Pack {
    let uuid = uuid::Uuid::from_u128(n);
    let module = uuid::Uuid::from_u128(n ^ (0xabcd_u128 << 64));
    let [major, minor, patch] = min_engine_version;
    let capabilities: Vec<String> = capabilities.iter().map(|c| format!("\"{c}\"")).collect();
    let manifest = format!(
        r#"{{"format_version": 2,
            "header": {{"name": "rp {n}", "description": "", "uuid": "{uuid}", "version": [1, 0, 0],
                        "min_engine_version": [{major}, {minor}, {patch}]}},
            "modules": [{{"type": "resources", "uuid": "{module}", "version": [1, 0, 0]}}],
            "capabilities": [{}]}}"#,
        capabilities.join(", ")
    );
    let mut all = vec![(
        PackPath::from_static("manifest.json"),
        bytes::Bytes::from(manifest),
    )];
    all.extend(files.iter().map(|(path, text)| {
        (
            PackPath::from_static(path),
            bytes::Bytes::from(text.clone()),
        )
    }));
    bedrock_addon::pack::Pack::from_files(all, &bedrock_addon::pack::OpenOptions::default())
        .expect("the pack opens")
}

/// The stack indices of the packs that got a `CapabilityHint`, which must be the only
/// findings.
fn hinted(packs: Vec<bedrock_addon::pack::Pack>) -> Vec<Option<usize>> {
    let stack = bedrock_addon::pack::PackStack::<bedrock_addon::pack::Resource>::of(
        packs,
        LoadOptions::default(),
    )
    .unwrap();
    let settings = stack.deferred_settings();
    settings
        .diagnostics()
        .map(|f| {
            assert_eq!(
                (f.code.name(), f.severity, f.location.path.as_ref()),
                ("CapabilityHint", Severity::Info, None),
                "{f:?}"
            );
            // The folder's sentence.
            assert_eq!(
                f.code.to_string(),
                "render configuration files are only used when Vibrant Visuals is available; add the \"pbr\" capability"
            );
            f.location.pack.as_ref().and_then(bedrock_addon::pack::PackRef::index)
        })
        .collect()
}

#[test]
fn c14_render_configs_without_the_pbr_capability_get_a_hint_deviation() {
    // Ours; vanilla has no message: the files load either way.
    let files = || vec![("lighting/a.json", lighting("t:l", 0.5))];
    let two = || {
        vec![
            ("lighting/a.json", lighting("t:l", 0.5)),
            ("shadows/global.json", shadows(8)),
        ]
    };
    // One hint per pack, however many files it ships.
    assert_eq!(
        hinted(vec![capable(1, [1, 21, 120], &[], &two())]),
        [Some(0)]
    );
    // The floor is 1.21.110, itself included.
    assert_eq!(
        hinted(vec![capable(1, [1, 21, 110], &[], &files())]),
        [Some(0)]
    );
    assert_eq!(hinted(vec![capable(1, [1, 21, 109], &[], &files())]), []);
    assert_eq!(hinted(vec![capable(1, [1, 20, 200], &[], &files())]), []);
    assert_eq!(
        hinted(vec![capable(1, [1, 22, 0], &[], &files())]),
        [Some(0)]
    );
    assert_eq!(
        hinted(vec![capable(1, [1, 26, 50], &[], &files())]),
        [Some(0)]
    );
    // Either capability is enough.
    assert_eq!(
        hinted(vec![capable(1, [1, 21, 120], &["pbr"], &files())]),
        []
    );
    assert_eq!(
        hinted(vec![capable(1, [1, 21, 120], &["raytraced"], &files())]),
        []
    );
    assert_eq!(
        hinted(vec![capable(1, [1, 21, 120], &["chemistry"], &files())]),
        [Some(0)]
    );
    // A pack without a settings file has nothing to hint at; a file that fails still counts.
    assert_eq!(hinted(vec![capable(1, [1, 21, 120], &[], &[])]), []);
    // Per pack, in stack order: the middle pack declares `pbr`.
    assert_eq!(
        hinted(vec![
            capable(
                1,
                [1, 21, 120],
                &[],
                &[("lighting/a.json", lighting("t:l1", 0.5))]
            ),
            capable(
                2,
                [1, 21, 120],
                &["pbr"],
                &[("lighting/a.json", lighting("t:l2", 0.5))]
            ),
            capable(3, [1, 21, 120], &[], &[("water/w.json", water("t:w"))]),
            capable(
                4,
                [1, 21, 100],
                &[],
                &[("lighting/a.json", lighting("t:l4", 0.5))]
            ),
        ]),
        [Some(0), Some(2)]
    );
}

#[test]
fn c14_the_hinted_packs_files_load_all_the_same() {
    let stack = bedrock_addon::pack::PackStack::<bedrock_addon::pack::Resource>::of(
        vec![capable(
            1,
            [1, 21, 120],
            &[],
            &[("lighting/a.json", lighting("t:l", 0.5))],
        )],
        LoadOptions::default(),
    )
    .unwrap();
    let settings = stack.deferred_settings();
    assert!(settings.lighting.contains(&lighting_id("t:l")));
    assert_eq!(
        settings
            .diagnostics()
            .map(|f| f.code.to_string())
            .collect::<Vec<_>>(),
        [
            "render configuration files are only used when Vibrant Visuals is available; add the \"pbr\" capability"
        ]
    );
}

#[test]
fn c14_a_base_game_pack_gets_no_hint() {
    // The rule is about third-party packs; vanilla's own layers declare no capability.
    let files = [("lighting/a.json", lighting("t:l", 0.5))];
    let hints = |origin: PackOrigin| {
        let pack = capable(1, [1, 21, 120], &[], &files);
        let entry = format!(
            r#"[{{"pack_id": "{}", "version": [1, 0, 0]}}]"#,
            pack.id().uuid
        );
        let world = WorldPacks::parse(None, Some(entry.as_bytes())).expect("a world list");
        let mut repository = PackRepository::new();
        repository.add_pack(pack, origin);
        let stacks = repository.stacks(&world, LoadOptions::default());
        let settings = stacks.resource.deferred_settings();
        assert!(settings.lighting.contains(&lighting_id("t:l")));
        settings.diagnostics().count()
    };
    assert_eq!(hints(PackOrigin::Package), 0);
    assert_eq!(hints(PackOrigin::User), 1);
}

#[test]
fn a_lower_single_file_that_fails_keeps_its_findings_deviation() {
    // Ours: the game stops reading at the first copy that loads.
    let stack = stack(&[
        &[("shadows/global.json", shadows(8))],
        &[("shadows/global.json", shadows(0))],
        &[("shadows/global.json", shadows(64))],
    ]);
    let settings = stack.deferred_settings();
    assert_eq!(
        origin(settings.shadow_settings().unwrap()),
        (Some(0), "shadows/global.json")
    );
    assert_eq!(
        located(settings.diagnostics(), Severity::Error)
            .iter()
            .map(|f| (f.0, f.1))
            .collect::<Vec<_>>(),
        [("InvalidValue", Some(1))]
    );
    assert_eq!(
        located(settings.diagnostics(), Severity::Info)
            .iter()
            .map(|f| (f.0, f.1))
            .collect::<Vec<_>>(),
        [("ShadowedDefinition", Some(2))]
    );
    assert_eq!(settings.shadows.rejected().len(), 1);
}

#[test]
fn an_empty_identifier_is_not_indexed_and_names_no_default() {
    let unnamed = doc("1.26.0", LIGHTING, &named("", ""));
    let stack = stack(&[
        &[("lighting/global.json", unnamed)],
        &[("lighting/global.json", lighting("t:lower", 0.3))],
    ]);
    let settings = stack.deferred_settings();
    assert_eq!(settings.lighting.len(), 1);
    assert!(!settings.lighting.contains(&lighting_id("")));
    assert_eq!(settings.defaults.lighting, Some(lighting_id("t:lower")));
    // Both notes are ours.
    assert_eq!(
        located(settings.diagnostics(), Severity::Info)
            .iter()
            .map(|f| (f.0, f.1, f.2.as_str()))
            .collect::<Vec<_>>(),
        [
            ("NamespaceRule", Some(0), "lighting/global.json"),
            ("MissingIdentifier", Some(0), "lighting/global.json")
        ]
    );
}
