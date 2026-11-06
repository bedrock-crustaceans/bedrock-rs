//! The Molang members of an animation file: the walk over them.

use super::{AnimationFile, Channel, Keyframe, OneOrMany};
use bedrock_addon_core::molang::{Molang, MolangFieldSpec, MolangFields, MolangVisitor, Step};
use bedrock_addon_shared_animation::molang::{ANIMATION_EVENT, ANIMATION_VALUE, event, particle};

fn molang3(visitor: &mut MolangVisitor<'_>, spec: &MolangFieldSpec, values: &mut [Molang; 3]) {
    for (index, value) in values.iter_mut().enumerate() {
        visitor.field(index, spec, value);
    }
}

fn channel(visitor: &mut MolangVisitor<'_>, key: &str, channel: &mut Option<Channel>) {
    let spec = &ANIMATION_VALUE;
    let Some(channel) = channel else { return };
    match channel {
        Channel::Uniform(value) => visitor.field(key, spec, value),
        Channel::Vec3(values) => {
            visitor.enter(Step::Key(key));
            molang3(visitor, spec, values);
            visitor.leave();
        }
        Channel::AxisSequence(elements) => {
            visitor.enter(Step::Key(key));
            for (index, element) in elements.iter_mut().enumerate() {
                match element {
                    super::AxisElement::Positional(value) => visitor.field(index, spec, value),
                    super::AxisElement::Axes(axes) => visitor.field(index, spec, axes),
                }
            }
            visitor.leave();
        }
        Channel::Keyframes(frames) => {
            visitor.enter(Step::Key(key));
            for (time, frame) in frames.iter_mut() {
                visitor.enter(Step::Key(time));
                match frame {
                    Keyframe::Vec3(values) => molang3(visitor, spec, values),
                    Keyframe::Split(parts) => {
                        for (name, values) in [("pre", &mut parts.pre), ("post", &mut parts.post)] {
                            if let Some(values) = values {
                                visitor.enter(Step::Key(name));
                                molang3(visitor, spec, values);
                                visitor.leave();
                            }
                        }
                    }
                }
                visitor.leave();
            }
            visitor.leave();
        }
    }
}

fn timed<T>(
    visitor: &mut MolangVisitor<'_>,
    member: &str,
    map: &mut Option<indexmap::IndexMap<String, OneOrMany<T>>>,
    mut each: impl FnMut(&mut MolangVisitor<'_>, Step<'_>, &mut T),
) {
    let Some(map) = map else { return };
    visitor.enter(Step::Key(member));
    for (time, entries) in map.iter_mut() {
        match entries {
            OneOrMany::One(entry) => each(visitor, Step::Key(time), entry),
            OneOrMany::Many(list) => {
                visitor.enter(Step::Key(time));
                for (index, entry) in list.iter_mut().enumerate() {
                    each(visitor, Step::Index(index), entry);
                }
                visitor.leave();
            }
        }
    }
    visitor.leave();
}

impl MolangFields for super::Animation {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let spec = &ANIMATION_VALUE;
        visitor.field("start_delay", spec, &mut self.start_delay);
        visitor.field("loop_delay", spec, &mut self.loop_delay);
        visitor.field("anim_time_update", spec, &mut self.anim_time_update);
        visitor.field("blend_weight", spec, &mut self.blend_weight);
        if let Some(bones) = &mut self.bones {
            visitor.enter(Step::Key("bones"));
            for (name, bone) in bones.iter_mut() {
                visitor.enter(Step::Key(name));
                channel(visitor, "position", &mut bone.position);
                channel(visitor, "rotation", &mut bone.rotation);
                channel(visitor, "scale", &mut bone.scale);
                visitor.leave();
            }
            visitor.leave();
        }
        timed(
            visitor,
            "particle_effects",
            &mut self.particle_effects,
            |v, step, e| {
                v.enter(step);
                particle(v, spec, e);
                v.leave();
            },
        );
        timed(visitor, "timeline", &mut self.timeline, |v, step, e| {
            event(v, step, &ANIMATION_EVENT, e);
        });
    }
}

impl MolangFields for AnimationFile {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visitor.enter(Step::Key("animations"));
        for (key, animation) in &mut self.animations {
            visitor.nested(key.as_str(), animation);
        }
        visitor.leave();
    }
}
