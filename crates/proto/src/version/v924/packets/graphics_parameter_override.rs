use bedrockrs_macros::{ProtoCodec, packet};
use vek::Vec3;

#[packet(id = 331)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct GraphicsParameterOverridePacket {
    pub values: Vec<GraphicsParameterOverrideKeyFrame>,
    #[endianness(le)]
    pub float_value: f32,
    #[endianness(le)]
    pub vec3_value: Vec3<f32>,
    pub biome_identifier: String,
    pub parameter_type: GraphicsParameterOverrideType,
    pub reset: bool,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct GraphicsParameterOverrideKeyFrame {
    #[endianness(le)]
    pub key: f32,
    #[endianness(le)]
    pub value: Vec3<f32>,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum GraphicsParameterOverrideType {
    SkyZenithColor = 0,
    SkyHorizonColor = 1,
    HorizonBlendMin = 2,
    HorizonBlendMax = 3,
    HorizonBlendStart = 4,
    HorizonBlendMieStart = 5,
    RayleighStrength = 6,
    SunMieStrength = 7,
    MoonMieStrength = 8,
    SunGlareShape = 9,
    Chlorophyll = 10,
    CDOM = 11,
    SuspendedSediment = 12,
    WavesDepth = 13,
    WavesFrequency = 14,
    WavesFrequencyScaling = 15,
    WavesSpeed = 16,
    WavesSpeedScaling = 17,
    WavesShape = 18,
    WavesOctaves = 19,
    WavesMix = 20,
    WavesPull = 21,
    WavesDirectionIncrement = 22,
    MidtonesContrast = 23,
    HighlightsContrast = 24,
    ShadowsContrast = 25,
}
