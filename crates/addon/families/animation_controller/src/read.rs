//! Reading animation controller files: the format-version gate, the 1.10-or-later schema read as
//! the game's node tree reads it, the 1.8 upgrade, then the gates, the Molang passes and the
//! reference stage. The documents are the derived reads of the version's `Document`; what has
//! a syntax of its own (a transition, a state's animation, a blend transition) reads by a leaf.

use facet::Facet;
use indexmap::IndexMap;

use super::codes;
use super::history::{AnimationControllerHistory, v1_8_0};
use super::{
    AnimationController, AnimationControllerFile, BlendTransition, StateAnimation, Transition,
};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, Diagnostic, Diagnostics, JsonPath, Severity, cereal};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::json::kit::{Expected, Node, Read};
use bedrock_addon_core::json::parse::Parsed as Text;
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::molang::{Molang, MolangSource, RawVersion};
use bedrock_addon_core::schema::{
    self, FromJson, KeyPattern, Keys, Schema, read_document, read_keyed,
};
use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::version::{ENGINE_VERSION, SemVersion};
use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};
use bedrock_addon_shared_animation::molang::{MolangRun, run};
use bedrock_addon_shared_animation::read::{Parsed, Written, format_version, input_of, text_of};
use bedrock_addon_shared_animation::walk::{IDENTIFIERS, TIME};

/// The 1.8 schema's `format_version` minimum and the controller floor.
pub(crate) const V1_8_0: SemVersion = SemVersion::new(1, 8, 0);
/// The 1.10 schema version: from here no upgrade runs.
pub(crate) const V1_10_0: SemVersion = SemVersion::new(1, 10, 0);
/// Below this version transition expressions are lower-cased.
pub(crate) const V1_17_30: SemVersion = SemVersion::new(1, 17, 30);
/// From this version the initial state's `on_entry` events fire when the controller starts.
pub(crate) const V1_18_10: SemVersion = SemVersion::new(1, 18, 10);

/// The controller keys: `controller\.animation\.[a-zA-Z0-9_.-]+`, at least one.
pub(crate) const CONTROLLERS: Keys = Keys::new(KeyPattern::Prefixed {
    prefix: "controller.animation.",
    extra: b"_.-",
    regex: r"controller\.animation\.[a-zA-Z0-9_.-]+",
})
.required();

/// A remap curve's keys: time keys, at least one.
pub(crate) const CURVE: Keys = Keys::new(TIME).required();

/// A 1.10-or-later controller file as its schema reads it: the version as written and the
/// controllers.
#[derive(Facet, Schema)]
struct Document {
    #[facet(schema::read_with = version_1_10)]
    format_version: Written,
    #[facet(schema::keys = CONTROLLERS)]
    animation_controllers: IndexMap<String, AnimationController>,
}

/// The 1.10 schema's `format_version` node.
fn version_1_10(node: &Node<'_>, read: &mut Read<'_>) -> Option<Written> {
    controller_format_version(node, read, &V1_10_0)
}

/// What the format-version gate read.
pub(crate) struct LoadedVersion {
    /// The version.
    pub(crate) version: SemVersion,
}

/// The format-version gate: the member found ignoring case, read as a version string or an array of
/// three unsigned integers; the three rejections are content-log errors that skip the file.
pub(crate) fn load_format_version(text: &Text<'_>, d: &mut Diagnostics) -> Option<LoadedVersion> {
    let member = text.value.as_object().and_then(|object| {
        object
            .iter()
            .find(|(key, _)| key.as_str().eq_ignore_ascii_case("format_version"))
            .map(|(key, value)| (key.as_str(), value))
    });
    let span = member.and_then(|(key, _)| text.spans.at(&JsonPath::of_keys([key])));
    let version = member.and_then(|(_, value)| read_version(value));
    let Some(version) = version else {
        let code = if member.is_none() {
            Code::from(&codes::ILL_FORMATTED_MISSING_FORMAT_VERSION)
        } else {
            Code::from(&codes::ILL_FORMATTED_UNSUPPORTED_VERSION)
        };
        let mut finding = Diagnostic::error(code);
        finding.location.span = span;
        d.push(finding);
        return None;
    };
    let engine = SemVersion::new(
        ENGINE_VERSION.major,
        ENGINE_VERSION.minor,
        ENGINE_VERSION.patch,
    );
    if version > engine {
        d.push(Diagnostic::error(
            codes::EXCEEDS_GAME.with([&version.to_string(), &engine.to_string()]),
        ));
        return None;
    }
    if version < V1_8_0 {
        d.push(Diagnostic::error(
            codes::UNKNOWN_FORMAT_VERSION.with([version.to_string()]),
        ));
        return None;
    }
    Some(LoadedVersion { version })
}

