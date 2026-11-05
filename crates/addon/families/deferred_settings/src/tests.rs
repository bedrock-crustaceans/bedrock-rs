//! Unit tests of the deferred-settings module: the link checker over a hand-built group, the
//! default-identifier rule and the walk's file filter.

use super::link::{self, Targets, Texture};
use super::load::{self, is_read};
use super::{DeferredSettings, LightingSettings, LocalLightSettings, WaterSettings};
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticDetail, Phase, Severity};
use bedrock_addon_core::document::{CombineRule, Content, Contribution, Registry, RegistryBuilder};
use bedrock_addon_core::id::{BlockId, LightingSettingsId};
use bedrock_addon_core::load::{LoadContext, LoadOptions};
use bedrock_addon_core::load::{StackCx, WalkFile};
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::pack_identity::{PackRef, PackSide};

/// A group from `(path, text)` files of one pack, through the loader's own parse and combine.
fn group(files: &[(&str, &str)]) -> DeferredSettings {
    let loader = load::load_group();
    let cx = LoadContext::detached();
    let parsed = files
        .iter()
        .map(|(path, text)| {
            let path = PackPath::new(path).unwrap();
            (loader.parse)(
                &WalkFile {
                    pack: PackRef::app_data(),
                    side: PackSide::Resource,
                    path: path.clone(),
                    key: path,
                    bytes: bytes::Bytes::copy_from_slice(text.as_bytes()),
                },
                &cx,
            )
        })
        .collect();
    let options = LoadOptions::default();
    (loader.combine)(parsed, &StackCx::new(&options, &[]))
}

/// A stack in which the named blocks exist and each texture path has the given image.
struct World {
    blocks: &'static [&'static str],
    textures: &'static [(&'static str, Texture)],
}

impl Targets for World {
    fn block(&self, id: &BlockId) -> bool {
        self.blocks.iter().any(|name| BlockId::parse(*name) == *id)
    }

    fn texture(&self, path: &str) -> Texture {
        self.textures
            .iter()
            .find(|(known, _)| *known == path)
            .map_or(Texture::Missing, |(_, texture)| *texture)
    }
}

/// `(code name, file, JSON path, message, detail)` of each finding.
fn found(
    d: &bedrock_addon_core::diagnostic::Diagnostics,
) -> Vec<(&'static str, String, String, String, String)> {
    d.iter()
        .map(|f: &Diagnostic| {
            assert_eq!(f.phase, Phase::Link);
            // The game logs these at the error level and goes on: kept errors.
            assert_eq!(f.severity, Severity::Error);
            assert!(f.kept);
            (
                f.code.name(),
                f.location
                    .path
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                f.location.json_path.to_string(),
                f.code.to_string(),
                f.detail
                    .as_deref()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
            )
        })
        .collect()
}

const LOCAL: &str = r#"{"format_version": "1.21.120", "minecraft:local_light_settings": {
    "demo:nothing": {"light_type": "point_light"},
    "demo:lamp": {"light_type": "point_light"},
    "minecraft:torch": {},
    "demo:also_nothing": {}
}}"#;

#[test]
fn t04_a_local_light_of_no_block_is_an_unresolved_reference() {
    let settings = group(&[("local_lighting/local_lighting.json", LOCAL)]);
    let world = World {
        blocks: &["demo:lamp", "minecraft:torch"],
        textures: &[],
    };
    assert_eq!(
        found(&link::check(&settings, &world)),
        [
            (
                "UnresolvedReference",
                "local_lighting/local_lighting.json".to_owned(),
                "minecraft:local_light_settings/demo:nothing".to_owned(),
                "demo:nothing is not a block".to_owned(),
                ": block 'demo:nothing'".to_owned(),
            ),
            (
                "UnresolvedReference",
                "local_lighting/local_lighting.json".to_owned(),
                "minecraft:local_light_settings/demo:also_nothing".to_owned(),
                "demo:also_nothing is not a block".to_owned(),
                ": block 'demo:also_nothing'".to_owned(),
            ),
        ]
    );
    // The entries the check names stay in the group: the game drops them at run time.
    assert_eq!(settings.local_lighting.lights.len(), 4);
    assert!(
        settings
            .local_lighting
            .lights
            .contains_key(&BlockId::parse("demo:nothing"))
    );
}

