//! The Molang members of an animation controller file: the walk over them.

use super::{AnimationControllerFile, State};
use bedrock_addon_core::molang::{MolangFields, MolangVisitor, Step};
use bedrock_addon_support::animation::molang::{
    CONTROLLER_EVENT, CONTROLLER_TRANSITION, CONTROLLER_VALUE, event, particle,
};

impl MolangFields for State {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let spec = &CONTROLLER_VALUE;
        if let Some(variables) = &mut self.variables {
            visitor.enter(Step::Key("variables"));
            for (name, variable) in variables.iter_mut() {
                visitor.enter(Step::Key(name));
                visitor.field("input", spec, &mut variable.input);
                visitor.leave();
            }
            visitor.leave();
        }
        if let Some(animations) = &mut self.animations {
            visitor.enter(Step::Key("animations"));
            for (index, animation) in animations.iter_mut().enumerate() {
                if let super::StateAnimation::Weighted(weights) = animation {
                    visitor.field(index, spec, weights);
                }
            }
            visitor.leave();
        }
        if let Some(transitions) = &mut self.transitions {
            visitor.enter(Step::Key("transitions"));
            for (index, transition) in transitions.iter_mut().enumerate() {
                visitor.enter(Step::Index(index));
                let super::Transition { target, condition } = transition;
                visitor.field(target.as_str(), &CONTROLLER_TRANSITION, condition);
                visitor.leave();
            }
            visitor.leave();
        }
        if let Some(effects) = &mut self.particle_effects {
            visitor.enter(Step::Key("particle_effects"));
            for (index, effect) in effects.iter_mut().enumerate() {
                visitor.enter(Step::Index(index));
                particle(visitor, spec, effect);
                visitor.leave();
            }
            visitor.leave();
        }
        for (member, events) in [
            ("on_entry", &mut self.on_entry),
            ("on_exit", &mut self.on_exit),
        ] {
            if let Some(events) = events {
                visitor.enter(Step::Key(member));
                for (index, e) in events.iter_mut().enumerate() {
                    event(visitor, Step::Index(index), &CONTROLLER_EVENT, e);
                }
                visitor.leave();
            }
        }
    }
}

impl MolangFields for AnimationControllerFile {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visitor.enter(Step::Key("animation_controllers"));
        for (key, controller) in &mut self.animation_controllers {
            visitor.enter(Step::Key(key));
            visitor.nested("states", &mut controller.states);
            visitor.leave();
        }
        visitor.leave();
    }
}
