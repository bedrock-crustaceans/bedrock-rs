//! The family's types and their constructors: vanilla's defaults and its case-folded views.

use indexmap::IndexMap;

use bedrock_addon_core::molang::Molang;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::known;
use bedrock_addon_support::animation::walk::atof;

use super::{EventString, ParticleEffect, SoundEffect};

/// An animation file (`animations/*.json`): `{"format_version", "animations": {"animation.<name>":
/// {…}}}`.
///
/// One schema serves every version from 1.8.0 up: the file's `format_version` must be a version
/// string of at least 1.8.0 and selects nothing else. Any member the schema does not declare, a
/// key that does not match its pattern or a value of the wrong JSON type drops the whole file,
/// as vanilla does; member names match ignoring case.
#[derive(Clone, Debug, PartialEq)]
#[doc(alias = "animations")]
pub struct AnimationFile {
    /// `format_version`: the version the file was written in (at least 1.8.0).
    pub format_version: FormatVersion,
    /// `animations`: each animation by its key as written (`animation.<name>`); the registry
    /// keys it lower-cased.
    pub animations: IndexMap<String, Animation>,
}

/// One animation (`animations."animation.<name>"`): bone channels, effects and timeline events
/// over time.
///
/// Every member is optional; the accessors give vanilla's defaults.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Animation {
    /// `loop`: stop, loop or hold on the last frame when finished (JSON `true`, `false` or
    /// `"hold_on_last_frame"`); absent stops.
    pub r#loop: Option<LoopMode>,
    /// `start_delay`: seconds to wait before playing; absent 0.
    pub start_delay: Option<Molang>,
    /// `loop_delay`: seconds to wait before each loop; absent 0.
    pub loop_delay: Option<Molang>,
    /// `anim_time_update`: how time advances; absent `query.anim_time + query.delta_time` at
    /// Molang version 13.
    pub anim_time_update: Option<Molang>,
    /// `blend_weight`: the weight the animation is blended with; absent 1.
    pub blend_weight: Option<Molang>,
    /// `override_previous_animation`: reset the bones to the default pose first; absent false.
    pub override_previous_animation: Option<bool>,
    /// `animation_length`: the length in seconds; absent, the last keyframe or timeline time.
    pub animation_length: Option<f32>,
    /// `bones`: each animated bone by its name as written (vanilla lower-cases it).
    pub bones: Option<IndexMap<String, Bone>>,
    /// `particle_effects`: particle events by their time key (`"0.5"`).
    pub particle_effects: Option<IndexMap<String, OneOrMany<ParticleEffect>>>,
    /// `sound_effects`: sound events by their time key.
    pub sound_effects: Option<IndexMap<String, OneOrMany<SoundEffect>>>,
    /// `timeline`: event strings by their time key (commands, entity events, Molang).
    pub timeline: Option<IndexMap<String, OneOrMany<EventString>>>,
}

/// One value, or an array of them, under one key (`particle_effects`, `sound_effects`,
/// `timeline`): shapes'.
pub use bedrock_addon_shared_shapes::OneOrMany;

/// What an animation does when it reaches its end.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum LoopMode {
    /// Stop applying the animation (JSON `false`, and the default).
    #[default]
    Stop,
    /// Start again (JSON `true`).
    Loop,
    /// Keep the last frame (JSON `"hold_on_last_frame"`).
    HoldOnLastFrame,
}

/// One animated bone (`bones.<name>`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Bone {
    /// `relative_to`: what the bone's rotation is relative to.
    pub relative_to: Option<RelativeTo>,
    /// `position`: the position channel.
    pub position: Option<Channel>,
    /// `rotation`: the rotation channel, in degrees.
    pub rotation: Option<Channel>,
    /// `scale`: the scale channel.
    pub scale: Option<Channel>,
}

/// A bone's `relative_to` object.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RelativeTo {
    /// `rotation`: `"entity"` (any case) makes the rotation relative to the entity; absent, to the
    /// parent bone. Kept as written.
    pub rotation: Option<String>,
}

/// A bone channel (`position`, `rotation`, `scale`) in one of vanilla's four shapes.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Channel {
    /// A single value for all three axes, one keyframe at time 0 (`"scale": 0.7`).
    Uniform(Molang),
    /// Three values, one keyframe at time 0 (`[x, y, z]`).
    Vec3([Molang; 3]),
    /// Rotations about single axes applied in order, one keyframe at time 0
    /// (`[{"y": …}, {"x": …}]`, `rotation` only); a three-element array may mix positional
    /// values and axis objects.
    AxisSequence(Vec<AxisElement>),
    /// Keyframes by their time key (`{"0.0": …, "0.5": …}`).
    Keyframes(IndexMap<String, Keyframe>),
}