#[test]
fn t05_a_local_light_of_a_block_of_the_stack_resolves() {
    let settings = group(&[("local_lighting/local_lighting.json", LOCAL)]);
    let world = World {
        blocks: &[
            "demo:lamp",
            "minecraft:torch",
            "demo:nothing",
            "demo:also_nothing",
        ],
        textures: &[],
    };
    assert!(link::check(&settings, &world).is_empty());
    // The lights themselves are untouched by the check.
    assert_eq!(settings.local_lighting.lights.len(), 4);
}

/// A water file named `identifier` whose caustics name `texture`.
fn water(identifier: &str, texture: Option<&str>) -> String {
    let caustics = texture.map_or(String::new(), |texture| {
        format!(r#", "caustics": {{"texture": "{texture}"}}"#)
    });
    format!(
        r#"{{"format_version": "1.26.20", "minecraft:water_settings": {{"description": {{"identifier": "{identifier}"}}{caustics}}}}}"#
    )
}

#[test]
fn w06_a_caustics_texture_must_be_an_image_of_whole_frames() {
    const AT: &str = "minecraft:water_settings/caustics/texture";
    let files = [
        (
            "water/a_missing.json",
            water("t:missing", Some("textures/none")),
        ),
        (
            "water/b_broken.json",
            water("t:broken", Some("textures/broken")),
        ),
        (
            "water/c_uneven.json",
            water("t:uneven", Some("textures/uneven")),
        ),
        (
            "water/d_frames.json",
            water("t:frames", Some("textures/frames")),
        ),
        (
            "water/e_square.json",
            water("t:square", Some("textures/square")),
        ),
        ("water/f_default.json", water("t:default", None)),
    ];
    let borrowed: Vec<(&str, &str)> = files.iter().map(|(p, t)| (*p, t.as_str())).collect();
    let settings = group(&borrowed);
    assert_eq!(settings.water.len(), 6);
    let world = World {
        blocks: &[],
        textures: &[
            ("textures/broken", Texture::Undecodable),
            (
                "textures/uneven",
                Texture::Image {
                    width: 16,
                    height: 40,
                },
            ),
            (
                "textures/frames",
                Texture::Image {
                    width: 16,
                    height: 48,
                },
            ),
            (
                "textures/square",
                Texture::Image {
                    width: 16,
                    height: 16,
                },
            ),
        ],
    };
    assert_eq!(
        found(&link::check(&settings, &world)),
        [
            (
                "TextureNotFound",
                "water/a_missing.json".to_owned(),
                AT.to_owned(),
                "Resource pack caustics texture could not be loaded. No texture could be found at \"textures/none\".".to_owned(),
                ": texture 'textures/none'".to_owned(),
            ),
            (
                "InvalidTexture",
                "water/b_broken.json".to_owned(),
                AT.to_owned(),
                // An image that cannot be decoded: the game's text.
                "Caustics texture \"textures/broken\" provided in resource pack could not be loaded due to an unknown error. Ensure the texture is a valid image and is in one of the supported image formats.".to_owned(),
                ": texture 'textures/broken'".to_owned(),
            ),
            (
                "InvalidTexture",
                "water/c_uneven.json".to_owned(),
                AT.to_owned(),
                // An image that does not divide into frames: the game's text.
                "Caustics texture \"textures/uneven\" provided in resource pack does not evenly divide into frames. Ensure the texture height is evenly divisible by the width.".to_owned(),
                ": texture 'textures/uneven'".to_owned(),
            ),
        ]
    );
}

#[test]
fn a_caustics_image_without_width_is_no_frame() {
    let file = water("t:w", Some("textures/empty"));
    let settings = group(&[("water/w.json", &file)]);
    let world = World {
        blocks: &[],
        textures: &[(
            "textures/empty",
            Texture::Image {
                width: 0,
                height: 16,
            },
        )],
    };
    assert_eq!(found(&link::check(&settings, &world)).len(), 1);
}

/// Each payload's default identifier that its map lacks is told with the noun of its own
/// message.
#[test]
fn r50_each_payload_has_the_noun_of_its_own_message() {
    use super::{AtmosphereSettings, ColorGradingSettings};
    use bedrock_addon_core::id::{AtmosphereSettingsId, ColorGradingSettingsId, WaterSettingsId};
    fn empty<T: bedrock_addon_core::document::Definition>(options: &LoadOptions) -> Registry<T> {
        RegistryBuilder::new(CombineRule::top_wins(), CombineRule::top_wins(), options).finish()
    }
    let options = LoadOptions::default();
    let lighting = (
        empty::<LightingSettings>(&options),
        LightingSettingsId::parse("demo:l"),
    );
    let atmosphere = (
        empty::<AtmosphereSettings>(&options),
        AtmosphereSettingsId::parse("demo:a"),
    );
    let water = (
        empty::<WaterSettings>(&options),
        WaterSettingsId::parse("demo:w"),
    );
    let grading = (
        empty::<ColorGradingSettings>(&options),
        ColorGradingSettingsId::parse("demo:g"),
    );
    let told: Vec<(&str, Severity, String, &str)> = load::unresolved_defaults(
        (&lighting.0, Some(&lighting.1)),
        (&atmosphere.0, Some(&atmosphere.1)),
        (&water.0, Some(&water.1)),
        (&grading.0, Some(&grading.1)),
    )
    .map(|finding| {
        let kind = match finding.detail.as_deref() {
            Some(DiagnosticDetail::Reference { kind, .. }) => *kind,
            _ => "",
        };
        (
            finding.code.name(),
            finding.severity,
            finding.code.to_string(),
            kind,
        )
    })
    .collect();
    let text = |noun: &str, id: &str| {
        format!("Could not find any matching settings for {noun} identifier '{id}'")
    };
    let kinds: Vec<&str> = told.iter().map(|found| found.3).collect();
    assert_eq!(
        told.iter()
            .map(|(code, severity, text, _)| (*code, *severity, text.as_str()))
            .collect::<Vec<_>>(),
        [
            (
                "UnresolvedReference",
                Severity::Error,
                text("lighting", "demo:l").as_str()
            ),
            (
                "UnresolvedReference",
                Severity::Error,
                text("atmosphere", "demo:a").as_str()
            ),
            (
                "UnresolvedReference",
                Severity::Error,
                text("water", "demo:w").as_str()
            ),
            (
                "UnresolvedReference",
                Severity::Error,
                text("color grading", "demo:g").as_str()
            ),
        ]
    );
    // Each names the kind of its own payload: four different kinds.
    let distinct: std::collections::BTreeSet<&str> = kinds.iter().copied().collect();
    assert_eq!(distinct.len(), 4, "{kinds:?}");
    // A payload whose default file gave no identifier has nothing to say.
    assert_eq!(
        load::unresolved_defaults(
            (&lighting.0, None),
            (&atmosphere.0, None),
            (&water.0, None),
            (&grading.0, None),
        )
        .count(),
        0
    );
}

#[test]
fn r50_a_default_identifier_the_map_lacks_is_the_games_could_not_find() {
    let options = LoadOptions::default();
    let empty: Registry<LightingSettings> =
        RegistryBuilder::new(CombineRule::top_wins(), CombineRule::top_wins(), &options).finish();
    let id = LightingSettingsId::parse("demo:gone");
    let finding = load::unresolved_default(&empty, &id, "lighting").expect("a finding");
    assert_eq!(finding.code.name(), "UnresolvedReference");
    // The game logs it at the error level and goes on with its built-ins: a kept error.
    assert_eq!(finding.severity, Severity::Error);
    assert!(finding.kept);
    assert_eq!(
        finding.code.to_string(),
        "Could not find any matching settings for lighting identifier 'demo:gone'"
    );
    assert!(matches!(
        finding.detail.as_deref(),
        Some(DiagnosticDetail::Reference { id, .. }) if id == "demo:gone"
    ));
    // A map that defines it has nothing to say.
    let document = LightingSettings::parse(
        &PackPath::from_static("lighting/global.json"),
        br#"{"format_version": "1.26.0", "minecraft:lighting_settings": {"description": {"identifier": "demo:gone"}}}"#,
        &LoadContext::detached(),
    )
    .unwrap();
    let mut builder =
        RegistryBuilder::new(CombineRule::top_wins(), CombineRule::top_wins(), &options);
    builder.push(Contribution::Stable(id.clone(), document));
    assert!(load::unresolved_default(&builder.finish(), &id, "lighting").is_none());
    // The loader's own default identifiers are always defined: no such finding.
    let settings = group(&[(
        "lighting/global.json",
        r#"{"format_version": "1.26.0", "minecraft:lighting_settings": {"description": {"identifier": "demo:day"}}}"#,
    )]);
    assert_eq!(
        settings.defaults.lighting,
        Some(LightingSettingsId::parse("demo:day"))
    );
    assert_eq!(settings.diagnostics().count(), 0);
}

#[test]
fn e11_the_walk_reads_only_what_the_game_reads() {
    let read = |path: &str| is_read(&PackPath::new(path).unwrap());
    for path in [
        "lighting/global.json",
        "lighting/sub/x.json",
        "atmospherics/a.json",
        "water/water.json",
        "color_grading/x.json",
        "cubemaps/x.json",
        "shadows/global.json",
        "pbr/global.json",
        "local_lighting/local_lighting.json",
        "local_lighting/x/local_lighting/local_lighting.json",
        // The suffix compare is all the game does below `local_lighting/`.
        "local_lighting/xlocal_lighting/local_lighting.json",
    ] {
        assert!(read(path), "{path}");
    }
    for path in [
        "lighting/extra.JSON",
        "lighting/notes.txt",
        "lighting/x.jso",
        "lighting/json",
        "water/x.json5",
        "shadows/other.json",
        "shadows/Global.json",
        "shadows/sub/global.json",
        "pbr/blocks.json",
        "pbr/sub/global.json",
        "local_lighting/extra.json",
        "local_lighting/local_lighting.JSON",
        "local_lighting/sub/local_lighting.json",
        "global.json",
    ] {
        assert!(!read(path), "{path}");
    }
}

#[test]
fn the_group_types_are_send_and_sync_with_a_stable_debug() {
    fn shared<T: Send + Sync>() {}
    shared::<DeferredSettings>();
    shared::<LightingSettings>();
    shared::<WaterSettings>();
    shared::<LocalLightSettings>();
    let settings = group(&[("local_lighting/local_lighting.json", LOCAL)]);
    assert_eq!(format!("{settings:?}"), format!("{settings:?}"));
}

/// `strtof` reads what C99's does: `(text, value)`; `None` when no character is a number.
#[test]
fn a_keyframe_key_is_read_as_c99_strtof_reads_it() {
    use super::read::strtof;
    let bits = |value: Option<f32>| value.map(f32::to_bits);
    for (text, value) in [
        ("0.5", Some(0.5)),
        ("\t\n\u{b}\u{c}\r 0.5", Some(0.5)),
        ("-0", Some(-0.0)),
        ("1e39", Some(f32::INFINITY)),
        ("-1e39", Some(f32::NEG_INFINITY)),
        ("inf", Some(f32::INFINITY)),
        ("-INFINITY", Some(f32::NEG_INFINITY)),
        ("x", None),
        ("", None),
        ("+", None),
        ("-.", None),
        // Hexadecimal: digits, a point, a binary exponent in decimal digits.
        ("0x10", Some(16.0)),
        ("0X10", Some(16.0)),
        ("0x.8p0", Some(0.5)),
        ("0x.8", Some(0.5)),
        ("0x1.", Some(1.0)),
        ("0xA", Some(10.0)),
        ("0xa.8", Some(10.5)),
        ("-0x1p-1", Some(-0.5)),
        ("-0x0", Some(-0.0)),
        ("0x1p+4", Some(16.0)),
        ("0x1p4.5", Some(16.0)),
        ("0x00000000000000000000000000000001p-1", Some(0.5)),
        ("0x.00000000000000000000000000000001p129", Some(2.0)),
        ("0x8000000000000000000000000p-99", Some(1.0)),
        // Rounded once, to nearest, ties to even: below a half, a half, above a half.
        ("0x1.0000008p0", Some(1.0)),
        ("0x1.000001p0", Some(1.0)),
        ("0x1.000003p0", Some(f32::from_bits(0x3F80_0002))),
        (
            "0x1.000001000000000000000001p0",
            Some(f32::from_bits(0x3F80_0001)),
        ),
        ("0x1.0000018p0", Some(f32::from_bits(0x3F80_0001))),
        // Beyond the type at either end.
        ("0x1p128", Some(f32::INFINITY)),
        ("-0x1p99999999999999999999", Some(f32::NEG_INFINITY)),
        ("0x1p-149", Some(f32::from_bits(1))),
        ("0x1p-151", Some(0.0)),
        ("0x1p-99999999999999999999", Some(0.0)),
        ("0x0p99999999999999999999", Some(0.0)),
        // `0x` without a hexadecimal digit, and an exponent without a digit.
        ("0x", Some(0.0)),
        ("0xp1", Some(0.0)),
        ("0x.", Some(0.0)),
        ("-0xg", Some(-0.0)),
        ("0x1p", Some(1.0)),
        ("0x1p+", Some(1.0)),
        ("0x1e2", Some(482.0)),
    ] {
        assert_eq!(bits(strtof(text)), bits(value), "{text:?}");
    }
    for text in ["nan", "-NaN", "nan(1)", "NANx"] {
        assert!(strtof(text).is_some_and(f32::is_nan), "{text:?}");
    }
}

// ---- cost: the functions that could be quadratic, each on inputs built beforehand ------------

/// Requires `run` on `build(4 n)` to take under eight times `run` on `build(n)`, by the
/// crate's one protocol (`bedrock_addon_core::test_cost`). `run` consumes its input, which is built
/// afresh for each run outside the measurement, and is told the size to check what it did.
#[track_caller]
fn near_linear<I>(what: &str, n: usize, build: impl Fn(usize) -> I, run: impl Fn(I, usize)) {
    bedrock_addon_core::test_cost::assert_linear_consuming_at(
        what,
        n,
        |n| (build(n), n),
        |(input, n)| run(input, n),
    );
}

/// A keyframe object of `n` keys, the times descending and each time written twice (the
/// second with a leading space), so that half of them stand.
fn keyframe_text(n: usize) -> String {
    use std::fmt::Write as _;
    let times = n / 2;
    let mut text = String::from("{");
    for index in (0..times).rev() {
        #[allow(clippy::cast_precision_loss, reason = "small counts")]
        let time = index as f64 / times as f64;
        write!(text, r#""{time:.6}": 1, " {time:.6}": 2,"#).unwrap();
    }
    text.pop();
    text.push('}');
    text
}

#[test]
#[allow(clippy::float_cmp, reason = "exact: the values the text holds")]
fn the_keyframes_of_one_member_are_read_and_merged_in_near_linear_time() {
    use super::read::{self, Node, Range};
    use super::value::Keyframes;
    use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    use bedrock_addon_core::json::JsonSlice;
    near_linear(
        "keyframes of one member",
        2_000,
        keyframe_text,
        |text, n| {
            let node = Node {
                slice: JsonSlice::new(&text),
                at: JsonPath::new(),
            };
            let mut d = Diagnostics::default();
            let read = read::keyframes(&node, &mut d, |n, d| {
                read::float_in(Range::new(0.0, 10.0), n, d)
            });
            let Ok(Keyframes::Frames(frames)) = read else {
                panic!("frames")
            };
            // Half of the keys stand, each the later of its two; the other half is reported.
            assert_eq!(frames.len(), n / 2);
            assert!(frames.windows(2).all(|pair| pair[0].time < pair[1].time));
            assert!(frames.iter().all(|frame| frame.value == 2.0));
            assert_eq!(d.len(), n);
        },
    );
}

#[test]
#[allow(
    clippy::float_cmp,
    reason = "exact: the values the keyframes were built with"
)]
fn the_keyframes_of_one_member_are_merged_in_near_linear_time() {
    use super::read::{self, Node};
    use super::value::Keyframe;
    use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    use bedrock_addon_core::json::JsonSlice;
    // The merge alone, on keyframes already read: `n` of them, the times descending and each
    // time twice, so that the reader's own (linear) cost does not hide the merge's.
    let frames = |n: usize| -> Vec<(Keyframe<f32>, Node<'static>)> {
        let times = n / 2;
        (0..n)
            .rev()
            .map(|index| {
                #[allow(clippy::cast_precision_loss, reason = "small counts")]
                let time = (index / 2) as f32 / times as f32;
                #[allow(clippy::cast_precision_loss, reason = "small counts")]
                let value = (index % 2) as f32;
                let node = Node {
                    slice: JsonSlice::new("1"),
                    at: JsonPath::new(),
                };
                (Keyframe { time, value }, node)
            })
            .collect()
    };
    near_linear("the merge of keyframes", 40_000, frames, |frames, n| {
        let mut d = Diagnostics::default();
        let kept = read::standing(frames, &mut d);
        assert_eq!(kept.len(), n / 2);
        assert!(kept.windows(2).all(|pair| pair[0].time < pair[1].time));
        // The later of each two stands: it was built with the value 0.
        assert!(kept.iter().all(|frame| frame.value == 0.0));
        assert_eq!(d.len(), n);
    });
}

