//! Round trips over generated input: documents written as text over every member's forms, and
//! values built in code. Each document that loads writes text that loads back to an equal
//! value; each built value is either written as text that reads back equal, or refused with
//! `NotRepresentable`.

use bedrock_addon::document::{Content, UnknownMembers};
use bedrock_addon::families::material::{
    KeyRules, Listed, MaterialDef, MaterialFile, MaterialFileForm, MaterialKey, MaterialList,
    SamplerState, StencilFace, Variant, VertexField,
};
use bedrock_addon::json::{EncodeError, Value};
use bedrock_addon::version::SemVersion;

use crate::common::{Rewrites, parse_at, parse_list, rendered, rewrites_at};

/// The kind of file a written text is compared for: what its reader drops differs.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Read {
    /// A `.material` file.
    File,
    /// A list file.
    List,
}

/// Whether `finding` of a read says that what the input has at `place` was dropped: an
/// `InvalidType` finding at that very path (an element of an array or a variant body of a
/// `.material` file; an element or a define of a list), or, in a list, the
/// finding of the `path` of the entry at `place`, missing or no string.
fn says_dropped(finding: &crate::common::Rendered, place: &str, read: Read) -> bool {
    let entry_path = matches!(
        finding.0,
        "core/invalid_type" | "core/missing_required_field"
    ) && finding.2.strip_suffix("/path") == Some(place);
    (finding.0 == "core/invalid_type" && finding.2 == place) || (read == Read::List && entry_path)
}

/// The written text against the input, as JSON values: equal, or the rewrites the first read
/// reported. The differences are returned for the test's totals.
#[track_caller]
fn compared_with_the_input(
    n: usize,
    text: &str,
    written: &str,
    findings: &[crate::common::Rendered],
    read: Read,
) -> Rewrites {
    let (found, dropped_at) = rewrites_at(text, written)
        .unwrap_or_else(|error| panic!("#{n} not written as authored: {error}\n{text}\n{written}"));
    // Nothing is dropped without a finding at its own place.
    for place in &dropped_at {
        assert!(
            findings
                .iter()
                .any(|finding| says_dropped(finding, place, read)),
            "#{n} dropped without a finding at {place}: {findings:#?}\n{text}\n{written}"
        );
    }
    // Nothing is clamped without a finding of the kind.
    let invalid = findings
        .iter()
        .filter(|finding| finding.0 == "core/invalid_value")
        .count();
    assert!(
        found.clamped <= invalid,
        "#{n} {found:?}\n{text}\n{written}"
    );
    found
}

/// A small deterministic generator (xorshift64*).
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    #[expect(
        clippy::cast_possible_truncation,
        reason = "the remainder is below `n`, which is a usize"
    )]
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    fn pick<'a>(&mut self, pool: &[&'a str]) -> &'a str {
        pool[self.below(pool.len())]
    }

    fn chance(&mut self, one_in: usize) -> bool {
        self.below(one_in) == 0
    }
}

/// An integer of 400 digits.
const WIDE_INTEGER: &str = "1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890";

/// Values of every JSON type, the valid forms of each member among them.
const VALUES: &[&str] = &[
    "null",
    "true",
    "false",
    "0",
    "7",
    "8",
    "300",
    "-1",
    "1.5",
    "3.0",
    "4294967295",
    "4294967296",
    "1e39",
    "-1e999",
    // Numbers no float holds: written as the shortest decimal of the float.
    "0.30000000000000004",
    "9007199254740993",
    "18446744073709551615",
    "-0.0",
    // Integers past 64 bits, which core reads as the double each spells: 2^64, and
    // one of 400 digits, past the doubles and clamped like `1e999`.
    "18446744073709551616",
    WIDE_INTEGER,
    "\"\"",
    "\"Less\"",
    "\"Keep\"",
    "\"One\"",
    "\"Point\"",
    "\"Clamp\"",
    "\"Pixel\"",
    "\"UV0\"",
    "\"Both\"",
    "\"Line\"",
    "\"Blending\"",
    "\"PolygonOffset\"",
    "\"Nope\"",
    "\"shaders/entity.vertex\"",
    "[]",
    "{}",
    "[\"Blending\", \"A\", 5]",
    "[[\"R8_UNORM\"], \"R8_UNORM\", [\"nope\", 1]]",
];

