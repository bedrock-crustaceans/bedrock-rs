//! Reading animation files: the `actor_animation` schema (the only one) walked over the text,
//! then the Molang passes and the crate's own findings.

use indexmap::IndexMap;

use super::{
    Animation, AnimationFile, AxisElement, Bone, Channel, EventString, Keyframe, KeyframeParts,
    LerpMode, LoopMode, OneOrMany, RelativeTo,
};
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, JsonPath};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::PackType;
use bedrock_addon_core::version::{ENGINE_VERSION, SemVersion};
use bedrock_addon_support::animation::event::command_version;
use bedrock_addon_support::animation::molang::{MolangRun, run};
use bedrock_addon_support::animation::read::{
    Parsed, format_version, input_of, read_particle, read_sound, text_of,
};
use bedrock_addon_support::animation::walk::{
    Group, Hit, Kind, NOT_ALLOWED_KEYS, NOT_ALLOWED_REGEX, Node, Pattern, Walk, text,
};

/// The animation schema's `format_version` minimum.
pub(crate) const ANIMATION_MINIMUM: SemVersion = SemVersion::new(1, 8, 0);

/// The animation key pattern.
pub(crate) const ANIMATION_KEY: Pattern =
    Pattern::Prefixed("animation.", r"animation\.[a-zA-Z0-9_.-]+");

