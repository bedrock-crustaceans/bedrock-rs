//! The keyword tables of the material readers: every name the client's static maps hold, and
//! the member keys the readers look up.

// Each table holds every name the client accepts.

/// The render states of `states`, with their bits.
pub const RENDER_STATES: [(&str, u16); 11] = [
    ("DisableDepthTest", 0x1),
    ("Blending", 0x2),
    ("DisableCulling", 0x4),
    ("DisableColorWrite", 0x8),
    ("DisableAlphaWrite", 0x10),
    ("DisableDepthWrite", 0x20),
    ("StencilWrite", 0x40),
    ("InvertCulling", 0x80),
    ("EnableStencilTest", 0x100),
    ("EnableAlphaToCoverage", 0x200),
    ("Wireframe", 0x400),
];

/// The one render state the client names and ignores.
pub const IGNORED_RENDER_STATE: &str = "PolygonOffset";

/// The comparison functions of `depthFunc`, a face's `stencilFunc` and a sampler's
/// `comparisonFunc`.
pub const COMPARISON_FUNCS: [&str; 7] = [
    "Equal",
    "NotEqual",
    "Always",
    "Less",
    "Greater",
    "GreaterEqual",
    "LessEqual",
];

/// The stencil operations of a face.
pub const STENCIL_OPS: [&str; 2] = ["Keep", "Replace"];

/// The blend factors of `blendSrc`, `blendDst`, `alphaSrc` and `alphaDst`.
pub const BLEND_TARGETS: [&str; 9] = [
    "DestColor",
    "SourceColor",
    "Zero",
    "One",
    "OneMinusDestColor",
    "OneMinusSrcColor",
    "SourceAlpha",
    "DestAlpha",
    "OneMinusSrcAlpha",
];

/// The texture filters of a sampler. The JSON spelling is `PCF`.
pub const TEXTURE_FILTERS: [&str; 6] = [
    "Point",
    "Bilinear",
    "Trilinear",
    "MipMapBilinear",
    "TexelAA",
    "PCF",
];

/// The wrap modes of a sampler.
pub const TEXTURE_WRAPS: [&str; 2] = ["Repeat", "Clamp"];

/// The shader stages of a sampler.
pub const SHADER_STAGES: [&str; 3] = ["Vertex", "Pixel", "Geometry"];

/// The vertex fields of `vertexFields[].field`.
pub const VERTEX_FIELDS: [&str; 14] = [
    "Position",
    "Color",
    "Normal",
    "Tangent",
    "UV0",
    "UV1",
    "UV2",
    "PBRTextureIdx",
    "BoneId0",
    "PreviousPosition",
    "HummingbirdPosition",
    "HummingbirdColor",
    "HummingbirdAdditional",
    "Mers",
];

/// The values of `msaaSupport`.
pub const MSAA_SUPPORTS: [&str; 3] = ["NonMSAA", "MSAA", "Both"];

/// The values of `primitiveMode`.
pub const PRIMITIVE_MODES: [&str; 6] = [
    "None",
    "QuadList",
    "TriangleList",
    "TriangleStrip",
    "LineList",
    "Line",
];

/// The texture formats of `renderTargetFormats`.
pub const TEXTURE_FORMATS: [&str; 28] = [
    "UNKNOWN_TEXTURE_FORMAT",
    "R32G32B32A32_FLOAT",
    "R16G16B16A16_FLOAT",
    "R16G16B16A16_UNORM",
    "R32G32_FLOAT",
    "R10G10B10A2_UNORM",
    "R8G8B8A8_UNORM",
    "R8G8B8A8_UNORM_SRGB",
    "R16G16_UNORM",
    "R16G16_UINT",
    "D32_FLOAT",
    "R32_FLOAT",
    "R32_UINT",
    "R24G8_TYPELESS",
    "D24_UNORM_S8_UINT",
    "R24_UNORM_X8_TYPELESS",
    "D16_UNORM",
    "R8_UNORM",
    "R8_UINT",
    "A8_UNORM",
    "BC3_UNORM",
    "R5G6B5_UNORM",
    "R5G5B5A1_UNORM",
    "B8G8R8A8_UNORM",
    "B8G8R8A8_UNORM_SRGB",
    "BC7_UNORM",
    "R4G4B4A4_UNORM",
    "R8G8B8_UNORM",
];

/// The number of sampler slots of a material; a `samplerIndex` is below it.
pub const SAMPLER_SLOTS: u32 = 8;

/// The vertex-shader stems a material drawn for an actor may have: the text of `vertexShader`
/// between its last `/` and the first `.` after it.
pub const ACTOR_SHADER_STEMS: [&str; 3] = ["entity", "banner", "pattern"];

/// The misspellings of `variants` the client asserts on, each
/// also with a `+` or `-` in front.
pub(super) const VARIANT_MISSPELLINGS: [&str; 3] = ["varient", "varients", "variant"];

/// The members of a material read without a `+` / `-` form: a prefixed spelling is ignored.
pub(super) const PLAIN_KEYS: [&str; 20] = [
    "vertexShader",
    "fragmentShader",
    "geometryShader",
    "depthFunc",
    "frontFace",
    "backFace",
    "stencilRefOverride",
    "stencilRef",
    "stencilReadMask",
    "stencilWriteMask",
    "blendSrc",
    "blendDst",
    "alphaSrc",
    "alphaDst",
    "vertexFields",
    "depthBias",
    "slopeScaledDepthBias",
    "msaaSupport",
    "primitiveMode",
    "renderTargetFormats",
];

/// The presence-only member, also read without a prefixed form.
pub(super) const IS_ANIMATED_TEXTURE: &str = "isAnimatedTexture";

/// The members of a sampler entry.
pub(super) const SAMPLER_KEYS: [&str; 5] = [
    "samplerIndex",
    "textureFilter",
    "textureWrap",
    "shaderStage",
    "comparisonFunc",
];

/// The members of a stencil face.
pub(super) const FACE_KEYS: [&str; 4] = [
    "stencilFunc",
    "stencilFailOp",
    "stencilDepthFailOp",
    "stencilPassOp",
];

/// The bit of the render state `name`, when the client knows it.
pub fn render_state_bit(name: &str) -> Option<u16> {
    RENDER_STATES
        .iter()
        .find(|(state, _)| *state == name)
        .map(|(_, bit)| *bit)
}