/// Every key the readers look up, with prefixes, misspellings and unknown ones.
const KEYS: &[&str] = &[
    "states",
    "+states",
    "-states",
    "defines",
    "+defines",
    "-defines",
    "samplerStates",
    "+samplerStates",
    "-samplerStates",
    "variants",
    "+variants",
    "-variants",
    "vertexShader",
    "fragmentShader",
    "geometryShader",
    "depthFunc",
    "frontFace",
    "backFace",
    "stencilRefOverride",
    "stencilRef",
    "stencilReadMask",
    "stencilWriteMask",
    "blendSrc",
    "blendDst",
    "alphaSrc",
    "alphaDst",
    "vertexFields",
    "depthBias",
    "slopeScaledDepthBias",
    "msaaSupport",
    "primitiveMode",
    "renderTargetFormats",
    "isAnimatedTexture",
    "+vertexShader",
    "-depthBias",
    "varient",
    "+variant",
    "vrGeometryShader",
    "version",
    "extra",
    "key_rules",
];

const SUB_KEYS: &[&str] = &[
    "samplerIndex",
    "textureFilter",
    "textureWrap",
    "shaderStage",
    "comparisonFunc",
    "+textureFilter",
    "stencilFunc",
    "stencilFailOp",
    "stencilDepthFailOp",
    "stencilPassOp",
    "-stencilFunc",
    "stencilPass",
    "field",
    "note",
];

const MATERIAL_KEYS: &[&str] = &[
    "a",
    "b:a",
    "+a",
    "c:b:a",
    "d:",
    ":e",
    "+",
    "materials",
    "version",
    "vertexShader",
    "a:x",
    "f.g",
];

/// An object of sub-members (a face, a sampler entry, a vertex field).
fn sub_object(rng: &mut Rng) -> String {
    let members: Vec<String> = (0..rng.below(4))
        .map(|_| format!("\"{}\": {}", rng.pick(SUB_KEYS), rng.pick(VALUES)))
        .collect();
    format!("{{{}}}", members.join(", "))
}

/// The value of one member: a plausible one for its key, or any value at all.
fn member_value(rng: &mut Rng, key: &str, depth: usize) -> String {
    if rng.chance(3) {
        return rng.pick(VALUES).to_owned();
    }
    let bare = key.trim_start_matches(['+', '-']);
    match bare {
        "samplerStates" | "vertexFields" => {
            let entries: Vec<String> = (0..rng.below(3))
                .map(|_| {
                    if rng.chance(6) {
                        rng.pick(VALUES).to_owned()
                    } else {
                        sub_object(rng)
                    }
                })
                .collect();
            format!("[{}]", entries.join(", "))
        }
        "frontFace" | "backFace" => sub_object(rng),
        "variants" if depth < 3 => {
            let entries: Vec<String> = (0..rng.below(3))
                .map(|_| {
                    let bodies: Vec<String> = (0..rng.below(3))
                        .map(|_| {
                            let name = rng.pick(&["skinning", "a", "a.b", "color"]);
                            let body = if rng.chance(8) {
                                rng.pick(VALUES).to_owned()
                            } else {
                                material(rng, depth + 1)
                            };
                            format!("\"{name}\": {body}")
                        })
                        .collect();
                    format!("{{{}}}", bodies.join(", "))
                })
                .collect();
            format!("[{}]", entries.join(", "))
        }
        _ => rng.pick(VALUES).to_owned(),
    }
}

/// One material body.
fn material(rng: &mut Rng, depth: usize) -> String {
    let members: Vec<String> = (0..rng.below(6))
        .map(|_| {
            let key = rng.pick(KEYS);
            format!("\"{key}\": {}", member_value(rng, key, depth))
        })
        .collect();
    format!("{{{}}}", members.join(", "))
}

