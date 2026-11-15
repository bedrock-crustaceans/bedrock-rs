//! Reading animation files: the `actor_animation` schema (the only one) read as the game's node
//! tree reads it, then the Molang passes and the crate's own findings. The document is the
//! derived read of [`Document`]; what has a syntax of its own (a loop mode, a channel, a
//! keyframe, a lerp mode) reads by a leaf here.

use facet::Facet;
use indexmap::IndexMap;

use super::{Animation, AnimationFile, AxisElement, Channel, Keyframe, LerpMode, LoopMode};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, Diagnostic, Diagnostics, JsonPath, Severity};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::json::kit::{Expected, Node, Read};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::molang::Molang;
use bedrock_addon_core::pack_identity::PackType;
use bedrock_addon_core::schema::{self, FromJson, KeyPattern, Keys, Schema, read_each, read_keyed};
use bedrock_addon_core::version::{ENGINE_VERSION, SemVersion};
use bedrock_addon_shared_animation::molang::{MolangRun, run};
use bedrock_addon_shared_animation::read::{Parsed, Written, format_version, input_of, text_of};
use bedrock_addon_shared_animation::walk::{AXIS, TIME};

/// The animation schema's `format_version` minimum.
pub(crate) const ANIMATION_MINIMUM: SemVersion = SemVersion::new(1, 8, 0);

/// The animation keys: `animation\.[a-zA-Z0-9_.-]+`, at least one.
const ANIMATIONS: Keys = Keys::new(KeyPattern::Prefixed {
    prefix: "animation.",
    extra: b"_.-",
    regex: r"animation\.[a-zA-Z0-9_.-]+",
})
.required();

/// An animation file as its schema reads it: the version as written and the animations.
#[derive(Facet, Schema)]
struct Document {
    #[facet(schema::read_with = version)]
    format_version: Written,
    #[facet(schema::keys = ANIMATIONS)]
    animations: IndexMap<String, Animation>,
}

/// The `format_version` node: a version string of at least 1.8.0.
fn version(node: &Node<'_>, read: &mut Read<'_>) -> Option<Written> {
    format_version(node, read, &ANIMATION_MINIMUM)
}

/// Reads one animation file (`animations/**/*.json`) of a pack with context `cx`.
pub(crate) fn parse_file(bytes: &[u8], cx: &LoadContext) -> Parsed<AnimationFile> {
    let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
    let mut diagnostics = Diagnostics::default();
    let failed = |diagnostics| Parsed {
        value: None,
        written: None,
        diagnostics,
    };
    let text = match text_of(bytes) {
        Ok(text) => text,
        Err(finding) => {
            diagnostics.push(*finding);
            return failed(diagnostics);
        }
    };
    let mut read = bedrock_addon_shared_animation::walk::read(cx, &mut diagnostics);
    let document =
        schema::read_document::<Document>(&Node::parsed(&text, JsonPath::new()), &mut read);
    drop(read);
    let Some(Document {
        format_version: Written(written),
        animations,
    }) = document
    else {
        return failed(diagnostics);
    };
    let mut file = AnimationFile {
        format_version: written.format_version(),
        animations,
    };
    if written
        > SemVersion::new(
            ENGINE_VERSION.major,
            ENGINE_VERSION.minor,
            ENGINE_VERSION.patch,
        )
    {
        // The game has no maximum format version and loads the file silently.
        let mut at = JsonPath::new();
        at.push("format_version");
        let code = if cx.pack_type == PackType::Resource {
            core_codes::FORMAT_VERSION_ABOVE_ENGINE.with([
                written.to_string(),
                format!(
                    "{}.{}.{}",
                    ENGINE_VERSION.major, ENGINE_VERSION.minor, ENGINE_VERSION.patch
                ),
            ])
        } else {
            // The game loads the file silently, so this finding is the crate's own.
            Code::from(&core_codes::UNSUPPORTED_VERSION)
        };
        diagnostics.push(Diagnostic::of(code).severity(Severity::Info).at(at));
    }
    super::validate::animation_findings(&file, cx, &mut diagnostics);
    let defaults = file
        .animations
        .values()
        .filter(|a| a.anim_time_update.is_none())
        .count();
    let keep = run(
        &mut file,
        &MolangRun {
            load: cx,
            written: input_of(&written),
            lowercase_transitions: false,
            transitions_typed: false,
        },
        defaults,
        &mut diagnostics,
    );
    let value = (keep && !diagnostics.has_errors()).then_some(file);
    Parsed {
        value,
        written: Some(written),
        diagnostics,
    }
}

/// `loop`: a bool, or the string `hold_on_last_frame`, matched case-sensitively.
impl FromJson for LoopMode {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let value = node.value();
        if let Some(looping) = value.as_bool() {
            return Some(if looping { Self::Loop } else { Self::Stop });
        }
        let Some(text) = value.as_string() else {
            read.wrong_accepting(node, "bool, string");
            return None;
        };
        if text.as_str() == "hold_on_last_frame" {
            return Some(Self::HoldOnLastFrame);
        }
        let code = core_codes::NOT_ALLOWED_REGEX.with([text.as_str(), "hold_on_last_frame"]);
        read.error(node, code);
        None
    }
}

