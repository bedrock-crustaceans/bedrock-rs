//! Writing an animation controller file back: members in declaration order, only those the value
//! holds, each in the form it was read.

use super::history::v1_8_0;
use super::{AnimationController, AnimationControllerFile, BlendTransition, State, StateAnimation};
use bedrock_addon_core::json::{EncodeError, Value};
use bedrock_addon_support::animation::write::{
    Object, event, float, floats, list, map, molang, particle, sound, string, version,
};

fn state(s: &State) -> Result<Value, EncodeError> {
    let mut o = Object::new();
    o.opt("variables", s.variables.as_ref(), |vars| {
        map(vars, |v| {
            let mut o = Object::new();
            o.put("input", molang(&v.input)?);
            o.opt("remap_curve", v.remap_curve.as_ref(), floats)?;
            Ok(o.done())
        })
    })?;
    o.opt("animations", s.animations.as_ref(), |list_| {
        list(list_, |a| match a {
            StateAnimation::Name(name) => Ok(string(name)),
            StateAnimation::Weighted(weights) => map(weights, molang),
        })
    })?;
    o.opt("transitions", s.transitions.as_ref(), |list_| {
        list(list_, |t| {
            let mut o = Object::new();
            o.put(&t.target, string(t.condition.as_str()));
            Ok(o.done())
        })
    })?;
    o.opt(
        "blend_transition",
        s.blend_transition.as_ref(),
        |b| match b {
            BlendTransition::Seconds(x) => float(*x),
            BlendTransition::Curve(curve) => floats(curve),
        },
    )?;
    o.opt(
        "blend_via_shortest_path",
        s.blend_via_shortest_path.as_ref(),
        |b| Ok(Value::from(*b)),
    )?;
    o.opt("particle_effects", s.particle_effects.as_ref(), |l| {
        list(l, particle)
    })?;
    o.opt("sound_effects", s.sound_effects.as_ref(), |l| {
        list(l, sound)
    })?;
    o.opt("on_entry", s.on_entry.as_ref(), |l| list(l, event))?;
    o.opt("on_exit", s.on_exit.as_ref(), |l| list(l, event))?;
    Ok(o.done())
}

fn controller(c: &AnimationController) -> Result<Value, EncodeError> {
    let mut o = Object::new();
    o.opt("initial_state", c.initial_state.as_ref(), |s| Ok(string(s)))?;
    o.put("states", map(&c.states, state)?);
    Ok(o.done())
}

impl AnimationControllerFile {
    /// The file as an untyped value.
    pub(crate) fn to_value(&self) -> Result<Value, EncodeError> {
        let mut o = Object::new();
        o.put("format_version", version(self.format_version));
        o.put(
            "animation_controllers",
            map(&self.animation_controllers, controller)?,
        );
        Ok(o.done())
    }
}

fn state_1_8(s: &v1_8_0::State) -> Result<Value, EncodeError> {
    let mut o = Object::new();
    if let Some(parameters) = &s.parameters {
        o.put(
            &parameters.key,
            list(&parameters.inputs, |s| Ok(string(s)))?,
        );
    }
    o.opt("animations", s.animations.as_ref(), |sets| {
        list(sets, |set| {
            let mut o = Object::new();
            o.put(&set.animation, list(&set.curves, floats)?);
            Ok(o.done())
        })
    })?;
    o.opt("transitions", s.transitions.as_ref(), |l| {
        list(l, |t| {
            let mut o = Object::new();
            o.put(&t.target, molang(&t.condition)?);
            Ok(o.done())
        })
    })?;
    o.opt("particle_effects", s.particle_effects.as_ref(), |l| {
        list(l, |p| {
            let mut o = Object::new();
            o.put("effect", string(&p.effect));
            o.opt("locator", p.locator.as_ref(), |s| Ok(string(s)))?;
            o.opt("pre_effect_script", p.pre_effect_script.as_ref(), |s| {
                Ok(string(s))
            })?;
            o.opt("bind_to_actor", p.bind_to_actor.as_ref(), |b| {
                Ok(Value::from(*b))
            })?;
            Ok(o.done())
        })
    })?;
    o.opt("on_entry", s.on_entry.as_ref(), |l| {
        list(l, |s| Ok(string(s)))
    })?;
    o.opt("on_exit", s.on_exit.as_ref(), |l| {
        list(l, |s| Ok(string(s)))
    })?;
    Ok(o.done())
}

impl v1_8_0::AnimationControllerFile {
    /// The file as an untyped value.
    pub(crate) fn to_value(&self) -> Result<Value, EncodeError> {
        let mut o = Object::new();
        o.put("format_version", version(self.format_version));
        o.put(
            "animation_controllers",
            map(&self.animation_controllers, |c| {
                let mut o = Object::new();
                o.opt("initial_state", c.initial_state.as_ref(), |s| Ok(string(s)))?;
                o.put("states", map(&c.states, state_1_8)?);
                Ok(o.done())
            })?,
        );
        Ok(o.done())
    }
}