/// One document: any of the root forms, with materials and stray root members.
fn document(rng: &mut Rng) -> String {
    let mut members: Vec<String> = (0..rng.below(5))
        .map(|_| {
            let value = if rng.chance(8) {
                rng.pick(VALUES).to_owned()
            } else {
                material(rng, 0)
            };
            format!("\"{}\": {value}", rng.pick(MATERIAL_KEYS))
        })
        .collect();
    match rng.below(6) {
        // A wrapper the client unwraps.
        0 | 1 => {
            let version = rng.pick(&["1.0.0", "1.0.0+build"]);
            members.insert(0, format!("\"version\": \"{version}\""));
            format!("{{\"materials\": {{{}}}}}", members.join(", "))
        }
        // A wrapper it does not.
        2 => {
            let version = rng.pick(&["\"1.0.1\"", "\"\"", "1", "\"1.0.0-beta\""]);
            members.push(format!("\"version\": {version}"));
            let extra = if rng.chance(2) { ", \"other\": {}" } else { "" };
            format!("{{\"materials\": {{{}}}{extra}}}", members.join(", "))
        }
        // The single form.
        3 => {
            let mut body = material(rng, 0);
            body.insert_str(1, "\"vertexShader\": \"shaders/x.vertex\", ");
            body.replace(", }", "}")
        }
        // The flat form.
        _ => format!("{{{}}}", members.join(", ")),
    }
}

#[test]
fn every_generated_document_that_loads_writes_text_that_loads_back_equal() {
    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
    let (mut loaded, mut rejected) = (0, 0);
    let (mut total, mut as_authored) = (Rewrites::default(), 0);
    for n in 0..6000 {
        let text = document(&mut rng);
        let Ok(first) = parse_at("materials/gen.material", &text) else {
            rejected += 1;
            continue;
        };
        loaded += 1;
        // A document that loaded has no value the writer cannot spell: it is never refused.
        let written = first
            .to_json()
            .unwrap_or_else(|error| panic!("#{n} not written: {error}\n{text}"));
        let second = parse_at("materials/gen.material", &written).unwrap_or_else(|rejected| {
            panic!(
                "#{n} the written text is refused: {:#?}\n{text}\n{written}",
                rejected.diagnostics
            )
        });
        assert_eq!(*first, *second, "#{n}\n{text}\n{written}");
        assert_eq!(second.to_json().unwrap(), written, "#{n}\n{text}");
        // The written text is the input as a JSON value, but for what the first read
        // reported as rewritten.
        let findings = rendered(first.diagnostics().iter());
        let found = compared_with_the_input(n, &text, &written, &findings, Read::File);
        as_authored += usize::from(found.is_none());
        total.add(found);
        // Nothing the reader drops is written: the second read drops nothing.
        let dropped = rendered(second.diagnostics().iter())
            .into_iter()
            .filter(|finding| finding.0 == "core/invalid_type")
            .filter(|finding| {
                let last = finding.2.rsplit('/').next().unwrap_or_default();
                last.parse::<usize>().is_ok()
            })
            .count();
        assert_eq!(dropped, 0, "#{n}\n{text}\n{written}");
    }
    // The generator reaches both outcomes in number.
    assert!(loaded > 2000, "{loaded} loaded");
    assert!(rejected > 500, "{rejected} rejected");
    // Most documents are written as authored, and each kind of rewrite occurs.
    eprintln!("generated documents: {loaded} loaded, {as_authored} written as authored, {total:?}");
    assert!(as_authored > 1000, "{as_authored} as authored");
    assert!(total.dropped > 100, "{total:?}");
    assert!(total.clamped > 20, "{total:?}");
    assert!(total.rounded > 0, "{total:?}");
}

/// One of the words a material states hold, or a word it does not.
fn word(rng: &mut Rng) -> String {
    rng.pick(&["Less", "One", "Keep", "Point", "UV0", "Nope", "", "Both"])
        .to_owned()
}

