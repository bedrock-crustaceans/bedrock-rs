//! Reading a fog document below 1.26.20 with the game's strict schema
//! `fog_description_schema` on the derived read: the payload types (`history::payload`) are the
//! schema, two members gated on the root's version (`schema::since_format`); what has a syntax of
//! its own reads by a leaf here (the identifier, the ranged floats, the distance type, the colour
//! forms).

use facet::Facet;

use super::DistanceType;
use super::codes;
use crate::history::v1_21_90::{
    CoeffColorJu, Description, Distance, FogColorJu, FogPayload, Volumetric,
};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, Diagnostic, Diagnostics, JsonPath, Severity};
use bedrock_addon_core::json::kit::{Lossy, MemberOrder, Node, Profile, Read, Texts, Versions};
use bedrock_addon_core::json::parse::{Options, parse_slice};
use bedrock_addon_core::json::{JsonSlice, Winner};
use bedrock_addon_core::schema::{self, FromJson, Schema, read_document as read_value};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::Reading;
use bedrock_addon_shared_shapes::color::is_hex;

/// The two `render_distance_type` patterns as the string node prints them: the argument of
/// the node's "not allowed" line.
const DISTANCE_PATTERNS: &str = "fixed or render";

/// The profile of that schema: the node tree's, names as written, the members of an object in
/// sorted key order (jsoncpp's `std::map`), and a lossy float told.
pub(crate) const PROFILE: Profile = Profile::TREE
    .with_members(MemberOrder::Sorted)
    .with_lossy(Lossy::Told);

/// The whole document as the schema reads it: `format_version` and the payload.
#[derive(Facet, Schema)]
#[facet(schema::molang_with = no_molang)]
struct Document {
    #[facet(schema::read_with = sem_version)]
    format_version: String,
    #[facet(rename = "minecraft:fog_settings")]
    settings: Settings,
}

/// `minecraft:fog_settings`.
#[derive(Facet, Schema)]
#[facet(schema::molang_with = no_molang)]
struct Settings {
    description: Description,
    distance: Option<Distance>,
    volumetric: Option<Volumetric>,
}

