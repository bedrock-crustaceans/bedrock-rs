//! Writing an animation file back: members in declaration order, only those the value holds, each
//! in the form it was read.

use facet_value::VArray;

use super::{Animation, AnimationFile, AxisElement, Bone, Channel, Keyframe, LoopMode, OneOrMany};
use bedrock_addon_core::json::{EncodeError, Value};
use bedrock_addon_core::molang::Molang;
use bedrock_addon_shared_animation::write::{
    Object, event, float, list, map, molang, particle, sound, string, version,
};

fn molang3(values: &[Molang; 3]) -> Result<Value, EncodeError> {
    values
        .iter()
        .map(molang)
        .collect::<Result<VArray, _>>()
        .map(Value::from)
}

fn one_or_many<T>(
    value: &OneOrMany<T>,
    mut write: impl FnMut(&T) -> Result<Value, EncodeError>,
) -> Result<Value, EncodeError> {
    match value {
        OneOrMany::One(one) => write(one),
        OneOrMany::Many(many) => list(many, write),
    }
}

fn channel(c: &Channel) -> Result<Value, EncodeError> {
    match c {
        Channel::Uniform(value) => molang(value),
        Channel::Vec3(values) => molang3(values),
        Channel::AxisSequence(elements) => list(elements, |e| match e {
            AxisElement::Positional(value) => molang(value),
            AxisElement::Axes(axes) => map(axes, molang),
        }),
        Channel::Keyframes(frames) => map(frames, |frame| match frame {
            Keyframe::Vec3(values) => molang3(values),
            Keyframe::Split(parts) => {
                let mut o = Object::new();
                o.opt("lerp_mode", parts.lerp_mode.as_ref(), |m| {
                    Ok(string(match m {
                        super::LerpMode::Linear => "linear",
                        super::LerpMode::CatmullRom => "catmullrom",
                    }))
                })?;
                o.opt("pre", parts.pre.as_ref(), molang3)?;
                o.opt("post", parts.post.as_ref(), molang3)?;
                Ok(o.done())
            }
        }),
    }
}

fn bone(b: &Bone) -> Result<Value, EncodeError> {
    let mut o = Object::new();
    o.opt("relative_to", b.relative_to.as_ref(), |r| {
        let mut o = Object::new();
        o.opt("rotation", r.rotation.as_ref(), |s| Ok(string(s)))?;
        Ok(o.done())
    })?;
    o.opt("position", b.position.as_ref(), channel)?;
    o.opt("rotation", b.rotation.as_ref(), channel)?;
    o.opt("scale", b.scale.as_ref(), channel)?;
    Ok(o.done())
}

fn animation(a: &Animation) -> Result<Value, EncodeError> {
    let mut o = Object::new();
    o.opt("loop", a.r#loop.as_ref(), |m| {
        Ok(match m {
            LoopMode::Stop => Value::from(false),
            LoopMode::Loop => Value::from(true),
            LoopMode::HoldOnLastFrame => string("hold_on_last_frame"),
        })
    })?;
    o.opt("start_delay", a.start_delay.as_ref(), molang)?;
    o.opt("loop_delay", a.loop_delay.as_ref(), molang)?;
    o.opt("anim_time_update", a.anim_time_update.as_ref(), molang)?;
    o.opt("blend_weight", a.blend_weight.as_ref(), molang)?;
    o.opt(
        "override_previous_animation",
        a.override_previous_animation.as_ref(),
        |b| Ok(Value::from(*b)),
    )?;
    o.opt("bones", a.bones.as_ref(), |bones| map(bones, bone))?;
    o.opt("particle_effects", a.particle_effects.as_ref(), |m| {
        map(m, |v| one_or_many(v, particle))
    })?;
    o.opt("sound_effects", a.sound_effects.as_ref(), |m| {
        map(m, |v| one_or_many(v, sound))
    })?;
    o.opt("timeline", a.timeline.as_ref(), |m| {
        map(m, |v| one_or_many(v, event))
    })?;
    o.opt("animation_length", a.animation_length.as_ref(), |x| {
        float(*x)
    })?;
    Ok(o.done())
}

impl AnimationFile {
    /// The file as an untyped value.
    pub(crate) fn to_value(&self) -> Result<Value, EncodeError> {
        // Not `write_envelope`: no `Family` reads the file, and the version is as written.
        let mut o = Object::new();
        o.put("format_version", version(self.format_version));
        o.put("animations", map(&self.animations, animation)?);
        Ok(o.done())
    }
}