/// `lerp_mode`: lower-cased, then matched against its keys.
impl FromJson for LerpMode {
    const EXPECTED: Option<Expected> = Some(Expected::String);

    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let value = read.string(node)?;
        match value.to_ascii_lowercase().as_str() {
            "linear" => Some(Self::Linear),
            "catmullrom" => Some(Self::CatmullRom),
            _ => {
                let code = core_codes::NOT_ALLOWED_KEYS.with([value, "linear, catmullrom"]);
                read.error(node, code);
                None
            }
        }
    }
}

/// One keyframe: three expressions, or the object of its parts.
impl FromJson for Keyframe {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let value = node.value();
        if value.is_array() {
            return expression3(node, read).map(Self::Vec3);
        }
        if value.is_object() {
            return FromJson::from_json(node, read).map(Self::Split);
        }
        read.wrong_accepting(node, "array, object");
        None
    }
}

bedrock_addon_core::impl_schema_leaf!(LoopMode, LerpMode, Keyframe, Channel);
bedrock_addon_core::impl_no_molang!(LoopMode, LerpMode, Keyframe, Channel);

/// `relative_to.rotation`: lower-cased, then matched against `entity`; kept as written.
pub(crate) fn entity(node: &Node<'_>, read: &mut Read<'_>) -> Option<String> {
    let value = read.string(node)?;
    let lower = value.to_ascii_lowercase();
    if lower != "entity" {
        read.error(node, core_codes::NOT_ALLOWED_REGEX.with([&lower, "entity"]));
        return None;
    }
    Some(value.to_owned())
}

/// A schema array of exactly three `ExpressionNode`s (`addChildArray(name, 3, …)`): told of by
/// its length first, then by its elements.
pub(crate) fn expression3(node: &Node<'_>, read: &mut Read<'_>) -> Option<[Molang; 3]> {
    let elements = read.array(node)?;
    let n = elements.len();
    if n != 3 {
        let def = if n < 3 {
            &core_codes::ARRAY_TOO_SMALL
        } else {
            &core_codes::ARRAY_TOO_LARGE
        };
        read.error(node, def.with([n.to_string(), "3".to_owned()]));
        return None;
    }
    let values: Vec<Molang> = elements
        .iter()
        .filter_map(|element| Molang::from_json(&element, read))
        .collect();
    values.try_into().ok()
}

/// The `position` and `scale` channels.
pub(crate) fn channel(node: &Node<'_>, read: &mut Read<'_>) -> Option<Channel> {
    read_channel(node, read, false)
}

/// The `rotation` channel, whose array may hold axis objects.
pub(crate) fn rotation_channel(node: &Node<'_>, read: &mut Read<'_>) -> Option<Channel> {
    read_channel(node, read, true)
}

/// Keyframes by their time key, at least one; axes by their name, at least one.
const KEYFRAMES: Keys = Keys::new(TIME).required();
const AXES: Keys = Keys::new(AXIS).required();

/// A bone channel in one of its four shapes. An array the array option's predicate refuses
/// (empty; neither three elements nor every element an object) is no option of the member:
/// vanilla moves on and ends with "child '%s' not valid here.".
fn read_channel(node: &Node<'_>, read: &mut Read<'_>, rotation: bool) -> Option<Channel> {
    let value = node.value();
    if value.is_object() {
        return read_keyed(node, read, &KEYFRAMES, Keyframe::from_json).map(Channel::Keyframes);
    }
    let Some(elements) = value.as_array() else {
        if value.is_null() {
            read.wrong_accepting(node, "ExpressionNode, array, object");
            return None;
        }
        return Molang::from_json(node, read).map(Channel::Uniform);
    };
    let accepted = match elements.len() {
        0 => false,
        3 => true,
        _ => elements.iter().all(Value::is_object),
    };
    if !accepted {
        let key = node.path().last_key().unwrap_or_default().to_owned();
        read.error(node, core_codes::CHILD_NOT_VALID.with([key]));
        return None;
    }
    let scalars = elements
        .iter()
        .all(|e| !e.is_object() && !e.is_array() && !e.is_null());
    let axes = read_each(node, read, |element, read| {
        let value = element.value();
        if value.is_object() && rotation {
            return read_keyed(element, read, &AXES, Molang::from_json).map(AxisElement::Axes);
        }
        if value.is_object() || value.is_array() || value.is_null() {
            read.wrong_accepting(element, "ExpressionNode");
            return None;
        }
        Molang::from_json(element, read).map(AxisElement::Positional)
    })?;
    if !scalars {
        return Some(Channel::AxisSequence(axes));
    }
    let values: Vec<Molang> = axes
        .into_iter()
        .filter_map(|e| match e {
            AxisElement::Positional(m) => Some(m),
            AxisElement::Axes(_) => None,
        })
        .collect();
    values.try_into().ok().map(Channel::Vec3)
}