/// One element of a [`Channel::AxisSequence`].
#[derive(Clone, Debug, PartialEq)]
pub enum AxisElement {
    /// A value at its array index (0 = x, 1 = y, 2 = z).
    Positional(Molang),
    /// Rotations about `x`, `y` or `z` (keys as written, any case).
    Axes(IndexMap<String, Molang>),
}

/// One keyframe of a [`Channel::Keyframes`] channel.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Keyframe {
    /// `[x, y, z]`, used before and after the keyframe.
    Vec3([Molang; 3]),
    /// `{"lerp_mode"?, "pre"?, "post"?}`.
    Split(KeyframeParts),
}

/// The object form of a [`Keyframe`].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct KeyframeParts {
    /// `lerp_mode`: how the curve leaves this keyframe; absent linear.
    pub lerp_mode: Option<LerpMode>,
    /// `pre`: the value approaching the keyframe.
    pub pre: Option<[Molang; 3]>,
    /// `post`: the value leaving the keyframe.
    pub post: Option<[Molang; 3]>,
}

/// A keyframe's interpolation (JSON matched ignoring case).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum LerpMode {
    /// `"linear"`.
    #[default]
    Linear,
    /// `"catmullrom"`.
    CatmullRom,
}

impl AnimationFile {
    /// An empty file at format 1.8.0, the version vanilla's own files use.
    pub fn new() -> Self {
        Self {
            format_version: known::V1_8_0,
            animations: IndexMap::new(),
        }
    }

    /// The animation whose key is `identifier`, ignoring case (vanilla lower-cases the key).
    pub fn get(&self, identifier: &str) -> Option<&Animation> {
        self.animations
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(identifier))
            .map(|(_, a)| a)
    }
}

impl Default for AnimationFile {
    fn default() -> Self {
        Self::new()
    }
}

impl Animation {
    /// An animation with every member absent.
    pub fn new() -> Self {
        Self::default()
    }

    /// `loop`, with its default (stop when finished).
    pub fn loop_mode(&self) -> LoopMode {
        self.r#loop.unwrap_or_default()
    }

    /// The bone named `name`, ignoring case (vanilla lower-cases bone names).
    pub fn bone(&self, name: &str) -> Option<&Bone> {
        self.bones
            .as_ref()?
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, b)| b)
    }

    /// The length in seconds: `animation_length` when written, else vanilla's computed length,
    /// the last keyframe time of every channel and every `timeline` time (particle and sound
    /// times do not count).
    pub fn length(&self) -> f32 {
        if let Some(length) = self.animation_length {
            return length;
        }
        let mut length: f32 = 0.0;
        for bone in self.bones.iter().flat_map(IndexMap::values) {
            for channel in [&bone.position, &bone.rotation, &bone.scale]
                .into_iter()
                .flatten()
            {
                if let Channel::Keyframes(frames) = channel {
                    for key in frames.keys() {
                        length = length.max(atof(key));
                    }
                }
            }
        }
        for key in self.timeline.iter().flat_map(IndexMap::keys) {
            length = length.max(atof(key));
        }
        length
    }
}

impl Bone {
    /// Whether the rotation is relative to the entity (`relative_to.rotation` is `"entity"` in
    /// any case) rather than to the parent bone.
    pub fn rotation_relative_to_entity(&self) -> bool {
        self.relative_to
            .as_ref()
            .and_then(|r| r.rotation.as_deref())
            .is_some_and(|r| r.eq_ignore_ascii_case("entity"))
    }
}

impl Channel {
    /// The keyframes of a keyframe channel in vanilla's order: by time, keys with equal times
    /// in member order (a stable sort). Other shapes have one keyframe at time 0 and give
    /// an empty list.
    pub fn keyframes_by_time(&self) -> Vec<(f32, &Keyframe)> {
        let Self::Keyframes(frames) = self else {
            return Vec::new();
        };
        let mut list: Vec<_> = frames
            .iter()
            .map(|(key, frame)| (atof(key), frame))
            .collect();
        list.sort_by(|a, b| a.0.total_cmp(&b.0));
        list
    }
}