/// One of the floats a material holds, or one the writer cannot spell as a number.
fn float(rng: &mut Rng) -> f32 {
    [
        0.0,
        1.5,
        -3.0,
        0.1,
        f32::MAX,
        f32::MIN,
        f32::NAN,
        f32::INFINITY,
    ][rng.below(8)]
}

/// Puts random members into `def`.
fn sprinkle(rng: &mut Rng, def: &mut MaterialDef) {
    if rng.chance(3) {
        def.vertex_shader = Some(word(rng));
    }
    if rng.chance(4) {
        def.depth_func = Some(word(rng));
    }
    if rng.chance(4) {
        def.depth_bias = Some(float(rng));
    }
    if rng.chance(6) {
        def.slope_scaled_depth_bias = Some(float(rng));
    }
    if rng.chance(4) {
        def.stencil_ref = Some([0, 300, u32::MAX][rng.below(3)]);
    }
    if rng.chance(4) {
        def.front_face = Some(StencilFace {
            stencil_func: rng.chance(2).then(|| word(rng)),
            ..StencilFace::default()
        });
    }
    if rng.chance(4) {
        def.sampler_states.add = Some(vec![SamplerState {
            sampler_index: rng.chance(2).then(|| [0, 7, 8][rng.below(3)]),
            texture_filter: rng.chance(2).then(|| word(rng)),
            ..SamplerState::default()
        }]);
    }
    if rng.chance(4) {
        def.vertex_fields = Some(vec![VertexField {
            field: rng.chance(2).then(|| word(rng)),
            ..VertexField::default()
        }]);
    }
    if rng.chance(5) {
        def.render_target_formats = Some(vec![vec![word(rng)], vec![]]);
    }
    if rng.chance(5) {
        let value = ["true", "null", "{}", "5"][rng.below(4)];
        def.is_animated_texture = Some(facet_json::from_str::<Value>(value).unwrap());
    }
    if rng.chance(8) {
        def.key_rules = Some(KeyRules::Legacy);
    }
    if rng.chance(5) {
        let key = rng.pick(&["vertexShader", "custom", "+states", "states", "varient"]);
        let value = ["\"s\"", "null", "[]", "5"][rng.below(4)];
        def.extra.insert(
            key.to_owned(),
            facet_json::from_str::<Value>(value).unwrap(),
        );
    }
}

/// A random built definition, with the things a reader would never produce among them.
fn built(rng: &mut Rng, depth: usize) -> MaterialDef {
    let strings = |rng: &mut Rng| -> Option<Vec<String>> {
        rng.chance(3).then(|| {
            (0..rng.below(3))
                .map(|_| {
                    rng.pick(&["Blending", "A", "PolygonOffset", "x"])
                        .to_owned()
                })
                .collect()
        })
    };
    let listed = |rng: &mut Rng| Listed {
        replace: strings(rng),
        add: strings(rng),
        sub: strings(rng),
    };
    let mut def = MaterialDef {
        states: listed(rng),
        defines: listed(rng),
        ..MaterialDef::default()
    };
    sprinkle(rng, &mut def);
    if depth < 2 && rng.chance(3) {
        let bodies = (0..rng.below(3))
            .map(|_| {
                let name = rng.pick(&["skinning", "a", "a.b"]).to_owned();
                (name, built(rng, depth + 1))
            })
            .collect();
        let list = Some(vec![Variant(bodies)]);
        match rng.below(3) {
            0 => def.variants.replace = list,
            1 => def.variants.add = list,
            _ => def.variants.sub = list,
        }
    }
    def
}

