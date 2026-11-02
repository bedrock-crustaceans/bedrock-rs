//! Constructors and the inherent helpers: vanilla's defaults and its case-folded views.

use super::{Animation, AnimationFile, Bone, Channel, LoopMode};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_support::animation::walk::atof;
use indexmap::IndexMap;

impl AnimationFile {
    /// An empty file at format 1.8.0, the version vanilla's own files use.
    #[must_use]
    pub fn new() -> Self {
        Self {
            format_version: FormatVersion::new(1, 8, 0),
            animations: IndexMap::new(),
        }
    }

    /// The animation whose key is `identifier`, ignoring case (vanilla lower-cases the key).
    #[must_use]
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
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// `loop`, with its default (stop when finished).
    #[must_use]
    pub fn loop_mode(&self) -> LoopMode {
        self.r#loop.unwrap_or_default()
    }

    /// The bone named `name`, ignoring case (vanilla lower-cases bone names).
    #[must_use]
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
    #[must_use]
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
    #[must_use]
    pub fn keyframes_by_time(&self) -> Vec<(f32, &super::Keyframe)> {
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
