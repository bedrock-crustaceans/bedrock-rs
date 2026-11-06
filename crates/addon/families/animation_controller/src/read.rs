//! Reading animation controller files: the format-version gate, the 1.10-or-later schema walked
//! over the text, the 1.8 upgrade, then the gates, the Molang passes and the reference stage.

use indexmap::IndexMap;

use super::history::{Any, v1_8_0};
use super::{
    AnimationController, AnimationControllerFile, BlendTransition, EventString, State,
    StateAnimation, StateVariable, Transition,
};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, Severity, VanillaText,
};
use bedrock_addon_core::json::JsonSlice;
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::molang::{MolangSource, RawVersion};
use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::version::{ENGINE_VERSION, SemVersion};
use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};
use bedrock_addon_shared_animation::event::command_version;
use bedrock_addon_shared_animation::molang::{MolangRun, run};
use bedrock_addon_shared_animation::read::{
    Parsed, format_version, input_of, read_particle, read_sound, text_of,
};
use bedrock_addon_shared_animation::walk::{
    Group, Hit, Kind, Node, Pattern, UNKNOWN_OPTION_TYPE, Walk, text,
};
use bedrock_addon_shared_shapes::TrueOnly;

/// The 1.8 schema's `format_version` minimum and the controller floor.
pub(crate) const V1_8_0: SemVersion = SemVersion::new(1, 8, 0);
/// The 1.10 schema version: from here no upgrade runs.
pub(crate) const V1_10_0: SemVersion = SemVersion::new(1, 10, 0);
/// Below this version transition expressions are lower-cased.
pub(crate) const V1_17_30: SemVersion = SemVersion::new(1, 17, 30);
/// From this version the initial state's `on_entry` events fire when the controller starts.
pub(crate) const V1_18_10: SemVersion = SemVersion::new(1, 18, 10);

/// The controller key pattern.
pub(crate) const CONTROLLER_KEY: Pattern = Pattern::Prefixed(
    "controller.animation.",
    r"controller\.animation\.[a-zA-Z0-9_.-]+",
);

/// `Error: ill-formatted/missing "format_version" field. Skipping.`
const ILL_FORMATTED: &str = "Error: ill-formatted/missing \"format_version\" field. Skipping.";
/// The "exceeds game version" text (two spaces before "Skipping", as the game writes it).
const EXCEEDS_GAME: &str =
    "Error: animation controller file format version: {} exceeds game version: {}.  Skipping.";

/// What the format-version gate read.
pub(crate) struct LoadedVersion {
    /// The version.
    pub(crate) version: SemVersion,
}

/// The format-version gate: the member found ignoring case, read as a version string or an array of
/// three unsigned integers; the three rejections are content-log errors that skip the file.
pub(crate) fn load_format_version(walk: &mut Walk, root: Node<'_>) -> Option<LoadedVersion> {
    let member = (root.kind() == Kind::Object)
        .then(|| walk.members(root))
        .flatten()
        .and_then(|members| {
            members
                .into_iter()
                .find(|(key, _)| key.eq_ignore_ascii_case("format_version"))
        });
    let version = member
        .as_ref()
        .and_then(|(_, node)| read_version(walk, *node));
    let Some(version) = version else {
        let code = if member.is_none() {
            DiagnosticCode::MissingFormatVersion {
                text: Some(VanillaText::Fixed(ILL_FORMATTED)),
            }
        } else {
            DiagnosticCode::UnsupportedVersion {
                text: Some(VanillaText::Fixed(ILL_FORMATTED)),
            }
        };
        walk.push(code, Severity::Error, member.map(|(_, n)| n.span()));
        return None;
    };
    let engine = SemVersion::new(
        ENGINE_VERSION.major,
        ENGINE_VERSION.minor,
        ENGINE_VERSION.patch,
    );
    if version > engine {
        walk.push(
            DiagnosticCode::UnsupportedVersion {
                text: text(EXCEEDS_GAME, &[&version.to_string(), &engine.to_string()]),
            },
            Severity::Error,
            None,
        );
        return None;
    }
    if version < V1_8_0 {
        walk.push(
            DiagnosticCode::UnsupportedVersion {
                text: Some(VanillaText::UnknownFormatVersion {
                    version: version.to_string(),
                }),
            },
            Severity::Error,
            None,
        );
        return None;
    }
    Some(LoadedVersion { version })
}