#[test]
fn every_built_value_is_written_as_text_that_reads_back_equal_or_is_refused() {
    let mut rng = Rng(0x1234_5678_9abc_def1);
    let (mut written_count, mut refused) = (0, 0);
    for n in 0..4000 {
        let materials = (0..rng.below(4))
            .map(|_| {
                let key = match rng.below(8) {
                    0 => MaterialKey {
                        extend: rng.chance(2),
                        name: rng.pick(&["a", "a:b", "", "+a"]).to_owned(),
                        parent: rng
                            .chance(2)
                            .then(|| rng.pick(&["p", "p:q", ""]).to_owned()),
                    },
                    _ => MaterialKey::parse(rng.pick(MATERIAL_KEYS)),
                };
                (key, built(&mut rng, 0))
            })
            .collect();
        let form = match rng.below(5) {
            0 => MaterialFileForm::Single {
                stem: "gen".to_owned(),
                wrapper: None,
            },
            1 | 2 => MaterialFileForm::Wrapped {
                version: SemVersion::parse(rng.pick(&["1.0.0", "1.0.0+b", "1.0.1", "2.0.0"]))
                    .unwrap(),
            },
            _ => MaterialFileForm::Flat,
        };
        let mut value = MaterialFile {
            form,
            materials,
            extra: UnknownMembers::default(),
        };
        if let MaterialFileForm::Single { .. } = value.form
            && rng.chance(2)
        {
            let mut def = built(&mut rng, 0);
            def.vertex_shader = Some("shaders/gen.vertex".to_owned());
            value.materials = [(MaterialKey::new("gen"), def)].into_iter().collect();
        }
        if rng.chance(10) {
            let stray = ["5", "{}", "null", "\"s\""][rng.below(4)];
            let key = rng.pick(&["n", "materials", "vertexShader"]);
            value.extra.insert(
                key.to_owned(),
                facet_json::from_str::<Value>(stray).unwrap(),
            );
        }
        match value.to_json() {
            Ok(text) => {
                written_count += 1;
                let read = parse_at("materials/gen.material", &text).unwrap_or_else(|rejected| {
                    panic!(
                        "#{n} written text is refused: {:#?}\n{text}",
                        rejected.diagnostics
                    )
                });
                // The key rules of a merge are never written: a file read on its own has none.
                for def in value.materials.values_mut() {
                    def.key_rules = None;
                }
                assert_eq!(*read, value, "#{n}\n{text}");
            }
            Err(EncodeError::NotRepresentable { .. }) => refused += 1,
            Err(other) => panic!("#{n}: {other}"),
        }
    }
    assert!(written_count > 300, "{written_count} written");
    assert!(refused > 300, "{refused} refused");
}

#[test]
fn every_generated_list_that_loads_writes_text_that_loads_back_equal() {
    let mut rng = Rng(0x0dd_ba11_5eed_cafe);
    let mut loaded = 0;
    let (mut total, mut as_authored) = (Rewrites::default(), 0);
    for n in 0..2000 {
        let entries: Vec<String> = (0..rng.below(5))
            .map(|_| {
                if rng.chance(8) {
                    return rng.pick(VALUES).to_owned();
                }
                let members: Vec<String> = (0..rng.below(5))
                    .map(|_| {
                        let key =
                            rng.pick(&["path", "defines", "+defines", "-defines", "tag", "x"]);
                        let value = if key == "path" && !rng.chance(4) {
                            "\"materials/a.material\""
                        } else {
                            rng.pick(VALUES)
                        };
                        format!("\"{key}\": {value}")
                    })
                    .collect();
                format!("{{{}}}", members.join(", "))
            })
            .collect();
        let text = if rng.chance(20) {
            rng.pick(VALUES).to_owned()
        } else {
            format!("[{}]", entries.join(", "))
        };
        let Ok(first) = parse_list("materials/fancy.json", &text) else {
            continue;
        };
        loaded += 1;
        let written = first
            .to_json()
            .unwrap_or_else(|error| panic!("#{n} not written: {error}\n{text}"));
        let second: bedrock_addon::document::Document<MaterialList> =
            parse_list("materials/fancy.json", &written).expect("reads back");
        assert_eq!(*first, *second, "#{n}\n{text}\n{written}");
        assert_eq!(second.to_json().unwrap(), written, "#{n}");
        let findings = rendered(first.diagnostics().iter());
        let found = compared_with_the_input(n, &text, &written, &findings, Read::List);
        as_authored += usize::from(found.is_none());
        total.add(found);
    }
    assert!(loaded > 1500, "{loaded} loaded");
    // A list holds no float: an entry or a define is dropped, a number past the doubles in a
    // member the list keeps is clamped, and nothing else differs.
    eprintln!("generated lists: {loaded} loaded, {as_authored} written as authored, {total:?}");
    assert!(as_authored > 200, "{as_authored} as authored");
    assert!(total.dropped > 100, "{total:?}");
    assert!(total.clamped > 0, "{total:?}");
    assert_eq!(total.rounded, 0, "{total:?}");
}

