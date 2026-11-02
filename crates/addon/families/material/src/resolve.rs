//! A material with everything it inherits applied, as the client's readers build it.

use std::collections::BTreeSet;

use indexmap::IndexMap;

use super::Materials;
use super::file::{KeyRules, MaterialDef, StencilFace};
use super::group::{Body, MaterialGroup};
use super::values::{
    RENDER_STATES, SAMPLER_SLOTS, TEXTURE_FORMATS, VERTEX_FIELDS, render_state_bit,
};

/// One stencil face of a resolved material.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ResolvedFace {
    /// The comparison function; `Always` by default.
    pub stencil_func: String,
    /// The operation when the stencil test fails; `Keep` by default.
    pub stencil_fail_op: String,
    /// The operation when the depth test fails; `Keep` by default.
    pub stencil_depth_fail_op: String,
    /// The operation when both pass; `Keep` by default.
    pub stencil_pass_op: String,
}

/// One sampler slot of a resolved material.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ResolvedSampler {
    /// Whether an entry set the slot.
    pub set: bool,
    /// The texture filter; `Point` by default.
    pub texture_filter: String,
    /// The wrap mode of all three axes; `Clamp` by default.
    pub texture_wrap: String,
    /// The shader stage; `Pixel` by default.
    pub shader_stage: String,
    /// The comparison function; by default one that has no JSON name.
    pub comparison_func: Option<String>,
}

/// A material with its parent chain applied: what the client holds after it created the
/// material ([`Materials::resolve`]).
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct ResolvedMaterial {
    /// The key rules of the material itself.
    pub key_rules: KeyRules,
    /// The render states that are on, in the order of
    /// [`RENDER_STATES`](super::RENDER_STATES).
    pub states: Vec<String>,
    /// The defines, bytewise. `ALPHA_TO_COVERAGE` is among them when the state
    /// `EnableAlphaToCoverage` is on.
    pub defines: Vec<String>,
    /// The eight sampler slots.
    pub samplers: Vec<ResolvedSampler>,
    /// `vertexShader`; empty by default.
    pub vertex_shader: String,
    /// `fragmentShader`; empty by default.
    pub fragment_shader: String,
    /// `geometryShader`; empty by default.
    pub geometry_shader: String,
    /// `depthFunc`; `Less` by default.
    pub depth_func: String,
    /// The front stencil face.
    pub front_face: ResolvedFace,
    /// The back stencil face.
    pub back_face: ResolvedFace,
    /// The stencil reference of both faces: the low byte of `stencilRef`.
    pub stencil_ref: u8,
    /// `stencilRefOverride`; also on once a `stencilRef` was read.
    pub stencil_ref_override: bool,
    /// `stencilReadMask`.
    pub stencil_read_mask: u32,
    /// `stencilWriteMask`.
    pub stencil_write_mask: u32,
    /// `blendSrc`.
    pub blend_src: String,
    /// `blendDst`.
    pub blend_dst: String,
    /// `alphaSrc`.
    pub alpha_src: String,
    /// `alphaDst`.
    pub alpha_dst: String,
    /// The vertex fields, in the order of [`VERTEX_FIELDS`](super::VERTEX_FIELDS).
    pub vertex_fields: Vec<String>,
    /// `depthBias`.
    pub depth_bias: f32,
    /// `slopeScaledDepthBias`.
    pub slope_scaled_depth_bias: f32,
    /// `msaaSupport`; `NonMSAA` by default.
    pub msaa_support: String,
    /// `primitiveMode`; `TriangleList` by default.
    pub primitive_mode: String,
    /// The render-target formats; `[[B8G8R8A8_UNORM]]` when nothing gives any.
    pub render_target_formats: Vec<Vec<String>>,
}

