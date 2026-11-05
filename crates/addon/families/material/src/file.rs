//! A `.material` file as written: its form, its keys and the members of one material.

use std::fmt;

use indexmap::IndexMap;

use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::version::SemVersion;

/// One `.material` file (`materials/entity.material`): named materials, each of which may
/// inherit from another of the same file.
///
/// - **Forms**: [`MaterialFileForm`]. The client has no `format_version` for these files; the
///   only version is the wrapper's `materials.version`, which must be exactly `1.0.0`.
/// - **Inheritance**: a key `name:parent` copies `parent` and applies its own members on top.
///   The parent must be a material of the same merged file.
/// - **Across packs**: every pack's copy of a file is merged as JSON before it is read
///   (`rp.materials().files`), so a pack ships only the materials it adds or changes.
#[doc(alias = "material")]
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialFile {
    /// How the file is laid out.
    pub form: MaterialFileForm,
    /// The materials in file order, by key. In the single form, the one material under the
    /// file stem.
    pub materials: IndexMap<MaterialKey, MaterialDef>,
    /// The root members that are no material: a member whose value is no object, and the
    /// `materials` member of a wrapper the client does not unwrap. Kept for write-back.
    pub extra: UnknownMembers,
}

impl MaterialFile {
    /// Which key rules the material at `key` is read with: its own
    /// [`key_rules`](MaterialDef::key_rules) when the stack merge set them, else the form's.
    /// `None` when the file has no such key.
    pub fn key_rules(&self, key: &MaterialKey) -> Option<KeyRules> {
        let def = self.materials.get(key)?;
        Some(def.key_rules.unwrap_or(self.form.key_rules()))
    }
}

/// How a `.material` file is laid out.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum MaterialFileForm {
    /// `{ "materials": { "version": "1.0.0", <key>: <material>, … } }`, with `materials` the
    /// root's only member. Every vanilla file but `sky.material` is written this way.
    Wrapped {
        /// `materials.version` as written. The client unwraps the file only when this equals
        /// `1.0.0` (build metadata aside), so a value read from a file always does.
        version: SemVersion,
    },
    /// The root object is the material map itself, with no version.
    Flat,
    /// The root object is one material, named after the file: the legacy form the client
    /// takes when the root has a string `vertexShader`.
    Single {
        /// The file name without its directory and extension (`materials/foo.material` is
        /// `foo`): the material's name.
        stem: String,
        /// The version of the `materials` wrapper around the material, when the file has
        /// one: the client looks for the `vertexShader` after it unwrapped the file.
        wrapper: Option<SemVersion>,
    },
}

impl MaterialFileForm {
    /// The key rules of the form's materials: versioned in a wrapped file, legacy otherwise.
    pub fn key_rules(&self) -> KeyRules {
        match self {
            Self::Wrapped { .. } => KeyRules::Versioned,
            Self::Flat | Self::Single { .. } => KeyRules::Legacy,
        }
    }
}

/// Which rules the `states`, `defines` and `samplerStates` members of a material follow.
///
/// The client stamps each material with its file's version and branches on the major number.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum KeyRules {
    /// A material of a wrapped `1.0.0` file: the plain key replaces what is inherited and
    /// hides the `+` / `-` keys; without it `+key` adds, then `-key` removes.
    Versioned,
    /// A material of a flat file: `states` and `defines` add to what is inherited, `+defines`
    /// is read only without `defines`, and `+states`, `-states`, `-defines` and
    /// `+samplerStates` are never read.
    Legacy,
}

/// The key of a material in its file: `[+]<name>[:<parent>]`.
///
/// The parent is the text after the **last** `:`. The merge of two packs' copies and the
/// duplicate check compare the text before the **first** `:` ([`base_name`](Self::base_name));
/// the two differ only for a key with two or more `:`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MaterialKey {
    /// A leading `+`: the body is applied onto the material of that name the group already
    /// has, instead of defining it.
    pub extend: bool,
    /// The material's name.
    pub name: String,
    /// The name of the material it inherits from.
    pub parent: Option<String>,
}

impl MaterialKey {
    /// The key of a material `name` without a parent.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            extend: false,
            name: name.into(),
            parent: None,
        }
    }

    /// The key written `text`.
    pub fn parse(text: &str) -> Self {
        let (extend, rest) = match text.strip_prefix('+') {
            Some(rest) => (true, rest),
            None => (false, text),
        };
        let (name, parent) = match rest.rsplit_once(':') {
            Some((name, parent)) => (name, Some(parent.to_owned())),
            None => (rest, None),
        };
        Self {
            extend,
            name: name.to_owned(),
            parent,
        }
    }

    /// The text before the first `:` of the key as written (a leading `+` included): what the
    /// merge of two copies matches keys on.
    pub fn base_name(&self) -> String {
        let text = self.to_string();
        base_name(&text).to_owned()
    }
}

