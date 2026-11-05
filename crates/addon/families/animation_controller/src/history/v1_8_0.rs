//! Format 1.8.0 up to 1.10.0: state `parameters` and blend curves instead of `variables`; and the
//! step `upgrade_v1_8_to_v1_10`.
//!
//! The 1.8 schema only validates; the step does the work of turning the file into the 1.10 shape.

use indexmap::IndexMap;

use super::super::parse::{V1_8_0, controller_format_version, read_controllers, read_objects};
use super::super::{
    AnimationController as LatestController, AnimationControllerFile as Latest, EventString,
    ParticleEffect as LatestParticleEffect, State as LatestState, StateAnimation, StateVariable,
    Transition as LatestTransition,
};
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Phase};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::molang::{
    Molang, MolangFields, MolangSource, MolangVisitor, RawVersion, Step,
};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{ParseCx, Upgrade, UpgradeCx, VersionStruct, Versioned};
use bedrock_addon_support::animation::event::command_version;
use bedrock_addon_support::animation::read::read_particle;
use bedrock_addon_support::animation::walk::{
    CHILD_NOT_VALID, Group, Kind, Node, Pattern, Walk, text,
};

/// A 1.8 controller file: `{"format_version", "animation_controllers"}` with the 1.8 state shape.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationControllerFile {
    /// `format_version`, as written.
    pub format_version: FormatVersion,
    /// `animation_controllers`: each controller by its key as written.
    pub animation_controllers: IndexMap<String, AnimationController>,
}

/// One 1.8 controller.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AnimationController {
    /// `initial_state`.
    pub initial_state: Option<String>,
    /// `states`, by name as written.
    pub states: IndexMap<String, State>,
}

/// One 1.8 state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct State {
    /// `parameters`: one Molang input per curve index, with the key as written (the step
    /// removes only a key spelled exactly `parameters`).
    pub parameters: Option<Parameters>,
    /// `animations`: each element `{"<animation>": [curve, …]}` (exactly one member).
    pub animations: Option<Vec<CurveSet>>,
    /// `transitions`: `{"<state>": <condition>}` elements (exactly one member each).
    pub transitions: Option<Vec<Transition>>,
    /// `particle_effects`: `pre_effect_script` is a plain string in 1.8.
    pub particle_effects: Option<Vec<ParticleEffect>>,
    /// `on_entry`: event strings.
    pub on_entry: Option<Vec<String>>,
    /// `on_exit`: event strings.
    pub on_exit: Option<Vec<String>>,
}

/// A 1.8 state's `parameters`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Parameters {
    /// The member's key as written (`parameters` in any case).
    pub key: String,
    /// The Molang sources, by curve index.
    pub inputs: Vec<String>,
}

/// One element of a 1.8 state's `animations`: an animation and one remap curve per parameter.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CurveSet {
    /// The animation's short name, as written.
    pub animation: String,
    /// Remap curves (`{"<input>": output}`), one per `parameters` index.
    pub curves: Vec<IndexMap<String, f32>>,
}

/// A 1.8 transition: the condition is a typed Molang node.
#[derive(Clone, Debug, PartialEq)]
pub struct Transition {
    /// The target state.
    pub target: String,
    /// The condition.
    pub condition: Molang,
}

/// A 1.8 particle effect.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ParticleEffect {
    /// `effect`.
    pub effect: String,
    /// `locator`.
    pub locator: Option<String>,
    /// `pre_effect_script`: a plain string in 1.8.
    pub pre_effect_script: Option<String>,
    /// `bind_to_actor`: only `false` may be written.
    pub bind_to_actor: Option<bool>,
}

// ---- the v1_8 schema ----------------------------------------------------------------------------

/// The walk of a whole 1.8 file.
pub(crate) fn read_file(walk: &mut Walk, root: Node<'_>) -> Option<AnimationControllerFile> {
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
        0 => controller_format_version(walk, node, &V1_8_0)
            .map(|v| version = Some(v))
            .into(),
        _ => read_controllers(walk, node, read_state)
            .map(|c| controllers = Some(c))
            .into(),
    });
    let file = AnimationControllerFile {
        format_version: version?.format_version(),
        animation_controllers: controllers?
            .into_iter()
            .map(|(key, (initial_state, states))| {
                (
                    key,
                    AnimationController {
                        initial_state,
                        states,
                    },
                )
            })
            .collect(),
    };
    ok.then_some(file)
}