impl ResolvedMaterial {
    /// The stem of [`vertex_shader`](Self::vertex_shader): the text between its last `/` and
    /// the first `.` after it. A material drawn for an actor needs one of
    /// [`ACTOR_SHADER_STEMS`](super::ACTOR_SHADER_STEMS).
    pub fn vertex_shader_stem(&self) -> &str {
        let name = self
            .vertex_shader
            .rsplit_once('/')
            .map_or(self.vertex_shader.as_str(), |(_, name)| name);
        name.split_once('.').map_or(name, |(stem, _)| stem)
    }
}

/// The material `name` of `group` with its chain applied: the steps from the material up to
/// its list entry are collected, then applied from the entry down. Nothing is shared between
/// calls and nothing was copied at load, so the cost is the size of the chain: each call walks
/// it again, and [`resolve_all`] is the way to resolve many.
pub(super) fn resolve(
    materials: &Materials,
    group: &MaterialGroup,
    name: &str,
) -> Option<ResolvedMaterial> {
    let material = group.get(name)?;
    let mut chain = Vec::new();
    let mut at = Some(material.node);
    while let Some(index) = at {
        let node = group.nodes.get(usize::try_from(index).ok()?)?;
        chain.push(node.body);
        at = node.parent;
    }
    let mut state = State::new();
    for body in chain.into_iter().rev() {
        state.step(materials, group, body)?;
    }
    Some(state.finish(material.key_rules))
}

/// Every material of `group` with its chain applied, in creation order.
///
/// The steps of the group are applied in the order they were made, which puts each after the
/// step it starts from: a step takes the state its parent step left, as its own when it is the
/// parent's last child and as a copy otherwise. No chain is walked twice, so the cost is the
/// number of steps plus the size of what is returned.
pub(super) fn resolve_all(
    materials: &Materials,
    group: &MaterialGroup,
) -> IndexMap<String, ResolvedMaterial> {
    let steps = group.nodes.len();
    let place = |index: u32| usize::try_from(index).ok().filter(|at| *at < steps);
    // Per step: the last step that starts from it.
    let mut last_child: Vec<Option<usize>> = vec![None; steps];
    for (index, node) in group.nodes.iter().enumerate() {
        if let Some(parent) = node.parent.and_then(place) {
            last_child[parent] = Some(index);
        }
    }
    // Per step: the material that ends there.
    let mut ends: Vec<Option<KeyRules>> = vec![None; steps];
    for material in group.iter() {
        if let Some(at) = place(material.node) {
            ends[at] = Some(material.key_rules);
        }
    }
    let mut states: Vec<Option<State>> = (0..steps).map(|_| None).collect();
    let mut resolved: Vec<Option<ResolvedMaterial>> = (0..steps).map(|_| None).collect();
    for (index, node) in group.nodes.iter().enumerate() {
        let state = match node.parent {
            None => Some(State::new()),
            // A parent step is always an earlier one.
            Some(parent) => place(parent).filter(|at| *at < index).and_then(|at| {
                if last_child[at] == Some(index) {
                    states[at].take()
                } else {
                    states[at].clone()
                }
            }),
        };
        let Some(mut state) = state else {
            continue;
        };
        if state.step(materials, group, node.body).is_none() {
            continue;
        }
        match (ends[index], last_child[index].is_some()) {
            (Some(rules), true) => {
                resolved[index] = Some(state.clone().finish(rules));
                states[index] = Some(state);
            }
            (Some(rules), false) => resolved[index] = Some(state.finish(rules)),
            (None, true) => states[index] = Some(state),
            (None, false) => {}
        }
    }
    group
        .iter()
        .filter_map(|material| {
            let resolved = resolved.get_mut(place(material.node)?)?.take()?;
            Some((material.name.clone(), resolved))
        })
        .collect()
}