/// A version read without wildcards: a version string, or exactly three unsigned integers.
fn read_version(value: &Value) -> Option<SemVersion> {
    if let Some(text) = value.as_string() {
        let text = text.as_str();
        if text == "beta" {
            return None;
        }
        return SemVersion::parse(text).filter(|v| !v.is_any());
    }
    let numbers: Vec<u16> = value
        .as_array()?
        .iter()
        .map(|element| {
            let number = element.as_number().filter(|number| !number.is_float())?;
            number.to_u64().and_then(|n| u16::try_from(n).ok())
        })
        .collect::<Option<_>>()?;
    let [major, minor, patch] = numbers.as_slice() else {
        return None;
    };
    Some(SemVersion::new(*major, *minor, *patch))
}

/// Reads one controller file of a pack with context `cx`: version gate, the struct its version
/// selects, the upgrade, then [`finish`].
pub(crate) fn parse_file(bytes: &[u8], cx: &LoadContext) -> Parsed<AnimationControllerFile> {
    let (any, written, mut diagnostics) = parse_as_written(bytes, cx);
    let (Some(any), Some(written)) = (any, written) else {
        return Parsed {
            value: None,
            written: None,
            diagnostics,
        };
    };
    let value = upgrade(any, &written, cx, &mut diagnostics);
    let value = value.and_then(|file| finish(file, &written, cx, &mut diagnostics));
    Parsed {
        value,
        written: Some(written),
        diagnostics,
    }
}

/// The file as written: the struct its version selects, without upgrading or checking it
/// further.
pub(crate) fn parse_as_written(
    bytes: &[u8],
    cx: &LoadContext,
) -> (
    Option<AnimationControllerHistory>,
    Option<SemVersion>,
    Diagnostics,
) {
    let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
    let mut diagnostics = Diagnostics::default();
    let text = match text_of(bytes) {
        Ok(text) => text,
        Err(finding) => {
            diagnostics.push(*finding);
            return (None, None, diagnostics);
        }
    };
    let Some(loaded) = load_format_version(&text, &mut diagnostics) else {
        return (None, None, diagnostics);
    };
    let version = loaded.version;
    if version.is_pre_release() {
        diagnostics.push(
            Diagnostic::error(core_codes::PRERELEASE_FORMAT_VERSION.with([version.to_string()]))
                .severity(Severity::Info),
        );
    }
    let mut read = bedrock_addon_shared_animation::walk::read(cx, &mut diagnostics);
    let any = if version < V1_10_0 {
        v1_8_0::read_file(&text, &mut read).map(AnimationControllerHistory::V1_8_0)
    } else {
        read_file(&text, &mut read).map(AnimationControllerHistory::V1_10_0)
    };
    let respelled = read.take_respelled();
    drop(read);
    // A declared member spelled in another case is matched as vanilla matches it, with an
    // information of the crate's own.
    for (at, span) in respelled {
        let mut finding = Diagnostic::of(&core_codes::NON_CANONICAL_MEMBER_CASE).at(at);
        finding.location.span = span;
        diagnostics.push(finding);
    }
    let any = any.filter(|_| !diagnostics.has_errors());
    (any, Some(version), diagnostics)
}

/// The `format_version` member of a controller schema: as [`format_version`], except that the
/// array form, which passed the format-version gate, is `UnsupportedVersion` with the node's
/// text.
pub(crate) fn controller_format_version(
    node: &Node<'_>,
    read: &mut Read<'_>,
    minimum: &SemVersion,
) -> Option<Written> {
    if node.value().is_array() {
        // A version node can only be parsed from a JSON string.
        read.error(node, codes::UNKNOWN_OPTION_TYPE.with(["SemVersion"]));
        return None;
    }
    format_version(node, read, minimum)
}