const STATE_MEMBERS: [Group; 6] = [
    Group::name("parameters"),
    Group::name("animations"),
    Group::name("transitions"),
    Group::name("particle_effects"),
    Group::name("on_entry"),
    Group::name("on_exit"),
];

fn read_state(walk: &mut Walk, node: Node<'_>) -> Option<State> {
    let mut state = State::default();
    let ok = walk.object(node, &STATE_MEMBERS, |walk, group, key, v| match group {
        0 => walk
            .strings(v)
            .map(|inputs| {
                state.parameters = Some(Parameters {
                    key: key.to_owned(),
                    inputs,
                });
            })
            .into(),
        1 => read_curve_sets(walk, v)
            .map(|x| state.animations = Some(x))
            .into(),
        2 => read_transitions(walk, v)
            .map(|x| state.transitions = Some(x))
            .into(),
        3 => read_objects(walk, v, read_particle_1_8)
            .map(|x| state.particle_effects = Some(x))
            .into(),
        4 => walk.strings(v).map(|x| state.on_entry = Some(x)).into(),
        _ => walk.strings(v).map(|x| state.on_exit = Some(x)).into(),
    });
    ok.then_some(state)
}

/// `animations`: object elements only, exactly one member each, whose value is an array of
/// curve objects.
fn read_curve_sets(walk: &mut Walk, node: Node<'_>) -> Option<Vec<CurveSet>> {
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
            |walk, _, animation, v| {
                let mut curves = Vec::new();
                let ok = walk.array(v, |walk, _, curve| {
                    if curve.kind() != Kind::Object {
                        walk.bad_type("object", curve);
                        return false;
                    }
                    walk.float_map(curve, Pattern::Time, false)
                        .map(|c| curves.push(c))
                        .is_some()
                });
                if ok {
                    list.push(CurveSet {
                        animation: animation.to_owned(),
                        curves,
                    });
                }
                ok.into()
            },
        )
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
                walk.molang(v)
                    .map(|condition| {
                        list.push(Transition {
                            target: target.to_owned(),
                            condition,
                        });
                    })
                    .into()
            },
        )
    });
    ok.then_some(list)
}

fn read_particle_1_8(walk: &mut Walk, node: Node<'_>) -> Option<ParticleEffect> {
    // The 1.8 `pre_effect_script` is a string node; the rest is the shared event shape.
    let mut script = None;
    let members = walk.members(node)?;
    for (key, value) in &members {
        if key.eq_ignore_ascii_case("pre_effect_script") && value.kind() != Kind::String {
            walk.enter(key);
            walk.bad_type("string", *value);
            walk.leave();
            return None;
        }
        if key.eq_ignore_ascii_case("pre_effect_script") {
            script = walk.string(*value);
        }
    }
    let effect: LatestParticleEffect = read_particle(walk, node)?;
    Some(ParticleEffect {
        effect: effect.effect,
        locator: effect.locator,
        pre_effect_script: script,
        bind_to_actor: effect.bind_to_actor,
    })
}

// ---- the step -----------------------------------------------------------------------------------

impl Versioned for AnimationControllerFile {
    const VERSION: FormatVersion = FormatVersion::new(1, 8, 0);
}

impl Versioned for Latest {
    const VERSION: FormatVersion = FormatVersion::new(1, 10, 0);
}

impl Upgrade for AnimationControllerFile {
    type Next = Latest;