/// The text of `key` before its first `:`.
pub(super) fn base_name(key: &str) -> &str {
    key.split_once(':').map_or(key, |(base, _)| base)
}

/// The key as written.
impl fmt::Display for MaterialKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.extend {
            f.write_str("+")?;
        }
        f.write_str(&self.name)?;
        if let Some(parent) = &self.parent {
            write!(f, ":{parent}")?;
        }
        Ok(())
    }
}

/// The three spellings of a list member: `key` (replace), `+key` (add) and `-key` (remove).
///
/// Which of them the client reads depends on the material's [`KeyRules`].
#[derive(Clone, Debug, PartialEq)]
pub struct Listed<T> {
    /// The plain key.
    pub replace: Option<Vec<T>>,
    /// The `+` key.
    pub add: Option<Vec<T>>,
    /// The `-` key.
    pub sub: Option<Vec<T>>,
}

impl<T> Default for Listed<T> {
    fn default() -> Self {
        Self {
            replace: None,
            add: None,
            sub: None,
        }
    }
}

impl<T> Listed<T> {
    /// Whether none of the three keys is written.
    pub fn is_empty(&self) -> bool {
        self.replace.is_none() && self.add.is_none() && self.sub.is_none()
    }
}

/// One face of the stencil test (`frontFace`, `backFace`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StencilFace {
    /// `stencilFunc`: a comparison function ([`COMPARISON_FUNCS`](super::COMPARISON_FUNCS));
    /// `Always` when nothing sets it.
    pub stencil_func: Option<String>,
    /// `stencilFailOp`: a stencil operation ([`STENCIL_OPS`](super::STENCIL_OPS)); `Keep` when
    /// nothing sets it.
    pub stencil_fail_op: Option<String>,
    /// `stencilDepthFailOp`; `Keep` when nothing sets it.
    pub stencil_depth_fail_op: Option<String>,
    /// `stencilPassOp`; `Keep` when nothing sets it.
    pub stencil_pass_op: Option<String>,
    /// The members the client does not read (vanilla's own `stencilPass` among them).
    pub extra: UnknownMembers,
}

/// One entry of `samplerStates`: what it sets on the sampler slot `samplerIndex`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SamplerState {
    /// `samplerIndex`: the slot, below [`SAMPLER_SLOTS`](super::SAMPLER_SLOTS). The client
    /// skips an entry without one.
    pub sampler_index: Option<u32>,
    /// `textureFilter` ([`TEXTURE_FILTERS`](super::TEXTURE_FILTERS)).
    pub texture_filter: Option<String>,
    /// `textureWrap` ([`TEXTURE_WRAPS`](super::TEXTURE_WRAPS)), applied to all three axes.
    pub texture_wrap: Option<String>,
    /// `shaderStage` ([`SHADER_STAGES`](super::SHADER_STAGES)); not read in a legacy material.
    pub shader_stage: Option<String>,
    /// `comparisonFunc` ([`COMPARISON_FUNCS`](super::COMPARISON_FUNCS)); not read in a legacy
    /// material.
    pub comparison_func: Option<String>,
    /// The members the client does not read.
    pub extra: UnknownMembers,
}

/// `stencilRefOverride` as a material writes it. The client reads the member with `asBool`,
/// which takes a boolean or a number and reads a number as "not zero"; vanilla's own
/// `ui.material` writes `0`. The spelling is kept so that the file is written back as it was
/// written; [`is_on`](Self::is_on) is what the client reads.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StencilRefOverride {
    /// Written as `true` or `false`.
    Flag(bool),
    /// Written as a number: on unless it is zero.
    Number(f64),
}

impl StencilRefOverride {
    /// Whether the override is on.
    pub fn is_on(self) -> bool {
        match self {
            Self::Flag(on) => on,
            Self::Number(number) => number != 0.0,
        }
    }
}

impl From<bool> for StencilRefOverride {
    fn from(on: bool) -> Self {
        Self::Flag(on)
    }
}

/// One entry of `vertexFields`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VertexField {
    /// `field` ([`VERTEX_FIELDS`](super::VERTEX_FIELDS)); an entry without one adds `Position`.
    pub field: Option<String>,
    /// The members the client does not read.
    pub extra: UnknownMembers,
}

/// One element of a `variants` array: an object of variant name → body. The client registers
/// each as `<material>.<variant>`; vanilla writes one variant per element so that the order is
/// kept.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Variant(pub IndexMap<String, MaterialDef>);