/// The keys two generated loads spell their materials with: three names, each alone, under a
/// parent and as an extension.
const GROUP_KEYS: &[&str] = &["P", "C:P", "+P", "C", "+C", "D:C", "+D", "D:P", "P:D"];

/// What a generated material writes of variants.
const GROUP_VARIANTS: &[&str] = &[
    "",
    "",
    r#", "+variants": [{"v": {}}]"#,
    r#", "variants": [{"v": {}}]"#,
    r#", "+variants": [{"w": {}}]"#,
    r#", "variants": []"#,
    r#", "+variants": [{"v": {}}, {"v.x": {}}]"#,
    r#", "+variants": [{"w.x": {}}]"#,
];

/// One material of a generated load: its key and what it writes of variants (indices into
/// the two vocabularies).
type Entry = (usize, usize);

/// The text of a generated file, `wrapped` or flat. What a material writes besides its
/// variants is the same in both loads of a pair: it depends on the file and on the name the
/// key creates alone. One name in three writes a plain `defines`, which the versioned rules
/// read as "drop the inherited ones" and the legacy rules as "add".
fn group_file(file: usize, entries: &[Entry], funcs: &[usize], wrapped: bool) -> String {
    let members: Vec<String> = entries
        .iter()
        .map(|&(key, variants)| {
            let key = GROUP_KEYS[key];
            let name = MaterialKey::parse(key).name;
            let choice = funcs[file * 3 + (name.as_bytes()[0] % 3) as usize];
            let func = ["Always", "Equal", "Less"][choice];
            let defines = ["defines", "+defines", "+defines"][choice];
            format!(
                r#""{key}": {{"{defines}": ["F{file}_{name}"], "depthFunc": "{func}"{}}}"#,
                GROUP_VARIANTS[variants]
            )
        })
        .collect();
    if wrapped {
        crate::common::wrapped(&members.join(", "))
    } else {
        crate::common::flat(&members.join(", "))
    }
}

/// The stack of one pack whose common list names the two generated files; `wrapped` says of
/// each whether it is written in the wrapped form.
fn group_stack(
    files: &[Vec<Entry>; 2],
    funcs: &[usize],
    wrapped: [bool; 2],
) -> bedrock_addon::pack::PackStack<bedrock_addon::pack::Resource> {
    let listing = r#"[{"path": "materials/a.material", "+defines": ["A"]}, {"path": "materials/b.material", "+defines": ["B"]}]"#;
    let (a, b) = (
        group_file(0, &files[0], funcs, wrapped[0]),
        group_file(1, &files[1], funcs, wrapped[1]),
    );
    crate::common::stack(&[&[
        ("materials/common.json", listing),
        ("materials/a.material", &a),
        ("materials/b.material", &b),
    ]])
}

/// Up to three materials under different keys.
fn group_entries(rng: &mut Rng) -> Vec<Entry> {
    let mut entries: Vec<Entry> = Vec::new();
    for _ in 0..=rng.below(3) {
        let key = rng.below(GROUP_KEYS.len());
        if entries.iter().all(|entry| entry.0 != key) {
            entries.push((key, rng.below(GROUP_VARIANTS.len())));
        }
    }
    entries
}

