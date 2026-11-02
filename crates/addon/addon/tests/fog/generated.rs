//! The generated round trip (FAMILY-NOTES, wave 4): a few thousand fog documents built from
//! the forms of every member at every version of the chain, with a fixed seed. Every one that
//! loads either writes text that loads again to an equal value (and writes the same text
//! again), or is refused with `NotRepresentable` and a path.

use bedrock_addon::document::Content;
use bedrock_addon::json::EncodeError;

use super::common::lines;
use super::support::{base_game, engine, pack, parse};

/// xorshift64*: a fixed sequence for a fixed seed.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, n: usize) -> usize {
        usize::try_from(self.next() % n as u64).unwrap()
    }

    fn chance(&mut self, percent: usize) -> bool {
        self.below(100) < percent
    }

    fn pick<'a>(&mut self, pool: &[&'a str]) -> &'a str {
        pool[self.below(pool.len())]
    }
}

const VERSIONS: [&str; 12] = [
    "1.16.100",
    "1.20.80",
    "1.21",
    "1.21.0",
    "1.21.90",
    "1.21.90",
    "1.26.19",
    "1.26.20-beta.1",
    "1.26.20",
    "1.26.20",
    "1.26.20",
    "1.26.21",
];
const IDENTIFIERS: [&str; 6] = [
    "demo:fog",
    "Demo:Fog_1.x",
    "demo:fog",
    "myfog",
    "minecraft:fog_x",
    "a:b",
];
const STARTS: [&str; 8] = ["0", "0.5", "0.92", "1", "-0.5", "1.5", "-2", "10"];
const ENDS: [&str; 8] = ["0", "1", "1.0", "0.7", "5", "60.5", "1e3", "-1"];
const TYPES: [&str; 6] = [
    "\"fixed\"",
    "\"render\"",
    "\"fixed\"",
    "\"render\"",
    "\"Render\"",
    "\"dynamic\"",
];
const FOG_COLORS: [&str; 9] = [
    "\"#ABD2FF\"",
    "\"#44aff5\"",
    "[255, 128, 0]",
    "[0, 0, 0]",
    "[300, -5, 128]",
    "[0.5, 1.0, 0]",
    "[1, 2]",
    "\"#FFF\"",
    "[true, null, 7, 9]",
];
const PERCENTS: [&str; 6] = ["0", "0.25", "0.6", "1", "1.0", "1.2"];
const SECONDS: [&str; 6] = ["0", "2", "5", "30", "4.5", "-1"];
const DENSITIES: [&str; 7] = [
    "0",
    "0.05",
    "1",
    "0.5",
    "1.5",
    "{\"0\": 0.1, \"0.5\": 0.2}",
    "{\"0.5\": 0.1, \"0\": 0.2, \"0.0\": 0.3}",
];
const HEIGHTS: [&str; 8] = [
    "320",
    "320.0",
    "64",
    "-64",
    "10.5",
    "-70",
    "{\"0\": 100, \"0.5\": 50}",
    "{\"0\": 20}",
];
const COEFFICIENTS: [&str; 7] = [
    "\"#000000\"",
    "[0.06, 0.06, 0.06]",
    "[2, -1, 0.5]",
    "[0, 0, 0]",
    "{\"0\": [0, 0, 0], \"0.5\": \"#ffFFff\"}",
    "[1, 0]",
    "\"red\"",
];
const GS: [&str; 6] = ["0.4", "-1", "1", "0", "-1.5", "{\"0.25\": 0.5}"];

/// The four members of a distance setting; `all` writes every one (the older schema needs
/// them), otherwise each is written half the time.
fn base(rng: &mut Rng, all: bool) -> Vec<String> {
    let mut members = Vec::new();
    for (key, pool) in [
        ("fog_start", &STARTS[..]),
        ("fog_end", &ENDS[..]),
        ("render_distance_type", &TYPES[..]),
        ("fog_color", &FOG_COLORS[..]),
    ] {
        if all || rng.chance(50) {
            // Mostly the plain forms at the front of each pool.
            let pool = if rng.chance(80) { &pool[..4] } else { pool };
            members.push(format!("\"{key}\": {}", rng.pick(pool)));
        }
    }
    members
}

fn transition(rng: &mut Rng, all: bool) -> String {
    let mut members = Vec::new();
    if all || rng.chance(60) {
        members.push(format!("\"init_fog\": {{{}}}", base(rng, all).join(", ")));
    }
    for (key, pool) in [
        ("min_percent", &PERCENTS[..]),
        ("mid_seconds", &SECONDS[..]),
        ("mid_percent", &PERCENTS[..]),
        ("max_seconds", &SECONDS[..]),
    ] {
        if all || rng.chance(60) {
            let pool = if rng.chance(85) { &pool[..4] } else { pool };
            members.push(format!("\"{key}\": {}", rng.pick(pool)));
        }
    }
    format!("\"transition_fog\": {{{}}}", members.join(", "))
}

fn distance(rng: &mut Rng, all: bool) -> String {
    let mut entries = Vec::new();
    for name in [
        "air",
        "weather",
        "water",
        "lava",
        "lava_resistance",
        "powder_snow",
    ] {
        if !rng.chance(35) {
            continue;
        }
        let mut members = base(rng, all);
        // On water mostly; on another entry it is dropped (older) or unknown (1.26.20).
        if rng.chance(if name == "water" { 50 } else { 8 }) {
            members.push(transition(rng, all));
        }
        if rng.chance(4) {
            members.push("\"fog_density\": 1".to_owned());
        }
        entries.push(format!("\"{name}\": {{{}}}", members.join(", ")));
    }
    format!("\"distance\": {{{}}}", entries.join(", "))
}