/// The bit of `EnableAlphaToCoverage`.
const ALPHA_TO_COVERAGE_BIT: u16 = 0x200;
/// The define the client inserts while that state is on.
const ALPHA_TO_COVERAGE: &str = "ALPHA_TO_COVERAGE";
/// What an empty list of render-target formats becomes.
const DEFAULT_FORMAT: &str = "B8G8R8A8_UNORM";
/// What the list becomes for an animated texture.
const ANIMATED_FORMAT: &str = "R8G8B8A8_UNORM";
/// What the client pushes for a format name it does not know.
const UNKNOWN_FORMAT: &str = "UNKNOWN_TEXTURE_FORMAT";

/// A material while its chain is applied.
#[derive(Clone)]
struct State {
    states: u16,
    defines: BTreeSet<String>,
    vertex_fields: BTreeSet<usize>,
    resolved: ResolvedMaterial,
}

impl State {
    /// The client's defaults for a material without a parent.
    fn new() -> Self {
        let face = || ResolvedFace {
            stencil_func: "Always".to_owned(),
            stencil_fail_op: "Keep".to_owned(),
            stencil_depth_fail_op: "Keep".to_owned(),
            stencil_pass_op: "Keep".to_owned(),
        };
        let resolved = ResolvedMaterial {
            key_rules: KeyRules::Legacy,
            states: Vec::new(),
            defines: Vec::new(),
            samplers: (0..SAMPLER_SLOTS).map(|_| default_sampler()).collect(),
            vertex_shader: String::new(),
            fragment_shader: String::new(),
            geometry_shader: String::new(),
            depth_func: "Less".to_owned(),
            front_face: face(),
            back_face: face(),
            stencil_ref: 0,
            stencil_ref_override: false,
            stencil_read_mask: u32::MAX,
            stencil_write_mask: u32::MAX,
            blend_src: "SourceAlpha".to_owned(),
            blend_dst: "OneMinusSrcAlpha".to_owned(),
            alpha_src: "One".to_owned(),
            alpha_dst: "OneMinusSrcAlpha".to_owned(),
            vertex_fields: Vec::new(),
            depth_bias: 0.0,
            slope_scaled_depth_bias: 0.0,
            msaa_support: "NonMSAA".to_owned(),
            primitive_mode: "TriangleList".to_owned(),
            render_target_formats: Vec::new(),
        };
        Self {
            states: 0,
            defines: BTreeSet::new(),
            vertex_fields: BTreeSet::new(),
            resolved,
        }
    }

    /// Applies one step of a chain; `None` when the step names what the group does not have.
    fn step(&mut self, materials: &Materials, group: &MaterialGroup, body: Body) -> Option<()> {
        match body {
            Body::Base(index) => {
                let defines = group.bases.get(usize::try_from(index).ok()?)?;
                self.defines.extend(defines.iter().cloned());
            }
            Body::Def { file, entry } => {
                let (_, merged) = materials.files.get_index(usize::try_from(file).ok()?)?;
                let (_, def) = merged.materials.get_index(usize::try_from(entry).ok()?)?;
                let rules = def.key_rules.unwrap_or_else(|| merged.form.key_rules());
                self.apply(def, rules);
            }
        }
        Some(())
    }