/// A local-light file of `n` blocks, each with a channel that is clamped and an unknown
/// member.
fn local_light_text(n: usize, namespace: &str) -> String {
    let entries: Vec<String> = (0..n)
        .map(|i| {
            format!(
                r#""{namespace}:block_{i}": {{"light_type": "point_light", "light_color": [{}, 300, 3], "note": {i}}}"#,
                i % 256
            )
        })
        .collect();
    format!(
        r#"{{"format_version": "1.21.120", "minecraft:local_light_settings": {{{}}}}}"#,
        entries.join(",")
    )
}

#[test]
fn the_entries_of_a_local_light_file_are_read_and_written_in_near_linear_time() {
    near_linear(
        "local-light entries",
        400,
        |n| local_light_text(n, "test"),
        |text, n| {
            let document = LocalLightSettings::parse(
                &PackPath::from_static("local_lighting/local_lighting.json"),
                text.as_bytes(),
                &LoadContext::detached(),
            )
            .unwrap();
            assert_eq!(document.lights.len(), n);
            // One clamp info and one unknown member per entry.
            assert_eq!(document.diagnostics().len(), 2 * n);
            assert!(document.to_json().unwrap().len() > text.len() / 2);
        },
    );
}

/// The packs and the parsed files of a two-pack stack, as `combine` takes them: in the top
/// pack `n` lighting files (every fourth repeats an identifier of its pack) and a local-light
/// file of `n` blocks; in the lower pack `n / 4` lighting files of the top pack's identifiers
/// and a local-light file of the same `n` blocks. Each pack lists `n` more files the game does
/// not read.
fn stack_files(
    n: usize,
) -> (
    Vec<bedrock_addon_core::load::StackPack>,
    Vec<load::SettingsFile>,
) {
    use bedrock_addon_core::load::{ListedFile, StackPack};
    use bedrock_addon_core::pack_identity::{PackIdVersion, PackOrigin, PackType};
    use bedrock_addon_core::version::SemVersion;
    let loader = load::load_group();
    let cx = LoadContext::detached();
    let lights = local_light_text(n, "test");
    let mut packs = Vec::new();
    let mut parsed = Vec::new();
    for (index, count) in [(0_usize, n), (1, n / 4)] {
        let pack = PackRef::new(
            index,
            PackIdVersion::new(
                uuid::Uuid::from_u128(index as u128 + 1),
                SemVersion::new(1, 0, 0),
                PackType::Resource,
            ),
            format!("pack {index}"),
        );
        let mut files: Vec<(String, String)> = (0..count)
            .map(|i| {
                let id = if i % 4 == 3 { i - 1 } else { i };
                (
                    format!("lighting/f{i:06}.json"),
                    format!(
                        r#"{{"format_version": "1.26.0", "minecraft:lighting_settings": {{"description": {{"identifier": "t:l{id}"}}}}}}"#
                    ),
                )
            })
            .collect();
        files.push((
            "local_lighting/local_lighting.json".to_owned(),
            lights.clone(),
        ));
        for (path, text) in &files {
            let path = PackPath::new(path).unwrap();
            parsed.push((loader.parse)(
                &WalkFile {
                    pack: pack.clone(),
                    side: PackSide::Resource,
                    path: path.clone(),
                    key: path,
                    bytes: bytes::Bytes::copy_from_slice(text.as_bytes()),
                },
                &cx,
            ));
        }
        let listed = (0..n).map(|i| {
            let path = PackPath::new(&format!("lighting/n{i:06}.txt")).unwrap();
            ListedFile {
                key: path.clone(),
                path,
            }
        });
        packs.push(StackPack::new(pack, PackOrigin::Dev, cx.clone()).with_files(listed));
    }
    (packs, parsed)
}

