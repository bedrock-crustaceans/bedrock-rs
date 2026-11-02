//! Every ranged member of the eight payloads at both of its bounds, from one table
//! the bound itself loads, the next `f32` beyond
//! it is refused with the range finding, and the finding's text names both bounds.

use bedrock_addon::document::Content;
use bedrock_addon::families::deferred_settings::{
    AtmosphereSettings, ColorGradingSettings, CubemapSettings, LightingSettings, ShadowSettings,
    WaterSettings,
};

use super::common::*;

/// The payload a row belongs to.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Payload {
    Lighting,
    Water,
    Atmosphere,
    Grading,
    Cubemap,
    Shadows,
}

impl Payload {
    /// The payload's key and whether it takes a `description`.
    fn key(self) -> (&'static str, bool) {
        match self {
            Self::Lighting => (LIGHTING, true),
            Self::Water => (WATER, true),
            Self::Atmosphere => (ATMOSPHERE, true),
            Self::Grading => (GRADING, true),
            Self::Cubemap => (CUBEMAP, true),
            Self::Shadows => (SHADOWS, false),
        }
    }

    /// Loads `text`: nothing, or the errors of its rejection.
    fn load(self, text: &str) -> Result<(), Vec<(&'static str, String, String)>> {
        fn run<T: Content>(text: &str) -> Result<(), Vec<(&'static str, String, String)>> {
            parse::<T>(text)
                .map(|_| ())
                .map_err(|rejected| errors(&rejected.diagnostics))
        }
        match self {
            Self::Lighting => run::<LightingSettings>(text),
            Self::Water => run::<WaterSettings>(text),
            Self::Atmosphere => run::<AtmosphereSettings>(text),
            Self::Grading => run::<ColorGradingSettings>(text),
            Self::Cubemap => run::<CubemapSettings>(text),
            Self::Shadows => run::<ShadowSettings>(text),
        }
    }
}

/// How a member holds its ranged number(s).
#[derive(Clone, Copy, Debug)]
enum Form {
    /// A number.
    Number,
    /// A number, or keyframes of numbers.
    Keyframed,
    /// Three numbers, as an array or as an object of `x`, `y`, `z`.
    Vec3,
    /// An integer.
    Integer,
}

/// One ranged member.
#[derive(Clone, Copy, Debug)]
struct Row {
    /// The file under `families/deferred_settings/src/` and the constant in it that holds the bounds.
    source: (&'static str, &'static str),
    payload: Payload,
    /// Every struct version that declares the member at `path`.
    versions: &'static [&'static str],
    /// The member's path below the payload.
    path: &'static str,
    form: Form,
    min: f32,
    max: f32,
    /// Members the innermost object needs beside the ranged one.
    beside: &'static str,
}

const fn row(
    source: (&'static str, &'static str),
    payload: Payload,
    versions: &'static [&'static str],
    path: &'static str,
    form: Form,
    min: f32,
    max: f32,
) -> Row {
    Row {
        source,
        payload,
        versions,
        path,
        form,
        min,
        max,
        beside: "",
    }
}

const LIGHTING_ALL: &[&str] = &["1.21.40", "1.21.60", "1.21.70", "1.21.80", "1.26.0"];
const LIGHTING_FLAT: &[&str] = &["1.21.40", "1.21.60", "1.21.70"];
const LIGHTING_ORBITAL: &[&str] = &["1.21.80", "1.26.0"];
const LIGHTING_PLAIN: &[&str] = &["1.21.40", "1.21.60", "1.21.70", "1.21.80"];
const WATER_ALL: &[&str] = &["1.21.40", "1.21.80", "1.21.120", "1.26.0", "1.26.20"];
const WATER_FROM_80: &[&str] = &["1.21.80", "1.21.120", "1.26.0", "1.26.20"];
const ATMOSPHERE_ALL: &[&str] = &["1.21.40", "1.26.20"];
const GRADING_ALL: &[&str] = &["1.21.40", "1.21.90"];
const CUBEMAP_ALL: &[&str] = &["1.21.130"];