/// Reads one animation file (`animations/**/*.json`) of a pack with context `cx`.
pub(crate) fn parse_file(bytes: &[u8], cx: &LoadContext) -> Parsed<AnimationFile> {
    let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
    let mut walk = Walk::new();
    let text = match text_of(bytes) {
        Ok(text) => text,
        Err(finding) => {
            walk.diagnostics.push(*finding);
            return Parsed {
                value: None,
                written: None,
                diagnostics: walk.diagnostics,
            };
        }
    };
    let commands = command_version(cx.min_engine_version.as_ref());
    let read = Node::root(text).and_then(|root| read_file(&mut walk, root, commands));
    let mut diagnostics = walk.diagnostics;
    let Some((mut file, written)) = read else {
        return Parsed {
            value: None,
            written: None,
            diagnostics,
        };
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
            DiagnosticCode::FormatVersionAboveEngine {
                version: written.to_string(),
            }
        } else {
            // The game loads the file silently, so this finding is the crate's own.
            DiagnosticCode::UnsupportedVersion { text: None }
        };
        diagnostics.push(Diagnostic::info(code).at(at));
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

/// The schema walk of a whole file.
pub(crate) fn read_file(
    walk: &mut Walk,
    root: Node<'_>,
    commands: u8,
) -> Option<(AnimationFile, SemVersion)> {
    if root.kind() != Kind::Object {
        walk.bad_type("object", root);
        return None;
    }
    let mut written = None;
    let mut animations = None;
    let groups = [
        Group::required("format_version"),
        Group::required("animations"),
    ];
    let ok = walk.object(root, &groups, |walk, group, _, node| match group {
        0 => format_version(walk, node, &ANIMATION_MINIMUM)
            .map(|v| written = Some(v))
            .into(),
        _ => read_animations(walk, node, commands)
            .map(|map| animations = Some(map))
            .into(),
    });
    let written = written?;
    let file = AnimationFile {
        format_version: written.format_version(),
        animations: animations?,
    };
    ok.then_some((file, written))
}

fn read_animations(
    walk: &mut Walk,
    node: Node<'_>,
    commands: u8,
) -> Option<IndexMap<String, Animation>> {
    if node.kind() != Kind::Object {
        walk.bad_type("object", node);
        return None;
    }
    let mut map = IndexMap::new();
    let ok = walk.object(
        node,
        &[Group::pattern(ANIMATION_KEY, true)],
        |walk, _, key, v| {
            read_animation(walk, v, commands)
                .map(|a| map.insert(key.to_owned(), a))
                .into()
        },
    );
    ok.then_some(map)
}

const ANIMATION_MEMBERS: [Group; 11] = [
    Group::name("loop"),
    Group::name("start_delay"),
    Group::name("loop_delay"),
    Group::name("anim_time_update"),
    Group::name("blend_weight"),
    Group::name("override_previous_animation"),
    Group::name("bones"),
    Group::name("particle_effects"),
    Group::name("sound_effects"),
    Group::name("timeline"),
    Group::name("animation_length"),
];

fn read_animation(walk: &mut Walk, node: Node<'_>, commands: u8) -> Option<Animation> {
    if node.kind() != Kind::Object {
        walk.bad_type("object", node);
        return None;
    }
    let mut a = Animation::default();
    let ok = walk.object(node, &ANIMATION_MEMBERS, |walk, group, _, v| match group {
        0 => read_loop(walk, v).map(|m| a.r#loop = Some(m)).into(),
        1 => walk.molang(v).map(|m| a.start_delay = Some(m)).into(),
        2 => walk.molang(v).map(|m| a.loop_delay = Some(m)).into(),
        3 => walk.molang(v).map(|m| a.anim_time_update = Some(m)).into(),
        4 => walk.molang(v).map(|m| a.blend_weight = Some(m)).into(),
        5 => walk
            .bool(v)
            .map(|b| a.override_previous_animation = Some(b))
            .into(),
        6 => read_bones(walk, v).map(|b| a.bones = Some(b)).into(),
        7 => read_timed(walk, v, read_particle)
            .map(|p| a.particle_effects = Some(p))
            .into(),
        8 => read_timed(walk, v, read_sound)
            .map(|s| a.sound_effects = Some(s))
            .into(),
        9 => read_timeline(walk, v, commands)
            .map(|t| a.timeline = Some(t))
            .into(),
        _ => walk.float(v).map(|x| a.animation_length = Some(x)).into(),
    });
    ok.then_some(a)
}

fn read_loop(walk: &mut Walk, node: Node<'_>) -> Option<LoopMode> {
    match node.kind() {
        Kind::Bool => walk
            .bool(node)
            .map(|b| if b { LoopMode::Loop } else { LoopMode::Stop }),
        Kind::String => {
            let value = walk.string(node)?;
            // The value `hold_on_last_frame` is matched case-sensitively.
            if value == "hold_on_last_frame" {
                Some(LoopMode::HoldOnLastFrame)
            } else {
                walk.error(
                    DiagnosticCode::InvalidValue {
                        text: text(NOT_ALLOWED_REGEX, &[&value, "hold_on_last_frame"]),
                    },
                    node.span(),
                );
                None
            }
        }
        _ => {
            walk.bad_type("bool, string", node);
            None
        }
    }
}

fn read_bones(walk: &mut Walk, node: Node<'_>) -> Option<IndexMap<String, Bone>> {
    if node.kind() != Kind::Object {
        walk.bad_type("object", node);
        return None;
    }
    let mut bones = IndexMap::new();
    let ok = walk.object(
        node,
        &[Group::pattern(Pattern::Identifier, true)],
        |walk, _, key, v| {
            read_bone(walk, v)
                .map(|b| bones.insert(key.to_owned(), b))
                .into()
        },
    );
    ok.then_some(bones)
}

const BONE_MEMBERS: [Group; 4] = [
    Group::name("relative_to"),
    Group::name("position"),
    Group::name("rotation"),
    Group::name("scale"),
];

fn read_bone(walk: &mut Walk, node: Node<'_>) -> Option<Bone> {
    if node.kind() != Kind::Object {
        walk.bad_type("object", node);
        return None;
    }
    let mut bone = Bone::default();
    let ok = walk.object(node, &BONE_MEMBERS, |walk, group, _, v| match group {
        0 => read_relative_to(walk, v)
            .map(|r| bone.relative_to = Some(r))
            .into(),
        1 => channel_hit(read_channel(walk, v, false), &mut bone.position),
        2 => channel_hit(read_channel(walk, v, true), &mut bone.rotation),
        _ => channel_hit(read_channel(walk, v, false), &mut bone.scale),
    });
    ok.then_some(bone)
}

fn channel_hit(read: Result<Option<Channel>, ChannelRefused>, slot: &mut Option<Channel>) -> Hit {
    match read {
        Ok(Some(channel)) => {
            *slot = Some(channel);
            Hit::Ok
        }
        Ok(None) => Hit::Failed,
        Err(ChannelRefused) => Hit::NoOption,
    }
}

fn read_relative_to(walk: &mut Walk, node: Node<'_>) -> Option<RelativeTo> {
    if node.kind() != Kind::Object {
        walk.bad_type("object", node);
        return None;
    }
    let mut relative = RelativeTo::default();
    let ok = walk.object(node, &[Group::name("rotation")], |walk, _, _, v| {
        let Some(value) = walk.string(v) else {
            return Hit::Failed;
        };
        // The node lower-cases the value, then matches `entity` ignoring case.
        let lower = value.to_ascii_lowercase();
        if lower == "entity" {
            relative.rotation = Some(value);
            Hit::Ok
        } else {
            walk.error(
                DiagnosticCode::InvalidValue {
                    text: text(NOT_ALLOWED_REGEX, &[&lower, "entity"]),
                },
                v.span(),
            );
            Hit::Failed
        }
    });
    ok.then_some(relative)
}

/// The array option's predicate refused a channel's array.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("the array is not accepted as a channel")]
struct ChannelRefused;

/// A bone channel. `Err` when the array option's predicate refuses the array (vanilla moves on
/// and ends with "child '%s' not valid here.").
fn read_channel(
    walk: &mut Walk,
    node: Node<'_>,
    rotation: bool,
) -> Result<Option<Channel>, ChannelRefused> {
    match node.kind() {
        Kind::Number | Kind::Bool | Kind::String => Ok(walk.molang(node).map(Channel::Uniform)),
        Kind::Array => {
            let kinds = walk.kinds(node);
            // The array option's predicate: not empty; three elements, or every element
            // an object.
            let accepted = match kinds.len() {
                0 => false,
                3 => true,
                _ => kinds.iter().all(|k| *k == Kind::Object),
            };
            if !accepted {
                return Err(ChannelRefused);
            }
            let scalars = kinds
                .iter()
                .all(|k| matches!(k, Kind::Number | Kind::Bool | Kind::String));
            let mut elements = Vec::new();
            let ok = walk.array(node, |walk, _, element| match element.kind() {
                Kind::Number | Kind::Bool | Kind::String => walk
                    .molang(element)
                    .map(|m| elements.push(AxisElement::Positional(m)))
                    .is_some(),
                Kind::Object if rotation => read_axes(walk, element)
                    .map(|axes| elements.push(AxisElement::Axes(axes)))
                    .is_some(),
                _ => {
                    walk.bad_type("ExpressionNode", element);
                    false
                }
            });
            if !ok {
                return Ok(None);
            }
            if scalars {
                let values: Vec<_> = elements
                    .into_iter()
                    .filter_map(|e| match e {
                        AxisElement::Positional(m) => Some(m),
                        AxisElement::Axes(_) => None,
                    })
                    .collect();
                Ok(values.try_into().ok().map(Channel::Vec3))
            } else {
                Ok(Some(Channel::AxisSequence(elements)))
            }
        }
        Kind::Object => Ok(read_keyframes(walk, node).map(Channel::Keyframes)),
        Kind::Null => {
            walk.bad_type("ExpressionNode, array, object", node);
            Ok(None)
        }
    }
}

fn read_axes(
    walk: &mut Walk,
    node: Node<'_>,
) -> Option<IndexMap<String, bedrock_addon_core::molang::Molang>> {
    let mut axes = IndexMap::new();
    let ok = walk.object(
        node,
        &[Group::pattern(Pattern::Axis, true)],
        |walk, _, key, v| {
            walk.molang(v)
                .map(|m| axes.insert(key.to_owned(), m))
                .into()
        },
    );
    ok.then_some(axes)
}

fn read_keyframes(walk: &mut Walk, node: Node<'_>) -> Option<IndexMap<String, Keyframe>> {
    let mut frames = IndexMap::new();
    let ok = walk.object(
        node,
        &[Group::pattern(Pattern::Time, true)],
        |walk, _, key, v| {
            let frame = match v.kind() {
                Kind::Array => walk.molang3(v).map(Keyframe::Vec3),
                Kind::Object => read_keyframe_parts(walk, v).map(Keyframe::Split),
                _ => {
                    walk.bad_type("array, object", v);
                    None
                }
            };
            frame.map(|f| frames.insert(key.to_owned(), f)).into()
        },
    );
    ok.then_some(frames)
}

const KEYFRAME_MEMBERS: [Group; 3] = [
    Group::name("lerp_mode"),
    Group::name("pre"),
    Group::name("post"),
];

fn read_keyframe_parts(walk: &mut Walk, node: Node<'_>) -> Option<KeyframeParts> {
    let mut parts = KeyframeParts::default();
    let ok = walk.object(node, &KEYFRAME_MEMBERS, |walk, group, _, v| match group {
        0 => read_lerp_mode(walk, v)
            .map(|m| parts.lerp_mode = Some(m))
            .into(),
        1 => walk.molang3(v).map(|m| parts.pre = Some(m)).into(),
        _ => walk.molang3(v).map(|m| parts.post = Some(m)).into(),
    });
    ok.then_some(parts)
}

fn read_lerp_mode(walk: &mut Walk, node: Node<'_>) -> Option<LerpMode> {
    let value = walk.string(node)?;
    // The enum value is lower-cased before it is matched.
    match value.to_ascii_lowercase().as_str() {
        "linear" => Some(LerpMode::Linear),
        "catmullrom" => Some(LerpMode::CatmullRom),
        _ => {
            walk.error(
                DiagnosticCode::InvalidValue {
                    text: text(NOT_ALLOWED_KEYS, &[&value, "linear, catmullrom"]),
                },
                node.span(),
            );
            None
        }
    }
}

/// `"<time>": T | [T]` members (`particle_effects`, `sound_effects`).
fn read_timed<T>(
    walk: &mut Walk,
    node: Node<'_>,
    mut each: impl FnMut(&mut Walk, Node<'_>) -> Option<T>,
) -> Option<IndexMap<String, OneOrMany<T>>> {
    if node.kind() != Kind::Object {
        walk.bad_type("object", node);
        return None;
    }
    let mut map = IndexMap::new();
    let ok = walk.object(
        node,
        &[Group::pattern(Pattern::Time, false)],
        |walk, _, key, v| {
            let entry = match v.kind() {
                Kind::Object => each(walk, v).map(OneOrMany::One),
                Kind::Array => {
                    let mut list = Vec::new();
                    let ok = walk.array(v, |walk, _, element| {
                        if element.kind() != Kind::Object {
                            walk.bad_type("object", element);
                            return false;
                        }
                        each(walk, element).map(|e| list.push(e)).is_some()
                    });
                    ok.then_some(OneOrMany::Many(list))
                }
                _ => {
                    walk.bad_type("object, array", v);
                    None
                }
            };
            entry.map(|e| map.insert(key.to_owned(), e)).into()
        },
    );
    ok.then_some(map)
}

fn read_timeline(
    walk: &mut Walk,
    node: Node<'_>,
    commands: u8,
) -> Option<IndexMap<String, OneOrMany<EventString>>> {
    if node.kind() != Kind::Object {
        walk.bad_type("object", node);
        return None;
    }
    let mut map = IndexMap::new();
    let ok = walk.object(
        node,
        &[Group::pattern(Pattern::Time, false)],
        |walk, _, key, v| {
            let entry = match v.kind() {
                Kind::String => walk
                    .string(v)
                    .map(|s| OneOrMany::One(EventString::classify(&s, commands))),
                Kind::Array => walk.strings(v).map(|list| {
                    OneOrMany::Many(
                        list.iter()
                            .map(|s| EventString::classify(s, commands))
                            .collect(),
                    )
                }),
                _ => {
                    walk.bad_type("string, array", v);
                    None
                }
            };
            entry.map(|e| map.insert(key.to_owned(), e)).into()
        },
    );
    ok.then_some(map)
}

#[cfg(test)]
mod tests {
    use super::ChannelRefused;

    #[test]
    fn a_refused_channel_is_an_error_with_a_lowercase_message() {
        let error: &dyn std::error::Error = &ChannelRefused;
        let text = error.to_string();
        assert!(text.starts_with(|c: char| c.is_lowercase()) && !text.ends_with('.'));
    }
}
