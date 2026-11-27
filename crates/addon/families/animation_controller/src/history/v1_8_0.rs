//! Format 1.8.0 up to 1.10.0: state `parameters` and blend curves instead of `variables`; and the
//! step `upgrade_v1_8_to_v1_10`.
//!
//! The 1.8 schema only validates; the step does the work of turning the file into the 1.10 shape.

use facet::Facet;
use indexmap::IndexMap;

use super::super::read::{CONTROLLERS, V1_8_0, controller_format_version, single};
use super::super::{
    AnimationController as LatestController, AnimationControllerFile as Latest, EventString,
    ParticleEffect as LatestParticleEffect, State as LatestState, StateAnimation, StateVariable,
    Transition as LatestTransition,
};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, JsonPath, Phase, Severity};
use bedrock_addon_core::json::kit::{Expected, Node, Read};
use bedrock_addon_core::json::parse::{Options, Parsed as Text, parse_slice};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value, Winner};
use bedrock_addon_core::molang::{
    Molang, MolangFields, MolangSource, MolangVisitor, RawVersion, Step,
};
use bedrock_addon_core::schema::{self, FromJson, Schema, read_document, read_each, read_keyed};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::known;
use bedrock_addon_core::versioning::{ParseCx, Upgrade, UpgradeCx, VersionStruct, Versioned};
use bedrock_addon_shared_animation::event::command_version;
use bedrock_addon_shared_animation::read::Written;
use bedrock_addon_shared_animation::walk::{IDENTIFIERS, TIMES};

/// A 1.8 controller file: `{"format_version", "animation_controllers"}` with the 1.8 state shape.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationControllerFile {
    /// `format_version`, as written.
    pub format_version: FormatVersion,
    /// `animation_controllers`: each controller by its key as written.
    pub animation_controllers: IndexMap<String, AnimationController>,
}

/// One 1.8 controller.
#[derive(Clone, Debug, Default, PartialEq, Facet, Schema)]
#[facet(schema::molang_with = by_file_walk)]
pub struct AnimationController {
    /// `initial_state`.
    pub initial_state: Option<String>,
    /// `states`, by name as written.
    #[facet(schema::keys = IDENTIFIERS)]
    pub states: IndexMap<String, State>,
}

/// One 1.8 state.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::molang_with = by_file_walk)]
pub struct State {
    /// `parameters`: one Molang input per curve index, with the key as written (the step
    /// removes only a key spelled exactly `parameters`).
    #[facet(schema::read_with = parameters)]
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
#[derive(Clone, Debug, Default, PartialEq, Eq, Facet)]
pub struct Parameters {
    /// The member's key as written (`parameters` in any case).
    pub key: String,
    /// The Molang sources, by curve index.
    pub inputs: Vec<String>,
}

/// One element of a 1.8 state's `animations`: an animation and one remap curve per parameter.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct CurveSet {
    /// The animation's short name, as written.
    pub animation: String,
    /// Remap curves (`{"<input>": output}`), one per `parameters` index.
    pub curves: Vec<IndexMap<String, f32>>,
}

/// A 1.8 transition: the condition is a typed Molang node.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Transition {
    /// The target state.
    pub target: String,
    /// The condition.
    pub condition: Molang,
}

/// A 1.8 particle effect.
#[derive(Clone, Debug, Default, PartialEq, Eq, Facet)]
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

/// A 1.8 controller file as its schema reads it.
#[derive(Facet, Schema)]
struct Document {
    #[facet(schema::read_with = version_1_8)]
    format_version: Written,
    #[facet(schema::keys = CONTROLLERS)]
    animation_controllers: IndexMap<String, AnimationController>,
}

/// The 1.8 schema's `format_version` node.
fn version_1_8(node: &Node<'_>, read: &mut Read<'_>) -> Option<Written> {
    controller_format_version(node, read, &V1_8_0)
}

/// The Molang walk of a value the file's walk reaches by its own: nothing of its own.
fn by_file_walk<T>(_: &mut T, _: &mut MolangVisitor<'_>) {}

/// The read of a whole 1.8 file.
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

/// `parameters`: strings, with the member's key as written.
fn parameters(node: &Node<'_>, read: &mut Read<'_>) -> Option<Parameters> {
    let inputs = read_each(node, read, String::from_json)?;
    let key = node.path().last_key().unwrap_or_default().to_owned();
    Some(Parameters { key, inputs })
}

/// `{"<animation>": [curve, …]}`, each curve an object of time keys.
impl FromJson for CurveSet {
    const EXPECTED: Option<Expected> = Some(Expected::Object);

    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let (animation, curves) = single(node, read, |node, read| {
            read_each(node, read, |curve, read| {
                read_keyed(curve, read, &TIMES, f32::from_json)
            })
        })?;
        Some(Self { animation, curves })
    }
}

