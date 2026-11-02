//! The generated round trip (FAMILY-NOTES, wave 4): a few thousand render controller files
//! built from the forms of every member, with a fixed seed. Every one that loads either writes
//! text that loads again to an equal value (and writes the same text again), or is refused
//! with `NotRepresentable` and a path.

use std::fmt::Write as _;

use bedrock_addon::document::Content;
use bedrock_addon::json::EncodeError;

use super::common::{lines, rp, unchecked};
use super::rows::parse;

/// xorshift64*: a fixed sequence for a fixed seed.
pub struct Rng(pub u64);

impl Rng {
    pub fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// A number below `n`.
    pub fn below(&mut self, n: usize) -> usize {
        usize::try_from(self.next() % n as u64).unwrap()
    }

    /// True `percent` times in a hundred.
    pub fn chance(&mut self, percent: usize) -> bool {
        self.below(100) < percent
    }

    pub fn pick<'a>(&mut self, pool: &[&'a str]) -> &'a str {
        pool[self.below(pool.len())]
    }
}

const VERSIONS: [&str; 14] = [
    "\"1.8.0\"",
    "\"1.8.0\"",
    "\"1.8.0\"",
    "\"1.8.0\"",
    "\"1.10.0\"",
    "\"1.10\"",
    "\"65535.0.0\"",
    "\"1.26.50\"",
    "\"1.10\"",
    "\"1.10.0\"",
    "\"1.10.0-beta.1\"",
    "\"99.0.0\"",
    "\"1.7.0\"",
    "1.8",
];
const KEYS: [&str; 5] = [
    "controller.render.a",
    "controller.render.B",
    "Controller.Render.c_1",
    "controller.render.d:e-f.9",
    "controller.render.zz",
];
/// Resource expressions: valid, array lookups, a string literal, an apostrophe, a broken one.
const EXPRESSIONS: [&str; 9] = [
    "\"Geometry.default\"",
    "\"texture.default\"",
    "\"Array.skins[q.variant]\"",
    "\"array.G[0]\"",
    "\"q.get_name == 'Toast' ? Texture.A : texture.b\"",
    "\"textures/bob's_Hat\"",
    "\"Material.default +\"",
    "\"\"",
    "\"material.default\"",
];
/// Values of an `ExpressionNode` member.
const VALUES: [&str; 18] = [
    "0.25",
    "3",
    "1e2",
    "-0.125",
    "true",
    "\"query.is_baby\"",
    "\"math.sin(q.life_time * 90) > 0 ? 1 : 0\"",
    "\"v.x = 1; return v.x;\"",
    "0.5",
    "1",
    "-2",
    "true",
    "false",
    "\"this\"",
    "\"q.is_baby ? 0.5 : 1\"",
    "\"Q.Variant == 1\"",
    "\"1 +\"",
    "\"1e\"",
];
const BONES: [&str; 6] = ["*", "head", "Arm*", "leg.l:0", "body_1", "left-arm"];
const ARRAYS: [&str; 4] = ["Array.skins", "array.G", "array.m_1", "Array.x:y"];

fn list(rng: &mut Rng, min: usize, max: usize, pool: &[&str]) -> String {
    let n = min + rng.below(max - min + 1);
    let items: Vec<&str> = (0..n).map(|_| rng.pick(pool)).collect();
    format!("[{}]", items.join(", "))
}

fn bones(rng: &mut Rng, pool: &[&str]) -> String {
    let elements = rng.below(4);
    let mut out = Vec::new();
    for _ in 0..elements {
        let members = 1 + rng.below(2);
        let pairs: Vec<String> = (0..members)
            .map(|_| {
                let bone = if rng.chance(3) {
                    BONES[5]
                } else {
                    rng.pick(&BONES[..5])
                };
                format!("\"{bone}\": {}", rng.pick(pool))
            })
            .collect();
        out.push(format!("{{{}}}", pairs.join(", ")));
    }
    format!("[{}]", out.join(", "))
}

fn color(rng: &mut Rng) -> String {
    let mut channels = Vec::new();
    for c in ["r", "g", "b", "a"] {
        if rng.chance(50) {
            channels.push(format!("\"{c}\": {}", rng.pick(&VALUES)));
        }
    }
    format!("{{{}}}", channels.join(", "))
}

fn arrays(rng: &mut Rng) -> String {
    let mut kinds = Vec::new();
    for kind in ["geometries", "materials", "textures"] {
        if rng.chance(40) {
            let names = rng.below(3);
            let lists: Vec<String> = (0..names)
                .map(|_| {
                    let name = rng.pick(&ARRAYS);
                    let min = usize::from(!rng.chance(5));
                    format!("\"{name}\": {}", list(rng, min, 3, &EXPRESSIONS))
                })
                .collect();
            kinds.push(format!("\"{kind}\": {{{}}}", lists.join(", ")));
        }
    }
    format!("{{{}}}", kinds.join(", "))
}