#[test]
fn the_files_of_a_stack_are_combined_in_near_linear_time() {
    let loader = load::load_group();
    near_linear("combine", 1_600, stack_files, |(packs, parsed), n| {
        let options = LoadOptions::default();
        let settings = (loader.combine)(parsed, &StackCx::new(&options, &packs));
        // Three of four of the top pack's files define an identifier; the fourth is a second
        // file of one, and every file of the lower pack is shadowed.
        assert_eq!(settings.lighting.len(), n - n / 4);
        assert_eq!(settings.local_lighting.lights.len(), n);
        let mut infos: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
        let mut errors = 0;
        for finding in settings.diagnostics() {
            match finding.severity {
                Severity::Info => *infos.entry(finding.code.name()).or_default() += 1,
                Severity::Error => errors += 1,
                _ => {}
            }
        }
        assert_eq!(errors, n / 4 + n / 16);
        // The clamped channel of each light of both files, the files the game does not read
        // of both packs, the lower pack's lights and loaded files, all shadowed, and the
        // unknown member of each light (an information).
        assert_eq!(
            infos.into_iter().collect::<Vec<_>>(),
            [
                ("CapabilityHint", 2),
                ("ClampedValue", 2 * n),
                ("IgnoredFile", 2 * n),
                ("ShadowedDefinition", n + n / 4 - n / 16),
                ("UnrecognizedField", 2 * n),
            ]
        );
    });
}