/// The ranged members, with the bounds of `families/render_configs` schema.md.
#[allow(clippy::too_many_lines, reason = "one row per ranged member")]
fn rows() -> Vec<Row> {
    use Form::{Integer, Keyframed, Number, Vec3};
    use Payload::{Atmosphere, Cubemap, Grading, Lighting, Shadows, Water};
    let l = "lighting.rs";
    let w = "water.rs";
    let a = "atmosphere.rs";
    let g = "color_grading.rs";
    let cubemap = "cubemap.rs";
    let mut rows = vec![
        // lighting
        row(
            (l, "ILLUMINANCE"),
            Lighting,
            LIGHTING_FLAT,
            "directional_lights/sun/illuminance",
            Keyframed,
            0.0,
            110_000.0,
        ),
        row(
            (l, "ILLUMINANCE"),
            Lighting,
            LIGHTING_FLAT,
            "directional_lights/moon/illuminance",
            Keyframed,
            0.0,
            110_000.0,
        ),
        row(
            (l, "ORBITAL_OFFSET"),
            Lighting,
            LIGHTING_FLAT,
            "directional_lights/orbital_offset_degrees",
            Keyframed,
            0.0,
            359.99,
        ),
        row(
            (l, "ILLUMINANCE"),
            Lighting,
            LIGHTING_ORBITAL,
            "directional_lights/orbital/sun/illuminance",
            Keyframed,
            0.0,
            110_000.0,
        ),
        row(
            (l, "ILLUMINANCE"),
            Lighting,
            LIGHTING_ORBITAL,
            "directional_lights/orbital/moon/illuminance",
            Keyframed,
            0.0,
            110_000.0,
        ),
        row(
            (l, "ORBITAL_OFFSET"),
            Lighting,
            LIGHTING_ORBITAL,
            "directional_lights/orbital/orbital_offset_degrees",
            Keyframed,
            0.0,
            359.99,
        ),
        row(
            (l, "DESATURATION"),
            Lighting,
            LIGHTING_ALL,
            "emissive/desaturation",
            Number,
            0.0,
            1.0,
        ),
        Row {
            beside: r##""color": "#FFFFFF""##,
            ..row(
                (l, "AMBIENT"),
                Lighting,
                LIGHTING_PLAIN,
                "ambient/illuminance",
                Number,
                0.0,
                5.0,
            )
        },
        Row {
            beside: r##""color": "#FFFFFF""##,
            ..row(
                (l, "AMBIENT"),
                Lighting,
                &["1.26.0"],
                "ambient/illuminance",
                Keyframed,
                0.0,
                5.0,
            )
        },
        row(
            (l, "SKY"),
            Lighting,
            &["1.21.70", "1.21.80"],
            "sky/intensity",
            Number,
            0.1,
            1.0,
        ),
        row(
            (l, "SKY"),
            Lighting,
            &["1.26.0"],
            "sky/intensity",
            Keyframed,
            0.1,
            1.0,
        ),
        // water
        row(
            (w, "CHLOROPHYLL"),
            Water,
            WATER_ALL,
            "particle_concentrations/chlorophyll",
            Number,
            0.0,
            10.0,
        ),
        row(
            (w, "SUSPENDED_SEDIMENT"),
            Water,
            WATER_ALL,
            "particle_concentrations/suspended_sediment",
            Number,
            0.0,
            300.0,
        ),
        row(
            (w, "CDOM"),
            Water,
            WATER_ALL,
            "particle_concentrations/cdom",
            Number,
            0.0,
            15.0,
        ),
        row(
            (w, "FREQUENCY"),
            Water,
            WATER_ALL,
            "waves/frequency",
            Number,
            0.01,
            3.0,
        ),
        row(
            (w, "OCTAVES"),
            Water,
            WATER_ALL,
            "waves/octaves",
            Integer,
            1.0,
            30.0,
        ),
        row(
            (w, "DEPTH"),
            Water,
            WATER_ALL,
            "waves/depth",
            Number,
            0.0,
            3.0,
        ),
        row(
            (w, "SAMPLE_WIDTH"),
            Water,
            &["1.21.40", "1.21.80"],
            "waves/sampleWidth",
            Number,
            0.01,
            1.0,
        ),
        row(
            (w, "SPEED"),
            Water,
            WATER_ALL,
            "waves/speed",
            Number,
            0.01,
            10.0,
        ),
        row(
            (w, "PULL"),
            Water,
            WATER_ALL,
            "waves/pull",
            Number,
            -1.0,
            1.0,
        ),
        row((w, "MIX"), Water, WATER_ALL, "waves/mix", Number, 0.0, 1.0),
        row(
            (w, "SCALING"),
            Water,
            WATER_ALL,
            "waves/frequency_scaling",
            Number,
            0.0,
            2.0,
        ),
        row(
            (w, "SCALING"),
            Water,
            WATER_ALL,
            "waves/speed_scaling",
            Number,
            0.0,
            2.0,
        ),
        row(
            (w, "DIRECTION_INCREMENT"),
            Water,
            WATER_FROM_80,
            "waves/direction_increment",
            Number,
            0.0,
            360.0,
        ),
        row(
            (w, "SHAPE"),
            Water,
            WATER_ALL,
            "waves/shape",
            Number,
            1.0,
            10.0,
        ),
        row(
            (w, "FRAME_LENGTH"),
            Water,
            WATER_FROM_80,
            "caustics/frame_length",
            Number,
            0.01,
            5.0,
        ),
        row(
            (w, "SCALE"),
            Water,
            WATER_FROM_80,
            "caustics/scale",
            Number,
            0.1,
            5.0,
        ),
        row(
            (w, "POWER"),
            Water,
            WATER_FROM_80,
            "caustics/power",
            Number,
            1.0,
            6.0,
        ),
        row(
            (w, "CONTRIBUTION"),
            Water,
            &["1.26.0", "1.26.20"],
            "biome_water_color_contribution",
            Number,
            0.0,
            1.0,
        ),
        // atmosphere
        row(
            (a, "STOP"),
            Atmosphere,
            ATMOSPHERE_ALL,
            "horizon_blend_stops/min",
            Keyframed,
            0.0,
            1.0,
        ),
        row(
            (a, "STOP"),
            Atmosphere,
            ATMOSPHERE_ALL,
            "horizon_blend_stops/start",
            Keyframed,
            0.0,
            1.0,
        ),
        row(
            (a, "MIE_START"),
            Atmosphere,
            ATMOSPHERE_ALL,
            "horizon_blend_stops/mie_start",
            Keyframed,
            0.0,
            1.2,
        ),
        row(
            (a, "STOP"),
            Atmosphere,
            ATMOSPHERE_ALL,
            "horizon_blend_stops/max",
            Keyframed,
            0.0,
            1.0,
        ),
        row(
            (a, "RAYLEIGH"),
            Atmosphere,
            ATMOSPHERE_ALL,
            "rayleigh_strength",
            Keyframed,
            0.0,
            50.0,
        ),
        row(
            (a, "SUN_MIE"),
            Atmosphere,
            ATMOSPHERE_ALL,
            "sun_mie_strength",
            Keyframed,
            0.0,
            60.0,
        ),
        row(
            (a, "MOON_MIE"),
            Atmosphere,
            ATMOSPHERE_ALL,
            "moon_mie_strength",
            Keyframed,
            0.0,
            20.0,
        ),
        row(
            (a, "SUN_GLARE"),
            Atmosphere,
            ATMOSPHERE_ALL,
            "sun_glare_shape",
            Keyframed,
            0.0,
            50.0,
        ),
        // colour grading
        row(
            (g, "HIGHLIGHTS_MIN"),
            Grading,
            GRADING_ALL,
            "color_grading/highlights/highlightsMin",
            Number,
            1.0,
            20.0,
        ),
        row(
            (g, "SHADOWS_MAX"),
            Grading,
            GRADING_ALL,
            "color_grading/shadows/shadowsMax",
            Number,
            0.0,
            1.0,
        ),
        row(
            (g, "TEMPERATURE"),
            Grading,
            &["1.21.90"],
            "color_grading/temperature/temperature",
            Number,
            1000.0,
            15000.0,
        ),
        // cubemap
        row(
            (cubemap, "ILLUMINANCE"),
            Cubemap,
            CUBEMAP_ALL,
            "lighting/ambient_light_illuminance",
            Keyframed,
            0.0,
            100_000.0,
        ),
        row(
            (cubemap, "CONTRIBUTION"),
            Cubemap,
            CUBEMAP_ALL,
            "lighting/sky_light_contribution",
            Number,
            0.0,
            1.0,
        ),
        row(
            (cubemap, "CONTRIBUTION"),
            Cubemap,
            CUBEMAP_ALL,
            "lighting/directional_light_contribution",
            Number,
            0.0,
            1.0,
        ),
        // shadows
        row(
            ("shadows.rs", "TEXEL_SIZE"),
            Shadows,
            &["1.21.80"],
            "texel_size",
            Integer,
            1.0,
            1024.0,
        ),
    ];
    // The five controls of each of the three tonal ranges.
    for (path, source, min, max) in [
        ("color_grading/midtones/contrast", "CONTRAST", 0.0, 4.0),
        ("color_grading/midtones/gain", "GAIN", 0.0, 10.0),
        ("color_grading/midtones/gamma", "GAMMA", 0.0, 4.0),
        ("color_grading/midtones/offset", "OFFSET", -1.0, 1.0),
        ("color_grading/midtones/saturation", "SATURATION", 0.0, 10.0),
        ("color_grading/highlights/contrast", "CONTRAST", 0.0, 4.0),
        ("color_grading/highlights/gain", "GAIN", 0.0, 10.0),
        ("color_grading/highlights/gamma", "GAMMA", 0.0, 4.0),
        ("color_grading/highlights/offset", "OFFSET", -1.0, 1.0),
        (
            "color_grading/highlights/saturation",
            "SATURATION",
            0.0,
            10.0,
        ),
        ("color_grading/shadows/contrast", "CONTRAST", 0.0, 4.0),
        ("color_grading/shadows/gain", "GAIN", 0.0, 10.0),
        ("color_grading/shadows/gamma", "GAMMA", 0.0, 4.0),
        ("color_grading/shadows/offset", "OFFSET", -1.0, 1.0),
        ("color_grading/shadows/saturation", "SATURATION", 0.0, 10.0),
    ] {
        rows.push(row((g, source), Grading, GRADING_ALL, path, Vec3, min, max));
    }
    rows
}