/// Unwinds nested variant bodies with a loop: a body may hold variants of its own to any
/// depth a caller builds.
impl Drop for Variant {
    fn drop(&mut self) {
        let mut pending = vec![std::mem::take(&mut self.0)];
        while let Some(bodies) = pending.pop() {
            for (_, mut body) in bodies {
                let lists = [
                    body.variants.replace.take(),
                    body.variants.add.take(),
                    body.variants.sub.take(),
                ];
                for mut variant in lists.into_iter().flatten().flatten() {
                    pending.push(std::mem::take(&mut variant.0));
                }
            }
        }
    }
}

/// The members of one material, and of one variant body.
///
/// Every member is optional: an absent one keeps the value inherited from the parent, or from
/// the list entry's base material for a material without a parent
/// ([`Materials::resolve`](super::Materials::resolve) applies the chain). Keyword-valued
/// members are strings checked against the tables of this module.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MaterialDef {
    /// `states`, `+states`, `-states`: render states ([`RENDER_STATES`](super::RENDER_STATES)).
    pub states: Listed<String>,
    /// `defines`, `+defines`, `-defines`: preprocessor defines, free strings (`BLUR_STEPS 4`).
    pub defines: Listed<String>,
    /// `samplerStates`, `+samplerStates`, `-samplerStates`. The client never applies the `-`
    /// key.
    pub sampler_states: Listed<SamplerState>,
    /// `vertexShader`: a path the client uses as a key (`shaders/entity.vertex`).
    pub vertex_shader: Option<String>,
    /// `fragmentShader`.
    pub fragment_shader: Option<String>,
    /// `geometryShader`. Vanilla's files write `vrGeometryShader`, which is not read.
    pub geometry_shader: Option<String>,
    /// `depthFunc`: a comparison function; `Less` when nothing sets it.
    pub depth_func: Option<String>,
    /// `frontFace`.
    pub front_face: Option<StencilFace>,
    /// `backFace`.
    pub back_face: Option<StencilFace>,
    /// `stencilRefOverride`, as it is written; off when nothing sets it.
    pub stencil_ref_override: Option<StencilRefOverride>,
    /// `stencilRef`: the client keeps the low byte, for both faces, and turns
    /// `stencilRefOverride` on.
    pub stencil_ref: Option<u32>,
    /// `stencilReadMask`; `0xFFFF_FFFF` when nothing sets it.
    pub stencil_read_mask: Option<u32>,
    /// `stencilWriteMask`; `0xFFFF_FFFF` when nothing sets it.
    pub stencil_write_mask: Option<u32>,
    /// `blendSrc`: a blend factor; `SourceAlpha` when nothing sets it.
    pub blend_src: Option<String>,
    /// `blendDst`; `OneMinusSrcAlpha` when nothing sets it.
    pub blend_dst: Option<String>,
    /// `alphaSrc`; without it `blendSrc` of the same material is used, else `One`.
    pub alpha_src: Option<String>,
    /// `alphaDst`; without it `blendDst` of the same material is used, else
    /// `OneMinusSrcAlpha`.
    pub alpha_dst: Option<String>,
    /// `vertexFields`: when written, the set is cleared and `Position` is always in it.
    pub vertex_fields: Option<Vec<VertexField>>,
    /// `depthBias`; 0 when nothing sets it.
    pub depth_bias: Option<f32>,
    /// `slopeScaledDepthBias`; 0 when nothing sets it.
    pub slope_scaled_depth_bias: Option<f32>,
    /// `msaaSupport`; `NonMSAA` when nothing sets it.
    pub msaa_support: Option<String>,
    /// `primitiveMode`; `TriangleList` when nothing sets it.
    pub primitive_mode: Option<String>,
    /// `renderTargetFormats`: an array of arrays of texture formats.
    pub render_target_formats: Option<Vec<Vec<String>>>,
    /// `isAnimatedTexture`: the client tests that the member is there and never reads its
    /// value, so `false` counts too.
    pub is_animated_texture: Option<Value>,
    /// `variants`, `+variants`, `-variants`. The client never reads the `-` key.
    pub variants: Listed<Variant>,
    /// The key rules the stack merge worked out for this material: those of the highest pack
    /// whose copy of the file names it. `None` in a file read on its own, where the file's
    /// form decides ([`MaterialFile::key_rules`]). Not a member of the file; never written.
    pub key_rules: Option<KeyRules>,
    /// Every other member, as written: unknown keys (vanilla's own `vrGeometryShader`,
    /// `depthBiasOGL`, `slopeScaledDepthBiasOGL` among them), a known key whose value has the
    /// wrong type or is `null`, and a `+` / `-` spelling of a key that has none.
    pub extra: UnknownMembers,
}