fn controller(rng: &mut Rng) -> String {
    let mut members = Vec::new();
    if rng.chance(95) {
        members.push(format!("\"geometry\": {}", rng.pick(&EXPRESSIONS)));
    }
    if rng.chance(85) {
        let min = usize::from(!rng.chance(4));
        members.push(format!("\"textures\": {}", list(rng, min, 3, &EXPRESSIONS)));
    }
    if rng.chance(50) {
        let pool: Vec<&str> = EXPRESSIONS.iter().chain(&VALUES[..5]).copied().collect();
        members.push(format!("\"materials\": {}", bones(rng, &pool)));
    }
    if rng.chance(40) {
        members.push(format!("\"part_visibility\": {}", bones(rng, &VALUES)));
    }
    if rng.chance(40) {
        members.push(format!("\"arrays\": {}", arrays(rng)));
    }
    for key in ["color", "overlay_color", "on_fire_color", "is_hurt_color"] {
        if rng.chance(20) {
            members.push(format!("\"{key}\": {}", color(rng)));
        }
    }
    if rng.chance(25) {
        let offset_max = if rng.chance(8) { 3 } else { 2 };
        members.push(format!(
            "\"uv_anim\": {{\"offset\": {}, \"scale\": {}}}",
            list(rng, 2, offset_max, &VALUES),
            list(rng, 2, 2, &VALUES)
        ));
    }
    if rng.chance(25) {
        members.push(format!("\"light_color_multiplier\": {}", rng.pick(&VALUES)));
    }
    if rng.chance(20) {
        members.push(format!("\"ignore_lighting\": {}", rng.chance(50)));
    }
    if rng.chance(15) {
        members.push(format!("\"filter_lighting\": {}", !rng.chance(10)));
    }
    if rng.chance(15) {
        members.push(format!(
            "\"Rebuild_Animation_Matrices\": {}",
            rng.chance(50)
        ));
    }
    if rng.chance(2) {
        members.push("\"is_invisible_color\": {}".to_owned());
    }
    format!("{{{}}}", members.join(", "))
}

fn document(rng: &mut Rng) -> String {
    let mut text = String::from("{");
    if rng.chance(15) {
        // A comment's apostrophe: nothing after it is folded.
        text.push_str("\n// it's\n");
    }
    write!(text, "\"format_version\": {}, ", rng.pick(&VERSIONS)).unwrap();
    text.push_str("\"render_controllers\": {");
    let count = 1 + rng.below(3);
    let mut keys: Vec<&str> = Vec::new();
    while keys.len() < count {
        let key = rng.pick(&KEYS);
        if !keys.contains(&key) {
            keys.push(key);
        }
    }
    let controllers: Vec<String> = keys
        .iter()
        .map(|key| format!("\"{key}\": {}", controller(rng)))
        .collect();
    text.push_str(&controllers.join(", "));
    text.push_str("}}");
    text
}

/// The controller keys of a written file, in the text's order: the members two levels deep,
/// which the writer puts on a line each behind four spaces. A capital the writer protected
/// from the fold (`\u0042`) is decoded.
fn written_keys(written: &str) -> Vec<String> {
    written
        .lines()
        .filter_map(|line| line.strip_prefix("    \""))
        .filter_map(|line| line.split_once('"'))
        .map(|(key, _)| {
            let mut pieces = key.split("\\u");
            let mut decoded = pieces.next().unwrap_or_default().to_owned();
            for piece in pieces {
                let code = u32::from_str_radix(&piece[..4], 16).unwrap();
                decoded.push(char::from_u32(code).unwrap());
                decoded.push_str(&piece[4..]);
            }
            decoded
        })
        .collect()
}

#[test]
fn every_generated_file_that_loads_is_written_back_or_refused_with_a_path() {
    let mut rng = Rng(0x5EED_0FC0_FFEE);
    let contexts = [rp(1, 21, 100), rp(1, 16, 0), unchecked()];
    let (mut loaded, mut with_dropped, mut refused, mut dropped_after_loaded) = (0, 0, 0, 0);
    for n in 0..8000 {
        let text = document(&mut rng);
        let cx = &contexts[n % contexts.len()];
        let Ok(document) = parse(&text, cx) else {
            continue;
        };
        loaded += 1;
        with_dropped += usize::from(!document.dropped.is_empty());
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
                // The written members are the loaded and the dropped controllers together, in
                // the game's order (sorted keys).
                let mut keys: Vec<&str> = document
                    .render_controllers
                    .keys()
                    .chain(document.dropped.keys())
                    .map(String::as_str)
                    .collect();
                keys.sort_unstable();
                assert_eq!(written_keys(&written), keys, "document {n}:\n{written}");
                dropped_after_loaded += usize::from(
                    document.dropped.keys().next_back() > document.render_controllers.keys().next(),
                );
            }
            Err(EncodeError::NotRepresentable { at, .. }) => {
                refused += 1;
                assert!(
                    !at.is_empty(),
                    "document {n} refused without a path:\n{text}"
                );
            }
            Err(other) => panic!("document {n}: {other:?}\n{text}"),
        }
    }
    eprintln!(
        "generated render controller files: {loaded} of 8000 load, {with_dropped} with a dropped controller, {refused} refused"
    );
    // The generator is not vacuous: many files load, and many of those carry a dropped
    // controller.
    assert!(loaded > 800, "{loaded}");
    assert!(with_dropped > 100, "{with_dropped}");
    // The order was seen where it matters: a dropped controller sorted after a loaded one.
    assert!(dropped_after_loaded > 50, "{dropped_after_loaded}");
}