#[test]
fn two_generated_loads_of_the_same_members_in_either_form_whose_groups_are_equal_resolve_alike() {
    use bedrock_addon::families::material::MaterialGroupKind::Common;
    let (mut equal, mut equal_from_other_text, mut different, mut other_names) = (0, 0, 0, 0);
    // Of the pairs that differ in the form of a file alone: how many, how many of them have
    // different groups, and how many of those resolve differently.
    let (mut form_alone, mut form_alone_different, mut form_alone_resolved) = (0, 0, 0);
    for seed in [
        0x5eed_0001_u64,
        0x0bad_cafe_f00d_0002,
        0x1357_9bdf_2468_ace3,
    ] {
        let mut rng = Rng(seed);
        for n in 0..1500 {
            let funcs: Vec<usize> = (0..6).map(|_| rng.below(3)).collect();
            let one = [group_entries(&mut rng), group_entries(&mut rng)];
            // The second load: the first with some keys and some variants written anew.
            let mut other = one.clone();
            for file in &mut other {
                for at in 0..file.len() {
                    if rng.chance(4) {
                        let key = rng.below(GROUP_KEYS.len());
                        if file.iter().all(|entry| entry.0 != key) {
                            file[at].0 = key;
                        }
                    }
                    if rng.chance(2) {
                        file[at].1 = rng.below(GROUP_VARIANTS.len());
                    }
                }
            }
            // Each file of each load wrapped or flat, every combination in turn: a material
            // of a flat file is read with the legacy rules.
            let form = |bit: usize| n >> bit & 1 == 0;
            let (one_forms, other_forms) = ([form(0), form(1)], [form(2), form(3)]);
            let first = group_stack(&one, &funcs, one_forms);
            let second = group_stack(&other, &funcs, other_forms);
            let (first, second) = (first.materials(), second.materials());
            let names = |materials: &bedrock_addon::families::material::Materials| {
                materials.common.names().collect::<Vec<_>>()
            };
            let resolved = |materials: &bedrock_addon::families::material::Materials| {
                materials
                    .resolve_all(Common)
                    .into_iter()
                    .collect::<Vec<_>>()
            };
            let same_names = names(first) == names(second);
            if one == other && one_forms != other_forms {
                form_alone += 1;
                if first.common != second.common {
                    form_alone_different += 1;
                    form_alone_resolved += usize::from(resolved(first) != resolved(second));
                }
            }
            if first.common == second.common {
                equal += 1;
                equal_from_other_text += usize::from(one != other);
                assert!(
                    same_names,
                    "seed {seed:#x} #{n}: equal groups, other names\n{one:?}\n{other:?}\n{:?}\n{:?}",
                    names(first),
                    names(second)
                );
                assert!(
                    resolved(first) == resolved(second),
                    "seed {seed:#x} #{n}: equal groups resolve differently\n{one:?}\n{other:?}"
                );
            } else {
                different += 1;
                other_names += usize::from(!same_names);
            }
        }
    }
    eprintln!(
        "generated loads: {equal} equal ({equal_from_other_text} of other text), {different} different ({other_names} in their names); {form_alone} differ in a file's form alone ({form_alone_different} with different groups, {form_alone_resolved} of them resolve differently)"
    );
    assert!(equal_from_other_text > 150, "{equal_from_other_text}");
    assert!(other_names > 300, "{other_names}");
    // The form of a file is seen: groups that differ in it alone are different groups, and
    // some of them resolve differently.
    assert!(form_alone_different > 100, "{form_alone_different}");
    assert!(form_alone_resolved > 20, "{form_alone_resolved}");
}

#[test]
#[should_panic(expected = "dropped without a finding at x/defines/1")]
fn a_drop_is_matched_with_a_finding_at_its_own_path() {
    // One element is missing from the written text and one finding says that an element
    // was dropped, of another array: the counts agree, the places do not.
    let finding = crate::common::finding(
        "core/invalid_type",
        bedrock_addon::diagnostic::Severity::Info,
        "x/states/1",
        "",
    );
    compared_with_the_input(
        0,
        r#"{"x": {"states": ["Blending", "Wireframe"], "defines": ["A", 5]}}"#,
        r#"{"x": {"states": ["Blending", "Wireframe"], "defines": ["A"]}}"#,
        &[finding],
        Read::File,
    );
}
