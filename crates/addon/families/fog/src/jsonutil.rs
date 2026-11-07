//! Reading a fog document below 1.26.20 with the game's strict schema
//! `fog_description_schema`: one walk, two members gated on the root's version.

use super::DistanceType;
use super::codes;
use crate::history::v1_21_90::{
    CoeffColorJu, Coefficients, Density, DensitySetting, Description, Distance, FogColorJu,
    FogPayload, FogSetting, HenyeyGreensteinG, HgSetting, InitFog, MediaCoefficients,
    TransitionFog, Volumetric,
};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, Diagnostics, JsonPath, Severity};
use bedrock_addon_core::json::JsonSlice;
use bedrock_addon_core::json::node_tree::{
    Failure, Group, Hit, Kind, Lossy, Names, Node, Order, Rules, Validation, Walk, percent_f,
};
use bedrock_addon_shared_shapes::color::is_hex;

/// The rules of the game's `fog_description_schema`: names as written, the members of an object
/// in sorted key order (jsoncpp's `std::map`), strict, an object failed by any member, and a
/// lossy float told.
pub(crate) const RULES: Rules = Rules {
    names: Names::Exact,
    order: Order::Sorted,
    validation: Validation::Strict,
    failure: Failure::MemberFailed,
    lossy: Lossy::Told,
};

/// The two `render_distance_type` patterns as the string node prints them: the argument of
/// the node's "not allowed" line.
const DISTANCE_PATTERNS: &str = "fixed or render";

/// Which root of the schema reads a document.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Era {
    /// `[1.16.100, 1.21.0)`.
    V1_16_100,
    /// `[1.21.0, 1.21.90)`: `volumetric.density.weather`.
    V1_21_0,
    /// `[1.21.90, 1.26.20)`: `volumetric.henyey_greenstein_g`.
    V1_21_90,
}