/// Every finding of `found` is in the app-data pack's file `path`.
fn assert_located(found: &bedrock_addon_core::diagnostic::Diagnostics, path: &'static str) {
    assert!(!found.is_empty(), "no finding to place");
    for finding in found {
        assert_eq!(
            finding.location.pack,
            Some(PackRef::app_data()),
            "{finding:?}"
        );
        assert_eq!(
            finding.location.path,
            Some(PackPath::from_static(path)),
            "{finding:?}"
        );
    }
}

#[test]
fn findings_carry_the_file_and_pack() {
    const PATH: &str = "lighting/x.json";
    let cx = LoadContext::detached();
    let path = PackPath::from_static(PATH);
    // A loaded file with a finding (an unknown member), and one that is no JSON.
    let text = r#"{"format_version": "1.26.0", "minecraft:lighting_settings": {"description": {"identifier": "demo:day"}, "x": 1}}"#;
    let document = LightingSettings::parse(&path, text.as_bytes(), &cx).expect("the file loads");
    assert_located(document.diagnostics(), PATH);
    let rejected = LightingSettings::parse(&path, b"\xff", &cx).expect_err("the file is refused");
    assert_located(&rejected.diagnostics, PATH);
    let rejected = LightingSettings::parse(&path, b"{", &cx).expect_err("the file is refused");
    assert_located(&rejected.diagnostics, PATH);
}