/// The next `f32` above `value`.
pub fn next_up(value: f32) -> f32 {
    let bits = value.to_bits();
    if value == 0.0 {
        f32::from_bits(1)
    } else if value > 0.0 {
        f32::from_bits(bits + 1)
    } else {
        f32::from_bits(bits - 1)
    }
}

/// The next `f32` below `value`.
pub fn next_down(value: f32) -> f32 {
    -next_up(-value)
}

/// `value` written with the digits of the double it widens to, so that it reads back as
/// exactly that float.
pub fn exact(value: f32) -> String {
    let text = f64::from(value).to_string();
    #[allow(clippy::cast_possible_truncation, reason = "the check itself")]
    let back = text.parse::<f64>().unwrap() as f32;
    assert_eq!(back.to_bits(), value.to_bits(), "{text}");
    text
}

/// One document of a row: its text and the path the range finding has in it.
struct Case {
    text: String,
    at: String,
}

/// The document of `row` at `version` whose member holds `value`; `suffix` is appended to the
/// finding's path.
fn case(row: &Row, version: &str, value: &str, suffix: &str) -> Case {
    let (key, described) = row.payload.key();
    let mut names: Vec<&str> = row.path.split('/').collect();
    let member = names.pop().unwrap();
    let comma = if row.beside.is_empty() { "" } else { ", " };
    let mut members = format!(r#"{}{comma}"{member}": {value}"#, row.beside);
    for name in names.iter().rev() {
        members = format!(r#""{name}": {{{members}}}"#);
    }
    if described {
        members = named("t:ranged", &members);
    }
    Case {
        text: doc(version, key, &members),
        at: format!("{key}/{}{suffix}", row.path),
    }
}

/// Every document of `row` at `version` in which the one ranged number is `value` and any
/// other number of the member is `inside` the range.
fn cases(row: &Row, version: &str, value: &str, inside: &str) -> Vec<Case> {
    match row.form {
        Form::Number | Form::Integer => vec![case(row, version, value, "")],
        Form::Keyframed => vec![
            case(row, version, value, ""),
            case(row, version, &format!(r#"{{"0.5": {value}}}"#), "/0.5"),
            case(
                row,
                version,
                &format!(r#"{{"0": {inside}, "1": {value}}}"#),
                "/1",
            ),
        ],
        Form::Vec3 => {
            let mut out = Vec::new();
            for index in 0..3 {
                let mut array = [inside; 3];
                array[index] = value;
                out.push(case(
                    row,
                    version,
                    &format!("[{}]", array.join(", ")),
                    &format!("/{index}"),
                ));
                let name = ["x", "y", "z"][index];
                let object: Vec<String> = ["x", "y", "z"]
                    .iter()
                    .zip(array)
                    .map(|(name, value)| format!(r#""{name}": {value}"#))
                    .collect();
                out.push(case(
                    row,
                    version,
                    &format!("{{{}}}", object.join(", ")),
                    &format!("/{name}"),
                ));
            }
            out
        }
    }
}

/// The game's range text for a value outside `[min, max]` (`rangeMessage`: the bounds as
/// cereal's `toString` writes them, no value).
fn range_text(min: f32, max: f32) -> String {
    let number = bedrock_addon::diagnostic::cereal::number;
    format!(
        "the input is outside of the accepted range [{}, {}]",
        number(f64::from(min)),
        number(f64::from(max))
    )
}

#[test]
fn every_ranged_member_takes_its_bounds_and_refuses_the_next_float_beyond_them() {
    let mut documents = 0;
    for row in rows() {
        assert!(row.min < row.max, "{row:?}");
        // (the bound as written, the value beyond it as written)
        let ends: [(String, String); 2] = if matches!(row.form, Form::Integer) {
            [
                (row.min.to_string(), (row.min - 1.0).to_string()),
                (row.max.to_string(), (row.max + 1.0).to_string()),
            ]
        } else {
            let (below, above) = (next_down(row.min), next_up(row.max));
            assert!(below < row.min && above > row.max, "{row:?}");
            [
                (exact(row.min), exact(below)),
                (exact(row.max), exact(above)),
            ]
        };
        let inside = &ends[1].0;
        for version in row.versions {
            for (bound, beyond) in &ends {
                for accepted in cases(&row, version, bound, inside) {
                    documents += 1;
                    assert_eq!(
                        row.payload.load(&accepted.text),
                        Ok(()),
                        "the bound is inside the range: {}",
                        accepted.text
                    );
                }
                for refused in cases(&row, version, beyond, inside) {
                    documents += 1;
                    let text = range_text(row.min, row.max);
                    assert_eq!(
                        row.payload.load(&refused.text),
                        Err(vec![finding("InvalidValue", &refused.at, &text)]),
                        "the next float beyond the bound is outside the range: {}",
                        refused.text
                    );
                }
            }
        }
    }
    // The count moves only with the table.
    assert_eq!(documents, 1_524);
}

/// The bounds constants of `file` under `families/deferred_settings/src/`: `const NAME: Range = …` and the
/// integer pairs `const NAME: (u8, u8) = …` / `(u16, u16)`; and how often the file builds a
/// range at all.
fn declared(file: &str) -> (Vec<String>, usize) {
    let path = format!(
        "{}/../families/deferred_settings/src/{file}",
        env!("CARGO_MANIFEST_DIR")
    );
    let source = std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{path}: {error}"));
    let mut names = Vec::new();
    for line in source.lines() {
        let Some(rest) = line.trim_start().strip_prefix("const ") else {
            continue;
        };
        let Some((name, kind)) = rest.split_once(": ") else {
            continue;
        };
        if ["Range = ", "(u8, u8) = ", "(u16, u16) = "]
            .iter()
            .any(|prefix| kind.starts_with(prefix))
        {
            names.push(name.to_owned());
        }
    }
    names.sort();
    (names, source.matches("Range::new(").count())
}

#[test]
fn the_table_names_every_range_the_code_declares() {
    let rows = rows();
    for (file, integer_pairs) in [
        ("lighting.rs", 0),
        ("water.rs", 1),
        ("atmosphere.rs", 0),
        ("color_grading.rs", 0),
        ("cubemap.rs", 0),
        ("shadows.rs", 1),
        // No number of these two payloads has a range.
        ("pbr.rs", 0),
        ("local_light.rs", 0),
    ] {
        let mut tabled: Vec<String> = rows
            .iter()
            .filter(|row| row.source.0 == file)
            .map(|row| row.source.1.to_owned())
            .collect();
        tabled.sort();
        tabled.dedup();
        let (declared, built) = declared(file);
        assert_eq!(tabled, declared, "{file}");
        // Every range of the file is one of its constants: none is built in place.
        assert_eq!(built + integer_pairs, declared.len(), "{file}");
    }
    // One row per member and struct shape: no member twice at one version.
    let mut seen = std::collections::HashSet::new();
    for row in &rows {
        for version in row.versions {
            assert!(
                seen.insert((row.payload.key().0, *version, row.path)),
                "{row:?}"
            );
        }
    }
}