    /// Applies one material's members, in the order of the client's readers: the states, the
    /// shaders, the defines, then the runtime states.
    fn apply(&mut self, def: &MaterialDef, rules: KeyRules) {
        self.states(def, rules);
        let resolved = &mut self.resolved;
        set(&mut resolved.vertex_shader, def.vertex_shader.as_ref());
        set(&mut resolved.fragment_shader, def.fragment_shader.as_ref());
        set(&mut resolved.geometry_shader, def.geometry_shader.as_ref());
        self.defines(def, rules);
        let resolved = &mut self.resolved;
        set(&mut resolved.depth_func, def.depth_func.as_ref());
        apply_face(&mut resolved.front_face, def.front_face.as_ref());
        apply_face(&mut resolved.back_face, def.back_face.as_ref());
        if let Some(over) = def.stencil_ref_override {
            resolved.stencil_ref_override = over.is_on();
        }
        if let Some(reference) = def.stencil_ref {
            // `asUInt` truncated to one byte, for both faces; it turns the override on.
            resolved.stencil_ref = reference.to_le_bytes()[0];
            resolved.stencil_ref_override = true;
        }
        if let Some(mask) = def.stencil_read_mask {
            resolved.stencil_read_mask = mask;
        }
        if let Some(mask) = def.stencil_write_mask {
            resolved.stencil_write_mask = mask;
        }
        // An alpha factor that is not written copies the colour factor written beside it.
        set(&mut resolved.blend_src, def.blend_src.as_ref());
        set(&mut resolved.blend_dst, def.blend_dst.as_ref());
        set(
            &mut resolved.alpha_src,
            def.alpha_src.as_ref().or(def.blend_src.as_ref()),
        );
        set(
            &mut resolved.alpha_dst,
            def.alpha_dst.as_ref().or(def.blend_dst.as_ref()),
        );
        self.samplers(def, rules);
        if let Some(fields) = &def.vertex_fields {
            // Cleared, `Position` first, then each entry's field (`Position` without one).
            self.vertex_fields.clear();
            self.vertex_fields.insert(0);
            for field in fields {
                let name = field.field.as_deref().unwrap_or(VERTEX_FIELDS[0]);
                if let Some(index) = VERTEX_FIELDS.iter().position(|known| *known == name) {
                    self.vertex_fields.insert(index);
                }
            }
        }
        let resolved = &mut self.resolved;
        if let Some(bias) = def.depth_bias {
            resolved.depth_bias = bias;
        }
        if let Some(bias) = def.slope_scaled_depth_bias {
            resolved.slope_scaled_depth_bias = bias;
        }
        set(&mut resolved.msaa_support, def.msaa_support.as_ref());
        set(&mut resolved.primitive_mode, def.primitive_mode.as_ref());
        if let Some(lists) = &def.render_target_formats {
            let known = |name: &String| {
                if TEXTURE_FORMATS.contains(&name.as_str()) {
                    name.clone()
                } else {
                    UNKNOWN_FORMAT.to_owned()
                }
            };
            resolved.render_target_formats = lists
                .iter()
                .map(|list| list.iter().map(known).collect())
                .collect();
        }
        if def.is_animated_texture.is_some() {
            // The member's presence is the test; the value is never read.
            resolved.render_target_formats = vec![vec![ANIMATED_FORMAT.to_owned()]];
        } else if resolved.render_target_formats.is_empty() {
            resolved.render_target_formats = vec![vec![DEFAULT_FORMAT.to_owned()]];
        }
    }

    /// `states`: versioned, the plain key resets the bits and hides the prefixed keys, which
    /// otherwise set, then clear; legacy, the plain key sets bits and nothing else is read.
    fn states(&mut self, def: &MaterialDef, rules: KeyRules) {
        let bits = |names: &[String]| {
            names
                .iter()
                .filter_map(|name| render_state_bit(name))
                .fold(0u16, |bits, bit| bits | bit)
        };
        match (rules, &def.states.replace) {
            (KeyRules::Versioned, Some(names)) => self.states = bits(names),
            (KeyRules::Versioned, None) => {
                self.states |= def.states.add.as_deref().map_or(0, bits);
                self.states &= !def.states.sub.as_deref().map_or(0, bits);
            }
            (KeyRules::Legacy, Some(names)) => self.states |= bits(names),
            (KeyRules::Legacy, None) => {}
        }
        if self.states & ALPHA_TO_COVERAGE_BIT != 0 {
            self.defines.insert(ALPHA_TO_COVERAGE.to_owned());
        }
    }