fn volumetric(rng: &mut Rng) -> String {
    let mut parts = Vec::new();
    if rng.chance(60) {
        let mut entries = Vec::new();
        for name in ["air", "weather", "water", "lava", "lava_resistance"] {
            if !rng.chance(30) {
                continue;
            }
            let mut members = Vec::new();
            if rng.chance(85) {
                let pool = if rng.chance(85) {
                    &DENSITIES[..4]
                } else {
                    &DENSITIES[..]
                };
                members.push(format!("\"max_density\": {}", rng.pick(pool)));
            }
            if rng.chance(35) {
                members.push(format!("\"uniform\": {}", rng.chance(60)));
            }
            for key in ["zero_density_height", "max_density_height"] {
                if rng.chance(55) {
                    let pool = if rng.chance(85) {
                        &HEIGHTS[..5]
                    } else {
                        &HEIGHTS[..]
                    };
                    members.push(format!("\"{key}\": {}", rng.pick(pool)));
                }
            }
            entries.push(format!("\"{name}\": {{{}}}", members.join(", ")));
        }
        parts.push(format!("\"density\": {{{}}}", entries.join(", ")));
    }
    if rng.chance(40) {
        let mut entries = Vec::new();
        for name in ["air", "water", "cloud"] {
            if !rng.chance(45) {
                continue;
            }
            let mut members = Vec::new();
            for key in ["scattering", "absorption"] {
                if rng.chance(85) {
                    let pool = if rng.chance(85) {
                        &COEFFICIENTS[..4]
                    } else {
                        &COEFFICIENTS[..]
                    };
                    members.push(format!("\"{key}\": {}", rng.pick(pool)));
                }
            }
            entries.push(format!("\"{name}\": {{{}}}", members.join(", ")));
        }
        parts.push(format!(
            "\"media_coefficients\": {{{}}}",
            entries.join(", ")
        ));
    }
    if rng.chance(30) {
        let mut entries = Vec::new();
        for name in ["air", "water"] {
            if rng.chance(60) {
                let pool = if rng.chance(85) { &GS[..4] } else { &GS[..] };
                entries.push(format!(
                    "\"{name}\": {{\"henyey_greenstein_g\": {}}}",
                    rng.pick(pool)
                ));
            }
        }
        parts.push(format!(
            "\"henyey_greenstein_g\": {{{}}}",
            entries.join(", ")
        ));
    }
    if rng.chance(3) {
        parts.push("\"unknown\": [1, {\"a\": null}]".to_owned());
    }
    format!("\"volumetric\": {{{}}}", parts.join(", "))
}

fn document(rng: &mut Rng) -> String {
    let version = rng.pick(&VERSIONS);
    // The older schema requires every member of a setting; write them all most of the time.
    let all = !version.starts_with("1.26.2") || version.contains("beta");
    let all = all && rng.chance(90);
    let mut members = vec![format!(
        "\"description\": {{\"identifier\": \"{}\"}}",
        rng.pick(&IDENTIFIERS)
    )];
    if rng.chance(70) {
        members.push(distance(rng, all));
    }
    if rng.chance(55) {
        members.push(volumetric(rng));
    }
    if rng.chance(3) {
        members.push("\"extra\": {\"kept\": true}".to_owned());
    }
    format!(
        "{{\"format_version\": \"{version}\", \"minecraft:fog_settings\": {{{}}}}}",
        members.join(", ")
    )
}

#[test]
fn every_generated_document_that_loads_is_written_back_or_refused_with_a_path() {
    let mut rng = Rng(0x0F06_5EED);
    let contexts = [pack(), base_game(), engine(1, 26, 20)];
    let (mut loaded, mut upgraded, mut refused) = (0, 0, 0);
    for n in 0..6000 {
        let text = document(&mut rng);
        let cx = &contexts[n % contexts.len()];
        let Ok(document) = parse(&text, cx) else {
            continue;
        };
        loaded += 1;
        upgraded += usize::from(document.version().upgraded);
        match document.to_json() {
            Ok(written) => {
                let again = parse(&written, cx).unwrap_or_else(|rejected| {
                    panic!(
                        "document {n} loads and is written as text that does not load:\n{text}\n{written}\n{:?}",
                        lines(&rejected.diagnostics)
                    )
                });
                assert_eq!(
                    again.value(),
                    document.value(),
                    "document {n}:\n{text}\n{written}"
                );
                assert_eq!(again.to_json().unwrap(), written, "document {n}:\n{text}");
                // The rules V15–V20 are every era's: nothing that loads breaks
                // one, so everything is written at 1.26.20.
                assert!(
                    written.contains(r#""format_version": "1.26.20""#),
                    "document {n}:\n{text}\n{written}"
                );
            }
            Err(EncodeError::NotRepresentable { at, .. }) => {
                refused += 1;
                assert!(
                    at.to_string().starts_with("minecraft:fog_settings/"),
                    "document {n} refused at `{at}`:\n{text}"
                );
            }
            Err(other) => panic!("document {n}: {other:?}\n{text}"),
        }
    }
    eprintln!(
        "generated fog documents: {loaded} of 6000 load ({upgraded} upgraded), {refused} refused"
    );
    // The generator is not vacuous: both eras load, and every document that loads has a
    // text.
    assert!(loaded > 1000, "{loaded}");
    assert!(upgraded > 300 && upgraded < loaded, "{upgraded}");
    assert_eq!(refused, 0);
}