    /// `upgrade_v1_8_to_v1_10`: `format_version` becomes 1.10; each state's blend curves become
    /// `variables["<animation>_<i>"] = {input: parameters[i], remap_curve: curve_i}` and the
    /// animation's weight `variable.<animation>_0+…` (`"1.0f"` without curves); a key spelled
    /// exactly `parameters` is removed.
    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Latest {
        let commands = command_version(cx.load.min_engine_version.as_ref());
        let mut at = JsonPath::new();
        at.push("animation_controllers");
        let animation_controllers = self
            .animation_controllers
            .into_iter()
            .map(|(key, controller)| {
                at.push(&key);
                at.push("states");
                let states = controller
                    .states
                    .into_iter()
                    .map(|(name, state)| {
                        at.push(&name);
                        let state = upgrade_state(state, &at, commands, cx.diagnostics);
                        at.pop();
                        (name, state)
                    })
                    .collect();
                at.pop();
                at.pop();
                (
                    key,
                    LatestController {
                        initial_state: controller.initial_state,
                        states,
                        lowercase_transitions: cx.original_version < FormatVersion::new(1, 17, 30),
                        legacy_event_skipping: cx.original_version < FormatVersion::new(1, 18, 10),
                    },
                )
            })
            .collect();
        Latest {
            format_version: FormatVersion::new(1, 10, 0),
            animation_controllers,
        }
    }
}

/// A finding of the `v1_10` schema the upgraded document fails, at `at`.
fn schema_error(diagnostics: &mut Diagnostics, code: DiagnosticCode, at: &JsonPath) {
    diagnostics.push(Diagnostic::error(code).at(at.clone()).phase(Phase::Upgrade));
}

fn expression(text: String) -> Molang {
    Molang::Expr(MolangSource::string(text, RawVersion(-1)))
}

#[allow(
    clippy::too_many_lines,
    reason = "one state's whole step, in vanilla's order"
)]
fn upgrade_state(
    state: State,
    at: &JsonPath,
    commands: u8,
    diagnostics: &mut Diagnostics,
) -> LatestState {
    let inputs = state
        .parameters
        .as_ref()
        .map(|p| p.inputs.as_slice())
        .unwrap_or_default();
    let mut variables: Option<IndexMap<String, StateVariable>> = None;
    // How many of the inputs a curve of the state reads (the longest list of curves).
    let mut read = 0;
    // Where each variable's curve stands in the file: the entry and the curve's index.
    let mut made: IndexMap<String, (usize, usize)> = IndexMap::new();
    let animations = state.animations.map(|sets| {
        sets.into_iter()
            .enumerate()
            .map(|(entry, set)| {
                let mut weight = "1.0f".to_owned();
                read = read.max(set.curves.len());
                for (index, curve) in set.curves.into_iter().enumerate() {
                    let name = format!("{}_{index}", set.animation);
                    let input = if let Some(input) = inputs.get(index) {
                        expression(input.clone())
                    } else {
                        // A `parameters` index past the end reads as null, which the 1.10
                        // `input` node refuses.
                        let mut at = at.clone();
                        at.push("variables");
                        at.push(&name);
                        at.push("input");
                        schema_error(
                            diagnostics,
                            bedrock_addon_core::diagnostic::cereal::unknown_child_option(
                                "ExpressionNode",
                            ),
                            &at,
                        );
                        Molang::Const(0.0)
                    };
                    weight = if index == 0 {
                        format!("variable.{name}")
                    } else {
                        format!("{weight}+variable.{name}")
                    };
                    let variable = StateVariable {
                        input,
                        remap_curve: Some(curve),
                    };
                    // A curve of an earlier entry of the same name is written over, as the
                    // game's step writes over it; the crate notes where that happens.
                    let variables = variables.get_or_insert_with(IndexMap::new);
                    if let Some((earlier, _)) = made.insert(name.clone(), (entry, index))
                        && variables
                            .get(&name)
                            .is_some_and(|had| had.remap_curve != variable.remap_curve)
                    {
                        let mut at = at.clone();
                        at.push("animations");
                        at.push_index(earlier);
                        at.push(&set.animation);
                        at.push_index(index);
                        diagnostics.push(
                            Diagnostic::info(DiagnosticCode::DroppedField { text: None })
                                .at(at)
                                .phase(Phase::Upgrade),
                        );
                    }
                    variables.insert(name, variable);
                }
                let mut weights = IndexMap::new();
                weights.insert(set.animation, expression(weight));
                StateAnimation::Weighted(weights)
            })
            .collect()
    });
    // An input no curve reads goes with the `parameters` the step removes: noted at the
    // input. The game says nothing here, so the crate reports it as an information.
    if let Some(parameters) = &state.parameters
        && parameters.key == "parameters"
    {
        for index in read..parameters.inputs.len() {
            let mut at = at.clone();
            at.push(&parameters.key);
            at.push_index(index);
            diagnostics.push(
                Diagnostic::info(DiagnosticCode::DroppedField { text: None })
                    .at(at)
                    .phase(Phase::Upgrade),
            );
        }
    }
    if let Some(parameters) = &state.parameters
        && parameters.key != "parameters"
    {
        // The step removes only the key spelled exactly `parameters`; any other casing stays and
        // fails the 1.10 schema.
        let mut at = at.clone();
        at.push(&parameters.key);
        schema_error(
            diagnostics,
            DiagnosticCode::UnrecognizedField {
                text: text(CHILD_NOT_VALID, &[&parameters.key]),
            },
            &at,
        );
    }
    let transitions = state.transitions.map(|list| {
        list.into_iter()
            .enumerate()
            .map(|(index, transition)| {
                let condition = match transition.condition {
                    Molang::Expr(source) => source,
                    other => {
                        // The v1_10 transition value is a string node.
                        let mut at = at.clone();
                        at.push("transitions");
                        at.push_index(index);
                        at.push(&transition.target);
                        schema_error(
                            diagnostics,
                            bedrock_addon_core::diagnostic::cereal::unknown_child_option("string"),
                            &at,
                        );
                        MolangSource::string(format!("{other:?}"), RawVersion(-1))
                    }
                };
                LatestTransition {
                    target: transition.target,
                    condition,
                }
            })
            .collect()
    });
    let particle_effects = state.particle_effects.map(|list| {
        list.into_iter()
            .map(|p| LatestParticleEffect {
                effect: p.effect,
                locator: p.locator,
                pre_effect_script: p.pre_effect_script.map(expression),
                bind_to_actor: p.bind_to_actor,
            })
            .collect()
    });
    let events = |list: Option<Vec<String>>| {
        list.map(|list| {
            list.iter()
                .map(|s| EventString::classify(s, commands))
                .collect()
        })
    };
    LatestState {
        variables,
        animations,
        transitions,
        particle_effects,
        on_entry: events(state.on_entry),
        on_exit: events(state.on_exit),
        ..LatestState::default()
    }
}