/// A fog document holds no Molang.
pub(crate) fn no_molang<T>(_: &mut T, _: &mut bedrock_addon_core::molang::MolangVisitor<'_>) {}

/// Reads the whole document (the root holds `format_version` and the payload) with the root of
/// the format `version`, findings below `at`. `None` when validation failed; a member `version`
/// does not declare is an unknown member, so the result holds none.
pub(crate) fn read_document(
    document: JsonSlice<'_>,
    version: FormatVersion,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> Option<FogPayload> {
    let text = parse_slice(document, Options::winner(Winner::Last)).ok()?;
    let mut found = Diagnostics::default();
    let mut read = Read::new(Reading::TREE, &mut found)
        .profile(PROFILE)
        .texts(Texts::TREE)
        .versions(Versions {
            format: Some(version),
            engine: None,
        });
    let read_it = read_value::<Document>(&Node::parsed(&text, at.clone()), &mut read);
    drop(read);
    let failed = found.has_errors();
    d.extend(found);
    let Document {
        format_version,
        settings,
    } = read_it.filter(|_| !failed)?;
    Some(FogPayload {
        format_version,
        description: settings.description,
        distance: settings.distance,
        volumetric: settings.volumetric,
    })
}

/// `format_version`: `addChild<SemVersion>` with the root's own version as its minimum, which
/// the dispatcher never routes a lower one to: a string, held as written.
fn sem_version(node: &Node<'_>, read: &mut Read<'_>) -> Option<String> {
    let Some(text) = node.value().as_string() else {
        read.wrong_accepting(node, "SemVersion");
        return None;
    };
    Some(text.as_str().to_owned())
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

/// `description.identifier`: a namespaced name. The node's own message is not recorded; the file
/// fails with the closing line.
pub(crate) fn identifier(node: &Node<'_>, read: &mut Read<'_>) -> Option<String> {
    let text = String::from_json(node, read)?;
    if !is_namespaced(&text) {
        read.error(node, Code::from(&core_codes::INVALID_VALUE));
        return None;
    }
    Some(text)
}

/// C's `%f` of a float: six decimals, no exponent.
fn percent_f(x: f32) -> String {
    format!("{:.6}", f64::from(x))
}

/// A float node with the packed `(min, max)` pair of its schema node; `f32::MAX` is the node's
/// default maximum.
fn ranged(node: &Node<'_>, read: &mut Read<'_>, min: f32, max: f32) -> Option<f32> {
    let x = f32::from_json(node, read)?;
    if x < min || x > max {
        let code = core_codes::OUTSIDE_RANGE.with([percent_f(x), percent_f(min), percent_f(max)]);
        read.error(node, code);
        return None;
    }
    Some(x)
}

/// A float of at least 0 (`fog_end`, the seconds of a transition).
pub(crate) fn positive(node: &Node<'_>, read: &mut Read<'_>) -> Option<f32> {
    ranged(node, read, 0.0, f32::MAX)
}

/// A float of 0 to 1 (the percents of a transition, `max_density`).
pub(crate) fn unit(node: &Node<'_>, read: &mut Read<'_>) -> Option<f32> {
    ranged(node, read, 0.0, 1.0)
}

/// A height of −64 to 320.
pub(crate) fn height(node: &Node<'_>, read: &mut Read<'_>) -> Option<f32> {
    ranged(node, read, -64.0, 320.0)
}

/// `henyey_greenstein_g`: −1 to 1.
pub(crate) fn scattering_g(node: &Node<'_>, read: &mut Read<'_>) -> Option<f32> {
    ranged(node, read, -1.0, 1.0)
}

/// `render_distance_type`: lower-cased, it fully matches `fixed` or `render`.
pub(crate) fn distance_type(node: &Node<'_>, read: &mut Read<'_>) -> Option<DistanceType> {
    let text = String::from_json(node, read)?;
    let distance = match text.to_ascii_lowercase().as_str() {
        "fixed" => DistanceType::Fixed,
        "render" => DistanceType::Render,
        _ => {
            let code = core_codes::NOT_ALLOWED_REGEX.with([text.as_str(), DISTANCE_PATTERNS]);
            read.error(node, code);
            return None;
        }
    };
    if text.bytes().any(|byte| byte.is_ascii_uppercase()) {
        // The type is held, and written in lower case: the spelling of the file is noted (ours;
        // the game reads both alike).
        read.push(told(
            node,
            Code::from(&core_codes::VALUE_REWRITTEN),
            Severity::Info,
        ));
    }
    Some(distance)
}

/// A finding of `code` at `node`, at `severity`.
fn told(node: &Node<'_>, code: Code, severity: Severity) -> Diagnostic {
    let mut finding = Diagnostic::error(code)
        .severity(severity)
        .at(node.path().clone());
    finding.location.span = node.span();
    finding
}

/// The string form of a colour node, or its error.
fn hex(node: &Node<'_>, read: &mut Read<'_>) -> Option<String> {
    let text = String::from_json(node, read)?;
    // `#rrggbb` in either case, in full.
    if is_hex(&text, 6) {
        return Some(text);
    }
    read.error(node, Code::from(&codes::COLOR_FORMAT));
    None
}

/// The first three elements of a colour array; fewer fail the node. An element behind the
/// third is not read by the game and is not held: noted at the element (ours; INFO).
fn rgb_nodes<'v>(node: &Node<'v>, read: &mut Read<'_>) -> Option<[Node<'v>; 3]> {
    let elements = read.array(node)?;
    let elements: Vec<Node<'v>> = elements.iter().collect();
    let [r, g, b, rest @ ..] = elements.as_slice() else {
        read.error(node, Code::from(&codes::COLOR_SHAPE));
        return None;
    };
    for element in rest {
        read.push(told(
            element,
            Code::from(&core_codes::DROPPED_FIELD),
            Severity::Info,
        ));
    }
    Some([r.clone(), g.clone(), b.clone()])
}

/// Neither a string nor an array: the base node's validation refuses it (its text is not
/// recorded).
fn not_a_color(node: &Node<'_>, read: &mut Read<'_>) {
    read.error(node, Code::from(&core_codes::INVALID_TYPE));
}

/// `fog_color` / `init_fog.fog_color`: `#rrggbb`, or an array read through
/// `parseValue<mce::Color>`: an integral element (an integer or a bool) clamped to 0–255, a
/// real 255 with "Expected Integral Number." while the file still loads. A bool comes with an
/// `InvalidType` information at the element. Any other element fails the schema's colour
/// validator, so the file is refused.
impl FromJson for FogColorJu {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let value = node.value();
        if value.is_string() {
            return hex(node, read).map(Self::Hex);
        }
        if !value.is_array() {
            not_a_color(node, read);
            return None;
        }
        let nodes = rgb_nodes(node, read)?;
        let mut rgb = [255_i32; 3];
        let mut ok = true;
        for (index, element) in nodes.iter().enumerate() {
            let value = element.value();
            if value.is_null() {
                // The schema's colour validator ANDs `parseValue<float>(element, true)` over the
                // elements, which fails a `null` without a line. `registerFogFromJson` then
                // logs "Failed to validate fog schema" (`_contentLog(true, 3, 26)`) and never
                // registers the fog: the file is refused, and the line is the closing one's.
                read.error(element, Code::from(&core_codes::INVALID_TYPE));
                ok = false;
            } else if let Some(flag) = value.as_bool() {
                // Read as vanilla reads it, and said: the element is written as the number it
                // was read as. `isNumeric` and `isIntegral` both pass a bool without a line, so
                // it is an information.
                read.push(told(
                    element,
                    Code::from(&core_codes::INVALID_TYPE),
                    Severity::Info,
                ));
                rgb[index] = i32::from(flag);
            } else if value.is_number() {
                let text = element.text().map_or("", |text| text.text);
                if let Ok(written) = text.parse::<i64>() {
                    let clamped = written.clamp(0, 255);
                    if clamped != written {
                        // Ours; vanilla clamps without a word.
                        let code = Code::from(&core_codes::CLAMPED_VALUE);
                        read.push(told(element, code, Severity::Info));
                    }
                    rgb[index] = i32::try_from(clamped).unwrap_or(255);
                } else {
                    // V10e. `parseValue<mce::Color>` logs the line at level 3
                    // (`_contentLog(true, 3, 17)`) and the fog reader ignores the parse result,
                    // so the file loads: a kept error.
                    let finding = told(element, Code::from(&codes::INTEGRAL), Severity::Error);
                    read.findings_mut().push_kept_error(finding);
                }
            } else {
                // Validation reads each element as a float first: `parseValue<float>` logs
                // "Expected Number." at level 3 (`_contentLog(true, 3, 17)`) and fails, and the
                // file is refused as for a `null`.
                read.error(element, Code::from(&codes::EXPECTED_NUMBER));
                ok = false;
            }
        }
        ok.then_some(Self::Array(rgb))
    }
}

/// `scattering` / `absorption`: `#rrggbb`, or an array of at least three numbers, each clamped to
/// 0–1 by the game when it is used.
impl FromJson for CoeffColorJu {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let value = node.value();
        if value.is_string() {
            return hex(node, read).map(Self::Hex);
        }
        if !value.is_array() {
            not_a_color(node, read);
            return None;
        }
        let nodes = rgb_nodes(node, read)?;
        let mut rgb = [0.0_f32; 3];
        let mut ok = true;
        for (index, element) in nodes.iter().enumerate() {
            match f32::from_json(element, read) {
                Some(x) => rgb[index] = x,
                None => ok = false,
            }
        }
        ok.then_some(Self::Array(rgb))
    }
}

bedrock_addon_core::impl_schema_leaf!(FogColorJu, CoeffColorJu);
bedrock_addon_core::impl_no_molang!(FogColorJu, CoeffColorJu);