/// `{"<state>": <condition>}`, the condition a typed Molang node.
impl FromJson for Transition {
    const EXPECTED: Option<Expected> = Some(Expected::Object);

    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let (target, condition) = single(node, read, Molang::from_json)?;
        Some(Self { target, condition })
    }
}

/// The shared particle event, but for `pre_effect_script`, a string node in 1.8: one of another
/// type is told before anything else of the effect is read, and the effect is not read.
impl FromJson for ParticleEffect {
    const EXPECTED: Option<Expected> = Some(Expected::Object);

    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let members = read.object(node)?;
        let mut script = None;
        for (key, value) in members.iter() {
            if !key.eq_ignore_ascii_case("pre_effect_script") {
                continue;
            }
            let Some(text) = value.value().as_string() else {
                read.wrong_accepting(&value, "string");
                return None;
            };
            script = Some(text.as_str().to_owned());
        }
        let effect = LatestParticleEffect::from_json(node, read)?;
        Some(Self {
            effect: effect.effect,
            locator: effect.locator,
            pre_effect_script: script,
            bind_to_actor: effect.bind_to_actor,
        })
    }
}

bedrock_addon_core::impl_schema_leaf!(Parameters, CurveSet, Transition, ParticleEffect);
bedrock_addon_core::impl_no_molang!(Parameters, CurveSet, Transition, ParticleEffect);

// ---- the step -----------------------------------------------------------------------------------

impl Versioned for AnimationControllerFile {
    const VERSION: FormatVersion = known::V1_8_0;
}

impl Versioned for Latest {
    const VERSION: FormatVersion = known::V1_10_0;
}

impl Upgrade for AnimationControllerFile {
    type Next = Latest;

    /// `upgrade_v1_8_to_v1_10`: `format_version` becomes 1.10; each state's blend curves become
    /// `variables["<animation>_<i>"] = {input: parameters[i], remap_curve: curve_i}` and the
    /// animation's weight `variable.<animation>_0+…` (`"1.0f"` without curves); a key spelled
    /// exactly `parameters` is removed.
    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Latest {
        let commands = command_version(cx.load.min_engine_version.as_ref());
        let mut at = JsonPath::new().child("animation_controllers");
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
            format_version: known::V1_10_0,
            animation_controllers,
        }
    }
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
        .map_or_default(|p| p.inputs.as_slice());
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
                        let at = at.child("variables").child(&name).child("input");
                        diagnostics.push(
                            Diagnostic::error(
                                bedrock_addon_core::diagnostic::cereal::unknown_child_option(
                                    "ExpressionNode",
                                ),
                            )
                            .at(at.clone())
                            .phase(Phase::Upgrade),
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
                        let at = at
                            .child("animations")
                            .index(earlier)
                            .child(&set.animation)
                            .index(index);
                        diagnostics.push(
                            Diagnostic::of(&core_codes::DROPPED_FIELD)
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
            let at = at.child(&parameters.key).index(index);
            diagnostics.push(
                Diagnostic::of(&core_codes::DROPPED_FIELD)
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
        let at = at.child(&parameters.key);
        diagnostics.push(
            Diagnostic::of(core_codes::CHILD_NOT_VALID.with([parameters.key.as_str()]))
                .severity(Severity::Error)
                .at(at.clone())
                .phase(Phase::Upgrade),
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
                        let at = at
                            .child("transitions")
                            .index(index)
                            .child(&transition.target);
                        diagnostics.push(
                            Diagnostic::error(
                                bedrock_addon_core::diagnostic::cereal::unknown_child_option(
                                    "string",
                                ),
                            )
                            .at(at.clone())
                            .phase(Phase::Upgrade),
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
                            &bedrock_addon_shared_animation::molang::CONTROLLER_TRANSITION_1_8,
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

/// The payload read with its spans, as the controller loader reads its text.
fn text_of(payload: JsonSlice<'_>) -> Option<Text<'_>> {
    parse_slice(payload, Options::winner(Winner::LastInPlace)).ok()
}

/// `read` of the payload with its findings into `d`; the value when it raised no error.
fn read_payload<T>(
    payload: JsonSlice<'_>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
    read: impl FnOnce(&Text<'_>, &mut Read<'_>) -> Option<T>,
) -> Option<T> {
    let text = text_of(payload)?;
    let mut found = Diagnostics::default();
    let mut reader = bedrock_addon_shared_animation::walk::read(cx.load, &mut found);
    let file = read(&text, &mut reader);
    drop(reader);
    let failed = found.has_errors();
    d.extend(found);
    file.filter(|_| !failed)
}

impl VersionStruct for AnimationControllerFile {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        read_payload(payload, cx, d, read_file)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        self.to_value()
    }
}

impl VersionStruct for Latest {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let written = bedrock_addon_core::version::SemVersion::new(
            cx.original_version.major,
            cx.original_version.minor,
            cx.original_version.patch,
        );
        let mut file = read_payload(payload, cx, d, super::super::read::read_file)?;
        super::super::read::set_gates(&mut file, &written);
        Some(file)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        self.to_value()
    }
}