// ---- the chain's struct traits ----------------------------------------------------------------

impl MolangFields for AnimationControllerFile {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visitor.enter(Step::Key("animation_controllers"));
        for (key, controller) in &mut self.animation_controllers {
            visitor.enter(Step::Key(key));
            visitor.enter(Step::Key("states"));
            for (name, state) in &mut controller.states {
                visitor.enter(Step::Key(name));
                if let Some(transitions) = &mut state.transitions {
                    visitor.enter(Step::Key("transitions"));
                    for (index, t) in transitions.iter_mut().enumerate() {
                        visitor.enter(Step::Index(index));
                        visitor.field(
                            t.target.as_str(),
                            &bedrock_addon_support::animation::molang::CONTROLLER_TRANSITION_1_8,
                            &mut t.condition,
                        );
                        visitor.leave();
                    }
                    visitor.leave();
                }
                visitor.leave();
            }
            visitor.leave();
            visitor.leave();
        }
        visitor.leave();
    }
}

impl VersionStruct for AnimationControllerFile {
    fn parse(payload: JsonSlice<'_>, _: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let mut walk = Walk::new();
        let file = Node::of(payload).and_then(|root| read_file(&mut walk, root));
        let failed = walk.diagnostics.has_errors();
        d.extend(walk.diagnostics);
        file.filter(|_| !failed)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        self.to_value()
    }
}

impl VersionStruct for Latest {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let mut walk = Walk::new();
        let commands = command_version(cx.load.min_engine_version.as_ref());
        let written = bedrock_addon_core::version::SemVersion::new(
            cx.original_version.major,
            cx.original_version.minor,
            cx.original_version.patch,
        );
        let file = Node::of(payload)
            .and_then(|root| super::super::parse::read_file(&mut walk, root, commands, &written));
        let failed = walk.diagnostics.has_errors();
        d.extend(walk.diagnostics);
        let mut file = file.filter(|_| !failed)?;
        super::super::parse::set_gates(&mut file, &written);
        Some(file)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        self.to_value()
    }
}