/// A version read without wildcards: a version string, or exactly three unsigned integers.
fn read_version(walk: &mut Walk, node: Node<'_>) -> Option<SemVersion> {
    match node.kind() {
        Kind::String => {
            let text =
                bedrock_addon_core::json::parse_at::<String>(JsonSlice::new(node.text())).ok()?;
            if text == "beta" {
                return None;
            }
            SemVersion::parse(&text).filter(|v| !v.is_any())
        }
        Kind::Array => {
            let mut numbers = Vec::new();
            walk_quiet(walk, node, &mut numbers);
            let [major, minor, patch] = numbers.as_slice() else {
                return None;
            };
            Some(SemVersion::new(*major, *minor, *patch))
        }
        _ => None,
    }
}

/// The unsigned integer elements of an array, without findings.
fn walk_quiet(walk: &mut Walk, node: Node<'_>, out: &mut Vec<u16>) {
    let mut scratch = Walk::new();
    let _ = walk;
    scratch.array(node, |_, _, element| {
        if element.kind() != Kind::Number {
            return false;
        }
        match element.text().parse::<u16>() {
            Ok(n) => {
                out.push(n);
                true
            }
            Err(_) => false,
        }
    });
    if out.len() != 3 || scratch.diagnostics.has_errors() {
        out.clear();
    }
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
) -> (Option<Any>, Option<SemVersion>, Diagnostics) {
    let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
    let mut walk = Walk::new();
    let text = match text_of(bytes) {
        Ok(text) => text,
        Err(finding) => {
            walk.diagnostics.push(*finding);
            return (None, None, walk.diagnostics);
        }
    };
    let Some(root) = Node::root(text) else {
        return (None, None, walk.diagnostics);
    };
    let Some(loaded) = load_format_version(&mut walk, root) else {
        return (None, None, walk.diagnostics);
    };
    let version = loaded.version;
    if version.is_pre_release() {
        walk.push(
            DiagnosticCode::PrereleaseFormatVersion {
                version: version.to_string(),
            },
            Severity::Info,
            None,
        );
    }
    let commands = command_version(cx.min_engine_version.as_ref());
    let any = if version < V1_10_0 {
        v1_8_0::read_file(&mut walk, root).map(Any::V1_8_0)
    } else {
        read_file(&mut walk, root, commands, &version).map(Any::V1_10_0)
    };
    non_canonical(&mut walk);
    let any = any.filter(|_| !walk.diagnostics.has_errors());
    (any, Some(version), walk.diagnostics)
}

/// Reports a declared member spelled in another case: it is matched as vanilla matches it,
/// with an info of the crate's own.
fn non_canonical(walk: &mut Walk) {
    for (at, span) in std::mem::take(&mut walk.non_canonical) {
        walk.diagnostics.push(
            Diagnostic::info(DiagnosticCode::NonCanonicalMemberCase)
                .at(at)
                .span(span),
        );
    }
}

/// The `format_version` member of a controller schema: as [`format_version`], except that the
/// array form, which passed the format-version gate, is `UnsupportedVersion` with the node's
/// text.
pub(crate) fn controller_format_version(
    walk: &mut Walk,
    node: Node<'_>,
    minimum: &SemVersion,
) -> Option<SemVersion> {
    if node.kind() == Kind::Array {
        // A version node can only be parsed from a JSON string.
        walk.error(
            DiagnosticCode::UnsupportedVersion {
                text: text(UNKNOWN_OPTION_TYPE, &["SemVersion"]),
            },
            node.span(),
        );
        return None;
    }
    format_version(walk, node, minimum)
}