/// Reads the whole document (the root holds `format_version` and the payload) with the root of
/// `era`, findings below `at`. `None` when validation failed; a member `era` does not declare
/// is an unknown member, so the result holds none.
pub(crate) fn read_document(
    document: JsonSlice<'_>,
    era: Era,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> Option<FogPayload> {
    let mut walk = Walk::new(RULES, at);
    let read = Node::of(document).and_then(|root| root_node(&mut walk, root, era));
    let failed = walk.diagnostics.has_errors();
    d.extend(walk.diagnostics);
    read.filter(|_| !failed)
}

/// A group list from member names, each required or not.
fn groups(names: &[(&'static str, bool)]) -> Vec<Group> {
    names
        .iter()
        .map(|(name, required)| {
            if *required {
                Group::required(name)
            } else {
                Group::name(name)
            }
        })
        .collect()
}

/// An object node, or the type error.
fn is_object(walk: &mut Walk, node: Node<'_>) -> bool {
    if node.kind() == Kind::Object {
        return true;
    }
    walk.bad_type("object", node);
    false
}

fn root_node(walk: &mut Walk, root: Node<'_>, era: Era) -> Option<FogPayload> {
    if !is_object(walk, root) {
        return None;
    }
    let mut format_version = None;
    let mut settings = None;
    let members = groups(&[("format_version", true), ("minecraft:fog_settings", true)]);
    let ok = walk.object(root, &members, |walk, group, _, v| {
        if group == 0 {
            // `addChild<SemVersion>` with the root's own version as its minimum: the
            // dispatcher never routes a lower one here.
            if v.kind() != Kind::String {
                walk.bad_type("SemVersion", v);
                return Hit::Failed;
            }
            walk.string(v).map(|s| format_version = Some(s)).into()
        } else {
            settings_node(walk, v, era)
                .map(|s| settings = Some(s))
                .into()
        }
    });
    let (description, distance, volumetric) = settings.filter(|_| ok)?;
    Some(FogPayload {
        format_version: format_version?,
        description,
        distance,
        volumetric,
    })
}

type Settings = (Description, Option<Distance>, Option<Volumetric>);

fn settings_node(walk: &mut Walk, node: Node<'_>, era: Era) -> Option<Settings> {
    if !is_object(walk, node) {
        return None;
    }
    let mut description = None;
    let mut distance = None;
    let mut volumetric = None;
    let members = groups(&[
        ("description", true),
        ("distance", false),
        ("volumetric", false),
    ]);
    let ok = walk.object(node, &members, |walk, group, _, v| match group {
        0 => description_node(walk, v)
            .map(|x| description = Some(x))
            .into(),
        1 => distance_node(walk, v).map(|x| distance = Some(x)).into(),
        _ => volumetric_node(walk, v, era)
            .map(|x| volumetric = Some(x))
            .into(),
    });
    ok.then_some((description?, distance, volumetric))
}

/// V9: the lower-cased identifier fully matches `[a-z0-9._]+:[a-z0-9._]+`.
// The game's schema validator and the `/fog` command share this check.
pub(crate) fn is_namespaced(identifier: &str) -> bool {
    let part = |text: &str| {
        !text.is_empty()
            && text
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'_')
    };
    identifier
        .split_once(':')
        .is_some_and(|(namespace, name)| part(namespace) && part(name))
}

fn description_node(walk: &mut Walk, node: Node<'_>) -> Option<Description> {
    if !is_object(walk, node) {
        return None;
    }
    let mut identifier = None;
    let ok = walk.object(node, &groups(&[("identifier", true)]), |walk, _, _, v| {
        let Some(text) = walk.string(v) else {
            return Hit::Failed;
        };
        if !is_namespaced(&text) {
            // The node's own message is not recorded; the file fails with the closing line.
            walk.error(Code::from(&core_codes::INVALID_VALUE), v.span());
            return Hit::Failed;
        }
        identifier = Some(text);
        Hit::Ok
    });
    ok.then_some(Description {
        identifier: identifier?,
    })
}

fn distance_node(walk: &mut Walk, node: Node<'_>) -> Option<Distance> {
    if !is_object(walk, node) {
        return None;
    }
    let mut distance = Distance::default();
    let members = groups(&[
        ("air", false),
        ("weather", false),
        ("water", false),
        ("lava", false),
        ("lava_resistance", false),
        ("powder_snow", false),
    ]);
    let ok = walk.object(node, &members, |walk, group, _, v| {
        let Some(setting) = fog_setting(walk, v) else {
            return Hit::Failed;
        };
        let slot = match group {
            0 => &mut distance.air,
            1 => &mut distance.weather,
            2 => &mut distance.water,
            3 => &mut distance.lava,
            4 => &mut distance.lava_resistance,
            _ => &mut distance.powder_snow,
        };
        *slot = Some(setting);
        Hit::Ok
    });
    ok.then_some(distance)
}

/// A float node with the packed `(min, max)` pair of its schema node; `f32::MAX` is the
/// node's default maximum.
fn ranged(walk: &mut Walk, node: Node<'_>, min: f32, max: f32) -> Option<f32> {
    let x = walk.float(node)?;
    if x < min || x > max {
        walk.invalid(
            core_codes::OUTSIDE_RANGE.with([percent_f(x), percent_f(min), percent_f(max)]),
            node,
        );
        return None;
    }
    Some(x)
}

/// `render_distance_type`: lower-cased, it fully matches `fixed` or `render`.
fn distance_type(walk: &mut Walk, node: Node<'_>) -> Option<DistanceType> {
    let text = walk.string(node)?;
    let read = match text.to_ascii_lowercase().as_str() {
        "fixed" => DistanceType::Fixed,
        "render" => DistanceType::Render,
        _ => {
            walk.invalid(
                core_codes::NOT_ALLOWED_REGEX.with([text.as_str(), DISTANCE_PATTERNS]),
                node,
            );
            return None;
        }
    };
    if text.bytes().any(|byte| byte.is_ascii_uppercase()) {
        // The type is held, and written in lower case: the spelling of the file is noted
        // (ours; the game reads both alike).
        walk.push(
            Code::from(&core_codes::VALUE_REWRITTEN),
            Severity::Info,
            Some(node.span()),
        );
    }
    Some(read)
}

/// The string form of a colour node, or its error.
fn hex(walk: &mut Walk, node: Node<'_>) -> Option<String> {
    let text = walk.string(node)?;
    // `#rrggbb` in either case, in full.
    if is_hex(&text, 6) {
        Some(text)
    } else {
        walk.error(Code::from(&codes::COLOR_FORMAT), node.span());
        None
    }
}

/// The first three elements of a colour array; fewer fail the node. An element behind the
/// third is not read by the game and is not held: noted
/// at the element (ours; INFO).
fn rgb_nodes<'a>(walk: &mut Walk, node: Node<'a>) -> Option<[Node<'a>; 3]> {
    let elements = walk.elements(node)?;
    if let [r, g, b, rest @ ..] = elements.as_slice() {
        for (offset, element) in rest.iter().enumerate() {
            walk.enter_index(3 + offset);
            walk.push(
                Code::from(&core_codes::DROPPED_FIELD),
                Severity::Info,
                Some(element.span()),
            );
            walk.leave();
        }
        Some([*r, *g, *b])
    } else {
        walk.error(Code::from(&codes::COLOR_SHAPE), node.span());
        None
    }
}

/// Neither a string nor an array: the base node's validation refuses it (its text is not
/// recorded).
fn not_a_color(walk: &mut Walk, node: Node<'_>) {
    walk.error(Code::from(&core_codes::INVALID_TYPE), node.span());
}

/// `fog_color` / `init_fog.fog_color`: `#rrggbb`, or an array read through
/// `parseValue<mce::Color>`: an integral element (an integer or a bool) clamped to 0–255, a
/// real 255 with "Expected Integral Number." while the file still loads. A bool comes with an
/// `InvalidType` information at the element. Any other element
/// fails the schema's colour validator, so the file is refused.
fn fog_color(walk: &mut Walk, node: Node<'_>) -> Option<FogColorJu> {
    match node.kind() {
        Kind::String => hex(walk, node).map(FogColorJu::Hex),
        Kind::Array => {
            let nodes = rgb_nodes(walk, node)?;
            let mut rgb = [255_i32; 3];
            let mut ok = true;
            for (index, element) in nodes.into_iter().enumerate() {
                walk.enter_index(index);
                match element.kind() {
                    // The schema's colour validator
                    // ANDs `parseValue<float>(element, true)` over the elements, which fails a
                    // `null` without a line. `registerFogFromJson` then logs "Failed to
                    // validate fog schema" (`_contentLog(true, 3, 26)`) and never registers
                    // the fog: the file is refused, and the line is the closing one's.
                    Kind::Null => {
                        walk.error(Code::from(&core_codes::INVALID_TYPE), element.span());
                        ok = false;
                    }
                    // Read as vanilla reads it, and said: the element is
                    // written as the number it was read as. `isNumeric` and `isIntegral` both
                    // pass a bool without a line, so it is an information.
                    Kind::Bool => {
                        walk.push(
                            Code::from(&core_codes::INVALID_TYPE),
                            Severity::Info,
                            Some(element.span()),
                        );
                        rgb[index] = i32::from(element.text() == "true");
                    }
                    Kind::Number => match element.text().parse::<i64>() {
                        Ok(written) => {
                            let clamped = written.clamp(0, 255);
                            if clamped != written {
                                // Ours; vanilla clamps without a word.
                                walk.push(
                                    Code::from(&core_codes::CLAMPED_VALUE),
                                    Severity::Info,
                                    Some(element.span()),
                                );
                            }
                            rgb[index] = i32::try_from(clamped).unwrap_or(255);
                        }
                        // V10e. `parseValue<mce::Color>` logs the line at level 3
                        // (`_contentLog(true, 3, 17)`) and the fog reader ignores the
                        // parse result, so the file loads: a kept error.
                        Err(_) => {
                            walk.push_kept(Code::from(&codes::INTEGRAL), Some(element.span()));
                        }
                    },
                    _ => {
                        // Validation reads each element as a float first:
                        // `parseValue<float>` logs "Expected Number." at level 3
                        // (`_contentLog(true, 3, 17)`) and fails, and the file is
                        // refused as for a `null`.
                        walk.error(Code::from(&codes::EXPECTED_NUMBER), element.span());
                        ok = false;
                    }
                }
                walk.leave();
            }
            ok.then_some(FogColorJu::Array(rgb))
        }
        _ => {
            not_a_color(walk, node);
            None
        }
    }
}

/// `scattering` / `absorption`: `#rrggbb`, or an array of at least three numbers, each
/// clamped to 0–1 by the game when it is used.
fn coefficient(walk: &mut Walk, node: Node<'_>) -> Option<CoeffColorJu> {
    match node.kind() {
        Kind::String => hex(walk, node).map(CoeffColorJu::Hex),
        Kind::Array => {
            let nodes = rgb_nodes(walk, node)?;
            let mut rgb = [0.0_f32; 3];
            let mut ok = true;
            for (index, element) in nodes.into_iter().enumerate() {
                walk.enter_index(index);
                match walk.float(element) {
                    Some(x) => rgb[index] = x,
                    None => ok = false,
                }
                walk.leave();
            }
            ok.then_some(CoeffColorJu::Array(rgb))
        }
        _ => {
            not_a_color(walk, node);
            None
        }
    }
}

/// The four members every distance node requires.
#[derive(Default)]
struct Base {
    fog_color: Option<FogColorJu>,
    fog_start: Option<f32>,
    fog_end: Option<f32>,
    render_distance_type: Option<DistanceType>,
}

impl Base {
    /// Reads member `group` (0–3) of a distance node.
    fn read(&mut self, walk: &mut Walk, group: usize, v: Node<'_>) -> bool {
        match group {
            0 => fog_color(walk, v)
                .map(|x| self.fog_color = Some(x))
                .is_some(),
            // `fog_start` keeps the node's default range.
            1 => walk.float(v).map(|x| self.fog_start = Some(x)).is_some(),
            2 => ranged(walk, v, 0.0, f32::MAX)
                .map(|x| self.fog_end = Some(x))
                .is_some(),
            _ => distance_type(walk, v)
                .map(|x| self.render_distance_type = Some(x))
                .is_some(),
        }
    }
}

const BASE_MEMBERS: [(&str, bool); 4] = [
    ("fog_color", true),
    ("fog_start", true),
    ("fog_end", true),
    ("render_distance_type", true),
];

fn fog_setting(walk: &mut Walk, node: Node<'_>) -> Option<FogSetting> {
    if !is_object(walk, node) {
        return None;
    }
    let mut base = Base::default();
    let mut transition_fog = None;
    let mut members = BASE_MEMBERS.to_vec();
    // Allowed on all six entries.
    members.push(("transition_fog", false));
    let ok = walk.object(node, &groups(&members), |walk, group, _, v| {
        if group < 4 {
            base.read(walk, group, v).into()
        } else {
            transition(walk, v).map(|x| transition_fog = Some(x)).into()
        }
    });
    ok.then_some(FogSetting {
        fog_color: base.fog_color?,
        fog_start: base.fog_start?,
        fog_end: base.fog_end?,
        render_distance_type: base.render_distance_type?,
        transition_fog,
    })
}

fn init_fog(walk: &mut Walk, node: Node<'_>) -> Option<InitFog> {
    if !is_object(walk, node) {
        return None;
    }
    let mut base = Base::default();
    let ok = walk.object(node, &groups(&BASE_MEMBERS), |walk, group, _, v| {
        base.read(walk, group, v).into()
    });
    ok.then_some(InitFog {
        fog_color: base.fog_color?,
        fog_start: base.fog_start?,
        fog_end: base.fog_end?,
        render_distance_type: base.render_distance_type?,
    })
}

#[allow(clippy::similar_names, reason = "the JSON members' own names")]
fn transition(walk: &mut Walk, node: Node<'_>) -> Option<TransitionFog> {
    if !is_object(walk, node) {
        return None;
    }
    let mut init = None;
    let mut numbers = [None; 4];
    let members = groups(&[
        ("init_fog", true),
        ("min_percent", true),
        ("mid_seconds", true),
        ("mid_percent", true),
        ("max_seconds", true),
    ]);
    let ok = walk.object(node, &members, |walk, group, _, v| match group {
        0 => init_fog(walk, v).map(|x| init = Some(x)).into(),
        // The percents are [0, 1]; the seconds have a minimum of 0 only.
        1 | 3 => ranged(walk, v, 0.0, 1.0)
            .map(|x| numbers[group - 1] = Some(x))
            .into(),
        _ => ranged(walk, v, 0.0, f32::MAX)
            .map(|x| numbers[group - 1] = Some(x))
            .into(),
    });
    let [min_percent, mid_seconds, mid_percent, max_seconds] = numbers;
    ok.then_some(TransitionFog {
        init_fog: init?,
        min_percent: min_percent?,
        mid_seconds: mid_seconds?,
        mid_percent: mid_percent?,
        max_seconds: max_seconds?,
    })
}

fn volumetric_node(walk: &mut Walk, node: Node<'_>, era: Era) -> Option<Volumetric> {
    if !is_object(walk, node) {
        return None;
    }
    let mut volumetric = Volumetric::default();
    let mut members = vec![("density", false), ("media_coefficients", false)];
    if era >= Era::V1_21_90 {
        // `FOG_DEFINITION_VERSION_1_21_90`.
        members.push(("henyey_greenstein_g", false));
    }
    let ok = walk.object(node, &groups(&members), |walk, group, _, v| match group {
        0 => density_node(walk, v, era)
            .map(|x| volumetric.density = Some(x))
            .into(),
        1 => media_node(walk, v)
            .map(|x| volumetric.media_coefficients = Some(x))
            .into(),
        _ => hg_node(walk, v)
            .map(|x| volumetric.henyey_greenstein_g = Some(x))
            .into(),
    });
    ok.then_some(volumetric)
}

fn density_node(walk: &mut Walk, node: Node<'_>, era: Era) -> Option<Density> {
    if !is_object(walk, node) {
        return None;
    }
    let mut density = Density::default();
    let mut members = vec![
        ("air", false),
        ("water", false),
        ("lava", false),
        ("lava_resistance", false),
    ];
    if era >= Era::V1_21_0 {
        // `FOG_DEFINITION_VERSION_1_21_0`.
        members.push(("weather", false));
    }
    let ok = walk.object(node, &groups(&members), |walk, group, _, v| {
        let Some(setting) = density_setting(walk, v) else {
            return Hit::Failed;
        };
        let slot = match group {
            0 => &mut density.air,
            1 => &mut density.water,
            2 => &mut density.lava,
            3 => &mut density.lava_resistance,
            _ => &mut density.weather,
        };
        *slot = Some(setting);
        Hit::Ok
    });
    ok.then_some(density)
}

fn density_setting(walk: &mut Walk, node: Node<'_>) -> Option<DensitySetting> {
    if !is_object(walk, node) {
        return None;
    }
    let mut max_density = None;
    let mut max_density_height = None;
    let mut zero_density_height = None;
    let mut uniform = None;
    let members = groups(&[
        ("max_density", true),
        ("max_density_height", false),
        ("zero_density_height", false),
        ("uniform", false),
    ]);
    let ok = walk.object(node, &members, |walk, group, _, v| match group {
        0 => ranged(walk, v, 0.0, 1.0)
            .map(|x| max_density = Some(x))
            .into(),
        1 => ranged(walk, v, -64.0, 320.0)
            .map(|x| max_density_height = Some(x))
            .into(),
        2 => ranged(walk, v, -64.0, 320.0)
            .map(|x| zero_density_height = Some(x))
            .into(),
        _ => walk.bool(v).map(|x| uniform = Some(x)).into(),
    });
    ok.then_some(DensitySetting {
        max_density: max_density?,
        max_density_height,
        zero_density_height,
        uniform,
    })
}

fn media_node(walk: &mut Walk, node: Node<'_>) -> Option<MediaCoefficients> {
    if !is_object(walk, node) {
        return None;
    }
    let mut media = MediaCoefficients::default();
    let members = groups(&[("air", false), ("water", false), ("cloud", false)]);
    let ok = walk.object(node, &members, |walk, group, _, v| {
        if !is_object(walk, v) {
            return Hit::Failed;
        }
        let mut scattering = None;
        let mut absorption = None;
        let pair = groups(&[("scattering", true), ("absorption", true)]);
        let ok = walk.object(v, &pair, |walk, member, _, v| {
            let color = coefficient(walk, v);
            let read = color.is_some();
            if member == 0 {
                scattering = color;
            } else {
                absorption = color;
            }
            read.into()
        });
        let (true, Some(scattering), Some(absorption)) = (ok, scattering, absorption) else {
            return Hit::Failed;
        };
        let slot = match group {
            0 => &mut media.air,
            1 => &mut media.water,
            _ => &mut media.cloud,
        };
        *slot = Some(Coefficients {
            scattering,
            absorption,
        });
        Hit::Ok
    });
    ok.then_some(media)
}

fn hg_node(walk: &mut Walk, node: Node<'_>) -> Option<HenyeyGreensteinG> {
    if !is_object(walk, node) {
        return None;
    }
    let mut hg = HenyeyGreensteinG::default();
    let members = groups(&[("air", false), ("water", false)]);
    let ok = walk.object(node, &members, |walk, group, _, v| {
        if !is_object(walk, v) {
            return Hit::Failed;
        }
        let mut g = None;
        let member = groups(&[("henyey_greenstein_g", true)]);
        let ok = walk.object(v, &member, |walk, _, _, v| {
            ranged(walk, v, -1.0, 1.0).map(|x| g = Some(x)).into()
        });
        let (true, Some(g)) = (ok, g) else {
            return Hit::Failed;
        };
        let setting = Some(HgSetting {
            henyey_greenstein_g: g,
        });
        if group == 0 {
            hg.air = setting;
        } else {
            hg.water = setting;
        }
        Hit::Ok
    });
    ok.then_some(hg)
}