    /// `defines`: versioned, the plain key clears the set and hides the prefixed keys, which
    /// otherwise insert, then erase; legacy, the plain key inserts without clearing, `+defines`
    /// inserts only without it, and `-defines` is never read.
    fn defines(&mut self, def: &MaterialDef, rules: KeyRules) {
        let listed = &def.defines;
        match (rules, &listed.replace) {
            (KeyRules::Versioned, Some(names)) => {
                self.defines = names.iter().cloned().collect();
            }
            (KeyRules::Versioned, None) => {
                self.defines.extend(listed.add.iter().flatten().cloned());
                for name in listed.sub.iter().flatten() {
                    self.defines.remove(name);
                }
            }
            (KeyRules::Legacy, Some(names)) => self.defines.extend(names.iter().cloned()),
            (KeyRules::Legacy, None) => {
                self.defines.extend(listed.add.iter().flatten().cloned());
            }
        }
    }

    /// `samplerStates`: versioned, the plain key resets the eight slots and hides
    /// `+samplerStates`, which otherwise applies onto the inherited slots; legacy, only the
    /// plain key, without a reset, reading the index, the filter and the wrap. `-samplerStates`
    /// is never applied.
    fn samplers(&mut self, def: &MaterialDef, rules: KeyRules) {
        let listed = &def.sampler_states;
        let entries = match (rules, &listed.replace) {
            (KeyRules::Versioned, Some(entries)) => {
                for slot in &mut self.resolved.samplers {
                    *slot = default_sampler();
                }
                entries
            }
            (KeyRules::Versioned, None) => match &listed.add {
                Some(entries) => entries,
                None => return,
            },
            (KeyRules::Legacy, Some(entries)) => entries,
            (KeyRules::Legacy, None) => return,
        };
        for entry in entries {
            // An entry without an index, or with one past the slots, is skipped.
            let index = entry
                .sampler_index
                .and_then(|index| usize::try_from(index).ok());
            let Some(slot) = index.and_then(|index| self.resolved.samplers.get_mut(index)) else {
                continue;
            };
            slot.set = true;
            set(&mut slot.texture_filter, entry.texture_filter.as_ref());
            set(&mut slot.texture_wrap, entry.texture_wrap.as_ref());
            if rules == KeyRules::Versioned {
                set(&mut slot.shader_stage, entry.shader_stage.as_ref());
                if let Some(func) = &entry.comparison_func {
                    slot.comparison_func = Some(func.clone());
                }
            }
        }
    }

    /// The material as resolved.
    fn finish(mut self, key_rules: KeyRules) -> ResolvedMaterial {
        self.resolved.key_rules = key_rules;
        self.resolved.states = RENDER_STATES
            .iter()
            .filter(|(_, bit)| self.states & bit != 0)
            .map(|(name, _)| (*name).to_owned())
            .collect();
        self.resolved.defines = self.defines.into_iter().collect();
        self.resolved.vertex_fields = self
            .vertex_fields
            .into_iter()
            .filter_map(|index| VERTEX_FIELDS.get(index))
            .map(|name| (*name).to_owned())
            .collect();
        self.resolved
    }
}

/// A sampler slot nothing set.
fn default_sampler() -> ResolvedSampler {
    ResolvedSampler {
        set: false,
        texture_filter: "Point".to_owned(),
        texture_wrap: "Clamp".to_owned(),
        shader_stage: "Pixel".to_owned(),
        comparison_func: None,
    }
}

/// Overwrites `slot` when the material writes the member.
fn set(slot: &mut String, written: Option<&String>) {
    if let Some(written) = written {
        slot.clone_from(written);
    }
}

/// Applies a face a material writes onto the inherited one, member by member.
fn apply_face(face: &mut ResolvedFace, written: Option<&StencilFace>) {
    let Some(written) = written else {
        return;
    };
    set(&mut face.stencil_func, written.stencil_func.as_ref());
    set(&mut face.stencil_fail_op, written.stencil_fail_op.as_ref());
    set(
        &mut face.stencil_depth_fail_op,
        written.stencil_depth_fail_op.as_ref(),
    );
    set(&mut face.stencil_pass_op, written.stencil_pass_op.as_ref());
}