/// Runs the 1.8 step when the file is a 1.8 file.
pub(crate) fn upgrade(
    any: AnimationControllerHistory,
    written: &SemVersion,
    cx: &LoadContext,
    diagnostics: &mut Diagnostics,
) -> Option<AnimationControllerFile> {
    match any {
        AnimationControllerHistory::V1_10_0(file) => Some(file),
        AnimationControllerHistory::V1_8_0(old) => {
            let mut found = Diagnostics::default();
            let file = old.upgrade(&mut UpgradeCx {
                load: cx,
                original_version: written.format_version(),
                identifier: "",
                actor_type: ActorType::Mob,
                skip_section_a: false,
                reading: cx.reading,
                diagnostics: &mut found,
            });
            let failed = found.has_errors();
            diagnostics.extend(found);
            (!failed).then_some(file)
        }
    }
}

/// The gates, the Molang passes and the reference stage on the newest struct; `None` when the
/// file does not load.
pub(crate) fn finish(
    mut file: AnimationControllerFile,
    written: &SemVersion,
    cx: &LoadContext,
    diagnostics: &mut Diagnostics,
) -> Option<AnimationControllerFile> {
    set_gates(&mut file, written);
    super::validate::controller_findings(&file, written, cx, diagnostics);
    let keep = run(
        &mut file,
        &MolangRun {
            load: cx,
            written: input_of(written),
            lowercase_transitions: *written < V1_17_30,
            transitions_typed: *written < V1_10_0,
        },
        0,
        diagnostics,
    );
    (keep && !diagnostics.has_errors()).then_some(file)
}

/// The two gates of a file of version `written`, on every controller (the crate sets them per file;
/// the game sets them per cached schema bucket).
pub(crate) fn set_gates(file: &mut AnimationControllerFile, written: &SemVersion) {
    for controller in file.animation_controllers.values_mut() {
        controller.lowercase_transitions = *written < V1_17_30;
        controller.legacy_event_skipping = *written < V1_18_10;
    }
}

// ---- the v1_10-or-later schema -----------------------------------------------------------------

/// The read of a whole v1.10+ file.
pub(crate) fn read_file(text: &Text<'_>, read: &mut Read<'_>) -> Option<AnimationControllerFile> {
    let Document {
        format_version: Written(version),
        animation_controllers,
    } = read_document(&Node::parsed(text, JsonPath::new()), read)?;
    Some(AnimationControllerFile {
        format_version: version.format_version(),
        animation_controllers,
    })
}

/// The one member of an object whose child count is exactly one (`transitions`, the 1.8
/// `animations`): its name, which must be an identifier, and its value read by `value`.
pub(crate) fn single<V>(
    node: &Node<'_>,
    read: &mut Read<'_>,
    value: impl FnMut(&Node<'_>, &mut Read<'_>) -> Option<V>,
) -> Option<(String, V)> {
    let n = read.object(node)?.len();
    if n != 1 {
        let code = if n < 1 {
            cereal::too_few_children(n, 1)
        } else {
            cereal::too_many_children(n, 1)
        };
        read.error(node, code);
        return None;
    }
    read_keyed(node, read, &IDENTIFIERS, value)?
        .into_iter()
        .next()
}

/// `{"<state>": "<condition>"}`.
impl FromJson for Transition {
    const EXPECTED: Option<Expected> = Some(Expected::Object);

    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let (target, condition) = single(node, read, String::from_json)?;
        Some(Self {
            target,
            condition: MolangSource::string(condition, RawVersion(-1)),
        })
    }
}

/// A short name, or short names with their weights.
impl FromJson for StateAnimation {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let value = node.value();
        if let Some(name) = value.as_string() {
            return Some(Self::Name(name.as_str().to_owned()));
        }
        if value.is_object() {
            return read_keyed(node, read, &IDENTIFIERS, Molang::from_json).map(Self::Weighted);
        }
        read.wrong_accepting(node, "string, object");
        None
    }
}

/// A number of seconds, or a curve whose keys are identifiers (not time keys).
impl FromJson for BlendTransition {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let value = node.value();
        if value.is_number() {
            return f32::from_json(node, read).map(Self::Seconds);
        }
        if value.is_object() {
            return read_keyed(node, read, &IDENTIFIERS, f32::from_json).map(Self::Curve);
        }
        read.wrong_accepting(node, "float, object");
        None
    }
}

bedrock_addon_core::impl_schema_leaf!(Transition, StateAnimation, BlendTransition);
bedrock_addon_core::impl_no_molang!(Transition, StateAnimation, BlendTransition);