/// Runs the 1.8 step when the file is a 1.8 file.
pub(crate) fn upgrade(
    any: Any,
    written: &SemVersion,
    cx: &LoadContext,
    diagnostics: &mut Diagnostics,
) -> Option<AnimationControllerFile> {
    match any {
        Any::V1_10_0(file) => Some(file),
        Any::V1_8_0(old) => {
            let mut found = Diagnostics::default();
            let file = old.upgrade(&mut UpgradeCx {
                load: cx,
                original_version: written.format_version(),
                identifier: "",
                actor_type: ActorType::Mob,
                skip_section_a: false,
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

/// The walk of a whole v1.10+ file.
pub(crate) fn read_file(
    walk: &mut Walk,
    root: Node<'_>,
    commands: u8,
    written: &SemVersion,
) -> Option<AnimationControllerFile> {
    if root.kind() != Kind::Object {
        walk.bad_type("object", root);
        return None;
    }
    let mut version = None;
    let mut controllers = None;
    let groups = [
        Group::required("format_version"),
        Group::required("animation_controllers"),
    ];
    let ok = walk.object(root, &groups, |walk, group, _, node| match group {
        0 => controller_format_version(walk, node, &V1_10_0)
            .map(|v| version = Some(v))
            .into(),
        _ => read_controllers(walk, node, |walk, node| read_state(walk, node, commands))
            .map(|c| controllers = Some(c))
            .into(),
    });
    let _ = written;
    let file = AnimationControllerFile {
        format_version: version?.format_version(),
        animation_controllers: latest_controllers(controllers?),
    };
    ok.then_some(file)
}

/// Controllers by key: `initial_state` and the states.
pub(crate) type Controllers<S> = IndexMap<String, (Option<String>, IndexMap<String, S>)>;

/// `animation_controllers`: controllers by key, each with `states` and `initial_state`; the
/// state shape is the version's.
pub(crate) fn read_controllers<S>(
    walk: &mut Walk,
    node: Node<'_>,
    mut state: impl FnMut(&mut Walk, Node<'_>) -> Option<S>,
) -> Option<Controllers<S>> {
    if node.kind() != Kind::Object {
        walk.bad_type("object", node);
        return None;
    }
    let mut map = IndexMap::new();
    let ok = walk.object(
        node,
        &[Group::pattern(CONTROLLER_KEY, true)],
        |walk, _, key, v| {
            if v.kind() != Kind::Object {
                walk.bad_type("object", v);
                return Hit::Failed;
            }
            let mut initial = None;
            let mut states = None;
            let groups = [Group::required("states"), Group::name("initial_state")];
            let ok = walk.object(v, &groups, |walk, group, _, v| match group {
                0 => {
                    if v.kind() != Kind::Object {
                        walk.bad_type("object", v);
                        return Hit::Failed;
                    }
                    let mut map = IndexMap::new();
                    let ok = walk.object(
                        v,
                        &[Group::pattern(Pattern::Identifier, true)],
                        |walk, _, name, v| {
                            if v.kind() != Kind::Object {
                                walk.bad_type("object", v);
                                return Hit::Failed;
                            }
                            state(walk, v)
                                .map(|s| map.insert(name.to_owned(), s))
                                .into()
                        },
                    );
                    if ok {
                        states = Some(map);
                    }
                    ok.into()
                }
                _ => walk.string(v).map(|s| initial = Some(s)).into(),
            });
            match (ok, states) {
                (true, Some(states)) => {
                    map.insert(key.to_owned(), (initial, states));
                    Hit::Ok
                }
                _ => Hit::Failed,
            }
        },
    );
    ok.then_some(map)
}

fn latest_controllers(
    controllers: IndexMap<String, (Option<String>, IndexMap<String, State>)>,
) -> IndexMap<String, AnimationController> {
    controllers
        .into_iter()
        .map(|(key, (initial_state, states))| {
            (
                key,
                AnimationController {
                    initial_state,
                    states,
                    lowercase_transitions: false,
                    legacy_event_skipping: false,
                },
            )
        })
        .collect()
}

const STATE_MEMBERS: [Group; 9] = [
    Group::name("variables"),
    Group::name("animations"),
    Group::name("transitions"),
    Group::name("blend_transition"),
    Group::name("blend_via_shortest_path"),
    Group::name("particle_effects"),
    Group::name("sound_effects"),
    Group::name("on_entry"),
    Group::name("on_exit"),
];

fn read_state(walk: &mut Walk, node: Node<'_>, commands: u8) -> Option<State> {
    let mut state = State::default();
    let ok = walk.object(node, &STATE_MEMBERS, |walk, group, _, v| match group {
        0 => read_variables(walk, v)
            .map(|x| state.variables = Some(x))
            .into(),
        1 => read_state_animations(walk, v)
            .map(|x| state.animations = Some(x))
            .into(),
        2 => read_transitions(walk, v)
            .map(|x| state.transitions = Some(x))
            .into(),
        3 => read_blend_transition(walk, v)
            .map(|x| state.blend_transition = Some(x))
            .into(),
        4 => match walk.bool(v) {
            Some(true) => {
                state.blend_via_shortest_path = Some(TrueOnly);
                Hit::Ok
            }
            Some(false) => {
                // The allowed-value set `{true}`.
                walk.not_allowed("false", v);
                Hit::Failed
            }
            None => Hit::Failed,
        },
        5 => read_objects(walk, v, read_particle)
            .map(|x| state.particle_effects = Some(x))
            .into(),
        6 => read_objects(walk, v, read_sound)
            .map(|x| state.sound_effects = Some(x))
            .into(),
        7 => read_events(walk, v, commands)
            .map(|x| state.on_entry = Some(x))
            .into(),
        _ => read_events(walk, v, commands)
            .map(|x| state.on_exit = Some(x))
            .into(),
    });
    ok.then_some(state)
}

fn read_variables(walk: &mut Walk, node: Node<'_>) -> Option<IndexMap<String, StateVariable>> {
    if node.kind() != Kind::Object {
        walk.bad_type("object", node);
        return None;
    }
    let mut map = IndexMap::new();
    let ok = walk.object(
        node,
        &[Group::pattern(Pattern::Identifier, true)],
        |walk, _, name, v| {
            if v.kind() != Kind::Object {
                walk.bad_type("object", v);
                return Hit::Failed;
            }
            let mut input = None;
            let mut remap_curve = None;
            let groups = [Group::required("input"), Group::name("remap_curve")];
            let ok = walk.object(v, &groups, |walk, group, _, v| {
                if group == 0 {
                    return walk.molang(v).map(|m| input = Some(m)).into();
                }
                if v.kind() != Kind::Object {
                    walk.bad_type("object", v);
                    return Hit::Failed;
                }
                walk.float_map(v, Pattern::Time, true)
                    .map(|c| remap_curve = Some(c))
                    .into()
            });
            match (ok, input) {
                (true, Some(input)) => {
                    map.insert(name.to_owned(), StateVariable { input, remap_curve });
                    Hit::Ok
                }
                _ => Hit::Failed,
            }
        },
    );
    ok.then_some(map)
}

fn read_state_animations(walk: &mut Walk, node: Node<'_>) -> Option<Vec<StateAnimation>> {
    let mut list = Vec::new();
    let ok = walk.array(node, |walk, _, element| match element.kind() {
        Kind::String => walk
            .string(element)
            .map(|s| list.push(StateAnimation::Name(s)))
            .is_some(),
        Kind::Object => {
            let mut weights = IndexMap::new();
            let ok = walk.object(
                element,
                &[Group::pattern(Pattern::Identifier, true)],
                |walk, _, name, v| {
                    walk.molang(v)
                        .map(|m| weights.insert(name.to_owned(), m))
                        .into()
                },
            );
            if ok {
                list.push(StateAnimation::Weighted(weights));
            }
            ok
        }
        _ => {
            walk.bad_type("string, object", element);
            false
        }
    });
    ok.then_some(list)
}

fn read_transitions(walk: &mut Walk, node: Node<'_>) -> Option<Vec<Transition>> {
    let mut list = Vec::new();
    let ok = walk.array(node, |walk, _, element| {
        if element.kind() != Kind::Object {
            walk.bad_type("object", element);
            return false;
        }
        if !walk.child_count(element, 1) {
            return false;
        }
        walk.object(
            element,
            &[Group::pattern(Pattern::Identifier, true)],
            |walk, _, target, v| {
                walk.string(v)
                    .map(|condition| {
                        list.push(Transition {
                            target: target.to_owned(),
                            condition: MolangSource::string(condition, RawVersion(-1)),
                        });
                    })
                    .into()
            },
        )
    });
    ok.then_some(list)
}

fn read_blend_transition(walk: &mut Walk, node: Node<'_>) -> Option<BlendTransition> {
    match node.kind() {
        Kind::Number => walk.float(node).map(BlendTransition::Seconds),
        // The object option's keys are `[a-zA-Z0-9_.-]+`, not time keys.
        Kind::Object => walk
            .float_map(node, Pattern::Identifier, true)
            .map(BlendTransition::Curve),
        _ => {
            walk.bad_type("float, object", node);
            None
        }
    }
}

/// An array of objects read by `each`.
pub(crate) fn read_objects<T>(
    walk: &mut Walk,
    node: Node<'_>,
    mut each: impl FnMut(&mut Walk, Node<'_>) -> Option<T>,
) -> Option<Vec<T>> {
    let mut list = Vec::new();
    let ok = walk.array(node, |walk, _, element| {
        if element.kind() != Kind::Object {
            walk.bad_type("object", element);
            return false;
        }
        each(walk, element).map(|x| list.push(x)).is_some()
    });
    ok.then_some(list)
}

fn read_events(walk: &mut Walk, node: Node<'_>, commands: u8) -> Option<Vec<EventString>> {
    walk.strings(node).map(|list| {
        list.iter()
            .map(|s| EventString::classify(s, commands))
            .collect()
    })
}
